use crate::tr909::{
    Tr909FillRecipeId, Tr909PatternAdoption, Tr909PhraseVariation, select_tr909_fill_recipe_id,
};

use super::shared_transport_tr909::RealtimeTr909RenderState;

/// Fixed callback-safe drum-owner levels for one Fill step.
///
/// These are preset velocities, not source analysis and not Session state. Source-aware recipe
/// selection belongs in the product policy before the prepared render state reaches the callback.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Tr909FillVoiceTrigger {
    pub(super) kick: f32,
    pub(super) snare: f32,
    pub(super) hat: f32,
}

impl Tr909FillVoiceTrigger {
    const fn new(kick: f32, snare: f32, hat: f32) -> Self {
        Self { kick, snare, hat }
    }
}

/// One authoritative musical event on the prepared 32-step Fill grid.
///
/// `Choke` deliberately has no sounding owner. It changes callback-local voice articulation and
/// must therefore never be counted as a hit by trigger-policy or QA code.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) enum Tr909FillStep {
    #[default]
    Rest,
    Hit(Tr909FillVoiceTrigger),
    Choke,
    DiveStomp(Tr909FillVoiceTrigger),
}

impl Tr909FillStep {
    pub(super) const fn is_sounding(self) -> bool {
        self.trigger().is_some()
    }

    pub(super) const fn trigger(self) -> Option<Tr909FillVoiceTrigger> {
        match self {
            Self::Hit(trigger) | Self::DiveStomp(trigger) => Some(trigger),
            Self::Rest | Self::Choke => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Tr909FillFocusHoleProfile {
    pub(super) start_beat: f64,
    pub(super) attack_beats: f64,
    pub(super) stomp_beat: f64,
    pub(super) release_beats: f64,
    pub(super) min_gain: f32,
}

/// Named beat-domain arrangement envelope paired with a Fill recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Tr909FillFocusProfile {
    pub(super) bar_beats: f64,
    pub(super) start_beat: f64,
    pub(super) attack_beats: f64,
    pub(super) release_start_beat: f64,
    pub(super) min_gain: f32,
    pub(super) signature_hole: Option<Tr909FillFocusHoleProfile>,
}

#[derive(Clone, Copy, Debug)]
struct Tr909FillRecipe {
    steps: [Tr909FillStep; 32],
    focus: Tr909FillFocusProfile,
}

pub(super) const BASE_FILL_FOCUS: Tr909FillFocusProfile = Tr909FillFocusProfile {
    bar_beats: 4.0,
    start_beat: 2.92,
    attack_beats: 0.08,
    release_start_beat: 3.94,
    min_gain: 0.12,
    signature_hole: None,
};

const CHOKE_DIVE_STOMP_FOCUS: Tr909FillFocusProfile = Tr909FillFocusProfile {
    signature_hole: Some(Tr909FillFocusHoleProfile {
        start_beat: 3.50,
        attack_beats: 0.03,
        stomp_beat: 3.625,
        release_beats: 0.035,
        min_gain: 0.02,
    }),
    ..BASE_FILL_FOCUS
};

const LONG_CHOKE_DIVE_STOMP_FOCUS: Tr909FillFocusProfile = Tr909FillFocusProfile {
    signature_hole: Some(Tr909FillFocusHoleProfile {
        // The bed reaches silence before step 27 chokes the drum voices, then stays absent for
        // three 32nd-note slots. At 132 BPM this exposes roughly 170 ms before the late stomp.
        start_beat: 3.34,
        attack_beats: 0.035,
        stomp_beat: 3.75,
        release_beats: 0.04,
        min_gain: 0.0,
    }),
    ..BASE_FILL_FOCUS
};

const BREAK_CUT_STOMP_FOCUS: Tr909FillFocusProfile = Tr909FillFocusProfile {
    // This is a half-bar arrangement takeover, not another short closing pocket. The non-TR-909
    // bed reaches silence on beat three and stays absent until the late stomp has spoken.
    start_beat: 1.93,
    attack_beats: 0.07,
    release_start_beat: 3.90,
    min_gain: 0.0,
    signature_hole: None,
    ..BASE_FILL_FOCUS
};

const fn phrase_drive_common_steps() -> [Tr909FillStep; 32] {
    let mut steps = [Tr909FillStep::Rest; 32];
    steps[0] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(1.18, 0.0, 0.0));
    steps[8] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.88, 0.0, 0.0));
    steps[12] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.0, 1.02, 0.0));
    steps[16] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(1.0, 0.0, 0.0));
    steps[18] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.0, 0.0, 0.72));
    steps[20] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.0, 1.0, 0.0));
    steps[22] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.0, 0.0, 0.84));
    steps[24] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(1.24, 0.0, 0.0));
    steps[26] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.0, 0.82, 0.0));
    steps[27] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.0, 0.0, 0.82));
    steps
}

