use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde_json::{Value, json};

use crate::{
    jam_app::JamAppState,
    observer::observer_snapshot,
    ui::{JamShellState, ShellLaunchMode},
};

use super::{AppLaunch, LaunchMode, UserSessionObserver, launch_summary, timestamp_now};

pub(super) fn run_live_master_dawproject_execute(
    launch: &AppLaunch,
    raw_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut observer = open_observer(launch)?;
    let (mut summary, shell) = live_master_dawproject_execute_summary(launch)?;
    let observer_result = match observer.as_mut() {
        Some(observer) => observer.record(json!({
            "event": "live_master_dawproject_execute",
            "schema": "riotbox.user_session_observer.v1",
            "timestamp_ms": timestamp_now(),
            "opt_in": true,
            "capture_context": "non_interactive_metadata_only_cli",
            "raw_audio_recording": false,
            "realtime_callback_io": false,
            "daw_host_launch": false,
            "argv": raw_args,
            "launch": launch_summary(launch),
            "summary": summary.clone(),
            "snapshot": observer_snapshot(&shell),
        })),
        None => Ok(()),
    };
    apply_observer_status(&mut summary, observer.is_some(), &observer_result);
    if let Err(error) = observer_result {
        eprintln!(
            "live-master DAWproject completed, but the optional observer write failed: {error}"
        );
    }
    serde_json::to_writer_pretty(std::io::stdout(), &summary)?;
    println!();
    Ok(())
}

fn live_master_dawproject_execute_summary(
    launch: &AppLaunch,
) -> Result<(Value, JamShellState), Box<dyn std::error::Error>> {
    let LaunchMode::LiveMasterDawprojectExecute {
        session_path,
        destination_path,
    } = &launch.mode
    else {
        return Err("not a live-master DAWproject execute launch".into());
    };
    let mut state = JamAppState::from_json_files_for_export_metadata(session_path)?;
    let summary = match state
        .commit_and_save_live_master_dawproject_export(destination_path, timestamp_now())
    {
        Ok(receipt) => json!({
            "mode": "live_master_dawproject_execute", "status": "ready", "ready": true,
            "writes_files": true, "mutates_session": true, "observer_events": false,
            "boundary": crate::jam_app::LIVE_MASTER_DAWPROJECT_ACTION_BOUNDARY_ID,
            "receipt_boundary": receipt.export_boundary.as_proof_str(),
            "session_path": session_path, "destination_path": destination_path,
            "readiness_blockers": [],
            "receipt": { "receipt_id": receipt.receipt_id, "pack_id": receipt.pack_id,
                "export_scope": receipt.export_scope, "export_role": receipt.export_role,
                "sha256": receipt.export_hash, "project_xml_sha256": receipt.normalized_manifest_hash,
                "artifact_count": receipt.artifact_set.len(), "qa_gates": receipt.qa_gates },
            "scope_note": "archive readback and byte-identical recorded audio are proven; DAW host import, audible DAW output, release, and quality remain unproven",
        }),
        Err(error) => json!({
            "mode": "live_master_dawproject_execute", "status": "blocked", "ready": false,
            "writes_files": false, "mutates_session": false, "observer_events": false,
            "boundary": crate::jam_app::LIVE_MASTER_DAWPROJECT_ACTION_BOUNDARY_ID,
            "receipt_boundary": "daw_session.live_master_dawproject_v1",
            "session_path": session_path, "destination_path": destination_path,
            "readiness_blockers": [error.to_string()], "receipt": null,
            "scope_note": "no DAWproject archive or DAW Session receipt was committed",
        }),
    };
    Ok((summary, JamShellState::new(state, ShellLaunchMode::Load)))
}

fn open_observer(
    launch: &AppLaunch,
) -> Result<Option<UserSessionObserver>, Box<dyn std::error::Error>> {
    let Some(observer_path) = launch.observer_path.as_deref() else {
        return Ok(None);
    };
    let LaunchMode::LiveMasterDawprojectExecute {
        session_path,
        destination_path,
    } = &launch.mode
    else {
        return Err("live-master DAWproject observer requires export mode".into());
    };
    validate_observer_path(observer_path, session_path, destination_path)?;
    Ok(Some(UserSessionObserver::open_new(observer_path)?))
}

