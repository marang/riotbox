use serde::{Deserialize, Serialize};

use crate::{
    action::SourceMonitorMode,
    session::{MacroState, MixerState, SessionFile, Tr909ReinforcementModeState},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StyleProfileId {
    FeralRebuild,
}

impl StyleProfileId {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::FeralRebuild => "feral_rebuild",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerformancePresetId {
    FeralBreakAlphaV1,
    FeralBreakAlphaV2,
}

impl PerformancePresetId {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::FeralBreakAlphaV1 => "Feral Break Alpha",
            Self::FeralBreakAlphaV2 => "Feral Break Alpha v2",
        }
    }

    #[must_use]
    pub const fn contract_id(self) -> &'static str {
        match self {
            Self::FeralBreakAlphaV1 => "feral_break_alpha_v1",
            Self::FeralBreakAlphaV2 => "feral_break_alpha_v2",
        }
    }

    #[must_use]
    pub const fn definition(self) -> PerformancePresetDefinition {
        match self {
            Self::FeralBreakAlphaV1 => PerformancePresetDefinition {
                profile_id: StyleProfileId::FeralRebuild,
                w30_role: PresetW30Role::SourceHookLead,
                tr909_role: PresetTr909Role::BreakPressure,
                tr909_reinforcement_mode: Tr909ReinforcementModeState::SourceSupport,
                mc202_role: PresetMc202Role::SourceEvidenceSelected,
                bass_ownership: PresetBassOwnership::LivePerformancePolicy,
                source_monitor_mode: SourceMonitorMode::Blend,
                w30_hook_selection_policy:
                    crate::session::W30HookSelectionPolicy::TransportBoundaryV1,
                macro_state: MacroState {
                    source_retain: 0.68,
                    chaos: 0.52,
                    mc202_touch: 0.82,
                    // Keep the source hook open enough that a committed retrigger remains
                    // an obvious phase reset; heavier grit belongs to the damage gesture.
                    w30_grit: 0.40,
                    tr909_slam: 0.70,
                    scene_aggression: 0.80,
                    capture_eagerness: 0.76,
                    dirt_room_intensity: 0.66,
                },
                mixer_state: MixerState {
                    source_level: 0.72,
                    drum_level: 0.84,
                    music_level: 0.72,
                    fx_send_level: 0.32,
                    master_level: 0.82,
                },
            },
            Self::FeralBreakAlphaV2 => PerformancePresetDefinition {
                profile_id: StyleProfileId::FeralRebuild,
                w30_role: PresetW30Role::SourceHookLead,
                tr909_role: PresetTr909Role::BreakPressure,
                // The committed V2 preset action owns the typed break-pressure vocabulary
                // needed by its documented `s`/`f` path. It does not invent a source-derived
                // phrase or pattern reference.
                tr909_reinforcement_mode: Tr909ReinforcementModeState::BreakReinforce,
                mc202_role: PresetMc202Role::SourceEvidenceSelected,
                bass_ownership: PresetBassOwnership::LivePerformancePolicy,
                // V2 promotes the captured W-30 hook to the performance lead. Keeping the raw
                // source in Blend can double the same break at its original source phase while
                // the pad restarts its captured downbeat on the performance grid.
                source_monitor_mode: SourceMonitorMode::Riotbox,
                // RBX-285 keeps the shipped preset at the current transport boundary until
                // the frozen Development comparison establishes one candidate winner.
                w30_hook_selection_policy:
                    crate::session::W30HookSelectionPolicy::TransportBoundaryV1,
                macro_state: MacroState {
                    // V2 gives the promoted source hook a more hostile sampler character while
                    // the live performance policy still decides which lane hits hardest.
                    source_retain: 0.56,
                    chaos: 0.58,
                    mc202_touch: 0.82,
                    w30_grit: 0.64,
                    tr909_slam: 0.76,
                    scene_aggression: 0.86,
                    capture_eagerness: 0.76,
                    dirt_room_intensity: 0.74,
                },
                mixer_state: MixerState {
                    source_level: 0.60,
                    drum_level: 0.88,
                    // The nonlinear hook character supplies perceived bite. Keep electrical
                    // headroom for source-dependent peaks instead of manufacturing impact with
                    // a hotter music bus.
                    music_level: 0.58,
                    fx_send_level: 0.38,
                    master_level: 0.82,
                },
            },
        }
    }

    pub fn apply_to_session(self, session: &mut SessionFile) {
        let definition = self.definition();
        session.runtime_state.style.active_profile = Some(definition.profile_id);
        session.runtime_state.style.active_preset = Some(self);
        session.runtime_state.style.w30_hook_selection_policy =
            definition.w30_hook_selection_policy;
        session.runtime_state.source_monitor.mode = definition.source_monitor_mode;
        session.runtime_state.macro_state = definition.macro_state;
        session.runtime_state.mixer_state = definition.mixer_state;
        session.runtime_state.lane_state.tr909.reinforcement_mode =
            Some(definition.tr909_reinforcement_mode);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PresetW30Role {
    SourceHookLead,
}

impl PresetW30Role {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SourceHookLead => "source_hook_lead",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PresetTr909Role {
    BreakPressure,
}

impl PresetTr909Role {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::BreakPressure => "break_pressure",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PresetMc202Role {
    SourceEvidenceSelected,
}

impl PresetMc202Role {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SourceEvidenceSelected => "source_evidence_selected",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PresetBassOwnership {
    LivePerformancePolicy,
}

impl PresetBassOwnership {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::LivePerformancePolicy => "live_performance_policy",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PerformancePresetDefinition {
    pub profile_id: StyleProfileId,
    pub w30_role: PresetW30Role,
    pub tr909_role: PresetTr909Role,
    pub tr909_reinforcement_mode: Tr909ReinforcementModeState,
    pub mc202_role: PresetMc202Role,
    pub bass_ownership: PresetBassOwnership,
    pub source_monitor_mode: SourceMonitorMode,
    pub w30_hook_selection_policy: crate::session::W30HookSelectionPolicy,
    pub macro_state: MacroState,
    pub mixer_state: MixerState,
}

#[cfg(test)]
mod tests {
    use crate::{
        action::SourceMonitorMode,
        session::{SessionFile, Tr909ReinforcementModeState, W30HookSelectionPolicy},
    };

    use super::{PerformancePresetId, PresetBassOwnership, StyleProfileId};

    #[test]
    fn feral_break_alpha_applies_one_named_versioned_session_recipe() {
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-07-17T14:20:00Z");

        PerformancePresetId::FeralBreakAlphaV1.apply_to_session(&mut session);

        assert_eq!(
            session.runtime_state.style.active_profile,
            Some(StyleProfileId::FeralRebuild)
        );
        assert_eq!(
            session.runtime_state.style.active_preset,
            Some(PerformancePresetId::FeralBreakAlphaV1)
        );
        assert_eq!(
            session.runtime_state.source_monitor.mode,
            SourceMonitorMode::Blend
        );
        assert_eq!(
            session.runtime_state.lane_state.tr909.reinforcement_mode,
            Some(Tr909ReinforcementModeState::SourceSupport)
        );
        assert_eq!(
            PerformancePresetId::FeralBreakAlphaV1
                .definition()
                .bass_ownership,
            PresetBassOwnership::LivePerformancePolicy
        );
    }

    #[test]
    fn applying_preset_does_not_invent_source_derived_lane_material() {
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-07-17T14:21:00Z");

        PerformancePresetId::FeralBreakAlphaV1.apply_to_session(&mut session);

        assert!(
            session
                .runtime_state
                .lane_state
                .mc202
                .source_phrase_plan
                .is_none()
        );
        assert!(session.runtime_state.lane_state.w30.last_capture.is_none());
        assert!(session.runtime_state.lane_state.tr909.pattern_ref.is_none());
    }

    #[test]
    fn alpha_v2_strengthens_the_transformed_hook_with_explicit_headroom() {
        let v1 = PerformancePresetId::FeralBreakAlphaV1.definition();
        let v2 = PerformancePresetId::FeralBreakAlphaV2.definition();

        assert!(v2.macro_state.w30_grit > v1.macro_state.w30_grit);
        assert!(v2.mixer_state.music_level < v1.mixer_state.music_level);
        assert_eq!(v2.source_monitor_mode, SourceMonitorMode::Riotbox);
        assert_eq!(
            v2.tr909_reinforcement_mode,
            Tr909ReinforcementModeState::BreakReinforce
        );
        assert_eq!(
            v2.w30_hook_selection_policy,
            W30HookSelectionPolicy::TransportBoundaryV1
        );
    }
}