const fn phrase_drive_accent_ghost_steps() -> [Tr909FillStep; 32] {
    let mut steps = phrase_drive_common_steps();
    steps[28] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(1.34, 1.12, 0.0));
    steps[29] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.0, 0.42, 0.0));
    steps[30] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.0, 0.0, 0.56));
    steps
}

const fn phrase_drive_choke_dive_stomp_steps() -> [Tr909FillStep; 32] {
    let mut steps = phrase_drive_common_steps();
    steps[28] = Tr909FillStep::Choke;
    steps[29] = Tr909FillStep::DiveStomp(Tr909FillVoiceTrigger::new(1.40, 1.16, 0.0));
    steps[30] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.0, 0.0, 0.38));
    steps
}

const fn phrase_drive_long_choke_dive_stomp_steps() -> [Tr909FillStep; 32] {
    let mut steps = phrase_drive_common_steps();
    // A compact three-hit setup makes the following absence intentional rather than accidental.
    steps[24] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(1.18, 0.0, 0.20));
    steps[25] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.0, 0.46, 0.0));
    steps[26] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.86, 0.94, 0.0));
    steps[27] = Tr909FillStep::Choke;
    steps[28] = Tr909FillStep::Rest;
    steps[29] = Tr909FillStep::Rest;
    steps[30] = Tr909FillStep::DiveStomp(LONG_CHOKE_STOMP_TRIGGER);
    steps[31] = Tr909FillStep::Rest;
    steps
}

const fn phrase_drive_break_cut_stomp_steps() -> [Tr909FillStep; 32] {
    let mut steps = phrase_drive_common_steps();

    // Beat three starts a drum-owned call once the source and melodic bed have left the room.
    steps[16] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(1.12, 0.0, 0.0));
    steps[17] = Tr909FillStep::Rest;
    steps[18] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.0, 0.0, 0.68));
    steps[19] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.0, 0.78, 0.0));
    steps[20] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(1.08, 0.88, 0.0));
    steps[21] = Tr909FillStep::Rest;
    steps[22] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.0, 0.0, 0.92));
    steps[23] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.0, 1.10, 0.0));

    // Beat four answers with a compact rush, then removes all drum tails before the late stomp.
    steps[24] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(1.24, 0.0, 0.24));
    steps[25] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.0, 0.58, 0.0));
    steps[26] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.92, 1.02, 0.0));
    steps[27] = Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.0, 0.0, 0.96));
    steps[28] = Tr909FillStep::Choke;
    steps[29] = Tr909FillStep::Rest;
    steps[30] = Tr909FillStep::DiveStomp(LONG_CHOKE_STOMP_TRIGGER);
    steps[31] = Tr909FillStep::Rest;
    steps
}

const LONG_CHOKE_STOMP_TRIGGER: Tr909FillVoiceTrigger = Tr909FillVoiceTrigger::new(1.44, 1.20, 0.0);

const PHRASE_DRIVE_ACCENT_GHOST_V1: Tr909FillRecipe = Tr909FillRecipe {
    steps: phrase_drive_accent_ghost_steps(),
    focus: BASE_FILL_FOCUS,
};

const PHRASE_DRIVE_CHOKE_DIVE_STOMP_V1: Tr909FillRecipe = Tr909FillRecipe {
    steps: phrase_drive_choke_dive_stomp_steps(),
    focus: CHOKE_DIVE_STOMP_FOCUS,
};

const PHRASE_DRIVE_LONG_CHOKE_DIVE_STOMP_V2: Tr909FillRecipe = Tr909FillRecipe {
    steps: phrase_drive_long_choke_dive_stomp_steps(),
    focus: LONG_CHOKE_DIVE_STOMP_FOCUS,
};

