#[test]
fn render_mix_buffer_includes_live_mc202_bass_seam() {
    let mut tr909_state = Tr909CallbackState::default();
    let mut w30_preview_state = W30PreviewCallbackState::default();
    let mut w30_resample_state = W30ResampleTapCallbackState::default();
    let mut buffer = vec![0.0_f32; 44_100 * 2];

    render_mix_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeTr909RenderState {
            mode: Tr909RenderMode::Idle,
            routing: Tr909RenderRouting::SourceOnly,
            source_support_profile: None,
            source_support_context: None,
            pattern_adoption: None,
            phrase_variation: None,
            takeover_profile: None,
            drum_bus_level: 0.0,
            slam_enabled: false,
            slam_intensity: 0.0,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            source_bar_grid_anchor_position_beats: None,
        },
        &RealtimeMc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            note_budget: Mc202NoteBudget::Balanced,
            contour_hint: Mc202ContourHint::Neutral,
            hook_response: Mc202HookResponse::Direct,
            source_phrase_plan: Some(mc202_source_plan()),
            touch: 0.78,
            music_bus_level: 0.64,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
        },
        &mut tr909_state,
        &mut W30MixRenderState {
            preview_render: &RealtimeW30PreviewRenderState {
                mode: W30PreviewRenderMode::Idle,
                routing: W30PreviewRenderRouting::Silent,
                source_profile: None,
                trigger_revision: 0,
                trigger_velocity: 0.0,
                source_window_preview: RealtimeW30PreviewSampleWindow::default(),
                pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
                music_bus_level: 0.0,
                grit_level: 0.0,
                is_transport_running: true,
                tempo_bpm: 128.0,
                position_beats: 32.0,
            },
            preview_state: &mut w30_preview_state,
            resample_render: &RealtimeW30ResampleTapState {
                mode: W30ResampleTapMode::Idle,
                routing: W30ResampleTapRouting::Silent,
                source_profile: None,
                source_audio: RealtimeW30ResampleSourceWindow::default(),
                lineage_capture_count: 0,
                generation_depth: 0,
                music_bus_level: 0.0,
                grit_level: 0.0,
                is_transport_running: true,
                tempo_bpm: 128.0,
                position_beats: 32.0,
            },
            resample_state: &mut w30_resample_state,
        },
    );

    let metrics = signal_metrics(&buffer);
    assert!(metrics.active_samples > 10_000);
    assert!(metrics.rms > 0.001);
}

#[test]
fn offline_tr909_render_produces_reviewable_metrics_for_fill() {
    let buffer = render_tr909_offline(
        &Tr909RenderState {
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
        44_100,
        2,
        44_100,
    );

    let metrics = signal_metrics(&buffer);

    assert!(metrics.active_samples > 1_000);
    assert!(metrics.peak_abs > 0.001);
    assert!(metrics.rms > 0.001);
}

#[test]
fn break_reinforce_slam_carries_physical_transient_and_body() {
    let buffer = render_tr909_offline(
        &Tr909RenderState {
            mode: Tr909RenderMode::BreakReinforce,
            routing: Tr909RenderRouting::DrumBusSupport,
            pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
            phrase_variation: Some(Tr909PhraseVariation::PhraseLift),
            drum_bus_level: 0.80,
            slam_intensity: 0.66,
            is_transport_running: true,
            tempo_bpm: 130.0,
            position_beats: 0.0,
            ..Tr909RenderState::default()
        },
        48_000,
        2,
        48_000,
    );

    let metrics = signal_metrics(&buffer);

    assert!(
        metrics.peak_abs > 0.22,
        "drum peak stayed too polite: {metrics:?}"
    );
    assert!(
        metrics.rms > 0.012,
        "drum body stayed too weak: {metrics:?}"
    );
    assert!(
        metrics.active_sample_ratio > 0.12,
        "drum envelope collapsed to click-sized support: {metrics:?}"
    );
    assert_eq!(metrics.clip_count, 0, "drum support clipped: {metrics:?}");
}

#[test]
fn fill_slam_intensity_creates_bounded_punch_not_just_pitch_drift() {
    let base = Tr909RenderState {
        mode: Tr909RenderMode::Fill,
        routing: Tr909RenderRouting::DrumBusSupport,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseLift),
        drum_bus_level: 0.82,
        slam_intensity: 0.70,
        is_transport_running: true,
        tempo_bpm: 132.0,
        position_beats: 13.0,
        ..Tr909RenderState::default()
    };
    let slammed = Tr909RenderState {
        slam_enabled: true,
        slam_intensity: 0.85,
        ..base.clone()
    };

    let before = render_tr909_offline(&base, 48_000, 2, 48_000);
    let after = render_tr909_offline(&slammed, 48_000, 2, 48_000);
    let before_metrics = signal_metrics(&before);
    let after_metrics = signal_metrics(&after);
    let delta = signal_delta_metrics(&before, &after);
    let before_low = test_band_rms(&before, 40.0, 120.0, 48_000, 2);
    let after_low = test_band_rms(&after, 40.0, 120.0, 48_000, 2);
    let before_attack = test_band_rms(&before, 2_000.0, 10_000.0, 48_000, 2);
    let after_attack = test_band_rms(&after, 2_000.0, 10_000.0, 48_000, 2);

    assert!(
        after_metrics.rms > before_metrics.rms * 1.25,
        "fill slam did not add enough body: before={before_metrics:?} after={after_metrics:?}"
    );
    assert!(
        delta.rms > 0.015 && delta.peak_abs > 0.08,
        "fill slam remained too subtle: {delta:?}"
    );
    assert!(
        after_low > before_low * 3.0,
        "fill slam did not add absolute kick body: before={before_low:.6} after={after_low:.6}"
    );
    assert!(
        after_low / after_attack > before_low / before_attack * 1.20,
        "fill slam stayed a gain-only change instead of shifting toward kick body: low before/after={before_low:.6}/{after_low:.6} attack before/after={before_attack:.6}/{after_attack:.6}"
    );
    assert_eq!(
        after_metrics.clip_count, 0,
        "fill slam clipped: {after_metrics:?}"
    );
}

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
    assert_eq!(
        fill_triggered,
        vec![0, 8, 12, 16, 18, 19, 20, 22, 23, 30]
    );
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

