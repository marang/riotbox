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

fn positive_realtime_resample_source() -> RealtimeW30ResampleSourceWindow {
    let mut samples = [0.0; W30_RESAMPLE_SOURCE_WINDOW_LEN];
    for (index, sample) in samples.iter_mut().enumerate() {
        let phase = index as f32 / 37.0;
        *sample = phase.sin() * 0.34 + (phase * 2.7).sin() * 0.09;
    }
    RealtimeW30ResampleSourceWindow {
        source_start_frame: 0,
        source_sample_rate: 44_100,
        source_frame_count: W30_RESAMPLE_SOURCE_WINDOW_LEN as u64,
        sample_count: W30_RESAMPLE_SOURCE_WINDOW_LEN,
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
            gate_step_fraction: 0.0,
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
        gate_step_fraction: 0.0,
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
fn w30_grid_gate_chokes_before_the_next_source_percussion_can_drift() {
    let mut render = RealtimeW30PreviewRenderState {
        mode: W30PreviewRenderMode::LiveRecall,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
        trigger_revision: 0,
        trigger_velocity: 0.0,
        source_window_preview: RealtimeW30PreviewSampleWindow::default(),
        pad_playback: RealtimeW30PadPlaybackSampleWindow {
            sample_count: 1,
            gate_step_fraction: 0.4,
            ..Default::default()
        },
        music_bus_level: 0.64,
        grit_level: 0.0,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    };

    let before_fade = W30PreviewCallbackState {
        pad_playback_age_frames: 2_000,
        ..Default::default()
    };
    let during_fade = W30PreviewCallbackState {
        pad_playback_age_frames: 4_200,
        ..Default::default()
    };
    let after_gate = W30PreviewCallbackState {
        pad_playback_age_frames: 5_000,
        ..Default::default()
    };

    let gate = w30_pad_grid_gate(&render, 48_000);
    assert_eq!(w30_pad_grid_gate_gain(gate, &before_fade), 1.0);
    assert!((w30_pad_grid_gate_gain(gate, &during_fade) - 0.5).abs() < 0.001);
    assert_eq!(w30_pad_grid_gate_gain(gate, &after_gate), 0.0);

    render.pad_playback.gate_step_fraction = f32::NAN;
    assert_eq!(w30_pad_grid_gate(&render, 48_000), None);
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
        gate_step_fraction: 0.0,
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
        gate_step_fraction: 0.0,
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
        gate_step_fraction: 0.0,
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
fn transport_stop_fades_an_active_w30_preview_and_latches_silence() {
    let mut state = W30PreviewCallbackState::default();
    let running_render = RealtimeW30PreviewRenderState {
        mode: W30PreviewRenderMode::RawCaptureAudition,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
        trigger_revision: 0,
        trigger_velocity: 0.0,
        source_window_preview: positive_realtime_source_window(),
        pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
        music_bus_level: 0.64,
        grit_level: 0.0,
        is_transport_running: true,
        tempo_bpm: 126.0,
        position_beats: 8.0,
    };
    let mut running = [0.0_f32; 1_024];
    render_w30_preview_buffer(&mut running, 44_100, 2, &running_render, &mut state);
    assert!(running.iter().any(|sample| sample.abs() > 0.0001));

    let mut stopped = [0.0_f32; 1_024];
    render_w30_preview_buffer(
        &mut stopped,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            is_transport_running: false,
            position_beats: 8.25,
            ..running_render
        },
        &mut state,
    );

    let fade_sample_count = usize::try_from(44_100 / 200).unwrap() * 2;
    assert!(stopped[..fade_sample_count]
        .iter()
        .any(|sample| sample.abs() > 0.0001));
    assert!(
        (stopped[0] - running[running.len() - 2]).abs() < 0.10,
        "W-30 transport-stop fade introduced a hard edge"
    );
    assert!(stopped[fade_sample_count..]
        .iter()
        .all(|sample| sample.abs() <= f32::EPSILON));

    let mut latched = [0.0_f32; 1_024];
    render_w30_preview_buffer(
        &mut latched,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            is_transport_running: false,
            position_beats: 8.25,
            ..running_render
        },
        &mut state,
    );
    assert!(latched.iter().all(|sample| sample.abs() <= f32::EPSILON));

    let mut manually_retriggered = [0.0_f32; 1_024];
    render_w30_preview_buffer(
        &mut manually_retriggered,
        44_100,
        2,
        &RealtimeW30PreviewRenderState {
            trigger_revision: 1,
            is_transport_running: false,
            position_beats: 8.25,
            ..running_render
        },
        &mut state,
    );
    assert!(manually_retriggered
        .iter()
        .any(|sample| sample.abs() > 0.0001));

    let mut resumed = [0.0_f32; 1_024];
    render_w30_preview_buffer(&mut resumed, 44_100, 2, &running_render, &mut state);
    assert!(resumed.iter().any(|sample| sample.abs() > 0.0001));
}

#[test]
fn transport_stop_fades_the_internal_resample_tap_and_stays_silent() {
    let mut state = W30ResampleTapCallbackState::default();
    let running_render = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::RawCapture),
        source_audio: positive_realtime_resample_source(),
        lineage_capture_count: 1,
        generation_depth: 0,
        variation: W30ResampleTapVariation::Base,
        variation_revision: 0,
        variation_intensity: 0.0,
        hard_policy: W30ResampleTapHardPolicy::Unavailable,
        hard_trigger_mask: 0,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_transient_contrast: 0.0,
        music_bus_level: 0.58,
        grit_level: 0.4,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 0.0,
    };
    let mut running = [0.0_f32; 1_024];
    render_w30_resample_tap_buffer(&mut running, 44_100, 2, &running_render, &mut state);
    assert!(running.iter().any(|sample| sample.abs() > 0.0001));
    let expected_beats = 512.0 * 128.0 / 60.0 / 44_100.0;
    assert!(
        (state.beat_position - expected_beats).abs() < 1.0e-9,
        "resample tap drifted from transport tempo: expected {expected_beats}, got {}",
        state.beat_position
    );

    let mut stopped = [0.0_f32; 1_024];
    render_w30_resample_tap_buffer(
        &mut stopped,
        44_100,
        2,
        &RealtimeW30ResampleTapState {
            is_transport_running: false,
            ..running_render
        },
        &mut state,
    );
    let fade_sample_count = usize::try_from(44_100 / 200).unwrap() * 2;
    assert!(stopped[..fade_sample_count]
        .iter()
        .any(|sample| sample.abs() > 0.0001));
    assert!(
        (stopped[0] - running[running.len() - 2]).abs() < 0.10,
        "resample-tap transport-stop fade introduced a hard edge"
    );
    assert!(stopped[fade_sample_count..]
        .iter()
        .all(|sample| sample.abs() <= f32::EPSILON));

    let mut latched = [0.0_f32; 1_024];
    render_w30_resample_tap_buffer(
        &mut latched,
        44_100,
        2,
        &RealtimeW30ResampleTapState {
            is_transport_running: false,
            ..running_render
        },
        &mut state,
    );
    assert!(latched.iter().all(|sample| sample.abs() <= f32::EPSILON));
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
fn w30_grit_adds_source_backed_bite_without_changing_the_clean_path() {
    let mut source = RealtimeW30PreviewRenderState {
        mode: W30PreviewRenderMode::LiveRecall,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
        trigger_revision: 0,
        trigger_velocity: 0.0,
        source_window_preview: RealtimeW30PreviewSampleWindow::default(),
        pad_playback: RealtimeW30PadPlaybackSampleWindow::default(),
        music_bus_level: 0.8,
        grit_level: 0.0,
        is_transport_running: true,
        tempo_bpm: 130.0,
        position_beats: 0.0,
    };
    source.pad_playback.sample_count = 1_024;
    source.pad_playback.source_sample_rate = 44_100;
    source.pad_playback.playback_frame_count = 1_024;
    source.pad_playback.loop_enabled = true;
    for (index, sample) in source
        .pad_playback
        .samples
        .iter_mut()
        .take(source.pad_playback.sample_count)
        .enumerate()
    {
        let phase = index as f32 / 32.0;
        *sample = phase.sin() * 0.38 + (phase * 3.7).sin() * 0.08;
    }

    let mut clean = vec![0.0; 4_096];
    let mut clean_state = W30PreviewCallbackState::default();
    render_w30_preview_buffer(&mut clean, 44_100, 2, &source, &mut clean_state);

    source.grit_level = 0.64;
    let mut bitten = vec![0.0; 4_096];
    let mut bitten_state = W30PreviewCallbackState::default();
    render_w30_preview_buffer(&mut bitten, 44_100, 2, &source, &mut bitten_state);

    let delta_rms = clean
        .iter()
        .zip(&bitten)
        .map(|(clean, bitten)| (bitten - clean).powi(2))
        .sum::<f32>()
        / clean.len() as f32;
    let delta_rms = delta_rms.sqrt();
    let clean_edge_rms = adjacent_sample_delta_rms(&clean);
    let bitten_edge_rms = adjacent_sample_delta_rms(&bitten);
    assert!(delta_rms > 0.02, "bite delta was too small: {delta_rms}");
    assert!(
        bitten_edge_rms > clean_edge_rms * 1.25,
        "bite did not add enough source-motion edge: clean={clean_edge_rms}, bitten={bitten_edge_rms}"
    );
    assert!(bitten.iter().all(|sample| sample.is_finite()));
    assert!(bitten.iter().all(|sample| sample.abs() < 1.0));
}