const PHRASE_DRIVE_BREAK_CUT_STOMP_V1: Tr909FillRecipe = Tr909FillRecipe {
    steps: phrase_drive_break_cut_stomp_steps(),
    focus: BREAK_CUT_STOMP_FOCUS,
};

fn selected_phrase_drive_recipe(render: &RealtimeTr909RenderState) -> &'static Tr909FillRecipe {
    match fill_recipe_id(render) {
        Tr909FillRecipeId::PhraseDriveChokeDiveStompV1 => &PHRASE_DRIVE_CHOKE_DIVE_STOMP_V1,
        Tr909FillRecipeId::PhraseDriveLongChokeDiveStompV2 => {
            &PHRASE_DRIVE_LONG_CHOKE_DIVE_STOMP_V2
        }
        Tr909FillRecipeId::PhraseDriveBreakCutStompV1 => &PHRASE_DRIVE_BREAK_CUT_STOMP_V1,
        Tr909FillRecipeId::PhraseDriveAccentGhostV1 | Tr909FillRecipeId::GenericFillV1 => {
            &PHRASE_DRIVE_ACCENT_GHOST_V1
        }
    }
}

pub(super) fn fill_recipe_id(render: &RealtimeTr909RenderState) -> Tr909FillRecipeId {
    select_tr909_fill_recipe_id(render.pattern_adoption, render.phrase_variation)
}

pub(super) fn fill_focus_profile(render: &RealtimeTr909RenderState) -> Tr909FillFocusProfile {
    if matches!(
        render.phrase_variation,
        Some(Tr909PhraseVariation::PhraseDrive)
    ) {
        selected_phrase_drive_recipe(render).focus
    } else {
        BASE_FILL_FOCUS
    }
}

/// Project an absolute Session transport position onto the confirmed source-bar phase.
///
/// The source anchor is already confirmed and stored in the product spine before it reaches this
/// callback-safe projection. Missing or non-finite derived anchors preserve the legacy zero-phase
/// behavior rather than inventing a new timing truth in the audio crate.
pub(super) fn fill_bar_aligned_position_beats(
    render: &RealtimeTr909RenderState,
    position_beats: f64,
) -> f64 {
    render
        .source_bar_grid_anchor_position_beats
        .filter(|anchor| anchor.is_finite())
        .map_or(position_beats, |anchor| position_beats - anchor)
}

pub(super) fn fill_step(
    render: &RealtimeTr909RenderState,
    subdivision: u32,
    step: i64,
) -> Tr909FillStep {
    let subdivision = i64::from(subdivision).max(1);
    let step_in_bar = step.rem_euclid(subdivision * 4);
    if matches!(
        render.phrase_variation,
        Some(Tr909PhraseVariation::PhraseDrive)
    ) && subdivision == 8
    {
        return selected_phrase_drive_recipe(render).steps[step_in_bar as usize];
    }
    generic_fill_step(render, subdivision, step_in_bar)
}

fn generic_fill_step(
    render: &RealtimeTr909RenderState,
    subdivision: i64,
    step_in_bar: i64,
) -> Tr909FillStep {
    let beat_in_bar = step_in_bar / subdivision;
    let step_in_beat = step_in_bar % subdivision;
    let half_beat = subdivision / 2;
    let backbone = match render.pattern_adoption {
        Some(Tr909PatternAdoption::SupportPulse) => step_in_beat == 0,
        Some(Tr909PatternAdoption::TakeoverGrid) => {
            step_in_beat == 0 || step_in_beat == half_beat || step_in_beat + 1 == subdivision
        }
        Some(Tr909PatternAdoption::MainlineDrive) | None => {
            step_in_beat == 0 || step_in_beat == half_beat
        }
    };
    let pickup = match render.phrase_variation {
        Some(Tr909PhraseVariation::PhraseDrive) => false,
        Some(Tr909PhraseVariation::PhraseRelease) => beat_in_bar == 3 && step_in_beat >= half_beat,
        Some(Tr909PhraseVariation::PhraseAnchor | Tr909PhraseVariation::PhraseLift) | None => {
            beat_in_bar == 3
        }
    };
    if !backbone && !pickup {
        return Tr909FillStep::Rest;
    }

    let trigger = if step_in_beat == 0 {
        Tr909FillVoiceTrigger::new(
            if beat_in_bar.rem_euclid(2) == 0 {
                1.0
            } else {
                0.82
            },
            if beat_in_bar == 2 { 0.58 } else { 0.0 },
            0.12,
        )
    } else if step_in_beat == half_beat {
        Tr909FillVoiceTrigger::new(0.0, 0.92, 0.16)
    } else {
        Tr909FillVoiceTrigger::new(
            0.0,
            if beat_in_bar == 3 && step_in_beat + 1 == subdivision {
                1.08
            } else {
                0.0
            },
            0.72,
        )
    };
    Tr909FillStep::Hit(trigger)
}

