fn run_daw_session_writer_export_execute(
    launch: &AppLaunch,
    raw_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let (summary, shell) = daw_session_writer_export_execute_summary(launch)?;
    if let Some(path) = launch.observer_path.as_deref() {
        let mut observer = UserSessionObserver::open(path)?;
        observer.record(json!({
            "event": "daw_session_writer_export_execute",
            "schema": "riotbox.user_session_observer.v1",
            "timestamp_ms": timestamp_now(),
            "opt_in": true,
            "capture_context": "non_interactive_cli",
            "raw_audio_recording": false,
            "realtime_callback_io": false,
            "argv": raw_args,
            "launch": launch_summary(launch),
            "summary": summary.clone(),
            "snapshot": observer_snapshot(&shell),
        }))?;
    }
    serde_json::to_writer_pretty(std::io::stdout(), &summary)?;
    println!();
    Ok(())
}

fn run_daw_session_host_import_proof_export_execute(
    launch: &AppLaunch,
    raw_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let (summary, shell) = daw_session_host_import_proof_export_execute_summary(launch)?;
    if let Some(path) = launch.observer_path.as_deref() {
        let mut observer = UserSessionObserver::open(path)?;
        observer.record(json!({
            "event": "daw_session_host_import_proof_export_execute",
            "schema": "riotbox.user_session_observer.v1",
            "timestamp_ms": timestamp_now(),
            "opt_in": true,
            "capture_context": "non_interactive_cli",
            "raw_audio_recording": false,
            "realtime_callback_io": false,
            "argv": raw_args,
            "launch": launch_summary(launch),
            "summary": summary.clone(),
            "snapshot": observer_snapshot(&shell),
        }))?;
    }
    serde_json::to_writer_pretty(std::io::stdout(), &summary)?;
    println!();
    Ok(())
}

fn daw_session_writer_export_execute_summary(
    launch: &AppLaunch,
) -> Result<(Value, JamShellState), Box<dyn std::error::Error>> {
    let LaunchMode::DawSessionWriterExportExecute {
        session_path,
        destination_path,
    } = &launch.mode
    else {
        return Err("not a DAW session writer export execute launch".into());
    };

    let mut state = JamAppState::from_json_files(session_path, None::<&Path>)?;
    let summary = match state.commit_daw_session_writer_export(
        session_path.parent(),
        destination_path,
        timestamp_now(),
    ) {
        Ok(receipt) => {
            state.save()?;
            daw_session_writer_export_execute_ready_summary(
                session_path,
                destination_path,
                &state,
                &receipt,
                launch.observer_path.is_some(),
            )
        }
        Err(error) => daw_session_writer_export_execute_blocked_summary(
            session_path,
            destination_path,
            &state,
            &error,
            launch.observer_path.is_some(),
        ),
    };
    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    Ok((summary, shell))
}

fn daw_session_host_import_proof_export_execute_summary(
    launch: &AppLaunch,
) -> Result<(Value, JamShellState), Box<dyn std::error::Error>> {
    let LaunchMode::DawSessionHostImportProofExportExecute {
        session_path,
        proof_path,
    } = &launch.mode
    else {
        return Err("not a DAW session host import proof export execute launch".into());
    };

    let mut state = JamAppState::from_json_files(session_path, None::<&Path>)?;
    let summary = match state.commit_daw_session_host_import_proof_export(
        proof_path,
        timestamp_now(),
    ) {
        Ok(receipt) => {
            state.save()?;
            daw_session_host_import_proof_export_execute_ready_summary(
                session_path,
                proof_path,
                &state,
                &receipt,
                launch.observer_path.is_some(),
            )
        }
        Err(error) => daw_session_host_import_proof_export_execute_blocked_summary(
            session_path,
            proof_path,
            &state,
            &error,
            launch.observer_path.is_some(),
        ),
    };
    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    Ok((summary, shell))
}

fn daw_session_writer_export_execute_ready_summary(
    session_path: &Path,
    destination_path: &Path,
    state: &JamAppState,
    receipt: &riotbox_core::session::ExportReceiptState,
    observer_events: bool,
) -> Value {
    let surface_gate = state.daw_session_export_surface_gate();
    json!({
        "mode": "daw_session_writer_export_execute",
        "status": "ready",
        "ready": true,
        "writes_files": true,
        "mutates_session": true,
        "observer_events": observer_events,
        "boundary": crate::jam_app::DAW_SESSION_LOCAL_PROJECT_WRITER_BOUNDARY_ID,
        "session_path": session_path,
        "destination_path": destination_path,
        "readiness_blockers": [],
        "receipt": daw_session_writer_proof_receipt_summary(receipt),
        "action": latest_daw_session_export_action_summary(
            state,
            riotbox_core::action::DawSessionExportBoundary::LocalProjectWriterV1,
        ),
        "commit_records": latest_daw_session_export_commit_records(
            state,
            riotbox_core::action::DawSessionExportBoundary::LocalProjectWriterV1,
        ),
        "daw_session_surface_gate": daw_session_surface_gate_summary(&surface_gate),
        "scope_note": "local writer proof only; host import and audible output remain unproven",
    })
}

