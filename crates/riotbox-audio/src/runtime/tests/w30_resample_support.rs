#[test]
fn w30_live_recall_uses_source_window_samples_when_available() {
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
        mode: W30PreviewRenderMode::LiveRecall,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
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
fn w30_pad_playback_uses_duration_window_beyond_fixed_preview_len() {
    let mut state = W30PreviewCallbackState::default();
    let frame_count = W30_PREVIEW_SAMPLE_WINDOW_LEN + 512;
    let mut duration_buffer = vec![0.0_f32; frame_count * 2];
    let mut fixed_preview_buffer = vec![0.0_f32; frame_count * 2];
    let mut pad_samples = [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN];
    let mut preview_samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
    preview_samples.fill(0.22);
    for (index, sample) in pad_samples.iter_mut().enumerate() {
        *sample = if index < W30_PREVIEW_SAMPLE_WINDOW_LEN {
            0.22
        } else {
            -0.31
        };
    }

    let duration_render = RealtimeW30PreviewRenderState {
        mode: W30PreviewRenderMode::LiveRecall,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
        trigger_revision: 0,
        trigger_velocity: 0.0,
        source_window_preview: RealtimeW30PreviewSampleWindow {
            source_start_frame: 0,
            source_end_frame: W30_PREVIEW_SAMPLE_WINDOW_LEN as u64,
            sample_count: W30_PREVIEW_SAMPLE_WINDOW_LEN,
            samples: preview_samples,
        },
        pad_playback: RealtimeW30PadPlaybackSampleWindow {
            source_start_frame: 0,
            source_end_frame: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN as u64,
            sample_count: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN,
            loop_enabled: true,
            samples: pad_samples,
        },
        music_bus_level: 0.64,
        grit_level: 0.0,
        is_transport_running: false,
        tempo_bpm: 0.0,
        position_beats: 0.0,
    };
    let fixed_preview_render = RealtimeW30PreviewRenderState {
        pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
        ..duration_render
    };

    render_w30_preview_buffer(
        &mut duration_buffer,
        48_000,
        2,
        &duration_render,
        &mut state,
    );
    render_w30_preview_buffer(
        &mut fixed_preview_buffer,
        48_000,
        2,
        &fixed_preview_render,
        &mut W30PreviewCallbackState::default(),
    );

    let late_start = W30_PREVIEW_SAMPLE_WINDOW_LEN * 2;
    assert!(
        duration_buffer[late_start..]
            .iter()
            .any(|sample| *sample < -0.01),
        "duration-aware W-30 pad playback did not reach samples beyond the fixed preview window"
    );
    assert_ne!(duration_buffer, fixed_preview_buffer);
}

#[test]
fn w30_preview_respects_zero_music_bus_level() {
    let mut state = W30PreviewCallbackState::default();
    let mut buffer = [0.0_f32; 512];

    render_w30_preview_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            mode: W30PreviewRenderMode::LiveRecall,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
            trigger_revision: 0,
            trigger_velocity: 0.0,
            source_window_preview: RealtimeW30PreviewSampleWindow::default(),
            pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
            music_bus_level: 0.0,
            grit_level: 0.6,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
        },
        &mut state,
    );

    assert!(buffer.iter().all(|sample| sample.abs() <= f32::EPSILON));
}

#[test]
fn promoted_w30_audition_is_more_present_than_pinned_recall() {
    let mut pinned_state = W30PreviewCallbackState::default();
    let mut audition_state = W30PreviewCallbackState::default();
    let mut pinned = [0.0_f32; 512];
    let mut audition = [0.0_f32; 512];

    render_w30_preview_buffer(
        &mut pinned,
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
        &mut pinned_state,
    );

    render_w30_preview_buffer(
        &mut audition,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            mode: W30PreviewRenderMode::PromotedAudition,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::PromotedAudition),
            trigger_revision: 0,
            trigger_velocity: 0.0,
            source_window_preview: RealtimeW30PreviewSampleWindow::default(),
            pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
            music_bus_level: 0.64,
            grit_level: 0.68,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 0.0,
        },
        &mut audition_state,
    );

    let pinned_peak = pinned
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let audition_peak = audition
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let pinned_energy = pinned.iter().map(|sample| sample.abs()).sum::<f32>();
    let audition_energy = audition.iter().map(|sample| sample.abs()).sum::<f32>();

    assert!(audition_peak > pinned_peak);
    assert!(audition_energy > pinned_energy);
}

#[test]
fn slice_pool_browse_preview_differs_from_promoted_recall() {
    let mut recall_state = W30PreviewCallbackState::default();
    let mut browse_state = W30PreviewCallbackState::default();
    let mut recall = [0.0_f32; 512];
    let mut browse = [0.0_f32; 512];

    render_w30_preview_buffer(
        &mut recall,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            mode: W30PreviewRenderMode::LiveRecall,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
            trigger_revision: 0,
            trigger_velocity: 0.0,
            source_window_preview: RealtimeW30PreviewSampleWindow::default(),
            pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
            music_bus_level: 0.64,
            grit_level: 0.0,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 32.0,
        },
        &mut recall_state,
    );

    render_w30_preview_buffer(
        &mut browse,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            mode: W30PreviewRenderMode::LiveRecall,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::SlicePoolBrowse),
            trigger_revision: 0,
            trigger_velocity: 0.0,
            source_window_preview: RealtimeW30PreviewSampleWindow::default(),
            pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
            music_bus_level: 0.64,
            grit_level: 0.0,
            is_transport_running: true,
            tempo_bpm: 126.0,
            position_beats: 32.0,
        },
        &mut browse_state,
    );

    let recall_peak = recall
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    let browse_peak = browse
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));

    assert!((browse_peak - recall_peak).abs() > 0.002);
    assert_ne!(browse, recall);
}

