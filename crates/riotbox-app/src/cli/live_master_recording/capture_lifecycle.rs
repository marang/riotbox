//! Owns the recorder's control lifecycle, not a second recording or Session model.
//! Both production and generated tests execute the same queue/stop/publication path.
use std::{
    ops::ControlFlow,
    time::{Duration, Instant},
};

use super::observation::Observation;
use super::terminal_input::RecordingInput;
use crate::cli::transport_sync::{TransportControl, publish_transport, sync_transport};
use crate::cli::{
    model::{AppLaunch, LaunchMode, live_master_recording_boundary},
    observer::timestamp_now,
};
use crate::jam_app::{JamAppError, JamAppState};
use riotbox_audio::runtime::{
    AudioRuntimeHealth, AudioRuntimeLifecycle, AudioRuntimeShell, AudioRuntimeTimingSnapshot,
    AudioTransportControlError, AudioTransportPositionUpdate, LiveMasterCaptureError,
    LiveMasterCaptureOutcome, LiveMasterCaptureProgress, LiveMasterCaptureRequest,
};
use riotbox_core::{TimestampMs, action::LiveRecordingDuration};
use serde_json::{Value, json};

mod capture_wait;
mod input_control;
use capture_wait::CaptureWait;
use input_control::{InputControl, execute as execute_with_control_observed_and_input};

pub(super) fn execute(
    launch: &AppLaunch,
    state: &mut JamAppState,
    runtime: &mut AudioRuntimeShell,
    observation: &mut Observation<'_>,
    input: Option<&mut dyn RecordingInput>,
) -> Result<Value, Box<dyn std::error::Error>> {
    // Borrow the large facade; the caller constructs the shell after this
    // control frame returns. Moving it through nested frames can exhaust a
    // normal test-thread stack while committing transport/refreshing its view.
    execute_with_control_observed_and_input(
        launch,
        state,
        &mut DeviceControl(runtime),
        observation,
        input,
    )
}

// Private dependency seam: only observations, projection and existing device
// operations vary. Session/queue policy and publication never live in an adapter.
trait RuntimeControl: TransportControl {
    fn health_snapshot(&self) -> AudioRuntimeHealth;
    fn update_render_states(&self, state: &JamAppState);
    fn begin_live_master_capture(
        &self,
        request: LiveMasterCaptureRequest,
    ) -> Result<(), LiveMasterCaptureError>;
    fn live_master_capture_progress(&self) -> Option<LiveMasterCaptureProgress>;
    fn abort_live_master_capture(&self) -> Option<LiveMasterCaptureProgress>;
    fn finish_live_master_capture(
        &self,
    ) -> Result<LiveMasterCaptureOutcome, LiveMasterCaptureError>;
    fn stop(&mut self);
    fn now(&self) -> Instant;
    fn timestamp_now(&self) -> TimestampMs;
    fn wait(&self, duration: Duration);
    fn monotonic_ns(&self) -> Option<u64>;
}

struct DeviceControl<'a>(&'a mut AudioRuntimeShell);

#[cfg(test)]
mod tests;

impl TransportControl for DeviceControl<'_> {
    fn timing_snapshot(&self) -> Option<AudioRuntimeTimingSnapshot> {
        self.0.timing_snapshot()
    }
    fn update_transport_state(
        &self,
        playing: bool,
        bpm: f32,
        position: AudioTransportPositionUpdate,
    ) -> Result<(), AudioTransportControlError> {
        self.0.update_transport_state(playing, bpm, position)
    }
}

impl RuntimeControl for DeviceControl<'_> {
    fn health_snapshot(&self) -> AudioRuntimeHealth {
        self.0.health_snapshot()
    }
    fn update_render_states(&self, state: &JamAppState) {
        self.0
            .update_tr909_render_state(&state.runtime.tr909_render);
        self.0
            .update_mc202_render_state(&state.runtime.mc202_render);
        self.0
            .update_w30_preview_render_state(&state.runtime.w30_preview);
        self.0
            .update_w30_resample_tap_state(&state.runtime.w30_resample_tap);
        self.0
            .update_source_monitor_control_state(&state.source_monitor_control_state());
    }
    fn begin_live_master_capture(
        &self,
        request: LiveMasterCaptureRequest,
    ) -> Result<(), LiveMasterCaptureError> {
        self.0.begin_live_master_capture(request)
    }
    fn live_master_capture_progress(&self) -> Option<LiveMasterCaptureProgress> {
        self.0.live_master_capture_progress()
    }
    fn abort_live_master_capture(&self) -> Option<LiveMasterCaptureProgress> {
        self.0.abort_live_master_capture()
    }
    fn finish_live_master_capture(
        &self,
    ) -> Result<LiveMasterCaptureOutcome, LiveMasterCaptureError> {
        self.0.finish_live_master_capture()
    }
    fn stop(&mut self) {
        self.0.stop();
    }
    fn now(&self) -> Instant {
        Instant::now()
    }
    fn timestamp_now(&self) -> TimestampMs {
        timestamp_now()
    }
    fn wait(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
    fn monotonic_ns(&self) -> Option<u64> {
        super::observation::monotonic_ns()
    }
}