fn adjacent_sample_delta_rms(samples: &[f32]) -> f32 {
    let square_sum = samples
        .windows(2)
        .map(|window| (window[1] - window[0]).powi(2))
        .sum::<f32>();
    (square_sum / samples.len().saturating_sub(1).max(1) as f32).sqrt()
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
            source_audio: RealtimeW30ResampleSourceWindow::default(),
            lineage_capture_count: 0,
            generation_depth: 0,
            variation: W30ResampleTapVariation::Base,
            variation_revision: 0,
            variation_intensity: 0.0,
            hard_policy: W30ResampleTapHardPolicy::Unavailable,
            hard_trigger_mask: 0,
            hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            hard_transient_contrast: 0.0,
            music_bus_level: 0.64,
            grit_level: 0.4,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 0.0,
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
            source_audio: positive_realtime_resample_source(),
            lineage_capture_count: 2,
            generation_depth: 1,
            variation: W30ResampleTapVariation::Base,
            variation_revision: 0,
            variation_intensity: 0.0,
            hard_policy: W30ResampleTapHardPolicy::Unavailable,
            hard_trigger_mask: 0,
            hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            hard_transient_contrast: 0.0,
            music_bus_level: 0.58,
            grit_level: 0.62,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 0.0,
        },
        &mut state,
    );

    assert!(buffer.iter().any(|sample| sample.abs() > 0.0001));
}

