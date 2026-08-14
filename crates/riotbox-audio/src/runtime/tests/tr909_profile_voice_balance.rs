#[test]
fn tr909_source_support_profiles_shape_distinct_drum_voice_balance() {
    let steady = render_tr909_offline(
        &tr909_source_support_state(
            Tr909SourceSupportProfile::SteadyPulse,
            Tr909PatternAdoption::SupportPulse,
            Tr909PhraseVariation::PhraseAnchor,
        ),
        44_100,
        2,
        44_100,
    );
    let break_lift = render_tr909_offline(
        &tr909_source_support_state(
            Tr909SourceSupportProfile::BreakLift,
            Tr909PatternAdoption::TakeoverGrid,
            Tr909PhraseVariation::PhraseLift,
        ),
        44_100,
        2,
        44_100,
    );
    let drop_drive = render_tr909_offline(
        &tr909_source_support_state(
            Tr909SourceSupportProfile::DropDrive,
            Tr909PatternAdoption::MainlineDrive,
            Tr909PhraseVariation::PhraseDrive,
        ),
        44_100,
        2,
        44_100,
    );

    let steady_metrics = signal_metrics(&steady);
    let break_metrics = signal_metrics(&break_lift);
    let drop_metrics = signal_metrics(&drop_drive);
    let break_delta = signal_delta_metrics(&steady, &break_lift);
    let drop_delta = signal_delta_metrics(&steady, &drop_drive);

    assert!(steady_metrics.active_samples > 1_000);
    assert!(break_delta.rms > 0.002, "break delta {break_delta:?}");
    assert!(drop_delta.rms > 0.002, "drop delta {drop_delta:?}");
    assert!(
        tr909_low_band_rms(&drop_drive, 44_100, 2)
            > tr909_low_band_rms(&break_lift, 44_100, 2) * 1.03,
        "drop profile should carry more kick/low pressure"
    );
    assert!(
        tr909_high_band_proxy_rms(&break_lift, 44_100, 2)
            > tr909_high_band_proxy_rms(&steady, 44_100, 2) * 1.08,
        "break profile should carry more snare/hat energy"
    );
    assert!(drop_metrics.rms > steady_metrics.rms);
    assert!(break_metrics.rms > steady_metrics.rms);
}

#[test]
fn tr909_fill_support_and_takeover_roles_are_audibly_distinguishable() {
    let support = render_tr909_offline(
        &tr909_source_support_state(
            Tr909SourceSupportProfile::BreakLift,
            Tr909PatternAdoption::SupportPulse,
            Tr909PhraseVariation::PhraseAnchor,
        ),
        44_100,
        2,
        44_100,
    );
    let fill = render_tr909_offline(
        &Tr909RenderState {
            mode: Tr909RenderMode::Fill,
            routing: Tr909RenderRouting::DrumBusSupport,
            pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
            phrase_variation: Some(Tr909PhraseVariation::PhraseLift),
            drum_bus_level: 0.78,
            slam_intensity: 0.34,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Tr909RenderState::default()
        },
        44_100,
        2,
        44_100,
    );
    let takeover = render_tr909_offline(
        &Tr909RenderState {
            mode: Tr909RenderMode::Takeover,
            routing: Tr909RenderRouting::DrumBusTakeover,
            pattern_adoption: Some(Tr909PatternAdoption::TakeoverGrid),
            phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
            takeover_profile: Some(Tr909TakeoverRenderProfile::ControlledPhrase),
            drum_bus_level: 0.82,
            slam_intensity: 0.42,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
            ..Tr909RenderState::default()
        },
        44_100,
        2,
        44_100,
    );

    let support_metrics = signal_metrics(&support);
    let fill_metrics = signal_metrics(&fill);
    let takeover_metrics = signal_metrics(&takeover);

    assert!(fill_metrics.onset_count >= support_metrics.onset_count);
    assert!(takeover_metrics.rms > support_metrics.rms * 1.12);
    assert!(signal_delta_metrics(&support, &fill).rms > 0.002);
    assert!(signal_delta_metrics(&fill, &takeover).rms > 0.002);
}

fn tr909_source_support_state(
    support_profile: Tr909SourceSupportProfile,
    pattern_adoption: Tr909PatternAdoption,
    phrase_variation: Tr909PhraseVariation,
) -> Tr909RenderState {
    Tr909RenderState {
        mode: Tr909RenderMode::SourceSupport,
        routing: Tr909RenderRouting::DrumBusSupport,
        source_support_profile: Some(support_profile),
        source_support_context: Some(Tr909SourceSupportContext::TransportBar),
        pattern_adoption: Some(pattern_adoption),
        phrase_variation: Some(phrase_variation),
        drum_bus_level: 0.80,
        slam_intensity: 0.28,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 32.0,
        ..Tr909RenderState::default()
    }
}

fn counter_rhythm_realtime_state(
    counter_rhythm: Option<Tr909CounterRhythm>,
) -> RealtimeTr909RenderState {
    SharedTr909RenderState::new(&Tr909RenderState {
        mode: Tr909RenderMode::BreakReinforce,
        routing: Tr909RenderRouting::DrumBusSupport,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
        counter_rhythm,
        counter_rhythm_phase: Tr909CounterRhythmPhase::SourceSupported,
        drum_bus_level: 0.80,
        slam_enabled: true,
        slam_intensity: 0.85,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 0.0,
        ..Tr909RenderState::default()
    })
    .snapshot()
}

