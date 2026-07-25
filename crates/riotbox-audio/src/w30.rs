pub const W30_PREVIEW_SAMPLE_WINDOW_LEN: usize = 2_048;
pub const W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN: usize = 16_384;
/// Callback-safe full-duration mono proxy for the active internal resample artifact.
///
/// The source artifact may be longer than this fixed payload. Control-plane projection then keeps
/// evenly spaced PCM frames across the complete artifact, matching the existing committed-pad
/// playback seam instead of reducing the resample to one short transient grain.
pub const W30_RESAMPLE_SOURCE_WINDOW_LEN: usize = W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN;
pub const W30_PAD_CHOP_SLICE_COUNT: usize = 8;
pub const W30_RESAMPLE_HARD_SLICE_COUNT: usize = 8;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum W30PreviewRenderMode {
    Idle,
    LiveRecall,
    RawCaptureAudition,
    PromotedAudition,
}

impl W30PreviewRenderMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::LiveRecall => "live_recall",
            Self::RawCaptureAudition => "raw_capture_audition",
            Self::PromotedAudition => "promoted_audition",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum W30PreviewRenderRouting {
    Silent,
    MusicBusPreview,
}

impl W30PreviewRenderRouting {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Silent => "silent",
            Self::MusicBusPreview => "music_bus_preview",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum W30PreviewSourceProfile {
    PinnedRecall,
    PromotedRecall,
    SlicePoolBrowse,
    RawCaptureAudition,
    PromotedAudition,
}

impl W30PreviewSourceProfile {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::PinnedRecall => "pinned_recall",
            Self::PromotedRecall => "promoted_recall",
            Self::SlicePoolBrowse => "slice_pool_browse",
            Self::RawCaptureAudition => "raw_capture_audition",
            Self::PromotedAudition => "promoted_audition",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum W30ResampleTapMode {
    Idle,
    CaptureLineageReady,
}

impl W30ResampleTapMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::CaptureLineageReady => "capture_lineage_ready",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum W30ResampleTapRouting {
    Silent,
    InternalCaptureTap,
}

impl W30ResampleTapRouting {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Silent => "silent",
            Self::InternalCaptureTap => "internal_capture_tap",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum W30ResampleTapSourceProfile {
    RawCapture,
    PromotedCapture,
    PinnedCapture,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum W30ResampleTapAvailability {
    Idle,
    SourceAudioUnavailable,
    SourceAudioReady,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum W30ResampleTapVariation {
    #[default]
    Base,
    HardDamage,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum W30ResampleTapHardPolicy {
    #[default]
    Unavailable,
    SourceTransientChop,
    SourceTextureBite,
}

impl W30ResampleTapHardPolicy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::SourceTransientChop => "source_transient_chop",
            Self::SourceTextureBite => "source_texture_bite",
        }
    }
}

impl W30ResampleTapVariation {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::HardDamage => "hard_damage",
        }
    }
}

impl W30ResampleTapAvailability {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::SourceAudioUnavailable => "source_audio_unavailable",
            Self::SourceAudioReady => "source_audio_ready",
        }
    }
}