#[test]
fn fill_kick_snare_and_hat_tails_overlap_without_truncating_each_other() {
    let render = cursor_20_phrase_drive_fill_render();
    let mut voices = tr909_fill_voice::Tr909FillVoiceState::default();
    voices.start();
    let sample_rate = 48_000;
    let frames_per_step =
        (sample_rate as f32 * 60.0 / render.tempo_bpm / render_subdivision(&render) as f32).round()
            as usize;

    voices.trigger(
        test_fill_recipe_trigger(&render, 18),
        trigger_envelope(&render),
        18,
    );
    for _ in 0..frames_per_step {
        let _ = voices.render_sample(&render, sample_rate);
    }
    voices.trigger(
        test_fill_recipe_trigger(&render, 19),
        trigger_envelope(&render),
        19,
    );
    for _ in 0..frames_per_step {
        let _ = voices.render_sample(&render, sample_rate);
    }
    voices.trigger(
        test_fill_recipe_trigger(&render, 20),
        trigger_envelope(&render),
        20,
    );

    let mut kick = Vec::with_capacity(1_024);
    let mut snare = Vec::with_capacity(1_024);
    let mut hat = Vec::with_capacity(1_024);
    for _ in 0..1_024 {
        let sample = voices.render_sample(&render, sample_rate);
        kick.push(sample.kick);
        snare.push(sample.snare);
        hat.push(sample.hat);
    }

    let kick_rms = test_signal_rms(&kick);
    let snare_rms = test_signal_rms(&snare);
    let hat_rms = test_signal_rms(&hat);
    assert!(kick_rms > 0.10, "kick tail was truncated: {kick_rms}");
    assert!(
        snare_rms > 0.04,
        "snare tail was truncated: {snare_rms}"
    );
    assert!(
        hat_rms > 0.001,
        "the setup hat tail did not survive into the following snare/kick call: {hat_rms}"
    );
}

#[test]
fn phrase_drive_signature_chokes_cleanly_and_exposes_a_pitch_dive_with_flam() {
    let render = cursor_20_phrase_drive_fill_render();
    let sample_rate = 48_000;
    let envelope = trigger_envelope(&render);

    let mut choked_voices = tr909_fill_voice::Tr909FillVoiceState::default();
    choked_voices.start();
    choked_voices.trigger(
        test_fill_recipe_trigger(&render, 23),
        envelope,
        23,
    );
    for _ in 0..128 {
        let _ = choked_voices.render_sample(&render, sample_rate);
    }
    choked_voices.choke();
    let choked = (0..768)
        .map(|_| {
            let sample = choked_voices.render_sample(&render, sample_rate);
            sample.kick + sample.snare + sample.hat
        })
        .collect::<Vec<_>>();

    assert!(
        test_signal_rms(&choked[..96]) > 0.01,
        "choke fixture had no tail to remove"
    );
    assert!(
        choked[384..].iter().all(|sample| *sample == 0.0),
        "six-millisecond choke left a hidden audible tail"
    );
    assert!(
        (choked[288] - choked[287]).abs() < 0.02,
        "choke ended with a click-sized discontinuity"
    );

    let mut stomp_voices = tr909_fill_voice::Tr909FillVoiceState::default();
    stomp_voices.start();
    stomp_voices.trigger_dive_stomp(
        test_fill_recipe_trigger(&render, 30),
        envelope,
        30,
    );
    let samples = (0..4_096)
        .map(|_| stomp_voices.render_sample(&render, sample_rate))
        .collect::<Vec<_>>();
    let kick = samples.iter().map(|sample| sample.kick).collect::<Vec<_>>();
    let snare = samples
        .iter()
        .map(|sample| sample.snare)
        .collect::<Vec<_>>();
    let early_kick_high = test_band_rms(&kick[..720], 120.0, 300.0, sample_rate, 1);
    let late_kick_low = test_band_rms(&kick[1_200..2_640], 40.0, 90.0, sample_rate, 1);
    let late_kick_high = test_band_rms(&kick[1_200..2_640], 120.0, 300.0, sample_rate, 1);
    let first_crack = test_signal_rms(&snare[..96]);
    let pre_flam = test_signal_rms(&snare[432..528]);
    let second_crack = test_signal_rms(&snare[528..624]);

    eprintln!(
        "signature stomp early_high={early_kick_high:.6} late_low={late_kick_low:.6} late_high={late_kick_high:.6} first_crack={first_crack:.6} pre_flam={pre_flam:.6} second_crack={second_crack:.6}"
    );
    assert!(
        early_kick_high > late_kick_high * 1.35,
        "DiveStomp kick did not fall out of its high opening pitch"
    );
    assert!(
        late_kick_low > late_kick_high * 1.25,
        "DiveStomp kick did not settle into low drum body"
    );
    assert!(
        second_crack > pre_flam * 1.20 && second_crack > first_crack * 0.35,
        "delayed snare flam was not a distinct subordinate second crack"
    );
}

