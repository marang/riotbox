use std::{
    fs, io,
    path::{Path, PathBuf},
    time::Duration,
};

use riotbox_audio::runtime::{AudioRuntimeLifecycle, AudioRuntimeShell};
use serde_json::{Value, json};

use crate::{
    jam_app::JamAppState,
    observer::observer_snapshot,
    ui::{JamShellState, ShellLaunchMode},
};

use super::{AppLaunch, LaunchMode, UserSessionObserver, launch_summary, timestamp_now};

pub(super) fn run_live_master_recording_execute(
    launch: &AppLaunch,
    raw_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut observer = open_live_master_recording_observer(launch)?;
    let (mut summary, shell) = match live_master_recording_execute_summary(launch) {
        Ok(result) => result,
        Err(error) => {
            if let Some(observer) = observer.as_mut()
                && let Err(observer_error) = observer.record(json!({
                    "event": "live_master_recording_execute_failed",
                    "schema": "riotbox.user_session_observer.v1",
                    "timestamp_ms": timestamp_now(),
                    "opt_in": true,
                    "capture_context": "non_interactive_real_audio_cli",
                    "raw_audio_recording": false,
                    "live_master_recording": true,
                    "realtime_callback_io": false,
                    "argv": raw_args,
                    "launch": launch_summary(launch),
                    "failure_reason": error.to_string(),
                }))
            {
                eprintln!(
                    "live master recording observer could not record launch failure: {observer_error}"
                );
            }
            return Err(error);
        }
    };
    let observer_result = match observer.as_mut() {
        Some(observer) => observer.record(json!({
            "event": "live_master_recording_execute",
            "schema": "riotbox.user_session_observer.v1",
            "timestamp_ms": timestamp_now(),
            "opt_in": true,
            "capture_context": "non_interactive_real_audio_cli",
            "raw_audio_recording": false,
            "live_master_recording": true,
            "realtime_callback_io": false,
            "argv": raw_args,
            "launch": launch_summary(launch),
            "summary": summary.clone(),
            "snapshot": observer_snapshot(&shell),
        })),
        None => Ok(()),
    };
    apply_live_master_observer_status(&mut summary, observer.is_some(), &observer_result);
    if let Err(error) = observer_result {
        eprintln!(
            "live master recording completed, but the optional observer write failed: {error}"
        );
    }
    serde_json::to_writer_pretty(std::io::stdout(), &summary)?;
    println!();
    if summary.get("ready").and_then(Value::as_bool) == Some(true) {
        Ok(())
    } else {
        Err(summary
            .get("failure_reason")
            .and_then(Value::as_str)
            .unwrap_or("live master recording failed")
            .to_owned()
            .into())
    }
}

pub(super) fn apply_live_master_observer_status(
    summary: &mut Value,
    observer_requested: bool,
    observer_result: &io::Result<()>,
) {
    match (observer_requested, observer_result) {
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

fn open_live_master_recording_observer(
    launch: &AppLaunch,
) -> Result<Option<UserSessionObserver>, Box<dyn std::error::Error>> {
    let Some(observer_path) = launch.observer_path.as_deref() else {
        return Ok(None);
    };
    let LaunchMode::LiveMasterRecordingExecute {
        session_path,
        source_graph_path,
        destination_path,
    } = &launch.mode
    else {
        return Err("live master observer requires live master recording mode".into());
    };
    validate_live_master_observer_path(
        observer_path,
        session_path,
        source_graph_path.as_deref(),
        destination_path,
    )?;
    Ok(Some(UserSessionObserver::open_new(observer_path)?))
}

pub(super) fn validate_live_master_observer_path(
    observer_path: &Path,
    session_path: &Path,
    source_graph_path: Option<&Path>,
    destination_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let proof_path = crate::jam_app::live_master_recording_proof_path(destination_path)?;
    let mut protected_paths = vec![
        ("Session", fs::canonicalize(session_path)?),
        (
            "recording destination",
            canonical_future_path(destination_path, false)?,
        ),
        (
            "recording proof",
            canonical_future_path(&proof_path, false)?,
        ),
    ];
    if let Some(source_graph_path) = source_graph_path {
        protected_paths.push(("Source Graph", fs::canonicalize(source_graph_path)?));
    }
    let observer_exists = match fs::symlink_metadata(observer_path) {
        Ok(_) => true,
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Err(error) => return Err(error.into()),
    };
    let observer_identity = if observer_exists {
        fs::canonicalize(observer_path).unwrap_or_else(|_| observer_path.to_path_buf())
    } else {
        canonical_future_path(observer_path, true)?
    };
    if let Some((label, _)) = protected_paths
        .iter()
        .find(|(_, protected_path)| *protected_path == observer_identity)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("live master observer path aliases the {label} path"),
        )
        .into());
    }
    if observer_exists {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "live master observer path must be fresh and must not already exist",
        )
        .into());
    }
    Ok(())
}

