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
        source_revision: 1,
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
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 0,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_lengths: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_gesture: Default::default(),
        hard_transient_contrast: 0.0,
        music_bus_level: 0.58,
        hard_attack_bite: Default::default(),
        hard_low_impact: Default::default(),
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
fn fresh_stopped_resample_tap_has_no_implicit_audition() {
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
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 0,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_lengths: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_gesture: Default::default(),
        hard_transient_contrast: 0.0,
        music_bus_level: 0.72,
        hard_attack_bite: Default::default(),
        hard_low_impact: Default::default(),
        grit_level: 0.6,
        is_transport_running: false,
        tempo_bpm: 128.0,
        position_beats: 7.25,
    };

    render_w30_resample_tap_buffer(&mut buffer, 44_100, 2, &render, &mut state);

    assert!(buffer.iter().all(|sample| sample.abs() <= f32::EPSILON));
    assert!(!state.was_active);
    assert_eq!(state.beat_position, 7.25);
}

#[test]
fn fresh_mid_bar_hard_start_does_not_leak_slot_zero_into_an_inactive_slot() {
    let mut state = W30ResampleTapCallbackState::default();
    let mut buffer = [0.0_f32; 1_024];
    let mut source_audio = positive_realtime_resample_source();
    source_audio.samples.fill(0.0);
    source_audio.samples[0] = 0.8;
    source_audio.samples[1] = 0.4;
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
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 0b0000_0001,
        hard_slice_cursors: [0, 2_112, 4_240, 6_016, 8_256, 10_048, 12_176, 14_352],
        hard_attack_lengths: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_gesture: Default::default(),
        hard_transient_contrast: 1.8,
        music_bus_level: 0.72,
        hard_attack_bite: Default::default(),
        hard_low_impact: Default::default(),
        grit_level: 0.6,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.5,
    };

    render_w30_resample_tap_buffer(&mut buffer, 44_100, 2, &render, &mut state);

    assert!(buffer.iter().all(|sample| sample.abs() <= f32::EPSILON));
    assert_eq!(state.last_step, 1);
    assert_eq!(state.envelope, 1.0);
}

#[test]
fn running_seek_resynchronizes_hard_step_and_does_not_restart_slot_zero() {
    let mut state = W30ResampleTapCallbackState::default();
    let mut source_audio = positive_realtime_resample_source();
    source_audio.samples.fill(0.0);
    source_audio.samples[0] = 0.8;
    source_audio.samples[1] = 0.4;
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
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 0b0000_0001,
        hard_slice_cursors: [0, 2_112, 4_240, 6_016, 8_256, 10_048, 12_176, 14_352],
        hard_attack_lengths: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_gesture: Default::default(),
        hard_transient_contrast: 1.8,
        music_bus_level: 0.72,
        hard_attack_bite: Default::default(),
        hard_low_impact: Default::default(),
        grit_level: 0.6,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    };
    let mut initial = [0.0_f32; 1_024];
    render_w30_resample_tap_buffer(&mut initial, 44_100, 2, &render, &mut state);
    assert!(initial.iter().any(|sample| sample.abs() > 0.0001));

    let mut after_seek = [0.0_f32; 1_024];
    render_w30_resample_tap_buffer(
        &mut after_seek,
        44_100,
        2,
        &RealtimeW30ResampleTapState {
            position_beats: 0.5,
            ..render
        },
        &mut state,
    );

    let transition_sample_count = usize::try_from(44_100 / 200).unwrap() * 2;
    assert_eq!(state.last_step, 1);
    assert!(after_seek[transition_sample_count..]
        .iter()
        .all(|sample| sample.abs() <= f32::EPSILON));
}

#[test]
fn active_source_revision_resets_history_behind_a_click_safe_transition() {
    let mut state = W30ResampleTapCallbackState::default();
    let source = positive_realtime_resample_source();
    let render = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
        source_audio: source,
        lineage_capture_count: 2,
        generation_depth: 1,
        variation: W30ResampleTapVariation::Base,
        variation_revision: 0,
        variation_intensity: 0.0,
        hard_policy: W30ResampleTapHardPolicy::Unavailable,
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 0,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_lengths: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_gesture: Default::default(),
        hard_transient_contrast: 0.0,
        music_bus_level: 0.72,
        hard_attack_bite: Default::default(),
        hard_low_impact: Default::default(),
        grit_level: 0.6,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    };
    let mut before = [0.0_f32; 1_024];
    render_w30_resample_tap_buffer(&mut before, 44_100, 2, &render, &mut state);

    let mut replacement_source = source;
    replacement_source.source_revision = 2;
    for sample in &mut replacement_source.samples {
        *sample = -*sample;
    }
    let mut after = [0.0_f32; 1_024];
    render_w30_resample_tap_buffer(
        &mut after,
        44_100,
        2,
        &RealtimeW30ResampleTapState {
            source_audio: replacement_source,
            position_beats: state.beat_position,
            ..render
        },
        &mut state,
    );

    assert_eq!(state.last_source_revision, 2);
    assert!(
        (after[0] - before[before.len() - 2]).abs() < 0.02,
        "source replacement introduced a hard boundary jump"
    );
    assert!(after.iter().any(|sample| sample.abs() > 0.0001));
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
            hard_output_gain: 1.0,
            hard_hit_window_compensation_gain: 1.0,
            hard_impact_body_eq_gain_db: 0.0,
            hard_trigger_mask: 0,
            hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            hard_attack_lengths: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            hard_gesture: Default::default(),
            hard_transient_contrast: 0.0,
            music_bus_level: 0.64,
            hard_attack_bite: Default::default(),
            hard_low_impact: Default::default(),
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
            hard_output_gain: 1.0,
            hard_hit_window_compensation_gain: 1.0,
            hard_impact_body_eq_gain_db: 0.0,
            hard_trigger_mask: 0,
            hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            hard_attack_lengths: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            hard_gesture: Default::default(),
            hard_transient_contrast: 0.0,
            music_bus_level: 0.58,
            hard_attack_bite: Default::default(),
            hard_low_impact: Default::default(),
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
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 0,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_lengths: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_gesture: Default::default(),
        hard_transient_contrast: 0.0,
        music_bus_level: 0.72,
        hard_attack_bite: Default::default(),
        hard_low_impact: Default::default(),
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
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 0b1011_0111,
        hard_slice_cursors: [320, 2_112, 4_240, 6_016, 8_256, 10_048, 12_176, 14_352],
        hard_attack_lengths: [256; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_gesture: Default::default(),
        hard_transient_contrast: 1.8,
        music_bus_level: 0.64,
        hard_attack_bite: Default::default(),
        hard_low_impact: Default::default(),
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
        hard_callback.hard_attack_sample_cursor >= f64::from(hard.hard_slice_cursors[0]),
        "hard gesture did not start its source-derived local attack"
    );
}

