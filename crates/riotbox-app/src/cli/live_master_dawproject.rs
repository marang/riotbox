use crate::cli::model::AppLaunch;
use crate::cli::model::LaunchMode;
use crate::cli::observer::UserSessionObserver;
use crate::cli::observer::launch_summary;
use crate::cli::observer::timestamp_now;
use crate::jam_app::JamAppError;
use crate::jam_app::JamAppState;
use crate::observer::observer_snapshot;
use crate::ui::JamShellState;
use crate::ui::ShellLaunchMode;
use riotbox_core::action::Action;
use riotbox_core::action::ActionCommand;
use riotbox_core::action::ActionParams;
use riotbox_core::action::DawSessionExportBoundary;
use riotbox_core::session::ExportReceiptState;
use serde_json::Value;
use serde_json::json;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

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
    let history_start = state.queue.history().len();
    let result =
        state.commit_and_save_live_master_dawproject_export(destination_path, timestamp_now());
    let summary = export_result_summary(
        &state,
        session_path,
        destination_path,
        history_start,
        result,
    );
    Ok((summary, JamShellState::new(state, ShellLaunchMode::Load)))
}

fn export_result_summary(
    state: &JamAppState,
    session_path: &Path,
    destination_path: &Path,
    history_start: usize,
    result: Result<ExportReceiptState, JamAppError>,
) -> Value {
    match result {
        Ok(receipt) => json!({
            "mode": "live_master_dawproject_execute", "status": "ready", "ready": true,
            "writes_files": true, "mutates_session": true, "observer_events": false,
            "boundary": state.queue.history_action(receipt.created_by_action)
                .and_then(dawproject_action_boundary),
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
            "boundary": state.queue.history()[history_start..].iter().rev()
                .find(|action| matches!(&action.params,
                    ActionParams::DawSessionExport { destination_path: Some(path), .. }
                        if Path::new(path) == destination_path))
                .and_then(dawproject_action_boundary),
            "receipt_boundary": null,
            "session_path": session_path, "destination_path": destination_path,
            "readiness_blockers": [error.to_string()], "receipt": null,
            "scope_note": "no DAWproject archive or DAW Session receipt was committed",
        }),
    }
}

