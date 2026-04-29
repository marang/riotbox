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
            slam_intensity: 0.0,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
        },
        &RealtimeMc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
            note_budget: Mc202NoteBudget::Balanced,
            contour_hint: Mc202ContourHint::Neutral,
            hook_response: Mc202HookResponse::Direct,
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
                lineage_capture_count: 0,
                generation_depth: 0,
                music_bus_level: 0.0,
                grit_level: 0.0,
                is_transport_running: true,
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
fn offline_mc202_render_produces_distinct_follower_and_answer_metrics() {
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
            phrase_shape: Mc202PhraseShape::AnswerHook,
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

    assert!(follower_metrics.active_samples > 10_000);
    assert!(answer_metrics.active_samples > 10_000);
    assert!((follower_metrics.rms - answer_metrics.rms).abs() > 0.001);
}

#[test]
fn offline_mc202_render_produces_distinct_instigator_metrics() {
    let follower = render_mc202_offline(
        &Mc202RenderState {
            mode: Mc202RenderMode::Follower,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::FollowerDrive,
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
            slam_intensity: 0.6,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 0.0,
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
            slam_intensity: 0.35,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
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
            slam_intensity: 0.35,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
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
        slam_intensity: 0.35,
        is_transport_running: true,
        tempo_bpm: 126.0,
        position_beats: 0.0,
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
    assert_eq!(scene_active, transport_active);
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
            slam_intensity: 0.45,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
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
            slam_intensity: 0.45,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
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