#[test]
fn fill_voices_have_distinct_body_and_attack_regions() {
    let render = cursor_20_phrase_drive_fill_render();
    let sample_rate = 48_000;
    let kick = render_isolated_fill_voice_component(&render, 16, sample_rate, 4_096, 0);
    let snare = render_isolated_fill_voice_component(&render, 20, sample_rate, 4_096, 1);
    let hat = render_isolated_fill_voice_component(&render, 18, sample_rate, 4_096, 2);

    let kick_low = test_band_rms(&kick, 40.0, 120.0, sample_rate, 1);
    let snare_body = test_band_rms(&snare, 120.0, 500.0, sample_rate, 1);
    let hat_attack = test_band_rms(&hat, 2_000.0, 10_000.0, sample_rate, 1);
    let kick_attack = test_band_rms(&kick, 2_000.0, 10_000.0, sample_rate, 1);
    let hat_body = test_band_rms(&hat, 120.0, 500.0, sample_rate, 1);

    assert!(
        kick_low > 0.04,
        "kick had no absolute low drum body: {kick_low}"
    );
    assert!(
        snare_body > 0.04,
        "snare had no absolute 120-500 Hz body: {snare_body}"
    );
    assert!(
        hat_attack > 0.02,
        "hat had no absolute 2-10 kHz attack: {hat_attack}"
    );
    assert!(
        kick_low > kick_attack * 2.0,
        "kick collapsed into click energy"
    );
    assert!(
        hat_attack > hat_body * 1.5,
        "hat collapsed into low-mid body"
    );
}

#[test]
fn phrase_drive_fill_adds_body_and_transient_shape_over_candidate6_composite_control() {
    let render_state = Tr909RenderState {
        mode: Tr909RenderMode::Fill,
        routing: Tr909RenderRouting::DrumBusSupport,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
        drum_bus_level: 0.799_204_35,
        slam_enabled: false,
        slam_intensity: 0.659_204_36,
        is_transport_running: true,
        tempo_bpm: 131.878,
        position_beats: 20.0,
        ..Tr909RenderState::default()
    };
    let sample_rate = 48_000;
    let channel_count = 2;
    let bar_frames = (sample_rate as f32 * 60.0 / render_state.tempo_bpm * 4.0).round() as usize;
    let candidate = render_tr909_offline(&render_state, sample_rate, channel_count, bar_frames);
    let composite =
        render_candidate6_composite_control(&render_state, sample_rate, channel_count, bar_frames);
    let last_beat_start = candidate.len() * 3 / 4;
    let candidate_last = &candidate[last_beat_start..];
    let composite_last = &composite[last_beat_start..];
    let candidate_kick = test_band_rms(candidate_last, 40.0, 120.0, sample_rate, 2);
    let candidate_body = test_band_rms(candidate_last, 120.0, 500.0, sample_rate, 2);
    let candidate_attack = test_band_rms(candidate_last, 2_000.0, 10_000.0, sample_rate, 2);
    let composite_body = test_band_rms(composite_last, 120.0, 500.0, sample_rate, 2);
    let composite_attack = test_band_rms(composite_last, 2_000.0, 10_000.0, sample_rate, 2);
    let candidate_metrics = signal_metrics(&candidate);
    let last_metrics = signal_metrics(candidate_last);
    let max_frame_delta = test_max_frame_delta(candidate_last, 2);
    let slot_sample_count = candidate_last.len() / 8;
    let slot_rms = (0..8)
        .map(|slot| {
            test_signal_rms(
                &candidate_last[slot * slot_sample_count..(slot + 1) * slot_sample_count],
            )
        })
        .collect::<Vec<_>>();

    eprintln!(
        "candidate bands low={candidate_kick:.6} body={candidate_body:.6} attack={candidate_attack:.6}; composite body={composite_body:.6} attack={composite_attack:.6}; slots={slot_rms:?}; candidate={candidate_metrics:?} last={last_metrics:?} frame_delta={max_frame_delta:.6}"
    );
    assert!(
        candidate_kick > 0.006,
        "fill had no absolute 40-120 Hz drum body"
    );
    assert!(
        candidate_body > 0.012,
        "fill had no absolute 120-500 Hz body"
    );
    assert!(
        candidate_attack > 0.006,
        "fill had no absolute 2-10 kHz attack"
    );
    assert!(
        candidate_body > composite_body * 1.20,
        "independent voices did not improve the exposed drum body: candidate={candidate_body:.6} control={composite_body:.6}"
    );
    assert!(
        candidate_attack > composite_attack * 1.05,
        "independent voices did not improve transient articulation: candidate={candidate_attack:.6} control={composite_attack:.6}"
    );
    assert!(
        last_metrics.peak_abs > 0.20,
        "final beat lacked a decisive drum transient"
    );
    assert_eq!(
        candidate_metrics.clip_count, 0,
        "fill clipped before the master bus"
    );
    assert!(
        candidate_metrics.peak_abs < master_bus_limiter_threshold(),
        "fill relied on the master limiter for headroom: {candidate_metrics:?}"
    );
    let choke_window_max = slot_rms[4..6]
        .iter()
        .copied()
        .fold(0.0_f32, f32::max);
    assert!(
        slot_rms[6] > slot_rms[0] * 1.05
            && slot_rms[6] > choke_window_max * 3.0
            && slot_rms[6] > slot_rms[7] * 1.05,
        "late DiveStomp did not dominate the setup, extended choke, and tail: {slot_rms:?}"
    );
    assert!(
        max_frame_delta < 0.70,
        "fill introduced a click-sized discontinuity: {max_frame_delta}"
    );
}

