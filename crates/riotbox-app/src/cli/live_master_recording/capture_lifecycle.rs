//! Owns the recorder's control lifecycle, not a second recording or Session model.
//! Both production and generated tests execute the same queue/stop/publication path.
use std::time::{Duration, Instant};

use super::observation::{Observation, WindowStopped};
use crate::cli::{
    model::{AppLaunch, LaunchMode, live_master_recording_boundary},
    observer::timestamp_now,
};
use crate::jam_app::JamAppState;
use riotbox_audio::runtime::{
    AudioRuntimeHealth, AudioRuntimeLifecycle, AudioRuntimeShell, AudioRuntimeTimingSnapshot,
    LiveMasterCaptureError, LiveMasterCaptureOutcome, LiveMasterCaptureProgress,
    LiveMasterCaptureRequest,
};
use riotbox_core::{TimestampMs, action::LiveRecordingDuration};
use serde_json::{Value, json};

pub(super) fn execute(
    launch: &AppLaunch,
    state: &mut JamAppState,
    runtime: &mut AudioRuntimeShell,
    observation: &mut Observation<'_>,
) -> Result<Value, Box<dyn std::error::Error>> {
    // Borrow the large facade; the caller constructs the shell after this
    // control frame returns. Moving it through nested frames can exhaust a
    // normal test-thread stack while committing transport/refreshing its view.
    execute_with_control_observed(launch, state, &mut DeviceControl(runtime), observation)
}

// Private dependency seam: only observations, projection and existing device
// operations vary. Session/queue policy and publication never live in an adapter.
trait RuntimeControl {
    fn health_snapshot(&self) -> AudioRuntimeHealth;
    fn timing_snapshot(&self) -> AudioRuntimeTimingSnapshot;
    fn update_transport_state(&self, playing: bool, bpm: f32, position: f64);
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

impl RuntimeControl for DeviceControl<'_> {
    fn health_snapshot(&self) -> AudioRuntimeHealth {
        self.0.health_snapshot()
    }
    fn timing_snapshot(&self) -> AudioRuntimeTimingSnapshot {
        self.0.timing_snapshot()
    }
    fn update_transport_state(&self, playing: bool, bpm: f32, position: f64) {
        self.0.update_transport_state(playing, bpm, position);
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

fn execute_with_control_observed(
    launch: &AppLaunch,
    state: &mut JamAppState,
    runtime: &mut impl RuntimeControl,
    observation: &mut Observation<'_>,
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
    if started_transport {
        state.commit_transport_toggle(runtime.timestamp_now());
    }
    sync_live_master_runtime(state, runtime);
    if let Err(error) = wait_for_live_transport(state, runtime) {
        stop_live_master_runtime(state, runtime, started_transport);
        return Err(error);
    }

    let plan = match state.queue_live_master_recording_with_duration(
        runtime.timestamp_now(),
        &output,
        destination_path,
        *duration,
    ) {
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
    let deadline = runtime.now()
        + Duration::from_secs_f64(expected_arm_seconds + expected_duration_seconds + 2.0);
    let completion = loop {
        sync_live_master_runtime(state, runtime);
        let health = runtime.health_snapshot();
        state.set_audio_health(health.clone());
        if health.lifecycle != AudioRuntimeLifecycle::Running
            || health.stream_error_count > 0
            || health.callback_scratch_overflow_count > 0
        {
            stop_live_master_runtime(state, runtime, started_transport);
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
                stop_live_master_runtime(state, runtime, started_transport);
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
                // Only in-memory observations before stop; optional observer I/O
                // cannot prolong the live output window.
                break observation
                    .enabled()
                    .then(|| (runtime.monotonic_ns(), progress));
            }
        }
        if runtime.now() >= deadline {
            stop_live_master_runtime(state, runtime, started_transport);
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
        runtime.wait(Duration::from_millis(10));
    };

    sync_live_master_runtime(state, runtime);
    let final_health = runtime.health_snapshot();
    state.set_audio_health(final_health.clone());
    stop_live_master_runtime(state, runtime, started_transport);
    let stopped_ns = observation
        .enabled()
        .then(|| runtime.monotonic_ns())
        .flatten();
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

    if let Some((complete_ns, progress)) = completion {
        observation.record(&WindowStopped::new(
            &plan,
            complete_ns,
            stopped_ns,
            progress.fault_count(),
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
        runtime.timestamp_now(),
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
) {
    stop_cli_started_transport(state, runtime, started_transport);
    runtime.stop();
    state.set_audio_health(runtime.health_snapshot());
}

fn blocked_live_master_recording_result(
    launch: &AppLaunch,
    state: &mut JamAppState,
    runtime: &mut impl RuntimeControl,
    started_transport: bool,
    plan: Option<&crate::jam_app::LiveMasterRecordingPlan>,
    output: &riotbox_audio::runtime::AudioOutputInfo,
    reason: String,
) -> Value {
    stop_cli_started_transport(state, runtime, started_transport);
    runtime.stop();
    state.set_audio_health(runtime.health_snapshot());
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
) {
    if started_transport && state.runtime.transport.is_playing {
        state.commit_transport_toggle(runtime.timestamp_now());
        runtime.update_transport_state(
            false,
            state.runtime.tr909_render.tempo_bpm,
            state.runtime.transport.position_beats,
        );
    }
}

fn wait_for_live_transport(
    state: &mut JamAppState,
    runtime: &impl RuntimeControl,
) -> Result<(), Box<dyn std::error::Error>> {
    let deadline = runtime.now() + Duration::from_secs(2);
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
        if runtime.now() >= deadline {
            return Err("audio transport did not confirm before live capture".into());
        }
        runtime.wait(Duration::from_millis(10));
    }
}

fn sync_live_master_runtime(state: &mut JamAppState, runtime: &impl RuntimeControl) {
    state.apply_audio_timing_snapshot(runtime.timing_snapshot(), runtime.timestamp_now());
    runtime.update_transport_state(
        state.runtime.transport.is_playing,
        state.runtime.tr909_render.tempo_bpm,
        state.runtime.transport.position_beats,
    );
    runtime.update_render_states(state);
}