#[test]
fn w30_resample_tap_is_deterministic_and_follows_source_material() {
    let source = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
        source_audio: positive_realtime_resample_source(),
        lineage_capture_count: 2,
        generation_depth: 1,
        variation: W30ResampleTapVariation::Base,
        variation_revision: 0,
        variation_intensity: 0.0,
        hard_policy: W30ResampleTapHardPolicy::Unavailable,
        hard_trigger_mask: 0,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_transient_contrast: 0.0,
        music_bus_level: 0.72,
        grit_level: 0.62,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 0.0,
    };
    let mut inverted = source;
    for sample in inverted
        .source_audio
        .samples
        .iter_mut()
        .take(inverted.source_audio.sample_count)
    {
        *sample = -*sample;
    }

    let render = |state: &RealtimeW30ResampleTapState| {
        let mut callback = W30ResampleTapCallbackState::default();
        let mut buffer = [0.0_f32; 1_024];
        render_w30_resample_tap_buffer(&mut buffer, 44_100, 2, state, &mut callback);
        buffer
    };
    let first = render(&source);
    let repeated = render(&source);
    let contrasting = render(&inverted);
    let mut raw_source = [0.0_f32; 1_024];
    for (frame, stereo) in raw_source.chunks_exact_mut(2).enumerate() {
        stereo.fill(source.source_audio.samples[frame]);
    }
    let dry_delta_rms = (first
        .iter()
        .zip(raw_source.iter())
        .map(|(rendered, raw)| (rendered - raw).powi(2))
        .sum::<f32>()
        / first.len() as f32)
        .sqrt();

    assert_eq!(first, repeated, "same source and state must render deterministically");
    assert_ne!(first, contrasting, "contrasting source PCM must change output");
    assert!(
        dry_delta_rms > 0.01,
        "tap collapsed to raw source PCM: delta RMS {dry_delta_rms}"
    );
    let polarity_product = first
        .iter()
        .zip(contrasting.iter())
        .map(|(left, right)| left * right)
        .sum::<f32>();
    assert!(
        polarity_product < -0.001,
        "tap output did not follow inverted source polarity: {polarity_product}"
    );
}

