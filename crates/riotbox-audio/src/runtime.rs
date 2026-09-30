use crate::runtime::{
    fill_focus::FillFocusRenderState,
    shared_w30_resample_callback::CallbackTimingSnapshot,
    tr909_tail_telemetry::{
        w30_mode_from_u32, w30_mode_to_u32, w30_routing_from_u32, w30_routing_to_u32,
        w30_source_profile_from_u32, w30_source_profile_to_u32,
    },
};
use std::sync::atomic::{AtomicU64, Ordering};

// Existing runtime test fixtures retain their private parent paths only in tests.
#[cfg(test)]
use crate::{
    mc202::{Mc202ContourHint, Mc202HookResponse, Mc202NoteBudget, Mc202SourcePhraseRenderPlan},
    runtime::{
        fill_focus::apply_fill_focus_to_non_tr909_bed,
        render_tr909_w30_preview::{
            render_tr909_buffer, render_w30_preview_buffer, render_w30_resample_tap_buffer,
            should_trigger_w30_step, w30_chop_slice_cursor, w30_pad_grid_gate,
            w30_pad_grid_gate_gain, w30_pad_playback_sample, w30_pad_playback_signature,
        },
        shared_mc202::{RealtimeMc202RenderState, SharedMc202RenderState},
        shared_transport_tr909::{
            AudioRuntimeShellTestParts, RealtimeTr909RenderState, SharedTr909RenderState,
            SharedTransportTimingState,
        },
        shared_w30_resample_callback::{
            RealtimeW30ResampleSourceWindow, RealtimeW30ResampleTapState,
            SharedW30ResampleTapState, Tr909CallbackState, W30MixRenderState,
            W30PreviewCallbackState, W30ResampleTapCallbackState, render_mix_buffer,
        },
        source_monitor::{
            SharedSourceMonitorRenderState, SourceMonitorCallbackState,
            apply_source_monitor_policy_with_state,
        },
        telemetry::RuntimeTelemetry,
        tr909_tail_telemetry::{envelope_decay, mode_to_u32},
        w30_preview_snapshot::{
            RealtimeW30PadPlaybackSampleWindow, RealtimeW30PreviewRenderState,
            RealtimeW30PreviewSampleWindow, SharedW30PreviewRenderState, W30PreviewSnapshotCache,
        },
        w30_tr909_signal_helpers::{
            break_performance_slam, fill_performance_slam, render_gain, render_subdivision,
            should_trigger_step, tr909_step_waveform, trigger_envelope, trigger_frequency,
        },
    },
    source_audio::SourceAudioCache,
    w30::{
        W30_PAD_CHOP_SLICE_COUNT, W30_PREVIEW_SAMPLE_WINDOW_LEN, W30_RESAMPLE_SOURCE_WINDOW_LEN,
        W30HookArticulationProfile, W30ResampleSourceWindow, W30ResampleTapAvailability,
    },
};
#[cfg(test)]
use std::sync::Arc;

mod fill_focus;
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