fn canonical_future_path(path: &Path, create_parent: bool) -> io::Result<PathBuf> {
    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "live master path requires an explicit file name",
        )
    })?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if create_parent {
        fs::create_dir_all(parent)?;
    }
    Ok(fs::canonicalize(parent)?.join(file_name))
}

fn live_master_recording_execute_summary(
    launch: &AppLaunch,
) -> Result<(Value, JamShellState), Box<dyn std::error::Error>> {
    let LaunchMode::LiveMasterRecordingExecute {
        session_path,
        source_graph_path,
        destination_path,
    } = &launch.mode
    else {
        return Err("not a live master recording execute launch".into());
    };

    let mut state = JamAppState::from_json_files(session_path, source_graph_path.as_deref())?;
    let mut runtime =
        AudioRuntimeShell::start_default_output_with_render_states_and_source_monitor(
            state.runtime.tr909_render.clone(),
            state.runtime.mc202_render,
            state.runtime.w30_preview.clone(),
            state.runtime.w30_resample_tap.clone(),
            state.source_monitor_render_state(),
        )?;
    let output = runtime
        .health_snapshot()
        .output
        .clone()
        .ok_or("live master recording runtime has no output identity")?;

    let started_transport = !state.runtime.transport.is_playing;
    if started_transport {
        state.commit_transport_toggle(timestamp_now());
    }
    sync_live_master_runtime(&mut state, &runtime);
    if let Err(error) = wait_for_live_transport(&mut state, &runtime) {
        stop_live_master_runtime(&mut state, &mut runtime, started_transport);
        return Err(error);
    }

    let plan = match state.queue_live_master_recording(timestamp_now(), &output, destination_path) {
        crate::jam_app::LiveMasterRecordingQueueResult::Enqueued(plan) => plan,
        crate::jam_app::LiveMasterRecordingQueueResult::Rejected { reason } => {
            return Ok(blocked_live_master_recording_result(
                launch,
                state,
                runtime,
                started_transport,
                None,
                &output,
                reason,
            ));
        }
        crate::jam_app::LiveMasterRecordingQueueResult::AlreadyPending => {
            return Ok(blocked_live_master_recording_result(
                launch,
                state,
                runtime,
                started_transport,
                None,
                &output,
                "another live recording export is already pending".into(),
            ));
        }
    };
    if let Err(error) = runtime.begin_live_master_capture(plan.request.clone()) {
        let reason = format!("live master callback capture could not start: {error:?}");
        state.reject_live_master_recording(plan.action_id, reason.clone());
        return Ok(blocked_live_master_recording_result(
            launch,
            state,
            runtime,
            started_transport,
            Some(&plan),
            &output,
            reason,
        ));
    }

    let expected_duration_seconds =
        plan.request.target_frame_count as f64 / f64::from(output.sample_rate);
    let expected_arm_seconds =
        (plan.requested_start_position_beats - state.runtime.transport.position_beats).max(0.0)
            * 60.0
            / f64::from(plan.confirmed_bpm);
    let deadline = std::time::Instant::now()
        + Duration::from_secs_f64(expected_arm_seconds + expected_duration_seconds + 2.0);
    loop {
        sync_live_master_runtime(&mut state, &runtime);
        let health = runtime.health_snapshot();
        state.set_audio_health(health.clone());
        if health.lifecycle != AudioRuntimeLifecycle::Running
            || health.stream_error_count > 0
            || health.callback_scratch_overflow_count > 0
        {
            stop_live_master_runtime(&mut state, &mut runtime, started_transport);
            runtime.abort_live_master_capture();
            let reason = format!("live master runtime faulted during capture: {health:?}");
            state.reject_live_master_recording(plan.action_id, reason.clone());
            return Ok(blocked_live_master_recording_result(
                launch,
                state,
                runtime,
                false,
                Some(&plan),
                &output,
                reason,
            ));
        }
        if let Some(progress) = runtime.live_master_capture_progress() {
            if progress.fault_count() > 0 {
                stop_live_master_runtime(&mut state, &mut runtime, started_transport);
                let progress = runtime.abort_live_master_capture().unwrap_or(progress);
                let reason = format!(
                    "live master callback capture faulted while waiting for or recording the bar window: {progress:?}"
                );
                state.reject_live_master_recording(plan.action_id, reason.clone());
                return Ok(blocked_live_master_recording_result(
                    launch,
                    state,
                    runtime,
                    false,
                    Some(&plan),
                    &output,
                    reason,
                ));
            }
            if progress.complete {
                break;
            }
        }
        if std::time::Instant::now() >= deadline {
            stop_live_master_runtime(&mut state, &mut runtime, started_transport);
            let progress = runtime.abort_live_master_capture();
            let reason = format!("live master callback capture stopped early: {progress:?}");
            state.reject_live_master_recording(plan.action_id, reason.clone());
            return Ok(blocked_live_master_recording_result(
                launch,
                state,
                runtime,
                false,
                Some(&plan),
                &output,
                reason,
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    sync_live_master_runtime(&mut state, &runtime);
    let final_health = runtime.health_snapshot();
    state.set_audio_health(final_health.clone());
    stop_live_master_runtime(&mut state, &mut runtime, started_transport);
    let stopped_health = runtime.health_snapshot();
    if stopped_health.lifecycle != AudioRuntimeLifecycle::Stopped {
        let progress = runtime.abort_live_master_capture();
        let reason = format!(
            "live master runtime did not stop before recording finalization: {stopped_health:?}; capture: {progress:?}"
        );
        state.reject_live_master_recording(plan.action_id, reason.clone());
        return Ok(blocked_live_master_recording_result(
            launch,
            state,
            runtime,
            false,
            Some(&plan),
            &output,
            reason,
        ));
    }

    let outcome = match runtime.finish_live_master_capture() {
        Ok(outcome) => outcome,
        Err(error) => {
            let reason = format!("live master callback capture did not finalize: {error:?}");
            state.reject_live_master_recording(plan.action_id, reason.clone());
            return Ok(blocked_live_master_recording_result(
                launch,
                state,
                runtime,
                false,
                Some(&plan),
                &output,
                reason,
            ));
        }
    };
    let receipt = match state.commit_and_save_live_master_recording(
        &plan,
        &outcome,
        &final_health,
        timestamp_now(),
    ) {
        Ok(receipt) => receipt,
        Err(error) => {
            return Ok(blocked_live_master_recording_result(
                launch,
                state,
                runtime,
                false,
                Some(&plan),
                &output,
                error.to_string(),
            ));
        }
    };

    let summary = json!({
        "mode": "live_master_recording_execute",
        "status": "ready",
        "ready": true,
        "writes_files": true,
        "mutates_session": true,
        "runtime_stopped": true,
        "observer_events": launch.observer_path.is_some(),
        "boundary": "runtime_master_bar_window_v2",
        "receipt_boundary": receipt.export_boundary.as_proof_str(),
        "session_path": session_path,
        "destination_path": destination_path,
        "proof_path": plan.proof_path,
        "duration_beats": crate::jam_app::LIVE_MASTER_RECORDING_DURATION_BEATS,
        "beats_per_bar": plan.beats_per_bar,
        "bar_grid_anchor_beat_cursor": plan.bar_grid_anchor_beat_cursor,
        "requested_start_position_beats": plan.requested_start_position_beats,
        "captured_start_position_beats": outcome.captured_start_position_beats,
        "captured_end_position_beats": outcome.captured_end_position_beats,
        "sample_rate_hz": output.sample_rate,
        "channel_count": output.channel_count,
        "frame_count": plan.request.target_frame_count,
        "host": output.host_name,
        "device": output.device_name,
        "readiness_blockers": [],
        "receipt": {
            "receipt_id": receipt.receipt_id,
            "pack_id": receipt.pack_id,
            "export_scope": receipt.export_scope,
            "export_role": receipt.export_role,
            "sha256": receipt.export_hash,
            "proof_sha256": receipt.normalized_manifest_hash,
            "qa_gates": receipt.qa_gates,
            "host_audio_readiness": receipt.live_recording_host_audio_readiness_report(),
        },
        "scope_note": "captured the next exact two-bar 4/4 window from the real post-limiter callback master; no input recording, offline substitute, new DSP, or release claim",
    });
    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    Ok((summary, shell))
}

fn stop_live_master_runtime(
    state: &mut JamAppState,
    runtime: &mut AudioRuntimeShell,
    started_transport: bool,
) {
    stop_cli_started_transport(state, runtime, started_transport);
    runtime.stop();
    state.set_audio_health(runtime.health_snapshot());
}

fn blocked_live_master_recording_result(
    launch: &AppLaunch,
    mut state: JamAppState,
    mut runtime: AudioRuntimeShell,
    started_transport: bool,
    plan: Option<&crate::jam_app::LiveMasterRecordingPlan>,
    output: &riotbox_audio::runtime::AudioOutputInfo,
    reason: String,
) -> (Value, JamShellState) {
    stop_cli_started_transport(&mut state, &runtime, started_transport);
    runtime.stop();
    state.set_audio_health(runtime.health_snapshot());
    let retained_paths = plan
        .into_iter()
        .flat_map(|plan| [&plan.destination_path, &plan.proof_path])
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    let summary = json!({
        "mode": "live_master_recording_execute",
        "status": "blocked",
        "ready": false,
        "writes_files": !retained_paths.is_empty(),
        "mutates_session": false,
        "runtime_stopped": true,
        "observer_events": launch.observer_path.is_some(),
        "boundary": "runtime_master_bar_window_v2",
        "destination_path": plan.map(|plan| &plan.destination_path),
        "proof_path": plan.map(|plan| &plan.proof_path),
        "action_id": plan.map(|plan| plan.action_id.0),
        "sample_rate_hz": output.sample_rate,
        "channel_count": output.channel_count,
        "host": output.host_name,
        "device": output.device_name,
        "failure_reason": reason,
        "readiness_blockers": [reason],
        "retained_paths": retained_paths,
        "scope_note": "capture failed closed without a live-recording receipt; any path that could not be proven absent is surfaced explicitly",
    });
    (summary, JamShellState::new(state, ShellLaunchMode::Load))
}

fn stop_cli_started_transport(
    state: &mut JamAppState,
    runtime: &AudioRuntimeShell,
    started_transport: bool,
) {
    if started_transport && state.runtime.transport.is_playing {
        state.commit_transport_toggle(timestamp_now());
        runtime.update_transport_state(
            false,
            state.runtime.tr909_render.tempo_bpm,
            state.runtime.transport.position_beats,
        );
    }
}

fn wait_for_live_transport(
    state: &mut JamAppState,
    runtime: &AudioRuntimeShell,
) -> Result<(), Box<dyn std::error::Error>> {
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    loop {
        sync_live_master_runtime(state, runtime);
        let timing = runtime.timing_snapshot();
        let expected_bpm = state
            .session
            .runtime_state
            .source_timing
            .confirmed_bpm
            .ok_or("live master recording Session BPM disappeared")?;
        if timing.is_transport_running && timing.tempo_bpm.to_bits() == expected_bpm.to_bits() {
            return Ok(());
        }
        let health = runtime.health_snapshot();
        if health.lifecycle != AudioRuntimeLifecycle::Running
            || health.stream_error_count > 0
            || health.callback_scratch_overflow_count > 0
        {
            return Err(format!("audio runtime faulted before live capture: {health:?}").into());
        }
        if std::time::Instant::now() >= deadline {
            return Err("audio transport did not confirm before live capture".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn sync_live_master_runtime(state: &mut JamAppState, runtime: &AudioRuntimeShell) {
    state.apply_audio_timing_snapshot(runtime.timing_snapshot(), timestamp_now());
    runtime.update_transport_state(
        state.runtime.transport.is_playing,
        state.runtime.tr909_render.tempo_bpm,
        state.runtime.transport.position_beats,
    );
    runtime.update_tr909_render_state(&state.runtime.tr909_render);
    runtime.update_mc202_render_state(&state.runtime.mc202_render);
    runtime.update_w30_preview_render_state(&state.runtime.w30_preview);
    runtime.update_w30_resample_tap_state(&state.runtime.w30_resample_tap);
    runtime.update_source_monitor_control_state(&state.source_monitor_control_state());
}
