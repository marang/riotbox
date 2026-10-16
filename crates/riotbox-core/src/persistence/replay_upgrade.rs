//! Explicit metadata-only sibling copy; the replay owner alone admits history.
use std::{
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::{
    replay::{ReplayPolicyUpgradeError, upgrade_replay_policy_to_committed_order},
    session::{ReplayPolicy, SessionFile},
};

use super::exclusive_file;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
pub enum ReplayChronologyCopyDisposition {
    Upgraded,
    AlreadyCommittedOrder,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReplayChronologyCopy {
    pub input_path: PathBuf,
    pub destination_path: PathBuf,
    pub previous_policy: ReplayPolicy,
    pub replay_policy: ReplayPolicy,
    pub disposition: ReplayChronologyCopyDisposition,
    pub action_count: usize,
    pub commit_record_count: usize,
}

#[derive(Debug)]
pub enum ReplayChronologyCopyError {
    InvalidPath { path: PathBuf, reason: &'static str },
    Io { path: PathBuf, source: io::Error },
    Json(serde_json::Error),
    Upgrade(ReplayPolicyUpgradeError),
    PublicationUnsupported { path: PathBuf, source: io::Error },
}

impl fmt::Display for ReplayChronologyCopyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath { path, reason } => write!(f, "{}: {reason}", path.display()),
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Json(source) => write!(f, "invalid Session metadata: {source}"),
            Self::Upgrade(source) => source.fmt(f),
            Self::PublicationUnsupported { path, source } => write!(
                f,
                "cannot publish Session copy at {}: hard-link publication is unsupported: {source}",
                path.display()
            ),
        }?;
        write!(f, "; no Session copy was published; input was not modified")
    }
}

impl Error for ReplayChronologyCopyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } | Self::PublicationUnsupported { source, .. } => Some(source),
            Self::Json(source) => Some(source),
            Self::Upgrade(source) => Some(source),
            Self::InvalidPath { .. } => None,
        }
    }
}

impl From<serde_json::Error> for ReplayChronologyCopyError {
    fn from(source: serde_json::Error) -> Self {
        Self::Json(source)
    }
}

/// Create a new sibling Session without opening referenced material or replacing
/// any destination entry. Only an admitted legacy policy token may change.
/// Success means publication, not full Session qualification or crash durability.
pub fn copy_session_with_committed_order(
    input: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> Result<ReplayChronologyCopy, ReplayChronologyCopyError> {
    copy_with_publication(
        input.as_ref(),
        destination.as_ref(),
        exclusive_file::publish,
    )
}

fn copy_with_publication<F>(
    input: &Path,
    destination: &Path,
    publish: F,
) -> Result<ReplayChronologyCopy, ReplayChronologyCopyError>
where
    F: FnOnce(&Path, &[u8]) -> Result<(), exclusive_file::ExclusiveFileError>,
{
    let input_path = resolved_leaf(input)?;
    let destination_path = resolved_leaf(destination)?;
    if input_path.parent() != destination_path.parent() {
        return Err(invalid(
            destination,
            "destination must be a sibling in the same existing directory",
        ));
    }
    let metadata =
        fs::symlink_metadata(&input_path).map_err(|source| io_error(&input_path, source))?;
    if !metadata.file_type().is_file() {
        return Err(invalid(
            input,
            "input must be a regular file, not a symlink or special file",
        ));
    }
    match fs::symlink_metadata(&destination_path) {
        Ok(_) => {
            return Err(invalid(
                destination,
                "destination is occupied; choose a new sibling filename",
            ));
        }
        Err(source) if source.kind() == io::ErrorKind::NotFound => {}
        Err(source) => return Err(io_error(&destination_path, source)),
    }
    // Exactly one input read. The typed validator and raw-token locator both
    // borrow these bytes; referenced paths are never opened or resolved.
    let bytes = fs::read(&input_path).map_err(|source| io_error(&input_path, source))?;
    let mut session: SessionFile = serde_json::from_slice(&bytes)?;
    let previous_policy = session.action_log.replay_policy;
    upgrade_replay_policy_to_committed_order(&mut session.action_log)
        .map_err(ReplayChronologyCopyError::Upgrade)?;
    let disposition = if previous_policy == session.action_log.replay_policy {
        ReplayChronologyCopyDisposition::AlreadyCommittedOrder
    } else {
        ReplayChronologyCopyDisposition::Upgraded
    };
    let output = policy_only_bytes(&bytes, session.action_log.replay_policy, disposition)?;
    publish(&destination_path, &output).map_err(|error| match error {
        exclusive_file::ExclusiveFileError::Link(source)
            if source.kind() == io::ErrorKind::Unsupported =>
        {
            ReplayChronologyCopyError::PublicationUnsupported {
                path: destination_path.clone(),
                source,
            }
        }
        other => io_error(&destination_path, other.into_io()),
    })?;
    Ok(ReplayChronologyCopy {
        input_path,
        destination_path,
        previous_policy,
        replay_policy: session.action_log.replay_policy,
        disposition,
        action_count: session.action_log.actions.len(),
        commit_record_count: session.action_log.commit_records.len(),
    })
}

// Borrowed raw JSON is an I/O locator, not another Session schema. Serde's
// existing typed Session decoder owns duplicate known-field/type admission.
#[derive(Deserialize)]
struct SessionToken<'a> {
    #[serde(borrow)]
    action_log: &'a RawValue,
}
#[derive(Deserialize)]
struct PolicyToken<'a> {
    #[serde(borrow)]
    replay_policy: &'a RawValue,
}