#[test]
fn h13_reverse_pickup_moves_backwards_into_the_registered_impact() {
    let sample_rate = 48_000_u32;
    let mut source_audio = positive_realtime_resample_source();
    source_audio.source_frame_count = 96_000;
    let mut render = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
        source_audio,
        lineage_capture_count: 2,
        generation_depth: 1,
        variation: W30ResampleTapVariation::HardDamage,
        variation_revision: 13,
        variation_intensity: 0.82,
        hard_policy: W30ResampleTapHardPolicy::SourceTransientChop,
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 0b0000_0001,
        hard_slice_cursors: [1_000, 0, 0, 0, 0, 0, 0, 14_000],
        hard_attack_lengths: [512; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_bite: Default::default(),
        hard_low_impact: Default::default(),
        hard_gesture: W30ResampleHardGesturePlan {
            recipe: W30ResampleHardGestureRecipe::SourceReverseIntoImpactV1,
            impact_slot: 0,
            pickup_slot: 7,
            body_gain: crate::w30::W30_RESAMPLE_H13_MIN_BODY_GAIN,
            impact_level_compensation: 0.95,
            pickup_gain: 0.4,
            selected_head_rms: 0.4,
            selected_body_rms: 0.2,
        },
        hard_transient_contrast: 2.0,
        music_bus_level: 0.72,
        grit_level: 0.5,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 3.5,
    };
    let mut callback = W30ResampleTapCallbackState::default();
    let mut enter_pickup = vec![0.0_f32; sample_rate as usize * 140 / 1_000 * 2];
    render_w30_resample_tap_buffer(
        &mut enter_pickup,
        sample_rate,
        2,
        &render,
        &mut callback,
    );
    assert!(callback.hard_reverse_pickup_frames_remaining > 0);
    let first_cursor = callback.hard_reverse_pickup_cursor;
    render.position_beats = callback.beat_position;

    let mut continue_pickup = vec![0.0_f32; sample_rate as usize * 20 / 1_000 * 2];
    render_w30_resample_tap_buffer(
        &mut continue_pickup,
        sample_rate,
        2,
        &render,
        &mut callback,
    );

    assert!(
        callback.hard_reverse_pickup_cursor < first_cursor,
        "registered pickup cursor did not move backwards"
    );
    assert!(continue_pickup.iter().any(|sample| sample.abs() > 0.001));
}

