fn run_w30_hook_dawproject_execute(
    launch: &AppLaunch,
    raw_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    write_w30_hook_dawproject_execute_output(launch, raw_args, &mut stdout)
}

fn write_w30_hook_dawproject_execute_output(
    launch: &AppLaunch,
    raw_args: &[String],
    output: &mut impl std::io::Write,
) -> Result<(), Box<dyn std::error::Error>> {
    let (summary, shell) = w30_hook_dawproject_execute_summary(launch)?;
    if let Some(path) = launch.observer_path.as_deref() {
        let mut observer = UserSessionObserver::open(path)?;
        observer.record(json!({
            "event": "w30_hook_dawproject_execute",
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
    serde_json::to_writer_pretty(&mut *output, &summary)?;
    writeln!(output)?;
    Ok(())
}

fn w30_hook_dawproject_execute_summary(
    launch: &AppLaunch,
) -> Result<(Value, JamShellState), Box<dyn std::error::Error>> {
    let LaunchMode::W30HookDawprojectExecute {
        session_path,
        destination_path,
    } = &launch.mode
    else {
        return Err("not a W-30 DAWproject execute launch".into());
    };

    let mut state = JamAppState::from_json_files(session_path, None::<&Path>)?;
    let summary = match state.commit_w30_hook_dawproject_export(
        session_path.parent(),
        destination_path,
        timestamp_now(),
    ) {
        Ok(receipt) => {
            state.save()?;
            json!({
                "mode": "w30_hook_dawproject_execute",
                "status": "ready",
                "ready": true,
                "writes_files": true,
                "mutates_session": true,
                "observer_events": launch.observer_path.is_some(),
                "boundary": crate::jam_app::W30_HOOK_DAWPROJECT_ACTION_BOUNDARY_ID,
                "receipt_boundary": receipt.export_boundary.as_proof_str(),
                "session_path": session_path,
                "destination_path": destination_path,
                "readiness_blockers": [],
                "receipt": {
                    "receipt_id": receipt.receipt_id,
                    "source_receipt_id": latest_w30_hook_dawproject_source_receipt_id(&state),
                    "pack_id": receipt.pack_id,
                    "export_scope": receipt.export_scope,
                    "export_role": receipt.export_role,
                    "sha256": receipt.export_hash,
                    "project_xml_sha256": receipt.normalized_manifest_hash,
                    "artifact_count": receipt.artifact_set.len(),
                    "qa_gates": receipt.qa_gates,
                },
                "action": latest_daw_session_export_action_summary(
                    &state,
                    riotbox_core::action::DawSessionExportBoundary::W30HookDawprojectV1,
                ),
                "commit_records": latest_daw_session_export_commit_records(
                    &state,
                    riotbox_core::action::DawSessionExportBoundary::W30HookDawprojectV1,
                ),
                "musician_use": ["open_or_import", "arrange", "loop_or_process"],
                "scope_note": "DAWproject structure and byte-identical embedded audio are proven; host import and audible DAW playback remain unproven",
            })
        }
        Err(error) => json!({
            "mode": "w30_hook_dawproject_execute",
            "status": "blocked",
            "ready": false,
            "writes_files": false,
            "mutates_session": false,
            "observer_events": launch.observer_path.is_some(),
            "boundary": crate::jam_app::W30_HOOK_DAWPROJECT_ACTION_BOUNDARY_ID,
            "receipt_boundary": "daw_session.w30_hook_dawproject_v1",
            "session_path": session_path,
            "destination_path": destination_path,
            "readiness_blockers": [error.to_string()],
            "receipt": null,
            "action": latest_daw_session_export_action_summary(
                &state,
                riotbox_core::action::DawSessionExportBoundary::W30HookDawprojectV1,
            ),
            "commit_records": [],
            "scope_note": "no DAWproject file or DAW Session receipt was committed",
        }),
    };
    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    Ok((summary, shell))
}

fn latest_w30_hook_dawproject_source_receipt_id(state: &JamAppState) -> Option<&str> {
    state
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .find_map(|action| match &action.params {
            riotbox_core::action::ActionParams::DawSessionExport {
                boundary: riotbox_core::action::DawSessionExportBoundary::W30HookDawprojectV1,
                receipt_id,
                ..
            } => receipt_id.as_deref(),
            _ => None,
        })
}
