#[test]
fn shared_render_state_tracks_updates() {
    let shared = SharedTr909RenderState::new(&Tr909RenderState::default());
    let mut state = Tr909RenderState {
        mode: Tr909RenderMode::Takeover,
        routing: Tr909RenderRouting::DrumBusTakeover,
        source_support_profile: None,
        pattern_adoption: None,
        phrase_variation: None,
        takeover_profile: Some(Tr909TakeoverRenderProfile::ControlledPhrase),
        drum_bus_level: 0.8,
        slam_intensity: 0.9,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 17.5,
        ..Tr909RenderState::default()
    };
    shared.update(&state);

    let snapshot = shared.snapshot();
    assert_eq!(snapshot.mode, Tr909RenderMode::Takeover);
    assert_eq!(snapshot.routing, Tr909RenderRouting::DrumBusTakeover);
    assert_eq!(
        snapshot.takeover_profile,
        Some(Tr909TakeoverRenderProfile::ControlledPhrase)
    );
    assert_eq!(snapshot.source_support_context, None);
    assert_eq!(snapshot.pattern_adoption, None);
    assert_eq!(snapshot.phrase_variation, None);
    assert_eq!(snapshot.tempo_bpm, 128.0);
    assert_eq!(snapshot.position_beats, 17.5);

    state.mode = Tr909RenderMode::SourceSupport;
    state.routing = Tr909RenderRouting::DrumBusSupport;
    state.source_support_profile = Some(Tr909SourceSupportProfile::DropDrive);
    state.source_support_context = Some(Tr909SourceSupportContext::SceneTarget);
    state.pattern_adoption = Some(Tr909PatternAdoption::MainlineDrive);
    state.phrase_variation = Some(Tr909PhraseVariation::PhraseDrive);
    state.takeover_profile = None;
    shared.update(&state);

    let updated = shared.snapshot();
    assert_eq!(updated.mode, Tr909RenderMode::SourceSupport);
    assert_eq!(updated.routing, Tr909RenderRouting::DrumBusSupport);
    assert_eq!(
        updated.source_support_profile,
        Some(Tr909SourceSupportProfile::DropDrive)
    );
    assert_eq!(
        updated.source_support_context,
        Some(Tr909SourceSupportContext::SceneTarget)
    );
    assert_eq!(
        updated.pattern_adoption,
        Some(Tr909PatternAdoption::MainlineDrive)
    );
    assert_eq!(
        updated.phrase_variation,
        Some(Tr909PhraseVariation::PhraseDrive)
    );

    state.source_support_context = Some(Tr909SourceSupportContext::TransportBar);
    shared.update(&state);

    let transport_fallback = shared.snapshot();
    assert_eq!(
        transport_fallback.source_support_context,
        Some(Tr909SourceSupportContext::TransportBar)
    );

    state.source_support_context = None;
    shared.update(&state);

    let unset = shared.snapshot();
    assert_eq!(unset.source_support_context, None);
}

#[test]
fn shared_mc202_render_state_tracks_updates() {
    let shared = SharedMc202RenderState::new(&Mc202RenderState::default());
    let render = Mc202RenderState {
        mode: Mc202RenderMode::Instigator,
        routing: Mc202RenderRouting::MusicBusBass,
        phrase_shape: Mc202PhraseShape::InstigatorSpike,
        note_budget: Mc202NoteBudget::Push,
        contour_hint: Mc202ContourHint::Lift,
        hook_response: Mc202HookResponse::AnswerSpace,
        touch: 0.90,
        music_bus_level: 0.64,
        is_transport_running: true,
        tempo_bpm: 130.0,
        position_beats: 41.5,
    };

    shared.update(&render);

    let snapshot = shared.snapshot();
    assert_eq!(snapshot.mode, Mc202RenderMode::Instigator);
    assert_eq!(snapshot.routing, Mc202RenderRouting::MusicBusBass);
    assert_eq!(snapshot.phrase_shape, Mc202PhraseShape::InstigatorSpike);
    assert_eq!(snapshot.note_budget, Mc202NoteBudget::Push);
    assert_eq!(snapshot.contour_hint, Mc202ContourHint::Lift);
    assert_eq!(snapshot.hook_response, Mc202HookResponse::AnswerSpace);
    assert_eq!(snapshot.touch, 0.90);
    assert_eq!(snapshot.music_bus_level, 0.64);
    assert!(snapshot.is_transport_running);
    assert_eq!(snapshot.tempo_bpm, 130.0);
    assert_eq!(snapshot.position_beats, 41.5);
}