fn dawproject_action_boundary(action: &Action) -> Option<DawSessionExportBoundary> {
    match &action.params {
        ActionParams::DawSessionExport {
            boundary:
                boundary @ (DawSessionExportBoundary::LiveMasterDawprojectV1
                | DawSessionExportBoundary::LiveMasterDawprojectV2),
            receipt_id: Some(_),
            ..
        } if action.command == ActionCommand::ExportDawSession => Some(*boundary),
        _ => None,
    }
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
    use riotbox_core::action::ActionDraft;
    use riotbox_core::action::ActionTarget;
    use riotbox_core::action::ActorType;
    use riotbox_core::action::CommitBoundary;
    use riotbox_core::action::Quantization;
    use riotbox_core::export_readiness::ExportReadinessContract;
    use riotbox_core::export_readiness::ExportReadinessStatus;
    use riotbox_core::export_readiness::ExportScope;
    use riotbox_core::export_readiness::ProductExportBoundary;
    use riotbox_core::export_readiness::ProductExportDestinationKind;
    use riotbox_core::export_readiness::ProductExportRole;
    use riotbox_core::queue::ActionQueue;
    use riotbox_core::transport::CommitBoundaryState;
    use riotbox_core::{persistence::save_session_json, session::SessionFile};

    #[test]
    fn successful_summary_uses_receipts_own_action_version_not_latest_action() {
        let mut queue = ActionQueue::new();
        let v1_id = queue.enqueue(
            daw_draft(DawSessionExportBoundary::LiveMasterDawprojectV1),
            10,
        );
        let commit_boundary = CommitBoundaryState {
            kind: CommitBoundary::Immediate,
            beat_index: 0,
            bar_index: 0,
            phrase_index: 0,
            scene_id: None,
        };
        queue
            .commit_pending_after_side_effect(v1_id, commit_boundary.clone(), 11, "fixture")
            .expect("committed V1 Action projection fixture");
        let v2_id = queue.enqueue(
            daw_draft(DawSessionExportBoundary::LiveMasterDawprojectV2),
            12,
        );
        queue
            .commit_pending_after_side_effect(v2_id, commit_boundary, 13, "fixture")
            .expect("committed V2 Action projection fixture");
        let state =
            JamAppState::from_parts(SessionFile::new("summary", "test", "now"), None, queue);
        for (action_id, boundary, receipt_boundary) in [
            (
                v1_id,
                "live_master_dawproject_v1",
                ProductExportBoundary::DawSessionLiveMasterDawprojectV1,
            ),
            (
                v2_id,
                "live_master_dawproject_v2",
                ProductExportBoundary::DawSessionLiveMasterDawprojectV2,
            ),
        ] {
            let contract = ExportReadinessContract {
                schema: "riotbox.export_readiness.v1".into(),
                status: ExportReadinessStatus::Reproducible,
                proof_schema: "generated-metadata-projection".into(),
                export_scope: ExportScope::DawSession,
                boundary: receipt_boundary,
                pack_id: "generated-metadata-projection".into(),
                export_role: ProductExportRole::ArrangementManifest,
                export_artifact: "live.dawproject".into(),
                source_sha256: "11".repeat(32),
                export_sha256: "22".repeat(32),
                normalized_manifest_sha256: "33".repeat(32),
                unsupported_scopes: vec![],
            };
            let receipt = ExportReceiptState::from_readiness_contract(
                action_id,
                13,
                &contract,
                "live.dawproject",
                "proof.json",
                None,
            );
            let summary = export_result_summary(
                &state,
                Path::new("session.json"),
                Path::new("live.dawproject"),
                0,
                Ok(receipt),
            );
            assert_eq!(summary["boundary"], boundary);
            assert_eq!(summary["receipt_boundary"], receipt_boundary.as_proof_str());
        }
    }

    #[test]
    fn failed_summary_reports_only_this_attempts_selected_boundary_without_a_receipt() {
        let mut queue = ActionQueue::new();
        for boundary in [
            DawSessionExportBoundary::LiveMasterDawprojectV1,
            DawSessionExportBoundary::LiveMasterDawprojectV2,
        ] {
            let id = queue.enqueue(daw_draft(boundary), 10);
            queue.reject(id, "synthetic preflight failure");
        }
        let state =
            JamAppState::from_parts(SessionFile::new("summary", "test", "now"), None, queue);
        for (history_start, expected) in [(1, json!("live_master_dawproject_v2")), (2, Value::Null)]
        {
            let summary = export_result_summary(
                &state,
                Path::new("session.json"),
                Path::new("live.dawproject"),
                history_start,
                Err(JamAppError::InvalidSession(
                    "synthetic preflight failure".into(),
                )),
            );
            assert_eq!(summary["status"], "blocked");
            assert_eq!(summary["boundary"], expected);
            assert_eq!(summary["receipt_boundary"], Value::Null);
            assert_eq!(summary["receipt"], Value::Null);
        }
    }

    fn daw_draft(boundary: DawSessionExportBoundary) -> ActionDraft {
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::ExportDawSession,
            Quantization::Immediate,
            ActionTarget::default(),
        );
        draft.params = ActionParams::DawSessionExport {
            export_scope: ExportScope::DawSession,
            boundary,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalFilePath,
            destination_path: Some("live.dawproject".into()),
            receipt_id: Some("source-recording".into()),
        };
        draft
    }

    #[test]
    fn metadata_only_cli_fails_closed_without_a_supported_recording_receipt() {
        let launch = crate::cli::args::parse_args([
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
        assert!(launch_summary(&launch).get("boundary").is_none());

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
        assert_eq!(summary["boundary"], Value::Null);
        assert_eq!(summary["receipt_boundary"], Value::Null);
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
