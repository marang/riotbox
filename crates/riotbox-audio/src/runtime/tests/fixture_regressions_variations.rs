#[test]
fn fixture_backed_w30_preview_audio_regressions_hold() {
    let fixtures: Vec<W30AudioFixtureCase> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/w30_preview_audio_regression.json"
    ))
    .expect("parse W-30 preview audio regression fixture");

    for fixture in fixtures {
        let mut callback_state = W30PreviewCallbackState::default();
        let mut buffer = [0.0_f32; 512];

        render_w30_preview_buffer(
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
        let rms =
            (buffer.iter().map(|sample| sample * sample).sum::<f32>() / buffer.len() as f32).sqrt();

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
        if let Some(min_rms) = fixture.expected.min_rms {
            assert!(rms >= min_rms, "{} RMS too low: got {rms}", fixture.name);
        }
        if let Some(max_rms) = fixture.expected.max_rms {
            assert!(rms <= max_rms, "{} RMS too high: got {rms}", fixture.name);
        }
    }
}

#[test]
fn offline_w30_preview_render_produces_reviewable_metrics() {
    let mut samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
    fill_positive_preview_ramp(&mut samples);

    let buffer = render_w30_preview_offline(
        &W30PreviewRenderState {
            mode: W30PreviewRenderMode::RawCaptureAudition,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
            active_bank_id: Some("bank-a".into()),
            focused_pad_id: Some("pad-01".into()),
            capture_id: Some("cap-01".into()),
            trigger_revision: 0,
            trigger_velocity: 0.0,
            source_window_preview: Some(W30PreviewSampleWindow {
                source_start_frame: 0,
                source_end_frame: W30_PREVIEW_SAMPLE_WINDOW_LEN as u64,
                sample_count: W30_PREVIEW_SAMPLE_WINDOW_LEN,
                samples,
            }),
            pad_playback: None,
            music_bus_level: 0.64,
            grit_level: 0.0,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 32.0,
        },
        44_100,
        2,
        256,
    );

    let metrics = signal_metrics(&buffer);

    assert_eq!(buffer.len(), 512);
    assert!(
        metrics.active_samples >= 300,
        "active sample count too low: got {}",
        metrics.active_samples
    );
    assert!(
        (0.019..=0.04).contains(&metrics.rms),
        "unexpected RMS: got {}",
        metrics.rms
    );
    assert!(
        (6.0..=18.0).contains(&metrics.sum),
        "unexpected sum: got {}",
        metrics.sum
    );
    assert!(
        (0.015..=0.08).contains(&metrics.peak_abs),
        "unexpected peak: got {}",
        metrics.peak_abs
    );
}

#[test]
fn fixture_backed_w30_resample_audio_regressions_hold() {
    let fixtures: Vec<W30ResampleAudioFixtureCase> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/w30_resample_audio_regression.json"
    ))
    .expect("parse W-30 resample audio regression fixture");

    for fixture in fixtures {
        let mut callback_state = W30ResampleTapCallbackState::default();
        let mut buffer = [0.0_f32; 512];

        render_w30_resample_tap_buffer(
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
    }
}

#[test]
fn pattern_adoption_variants_produce_distinct_activity() {
    let mut pulse_state = Tr909CallbackState::default();
    let mut drive_state = Tr909CallbackState::default();
    let mut grid_state = Tr909CallbackState::default();
    let mut pulse = [0.0_f32; 512];
    let mut drive = [0.0_f32; 512];
    let mut grid = [0.0_f32; 512];

    render_tr909_buffer(
        &mut pulse,
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
            slam_intensity: 0.35,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
        },
        &mut pulse_state,
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
            slam_intensity: 0.35,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
        },
        &mut drive_state,
    );

    render_tr909_buffer(
        &mut grid,
        44_100,
        2,
        &RealtimeTr909RenderState {
            mode: Tr909RenderMode::Takeover,
            routing: Tr909RenderRouting::DrumBusTakeover,
            source_support_profile: None,
            source_support_context: None,
            pattern_adoption: Some(Tr909PatternAdoption::TakeoverGrid),
            phrase_variation: Some(Tr909PhraseVariation::PhraseRelease),
            takeover_profile: Some(Tr909TakeoverRenderProfile::ControlledPhrase),
            drum_bus_level: 0.8,
            slam_intensity: 0.35,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
        },
        &mut grid_state,
    );

    let pulse_peak = pulse
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let drive_peak = drive
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let grid_peak = grid
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));

    assert_ne!(pulse_peak, drive_peak);
    assert_ne!(drive_peak, grid_peak);
    assert!(grid_peak > pulse_peak);
}

#[test]
fn phrase_variations_produce_distinct_activity() {
    let mut anchor_state = Tr909CallbackState::default();
    let mut drive_state = Tr909CallbackState::default();
    let mut release_state = Tr909CallbackState::default();
    let mut anchor = [0.0_f32; 512];
    let mut drive = [0.0_f32; 512];
    let mut release = [0.0_f32; 512];

    let base = RealtimeTr909RenderState {
        mode: Tr909RenderMode::Takeover,
        routing: Tr909RenderRouting::DrumBusTakeover,
        source_support_profile: None,
        source_support_context: None,
        pattern_adoption: Some(Tr909PatternAdoption::TakeoverGrid),
        phrase_variation: Some(Tr909PhraseVariation::PhraseAnchor),
        takeover_profile: Some(Tr909TakeoverRenderProfile::ControlledPhrase),
        drum_bus_level: 0.8,
        slam_intensity: 0.45,
        is_transport_running: true,
        tempo_bpm: 126.0,
        position_beats: 0.0,
    };

    render_tr909_buffer(&mut anchor, 44_100, 2, &base, &mut anchor_state);

    let mut drive_render = base;
    drive_render.phrase_variation = Some(Tr909PhraseVariation::PhraseDrive);
    render_tr909_buffer(&mut drive, 44_100, 2, &drive_render, &mut drive_state);

    let mut release_render = base;
    release_render.phrase_variation = Some(Tr909PhraseVariation::PhraseRelease);
    render_tr909_buffer(&mut release, 44_100, 2, &release_render, &mut release_state);

    let anchor_peak = anchor
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let drive_peak = drive
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let release_peak = release
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let anchor_active = anchor.iter().filter(|sample| sample.abs() > 0.0001).count();
    let release_active = release
        .iter()
        .filter(|sample| sample.abs() > 0.0001)
        .count();

    assert!(drive_peak > anchor_peak);
    assert!(release_peak < drive_peak);
    assert!(release_active < anchor_active);
}