#[test]
fn transient_h1_changes_the_attack_without_deleting_the_following_body() {
    let sample_rate = 44_100_u32;
    let frame_count = sample_rate as usize / 5;
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
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 0b0000_0001,
        hard_slice_cursors: [2_112, 0, 0, 0, 0, 0, 0, 0],
        hard_attack_lengths: [512, 0, 0, 0, 0, 0, 0, 0],
        hard_gesture: Default::default(),
        hard_transient_contrast: 1.8,
        music_bus_level: 0.72,
        hard_attack_bite: Default::default(),
        hard_low_impact: Default::default(),
        grit_level: 0.6,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    };
    let hard = RealtimeW30ResampleTapState {
        variation: W30ResampleTapVariation::HardDamage,
        variation_revision: 7,
        variation_intensity: 0.82,
        ..base
    };
    let mut base_buffer = vec![0.0_f32; frame_count * 2];
    let mut hard_buffer = vec![0.0_f32; frame_count * 2];
    render_w30_resample_tap_buffer(
        &mut base_buffer,
        sample_rate,
        2,
        &base,
        &mut W30ResampleTapCallbackState::default(),
    );
    render_w30_resample_tap_buffer(
        &mut hard_buffer,
        sample_rate,
        2,
        &hard,
        &mut W30ResampleTapCallbackState::default(),
    );

    let window_rms = |buffer: &[f32], start_ms: usize, end_ms: usize| {
        let start = start_ms * sample_rate as usize / 1_000 * 2;
        let end = end_ms * sample_rate as usize / 1_000 * 2;
        (buffer[start..end]
            .iter()
            .map(|sample| sample * sample)
            .sum::<f32>()
            / (end - start) as f32)
            .sqrt()
    };
    let attack_delta = window_rms(
        &hard_buffer
            .iter()
            .zip(&base_buffer)
            .map(|(hard, base)| hard - base)
            .collect::<Vec<_>>(),
        0,
        40,
    );
    let base_body = window_rms(&base_buffer, 40, 120);
    let hard_body = window_rms(&hard_buffer, 40, 120);
    let hard_late_body = window_rms(&hard_buffer, 120, 200);

    assert!(attack_delta > 0.01, "H1 attack path stayed too close to Base");
    assert!(
        hard_body / base_body.max(1.0e-9) >= 0.75,
        "H1 removed the 40–120 ms body: base={base_body}, hard={hard_body}"
    );
    assert!(
        hard_late_body > 0.01,
        "H1 collapsed the retained 120–200 ms body"
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
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 0,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_lengths: [256; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_gesture: Default::default(),
        hard_transient_contrast: 0.5,
        music_bus_level: 0.72,
        hard_attack_bite: Default::default(),
        hard_low_impact: Default::default(),
        grit_level: 0.5,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    };
    let uncalibrated_hard = RealtimeW30ResampleTapState {
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
        &uncalibrated_hard,
        &mut hard_callback,
    );

    let rms = |samples: &[f32]| {
        (samples.iter().map(|sample| sample * sample).sum::<f32>()
            / samples.len().max(1) as f32)
            .sqrt()
    };
    let raw_ratio = rms(&activated_hard) / rms(&continued_base).max(1.0e-9);
    let hard = RealtimeW30ResampleTapState {
        hard_output_gain: (1.05 / raw_ratio).clamp(0.25, 1.25),
        ..uncalibrated_hard
    };
    base_callback = W30ResampleTapCallbackState::default();
    hard_callback = W30ResampleTapCallbackState::default();
    base_warmup.fill(0.0);
    hard_warmup.fill(0.0);
    continued_base.fill(0.0);
    activated_hard.fill(0.0);
    render_w30_resample_tap_buffer(&mut base_warmup, 44_100, 2, &base, &mut base_callback);
    render_w30_resample_tap_buffer(&mut hard_warmup, 44_100, 2, &base, &mut hard_callback);
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
        delta_rms > 0.02,
        "texture-bite Hard variation stayed too close to Base: {delta_rms}"
    );
    let base_rms = rms(&continued_base);
    let hard_rms = rms(&activated_hard);
    assert!(
        hard_rms / base_rms <= 1.15,
        "texture-bite policy output compensation failed: base={base_rms}, hard={hard_rms}"
    );
    assert!((0_i64..16).all(|step| !should_trigger_w30_resample_step(&hard, step)));
    assert!(
        hard_callback.source_sample_cursor > 0.0,
        "texture policy should preserve continuous source playback"
    );
}

#[test]
fn unavailable_hard_policy_preserves_the_base_output_exactly() {
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
        hard_policy: W30ResampleTapHardPolicy::Unavailable,
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 0,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_lengths: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_gesture: Default::default(),
        hard_transient_contrast: 0.0,
        music_bus_level: 0.72,
        hard_attack_bite: Default::default(),
        hard_low_impact: Default::default(),
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
    let mut base_buffer = [0.0_f32; 8_192];
    let mut hard_buffer = [0.0_f32; 8_192];

    render_w30_resample_tap_buffer(
        &mut base_buffer,
        44_100,
        2,
        &base,
        &mut W30ResampleTapCallbackState::default(),
    );
    render_w30_resample_tap_buffer(
        &mut hard_buffer,
        44_100,
        2,
        &hard,
        &mut W30ResampleTapCallbackState::default(),
    );

    assert_eq!(hard_buffer, base_buffer);
}

#[test]
fn resample_base_preserves_phrase_flow_while_hard_uses_source_attack_body_regions() {
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
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 0b1011_0111,
        hard_slice_cursors: [320, 2_112, 4_240, 6_016, 8_256, 10_048, 12_176, 14_352],
        hard_attack_lengths: [256; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_gesture: Default::default(),
        hard_transient_contrast: 1.8,
        music_bus_level: 0.8,
        hard_attack_bite: Default::default(),
        hard_low_impact: Default::default(),
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
    assert_eq!(w30_resample_decay(&base, 48_000), 1.0);
    assert_eq!(w30_resample_decay(&hard, 48_000), 1.0);
}

#[test]
fn aligned_impact_v6_uses_each_source_onset_while_v5_remains_sample_stable() {
    let source_audio = positive_realtime_resample_source();
    let mut hard = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::RawCapture),
        source_audio,
        lineage_capture_count: 1,
        generation_depth: 1,
        variation: W30ResampleTapVariation::HardDamage,
        variation_revision: 1,
        variation_intensity: 0.82,
        hard_policy: W30ResampleTapHardPolicy::SourceTransientChop,
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 0xff,
        hard_slice_cursors: [320, 2_112, 4_240, 6_016, 8_256, 10_048, 12_176, 14_352],
        hard_attack_lengths: [128; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_bite: Default::default(),
        hard_low_impact: W30ResampleLowImpactPlan {
            recipe: W30ResampleLowImpactRecipe::SourcePhaseAlignedImpactV6,
            selected_onset_cursor: 4_240,
            ..W30ResampleLowImpactPlan::default()
        },
        hard_gesture: Default::default(),
        hard_transient_contrast: 1.8,
        music_bus_level: 0.8,
        grit_level: 0.5,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    };

    let v6_cursors = (0_i64..8)
        .map(|step| w30_resample_step_cursor(&hard, step))
        .collect::<Vec<_>>();
    assert_eq!(
        v6_cursors,
        hard.hard_slice_cursors
            .into_iter()
            .map(f64::from)
            .collect::<Vec<_>>()
    );

    hard.hard_low_impact.recipe = W30ResampleLowImpactRecipe::SourceAlignedImpactV5;
    let v5_cursors = (0_i64..8)
        .map(|step| w30_resample_step_cursor(&hard, step))
        .collect::<Vec<_>>();
    assert_eq!(v5_cursors, vec![4_240.0; 8]);
}