impl W30ResampleTapSourceProfile {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::RawCapture => "raw_capture",
            Self::PromotedCapture => "promoted_capture",
            Self::PinnedCapture => "pinned_capture",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct W30PreviewRenderState {
    pub mode: W30PreviewRenderMode,
    pub routing: W30PreviewRenderRouting,
    pub source_profile: Option<W30PreviewSourceProfile>,
    pub active_bank_id: Option<String>,
    pub focused_pad_id: Option<String>,
    pub capture_id: Option<String>,
    pub trigger_revision: u64,
    pub trigger_velocity: f32,
    pub source_window_preview: Option<W30PreviewSampleWindow>,
    pub pad_playback: Option<W30PadPlaybackSampleWindow>,
    pub music_bus_level: f32,
    pub grit_level: f32,
    pub is_transport_running: bool,
    pub tempo_bpm: f32,
    pub position_beats: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct W30PreviewSampleWindow {
    pub source_start_frame: u64,
    pub source_end_frame: u64,
    pub sample_count: usize,
    pub samples: [f32; W30_PREVIEW_SAMPLE_WINDOW_LEN],
}

#[derive(Clone, Debug, PartialEq)]
pub struct W30PadPlaybackSampleWindow {
    pub source_start_frame: u64,
    pub source_end_frame: u64,
    pub source_sample_rate: u32,
    pub playback_frame_count: u64,
    pub sample_count: usize,
    pub loop_enabled: bool,
    pub playback_rate: f32,
    pub reverse: bool,
    /// Fraction of one W-30 trigger step kept audible before a short choke.
    /// Zero disables the gate and preserves the full slice playback path.
    pub gate_step_fraction: f32,
    pub loop_crossfade_sample_count: usize,
    pub chop_slice_count: usize,
    pub chop_slice_starts: [u32; W30_PAD_CHOP_SLICE_COUNT],
    pub samples: [f32; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
}

impl Default for W30PreviewRenderState {
    fn default() -> Self {
        Self {
            mode: W30PreviewRenderMode::Idle,
            routing: W30PreviewRenderRouting::Silent,
            source_profile: None,
            active_bank_id: None,
            focused_pad_id: None,
            capture_id: None,
            trigger_revision: 0,
            trigger_velocity: 0.0,
            source_window_preview: None,
            pad_playback: None,
            music_bus_level: 0.0,
            grit_level: 0.0,
            is_transport_running: false,
            tempo_bpm: 0.0,
            position_beats: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct W30ResampleTapState {
    pub mode: W30ResampleTapMode,
    pub routing: W30ResampleTapRouting,
    pub availability: W30ResampleTapAvailability,
    pub source_profile: Option<W30ResampleTapSourceProfile>,
    pub source_capture_id: Option<String>,
    pub source_audio: Option<Box<W30ResampleSourceWindow>>,
    pub lineage_capture_count: u8,
    pub generation_depth: u8,
    pub variation: W30ResampleTapVariation,
    /// Stable action-log position of the committed gesture that activated the variation.
    ///
    /// Zero means that no post-resample variation gesture has committed.
    pub variation_revision: u64,
    pub variation_intensity: f32,
    pub hard_policy: W30ResampleTapHardPolicy,
    /// Eight source-derived eighth-note trigger decisions, least-significant bit first.
    pub hard_trigger_mask: u8,
    /// Source-proxy cursor for the detected local onset in each performed eighth-note slot.
    pub hard_slice_cursors: [u16; W30_RESAMPLE_HARD_SLICE_COUNT],
    /// Strongest positive 20 ms envelope rise divided by the mean source envelope.
    pub hard_transient_contrast: f32,
    pub music_bus_level: f32,
    pub grit_level: f32,
    pub is_transport_running: bool,
    pub tempo_bpm: f32,
    pub position_beats: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct W30ResampleSourceWindow {
    pub source_start_frame: u64,
    pub source_sample_rate: u32,
    pub source_frame_count: u64,
    pub sample_count: usize,
    pub samples: [f32; W30_RESAMPLE_SOURCE_WINDOW_LEN],
}

impl Default for W30ResampleTapState {
    fn default() -> Self {
        Self {
            mode: W30ResampleTapMode::Idle,
            routing: W30ResampleTapRouting::Silent,
            availability: W30ResampleTapAvailability::Idle,
            source_profile: None,
            source_capture_id: None,
            source_audio: None,
            lineage_capture_count: 0,
            generation_depth: 0,
            variation: W30ResampleTapVariation::Base,
            variation_revision: 0,
            variation_intensity: 0.0,
            hard_policy: W30ResampleTapHardPolicy::Unavailable,
            hard_trigger_mask: 0,
            hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            hard_transient_contrast: 0.0,
            music_bus_level: 0.0,
            grit_level: 0.0,
            is_transport_running: false,
            tempo_bpm: 0.0,
            position_beats: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        W30PreviewRenderMode, W30PreviewRenderRouting, W30PreviewRenderState,
        W30PreviewSourceProfile, W30ResampleTapAvailability, W30ResampleTapMode,
        W30ResampleTapRouting, W30ResampleTapSourceProfile, W30ResampleTapState,
    };

    #[test]
    fn default_preview_state_is_idle_and_silent() {
        let state = W30PreviewRenderState::default();

        assert_eq!(state.mode, W30PreviewRenderMode::Idle);
        assert_eq!(state.routing, W30PreviewRenderRouting::Silent);
        assert_eq!(state.source_profile, None);
        assert_eq!(state.active_bank_id, None);
        assert_eq!(state.focused_pad_id, None);
        assert_eq!(state.capture_id, None);
        assert_eq!(state.trigger_revision, 0);
        assert_eq!(state.trigger_velocity, 0.0);
        assert_eq!(state.source_window_preview, None);
        assert!(!state.is_transport_running);
    }

    #[test]
    fn preview_labels_stay_stable() {
        assert_eq!(W30PreviewRenderMode::Idle.label(), "idle");
        assert_eq!(W30PreviewRenderMode::LiveRecall.label(), "live_recall");
        assert_eq!(
            W30PreviewRenderMode::RawCaptureAudition.label(),
            "raw_capture_audition"
        );
        assert_eq!(
            W30PreviewRenderMode::PromotedAudition.label(),
            "promoted_audition"
        );
        assert_eq!(W30PreviewRenderRouting::Silent.label(), "silent");
        assert_eq!(
            W30PreviewRenderRouting::MusicBusPreview.label(),
            "music_bus_preview"
        );
        assert_eq!(
            W30PreviewSourceProfile::PinnedRecall.label(),
            "pinned_recall"
        );
        assert_eq!(
            W30PreviewSourceProfile::PromotedRecall.label(),
            "promoted_recall"
        );
        assert_eq!(
            W30PreviewSourceProfile::SlicePoolBrowse.label(),
            "slice_pool_browse"
        );
        assert_eq!(
            W30PreviewSourceProfile::RawCaptureAudition.label(),
            "raw_capture_audition"
        );
        assert_eq!(
            W30PreviewSourceProfile::PromotedAudition.label(),
            "promoted_audition"
        );
        assert_eq!(W30ResampleTapMode::Idle.label(), "idle");
        assert_eq!(
            W30ResampleTapMode::CaptureLineageReady.label(),
            "capture_lineage_ready"
        );
        assert_eq!(W30ResampleTapRouting::Silent.label(), "silent");
        assert_eq!(
            W30ResampleTapRouting::InternalCaptureTap.label(),
            "internal_capture_tap"
        );
        assert_eq!(W30ResampleTapAvailability::Idle.label(), "idle");
        assert_eq!(
            W30ResampleTapAvailability::SourceAudioUnavailable.label(),
            "source_audio_unavailable"
        );
        assert_eq!(
            W30ResampleTapAvailability::SourceAudioReady.label(),
            "source_audio_ready"
        );
        assert_eq!(
            W30ResampleTapSourceProfile::RawCapture.label(),
            "raw_capture"
        );
        assert_eq!(
            W30ResampleTapSourceProfile::PromotedCapture.label(),
            "promoted_capture"
        );
        assert_eq!(
            W30ResampleTapSourceProfile::PinnedCapture.label(),
            "pinned_capture"
        );
    }

    #[test]
    fn default_resample_tap_state_is_idle_and_silent() {
        let state = W30ResampleTapState::default();

        assert_eq!(state.mode, W30ResampleTapMode::Idle);
        assert_eq!(state.routing, W30ResampleTapRouting::Silent);
        assert_eq!(state.availability, W30ResampleTapAvailability::Idle);
        assert_eq!(state.source_profile, None);
        assert_eq!(state.source_capture_id, None);
        assert_eq!(state.source_audio, None);
        assert_eq!(state.lineage_capture_count, 0);
        assert_eq!(state.generation_depth, 0);
        assert!(!state.is_transport_running);
    }
}