#[test]
fn post_resample_hard_damage_is_an_immediate_callback_safe_variation() {
    let mut source_audio = positive_realtime_resample_source();
    source_audio.source_frame_count = 88_200;
    let base = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
        source_audio,
        lineage_capture_count: 2,
        generation_depth: 1,
        variation: W30ResampleTapVariation::Base,
        variation_revision: 0,
        variation_intensity: 0.0,
        hard_policy: W30ResampleTapHardPolicy::SourceTransientChop,
        hard_trigger_mask: 0b1011_0111,
        hard_slice_cursors: [320, 2_112, 4_240, 6_016, 8_256, 10_048, 12_176, 14_352],
        hard_transient_contrast: 1.8,
        music_bus_level: 0.64,
        grit_level: 0.68,
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 0.0,
    };
    let hard = RealtimeW30ResampleTapState {
        variation: W30ResampleTapVariation::HardDamage,
        variation_revision: 7,
        variation_intensity: 0.82,
        ..base
    };
    let mut base_callback = W30ResampleTapCallbackState::default();
    let mut hard_callback = W30ResampleTapCallbackState::default();
    let mut warmup = [0.0_f32; 4_096];
    let mut warmup_copy = [0.0_f32; 4_096];
    render_w30_resample_tap_buffer(&mut warmup, 44_100, 2, &base, &mut base_callback);
    render_w30_resample_tap_buffer(&mut warmup_copy, 44_100, 2, &base, &mut hard_callback);

    let mut continued_base = [0.0_f32; 4_096];
    let mut activated_hard = [0.0_f32; 4_096];
    render_w30_resample_tap_buffer(
        &mut continued_base,
        44_100,
        2,
        &base,
        &mut base_callback,
    );
    render_w30_resample_tap_buffer(
        &mut activated_hard,
        44_100,
        2,
        &hard,
        &mut hard_callback,
    );

    let delta_rms = (continued_base
        .iter()
        .zip(activated_hard.iter())
        .map(|(base, hard)| (hard - base).powi(2))
        .sum::<f32>()
        / continued_base.len() as f32)
        .sqrt();
    assert!(
        delta_rms > 0.01,
        "hard gesture collapsed to the running base tap: {delta_rms}"
    );
    assert_eq!(hard_callback.last_variation_revision, 7);
    assert!(
        hard_callback.source_sample_cursor >= f64::from(hard.hard_slice_cursors[0]),
        "hard gesture did not jump to its source-derived local onset"
    );
}

#[test]
fn texture_bite_changes_timbre_without_imposing_the_transient_trigger_grid() {
    let base = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::RawCapture),
        source_audio: positive_realtime_resample_source(),
        lineage_capture_count: 1,
        generation_depth: 1,
        variation: W30ResampleTapVariation::Base,
        variation_revision: 0,
        variation_intensity: 0.0,
        hard_policy: W30ResampleTapHardPolicy::SourceTextureBite,
        hard_trigger_mask: 0,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_transient_contrast: 0.5,
        music_bus_level: 0.72,
        grit_level: 0.5,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    };
    let hard = RealtimeW30ResampleTapState {
        variation: W30ResampleTapVariation::HardDamage,
        variation_revision: 9,
        variation_intensity: 0.82,
        ..base
    };
    let mut base_callback = W30ResampleTapCallbackState::default();
    let mut hard_callback = W30ResampleTapCallbackState::default();
    let mut base_warmup = [0.0_f32; 4_096];
    let mut hard_warmup = [0.0_f32; 4_096];
    render_w30_resample_tap_buffer(&mut base_warmup, 44_100, 2, &base, &mut base_callback);
    render_w30_resample_tap_buffer(&mut hard_warmup, 44_100, 2, &base, &mut hard_callback);

    let mut continued_base = [0.0_f32; 4_096];
    let mut activated_hard = [0.0_f32; 4_096];
    render_w30_resample_tap_buffer(
        &mut continued_base,
        44_100,
        2,
        &base,
        &mut base_callback,
    );
    render_w30_resample_tap_buffer(
        &mut activated_hard,
        44_100,
        2,
        &hard,
        &mut hard_callback,
    );

    let delta_rms = (continued_base
        .iter()
        .zip(activated_hard.iter())
        .map(|(base, hard)| (hard - base).powi(2))
        .sum::<f32>()
        / continued_base.len() as f32)
        .sqrt();
    assert!(delta_rms > 0.005);
    assert!((0_i64..16).all(|step| !should_trigger_w30_resample_step(&hard, step)));
    assert!(
        hard_callback.source_sample_cursor > 0.0,
        "texture policy should preserve continuous source playback"
    );
}