#[test]
fn source_selected_bite_changes_the_sustained_hard_gesture_without_affecting_base() {
    let mut source_audio = positive_realtime_resample_source();
    source_audio.source_frame_count = 88_200;
    let hard_without_bite = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
        source_audio,
        lineage_capture_count: 1,
        generation_depth: 1,
        variation: W30ResampleTapVariation::HardDamage,
        variation_revision: 17,
        variation_intensity: 0.82,
        hard_policy: W30ResampleTapHardPolicy::SourceTransientChop,
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 0b0000_0001,
        hard_slice_cursors: [320, 0, 0, 0, 0, 0, 0, 0],
        hard_attack_lengths: [64, 0, 0, 0, 0, 0, 0, 0],
        hard_attack_bite: Default::default(),
        hard_low_impact: Default::default(),
        hard_gesture: Default::default(),
        hard_transient_contrast: 2.0,
        music_bus_level: 0.72,
        grit_level: 0.5,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    };
    let hard_with_bite = RealtimeW30ResampleTapState {
        hard_attack_bite: W30ResampleAttackBitePlan {
            band: W30ResampleAttackBiteBand::Presence,
            input_gain: 4.0,
            output_gain: 0.75,
        },
        ..hard_without_bite
    };
    let mut without = vec![0.0_f32; 19_200];
    let mut with = vec![0.0_f32; 19_200];
    render_w30_resample_tap_buffer(
        &mut without,
        48_000,
        2,
        &hard_without_bite,
        &mut W30ResampleTapCallbackState::default(),
    );
    render_w30_resample_tap_buffer(
        &mut with,
        48_000,
        2,
        &hard_with_bite,
        &mut W30ResampleTapCallbackState::default(),
    );

    let body_start = 48_000 * 40 / 1_000 * 2;
    let body_end = 48_000 * 100 / 1_000 * 2;
    let sustained_delta_rms = (without[body_start..body_end]
        .iter()
        .zip(with[body_start..body_end].iter())
        .map(|(dry, bitten)| (bitten - dry).powi(2))
        .sum::<f32>()
        / (body_end - body_start) as f32)
        .sqrt();
    assert!(
        sustained_delta_rms > 0.005,
        "H4 RMS-matched nonlinear residual did not persist beyond the attack: {sustained_delta_rms}"
    );
    assert!(with.iter().all(|sample| sample.is_finite()));
    assert!(with.iter().all(|sample| sample.abs() <= 0.92));

    let base_without_bite = RealtimeW30ResampleTapState {
        variation: W30ResampleTapVariation::Base,
        variation_revision: 0,
        ..hard_without_bite
    };
    let base_with_bite = RealtimeW30ResampleTapState {
        hard_attack_bite: hard_with_bite.hard_attack_bite,
        ..base_without_bite
    };
    let mut base_without = vec![0.0_f32; 8_192];
    let mut base_with = vec![0.0_f32; 8_192];
    render_w30_resample_tap_buffer(
        &mut base_without,
        48_000,
        2,
        &base_without_bite,
        &mut W30ResampleTapCallbackState::default(),
    );
    render_w30_resample_tap_buffer(
        &mut base_with,
        48_000,
        2,
        &base_with_bite,
        &mut W30ResampleTapCallbackState::default(),
    );
    assert_eq!(
        base_without, base_with,
        "Hard bite plan must not alter the accepted Base path"
    );
}

#[test]
fn source_grit_slam_v1_is_deterministic_held_and_hard_only() {
    let hard = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
        source_audio: positive_realtime_resample_source(),
        lineage_capture_count: 1,
        generation_depth: 1,
        variation: W30ResampleTapVariation::HardDamage,
        variation_revision: 1,
        variation_intensity: 0.82,
        hard_policy: W30ResampleTapHardPolicy::SourceTransientChop,
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 1,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_lengths: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_bite: Default::default(),
        hard_low_impact: Default::default(),
        hard_gesture: Default::default(),
        hard_transient_contrast: 2.0,
        music_bus_level: 0.72,
        grit_level: 0.5,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    };
    let mut state = W30ResampleTapCallbackState::default();
    let first = w30_resample_hard_grit_sample(&hard, &mut state, 48_000, 0.12);
    let expected_first = 4.0 / 31.0;
    assert!((first - expected_first).abs() < f32::EPSILON);
    for _ in 0..5 {
        assert_eq!(
            w30_resample_hard_grit_sample(&hard, &mut state, 48_000, 0.91),
            expected_first,
            "8 kHz recipe must hold one quantized sample for six 48 kHz frames"
        );
    }
    assert_eq!(
        w30_resample_hard_grit_sample(&hard, &mut state, 48_000, 0.91),
        28.0 / 31.0
    );

    let base = RealtimeW30ResampleTapState {
        variation: W30ResampleTapVariation::Base,
        ..hard
    };
    assert_eq!(
        w30_resample_hard_grit_sample(&base, &mut state, 48_000, 0.123_456),
        0.123_456,
        "the accepted Base path must bypass the hard grit recipe"
    );
    assert_eq!(state.hard_grit_hold_frames_remaining, 0);
}