fn scaled_fill_trigger(
    render: &RealtimeTr909RenderState,
    mut trigger: Tr909FillVoiceTrigger,
    live_slam: f32,
) -> Tr909FillVoiceTrigger {
    let pressure = render.slam_intensity.clamp(0.0, 1.0);
    trigger.kick *= (0.88 + pressure * 0.24) * (1.0 + live_slam * 0.45);
    trigger.snare *= (0.92 + pressure * 0.16) * (1.0 + live_slam * 0.12);
    trigger.hat *= (1.02 - pressure * 0.10) * (1.0 - live_slam * 0.35);
    trigger
}

pub(super) fn prepared_fill_step(
    render: &RealtimeTr909RenderState,
    subdivision: u32,
    step: i64,
    live_slam: f32,
) -> Tr909FillStep {
    match fill_step(render, subdivision, step) {
        Tr909FillStep::Hit(trigger) => {
            Tr909FillStep::Hit(scaled_fill_trigger(render, trigger, live_slam))
        }
        Tr909FillStep::DiveStomp(trigger) => {
            Tr909FillStep::DiveStomp(scaled_fill_trigger(render, trigger, live_slam))
        }
        Tr909FillStep::Choke => Tr909FillStep::Choke,
        Tr909FillStep::Rest => Tr909FillStep::Rest,
    }
}

#[cfg(test)]
mod tests {
    use crate::tr909::{Tr909RenderMode, Tr909RenderRouting};

    use super::*;

    fn phrase_drive_render(pattern_adoption: Tr909PatternAdoption) -> RealtimeTr909RenderState {
        RealtimeTr909RenderState {
            mode: Tr909RenderMode::Fill,
            routing: Tr909RenderRouting::DrumBusSupport,
            source_support_profile: None,
            source_support_context: None,
            pattern_adoption: Some(pattern_adoption),
            phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
            takeover_profile: None,
            drum_bus_level: 0.80,
            slam_enabled: false,
            slam_intensity: 0.66,
            is_transport_running: true,
            tempo_bpm: 132.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        }
    }

    #[test]
    fn phrase_drive_break_cut_preserves_context_then_owns_a_distinct_half_bar() {
        let accent_ghost = phrase_drive_render(Tr909PatternAdoption::SupportPulse);
        let break_cut_stomp = phrase_drive_render(Tr909PatternAdoption::MainlineDrive);

        assert_eq!(
            fill_recipe_id(&accent_ghost),
            Tr909FillRecipeId::PhraseDriveAccentGhostV1
        );
        assert_eq!(
            fill_recipe_id(&break_cut_stomp),
            Tr909FillRecipeId::PhraseDriveBreakCutStompV1
        );
        for step in 0..16 {
            assert_eq!(
                fill_step(&accent_ghost, 8, step),
                fill_step(&break_cut_stomp, 8, step),
                "recipe prefix drifted at step {step}"
            );
        }
        assert!(
            (16..32).any(|step| {
                fill_step(&accent_ghost, 8, step) != fill_step(&break_cut_stomp, 8, step)
            }),
            "the takeover half must not collapse to the support control"
        );

        assert!(matches!(
            fill_step(&accent_ghost, 8, 28),
            Tr909FillStep::Hit(_)
        ));
        assert!(matches!(
            fill_step(&break_cut_stomp, 8, 28),
            Tr909FillStep::Choke
        ));
        assert!(matches!(
            fill_step(&break_cut_stomp, 8, 30),
            Tr909FillStep::DiveStomp(_)
        ));
        for step in [29, 31] {
            assert_eq!(fill_step(&break_cut_stomp, 8, step), Tr909FillStep::Rest);
        }
    }