#[test]
fn resample_base_preserves_phrase_flow_while_hard_damage_owns_the_chopped_role() {
    let mut source_audio = positive_realtime_resample_source();
    source_audio.source_frame_count = 88_200;
    let base = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::RawCapture),
        source_audio,
        lineage_capture_count: 1,
        generation_depth: 1,
        variation: W30ResampleTapVariation::Base,
        variation_revision: 0,
        variation_intensity: 0.0,
        hard_policy: W30ResampleTapHardPolicy::SourceTransientChop,
        hard_trigger_mask: 0b1011_0111,
        hard_slice_cursors: [320, 2_112, 4_240, 6_016, 8_256, 10_048, 12_176, 14_352],
        hard_transient_contrast: 1.8,
        music_bus_level: 0.8,
        grit_level: 0.5,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    };
    let hard = RealtimeW30ResampleTapState {
        variation: W30ResampleTapVariation::HardDamage,
        variation_revision: 11,
        variation_intensity: 0.82,
        ..base
    };

    assert!((0_i64..16).all(|step| !should_trigger_w30_resample_step(&base, step)));
    let hard_steps = (0_i64..8)
        .filter(|step| should_trigger_w30_resample_step(&hard, *step))
        .collect::<Vec<_>>();
    assert_eq!(hard_steps, vec![0, 1, 2, 4, 5, 7]);
    let hard_cursors = (0_i64..8)
        .map(|step| w30_resample_step_cursor(&hard, step))
        .collect::<Vec<_>>();
    assert_eq!(
        hard_cursors,
        hard.hard_slice_cursors
            .into_iter()
            .map(f64::from)
            .collect::<Vec<_>>()
    );
    assert_eq!(w30_resample_decay(&base), 1.0);
    assert_eq!(w30_resample_decay(&hard), 1.0);
}

#[test]
fn transient_chop_starts_the_selected_source_onset_on_the_eighth_note_grid() {
    let sample_rate = 44_100_u32;
    let tempo_bpm = 120.0_f32;
    let start_beats = 0.49_f64;
    let target_beats = 0.5_f64;
    let selected_cursor = 2_048_u16;
    let mut source_audio = RealtimeW30ResampleSourceWindow {
        source_start_frame: 0,
        source_sample_rate: sample_rate,
        source_frame_count: W30_RESAMPLE_SOURCE_WINDOW_LEN as u64,
        sample_count: W30_RESAMPLE_SOURCE_WINDOW_LEN,
        samples: [0.0; W30_RESAMPLE_SOURCE_WINDOW_LEN],
    };
    source_audio.samples[usize::from(selected_cursor)] = 0.8;
    source_audio.samples[usize::from(selected_cursor) + 1] = 0.4;
    let render = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
        source_audio,
        lineage_capture_count: 2,
        generation_depth: 1,
        variation: W30ResampleTapVariation::HardDamage,
        variation_revision: 7,
        variation_intensity: 0.82,
        hard_policy: W30ResampleTapHardPolicy::SourceTransientChop,
        hard_trigger_mask: 0b0000_0010,
        hard_slice_cursors: [0, selected_cursor, 0, 0, 0, 0, 0, 0],
        hard_transient_contrast: 1.8,
        music_bus_level: 0.8,
        grit_level: 0.5,
        is_transport_running: true,
        tempo_bpm,
        position_beats: start_beats,
    };
    let mut callback = W30ResampleTapCallbackState::default();
    let mut buffer = [0.0_f32; 2_048];

    render_w30_resample_tap_buffer(&mut buffer, sample_rate, 2, &render, &mut callback);

    let first_audible_frame = buffer
        .chunks_exact(2)
        .position(|frame| frame[0].abs() > 0.001)
        .expect("source-derived onset should render");
    let expected_grid_frame = ((target_beats - start_beats) * 60.0
        * f64::from(sample_rate)
        / f64::from(tempo_bpm))
    .ceil() as usize;
    assert!(
        first_audible_frame.abs_diff(expected_grid_frame) <= 1,
        "source onset landed off-grid: expected frame {expected_grid_frame}, got {first_audible_frame}"
    );
}