fn policy_only_bytes(
    bytes: &[u8],
    policy: ReplayPolicy,
    disposition: ReplayChronologyCopyDisposition,
) -> Result<Vec<u8>, serde_json::Error> {
    if disposition == ReplayChronologyCopyDisposition::AlreadyCommittedOrder {
        return Ok(bytes.to_vec());
    }
    let root: SessionToken<'_> = serde_json::from_slice(bytes)?;
    let log: PolicyToken<'_> = serde_json::from_str(root.action_log.get())?;
    let token = log.replay_policy.get().as_bytes();
    // Both RawValues borrow the original slice, including escaped keys/values.
    // This is safe pointer-address arithmetic only; no dereference or unsafe.
    let offset = token.as_ptr() as usize - bytes.as_ptr() as usize;
    let replacement = serde_json::to_vec(&policy)?;
    let mut output = Vec::with_capacity(bytes.len() - token.len() + replacement.len());
    output.extend_from_slice(&bytes[..offset]);
    output.extend_from_slice(&replacement);
    output.extend_from_slice(&bytes[offset + token.len()..]);
    Ok(output)
}

fn resolved_leaf(path: &Path) -> Result<PathBuf, ReplayChronologyCopyError> {
    let leaf = path
        .as_os_str()
        .as_encoded_bytes()
        .rsplit(|byte| std::path::is_separator(char::from(*byte)))
        .next()
        .unwrap_or_default();
    if leaf.is_empty() || leaf == b"." || leaf == b".." {
        return Err(invalid(path, "an ordinary explicit filename is required"));
    }
    let filename = path
        .file_name()
        .ok_or_else(|| invalid(path, "an ordinary explicit filename is required"))?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = fs::canonicalize(parent).map_err(|source| io_error(path, source))?;
    let resolved = parent.join(filename);
    if [path, resolved.as_path()].into_iter().any(|candidate| {
        candidate.components().any(|component| {
            component
                .as_os_str()
                .to_string_lossy()
                .ends_with(".riotbox-graphs")
        })
    }) {
        return Err(invalid(
            path,
            "immutable .riotbox-graphs storage is reserved",
        ));
    }
    Ok(resolved)
}

fn invalid(path: &Path, reason: &'static str) -> ReplayChronologyCopyError {
    ReplayChronologyCopyError::InvalidPath {
        path: path.to_path_buf(),
        reason,
    }
}
fn io_error(path: &Path, source: io::Error) -> ReplayChronologyCopyError {
    ReplayChronologyCopyError::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests;
