//! Optional post-stop observations; never an authority for capture or publication.
use std::io;

use crate::{cli::observer::UserSessionObserver, jam_app::LiveMasterRecordingPlan};
use riotbox_core::{
    ids::CaptureId,
    session::{ExportArtifactSourceGraphRef, ExportArtifactTimingGridRef},
};
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct WindowStopped {
    event: &'static str,
    schema: &'static str,
    observation_schema: &'static str,
    opt_in: bool,
    capture_context: &'static str,
    realtime_callback_io: bool,
    raw_audio_recording: bool,
    live_master_recording: bool,
    pid: u32,
    clock_domain: &'static str,
    window_complete_monotonic_ns: Option<u64>,
    runtime_stopped_monotonic_ns: Option<u64>,
    window_complete: bool,
    runtime_stopped: bool,
    fault_count: u64,
    action_id: u64,
    session_id: String,
    session_pre_capture_sha256: String,
    duration_bars: u8,
    duration_beats: u32,
    requested_start_position_beats: f64,
    frame_count: usize,
    sample_rate_hz: u32,
    channel_count: u16,
    source_graph_ref: Option<ExportArtifactSourceGraphRef>,
    timing_grid_ref: Option<ExportArtifactTimingGridRef>,
    source_capture_refs: Vec<CaptureId>,
    lineage_capture_refs: Vec<CaptureId>,
}

impl WindowStopped {
    pub(super) fn new(
        plan: &LiveMasterRecordingPlan,
        complete_ns: Option<u64>,
        stopped_ns: Option<u64>,
        fault_count: u64,
    ) -> Self {
        // A failed or inconsistent clock is unavailable, never a wall-clock
        // replacement or a qualified reversed interval.
        let (complete_ns, stopped_ns) = match (complete_ns, stopped_ns) {
            (Some(complete), Some(stopped)) if complete > stopped => (None, None),
            pair => pair,
        };
        Self {
            event: "live_master_recording_window_stopped",
            schema: "riotbox.user_session_observer.v1",
            observation_schema: "riotbox.live_master_recording_window_stopped.v1",
            opt_in: true,
            capture_context: "non_interactive_real_audio_cli",
            realtime_callback_io: false,
            raw_audio_recording: false,
            live_master_recording: true,
            pid: std::process::id(),
            clock_domain: "linux_clock_monotonic",
            window_complete_monotonic_ns: complete_ns,
            runtime_stopped_monotonic_ns: stopped_ns,
            window_complete: true,
            runtime_stopped: true,
            fault_count,
            action_id: plan.action_id.0,
            session_id: plan.session_id.clone(),
            session_pre_capture_sha256: plan.session_pre_capture_sha256.clone(),
            duration_bars: plan.duration.bars(),
            duration_beats: plan.duration.duration_beats(),
            requested_start_position_beats: plan.requested_start_position_beats,
            frame_count: plan.request.target_frame_count,
            sample_rate_hz: plan.output.sample_rate,
            channel_count: plan.output.channel_count,
            source_graph_ref: plan.source_graph_ref.clone(),
            timing_grid_ref: plan.timing_grid_ref.clone(),
            source_capture_refs: plan.source_capture_refs.clone(),
            lineage_capture_refs: plan.lineage_capture_refs.clone(),
        }
    }
}

pub(super) trait WindowStoppedSink {
    fn record_window_stopped(&mut self, milestone: &WindowStopped) -> io::Result<()>;
}

impl WindowStoppedSink for UserSessionObserver {
    fn record_window_stopped(&mut self, milestone: &WindowStopped) -> io::Result<()> {
        self.record(serde_json::to_value(milestone).map_err(io::Error::other)?)
    }
}

pub(super) struct Observation<'a> {
    sink: Option<&'a mut dyn WindowStoppedSink>,
    failure: Option<io::Error>,
}

impl<'a> Observation<'a> {
    pub(super) fn new(sink: Option<&'a mut dyn WindowStoppedSink>) -> Self {
        Self {
            sink,
            failure: None,
        }
    }

    pub(super) fn enabled(&self) -> bool {
        self.sink.is_some()
    }

    pub(super) fn record(&mut self, milestone: &WindowStopped) {
        if let Some(sink) = self.sink.as_mut()
            && let Err(error) = sink.record_window_stopped(milestone)
            && self.failure.is_none()
        {
            self.failure = Some(error);
        }
    }

    pub(super) fn into_result(self) -> io::Result<()> {
        self.failure.map_or(Ok(()), Err)
    }
}

#[cfg(target_os = "linux")]
pub(super) fn monotonic_ns() -> Option<u64> {
    let mut time = std::mem::MaybeUninit::<libc::timespec>::uninit();
    // SAFETY: clock_gettime initializes the timespec on success. Its storage
    // is valid and no reference to uninitialized memory is formed or read.
    if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, time.as_mut_ptr()) } != 0 {
        return None;
    }
    // SAFETY: the successful call above initialized this value.
    let time = unsafe { time.assume_init() };
    let seconds = u64::try_from(time.tv_sec).ok()?;
    let nanoseconds = u64::try_from(time.tv_nsec).ok()?;
    if nanoseconds >= 1_000_000_000 {
        return None;
    }
    seconds.checked_mul(1_000_000_000)?.checked_add(nanoseconds)
}

#[cfg(not(target_os = "linux"))]
pub(super) fn monotonic_ns() -> Option<u64> {
    None
}
