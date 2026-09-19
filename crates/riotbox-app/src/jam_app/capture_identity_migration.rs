//! Explicit offline metadata migration. This is not a performer action or replay event.
use super::{
    JamAppError,
    capture_artifacts::preflight_capture_artifact_hydration,
    capture_identity::{identity, read_wav},
};
use riotbox_core::{
    ids::CaptureId, persistence::save_session_json, session::CaptureAudioIdentityProvenance,
};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug, Serialize)]
pub struct CaptureIdentityMigrationEntry {
    pub capture_id: CaptureId,
    pub path: PathBuf,
    pub sha256: String,
    pub adopted: bool,
}

/// Reads only the selected captures and Session metadata. With `accept_current`
/// false this is a read-only preview. All selected inputs must pass before save.
pub fn migrate_legacy_capture_identities(
    session_path: &Path,
    capture_ids: &[CaptureId],
    accept_current: bool,
) -> Result<Vec<CaptureIdentityMigrationEntry>, JamAppError> {
    let invalid = |message: String| JamAppError::InvalidSession(message);
    if capture_ids.is_empty() {
        return Err(invalid("select at least one capture ID".into()));
    }
    if capture_ids.iter().collect::<BTreeSet<_>>().len() != capture_ids.len() {
        return Err(invalid("duplicate selected capture ID".into()));
    }
    super::persistence::graph_transaction::validate_mutable_destination(session_path)?;
    let original = std::fs::read(session_path)?;
    let mut session: riotbox_core::session::SessionFile = serde_json::from_slice(&original)?;
    let adopted_at = format!(
        "unix_ms:{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );
    let mut entries = Vec::new();
    let mut changes = false;
    for id in capture_ids {
        let indices: Vec<_> = session
            .captures
            .iter()
            .enumerate()
            .filter(|(_, c)| c.capture_id == *id)
            .map(|(i, _)| i)
            .collect();
        let [index] = indices.as_slice() else {
            return Err(invalid(format!(
                "capture {id} must identify exactly one Session capture"
            )));
        };
        let capture = &mut session.captures[*index];
        let path = preflight_capture_artifact_hydration(capture, session_path.parent())
            .map_err(|error| invalid(format!("{error:?}")))?;
        let (bytes, _) = read_wav(&path).map_err(invalid)?;
        let adopted_identity = identity(
            &bytes,
            CaptureAudioIdentityProvenance::AdoptedLegacyV1 {
                adopted_at: adopted_at.clone(),
            },
        );
        let missing = capture.audio_identity.is_none();
        if let Some(expected) = &capture.audio_identity {
            if !expected.is_valid() || expected.sha256 != adopted_identity.sha256 {
                return Err(invalid(format!(
                    "capture {id} already has an identity that does not match; migration cannot replace it"
                )));
            }
        } else if accept_current {
            capture.audio_identity = Some(adopted_identity.clone());
            changes = true;
        }
        entries.push(CaptureIdentityMigrationEntry {
            capture_id: id.clone(),
            path,
            sha256: adopted_identity.sha256,
            adopted: missing && accept_current,
        });
    }
    if changes {
        // Detect an intervening edit; the existing single-writer save contract
        // still applies. Never write WAVs or Source Graphs during migration.
        if std::fs::read(session_path)? != original {
            return Err(invalid("Session changed during capture migration".into()));
        }
        save_session_json(session_path, &session)?;
    }
    Ok(entries)
}
