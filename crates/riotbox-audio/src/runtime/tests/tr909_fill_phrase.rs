use crate::runtime::shared_transport_tr909::RealtimeTr909RenderState;
use crate::runtime::tests::tr909_fill_fixtures::{
    cursor_20_phrase_drive_fill_render, test_fill_recipe_trigger,
};
use crate::runtime::tr909_fill_recipe;
use crate::runtime::w30_tr909_signal_helpers::{render_subdivision, should_trigger_step};
use crate::runtime::{render_tr909_offline, signal_delta_metrics, signal_metrics};
use crate::tr909::{
    Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
    Tr909RenderState,
};

#[test]
fn cursor_20_phrase_drive_fill_builds_to_a_clear_close_against_break_reinforcement() {
    let break_reinforce = Tr909RenderState {
        mode: Tr909RenderMode::BreakReinforce,
        routing: Tr909RenderRouting::DrumBusSupport,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseDriveHardCut),
        drum_bus_level: 0.799_204_35,
        slam_enabled: false,
        slam_intensity: 0.659_204_36,
        is_transport_running: true,
        tempo_bpm: 131.878,
        position_beats: 20.0,
        source_bar_grid_anchor_position_beats: None,
        ..Tr909RenderState::default()
    };
    let fill = Tr909RenderState {
        mode: Tr909RenderMode::Fill,
        ..break_reinforce.clone()
    };
    let bar_frames = (48_000.0_f32 * 60.0 / 131.878 * 4.0).round() as usize;

    let before = render_tr909_offline(&break_reinforce, 48_000, 2, bar_frames);
    let after = render_tr909_offline(&fill, 48_000, 2, bar_frames);
    let delta = signal_delta_metrics(&before, &after);
    let beat_samples = before.len() / 4;
    let first_half_delta =
        signal_delta_metrics(&before[..beat_samples * 2], &after[..beat_samples * 2]);
    let takeover_half_delta =
        signal_delta_metrics(&before[beat_samples * 2..], &after[beat_samples * 2..]);
    let last_beat_delta =
        signal_delta_metrics(&before[beat_samples * 3..], &after[beat_samples * 3..]);
    let after_metrics = signal_metrics(&after);

    assert!(
        delta.rms > 0.02 && delta.peak_abs > 0.12,
        "fill stayed too close to break reinforcement: {delta:?}"
    );
    assert!(
        takeover_half_delta.rms > first_half_delta.rms * 1.15
            && last_beat_delta.peak_abs > first_half_delta.peak_abs * 1.25
            && last_beat_delta.silence_ratio > 0.08,
        "live PhraseDrive fill did not escalate into a decisive drum-owned half-bar and choke: first_half={first_half_delta:?} takeover_half={takeover_half_delta:?} last={last_beat_delta:?}"
    );
    assert_eq!(
        after_metrics.clip_count, 0,
        "fill clipped: {after_metrics:?}"
    );
}

#[test]
fn confirmed_source_bar_anchor_preserves_the_complete_fill_order_on_an_offset_grid() {
    let zero_phase = Tr909RenderState {
        mode: Tr909RenderMode::Fill,
        routing: Tr909RenderRouting::DrumBusSupport,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseDriveHardCut),
        drum_bus_level: 0.799_204_35,
        slam_enabled: false,
        slam_intensity: 0.659_204_36,
        is_transport_running: true,
        tempo_bpm: 132.0,
        position_beats: 20.0,
        source_bar_grid_anchor_position_beats: None,
        ..Tr909RenderState::default()
    };
    let source_aligned = Tr909RenderState {
        position_beats: 23.0,
        source_bar_grid_anchor_position_beats: Some(3.0),
        ..zero_phase.clone()
    };
    let wrong_zero_phase = Tr909RenderState {
        source_bar_grid_anchor_position_beats: None,
        ..source_aligned.clone()
    };
    let bar_frames = (48_000.0_f32 * 60.0 / 132.0 * 4.0).round() as usize;

    let expected = render_tr909_offline(&zero_phase, 48_000, 2, bar_frames);
    let aligned = render_tr909_offline(&source_aligned, 48_000, 2, bar_frames);
    let misordered = render_tr909_offline(&wrong_zero_phase, 48_000, 2, bar_frames);

    assert_eq!(
        aligned, expected,
        "confirmed source phase did not preserve beat-1 -> build -> payoff recipe order"
    );
    assert_ne!(
        misordered, expected,
        "the regression control unexpectedly hid the offset-grid recipe rotation"
    );
}

