use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    },
    time::Instant,
};

use crate::{
    mc202::{
        Mc202ContourHint, Mc202HookResponse, Mc202NoteBudget, Mc202PhraseShape, Mc202RenderMode,
        Mc202RenderRouting, Mc202RenderState, Mc202SourcePhraseRenderPlan, render_mc202_buffer,
    },
    source_audio::SourceAudioCache,
    tr909::{
        Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
        Tr909RenderState, Tr909SourceSupportContext, Tr909SourceSupportProfile,
        Tr909TakeoverRenderProfile,
    },
    w30::{
        W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN, W30_PREVIEW_SAMPLE_WINDOW_LEN,
        W30PadPlaybackSampleWindow, W30PreviewRenderMode, W30PreviewRenderRouting,
        W30PreviewRenderState, W30PreviewSampleWindow, W30PreviewSourceProfile, W30ResampleTapMode,
        W30ResampleTapRouting, W30ResampleTapSourceProfile, W30ResampleTapState,
    },
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

mod public_api_shell;
mod render_tr909_w30_preview;
mod runtime_mix_parity;
mod shared_mc202_w30_preview;
mod shared_transport_tr909;
mod shared_w30_resample_callback;
mod source_monitor;
mod tr909_tail_telemetry;
mod w30_tr909_signal_helpers;

pub use public_api_shell::*;
use render_tr909_w30_preview::{
    render_tr909_buffer, render_w30_preview_buffer, render_w30_resample_tap_buffer,
};
pub use runtime_mix_parity::*;
use shared_mc202_w30_preview::{
    RealtimeMc202RenderState, RealtimeW30PadPlaybackSampleWindow, RealtimeW30PreviewRenderState,
    RealtimeW30PreviewSampleWindow, SharedMc202RenderState, SharedW30PreviewRenderState,
};
#[cfg(test)]
use shared_transport_tr909::AudioRuntimeShellTestParts;
use shared_transport_tr909::{
    RealtimeTr909RenderState, RealtimeTransportTimingState, SharedTr909RenderState,
    SharedTransportTimingState,
};
use shared_w30_resample_callback::{
    CallbackTimingSnapshot, RealtimeW30ResampleTapState, SharedW30ResampleTapState,
    Tr909CallbackState, TransportTimingCallbackState, W30MixRenderState, W30PreviewCallbackState,
    W30ResampleTapCallbackState, advance_transport_timing, render_mix_buffer,
};
use source_monitor::{SharedSourceMonitorRenderState, apply_source_monitor_policy};
pub use source_monitor::{
    SourceMonitorAudioRoute, SourceMonitorAudioSource, SourceMonitorRenderState,
    render_source_monitor_mix_offline, source_monitor_route_for_cache,
    source_monitor_route_for_output,
};
use tr909_tail_telemetry::{
    RuntimeTelemetry, envelope_decay, mode_from_u32, mode_to_u32, pattern_adoption_from_u32,
    pattern_adoption_to_u32, phrase_variation_from_u32, phrase_variation_to_u32, routing_from_u32,
    routing_to_u32, support_context_from_u32, support_context_to_u32, support_profile_from_u32,
    support_profile_to_u32, takeover_profile_from_u32, takeover_profile_to_u32, w30_mode_from_u32,
    w30_mode_to_u32, w30_routing_from_u32, w30_routing_to_u32, w30_source_profile_from_u32,
    w30_source_profile_to_u32,
};
use w30_tr909_signal_helpers::{
    render_gain, render_subdivision, should_trigger_step, trigger_envelope, trigger_frequency,
    w30_envelope_decay, w30_preview_idle_bpm, w30_render_gain,
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
