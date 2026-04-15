#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909RenderMode {
    Idle,
    SourceSupport,
    Fill,
    BreakReinforce,
    Takeover,
}

impl Tr909RenderMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::SourceSupport => "source_support",
            Self::Fill => "fill",
            Self::BreakReinforce => "break_reinforce",
            Self::Takeover => "takeover",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tr909RenderRouting {
    SourceOnly,
    DrumBusSupport,
    DrumBusTakeover,
}

impl Tr909RenderRouting {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SourceOnly => "source_only",
            Self::DrumBusSupport => "drum_bus_support",
            Self::DrumBusTakeover => "drum_bus_takeover",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tr909RenderState {
    pub mode: Tr909RenderMode,
    pub routing: Tr909RenderRouting,
    pub pattern_ref: Option<String>,
    pub takeover_profile: Option<String>,
    pub drum_bus_level: f32,
    pub slam_intensity: f32,
    pub is_transport_running: bool,
    pub tempo_bpm: f32,
    pub position_beats: f64,
    pub current_scene_id: Option<String>,
}

impl Default for Tr909RenderState {
    fn default() -> Self {
        Self {
            mode: Tr909RenderMode::Idle,
            routing: Tr909RenderRouting::SourceOnly,
            pattern_ref: None,
            takeover_profile: None,
            drum_bus_level: 0.0,
            slam_intensity: 0.0,
            is_transport_running: false,
            tempo_bpm: 0.0,
            position_beats: 0.0,
            current_scene_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Tr909RenderMode, Tr909RenderRouting, Tr909RenderState};

    #[test]
    fn default_render_state_is_idle_and_source_only() {
        let state = Tr909RenderState::default();

        assert_eq!(state.mode, Tr909RenderMode::Idle);
        assert_eq!(state.routing, Tr909RenderRouting::SourceOnly);
        assert_eq!(state.mode.label(), "idle");
        assert_eq!(state.routing.label(), "source_only");
        assert_eq!(state.pattern_ref, None);
        assert_eq!(state.takeover_profile, None);
        assert!(!state.is_transport_running);
    }
}
