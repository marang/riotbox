use crate::runtime::{
    fill_focus::FillFocusRenderState,
    shared_w30_resample_callback::CallbackTimingSnapshot,
    tr909_tail_telemetry::{
        w30_mode_from_u32, w30_mode_to_u32, w30_routing_from_u32, w30_routing_to_u32,
        w30_source_profile_from_u32, w30_source_profile_to_u32,
    },
};
use std::sync::atomic::{AtomicU64, Ordering};

mod fill_focus;
#[cfg(any(test, feature = "limiter-calibration"))]
pub mod limiter_calibration;
mod live_master_capture;
mod public_api_shell;
mod render_tr909_w30_preview;
mod runtime_mix_parity;
mod shared_mc202;
mod shared_transport_tr909;
mod shared_w30_resample_callback;
mod source_monitor;
mod telemetry;
mod tr909_fill_recipe;
mod tr909_fill_voice;
mod tr909_tail_telemetry;
mod w30_filter_slam;
mod w30_preview_snapshot;
mod w30_tr909_signal_helpers;

pub use live_master_capture::{
    LIVE_MASTER_CALLBACK_GAP_THRESHOLD_MICROS, LIVE_MASTER_MAX_INTERLEAVED_SAMPLE_COUNT,
    LiveMasterCaptureError, LiveMasterCaptureOutcome, LiveMasterCaptureProgress,
    LiveMasterCaptureRequest,
};
pub use public_api_shell::*;

pub use runtime_mix_parity::*;

pub use source_monitor::{
    SourceMonitorAudioRoute, SourceMonitorAudioSource, SourceMonitorRenderState,
    render_source_monitor_mix_offline, source_monitor_route_for_cache,
    source_monitor_route_for_output,
};

const COHERENT_SNAPSHOT_READ_ATTEMPTS: usize = 3;

fn begin_coherent_snapshot_update(revision: &AtomicU64) {
    let previous = revision.fetch_add(1, Ordering::AcqRel);
    debug_assert_eq!(previous % 2, 0, "coherent snapshot update overlap");
}

fn finish_coherent_snapshot_update(revision: &AtomicU64) {
    let previous = revision.fetch_add(1, Ordering::Release);
    debug_assert_eq!(previous % 2, 1, "coherent snapshot update was not active");
}

fn coherent_snapshot<T>(revision: &AtomicU64, read: impl Fn() -> T) -> T {
    let mut last_read = None;
    for _ in 0..COHERENT_SNAPSHOT_READ_ATTEMPTS {
        let before = revision.load(Ordering::Acquire);
        if !before.is_multiple_of(2) {
            continue;
        }
        let snapshot = read();
        let after = revision.load(Ordering::Acquire);
        if before == after && after.is_multiple_of(2) {
            return snapshot;
        }
        last_read = Some(snapshot);
    }
    last_read.unwrap_or_else(read)
}

fn coherent_snapshot_or<T: Clone>(revision: &AtomicU64, previous: &T, read: impl Fn() -> T) -> T {
    for _ in 0..COHERENT_SNAPSHOT_READ_ATTEMPTS {
        let before = revision.load(Ordering::Acquire);
        if !before.is_multiple_of(2) {
            continue;
        }
        let snapshot = read();
        let after = revision.load(Ordering::Acquire);
        if before == after && after.is_multiple_of(2) {
            return snapshot;
        }
    }
    previous.clone()
}

#[cfg(test)]
mod tests;