    #[test]
    fn historical_candidate_9_recipe_remains_a_frozen_review_control() {
        let recipe = PHRASE_DRIVE_CHOKE_DIVE_STOMP_V1;

        assert!(matches!(recipe.steps[28], Tr909FillStep::Choke));
        assert!(matches!(recipe.steps[29], Tr909FillStep::DiveStomp(_)));
        assert_eq!(
            recipe.steps[30],
            Tr909FillStep::Hit(Tr909FillVoiceTrigger::new(0.0, 0.0, 0.38))
        );
        assert_eq!(recipe.steps[31], Tr909FillStep::Rest);
        let hole = recipe.focus.signature_hole.expect("historical focus hole");
        assert_eq!(hole.start_beat, 3.50);
        assert_eq!(hole.stomp_beat, 3.625);
        assert_eq!(hole.min_gain, 0.02);
    }

    #[test]
    fn historical_candidate_10_recipe_remains_a_frozen_review_control() {
        let recipe = PHRASE_DRIVE_LONG_CHOKE_DIVE_STOMP_V2;
        let sounding_per_beat: Vec<usize> = (0..4)
            .map(|beat| {
                recipe.steps[beat * 8..(beat + 1) * 8]
                    .iter()
                    .filter(|step| step.is_sounding())
                    .count()
            })
            .collect();

        assert_eq!(sounding_per_beat, [1, 2, 4, 4]);
        assert!(matches!(recipe.steps[24], Tr909FillStep::Hit(_)));
        assert!(matches!(recipe.steps[25], Tr909FillStep::Hit(_)));
        assert!(matches!(recipe.steps[26], Tr909FillStep::Hit(_)));
        assert_eq!(recipe.steps[27], Tr909FillStep::Choke);
        assert_eq!(recipe.steps[28], Tr909FillStep::Rest);
        assert_eq!(recipe.steps[29], Tr909FillStep::Rest);
        assert!(matches!(recipe.steps[30], Tr909FillStep::DiveStomp(_)));
        assert_eq!(recipe.steps[31], Tr909FillStep::Rest);

        let hole = recipe.focus.signature_hole.expect("historical focus hole");
        assert_eq!(hole.start_beat, 3.34);
        assert_eq!(hole.stomp_beat, 3.75);
        assert_eq!(hole.min_gain, 0.0);
    }

    #[test]
    fn break_cut_stomp_recipe_escalates_during_its_drum_owned_half_bar() {
        let render = phrase_drive_render(Tr909PatternAdoption::MainlineDrive);
        let sounding_per_beat: Vec<usize> = (0..4)
            .map(|beat| {
                (beat * 8..(beat + 1) * 8)
                    .filter(|step| fill_step(&render, 8, i64::from(*step)).is_sounding())
                    .count()
            })
            .collect();

        assert_eq!(sounding_per_beat, [1, 2, 6, 5]);
        for step in [28, 29] {
            assert!(!fill_step(&render, 8, step).is_sounding());
        }
        assert!(fill_step(&render, 8, 30).is_sounding());
    }

    #[test]
    fn fill_focus_landmarks_come_from_the_selected_recipe() {
        let accent_ghost = phrase_drive_render(Tr909PatternAdoption::SupportPulse);
        let break_cut_stomp = phrase_drive_render(Tr909PatternAdoption::MainlineDrive);

        assert_eq!(fill_focus_profile(&accent_ghost), BASE_FILL_FOCUS);
        let profile = fill_focus_profile(&break_cut_stomp);
        assert_eq!(profile.start_beat, 1.93);
        assert_eq!(profile.attack_beats, 0.07);
        assert_eq!(profile.release_start_beat, 3.90);
        assert_eq!(profile.min_gain, 0.0);
        assert_eq!(profile.signature_hole, None);
    }

    #[test]
    fn non_phrase_drive_fill_uses_the_generic_recipe_contract() {
        let mut render = phrase_drive_render(Tr909PatternAdoption::MainlineDrive);
        render.phrase_variation = Some(Tr909PhraseVariation::PhraseLift);

        assert_eq!(fill_recipe_id(&render), Tr909FillRecipeId::GenericFillV1);
        assert_eq!(fill_focus_profile(&render), BASE_FILL_FOCUS);
        assert!(matches!(fill_step(&render, 8, 0), Tr909FillStep::Hit(_)));
        assert_eq!(fill_step(&render, 8, 1), Tr909FillStep::Rest);
    }
}
