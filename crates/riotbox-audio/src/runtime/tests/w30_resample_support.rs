fn positive_realtime_source_window() -> RealtimeW30PreviewSampleWindow {
    let mut samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
    fill_positive_preview_ramp(&mut samples);
    RealtimeW30PreviewSampleWindow {
        source_start_frame: 0,
        source_end_frame: W30_PREVIEW_SAMPLE_WINDOW_LEN as u64,
        sample_count: W30_PREVIEW_SAMPLE_WINDOW_LEN,
        samples,
    }
}

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
            source_sample_rate: 48_000,
            playback_frame_count: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN as u64,
            sample_count: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN,
            loop_enabled: true,
            playback_rate: 1.0,
            reverse: false,
            loop_crossfade_sample_count: 128,
            chop_slice_count: 0,
            chop_slice_starts: [0; W30_PAD_CHOP_SLICE_COUNT],
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
fn w30_pad_playback_cursor_preserves_full_capture_duration() {
    let mut samples = [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN];
    for (index, sample) in samples.iter_mut().enumerate() {
        *sample = index as f32 / W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN as f32;
    }
    let window = RealtimeW30PadPlaybackSampleWindow {
        source_start_frame: 0,
        source_end_frame: 48_000,
        source_sample_rate: 48_000,
        playback_frame_count: 48_000,
        sample_count: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN,
        loop_enabled: true,
        playback_rate: 1.0,
        reverse: false,
        loop_crossfade_sample_count: 128,
        chop_slice_count: 0,
        chop_slice_starts: [0; W30_PAD_CHOP_SLICE_COUNT],
        samples,
    };
    let mut state = W30PreviewCallbackState::default();

    for _ in 0..24_000 {
        w30_pad_playback_sample(&window, &mut state, 48_000);
    }

    assert!(
        (state.pad_playback_cursor - W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN as f32 / 2.0).abs()
            < 1.0,
        "duration-aware cursor advanced to {} instead of the capture midpoint",
        state.pad_playback_cursor
    );
}

#[test]
fn w30_transport_steps_select_source_derived_chop_slices() {
    let shared = SharedW30PreviewRenderState::new(&W30PreviewRenderState::default());
    let mut normal = shared.snapshot();
    normal.mode = W30PreviewRenderMode::LiveRecall;
    normal.routing = W30PreviewRenderRouting::MusicBusPreview;
    normal.pad_playback = RealtimeW30PadPlaybackSampleWindow {
        sample_count: 8_192,
        playback_rate: 1.0,
        chop_slice_count: 4,
        chop_slice_starts: [100, 900, 2_100, 4_700, 0, 0, 0, 0],
        ..Default::default()
    };
    let damaged = RealtimeW30PreviewRenderState {
        pad_playback: RealtimeW30PadPlaybackSampleWindow {
            playback_rate: 0.78,
            ..normal.pad_playback
        },
        ..normal
    };

    assert_eq!(w30_chop_slice_cursor(&normal, 2), 2_100.0);
    assert_eq!(w30_chop_slice_cursor(&damaged, 2), 2_100.0);
    assert_eq!(w30_chop_slice_cursor(&damaged, 3), 900.0);
    assert!(!should_trigger_w30_step(&damaged, 3));
    assert!(should_trigger_w30_step(&damaged, 4));
}

#[test]
fn w30_pad_signature_tracks_every_active_chop_slice() {
    let shared = SharedW30PreviewRenderState::new(&W30PreviewRenderState::default());
    let mut first = shared.snapshot();
    first.pad_playback.sample_count = 8_192;
    first.pad_playback.chop_slice_count = 3;
    first.pad_playback.chop_slice_starts = [100, 900, 2_100, 0, 0, 0, 0, 0];
    let mut second = first;
    second.pad_playback.chop_slice_starts[2] = 4_700;

    assert_ne!(
        w30_pad_playback_signature(&first),
        w30_pad_playback_signature(&second)
    );
}

