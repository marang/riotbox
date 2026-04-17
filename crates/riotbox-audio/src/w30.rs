#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum W30PreviewRenderMode {
    Idle,
    LiveRecall,
    PromotedAudition,
}

impl W30PreviewRenderMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::LiveRecall => "live_recall",
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
    PromotedAudition,
}

impl W30PreviewSourceProfile {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::PinnedRecall => "pinned_recall",
            Self::PromotedRecall => "promoted_recall",
            Self::SlicePoolBrowse => "slice_pool_browse",
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
    pub music_bus_level: f32,
    pub grit_level: f32,
    pub is_transport_running: bool,
    pub tempo_bpm: f32,
    pub position_beats: f64,
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
    pub source_profile: Option<W30ResampleTapSourceProfile>,
    pub source_capture_id: Option<String>,
    pub lineage_capture_count: u8,
    pub generation_depth: u8,
    pub music_bus_level: f32,
    pub grit_level: f32,
    pub is_transport_running: bool,
}

impl Default for W30ResampleTapState {
    fn default() -> Self {
        Self {
            mode: W30ResampleTapMode::Idle,
            routing: W30ResampleTapRouting::Silent,
            source_profile: None,
            source_capture_id: None,
            lineage_capture_count: 0,
            generation_depth: 0,
            music_bus_level: 0.0,
            grit_level: 0.0,
            is_transport_running: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        W30PreviewRenderMode, W30PreviewRenderRouting, W30PreviewRenderState,
        W30PreviewSourceProfile, W30ResampleTapMode, W30ResampleTapRouting,
        W30ResampleTapSourceProfile, W30ResampleTapState,
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
        assert!(!state.is_transport_running);
    }

    #[test]
    fn preview_labels_stay_stable() {
        assert_eq!(W30PreviewRenderMode::Idle.label(), "idle");
        assert_eq!(W30PreviewRenderMode::LiveRecall.label(), "live_recall");
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
        assert_eq!(state.source_profile, None);
        assert_eq!(state.source_capture_id, None);
        assert_eq!(state.lineage_capture_count, 0);
        assert_eq!(state.generation_depth, 0);
        assert!(!state.is_transport_running);
    }
}