#[test]
fn phrase_drive_fill_is_sample_exact_across_callback_partitions_and_clean_bar_wraps() {
    let render = RealtimeTr909RenderState {
        tempo_bpm: 120.0,
        position_beats: 0.0,
        ..cursor_20_phrase_drive_fill_render()
    };
    let sample_rate = 48_000;
    let channel_count = 2;
    let frames_per_bar = 96_000;
    let frame_count = frames_per_bar * 2;

    let full_block = render_tr909_in_callback_blocks(
        &render,
        sample_rate,
        channel_count,
        frame_count,
        frame_count,
    );
    let callbacks_127 =
        render_tr909_in_callback_blocks(&render, sample_rate, channel_count, frame_count, 127);
    let callbacks_128 =
        render_tr909_in_callback_blocks(&render, sample_rate, channel_count, frame_count, 128);

    assert_eq!(full_block, callbacks_127);
    assert_eq!(full_block, callbacks_128);

    let fresh_second_bar = render_tr909_in_callback_blocks(
        &RealtimeTr909RenderState {
            position_beats: 4.0,
            ..render
        },
        sample_rate,
        channel_count,
        frames_per_bar,
        127,
    );
    assert_eq!(
        &callbacks_127[frames_per_bar * channel_count..],
        fresh_second_bar,
        "a continuous Fill bar restored stale voice tails at the wrap"
    );

    let boundary_sample = frames_per_bar * channel_count;
    let boundary_delta =
        (callbacks_127[boundary_sample] - callbacks_127[boundary_sample - channel_count]).abs();
    assert!(
        boundary_delta < 0.05,
        "continuous Fill bar wrap introduced a click-sized edge: {boundary_delta}"
    );
}

#[test]
fn fill_to_break_reinforce_starts_from_fresh_legacy_state_without_a_tail_edge() {
    let fill = RealtimeTr909RenderState {
        tempo_bpm: 120.0,
        position_beats: 0.0,
        ..cursor_20_phrase_drive_fill_render()
    };
    let break_reinforce = RealtimeTr909RenderState {
        mode: Tr909RenderMode::BreakReinforce,
        position_beats: 4.0,
        ..fill
    };
    let sample_rate = 48_000;
    let channel_count = 2;
    let frames_per_bar = 96_000;
    let transition_frames = 4_096;
    let mut state = Tr909CallbackState::default();
    let mut fill_bar = vec![0.0_f32; frames_per_bar * channel_count];
    render_tr909_buffer(&mut fill_bar, sample_rate, channel_count, &fill, &mut state);
    let mut transitioned = vec![0.0_f32; transition_frames * channel_count];
    render_tr909_buffer(
        &mut transitioned,
        sample_rate,
        channel_count,
        &break_reinforce,
        &mut state,
    );

    let mut fresh_state = Tr909CallbackState::default();
    let mut fresh = vec![0.0_f32; transition_frames * channel_count];
    render_tr909_buffer(
        &mut fresh,
        sample_rate,
        channel_count,
        &break_reinforce,
        &mut fresh_state,
    );
    assert_eq!(
        transitioned, fresh,
        "Fill voice or subdivision state leaked into BreakReinforce"
    );

    let boundary_delta = (transitioned[0] - fill_bar[fill_bar.len() - channel_count]).abs();
    assert!(
        boundary_delta < 0.08,
        "Fill-to-BreakReinforce introduced a click-sized edge: {boundary_delta}"
    );
    let mut boundary_window = fill_bar[fill_bar.len() - 512 * channel_count..].to_vec();
    boundary_window.extend_from_slice(&transitioned[..512 * channel_count]);
    assert!(
        test_max_frame_delta(&boundary_window, channel_count) < 0.35,
        "Fill-to-BreakReinforce boundary exceeded the local transient budget"
    );
}

#[test]
fn fill_one_subdivision_transport_jump_clears_in_flight_voices() {
    let render = RealtimeTr909RenderState {
        tempo_bpm: 120.0,
        position_beats: 0.0,
        ..cursor_20_phrase_drive_fill_render()
    };
    let sample_rate = 48_000;
    let channel_count = 2;
    let priming_frames = 512;
    let primed_position = priming_frames as f64 * 120.0 / 60.0 / sample_rate as f64;
    let jumped = RealtimeTr909RenderState {
        position_beats: primed_position + 0.125,
        ..render
    };
    let mut state = Tr909CallbackState::default();
    let mut priming = vec![0.0_f32; priming_frames * channel_count];
    render_tr909_buffer(
        &mut priming,
        sample_rate,
        channel_count,
        &render,
        &mut state,
    );
    assert!(
        signal_metrics(&priming).active_samples > 0,
        "priming Fill did not start a voice"
    );

    let mut after_jump = vec![0.0_f32; 1_024 * channel_count];
    render_tr909_buffer(
        &mut after_jump,
        sample_rate,
        channel_count,
        &jumped,
        &mut state,
    );
    let mut fresh_state = Tr909CallbackState::default();
    let mut fresh = vec![0.0_f32; after_jump.len()];
    render_tr909_buffer(
        &mut fresh,
        sample_rate,
        channel_count,
        &jumped,
        &mut fresh_state,
    );
    assert_eq!(
        after_jump, fresh,
        "an exact one-subdivision seek retained a stale Fill tail"
    );
}