#[test]
fn source_low_transient_punch_is_parallel_hard_only_audio() {
    let hard = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
        source_audio: positive_realtime_resample_source(),
        lineage_capture_count: 1,
        generation_depth: 1,
        variation: W30ResampleTapVariation::HardDamage,
        variation_revision: 1,
        variation_intensity: 0.82,
        hard_policy: W30ResampleTapHardPolicy::SourceTransientChop,
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 1,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_lengths: [64; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_bite: Default::default(),
        hard_low_impact: W30ResampleLowImpactPlan {
            recipe: W30ResampleLowImpactRecipe::SourceLowTransientPunchV1,
            low_band_attack_share: 0.4,
            low_band_attack_over_body: 2.0,
            low_band_attack_over_source: 0.8,
            ..W30ResampleLowImpactPlan::default()
        },
        hard_gesture: Default::default(),
        hard_transient_contrast: 2.0,
        music_bus_level: 0.72,
        grit_level: 0.5,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    };
    let mut state = W30ResampleTapCallbackState {
        hard_attack_mix: 1.0,
        hard_low_impact_low_alpha: 0.1,
        hard_low_impact_high_alpha: 0.3,
        ..Default::default()
    };
    assert_eq!(
        w30_resample_low_impact_sample(&hard, &mut state, 0.0, 0.1),
        0.1
    );
    let punched = w30_resample_low_impact_sample(&hard, &mut state, 0.8, 0.1);
    assert!(punched > 0.1, "source low transient was not returned in parallel");

    let base = RealtimeW30ResampleTapState {
        variation: W30ResampleTapVariation::Base,
        ..hard
    };
    assert_eq!(
        w30_resample_low_impact_sample(&base, &mut state, 0.8, 0.1),
        0.1,
        "the accepted Base path must bypass low-impact processing"
    );
}

#[test]
fn source_kick_impact_v2_lifts_body_and_head_without_damage_processing() {
    let hard = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
        source_audio: positive_realtime_resample_source(),
        lineage_capture_count: 1,
        generation_depth: 1,
        variation: W30ResampleTapVariation::HardDamage,
        variation_revision: 1,
        variation_intensity: 0.82,
        hard_policy: W30ResampleTapHardPolicy::SourceTransientChop,
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 1,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_lengths: [64; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_bite: Default::default(),
        hard_low_impact: W30ResampleLowImpactPlan {
            recipe: W30ResampleLowImpactRecipe::SourceKickImpactV2,
            low_band_attack_share: 0.4,
            low_band_attack_over_body: 2.0,
            low_band_attack_over_source: 0.8,
            ..W30ResampleLowImpactPlan::default()
        },
        hard_gesture: Default::default(),
        hard_transient_contrast: 2.0,
        music_bus_level: 0.72,
        grit_level: 0.5,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    };
    let mut state = W30ResampleTapCallbackState {
        hard_attack_mix: 1.0,
        hard_attack_head_mix: 1.0,
        hard_low_impact_low_alpha: 0.1,
        hard_low_impact_high_alpha: 0.3,
        hard_impact_presence_low_alpha: 0.2,
        hard_impact_presence_high_alpha: 0.6,
        ..Default::default()
    };
    assert_eq!(
        w30_resample_kick_impact_v2_sample(&hard, &mut state, 0.0),
        0.0
    );
    let punched = w30_resample_kick_impact_v2_sample(&hard, &mut state, 0.6);
    assert!(
        punched > 0.6,
        "source kick head/body did not become more forceful"
    );

    let base = RealtimeW30ResampleTapState {
        variation: W30ResampleTapVariation::Base,
        ..hard
    };
    assert_eq!(
        w30_resample_kick_impact_v2_sample(&base, &mut state, 0.6),
        0.6,
        "the accepted Base path must bypass kick-impact v2"
    );
}