#[test]
fn counter_rhythm_reweights_only_two_snare_hat_pairs_and_preserves_anchors_and_kick() {
    let control = counter_rhythm_realtime_state(None);
    let candidate = counter_rhythm_realtime_state(Some(Tr909CounterRhythm::EighthAnswer));
    let phase_control = RealtimeTr909RenderState {
        counter_rhythm_phase: Tr909CounterRhythmPhase::PhaseControl,
        ..candidate
    };
    let subdivision = i64::from(render_subdivision(&candidate));
    assert_eq!(subdivision, 4);

    let mut changed_multipliers = Vec::new();
    let mut phase_control_multipliers = Vec::new();
    for step in 0..subdivision * 4 {
        let control_balance = tr909_voice_balance(&control, step);
        let candidate_balance = tr909_voice_balance(&candidate, step);
        let phase_control_balance = tr909_voice_balance(&phase_control, step);
        assert_eq!(candidate_balance.kick, control_balance.kick, "step {step}");
        assert_eq!(phase_control_balance.kick, control_balance.kick, "step {step}");

        if candidate_balance.snare != control_balance.snare {
            let snare_multiplier = candidate_balance.snare / control_balance.snare;
            let hat_multiplier = candidate_balance.hat / control_balance.hat;
            assert!((snare_multiplier - hat_multiplier).abs() < 1.0e-6);
            changed_multipliers.push(snare_multiplier);
        } else {
            assert_eq!(candidate_balance.hat, control_balance.hat, "step {step}");
        }
        if phase_control_balance.snare != control_balance.snare {
            phase_control_multipliers
                .push(phase_control_balance.snare / control_balance.snare);
        } else {
            assert_eq!(phase_control_balance.hat, control_balance.hat, "step {step}");
        }
    }

    changed_multipliers.sort_by(f32::total_cmp);
    phase_control_multipliers.sort_by(f32::total_cmp);
    assert_eq!(changed_multipliers, vec![0.0, 0.0, 2.0, 2.0]);
    assert_eq!(phase_control_multipliers, changed_multipliers);
    assert_eq!(changed_multipliers.iter().sum::<f32>(), 4.0);

    for anchor in [0, subdivision * 2] {
        assert_eq!(
            tr909_voice_balance(&candidate, anchor),
            tr909_voice_balance(&control, anchor)
        );
        assert_eq!(
            tr909_voice_balance(&phase_control, anchor),
            tr909_voice_balance(&control, anchor)
        );
    }
}

#[test]
fn frozen_counter_rhythm_families_move_distinct_slots_and_are_audible_offline() {
    let eighth = counter_rhythm_realtime_state(Some(Tr909CounterRhythm::EighthAnswer));
    let late = counter_rhythm_realtime_state(Some(Tr909CounterRhythm::LateSixteenthPickup));
    let subdivision = i64::from(render_subdivision(&eighth));

    let eighth_accent = tr909_voice_balance(&eighth, subdivision / 2);
    let late_same_slot = tr909_voice_balance(&late, subdivision / 2);
    assert!(eighth_accent.snare > late_same_slot.snare * 1.4);

    let candidate_audio = render_tr909_offline(&Tr909RenderState {
        counter_rhythm: Some(Tr909CounterRhythm::EighthAnswer),
        ..Tr909RenderState {
            mode: Tr909RenderMode::BreakReinforce,
            routing: Tr909RenderRouting::DrumBusSupport,
            pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
            phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
            drum_bus_level: 0.80,
            slam_enabled: true,
            slam_intensity: 0.85,
            is_transport_running: true,
            tempo_bpm: 128.0,
            ..Tr909RenderState::default()
        }
    }, 48_000, 2, 96_000);
    let phase_control_audio = render_tr909_offline(&Tr909RenderState {
        counter_rhythm: Some(Tr909CounterRhythm::EighthAnswer),
        counter_rhythm_phase: Tr909CounterRhythmPhase::PhaseControl,
        ..Tr909RenderState {
            mode: Tr909RenderMode::BreakReinforce,
            routing: Tr909RenderRouting::DrumBusSupport,
            pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
            phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
            drum_bus_level: 0.80,
            slam_enabled: true,
            slam_intensity: 0.85,
            is_transport_running: true,
            tempo_bpm: 128.0,
            ..Tr909RenderState::default()
        }
    }, 48_000, 2, 96_000);

    assert!(signal_delta_metrics(&candidate_audio, &phase_control_audio).rms > 0.01);
}

fn tr909_low_band_rms(samples: &[f32], sample_rate: u32, channel_count: usize) -> f32 {
    signal_metrics(&tr909_one_pole_lowpass(
        samples,
        145.0,
        sample_rate,
        channel_count,
    ))
    .rms
}

fn tr909_high_band_proxy_rms(
    samples: &[f32],
    sample_rate: u32,
    channel_count: usize,
) -> f32 {
    let low = tr909_one_pole_lowpass(samples, 1_800.0, sample_rate, channel_count);
    let high = samples
        .iter()
        .zip(low.iter())
        .map(|(sample, low)| sample - low)
        .collect::<Vec<_>>();
    signal_metrics(&high).rms
}

fn tr909_one_pole_lowpass(
    samples: &[f32],
    cutoff_hz: f32,
    sample_rate: u32,
    channel_count: usize,
) -> Vec<f32> {
    if sample_rate == 0 || channel_count == 0 {
        return vec![0.0; samples.len()];
    }
    let rc = 1.0 / (std::f32::consts::TAU * cutoff_hz.max(1.0));
    let dt = 1.0 / sample_rate as f32;
    let alpha = dt / (rc + dt);
    let mut previous = vec![0.0; channel_count];
    samples
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            let channel = index % channel_count;
            previous[channel] += alpha * (sample - previous[channel]);
            previous[channel]
        })
        .collect()
}