#[test]
fn stopped_w30_preview_remains_audible_for_manual_previewing() {
    let mut state = W30PreviewCallbackState::default();
    let mut buffer = [0.0_f32; 512];

    render_w30_preview_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            mode: W30PreviewRenderMode::PromotedAudition,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::PromotedAudition),
            trigger_revision: 0,
            trigger_velocity: 0.0,
            source_window_preview: RealtimeW30PreviewSampleWindow::default(),
            pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
            music_bus_level: 0.64,
            grit_level: 0.72,
            is_transport_running: false,
            tempo_bpm: 0.0,
            position_beats: 0.0,
        },
        &mut state,
    );

    assert!(buffer.iter().any(|sample| sample.abs() > 0.0001));
}

#[test]
fn w30_trigger_revision_retriggers_preview_accent() {
    let mut state = W30PreviewCallbackState::default();
    let mut retriggered = [0.0_f32; 512];
    let render = RealtimeW30PreviewRenderState {
        mode: W30PreviewRenderMode::LiveRecall,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::PinnedRecall),
        trigger_revision: 0,
        trigger_velocity: 0.0,
        source_window_preview: RealtimeW30PreviewSampleWindow::default(),
        pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
        music_bus_level: 0.64,
        grit_level: 0.45,
        is_transport_running: true,
        tempo_bpm: 126.0,
        position_beats: 0.0,
    };

    let mut primed = [0.0_f32; 512];
    render_w30_preview_buffer(&mut primed, 44_100, 2, &render, &mut state);
    state.envelope = 0.0;
    state.was_active = true;
    state.last_trigger_revision = 0;

    let mut retrigger_render = render;
    retrigger_render.trigger_revision = 7;
    retrigger_render.trigger_velocity = 0.92;
    render_w30_preview_buffer(&mut retriggered, 44_100, 2, &retrigger_render, &mut state);

    assert!(retriggered.iter().any(|sample| sample.abs() > 0.0001));
    assert_eq!(state.last_trigger_revision, 7);
}

#[test]
fn w30_resample_tap_stays_silent_when_idle() {
    let mut state = W30ResampleTapCallbackState::default();
    let mut buffer = [0.0_f32; 512];

    render_w30_resample_tap_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeW30ResampleTapState {
            mode: W30ResampleTapMode::Idle,
            routing: W30ResampleTapRouting::Silent,
            source_profile: None,
            lineage_capture_count: 0,
            generation_depth: 0,
            music_bus_level: 0.64,
            grit_level: 0.4,
            is_transport_running: true,
        },
        &mut state,
    );

    assert!(buffer.iter().all(|sample| sample.abs() <= f32::EPSILON));
}

#[test]
fn w30_resample_tap_produces_audible_samples_when_lineage_is_ready() {
    let mut state = W30ResampleTapCallbackState::default();
    let mut buffer = [0.0_f32; 512];

    render_w30_resample_tap_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeW30ResampleTapState {
            mode: W30ResampleTapMode::CaptureLineageReady,
            routing: W30ResampleTapRouting::InternalCaptureTap,
            source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
            lineage_capture_count: 2,
            generation_depth: 1,
            music_bus_level: 0.58,
            grit_level: 0.62,
            is_transport_running: true,
        },
        &mut state,
    );

    assert!(buffer.iter().any(|sample| sample.abs() > 0.0001));
}

#[test]
fn w30_resample_tap_respects_zero_music_bus_level() {
    let mut state = W30ResampleTapCallbackState::default();
    let mut buffer = [0.0_f32; 512];

    render_w30_resample_tap_buffer(
        &mut buffer,
        44_100,
        2,
        &RealtimeW30ResampleTapState {
            mode: W30ResampleTapMode::CaptureLineageReady,
            routing: W30ResampleTapRouting::InternalCaptureTap,
            source_profile: Some(W30ResampleTapSourceProfile::PinnedCapture),
            lineage_capture_count: 3,
            generation_depth: 2,
            music_bus_level: 0.0,
            grit_level: 0.7,
            is_transport_running: false,
        },
        &mut state,
    );

    assert!(buffer.iter().all(|sample| sample.abs() <= f32::EPSILON));
}

#[test]
fn render_buffer_produces_audible_samples_for_support_mode() {
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
            drum_bus_level: 0.8,
            slam_intensity: 0.6,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 0.0,
        },
        &mut state,
    );

    assert!(buffer.iter().any(|sample| sample.abs() > 0.0001));
}

