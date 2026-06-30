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
        tr909_low_band_rms(&drop_drive) > tr909_low_band_rms(&break_lift) * 1.03,
        "drop profile should carry more kick/low pressure"
    );
    assert!(
        tr909_high_band_proxy_rms(&break_lift) > tr909_high_band_proxy_rms(&steady) * 1.08,
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

fn tr909_low_band_rms(samples: &[f32]) -> f32 {
    signal_metrics(&tr909_one_pole_lowpass(samples, 145.0)).rms
}

fn tr909_high_band_proxy_rms(samples: &[f32]) -> f32 {
    let low = tr909_one_pole_lowpass(samples, 1_800.0);
    let high = samples
        .iter()
        .zip(low.iter())
        .map(|(sample, low)| sample - low)
        .collect::<Vec<_>>();
    signal_metrics(&high).rms
}

fn tr909_one_pole_lowpass(samples: &[f32], cutoff_hz: f32) -> Vec<f32> {
    let rc = 1.0 / (std::f32::consts::TAU * cutoff_hz.max(1.0));
    let dt = 1.0 / 44_100.0;
    let alpha = dt / (rc + dt);
    let mut previous = 0.0;
    samples
        .iter()
        .map(|sample| {
            previous += alpha * (sample - previous);
            previous
        })
        .collect()
}
