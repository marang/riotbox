use super::report_model::PackCase;
use super::report_model::RenderPair;
use riotbox_audio::mc202::Mc202ContourHint;
use riotbox_audio::mc202::Mc202HookResponse;
use riotbox_audio::mc202::Mc202PhraseShape;
use riotbox_audio::mc202::Mc202RenderMode;
use riotbox_audio::mc202::Mc202RenderRouting;
use riotbox_audio::mc202::Mc202RenderState;
use riotbox_audio::mc202::Mc202SourcePhraseRenderPlan;
use riotbox_audio::tr909::Tr909PatternAdoption;
use riotbox_audio::tr909::Tr909PhraseVariation;
use riotbox_audio::tr909::Tr909RenderMode;
use riotbox_audio::tr909::Tr909RenderRouting;
use riotbox_audio::tr909::Tr909RenderState;
use riotbox_audio::tr909::Tr909SourceSupportContext;
use riotbox_audio::tr909::Tr909SourceSupportProfile;
use riotbox_audio::tr909::Tr909TakeoverRenderProfile;

pub(super) fn pack_cases() -> Vec<PackCase> {
    vec![
        PackCase {
            id: "tr909-support-to-fill",
            title: "TR-909 support -> fill",
            recipe_refs: "Recipe 2, Recipe 7",
            baseline_label: "steady source support",
            candidate_label: "fill with mainline drive",
            render_pair: RenderPair::Tr909 {
                baseline: tr909_support_state(
                    Tr909SourceSupportProfile::SteadyPulse,
                    Tr909SourceSupportContext::TransportBar,
                    Tr909PatternAdoption::SupportPulse,
                    Tr909PhraseVariation::PhraseAnchor,
                ),
                candidate: Tr909RenderState {
                    mode: Tr909RenderMode::Fill,
                    routing: Tr909RenderRouting::DrumBusSupport,
                    pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
                    phrase_variation: Some(Tr909PhraseVariation::PhraseLift),
                    drum_bus_level: 0.82,
                    is_transport_running: true,
                    tempo_bpm: 128.0,
                    position_beats: 32.0,
                    ..Tr909RenderState::default()
                },
            },
            min_rms_delta: 0.001,
            min_signal_delta_rms: 0.001,
            note: "The fill candidate should be busier and more assertive than steady support.",
        },
        PackCase {
            id: "tr909-support-to-takeover",
            title: "TR-909 support -> takeover",
            recipe_refs: "Recipe 2",
            baseline_label: "steady source support",
            candidate_label: "controlled phrase takeover",
            render_pair: RenderPair::Tr909 {
                baseline: tr909_support_state(
                    Tr909SourceSupportProfile::BreakLift,
                    Tr909SourceSupportContext::TransportBar,
                    Tr909PatternAdoption::SupportPulse,
                    Tr909PhraseVariation::PhraseAnchor,
                ),
                candidate: Tr909RenderState {
                    mode: Tr909RenderMode::Takeover,
                    routing: Tr909RenderRouting::DrumBusTakeover,
                    pattern_adoption: Some(Tr909PatternAdoption::TakeoverGrid),
                    phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
                    takeover_profile: Some(Tr909TakeoverRenderProfile::ControlledPhrase),
                    drum_bus_level: 0.86,
                    slam_intensity: 0.3,
                    is_transport_running: true,
                    tempo_bpm: 128.0,
                    position_beats: 32.0,
                    ..Tr909RenderState::default()
                },
            },
            min_rms_delta: 0.004,
            min_signal_delta_rms: 0.004,
            note: "The takeover candidate should be more forward than support without implying a finished performance mix.",
        },
        PackCase {
            id: "scene-transport-to-target-support",
            title: "Scene transport-bar support -> scene-target support",
            recipe_refs: "Recipe 10",
            baseline_label: "transport-bar support",
            candidate_label: "scene-target support accent",
            render_pair: RenderPair::Tr909 {
                baseline: tr909_support_state(
                    Tr909SourceSupportProfile::BreakLift,
                    Tr909SourceSupportContext::TransportBar,
                    Tr909PatternAdoption::SupportPulse,
                    Tr909PhraseVariation::PhraseAnchor,
                ),
                candidate: tr909_support_state(
                    Tr909SourceSupportProfile::BreakLift,
                    Tr909SourceSupportContext::SceneTarget,
                    Tr909PatternAdoption::SupportPulse,
                    Tr909PhraseVariation::PhraseAnchor,
                ),
            },
            min_rms_delta: 0.00005,
            min_signal_delta_rms: 0.00005,
            note: "The Scene-target candidate is intentionally subtle; it proves the current TR-909 support-accent seam, not a finished Scene transition engine.",
        },
        PackCase {
            id: "mc202-touch-low-to-high",
            title: "MC-202 touch low -> high",
            recipe_refs: "Recipe 2",
            baseline_label: "follower low touch",
            candidate_label: "follower high touch",
            render_pair: RenderPair::Mc202 {
                baseline: mc202_state(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::FollowerDrive,
                    0.12,
                ),
                candidate: mc202_state(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::FollowerDrive,
                    0.92,
                ),
            },
            min_rms_delta: 0.005,
            min_signal_delta_rms: 0.0055,
            note: "This proves the `<` / `>` touch gesture changes the same MC-202 phrase energy rather than only changing UI state.",
        },
        PackCase {
            id: "mc202-follower-to-pressure",
            title: "MC-202 follower -> pressure",
            recipe_refs: "Recipe 2",
            baseline_label: "follower drive",
            candidate_label: "pressure cell",
            render_pair: RenderPair::Mc202 {
                baseline: mc202_state(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::FollowerDrive,
                    0.78,
                ),
                candidate: mc202_state(
                    Mc202RenderMode::Pressure,
                    Mc202PhraseShape::PressureCell,
                    0.84,
                ),
            },
            min_rms_delta: 0.0001,
            min_signal_delta_rms: 0.004,
            note: "This proves the `P` pressure gesture changes a source-backed MC-202 render plan instead of relying on a primitive pattern.",
        },
        PackCase {
            id: "mc202-follower-to-instigator",
            title: "MC-202 follower -> instigator",
            recipe_refs: "Recipe 2",
            baseline_label: "follower drive",
            candidate_label: "instigator spike",
            render_pair: RenderPair::Mc202 {
                baseline: mc202_state(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::FollowerDrive,
                    0.78,
                ),
                candidate: mc202_state(
                    Mc202RenderMode::Instigator,
                    Mc202PhraseShape::InstigatorSpike,
                    0.90,
                ),
            },
            min_rms_delta: 0.0001,
            min_signal_delta_rms: 0.008,
            note: "This proves the `I` instigate gesture changes a source-backed MC-202 render plan toward a sharper high-register shove.",
        },
        PackCase {
            id: "mc202-follower-to-mutated-drive",
            title: "MC-202 follower -> mutated drive",
            recipe_refs: "Recipe 2",
            baseline_label: "follower drive",
            candidate_label: "mutated drive",
            render_pair: RenderPair::Mc202 {
                baseline: mc202_state(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::FollowerDrive,
                    0.78,
                ),
                candidate: mc202_state(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::MutatedDrive,
                    0.88,
                ),
            },
            min_rms_delta: 0.0001,
            min_signal_delta_rms: 0.005,
            note: "This proves the `G` phrase mutation gesture produces a different source-backed rendered phrase, not an identical placeholder tone.",
        },
        PackCase {
            id: "mc202-neutral-to-lift-contour",
            title: "MC-202 neutral -> lift contour",
            recipe_refs: "Recipe 2",
            baseline_label: "follower neutral contour",
            candidate_label: "follower lift contour",
            render_pair: RenderPair::Mc202 {
                baseline: mc202_state_with_contour(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::FollowerDrive,
                    0.78,
                    Mc202ContourHint::Neutral,
                ),
                candidate: mc202_state_with_contour(
                    Mc202RenderMode::Follower,
                    Mc202PhraseShape::FollowerDrive,
                    0.78,
                    Mc202ContourHint::Lift,
                ),
            },
            min_rms_delta: 0.0,
            min_signal_delta_rms: 0.004,
            note: "This proves the source-section contour hint changes the same MC-202 role at the render seam instead of only changing diagnostics.",
        },
    ]
}

