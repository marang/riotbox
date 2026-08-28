fn run_stem_package_source_matched_execute(
    launch: &AppLaunch,
    raw_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    write_stem_package_source_matched_execute_output(launch, raw_args, &mut stdout)
}

fn write_stem_package_source_matched_execute_output(
    launch: &AppLaunch,
    raw_args: &[String],
    output: &mut impl std::io::Write,
) -> Result<(), Box<dyn std::error::Error>> {
    let (summary, shell) = stem_package_source_matched_execute_summary(launch)?;
    if let Some(path) = launch.observer_path.as_deref() {
        let mut observer = UserSessionObserver::open(path)?;
        observer.record(json!({
            "event": "stem_package_source_matched_execute",
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

fn stem_package_source_matched_execute_summary(
    launch: &AppLaunch,
) -> Result<(Value, JamShellState), Box<dyn std::error::Error>> {
    let LaunchMode::StemPackageSourceMatchedExecute {
        session_path,
        source_graph_path,
        handoff_proof_path,
        destination_path,
    } = &launch.mode
    else {
        return Err("not a source-matched stem package execute launch".into());
    };

    let mut state = JamAppState::from_json_files(session_path, source_graph_path.as_deref())?;
    let summary = match state.commit_stem_package_export_from_product_handoff(
        handoff_proof_path,
        destination_path,
        timestamp_now(),
    ) {
        Ok(receipt) => {
            state.save()?;
            json!({
                "mode": "stem_package_source_matched_execute",
                "status": "ready",
                "ready": true,
                "writes_files": true,
                "boundary": "stem_package.source_matched_handoff_v1",
                "session_path": session_path,
                "source_graph_path": source_graph_path,
                "handoff_proof_path": handoff_proof_path,
                "destination_path": destination_path,
                "claimed_stem_roles": ["stem_drums", "stem_music", "stem_bass"],
                "readiness_blockers": [],
                "receipt": stem_package_receipt_summary(&receipt),
            })
        }
        Err(error) => json!({
            "mode": "stem_package_source_matched_execute",
            "status": "blocked",
            "ready": false,
            "writes_files": false,
            "boundary": "stem_package.source_matched_handoff_v1",
            "session_path": session_path,
            "source_graph_path": source_graph_path,
            "handoff_proof_path": handoff_proof_path,
            "destination_path": destination_path,
            "claimed_stem_roles": ["stem_drums", "stem_music", "stem_bass"],
            "readiness_blockers": [error.to_string()],
            "receipt": null,
        }),
    };
    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    Ok((summary, shell))
}

fn run_stem_package_w30_hook_execute(
    launch: &AppLaunch,
    raw_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    write_stem_package_w30_hook_execute_output(launch, raw_args, &mut stdout)
}

fn write_stem_package_w30_hook_execute_output(
    launch: &AppLaunch,
    raw_args: &[String],
    output: &mut impl std::io::Write,
) -> Result<(), Box<dyn std::error::Error>> {
    let (summary, shell) = stem_package_w30_hook_execute_summary(launch)?;
    if let Some(path) = launch.observer_path.as_deref() {
        let mut observer = UserSessionObserver::open(path)?;
        observer.record(json!({
            "event": "stem_package_w30_hook_execute",
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

fn stem_package_w30_hook_execute_summary(
    launch: &AppLaunch,
) -> Result<(Value, JamShellState), Box<dyn std::error::Error>> {
    let LaunchMode::StemPackageW30HookExecute {
        session_path,
        source_graph_path,
        destination_path,
    } = &launch.mode
    else {
        return Err("not a W-30 hook stem package execute launch".into());
    };

    let mut state = JamAppState::from_json_files(session_path, source_graph_path.as_deref())?;
    let summary = match state
        .commit_stem_package_export_w30_hook_loop(destination_path, timestamp_now())
    {
        Ok(receipt) => {
            state.save()?;
            json!({
                "mode": "stem_package_w30_hook_execute",
                "status": "ready",
                "ready": true,
                "writes_files": true,
                "boundary": "stem_package.w30_hook_loop_v4",
                "session_path": session_path,
                "source_graph_path": source_graph_path,
                "destination_path": destination_path,
                "claimed_stem_roles": ["w30_hook_loop"],
                "readiness_blockers": [],
                "receipt": stem_package_receipt_summary(&receipt),
            })
        }
        Err(error) => json!({
            "mode": "stem_package_w30_hook_execute",
            "status": "blocked",
            "ready": false,
            "writes_files": false,
            "boundary": "stem_package.w30_hook_loop_v4",
            "session_path": session_path,
            "source_graph_path": source_graph_path,
            "destination_path": destination_path,
            "claimed_stem_roles": ["w30_hook_loop"],
            "readiness_blockers": [error.to_string()],
            "receipt": null,
        }),
    };
    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    Ok((summary, shell))
}
