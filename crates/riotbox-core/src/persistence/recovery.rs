use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::session::SessionFile;

use super::PersistenceError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionRecoveryCandidateKind {
    CanonicalTarget,
    OrphanTemp,
    Autosave,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionRecoveryCandidateStatus {
    Missing,
    ParseableSession,
    InvalidSessionJson { reason: String },
    Unreadable { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionRecoveryCandidate {
    pub kind: SessionRecoveryCandidateKind,
    pub path: PathBuf,
    pub status: SessionRecoveryCandidateStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionRecoveryScan {
    pub target_path: PathBuf,
    pub candidates: Vec<SessionRecoveryCandidate>,
}

pub fn scan_session_recovery_candidates(
    target_path: impl AsRef<Path>,
) -> Result<SessionRecoveryScan, PersistenceError> {
    let target_path = target_path.as_ref().to_path_buf();
    let mut candidates = vec![candidate(
        SessionRecoveryCandidateKind::CanonicalTarget,
        target_path.clone(),
    )];

    let mut sibling_candidates = scan_sibling_recovery_candidates(&target_path)?;
    candidates.append(&mut sibling_candidates);

    Ok(SessionRecoveryScan {
        target_path,
        candidates,
    })
}

fn scan_sibling_recovery_candidates(
    target_path: &Path,
) -> Result<Vec<SessionRecoveryCandidate>, PersistenceError> {
    let Some(parent) = parent_dir(target_path) else {
        return Ok(Vec::new());
    };
    if !parent.exists() {
        return Ok(Vec::new());
    }

    let temp_prefix = temp_file_prefix(target_path);
    let autosave_matcher = AutosaveMatcher::new(target_path);
    let mut candidates = Vec::new();

    for entry in fs::read_dir(parent)? {
        let entry = entry?;
        let path = entry.path();
        if path == target_path {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        let kind = if file_name.starts_with(&temp_prefix) {
            Some(SessionRecoveryCandidateKind::OrphanTemp)
        } else if autosave_matcher.matches(file_name) {
            Some(SessionRecoveryCandidateKind::Autosave)
        } else {
            None
        };

        if let Some(kind) = kind {
            candidates.push(candidate(kind, path));
        }
    }

    candidates.sort_by(|left, right| {
        candidate_kind_rank(&left.kind)
            .cmp(&candidate_kind_rank(&right.kind))
            .then_with(|| left.path.cmp(&right.path))
    });
    Ok(candidates)
}

fn candidate(kind: SessionRecoveryCandidateKind, path: PathBuf) -> SessionRecoveryCandidate {
    let status = candidate_status(&path);
    SessionRecoveryCandidate { kind, path, status }
}

fn candidate_status(path: &Path) -> SessionRecoveryCandidateStatus {
    if !path.exists() {
        return SessionRecoveryCandidateStatus::Missing;
    }

    match fs::read_to_string(path) {
        Ok(json) => match serde_json::from_str::<SessionFile>(&json) {
            Ok(_) => SessionRecoveryCandidateStatus::ParseableSession,
            Err(error) => SessionRecoveryCandidateStatus::InvalidSessionJson {
                reason: error.to_string(),
            },
        },
        Err(error) => SessionRecoveryCandidateStatus::Unreadable {
            reason: error.to_string(),
        },
    }
}

fn parent_dir(path: &Path) -> Option<&Path> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
}

fn temp_file_prefix(path: &Path) -> String {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("riotbox.json");
    format!(".{file_name}.tmp-")
}

fn candidate_kind_rank(kind: &SessionRecoveryCandidateKind) -> u8 {
    match kind {
        SessionRecoveryCandidateKind::CanonicalTarget => 0,
        SessionRecoveryCandidateKind::OrphanTemp => 1,
        SessionRecoveryCandidateKind::Autosave => 2,
    }
}

struct AutosaveMatcher {
    prefix: String,
    suffix: String,
}

impl AutosaveMatcher {
    fn new(path: &Path) -> Self {
        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("riotbox");
        let suffix = path
            .extension()
            .and_then(|value| value.to_str())
            .map_or_else(String::new, |extension| format!(".{extension}"));

        Self {
            prefix: format!("{stem}.autosave"),
            suffix,
        }
    }

    fn matches(&self, file_name: &str) -> bool {
        let Some(rest) = file_name.strip_prefix(&self.prefix) else {
            return false;
        };

        if self.suffix.is_empty() {
            return rest.is_empty() || rest.starts_with('.');
        }

        rest == self.suffix || (rest.starts_with('.') && rest.ends_with(&self.suffix))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    fn session_json(session_id: &str) -> String {
        serde_json::to_string_pretty(&SessionFile::new(
            session_id,
            "riotbox-test",
            "2026-04-29T20:40:00Z",
        ))
        .expect("serialize session")
    }

    #[test]
    fn scanner_reports_target_temp_and_autosave_candidates_in_stable_order() {
        let dir = tempdir().expect("create temp dir");
        let target_path = dir.path().join("session.json");
        let temp_path = dir.path().join(".session.json.tmp-42-100");
        let autosave_path = dir.path().join("session.autosave.2026-04-29T204000Z.json");
        let autosave_latest_path = dir.path().join("session.autosave.json");
        let ignored_autosave_like_path = dir.path().join("session.autosave-draft.json");
        let ignored_path = dir.path().join("session.backup.json");

        fs::write(&target_path, session_json("canonical")).expect("write target");
        fs::write(&temp_path, "{ truncated").expect("write temp");
        fs::write(&autosave_path, session_json("autosave-old")).expect("write autosave");
        fs::write(&autosave_latest_path, "not json").expect("write latest autosave");
        fs::write(&ignored_autosave_like_path, session_json("ignored-draft"))
            .expect("write ignored autosave-like file");
        fs::write(&ignored_path, session_json("ignored")).expect("write ignored backup");

        let scan = scan_session_recovery_candidates(&target_path).expect("scan candidates");

        assert_eq!(scan.target_path, target_path);
        assert_eq!(scan.candidates.len(), 4);
        assert_eq!(
            scan.candidates
                .iter()
                .map(|candidate| &candidate.kind)
                .collect::<Vec<_>>(),
            vec![
                &SessionRecoveryCandidateKind::CanonicalTarget,
                &SessionRecoveryCandidateKind::OrphanTemp,
                &SessionRecoveryCandidateKind::Autosave,
                &SessionRecoveryCandidateKind::Autosave,
            ]
        );
        assert_eq!(
            scan.candidates
                .iter()
                .map(|candidate| candidate.path.file_name().unwrap().to_str().unwrap())
                .collect::<Vec<_>>(),
            vec![
                "session.json",
                ".session.json.tmp-42-100",
                "session.autosave.2026-04-29T204000Z.json",
                "session.autosave.json",
            ]
        );
        assert_eq!(
            scan.candidates[0].status,
            SessionRecoveryCandidateStatus::ParseableSession
        );
        assert!(matches!(
            scan.candidates[1].status,
            SessionRecoveryCandidateStatus::InvalidSessionJson { .. }
        ));
        assert_eq!(
            scan.candidates[2].status,
            SessionRecoveryCandidateStatus::ParseableSession
        );
        assert!(matches!(
            scan.candidates[3].status,
            SessionRecoveryCandidateStatus::InvalidSessionJson { .. }
        ));
        assert!(ignored_path.exists());
        assert!(ignored_autosave_like_path.exists());
        assert!(temp_path.exists());
        assert!(autosave_path.exists());
    }

    #[test]
    fn scanner_reports_missing_target_without_requiring_parent_directory() {
        let dir = tempdir().expect("create temp dir");
        let target_path = dir.path().join("missing").join("session.json");

        let scan = scan_session_recovery_candidates(&target_path).expect("scan candidates");

        assert_eq!(scan.target_path, target_path);
        assert_eq!(
            scan.candidates,
            vec![SessionRecoveryCandidate {
                kind: SessionRecoveryCandidateKind::CanonicalTarget,
                path: scan.target_path.clone(),
                status: SessionRecoveryCandidateStatus::Missing,
            }]
        );
    }
}