#[test]
fn cursor_20_phrase_drive_fill_builds_then_takes_over_for_a_choke_to_stomp_close() {
    let fill = RealtimeTr909RenderState {
        mode: Tr909RenderMode::Fill,
        routing: Tr909RenderRouting::DrumBusSupport,
        source_support_profile: None,
        source_support_context: None,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseDriveHardCut),
        takeover_profile: None,
        drum_bus_level: 0.799_204_35,
        slam_enabled: false,
        slam_intensity: 0.659_204_36,
        is_transport_running: true,
        tempo_bpm: 131.878,
        position_beats: 20.0,
        source_bar_grid_anchor_position_beats: None,
    };
    let break_reinforce = RealtimeTr909RenderState {
        mode: Tr909RenderMode::BreakReinforce,
        ..fill
    };
    let fill_triggered = (0_i64..32)
        .filter(|step| should_trigger_step(&fill, *step))
        .collect::<Vec<_>>();
    let break_triggered = (0_i64..16)
        .filter(|step| should_trigger_step(&break_reinforce, *step))
        .collect::<Vec<_>>();
    let fill_hits_per_beat = (0_i64..4)
        .map(|beat| {
            fill_triggered
                .iter()
                .filter(|step| beat * 8 <= **step && **step < (beat + 1) * 8)
                .count()
        })
        .collect::<Vec<_>>();
    let break_hits_per_beat = (0_i64..4)
        .map(|beat| {
            break_triggered
                .iter()
                .filter(|step| beat * 4 <= **step && **step < (beat + 1) * 4)
                .count()
        })
        .collect::<Vec<_>>();

    assert_eq!(render_subdivision(&fill), 8);
    assert_eq!(fill_triggered, vec![0, 8, 12, 16, 18, 19, 20, 22, 23, 30]);
    assert_eq!(fill_hits_per_beat, [1, 2, 6, 1]);
    assert_eq!(render_subdivision(&break_reinforce), 4);
    assert_eq!(break_triggered, (0_i64..16).collect::<Vec<_>>());
    assert_eq!(break_hits_per_beat, [4, 4, 4, 4]);
    assert_eq!(
        tr909_fill_recipe::fill_step(&fill, render_subdivision(&fill), 24),
        tr909_fill_recipe::Tr909FillStep::Choke
    );
    for step in 25..30 {
        assert_eq!(
            tr909_fill_recipe::fill_step(&fill, render_subdivision(&fill), step),
            tr909_fill_recipe::Tr909FillStep::Rest
        );
    }
    assert!(matches!(
        tr909_fill_recipe::fill_step(&fill, render_subdivision(&fill), 30),
        tr909_fill_recipe::Tr909FillStep::DiveStomp(_)
    ));
}

#[test]
fn phrase_drive_fill_assigns_the_contour_to_distinct_drum_owners() {
    let render = cursor_20_phrase_drive_fill_render();
    let contour_slots = [
        0_i64, 8, 12, 16, 18, 19, 20, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
    ];
    let triggered = contour_slots
        .into_iter()
        .map(|step| (step, test_fill_recipe_trigger(&render, step)))
        .collect::<Vec<_>>();
    let owner_map = triggered
        .iter()
        .map(|(step, trigger)| {
            (
                *step,
                trigger.kick > 0.0,
                trigger.snare > 0.0,
                trigger.hat > 0.0,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        owner_map,
        vec![
            (0, true, false, false),
            (8, true, false, false),
            (12, false, true, false),
            (16, true, false, false),
            (18, false, false, true),
            (19, false, true, false),
            (20, true, true, false),
            (22, false, false, true),
            (23, false, true, false),
            (24, false, false, false),
            (25, false, false, false),
            (26, false, false, false),
            (27, false, false, false),
            (28, false, false, false),
            (29, false, false, false),
            (30, true, true, false),
            (31, false, false, false),
        ]
    );
    for (step, trigger) in &triggered {
        let has_owner = trigger.kick > 0.0 || trigger.snare > 0.0 || trigger.hat > 0.0;
        assert_eq!(
            should_trigger_step(&render, *step),
            has_owner,
            "trigger policy and audible owner map diverged at step {step}"
        );
    }
    let sounding_events_per_beat = (0_i64..4)
        .map(|beat| {
            triggered
                .iter()
                .filter(|(step, trigger)| {
                    beat * 8 <= *step
                        && *step < (beat + 1) * 8
                        && (trigger.kick > 0.0 || trigger.snare > 0.0 || trigger.hat > 0.0)
                })
                .count()
        })
        .collect::<Vec<_>>();
    assert_eq!(sounding_events_per_beat, [1, 2, 6, 1]);
    assert!(
        (24_i64..30).all(|step| !should_trigger_step(&render, step)),
        "the final beat must reserve a perceptible choke/dropout before the payoff"
    );

    let setup_kick = test_fill_recipe_trigger(&render, 20);
    let setup_snare = test_fill_recipe_trigger(&render, 23);
    let payoff = test_fill_recipe_trigger(&render, 30);
    assert!(
        setup_kick.kick > 0.0 && setup_kick.snare > 0.0 && setup_snare.snare > 0.0,
        "the beat-three call must announce the destructive close"
    );
    assert!(
        payoff.kick > setup_kick.kick && payoff.snare > setup_snare.snare,
        "the late kick+snare dive-stomp must own the final-beat payoff"
    );
    assert_eq!(
        tr909_fill_recipe::fill_step(&render, render_subdivision(&render), 24),
        tr909_fill_recipe::Tr909FillStep::Choke
    );
    assert!(matches!(
        tr909_fill_recipe::fill_step(&render, render_subdivision(&render), 30),
        tr909_fill_recipe::Tr909FillStep::DiveStomp(_)
    ));
}

#[test]
fn phrase_drive_signature_is_scoped_to_the_mainline_golden_path() {
    let render = RealtimeTr909RenderState {
        pattern_adoption: Some(Tr909PatternAdoption::TakeoverGrid),
        ..cursor_20_phrase_drive_fill_render()
    };
    let old_payoff = test_fill_recipe_trigger(&render, 28);
    let old_ghost_snare = test_fill_recipe_trigger(&render, 29);
    let old_ghost_hat = test_fill_recipe_trigger(&render, 30);

    assert!(old_payoff.kick > 0.0 && old_payoff.snare > 0.0);
    assert!(old_ghost_snare.snare > 0.0);
    assert!(old_ghost_hat.hat > 0.0);
    for step in [28_i64, 29, 30] {
        assert!(should_trigger_step(&render, step));
        assert!(matches!(
            tr909_fill_recipe::fill_step(&render, render_subdivision(&render), step),
            tr909_fill_recipe::Tr909FillStep::Hit(_)
        ));
    }
}