#[test]
fn non_fill_modes_remain_sample_exact_with_the_legacy_composite_renderer() {
    let cases = [
        RealtimeTr909RenderState {
            mode: Tr909RenderMode::SourceSupport,
            routing: Tr909RenderRouting::DrumBusSupport,
            source_support_profile: Some(Tr909SourceSupportProfile::DropDrive),
            source_support_context: Some(Tr909SourceSupportContext::TransportBar),
            pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
            phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
            takeover_profile: None,
            drum_bus_level: 0.78,
            slam_enabled: false,
            slam_intensity: 0.35,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 8.0,
            source_bar_grid_anchor_position_beats: None,
        },
        RealtimeTr909RenderState {
            mode: Tr909RenderMode::BreakReinforce,
            routing: Tr909RenderRouting::DrumBusSupport,
            source_support_profile: None,
            source_support_context: None,
            pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
            phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
            takeover_profile: None,
            drum_bus_level: 0.82,
            slam_enabled: true,
            slam_intensity: 0.85,
            is_transport_running: true,
            tempo_bpm: 132.0,
            position_beats: 16.0,
            source_bar_grid_anchor_position_beats: None,
        },
        RealtimeTr909RenderState {
            mode: Tr909RenderMode::Takeover,
            routing: Tr909RenderRouting::DrumBusTakeover,
            source_support_profile: None,
            source_support_context: None,
            pattern_adoption: Some(Tr909PatternAdoption::TakeoverGrid),
            phrase_variation: Some(Tr909PhraseVariation::PhraseLift),
            takeover_profile: Some(Tr909TakeoverRenderProfile::ControlledPhrase),
            drum_bus_level: 0.76,
            slam_enabled: false,
            slam_intensity: 0.45,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 24.0,
            source_bar_grid_anchor_position_beats: None,
        },
    ];

    for render in cases {
        let current = render_tr909_in_callback_blocks(&render, 48_000, 2, 24_000, 24_000);
        let legacy = render_legacy_tr909_composite_control(&render, 48_000, 2, 24_000);
        assert_eq!(
            current, legacy,
            "non-Fill mode {:?} drifted from the established composite sample path",
            render.mode
        );
    }
}

#[test]
fn cursor_20_unslammed_fill_keeps_headroom_below_break_reinforcement() {
    let fill = RealtimeTr909RenderState {
        mode: Tr909RenderMode::Fill,
        routing: Tr909RenderRouting::DrumBusSupport,
        source_support_profile: None,
        source_support_context: None,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
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

    let fill_gain = render_gain(&fill);
    let break_gain = render_gain(&break_reinforce);

    assert!(
        fill_gain < break_gain,
        "the unslammed fill must not inherit the BreakReinforce pressure floor: fill={fill_gain:.6} break={break_gain:.6}"
    );
    assert!(fill_gain <= 0.46);
}

#[test]
fn explicit_break_slam_adds_low_mid_body_and_attack_without_gain_only_fallback() {
    let base = Tr909RenderState {
        mode: Tr909RenderMode::BreakReinforce,
        routing: Tr909RenderRouting::DrumBusSupport,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
        drum_bus_level: 0.82,
        slam_enabled: false,
        slam_intensity: 0.66,
        is_transport_running: true,
        tempo_bpm: 132.0,
        position_beats: 16.0,
        ..Tr909RenderState::default()
    };
    let slammed = Tr909RenderState {
        slam_enabled: true,
        slam_intensity: 0.85,
        ..base.clone()
    };

    let before = render_tr909_offline(&base, 48_000, 2, 48_000);
    let after = render_tr909_offline(&slammed, 48_000, 2, 48_000);
    let before_metrics = signal_metrics(&before);
    let after_metrics = signal_metrics(&after);
    let delta = signal_delta_metrics(&before, &after);
    let before_low_mid = tr909_low_band_rms(&before, 48_000, 2);
    let after_low_mid = tr909_low_band_rms(&after, 48_000, 2);

    assert!(
        after_low_mid > before_low_mid * 1.25,
        "break slam did not add low-mid body: before={before_low_mid:.6} after={after_low_mid:.6}"
    );
    assert!(
        delta.rms > 0.02 && delta.peak_abs > 0.12,
        "break slam remained a subtle control change: {delta:?}"
    );
    assert!(
        after_metrics.active_sample_ratio > before_metrics.active_sample_ratio * 1.10,
        "break slam remained a click instead of sustained punch: before={before_metrics:?} after={after_metrics:?}"
    );
    assert_eq!(
        after_metrics.clip_count, 0,
        "break slam clipped: {after_metrics:?}"
    );
}

#[test]
fn offline_mc202_render_stays_silent_until_source_phrase_exists() {
    let follower = render_mc202_offline(
        &Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            touch: 0.62,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        },
        44_100,
        2,
        44_100,
    );
    let answer = render_mc202_offline(
        &Mc202RenderState {
            mode: Mc202RenderMode::Answer,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::RootPulse,
            touch: 0.78,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        },
        44_100,
        2,
        44_100,
    );
    let follower_metrics = signal_metrics(&follower);
    let answer_metrics = signal_metrics(&answer);

    assert_eq!(follower_metrics.active_samples, 0);
    assert_eq!(follower_metrics.rms, 0.0);
    assert_eq!(answer_metrics.active_samples, 0);
    assert_eq!(answer_metrics.rms, 0.0);
}

#[test]
fn offline_mc202_render_produces_distinct_source_backed_instigator_metrics() {
    let follower = render_mc202_offline(
        &Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            source_phrase_plan: Some(mc202_source_plan()),
            touch: 0.78,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        },
        44_100,
        2,
        44_100,
    );
    let instigator = render_mc202_offline(
        &Mc202RenderState {
            mode: Mc202RenderMode::Instigator,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::InstigatorSpike,
            source_phrase_plan: Some(mc202_source_plan()),
            touch: 0.90,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Mc202RenderState::default()
        },
        44_100,
        2,
        44_100,
    );
    let follower_metrics = signal_metrics(&follower);
    let instigator_metrics = signal_metrics(&instigator);
    let delta_rms = (follower
        .iter()
        .zip(instigator.iter())
        .map(|(follower, instigator)| (follower - instigator).powi(2))
        .sum::<f32>()
        / follower.len() as f32)
        .sqrt();

    assert!(follower_metrics.active_samples > 10_000);
    assert!(instigator_metrics.active_samples > 8_000);
    assert!(
        delta_rms > 0.010,
        "instigator offline delta RMS {delta_rms}"
    );
}