fn tr909_support_state(
    profile: Tr909SourceSupportProfile,
    context: Tr909SourceSupportContext,
    adoption: Tr909PatternAdoption,
    variation: Tr909PhraseVariation,
) -> Tr909RenderState {
    Tr909RenderState {
        mode: Tr909RenderMode::SourceSupport,
        routing: Tr909RenderRouting::DrumBusSupport,
        source_support_profile: Some(profile),
        source_support_context: Some(context),
        pattern_adoption: Some(adoption),
        phrase_variation: Some(variation),
        drum_bus_level: 0.72,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
        current_scene_id: (context == Tr909SourceSupportContext::SceneTarget)
            .then(|| "scene-02-break".into()),
        ..Tr909RenderState::default()
    }
}

pub(super) fn mc202_state(
    mode: Mc202RenderMode,
    shape: Mc202PhraseShape,
    touch: f32,
) -> Mc202RenderState {
    mc202_state_with_contour(mode, shape, touch, Mc202ContourHint::Neutral)
}

fn mc202_state_with_contour(
    mode: Mc202RenderMode,
    shape: Mc202PhraseShape,
    touch: f32,
    contour_hint: Mc202ContourHint,
) -> Mc202RenderState {
    mc202_state_with_policy(mode, shape, touch, contour_hint, Mc202HookResponse::Direct)
}