fn daw_session_host_import_proof_export_execute_ready_summary(
    session_path: &Path,
    proof_path: &Path,
    state: &JamAppState,
    receipt: &riotbox_core::session::ExportReceiptState,
    observer_events: bool,
) -> Value {
    let surface_gate = state.daw_session_export_surface_gate();
    json!({
        "mode": "daw_session_host_import_proof_export_execute",
        "status": "ready",
        "ready": true,
        "writes_files": false,
        "mutates_session": true,
        "observer_events": observer_events,
        "boundary": "host_import_proof_v1",
        "session_path": session_path,
        "proof_path": proof_path,
        "readiness_blockers": [],
        "receipt": daw_session_host_import_receipt_summary(receipt),
        "action": latest_daw_session_export_action_summary(
            state,
            riotbox_core::action::DawSessionExportBoundary::HostImportProofV1,
        ),
        "commit_records": latest_daw_session_export_commit_records(
            state,
            riotbox_core::action::DawSessionExportBoundary::HostImportProofV1,
        ),
        "daw_session_surface_gate": daw_session_surface_gate_summary(&surface_gate),
        "scope_note": "host-import proof only; audible output remains unproven",
    })
}

fn daw_session_writer_export_execute_blocked_summary(
    session_path: &Path,
    destination_path: &Path,
    state: &JamAppState,
    error: &crate::jam_app::JamAppError,
    observer_events: bool,
) -> Value {
    let surface_gate = state.daw_session_export_surface_gate();
    json!({
        "mode": "daw_session_writer_export_execute",
        "status": "blocked",
        "ready": false,
        "writes_files": false,
        "mutates_session": false,
        "observer_events": observer_events,
        "boundary": crate::jam_app::DAW_SESSION_LOCAL_PROJECT_WRITER_BOUNDARY_ID,
        "session_path": session_path,
        "destination_path": destination_path,
        "readiness_blockers": [error.to_string()],
        "receipt": null,
        "action": latest_daw_session_export_action_summary(
            state,
            riotbox_core::action::DawSessionExportBoundary::LocalProjectWriterV1,
        ),
        "commit_records": [],
        "daw_session_surface_gate": daw_session_surface_gate_summary(&surface_gate),
        "scope_note": "local writer proof only; host import and audible output remain unproven",
    })
}

fn daw_session_host_import_proof_export_execute_blocked_summary(
    session_path: &Path,
    proof_path: &Path,
    state: &JamAppState,
    error: &crate::jam_app::JamAppError,
    observer_events: bool,
) -> Value {
    let surface_gate = state.daw_session_export_surface_gate();
    json!({
        "mode": "daw_session_host_import_proof_export_execute",
        "status": "blocked",
        "ready": false,
        "writes_files": false,
        "mutates_session": false,
        "observer_events": observer_events,
        "boundary": "host_import_proof_v1",
        "session_path": session_path,
        "proof_path": proof_path,
        "readiness_blockers": [error.to_string()],
        "receipt": null,
        "action": latest_daw_session_export_action_summary(
            state,
            riotbox_core::action::DawSessionExportBoundary::HostImportProofV1,
        ),
        "commit_records": [],
        "daw_session_surface_gate": daw_session_surface_gate_summary(&surface_gate),
        "scope_note": "host-import proof only; audible output remains unproven",
    })
}

fn latest_daw_session_export_action_summary(
    state: &JamAppState,
    expected_boundary: riotbox_core::action::DawSessionExportBoundary,
) -> Value {
    let action = state
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .find(|action| {
            action.command == riotbox_core::action::ActionCommand::ExportDawSession
                && matches!(
                    &action.params,
                    riotbox_core::action::ActionParams::DawSessionExport {
                        boundary,
                        ..
                    } if *boundary == expected_boundary
                )
        });

    let Some(action) = action else {
        return Value::Null;
    };

    let destination_path = match &action.params {
        riotbox_core::action::ActionParams::DawSessionExport {
            destination_path, ..
        } => destination_path.as_deref(),
        _ => None,
    };
    json!({
        "id": action.id,
        "command": action.command,
        "status": action.status,
        "destination_path": destination_path,
        "result": action.result,
    })
}

fn latest_daw_session_export_commit_records(
    state: &JamAppState,
    expected_boundary: riotbox_core::action::DawSessionExportBoundary,
) -> Vec<Value> {
    let Some(action_id) = latest_daw_session_export_action_id(state, expected_boundary) else {
        return Vec::new();
    };

    state
        .session
        .action_log
        .commit_records
        .iter()
        .filter(|record| record.action_id == action_id)
        .map(|record| {
            json!({
                "action_id": record.action_id,
                "commit_sequence": record.commit_sequence,
                "committed_at": record.committed_at,
                "boundary": record.boundary,
            })
        })
        .collect()
}

fn latest_daw_session_export_action_id(
    state: &JamAppState,
    expected_boundary: riotbox_core::action::DawSessionExportBoundary,
) -> Option<riotbox_core::ids::ActionId> {
    state
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .find(|action| {
            action.command == riotbox_core::action::ActionCommand::ExportDawSession
                && matches!(
                    &action.params,
                    riotbox_core::action::ActionParams::DawSessionExport {
                        boundary,
                        ..
                    } if *boundary == expected_boundary
                )
        })
        .map(|action| action.id)
}
