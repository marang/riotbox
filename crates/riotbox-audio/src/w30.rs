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
    PromotedAudition,
}

impl W30PreviewSourceProfile {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::PinnedRecall => "pinned_recall",
            Self::PromotedRecall => "promoted_recall",
            Self::PromotedAudition => "promoted_audition",
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
        W30PreviewSourceProfile,
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
            W30PreviewSourceProfile::PromotedAudition.label(),
            "promoted_audition"
        );
    }
}