#[test]
fn source_hit_shaper_v3_lifts_audible_head_and_following_body_windows() {
    let recipe = W30ResampleLowImpactRecipe::SourceHitShaperV3;
    let hard = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
        source_audio: positive_realtime_resample_source(),
        lineage_capture_count: 1,
        generation_depth: 1,
        variation: W30ResampleTapVariation::HardDamage,
        variation_revision: 1,
        variation_intensity: 0.82,
        hard_policy: W30ResampleTapHardPolicy::SourceTransientChop,
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 1,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_lengths: [64; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_bite: Default::default(),
        hard_low_impact: W30ResampleLowImpactPlan {
            recipe,
            low_band_attack_share: 0.4,
            low_band_attack_over_body: 2.0,
            low_band_attack_over_source: 0.8,
            ..W30ResampleLowImpactPlan::default()
        },
        hard_gesture: Default::default(),
        hard_transient_contrast: 2.0,
        music_bus_level: 0.72,
        grit_level: 0.5,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    };
    let sample_rate = 48_000_u32;
    let mut state = W30ResampleTapCallbackState {
        hard_low_impact_low_alpha: 1.0
            - (-std::f32::consts::TAU * 45.0 / sample_rate as f32).exp(),
        hard_low_impact_high_alpha: 1.0
            - (-std::f32::consts::TAU * 180.0 / sample_rate as f32).exp(),
        hard_impact_presence_low_alpha: 1.0
            - (-std::f32::consts::TAU * 900.0 / sample_rate as f32).exp(),
        hard_impact_presence_high_alpha: 1.0
            - (-std::f32::consts::TAU * 3_600.0 / sample_rate as f32).exp(),
        ..Default::default()
    };
    configure_w30_resample_low_impact(&hard, &mut state, sample_rate);
    let frame_count = recipe.minimum_hit_window_frames(sample_rate);
    let head_frames = sample_rate / 55;
    let mut dry = Vec::with_capacity(frame_count as usize);
    let mut shaped = Vec::with_capacity(frame_count as usize);
    for frame in 0..frame_count {
        let seconds = frame as f32 / sample_rate as f32;
        let body = 0.16 * (std::f32::consts::TAU * 90.0 * seconds).sin();
        let head_envelope = (1.0 - frame as f32 / head_frames as f32).clamp(0.0, 1.0);
        let head =
            0.055 * head_envelope * (std::f32::consts::TAU * 2_200.0 * seconds).sin();
        let source_hit = body + head;
        state.hard_attack_mix = 1.0;
        state.hard_attack_head_mix = head_envelope;
        dry.push(source_hit);
        shaped.push(w30_resample_hit_shaper_v3_sample(
            &hard, &mut state, source_hit,
        ));
    }

    let rms = |samples: &[f32]| {
        (samples
            .iter()
            .map(|sample| sample * sample)
            .sum::<f32>()
            / samples.len().max(1) as f32)
            .sqrt()
    };
    let body_start = (sample_rate / 50) as usize;
    let body_end = frame_count as usize;
    assert!(
        rms(&shaped[..head_frames as usize]) / rms(&dry[..head_frames as usize]) >= 1.2,
        "the source-owned attack head did not gain an audible articulation"
    );
    assert!(
        rms(&shaped[body_start..body_end]) / rms(&dry[body_start..body_end]) >= 1.2,
        "the source-owned 20-100 ms body did not gain an audible lift"
    );
}

#[test]
fn source_impact_shaper_v4_preserves_dry_head_without_fixed_low_boost() {
    let recipe = W30ResampleLowImpactRecipe::SourceImpactShaperV4;
    let hard = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
        source_audio: positive_realtime_resample_source(),
        lineage_capture_count: 1,
        generation_depth: 1,
        variation: W30ResampleTapVariation::HardDamage,
        variation_revision: 1,
        variation_intensity: 0.82,
        hard_policy: W30ResampleTapHardPolicy::SourceTransientChop,
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 12.0,
        hard_trigger_mask: 1,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_lengths: [64; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_bite: Default::default(),
        hard_low_impact: W30ResampleLowImpactPlan {
            recipe,
            presence_head_wet: recipe.head_wet(),
            role: W30ResampleLowImpactRole::TransientImpact,
            decision: W30ResampleLowImpactDecision::SourceHitSelected,
            ..W30ResampleLowImpactPlan::default()
        },
        hard_gesture: Default::default(),
        hard_transient_contrast: 2.0,
        music_bus_level: 0.72,
        grit_level: 0.5,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    };
    let sample_rate = 48_000_u32;
    let render_tone = |frequency_hz: f32, head_mix: f32| {
        let mut state = W30ResampleTapCallbackState::default();
        configure_w30_resample_low_impact(&hard, &mut state, sample_rate);
        let mut dry = Vec::with_capacity(2_400);
        let mut shaped = Vec::with_capacity(2_400);
        for frame in 0..2_400 {
            let source_hit = 0.4
                * (std::f32::consts::TAU * frequency_hz * frame as f32
                    / sample_rate as f32)
                    .sin();
            state.hard_attack_mix = 1.0;
            state.hard_attack_head_mix = head_mix;
            dry.push(source_hit);
            shaped.push(w30_resample_impact_shaper_v4_sample(
                &hard, &mut state, source_hit,
            ));
        }
        (dry, shaped)
    };
    let rms = |samples: &[f32]| {
        (samples
            .iter()
            .map(|sample| sample * sample)
            .sum::<f32>()
            / samples.len().max(1) as f32)
            .sqrt()
    };

    let (low_dry, low_shaped) = render_tone(90.0, 1.0);
    let low_ratio = rms(&low_shaped[480..]) / rms(&low_dry[480..]);
    assert!(
        (low_ratio - 1.0).abs() < 0.07,
        "V4 must keep linear presence-filter leakage below 0.6 dB at 90 Hz: {low_ratio}"
    );

    let (body_dry, body_shaped) = render_tone(120.0, 0.0);
    let body_ratio = rms(&body_shaped[480..]) / rms(&body_dry[480..]);
    assert!(
        body_ratio >= 1.8,
        "V4 must add a clean source-transient body: {body_ratio}"
    );

    let (presence_dry, presence_shaped) = render_tone(2_200.0, 1.0);
    let presence_delta = presence_dry[480..]
        .iter()
        .zip(&presence_shaped[480..])
        .map(|(dry, shaped)| (shaped - dry).abs())
        .sum::<f32>()
        / (presence_dry.len() - 480) as f32;
    assert!(
        presence_delta > 0.005,
        "V4 must create a material parallel presence residual: {presence_delta}"
    );

    let mut state = W30ResampleTapCallbackState::default();
    configure_w30_resample_low_impact(&hard, &mut state, sample_rate);
    state.hard_attack_mix = 1.0;
    state.hard_attack_head_mix = 1.0;
    assert_eq!(
        w30_resample_impact_shaper_v4_sample(&hard, &mut state, 0.7),
        0.7,
        "the first dry attack sample must survive intact"
    );
}