#[test]
fn base_resample_wraps_a_full_bar_artifact_on_the_transport_bar_boundary() {
    let output_sample_rate = 48_000_u32;
    let tempo_bpm = 130.0_f32;
    let expected_bar_frames =
        (4.0 * 60.0 / f64::from(tempo_bpm) * f64::from(output_sample_rate)).round() as usize;
    let mut source_audio = RealtimeW30ResampleSourceWindow {
        source_start_frame: 0,
        source_sample_rate: 44_100,
        source_frame_count: 81_237,
        sample_count: W30_RESAMPLE_SOURCE_WINDOW_LEN,
        samples: [0.0; W30_RESAMPLE_SOURCE_WINDOW_LEN],
    };
    source_audio.samples[0] = 0.8;
    let render = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
        source_audio,
        lineage_capture_count: 1,
        generation_depth: 1,
        variation: W30ResampleTapVariation::Base,
        variation_revision: 0,
        variation_intensity: 0.0,
        hard_policy: W30ResampleTapHardPolicy::SourceTransientChop,
        hard_trigger_mask: 0b1101_0111,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_transient_contrast: 5.0,
        music_bus_level: 0.8,
        grit_level: 0.82,
        is_transport_running: true,
        tempo_bpm,
        position_beats: 0.0,
    };
    let mut callback = W30ResampleTapCallbackState::default();
    let mut buffer = vec![0.0_f32; (expected_bar_frames + 32) * 2];

    render_w30_resample_tap_buffer(
        &mut buffer,
        output_sample_rate,
        2,
        &render,
        &mut callback,
    );

    let search_start = expected_bar_frames - 16;
    let search_end = expected_bar_frames + 16;
    let repeated_peak_frame = buffer[search_start * 2..search_end * 2]
        .chunks_exact(2)
        .enumerate()
        .max_by(|(_, left), (_, right)| left[0].abs().total_cmp(&right[0].abs()))
        .map(|(offset, _)| search_start + offset)
        .expect("bar-boundary search window");
    assert!(
        repeated_peak_frame.abs_diff(expected_bar_frames) <= 1,
        "Base loop restarted off the transport bar: expected frame {expected_bar_frames}, got {repeated_peak_frame}"
    );
}

#[test]
fn w30_resample_tap_stays_silent_without_source_audio() {
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
            source_audio: RealtimeW30ResampleSourceWindow::default(),
            lineage_capture_count: 2,
            generation_depth: 1,
            variation: W30ResampleTapVariation::Base,
            variation_revision: 0,
            variation_intensity: 0.0,
            hard_policy: W30ResampleTapHardPolicy::Unavailable,
            hard_trigger_mask: 0,
            hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            hard_transient_contrast: 0.0,
            music_bus_level: 0.58,
            grit_level: 0.62,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 0.0,
        },
        &mut state,
    );

    assert!(buffer.iter().all(|sample| sample.abs() <= f32::EPSILON));
}

#[test]
fn w30_resample_tap_does_not_invent_grid_progress_without_a_valid_tempo() {
    let mut state = W30ResampleTapCallbackState::default();
    let mut buffer = [0.0_f32; 1_024];
    let render = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
        source_audio: positive_realtime_resample_source(),
        lineage_capture_count: 2,
        generation_depth: 1,
        variation: W30ResampleTapVariation::Base,
        variation_revision: 0,
        variation_intensity: 0.0,
        hard_policy: W30ResampleTapHardPolicy::Unavailable,
        hard_trigger_mask: 0,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_transient_contrast: 0.0,
        music_bus_level: 0.58,
        grit_level: 0.62,
        is_transport_running: true,
        tempo_bpm: 0.0,
        position_beats: 0.0,
    };

    render_w30_resample_tap_buffer(&mut buffer, 44_100, 2, &render, &mut state);

    assert_eq!(state.beat_position, 0.0);
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
            source_audio: positive_realtime_resample_source(),
            lineage_capture_count: 3,
            generation_depth: 2,
            variation: W30ResampleTapVariation::Base,
            variation_revision: 0,
            variation_intensity: 0.0,
            hard_policy: W30ResampleTapHardPolicy::Unavailable,
            hard_trigger_mask: 0,
            hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            hard_transient_contrast: 0.0,
            music_bus_level: 0.0,
            grit_level: 0.7,
            is_transport_running: false,
            tempo_bpm: 128.0,
            position_beats: 0.0,
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