#[test]
fn w30_damage_direction_and_rate_change_sample_motion() {
    let mut samples = [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN];
    for (index, sample) in samples.iter_mut().enumerate() {
        *sample = index as f32 / W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN as f32 * 2.0 - 1.0;
    }
    let base = RealtimeW30PadPlaybackSampleWindow {
        source_start_frame: 0,
        source_end_frame: 48_000,
        source_sample_rate: 48_000,
        playback_frame_count: 48_000,
        sample_count: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN,
        loop_enabled: true,
        playback_rate: 1.0,
        reverse: false,
        loop_crossfade_sample_count: 128,
        chop_slice_count: 0,
        chop_slice_starts: [0; W30_PAD_CHOP_SLICE_COUNT],
        samples,
    };
    let damaged = RealtimeW30PadPlaybackSampleWindow {
        playback_rate: 0.82,
        reverse: true,
        ..base
    };
    let mut forward_state = W30PreviewCallbackState {
        pad_playback_cursor: 128.0,
        pad_playback_age_frames: 256,
        ..Default::default()
    };
    let mut damaged_state = W30PreviewCallbackState {
        pad_playback_cursor: 128.0,
        pad_playback_age_frames: 256,
        ..Default::default()
    };

    let forward = w30_pad_playback_sample(&base, &mut forward_state, 48_000);
    let reverse = w30_pad_playback_sample(&damaged, &mut damaged_state, 48_000);

    assert!(forward < -0.9, "forward ramp sample was {forward}");
    assert!(reverse > 0.9, "reverse ramp sample was {reverse}");
    assert!(
        damaged_state.pad_playback_cursor < forward_state.pad_playback_cursor,
        "damage pitch-rate should advance more slowly"
    );
}

#[test]
fn w30_loop_crossfade_keeps_wrap_boundary_click_safe() {
    let mut samples = [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN];
    for (index, sample) in samples.iter_mut().enumerate() {
        *sample = index as f32 / W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN as f32 * 2.0 - 1.0;
    }
    let window = RealtimeW30PadPlaybackSampleWindow {
        source_start_frame: 0,
        source_end_frame: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN as u64,
        source_sample_rate: 48_000,
        playback_frame_count: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN as u64,
        sample_count: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN,
        loop_enabled: true,
        playback_rate: 1.0,
        reverse: false,
        loop_crossfade_sample_count: 128,
        chop_slice_count: 0,
        chop_slice_starts: [0; W30_PAD_CHOP_SLICE_COUNT],
        samples,
    };
    let mut state = W30PreviewCallbackState {
        pad_playback_cursor: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN as f32 - 0.5,
        pad_playback_age_frames: 256,
        ..Default::default()
    };

    let before_wrap = w30_pad_playback_sample(&window, &mut state, 48_000);
    let after_wrap = w30_pad_playback_sample(&window, &mut state, 48_000);

    assert!(
        (before_wrap - after_wrap).abs() < 0.05,
        "loop boundary jumped from {before_wrap} to {after_wrap}"
    );
}

#[test]
fn w30_pad_trigger_attack_fades_in_once_without_loop_wrap_dropout() {
    let window = RealtimeW30PadPlaybackSampleWindow {
        source_start_frame: 0,
        source_end_frame: 1_024,
        source_sample_rate: 48_000,
        playback_frame_count: 1_024,
        sample_count: 1_024,
        loop_enabled: true,
        playback_rate: 1.0,
        reverse: false,
        loop_crossfade_sample_count: 64,
        chop_slice_count: 0,
        chop_slice_starts: [0; W30_PAD_CHOP_SLICE_COUNT],
        samples: [0.8; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
    };
    let mut state = W30PreviewCallbackState::default();

    let first = w30_pad_playback_sample(&window, &mut state, 48_000);
    for _ in 0..80 {
        w30_pad_playback_sample(&window, &mut state, 48_000);
    }
    state.pad_playback_cursor = 0.0;
    let after_wrap = w30_pad_playback_sample(&window, &mut state, 48_000);

    assert_eq!(first, 0.0);
    assert!(after_wrap > 0.75, "loop wrap incorrectly retriggered attack: {after_wrap}");
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
            source_window_preview: positive_realtime_source_window(),
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
            source_window_preview: positive_realtime_source_window(),
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
            source_window_preview: positive_realtime_source_window(),
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
            source_window_preview: positive_realtime_source_window(),
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
            source_window_preview: positive_realtime_source_window(),
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

    assert!((browse_peak - recall_peak).abs() > 0.0005);
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
            source_window_preview: positive_realtime_source_window(),
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
        source_window_preview: positive_realtime_source_window(),
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
            slam_enabled: false,
            slam_intensity: 0.6,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 0.0,
            source_bar_grid_anchor_position_beats: None,
        },
        &mut state,
    );

    assert!(buffer.iter().any(|sample| sample.abs() > 0.0001));
}