#[test]
fn shared_w30_preview_state_tracks_updates() {
    let shared = SharedW30PreviewRenderState::new(&W30PreviewRenderState::default());
    let mut state = W30PreviewRenderState {
        mode: W30PreviewRenderMode::LiveRecall,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::PinnedRecall),
        active_bank_id: Some("bank-a".into()),
        focused_pad_id: Some("pad-01".into()),
        capture_id: Some("cap-01".into()),
        trigger_revision: 3,
        trigger_velocity: 0.78,
        source_window_preview: None,
        pad_playback: None,
        music_bus_level: 0.55,
        grit_level: 0.68,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 21.0,
    };
    shared.update(&state);

    let snapshot = shared.snapshot();
    assert_eq!(snapshot.mode, W30PreviewRenderMode::LiveRecall);
    assert_eq!(snapshot.routing, W30PreviewRenderRouting::MusicBusPreview);
    assert_eq!(
        snapshot.source_profile,
        Some(W30PreviewSourceProfile::PinnedRecall)
    );
    assert_eq!(snapshot.trigger_revision, 3);
    assert_eq!(snapshot.trigger_velocity, 0.78);
    assert_eq!(snapshot.music_bus_level, 0.55);
    assert_eq!(snapshot.grit_level, 0.68);
    assert_eq!(snapshot.tempo_bpm, 128.0);
    assert_eq!(snapshot.position_beats, 21.0);

    state.mode = W30PreviewRenderMode::PromotedAudition;
    state.source_profile = Some(W30PreviewSourceProfile::PromotedAudition);
    state.grit_level = 0.82;
    shared.update(&state);

    let updated = shared.snapshot();
    assert_eq!(updated.mode, W30PreviewRenderMode::PromotedAudition);
    assert_eq!(
        updated.source_profile,
        Some(W30PreviewSourceProfile::PromotedAudition)
    );
    assert_eq!(updated.grit_level, 0.82);
}

#[test]
fn shared_w30_resample_tap_state_tracks_updates() {
    let shared = SharedW30ResampleTapState::new(&W30ResampleTapState::default());
    let mut state = W30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
        source_capture_id: Some("cap-03".into()),
        lineage_capture_count: 2,
        generation_depth: 1,
        music_bus_level: 0.61,
        grit_level: 0.72,
        is_transport_running: true,
    };
    shared.update(&state);

    let snapshot = shared.snapshot();
    assert_eq!(snapshot.mode, W30ResampleTapMode::CaptureLineageReady);
    assert_eq!(snapshot.routing, W30ResampleTapRouting::InternalCaptureTap);
    assert_eq!(
        snapshot.source_profile,
        Some(W30ResampleTapSourceProfile::PromotedCapture)
    );
    assert_eq!(snapshot.lineage_capture_count, 2);
    assert_eq!(snapshot.generation_depth, 1);
    assert_eq!(snapshot.music_bus_level, 0.61);
    assert_eq!(snapshot.grit_level, 0.72);
    assert!(snapshot.is_transport_running);

    state.source_profile = Some(W30ResampleTapSourceProfile::PinnedCapture);
    state.lineage_capture_count = 3;
    state.generation_depth = 2;
    shared.update(&state);

    let updated = shared.snapshot();
    assert_eq!(
        updated.source_profile,
        Some(W30ResampleTapSourceProfile::PinnedCapture)
    );
    assert_eq!(updated.lineage_capture_count, 3);
    assert_eq!(updated.generation_depth, 2);
}

#[test]
fn render_buffer_stays_silent_when_idle() {
    let mut state = Tr909CallbackState::default();
    let mut buffer = [0.0_f32; 128];

    render_tr909_buffer(
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
            drum_bus_level: 0.8,
            slam_intensity: 0.2,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 0.0,
        },
        &mut state,
    );

    assert!(buffer.iter().all(|sample| sample.abs() <= f32::EPSILON));
}

#[test]
fn w30_preview_stays_silent_when_idle() {
    let mut state = W30PreviewCallbackState::default();
    let mut buffer = [0.0_f32; 512];

    render_w30_preview_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            mode: W30PreviewRenderMode::Idle,
            routing: W30PreviewRenderRouting::Silent,
            source_profile: None,
            trigger_revision: 0,
            trigger_velocity: 0.0,
            source_window_preview: RealtimeW30PreviewSampleWindow::default(),
            pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
            music_bus_level: 0.64,
            grit_level: 0.4,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
        },
        &mut state,
    );

    assert!(buffer.iter().all(|sample| sample.abs() <= f32::EPSILON));
}

#[test]
fn w30_preview_produces_audible_samples_for_live_recall() {
    let mut state = W30PreviewCallbackState::default();
    let mut buffer = [0.0_f32; 512];

    render_w30_preview_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            mode: W30PreviewRenderMode::LiveRecall,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::PinnedRecall),
            trigger_revision: 0,
            trigger_velocity: 0.0,
            source_window_preview: RealtimeW30PreviewSampleWindow::default(),
            pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
            music_bus_level: 0.64,
            grit_level: 0.4,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
        },
        &mut state,
    );

    assert!(buffer.iter().any(|sample| sample.abs() > 0.0001));
}

