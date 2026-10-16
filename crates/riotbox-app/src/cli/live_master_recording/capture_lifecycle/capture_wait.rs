//! One consumed control-side supervision step; never an audio recorder or clock.
use std::{ops::ControlFlow, time::Instant};

use riotbox_audio::runtime::{AudioOutputInfo, AudioRuntimeHealth, AudioRuntimeLifecycle};
use serde_json::Value;

use super::{
    AppLaunch, JamAppState, RuntimeControl, blocked_capture_sync_failure,
    blocked_live_master_recording_result, stop_live_master_runtime, sync_live_master_runtime,
    with_transport_cleanup_error,
};
use crate::{
    cli::live_master_recording::observation::WindowStopped, jam_app::LiveMasterRecordingPlan,
};

pub(super) struct CaptureWait<'a> {
    launch: &'a AppLaunch,
    plan: &'a LiveMasterRecordingPlan,
    output: &'a AudioOutputInfo,
    started_transport: bool,
    deadline: Instant,
}

/// Proves stop only. Finish, late faults and publication are still unvalidated.
pub(super) struct StoppedCapture {
    final_health: AudioRuntimeHealth,
    milestone: Option<WindowStopped>,
}

impl StoppedCapture {
    pub(super) fn into_parts(self) -> (AudioRuntimeHealth, Option<WindowStopped>) {
        (self.final_health, self.milestone)
    }
}

impl<'a> CaptureWait<'a> {
    pub(super) fn new(
        launch: &'a AppLaunch,
        plan: &'a LiveMasterRecordingPlan,
        output: &'a AudioOutputInfo,
        started_transport: bool,
        deadline: Instant,
    ) -> Self {
        Self {
            launch,
            plan,
            output,
            started_transport,
            deadline,
        }
    }

    pub(super) fn poll(
        self,
        state: &mut JamAppState,
        runtime: &mut impl RuntimeControl,
        observe: bool,
    ) -> ControlFlow<Result<StoppedCapture, Value>, Self> {
        if let Err(error) = sync_live_master_runtime(state, runtime) {
            return ControlFlow::Break(Err(blocked_capture_sync_failure(
                self.launch,
                state,
                runtime,
                self.started_transport,
                self.plan,
                self.output,
                error,
            )));
        }
        let health = runtime.health_snapshot();
        state.set_audio_health(health.clone());
        if health.lifecycle != AudioRuntimeLifecycle::Running
            || health.stream_error_count > 0
            || health.callback_scratch_overflow_count > 0
        {
            let stop_result = stop_live_master_runtime(state, runtime, self.started_transport);
            runtime.abort_live_master_capture();
            let reason = with_transport_cleanup_error(
                format!("live master runtime faulted during capture: {health:?}"),
                stop_result,
            );
            state.reject_live_master_recording(self.plan.action_id, reason.clone());
            return ControlFlow::Break(Err(blocked_live_master_recording_result(
                self.launch,
                state,
                runtime,
                false,
                Some(self.plan),
                self.output,
                reason,
            )));
        }
        let Some(progress) = runtime.live_master_capture_progress() else {
            return self.pending_or_timeout(state, runtime);
        };
        if progress.fault_count() > 0 {
            let stop_result = stop_live_master_runtime(state, runtime, self.started_transport);
            let progress = runtime.abort_live_master_capture().unwrap_or(progress);
            let reason = with_transport_cleanup_error(
                format!(
                    "live master callback capture faulted while waiting for or recording the bar window: {progress:?}"
                ),
                stop_result,
            );
            state.reject_live_master_recording(self.plan.action_id, reason.clone());
            return ControlFlow::Break(Err(blocked_live_master_recording_result(
                self.launch,
                state,
                runtime,
                false,
                Some(self.plan),
                self.output,
                reason,
            )));
        }
        if !progress.complete {
            return self.pending_or_timeout(state, runtime);
        }
        let completion = observe.then(|| (runtime.monotonic_ns(), progress));

        if let Err(error) = sync_live_master_runtime(state, runtime) {
            return ControlFlow::Break(Err(blocked_capture_sync_failure(
                self.launch,
                state,
                runtime,
                self.started_transport,
                self.plan,
                self.output,
                error,
            )));
        }
        let final_health = runtime.health_snapshot();
        state.set_audio_health(final_health.clone());
        if let Err(error) = stop_live_master_runtime(state, runtime, self.started_transport) {
            runtime.abort_live_master_capture();
            let reason = format!("live master transport cleanup failed: {error}");
            state.reject_live_master_recording(self.plan.action_id, reason.clone());
            return ControlFlow::Break(Err(blocked_live_master_recording_result(
                self.launch,
                state,
                runtime,
                false,
                Some(self.plan),
                self.output,
                reason,
            )));
        }
        let stopped_ns = observe.then(|| runtime.monotonic_ns()).flatten();
        let stopped_health = runtime.health_snapshot();
        if stopped_health.lifecycle != AudioRuntimeLifecycle::Stopped {
            let progress = runtime.abort_live_master_capture();
            let reason = format!(
                "live master runtime did not stop before recording finalization: {stopped_health:?}; capture: {progress:?}"
            );
            state.reject_live_master_recording(self.plan.action_id, reason.clone());
            return ControlFlow::Break(Err(blocked_live_master_recording_result(
                self.launch,
                state,
                runtime,
                false,
                Some(self.plan),
                self.output,
                reason,
            )));
        }
        let milestone = completion.map(|(complete_ns, progress)| {
            WindowStopped::new(self.plan, complete_ns, stopped_ns, progress.fault_count())
        });
        ControlFlow::Break(Ok(StoppedCapture {
            final_health,
            milestone,
        }))
    }

    fn pending_or_timeout(
        self,
        state: &mut JamAppState,
        runtime: &mut impl RuntimeControl,
    ) -> ControlFlow<Result<StoppedCapture, Value>, Self> {
        if runtime.now() >= self.deadline {
            let stop_result = stop_live_master_runtime(state, runtime, self.started_transport);
            let progress = runtime.abort_live_master_capture();
            let reason = with_transport_cleanup_error(
                format!("live master callback capture stopped early: {progress:?}"),
                stop_result,
            );
            state.reject_live_master_recording(self.plan.action_id, reason.clone());
            return ControlFlow::Break(Err(blocked_live_master_recording_result(
                self.launch,
                state,
                runtime,
                false,
                Some(self.plan),
                self.output,
                reason,
            )));
        }
        ControlFlow::Continue(self)
    }
}
