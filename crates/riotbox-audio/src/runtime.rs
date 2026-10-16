use crate::runtime::{
    fill_focus::FillFocusRenderState,
    shared_w30_resample_callback::CallbackTimingSnapshot,
    tr909_tail_telemetry::{
        w30_mode_from_u32, w30_mode_to_u32, w30_routing_from_u32, w30_routing_to_u32,
        w30_source_profile_from_u32, w30_source_profile_to_u32,
    },
};
use control_snapshot::{
    COHERENT_SNAPSHOT_READ_ATTEMPTS, begin_coherent_snapshot_update, coherent_snapshot_or,
    finish_coherent_snapshot_update, try_coherent_snapshot,
};

mod control_snapshot;
mod control_sync {
    pub(super) use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
    #[cfg(test)]
    pub(super) fn model(work: impl Fn() + Send + Sync + 'static) {
        work();
    }
}
mod transport_timing;
pub use transport_timing::{
    AudioTransportControlError, AudioTransportPositionRequest, AudioTransportPositionUpdate,
};

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
mod timing_snapshot;
mod timing_sync {
    pub(super) use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
}
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

#[cfg(test)]
mod tests;