#[cfg(test)]
fn execute_with_control(
    launch: &AppLaunch,
    state: &mut JamAppState,
    runtime: &mut impl RuntimeControl,
) -> Result<Value, Box<dyn std::error::Error>> {
    execute_with_control_observed(launch, state, runtime, &mut Observation::new(None))
}

#[cfg(test)]
fn execute_with_control_and_input(
    launch: &AppLaunch,
    state: &mut JamAppState,
    runtime: &mut impl RuntimeControl,
    input: &mut dyn RecordingInput,
) -> Result<Value, Box<dyn std::error::Error>> {
    execute_with_control_observed_and_input(
        launch,
        state,
        runtime,
        &mut Observation::new(None),
        Some(input),
    )
}

#[cfg(test)]
fn execute_with_control_observed(
    launch: &AppLaunch,
    state: &mut JamAppState,
    runtime: &mut impl RuntimeControl,
    observation: &mut Observation<'_>,
) -> Result<Value, Box<dyn std::error::Error>> {
    execute_with_control_observed_and_input(launch, state, runtime, observation, None)
}

fn execute_policy(
    launch: &AppLaunch,
    state: &mut JamAppState,
    runtime: &mut impl RuntimeControl,
    observation: &mut Observation<'_>,
    mut input: Option<&mut dyn RecordingInput>,
    input_control: &mut InputControl,
) -> Result<Value, Box<dyn std::error::Error>> {
    let LaunchMode::LiveMasterRecordingExecute {
        session_path,
        destination_path,
        duration,
        ..
    } = &launch.mode
    else {
        return Err("not a live master recording execute launch".into());
    };

    let output = runtime
        .health_snapshot()
        .output
        .clone()
        .ok_or("live master recording runtime has no output identity")?;

    let started_transport = !state.runtime.transport.is_playing;
    if started_transport && let Err(error) = state.commit_transport_toggle(runtime.timestamp_now())
    {
        stop_live_master_runtime(state, runtime, false)?;
        return Err(error.into());
    }
    let confirmation = sync_live_master_runtime(state, runtime)
        .map_err(Box::<dyn std::error::Error>::from)
        .and_then(|()| wait_for_live_transport(state, runtime));
    if let Err(error) = confirmation {
        if let Err(stop_error) = stop_live_master_runtime(state, runtime, started_transport) {
            return Err(format!("{error}; transport cleanup failed: {stop_error}").into());
        }
        return Err(error);
    }

    let plan = match state.queue_live_master_recording_with_duration(
        runtime.timestamp_now(),
        &output,
        destination_path,
        *duration,
    ) {
        Ok(crate::jam_app::LiveMasterRecordingQueueResult::Enqueued(plan)) => plan,
        Ok(crate::jam_app::LiveMasterRecordingQueueResult::Rejected { reason }) => {
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
        Ok(crate::jam_app::LiveMasterRecordingQueueResult::AlreadyPending) => {
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
        Err(error) => {
            return Ok(blocked_live_master_recording_result(
                launch,
                state,
                runtime,
                started_transport,
                None,
                &output,
                error.to_string(),
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
    let deadline = runtime.now()
        + Duration::from_secs_f64(expected_arm_seconds + expected_duration_seconds + 2.0);
    let mut wait = CaptureWait::new(launch, &plan, &output, started_transport, deadline);
    let stopped = loop {
        match wait.poll(state, runtime, observation.enabled()) {
            ControlFlow::Continue(next) => {
                wait = next;
                if let Some(input) = input.as_deref_mut()
                    && let Err(error) = input_control.pending(state, input, runtime.timestamp_now())
                {
                    return Ok(blocked_acquired_capture_failure(
                        launch,
                        state,
                        runtime,
                        started_transport,
                        &plan,
                        &output,
                        format!("live master terminal input {error}"),
                    ));
                }
                runtime.wait(Duration::from_millis(10));
            }
            ControlFlow::Break(Ok(stopped)) => break stopped,
            ControlFlow::Break(Err(summary)) => return Ok(summary),
        }
    };
    let (final_health, milestone) = stopped.into_parts();
    if let Some(milestone) = milestone {
        observation.record(&milestone);
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
        runtime.timestamp_now(),
    ) {
        Ok(receipt) => receipt,
        Err(error) => {
            // The controller already consumed/finalized this take. A writer's
            // pre-publication refusal must retire its owned pending request.
            state.reject_live_master_recording(plan.action_id, error.to_string());
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
        "boundary": live_master_recording_boundary(plan.duration),
        "receipt_boundary": receipt.export_boundary.as_proof_str(),
        "session_path": session_path,
        "destination_path": destination_path,
        "proof_path": plan.proof_path,
        "duration_bars": plan.duration.bars(),
        "duration_beats": plan.duration.duration_beats(),
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
        "scope_note": format!("captured the next exact {}-bar 4/4 window from the real post-limiter callback master with the current scene unchanged; no input recording, offline substitute, new DSP, or release claim", plan.duration.bars()),
    });
    Ok(summary)
}

fn stop_live_master_runtime(
    state: &mut JamAppState,
    runtime: &mut impl RuntimeControl,
    started_transport: bool,
) -> Result<(), JamAppError> {
    let transport_result = stop_cli_started_transport(state, runtime, started_transport);
    // Stopping the device is a safety operation, not a newly committed Action.
    // Exhaustion may prevent a Pause record but must never skip the device stop.
    runtime.stop();
    state.set_audio_health(runtime.health_snapshot());
    transport_result
}

fn with_transport_cleanup_error(mut reason: String, result: Result<(), JamAppError>) -> String {
    if let Err(error) = result {
        reason.push_str(&format!("; transport cleanup failed: {error}"));
    }
    reason
}

fn blocked_capture_sync_failure(
    launch: &AppLaunch,
    state: &mut JamAppState,
    runtime: &mut impl RuntimeControl,
    started_transport: bool,
    plan: &crate::jam_app::LiveMasterRecordingPlan,
    output: &riotbox_audio::runtime::AudioOutputInfo,
    error: JamAppError,
) -> Value {
    blocked_acquired_capture_failure(
        launch,
        state,
        runtime,
        started_transport,
        plan,
        output,
        format!("live master timing commit failed: {error}"),
    )
}

fn blocked_acquired_capture_failure(
    launch: &AppLaunch,
    state: &mut JamAppState,
    runtime: &mut impl RuntimeControl,
    started_transport: bool,
    plan: &crate::jam_app::LiveMasterRecordingPlan,
    output: &riotbox_audio::runtime::AudioOutputInfo,
    reason: String,
) -> Value {
    let stop_result = stop_live_master_runtime(state, runtime, started_transport);
    runtime.abort_live_master_capture();
    let reason = with_transport_cleanup_error(reason, stop_result);
    state.reject_live_master_recording(plan.action_id, reason.clone());
    blocked_live_master_recording_result(launch, state, runtime, false, Some(plan), output, reason)
}

fn blocked_live_master_recording_result(
    launch: &AppLaunch,
    state: &mut JamAppState,
    runtime: &mut impl RuntimeControl,
    started_transport: bool,
    plan: Option<&crate::jam_app::LiveMasterRecordingPlan>,
    output: &riotbox_audio::runtime::AudioOutputInfo,
    mut reason: String,
) -> Value {
    if let Err(error) = stop_live_master_runtime(state, runtime, started_transport) {
        reason.push_str(&format!("; transport cleanup failed: {error}"));
    }
    let retained_paths = plan
        .into_iter()
        .flat_map(|plan| [&plan.destination_path, &plan.proof_path])
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    let duration = match launch.mode {
        LaunchMode::LiveMasterRecordingExecute { duration, .. } => Some(duration),
        _ => None,
    };
    let summary = json!({
        "mode": "live_master_recording_execute",
        "status": "blocked",
        "ready": false,
        "writes_files": !retained_paths.is_empty(),
        "mutates_session": false,
        "runtime_stopped": runtime.health_snapshot().lifecycle == AudioRuntimeLifecycle::Stopped,
        "observer_events": launch.observer_path.is_some(),
        "boundary": duration.map(live_master_recording_boundary),
        "duration_bars": duration.map(LiveRecordingDuration::bars),
        "duration_beats": duration.map(LiveRecordingDuration::duration_beats),
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
    summary
}

fn stop_cli_started_transport(
    state: &mut JamAppState,
    runtime: &impl RuntimeControl,
    started_transport: bool,
) -> Result<(), JamAppError> {
    if started_transport && state.runtime.transport.is_playing {
        state.commit_transport_toggle(runtime.timestamp_now())?;
        publish_transport(state, runtime)?;
    }
    Ok(())
}

fn wait_for_live_transport(
    state: &mut JamAppState,
    runtime: &impl RuntimeControl,
) -> Result<(), Box<dyn std::error::Error>> {
    let deadline = runtime.now() + Duration::from_secs(2);
    loop {
        sync_live_master_runtime(state, runtime)?;
        let timing = runtime.timing_snapshot();
        let expected_bpm = state
            .session
            .runtime_state
            .source_timing
            .confirmed_bpm
            .ok_or("live master recording Session BPM disappeared")?;
        if timing.is_some_and(|timing| {
            timing.is_transport_running && timing.tempo_bpm.to_bits() == expected_bpm.to_bits()
        }) {
            return Ok(());
        }
        let health = runtime.health_snapshot();
        if health.lifecycle != AudioRuntimeLifecycle::Running
            || health.stream_error_count > 0
            || health.callback_scratch_overflow_count > 0
        {
            return Err(format!("audio runtime faulted before live capture: {health:?}").into());
        }
        if runtime.now() >= deadline {
            return Err("audio transport did not confirm before live capture".into());
        }
        runtime.wait(Duration::from_millis(10));
    }
}

fn sync_live_master_runtime(
    state: &mut JamAppState,
    runtime: &impl RuntimeControl,
) -> Result<(), JamAppError> {
    sync_transport(state, runtime, runtime.timestamp_now())?;
    runtime.update_render_states(state);
    Ok(())
}