fn mc202_source_plan() -> Mc202SourcePhraseRenderPlan {
    Mc202SourcePhraseRenderPlan {
        active_mask: 0b0001_0001_0010_0101,
        semitones: [-12, 0, -7, 0, 0, -5, 0, 0, -10, 0, 0, 0, -3, 0, 0, 0],
        accent_mask: 0b0001_0000_0000_0001,
        destructive_mask: 0b0000_0000_0001_0000,
        pressure: 0.70,
        contrast: 0.56,
        bass_weight: 0.72,
        stab_bite: 0.26,
        gate_snap: 0.22,
    }
}

#[test]
fn render_buffer_respects_zero_drum_bus_level() {
    let mut state = Tr909CallbackState::default();
    let mut buffer = [0.0_f32; 512];

    render_tr909_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeTr909RenderState {
            mode: Tr909RenderMode::BreakReinforce,
            routing: Tr909RenderRouting::DrumBusSupport,
            source_support_profile: None,
            source_support_context: None,
            pattern_adoption: None,
            phrase_variation: None,
            takeover_profile: None,
            drum_bus_level: 0.0,
            slam_enabled: false,
            slam_intensity: 0.6,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        },
        &mut state,
    );

    assert!(buffer.iter().all(|sample| sample.abs() <= f32::EPSILON));
}

#[test]
fn source_support_profiles_produce_different_peak_levels() {
    let mut steady_state = Tr909CallbackState::default();
    let mut drive_state = Tr909CallbackState::default();
    let mut steady = [0.0_f32; 512];
    let mut drive = [0.0_f32; 512];

    render_tr909_buffer(
        &mut steady,
        44_100,
        2,
        &RealtimeTr909RenderState {
            mode: Tr909RenderMode::SourceSupport,
            routing: Tr909RenderRouting::DrumBusSupport,
            source_support_profile: Some(Tr909SourceSupportProfile::SteadyPulse),
            source_support_context: Some(Tr909SourceSupportContext::TransportBar),
            pattern_adoption: Some(Tr909PatternAdoption::SupportPulse),
            phrase_variation: Some(Tr909PhraseVariation::PhraseAnchor),
            takeover_profile: None,
            drum_bus_level: 0.8,
            slam_enabled: false,
            slam_intensity: 0.35,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        },
        &mut steady_state,
    );

    render_tr909_buffer(
        &mut drive,
        44_100,
        2,
        &RealtimeTr909RenderState {
            mode: Tr909RenderMode::SourceSupport,
            routing: Tr909RenderRouting::DrumBusSupport,
            source_support_profile: Some(Tr909SourceSupportProfile::DropDrive),
            source_support_context: Some(Tr909SourceSupportContext::SceneTarget),
            pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
            phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
            takeover_profile: None,
            drum_bus_level: 0.8,
            slam_enabled: false,
            slam_intensity: 0.35,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        },
        &mut drive_state,
    );

    let steady_peak = steady
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let drive_peak = drive
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));

    assert!(drive_peak > steady_peak);
}

#[test]
fn scene_target_context_adds_bounded_support_accent() {
    let mut transport_state = Tr909CallbackState::default();
    let mut scene_state = Tr909CallbackState::default();
    let mut transport = [0.0_f32; 512];
    let mut scene_target = [0.0_f32; 512];
    let base = RealtimeTr909RenderState {
        mode: Tr909RenderMode::SourceSupport,
        routing: Tr909RenderRouting::DrumBusSupport,
        source_support_profile: Some(Tr909SourceSupportProfile::BreakLift),
        source_support_context: Some(Tr909SourceSupportContext::TransportBar),
        pattern_adoption: Some(Tr909PatternAdoption::SupportPulse),
        phrase_variation: Some(Tr909PhraseVariation::PhraseAnchor),
        takeover_profile: None,
        drum_bus_level: 0.8,
        slam_enabled: false,
        slam_intensity: 0.35,
        is_transport_running: true,
        tempo_bpm: 126.0,
        position_beats: 0.0,
        source_bar_grid_anchor_position_beats: None,
    };

    render_tr909_buffer(&mut transport, 44_100, 2, &base, &mut transport_state);

    let mut scene_render = base;
    scene_render.source_support_context = Some(Tr909SourceSupportContext::SceneTarget);
    render_tr909_buffer(
        &mut scene_target,
        44_100,
        2,
        &scene_render,
        &mut scene_state,
    );

    let transport_peak = transport
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let scene_peak = scene_target
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let transport_active = transport
        .iter()
        .filter(|sample| sample.abs() > 0.0001)
        .count();
    let scene_active = scene_target
        .iter()
        .filter(|sample| sample.abs() > 0.0001)
        .count();

    assert!(scene_peak > transport_peak);
    assert!(scene_peak < transport_peak * 1.3);
    assert!(
        scene_active.abs_diff(transport_active) <= 4,
        "scene_active={scene_active} transport_active={transport_active}"
    );
}