fn validate_observer_path(
    observer: &Path,
    session: &Path,
    destination: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let session = fs::canonicalize(session)?;
    let destination = canonical_future_path(destination)?;
    let observer_identity = canonical_future_path(observer)?;
    if observer_identity == session || observer_identity == destination {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "live-master DAWproject observer path aliases the Session or archive destination",
        )
        .into());
    }
    if fs::symlink_metadata(observer).is_ok() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "live-master DAWproject observer path must be fresh",
        )
        .into());
    }
    Ok(())
}

fn canonical_future_path(path: &Path) -> io::Result<PathBuf> {
    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "live-master DAWproject path requires a file name",
        )
    })?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    Ok(fs::canonicalize(parent)?.join(file_name))
}

fn apply_observer_status(summary: &mut Value, requested: bool, result: &io::Result<()>) {
    match (requested, result) {
        (true, Ok(())) => {
            summary["observer_events"] = Value::Bool(true);
            summary["observer_status"] = Value::String("recorded".into());
        }
        (true, Err(error)) => {
            summary["observer_events"] = Value::Bool(false);
            summary["observer_status"] = Value::String("write_failed".into());
            summary["observer_failure_reason"] = Value::String(error.to_string());
        }
        (false, _) => {
            summary["observer_events"] = Value::Bool(false);
            summary["observer_status"] = Value::String("not_requested".into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riotbox_core::{persistence::save_session_json, session::SessionFile};

    #[test]
    fn cli_parses_metadata_only_live_master_dawproject_mode_and_fails_closed_without_a_v2_receipt()
    {
        let launch = super::super::parse_args([
            "--live-master-dawproject-execute".into(),
            "--session".into(),
            "session.json".into(),
            "--daw-session-destination".into(),
            "exports/live.dawproject".into(),
        ])
        .expect("parse live-master DAWproject mode");
        assert!(matches!(
            launch.mode,
            LaunchMode::LiveMasterDawprojectExecute { .. }
        ));

        let dir = tempfile::tempdir().expect("tempdir");
        let session_path = dir.path().join("session.json");
        let destination = dir.path().join("blocked.dawproject");
        save_session_json(
            &session_path,
            &SessionFile::new("blocked", "riotbox-test", "2026-09-08T00:00:00Z"),
        )
        .expect("write Session");
        let blocked = AppLaunch {
            mode: LaunchMode::LiveMasterDawprojectExecute {
                session_path,
                destination_path: destination.clone(),
            },
            observer_path: None,
        };
        let (summary, _) =
            live_master_dawproject_execute_summary(&blocked).expect("blocked summary");
        assert_eq!(summary["status"], "blocked");
        assert!(!destination.exists());
    }

    #[test]
    fn observer_path_is_fresh_and_cannot_alias_session_or_archive() {
        let dir = tempfile::tempdir().expect("tempdir");
        let session = dir.path().join("session.json");
        let archive = dir.path().join("live.dawproject");
        save_session_json(
            &session,
            &SessionFile::new("observer", "riotbox-test", "2026-09-08T00:00:00Z"),
        )
        .expect("write Session");
        assert!(validate_observer_path(&session, &session, &archive).is_err());
        let observer = dir.path().join("observer.ndjson");
        fs::write(&observer, b"existing").expect("existing observer");
        assert!(validate_observer_path(&observer, &session, &archive).is_err());
    }

    #[test]
    fn post_commit_observer_failure_is_reported_without_reclassifying_the_export() {
        let mut summary = json!({"status": "ready", "ready": true});
        let failure = Err(io::Error::other("synthetic observer failure"));
        apply_observer_status(&mut summary, true, &failure);
        assert_eq!(summary["status"], "ready");
        assert_eq!(summary["observer_status"], "write_failed");
        assert_eq!(summary["observer_events"], false);
    }
}