fn mc202_state_with_policy(
    mode: Mc202RenderMode,
    shape: Mc202PhraseShape,
    touch: f32,
    contour_hint: Mc202ContourHint,
    hook_response: Mc202HookResponse,
) -> Mc202RenderState {
    Mc202RenderState {
        mode,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape: shape,
        note_budget: mc202_note_budget_for_shape_and_hook_response(shape, hook_response),
        contour_hint,
        hook_response,
        source_phrase_plan: Some(mc202_source_phrase_fixture(shape)),
        touch,
        music_bus_level: 0.74,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
    }
}

fn mc202_source_phrase_fixture(shape: Mc202PhraseShape) -> Mc202SourcePhraseRenderPlan {
    let base = Mc202SourcePhraseRenderPlan {
        active_mask: 0b0001_0001_0010_0101,
        semitones: [-12, 0, -7, 0, 0, -5, 0, 0, -10, 0, 0, 0, -3, 0, 0, 0],
        accent_mask: 0b0001_0000_0000_0001,
        destructive_mask: 0b0000_0000_0001_0000,
        pressure: 0.68,
        contrast: 0.52,
        bass_weight: 0.70,
        stab_bite: 0.24,
        gate_snap: 0.20,
    };
    match shape {
        Mc202PhraseShape::PressureCell => Mc202SourcePhraseRenderPlan {
            active_mask: 0b0001_0001_0001_0001,
            semitones: [-19, 0, 0, 0, -17, 0, 0, 0, -22, 0, 0, 0, -14, 0, 0, 0],
            accent_mask: 0b0001_0001_0001_0001,
            pressure: 0.92,
            contrast: 0.60,
            bass_weight: 0.96,
            stab_bite: 0.10,
            gate_snap: 0.12,
            ..base
        },
        Mc202PhraseShape::InstigatorSpike => Mc202SourcePhraseRenderPlan {
            active_mask: 0b1001_0100_0010_1001,
            semitones: [-7, 0, 5, 0, 0, 12, 0, 0, -5, 0, 15, 0, 0, 0, 0, 19],
            accent_mask: 0b1000_0000_0010_0001,
            destructive_mask: 0b1000_0000_0000_0000,
            pressure: 0.56,
            contrast: 0.86,
            bass_weight: 0.22,
            stab_bite: 0.82,
            gate_snap: 0.78,
        },
        Mc202PhraseShape::MutatedDrive => Mc202SourcePhraseRenderPlan {
            active_mask: 0b1010_0101_0010_1001,
            semitones: [-12, 0, 4, 0, 7, 0, -10, 0, 0, 0, -5, 0, 9, 0, 0, -7],
            accent_mask: 0b1000_0001_0010_0001,
            pressure: 0.76,
            contrast: 0.74,
            bass_weight: 0.62,
            stab_bite: 0.40,
            gate_snap: 0.34,
            ..base
        },
        Mc202PhraseShape::RootPulse | Mc202PhraseShape::FollowerDrive => base,
    }
}

fn mc202_note_budget_for_shape_and_hook_response(
    shape: Mc202PhraseShape,
    hook_response: Mc202HookResponse,
) -> riotbox_audio::mc202::Mc202NoteBudget {
    let _ = hook_response;

    match shape {
        Mc202PhraseShape::PressureCell => riotbox_audio::mc202::Mc202NoteBudget::Sparse,
        Mc202PhraseShape::InstigatorSpike => riotbox_audio::mc202::Mc202NoteBudget::Push,
        Mc202PhraseShape::MutatedDrive => riotbox_audio::mc202::Mc202NoteBudget::Wide,
        Mc202PhraseShape::RootPulse | Mc202PhraseShape::FollowerDrive => {
            riotbox_audio::mc202::Mc202NoteBudget::Balanced
        }
    }
}