#[test]
fn controlled_phrase_takeover_profile_is_more_active_than_scene_lock() {
    let mut controlled_state = Tr909CallbackState::default();
    let mut lock_state = Tr909CallbackState::default();
    let mut controlled = [0.0_f32; 512];
    let mut scene_lock = [0.0_f32; 512];

    render_tr909_buffer(
        &mut controlled,
        44_100,
        2,
        &RealtimeTr909RenderState {
            mode: Tr909RenderMode::Takeover,
            routing: Tr909RenderRouting::DrumBusTakeover,
            source_support_profile: None,
            source_support_context: None,
            pattern_adoption: Some(Tr909PatternAdoption::TakeoverGrid),
            phrase_variation: Some(Tr909PhraseVariation::PhraseLift),
            takeover_profile: Some(Tr909TakeoverRenderProfile::ControlledPhrase),
            drum_bus_level: 0.8,
            slam_enabled: false,
            slam_intensity: 0.45,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        },
        &mut controlled_state,
    );

    render_tr909_buffer(
        &mut scene_lock,
        44_100,
        2,
        &RealtimeTr909RenderState {
            mode: Tr909RenderMode::Takeover,
            routing: Tr909RenderRouting::DrumBusTakeover,
            source_support_profile: None,
            source_support_context: None,
            pattern_adoption: Some(Tr909PatternAdoption::SupportPulse),
            phrase_variation: Some(Tr909PhraseVariation::PhraseAnchor),
            takeover_profile: Some(Tr909TakeoverRenderProfile::SceneLock),
            drum_bus_level: 0.8,
            slam_enabled: false,
            slam_intensity: 0.45,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        },
        &mut lock_state,
    );

    let controlled_active = controlled
        .iter()
        .filter(|sample| sample.abs() > 0.0001)
        .count();
    let scene_lock_active = scene_lock
        .iter()
        .filter(|sample| sample.abs() > 0.0001)
        .count();

    assert!(controlled_active > scene_lock_active);
}

#[test]
fn fixture_backed_tr909_audio_regressions_hold() {
    let fixtures: Vec<AudioFixtureCase> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/tr909_audio_regression.json"
    ))
    .expect("parse TR-909 audio regression fixture");

    for fixture in fixtures {
        let mut callback_state = Tr909CallbackState::default();
        let mut buffer = [0.0_f32; 512];

        render_tr909_buffer(
            &mut buffer,
            44_100,
            2,
            &fixture.render_state.to_realtime(),
            &mut callback_state,
        );

        let active_samples = buffer.iter().filter(|sample| sample.abs() > 0.0001).count();
        let peak_abs = buffer
            .iter()
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
        let sum = buffer.iter().sum::<f32>();

        assert!(
            active_samples >= fixture.expected.min_active_samples,
            "{} active sample count too low: got {active_samples}",
            fixture.name
        );
        assert!(
            active_samples <= fixture.expected.max_active_samples,
            "{} active sample count too high: got {active_samples}",
            fixture.name
        );
        assert!(
            peak_abs >= fixture.expected.min_peak_abs,
            "{} peak too low: got {peak_abs}",
            fixture.name
        );
        assert!(
            peak_abs <= fixture.expected.max_peak_abs,
            "{} peak too high: got {peak_abs}",
            fixture.name
        );
        if let Some(min_sum) = fixture.expected.min_sum {
            assert!(sum >= min_sum, "{} sum too low: got {sum}", fixture.name);
        }
        if let Some(max_sum) = fixture.expected.max_sum {
            assert!(sum <= max_sum, "{} sum too high: got {sum}", fixture.name);
        }
    }
}

fn cursor_20_phrase_drive_fill_render() -> RealtimeTr909RenderState {
    RealtimeTr909RenderState {
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
    }
}

fn test_fill_recipe_trigger(
    render: &RealtimeTr909RenderState,
    step: i64,
) -> tr909_fill_recipe::Tr909FillVoiceTrigger {
    tr909_fill_recipe::prepared_fill_step(
        render,
        render_subdivision(render),
        step,
        fill_performance_slam(render),
    )
    .trigger()
    .unwrap_or_default()
}

fn render_isolated_fill_voice_component(
    render: &RealtimeTr909RenderState,
    step: i64,
    sample_rate: u32,
    frame_count: usize,
    component: usize,
) -> Vec<f32> {
    let mut voices = tr909_fill_voice::Tr909FillVoiceState::default();
    voices.start();
    voices.trigger(
        test_fill_recipe_trigger(render, step),
        trigger_envelope(render),
        step,
    );
    (0..frame_count)
        .map(|_| {
            let sample = voices.render_sample(render, sample_rate);
            match component {
                0 => sample.kick,
                1 => sample.snare,
                _ => sample.hat,
            }
        })
        .collect()
}

fn render_tr909_in_callback_blocks(
    render: &RealtimeTr909RenderState,
    sample_rate: u32,
    channel_count: usize,
    frame_count: usize,
    callback_frame_count: usize,
) -> Vec<f32> {
    let mut state = Tr909CallbackState::default();
    let mut output = Vec::with_capacity(frame_count * channel_count);
    let beats_per_frame = f64::from(render.tempo_bpm) / 60.0 / f64::from(sample_rate);
    for frame_offset in (0..frame_count).step_by(callback_frame_count.max(1)) {
        let block_frames = (frame_count - frame_offset).min(callback_frame_count.max(1));
        let block_render = RealtimeTr909RenderState {
            position_beats: render.position_beats + frame_offset as f64 * beats_per_frame,
            ..*render
        };
        let mut block = vec![0.0_f32; block_frames * channel_count];
        render_tr909_buffer(
            &mut block,
            sample_rate,
            channel_count,
            &block_render,
            &mut state,
        );
        output.extend(block);
    }
    output
}