#[test]
fn source_aligned_impact_v5_changes_only_the_owned_presence_head() {
    let recipe = W30ResampleLowImpactRecipe::SourceAlignedImpactV5;
    let hard = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
        source_audio: positive_realtime_resample_source(),
        lineage_capture_count: 1,
        generation_depth: 1,
        variation: W30ResampleTapVariation::HardDamage,
        variation_revision: 1,
        variation_intensity: 0.82,
        hard_policy: W30ResampleTapHardPolicy::SourceTransientChop,
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        // V5 must ignore the historical V4 body-EQ control.
        hard_impact_body_eq_gain_db: 18.0,
        hard_trigger_mask: 1,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_lengths: [64; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_bite: Default::default(),
        hard_low_impact: W30ResampleLowImpactPlan {
            recipe,
            presence_head_wet: recipe.head_wet(),
            role: W30ResampleLowImpactRole::TransientImpact,
            decision: W30ResampleLowImpactDecision::SourceHitSelected,
            ..W30ResampleLowImpactPlan::default()
        },
        hard_gesture: Default::default(),
        hard_transient_contrast: 2.0,
        music_bus_level: 0.72,
        grit_level: 0.5,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    };
    let sample_rate = 48_000_u32;
    let render_tone = |frequency_hz: f32, head_mix: f32| {
        let mut state = W30ResampleTapCallbackState::default();
        configure_w30_resample_low_impact(&hard, &mut state, sample_rate);
        let mut dry = Vec::with_capacity(2_400);
        let mut shaped = Vec::with_capacity(2_400);
        for frame in 0..2_400 {
            let source_hit = 0.4
                * (std::f32::consts::TAU * frequency_hz * frame as f32
                    / sample_rate as f32)
                    .sin();
            state.hard_attack_mix = 1.0;
            state.hard_attack_head_mix = head_mix;
            dry.push(source_hit);
            shaped.push(w30_resample_aligned_impact_v5_sample(
                &hard, &mut state, source_hit,
            ));
        }
        (dry, shaped)
    };

    let (body_dry, body_shaped) = render_tone(120.0, 0.0);
    assert_eq!(
        body_shaped, body_dry,
        "V5 must leave the phase-coherent dry body sample-identical"
    );

    let (presence_dry, presence_shaped) = render_tone(2_200.0, 1.0);
    let presence_delta = presence_dry[480..]
        .iter()
        .zip(&presence_shaped[480..])
        .map(|(dry, shaped)| (shaped - dry).abs())
        .sum::<f32>()
        / (presence_dry.len() - 480) as f32;
    assert!(
        presence_delta > 0.005,
        "V5 must create a material phase-coherent presence residual: {presence_delta}"
    );
}