#[test]
fn w30_preview_produces_audible_samples_for_raw_capture_audition() {
    let mut state = W30PreviewCallbackState::default();
    let mut buffer = [0.0_f32; 512];

    render_w30_preview_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            mode: W30PreviewRenderMode::RawCaptureAudition,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
            trigger_revision: 0,
            trigger_velocity: 0.0,
            source_window_preview: RealtimeW30PreviewSampleWindow::default(),
            pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
            music_bus_level: 0.64,
            grit_level: 0.58,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
        },
        &mut state,
    );

    assert!(buffer.iter().any(|sample| sample.abs() > 0.0001));
}

#[test]
fn w30_raw_capture_audition_uses_source_window_samples_when_available() {
    let mut positive_state = W30PreviewCallbackState::default();
    let mut negative_state = W30PreviewCallbackState::default();
    let mut positive = [0.0_f32; 512];
    let mut negative = [0.0_f32; 512];
    let mut positive_samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
    let mut negative_samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
    fill_positive_preview_ramp(&mut positive_samples);
    for index in 0..W30_PREVIEW_SAMPLE_WINDOW_LEN {
        negative_samples[index] = -positive_samples[index];
    }

    let base_render = RealtimeW30PreviewRenderState {
        mode: W30PreviewRenderMode::RawCaptureAudition,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
        trigger_revision: 0,
        trigger_velocity: 0.0,
        source_window_preview: RealtimeW30PreviewSampleWindow {
            source_start_frame: 0,
            source_end_frame: W30_PREVIEW_SAMPLE_WINDOW_LEN as u64,
            sample_count: W30_PREVIEW_SAMPLE_WINDOW_LEN,
            samples: positive_samples,
        },
        pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
        music_bus_level: 0.64,
        grit_level: 0.0,
        is_transport_running: true,
        tempo_bpm: 126.0,
        position_beats: 0.0,
    };
    let negative_render = RealtimeW30PreviewRenderState {
        source_window_preview: RealtimeW30PreviewSampleWindow {
            samples: negative_samples,
            ..base_render.source_window_preview
        },
        ..base_render
    };

    render_w30_preview_buffer(&mut positive, 44_100, 2, &base_render, &mut positive_state);
    render_w30_preview_buffer(
        &mut negative,
        44_100,
        2,
        &negative_render,
        &mut negative_state,
    );

    assert!(positive.iter().any(|sample| *sample > 0.001));
    assert!(negative.iter().any(|sample| *sample < -0.001));
    assert_ne!(positive, negative);
}

#[test]
fn w30_promoted_audition_uses_source_window_samples_when_available() {
    let mut positive_state = W30PreviewCallbackState::default();
    let mut negative_state = W30PreviewCallbackState::default();
    let mut positive = [0.0_f32; 512];
    let mut negative = [0.0_f32; 512];
    let mut positive_samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
    let mut negative_samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
    fill_positive_preview_ramp(&mut positive_samples);
    for index in 0..W30_PREVIEW_SAMPLE_WINDOW_LEN {
        negative_samples[index] = -positive_samples[index];
    }

    let base_render = RealtimeW30PreviewRenderState {
        mode: W30PreviewRenderMode::PromotedAudition,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::PromotedAudition),
        trigger_revision: 0,
        trigger_velocity: 0.0,
        source_window_preview: RealtimeW30PreviewSampleWindow {
            source_start_frame: 0,
            source_end_frame: W30_PREVIEW_SAMPLE_WINDOW_LEN as u64,
            sample_count: W30_PREVIEW_SAMPLE_WINDOW_LEN,
            samples: positive_samples,
        },
        pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
        music_bus_level: 0.64,
        grit_level: 0.0,
        is_transport_running: true,
        tempo_bpm: 126.0,
        position_beats: 0.0,
    };
    let negative_render = RealtimeW30PreviewRenderState {
        source_window_preview: RealtimeW30PreviewSampleWindow {
            samples: negative_samples,
            ..base_render.source_window_preview
        },
        ..base_render
    };

    render_w30_preview_buffer(&mut positive, 44_100, 2, &base_render, &mut positive_state);
    render_w30_preview_buffer(
        &mut negative,
        44_100,
        2,
        &negative_render,
        &mut negative_state,
    );

    assert!(positive.iter().any(|sample| *sample > 0.001));
    assert!(negative.iter().any(|sample| *sample < -0.001));
    assert_ne!(positive, negative);
}