fn render_legacy_tr909_composite_control(
    render: &RealtimeTr909RenderState,
    sample_rate: u32,
    channel_count: usize,
    frame_count: usize,
) -> Vec<f32> {
    let mut state = Tr909CallbackState::default();
    let subdivision = render_subdivision(render);
    let current_step = (render.position_beats * f64::from(subdivision)).floor() as i64;
    state.beat_position = render.position_beats;
    state.last_step = current_step.saturating_sub(1);
    state.was_running = true;
    let beats_per_sample = f64::from(render.tempo_bpm) / 60.0 / f64::from(sample_rate.max(1));
    let mut output = vec![0.0; frame_count.saturating_mul(channel_count)];

    for frame_index in 0..frame_count {
        let step = (state.beat_position * f64::from(subdivision)).floor() as i64;
        if step != state.last_step {
            state.last_step = step;
            if should_trigger_step(render, step) {
                state.envelope = trigger_envelope(render);
                state.oscillator_hz = trigger_frequency(render, step);
                if break_performance_slam(render) > 0.0 {
                    state.oscillator_phase = 0.25;
                }
            }
        }

        let sample = if state.envelope > 0.0005 {
            let waveform = tr909_step_waveform(render, state.last_step, state.oscillator_phase);
            state.oscillator_phase =
                (state.oscillator_phase + state.oscillator_hz / sample_rate.max(1) as f32).fract();
            let rendered = waveform * state.envelope * render_gain(render);
            state.envelope *= envelope_decay(render);
            rendered
        } else {
            0.0
        };
        let base = frame_index * channel_count;
        output[base..base + channel_count].fill(sample);
        state.beat_position += beats_per_sample;
    }
    output
}

fn render_candidate6_composite_control(
    render_state: &Tr909RenderState,
    sample_rate: u32,
    channel_count: u16,
    frame_count: usize,
) -> Vec<f32> {
    let render = SharedTr909RenderState::new(render_state).snapshot();
    let channel_count = usize::from(channel_count.max(1));
    let mut state = Tr909CallbackState::default();
    let subdivision = render_subdivision(&render);
    let current_step = (render.position_beats * f64::from(subdivision)).floor() as i64;
    state.beat_position = render.position_beats;
    state.last_step = current_step.saturating_sub(1);
    state.was_running = true;
    let beats_per_sample = f64::from(render.tempo_bpm) / 60.0 / f64::from(sample_rate.max(1));
    let mut output = vec![0.0; frame_count.saturating_mul(channel_count)];

    for frame_index in 0..frame_count {
        let step = (state.beat_position * f64::from(subdivision)).floor() as i64;
        if step != state.last_step {
            state.last_step = step;
            if candidate6_should_trigger_step(&render, step) {
                state.envelope = trigger_envelope(&render);
                state.oscillator_hz = trigger_frequency(&render, step);
                state.oscillator_phase = fill_performance_slam(&render) * 0.25;
            }
        }

        let sample = if state.envelope > 0.0005 {
            let waveform = tr909_step_waveform(&render, state.last_step, state.oscillator_phase);
            state.oscillator_phase =
                (state.oscillator_phase + state.oscillator_hz / sample_rate.max(1) as f32).fract();
            let rendered = waveform * state.envelope * render_gain(&render);
            state.envelope *= envelope_decay(&render);
            rendered
        } else {
            0.0
        };
        let base = frame_index * channel_count;
        output[base..base + channel_count].fill(sample);
        state.beat_position += beats_per_sample;
    }
    output
}

fn candidate6_should_trigger_step(render: &RealtimeTr909RenderState, step: i64) -> bool {
    let subdivision = i64::from(render_subdivision(render)).max(1);
    let step_in_bar = step.rem_euclid(subdivision * 4);
    let beat_in_bar = step_in_bar / subdivision;
    let step_in_beat = step_in_bar % subdivision;
    match beat_in_bar {
        0 => step_in_beat == 0,
        1 => step_in_beat % (subdivision / 2).max(1) == 0,
        2 => step_in_beat % (subdivision / 4).max(1) == 0,
        _ => true,
    }
}

fn test_signal_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt()
}

fn test_band_rms(
    samples: &[f32],
    low_hz: f32,
    high_hz: f32,
    sample_rate: u32,
    channel_count: usize,
) -> f32 {
    if samples.is_empty() || sample_rate == 0 || channel_count == 0 {
        return 0.0;
    }
    let dt = 1.0 / sample_rate as f32;
    let low_alpha = dt / (1.0 / (std::f32::consts::TAU * low_hz.max(1.0)) + dt);
    let high_alpha = dt / (1.0 / (std::f32::consts::TAU * high_hz.max(1.0)) + dt);
    let mut low_state = vec![0.0_f32; channel_count];
    let mut high_state = vec![0.0_f32; channel_count];
    let mut energy = 0.0_f32;
    for (index, sample) in samples.iter().enumerate() {
        let channel = index % channel_count;
        low_state[channel] += low_alpha * (*sample - low_state[channel]);
        high_state[channel] += high_alpha * (*sample - high_state[channel]);
        let band = high_state[channel] - low_state[channel];
        energy += band * band;
    }
    (energy / samples.len() as f32).sqrt()
}

fn test_max_frame_delta(samples: &[f32], channel_count: usize) -> f32 {
    if channel_count == 0 {
        return 0.0;
    }
    samples
        .chunks_exact(channel_count)
        .map(|frame| frame[0])
        .collect::<Vec<_>>()
        .windows(2)
        .fold(0.0_f32, |max_delta, frame| {
            max_delta.max((frame[1] - frame[0]).abs())
        })
}