#[test]
fn exact_hit_calibration_preserves_the_owned_hit_but_lowers_between_hit_material() {
    let sample_rate = 48_000_u32;
    let recipe = W30ResampleLowImpactRecipe::SourceHitShaperV3;
    let render = RealtimeW30ResampleTapState {
        mode: W30ResampleTapMode::CaptureLineageReady,
        routing: W30ResampleTapRouting::InternalCaptureTap,
        source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
        source_audio: positive_realtime_resample_source(),
        lineage_capture_count: 1,
        generation_depth: 1,
        variation: W30ResampleTapVariation::HardDamage,
        variation_revision: 1,
        variation_intensity: 0.82,
        hard_policy: W30ResampleTapHardPolicy::SourceTransientChop,
        hard_output_gain: 0.5,
        hard_hit_window_compensation_gain: 2.2,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 1,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_lengths: [64; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_bite: Default::default(),
        hard_low_impact: W30ResampleLowImpactPlan {
            recipe,
            low_band_attack_share: 0.4,
            low_band_attack_over_body: 2.0,
            low_band_attack_over_source: 0.8,
            ..W30ResampleLowImpactPlan::default()
        },
        hard_gesture: Default::default(),
        hard_transient_contrast: 2.0,
        music_bus_level: 0.72,
        grit_level: 0.5,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    };
    let hold_frames = recipe.calibrated_hit_preservation_frames(sample_rate);
    let fade_frames = recipe.calibrated_hit_preservation_fade_frames(sample_rate);
    let total_frames = hold_frames + fade_frames;
    let mut state = W30ResampleTapCallbackState {
        beat_position: 0.25,
        hard_hit_preservation_frames_remaining: total_frames,
        hard_hit_preservation_total_frames: total_frames,
        ..Default::default()
    };

    let primary = w30_resample_calibrated_hit_preservation_sample(
        &render,
        &mut state,
        sample_rate,
        0.25,
    );
    state.hard_hit_preservation_frames_remaining =
        total_frames - recipe.minimum_hit_window_frames(sample_rate);
    let following_body = w30_resample_calibrated_hit_preservation_sample(
        &render,
        &mut state,
        sample_rate,
        0.25,
    );
    state.hard_hit_preservation_frames_remaining =
        total_frames - recipe.calibrated_late_body_start_frames(sample_rate);
    let calibrated_late_body = w30_resample_calibrated_hit_preservation_sample(
        &render,
        &mut state,
        sample_rate,
        0.25,
    );
    state.hard_hit_preservation_frames_remaining = 0;
    let between_hits = w30_resample_calibrated_hit_preservation_sample(
        &render,
        &mut state,
        sample_rate,
        0.25,
    );

    assert!((primary * render.hard_output_gain - 0.25).abs() < 1.0e-6);
    assert!(
        (following_body * render.hard_output_gain - 0.25).abs() < 1.0e-6,
        "the owned hit must retain unity before its calibrated late-body window"
    );
    assert!(
        (calibrated_late_body * render.hard_output_gain - 0.275).abs() < 1.0e-6,
        "the callback must apply the source-calibrated late-body target"
    );
    assert!((between_hits * render.hard_output_gain - 0.125).abs() < 1.0e-6);

    let preroll_frames = recipe.calibrated_hit_preroll_frames(sample_rate);
    let preroll_fade_frames = recipe.calibrated_hit_preroll_fade_frames(sample_rate);
    let frames_per_step = sample_rate / 4;
    let beat_position_at_fade_start =
        (1.0 - f64::from(preroll_frames + preroll_fade_frames) / f64::from(frames_per_step))
            / 2.0;
    let mut preroll_render = render;
    preroll_render.hard_trigger_mask = 0b0000_0010;
    let mut preroll_state = W30ResampleTapCallbackState {
        beat_position: beat_position_at_fade_start,
        ..Default::default()
    };
    let fade_start = w30_resample_calibrated_hit_preservation_sample(
        &preroll_render,
        &mut preroll_state,
        sample_rate,
        0.25,
    );
    preroll_state.beat_position =
        (1.0 - f64::from(preroll_frames) / f64::from(frames_per_step)) / 2.0;
    let full_preroll = w30_resample_calibrated_hit_preservation_sample(
        &preroll_render,
        &mut preroll_state,
        sample_rate,
        0.25,
    );

    assert!((fade_start - 0.25).abs() < 1.0e-6);
    assert!((full_preroll * render.hard_output_gain - 0.25).abs() < 1.0e-6);
}

#[test]
fn transient_chop_starts_the_selected_source_onset_on_the_eighth_note_grid() {
    let sample_rate = 44_100_u32;
    let tempo_bpm = 120.0_f32;
    let start_beats = 0.49_f64;
    let target_beats = 0.5_f64;
    let selected_cursor = 2_048_u16;
    let mut source_audio = RealtimeW30ResampleSourceWindow {
        source_revision: 2,
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
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 0b0000_0010,
        hard_slice_cursors: [0, selected_cursor, 0, 0, 0, 0, 0, 0],
        hard_attack_lengths: [256; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_gesture: Default::default(),
        hard_transient_contrast: 1.8,
        music_bus_level: 0.8,
        hard_attack_bite: Default::default(),
        hard_low_impact: Default::default(),
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
    let max_click_safe_preroll_frames = usize::try_from(sample_rate / 2_000).unwrap() + 2;
    assert!(
        first_audible_frame.abs_diff(expected_grid_frame) <= max_click_safe_preroll_frames,
        "source onset exceeded the bounded click-safe preroll: expected near frame {expected_grid_frame}, got {first_audible_frame}"
    );
}

#[test]
fn base_resample_wraps_a_full_bar_artifact_on_the_transport_bar_boundary() {
    let output_sample_rate = 48_000_u32;
    let tempo_bpm = 130.0_f32;
    let expected_bar_frames =
        (4.0 * 60.0 / f64::from(tempo_bpm) * f64::from(output_sample_rate)).round() as usize;
    let mut source_audio = RealtimeW30ResampleSourceWindow {
        source_revision: 3,
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
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 0b1101_0111,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_lengths: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_gesture: Default::default(),
        hard_transient_contrast: 5.0,
        music_bus_level: 0.8,
        hard_attack_bite: Default::default(),
        hard_low_impact: Default::default(),
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
            hard_output_gain: 1.0,
            hard_hit_window_compensation_gain: 1.0,
            hard_impact_body_eq_gain_db: 0.0,
            hard_trigger_mask: 0,
            hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            hard_attack_lengths: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            hard_gesture: Default::default(),
            hard_transient_contrast: 0.0,
            music_bus_level: 0.58,
            hard_attack_bite: Default::default(),
            hard_low_impact: Default::default(),
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
        hard_output_gain: 1.0,
        hard_hit_window_compensation_gain: 1.0,
        hard_impact_body_eq_gain_db: 0.0,
        hard_trigger_mask: 0,
        hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_attack_lengths: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
        hard_gesture: Default::default(),
        hard_transient_contrast: 0.0,
        music_bus_level: 0.58,
        hard_attack_bite: Default::default(),
        hard_low_impact: Default::default(),
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
            hard_output_gain: 1.0,
            hard_hit_window_compensation_gain: 1.0,
            hard_impact_body_eq_gain_db: 0.0,
            hard_trigger_mask: 0,
            hard_slice_cursors: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            hard_attack_lengths: [0; W30_RESAMPLE_HARD_SLICE_COUNT],
            hard_gesture: Default::default(),
            hard_transient_contrast: 0.0,
            music_bus_level: 0.0,
            hard_attack_bite: Default::default(),
            hard_low_impact: Default::default(),
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
