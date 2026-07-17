use riotbox_core::action::SourceMonitorMode;

#[test]
fn source_monitor_source_mode_replaces_generated_output_with_source_pcm() {
    let source = SourceAudioCache::from_interleaved_samples(
        "source.wav",
        44_100,
        2,
        vec![0.25, -0.25, 0.5, -0.5, 0.75, -0.75, 1.0, -1.0],
    )
    .expect("source cache");
    let generated = vec![0.05; 8];
    let render = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
        ..SourceMonitorRenderState::default()
    };

    let output = render_source_monitor_mix_offline(&generated, 44_100, 2, &render);

    assert_eq!(output[0], 0.25 * 0.88);
    assert_eq!(output[1], -0.25 * 0.88);
    assert!(signal_delta_metrics(&output, &generated).rms > 0.1);
}

#[test]
fn source_monitor_blend_keeps_generated_and_source_energy() {
    let source = SourceAudioCache::from_interleaved_samples(
        "source.wav",
        44_100,
        1,
        vec![0.5, 0.5, 0.5, 0.5],
    )
    .expect("source cache");
    let generated = vec![0.25; 8];
    let render = SourceMonitorRenderState {
        mode: SourceMonitorMode::Blend,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
        ..SourceMonitorRenderState::default()
    };

    let output = render_source_monitor_mix_offline(&generated, 44_100, 2, &render);

    assert!(output.iter().all(|sample| *sample > 0.25));
    assert!(signal_metrics(&output).rms > signal_metrics(&generated).rms);
}

#[test]
fn source_monitor_blend_leaves_hot_sum_for_the_master_limiter() {
    let source = SourceAudioCache::from_interleaved_samples(
        "hot-source.wav",
        48_000,
        1,
        vec![1.0; 16],
    )
    .expect("source cache");
    let generated = vec![1.0; 16];
    let render = SourceMonitorRenderState {
        mode: SourceMonitorMode::Blend,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
        ..SourceMonitorRenderState::default()
    };

    let pre_limiter = render_source_monitor_mix_offline(&generated, 48_000, 1, &render);

    assert!(pre_limiter.iter().all(|sample| (*sample - 1.24).abs() < 1.0e-6));
    assert!(signal_metrics(&pre_limiter).peak_abs > 1.0);
}

#[test]
fn source_monitor_mode_change_uses_callback_persistent_gain_ramp() {
    let source = SourceAudioCache::from_interleaved_samples(
        "mode-transition.wav",
        1_000,
        1,
        vec![1.0; 64],
    )
    .expect("source cache");
    let source_render = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm: 60.0,
        position_beats: 0.0,
        ..SourceMonitorRenderState::default()
    };
    let shared = SharedSourceMonitorRenderState::new(&source_render);
    let mut callback_state = SourceMonitorCallbackState::default();
    let mut source_block = vec![0.0; 4];
    let mut render = shared.snapshot();
    render.is_transport_running = true;
    render.tempo_bpm = 60.0;
    apply_source_monitor_policy_with_state(
        &mut source_block,
        1_000,
        1,
        &render,
        &mut callback_state,
    );

    shared.update(&SourceMonitorRenderState {
        mode: SourceMonitorMode::Riotbox,
        ..source_render
    });
    let mut riotbox_block = vec![1.0; 8];
    let mut render = shared.snapshot();
    render.is_transport_running = true;
    render.tempo_bpm = 60.0;
    render.position_beats = 0.004;
    apply_source_monitor_policy_with_state(
        &mut riotbox_block,
        1_000,
        1,
        &render,
        &mut callback_state,
    );

    assert!((source_block[3] - 0.88).abs() < 1.0e-6);
    assert!((riotbox_block[0] - 0.904).abs() < 1.0e-6);
    assert!((riotbox_block[4] - 1.0).abs() < 1.0e-6);
    assert!(
        (riotbox_block[0] - source_block[3]).abs() < 0.03,
        "mode switch retained a hard callback-edge jump: source={source_block:?} riotbox={riotbox_block:?}"
    );
}

#[test]
fn source_monitor_anchor_jump_crossfades_from_previous_source_cursor() {
    let mut samples = vec![1.0; 2_000];
    samples[1_000..].fill(-1.0);
    let source = SourceAudioCache::from_interleaved_samples(
        "anchor-transition.wav",
        1_000,
        1,
        samples,
    )
    .expect("source cache");
    let initial = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm: 60.0,
        position_beats: 0.0,
        ..SourceMonitorRenderState::default()
    };
    let shared = SharedSourceMonitorRenderState::new(&initial);
    let mut callback_state = SourceMonitorCallbackState::default();
    let mut before = vec![0.0; 100];
    let mut render = shared.snapshot();
    render.is_transport_running = true;
    render.tempo_bpm = 60.0;
    apply_source_monitor_policy_with_state(
        &mut before,
        1_000,
        1,
        &render,
        &mut callback_state,
    );

    let mut after = vec![0.0; 8];
    let mut render = shared.snapshot();
    render.is_transport_running = true;
    render.tempo_bpm = 60.0;
    render.position_beats = 1.0;
    apply_source_monitor_policy_with_state(
        &mut after,
        1_000,
        1,
        &render,
        &mut callback_state,
    );

    assert!((before[99] - 0.88).abs() < 1.0e-6);
    assert!((after[0] - 0.88).abs() < 1.0e-6);
    assert!(
        after[4] < -0.87,
        "anchor crossfade did not reach the new cursor: {after:?}"
    );
    let mut transition = vec![before[99]];
    transition.extend_from_slice(&after);
    let max_adjacent_delta = transition
        .windows(2)
        .map(|window| (window[1] - window[0]).abs())
        .fold(0.0_f32, f32::max);
    assert!(
        max_adjacent_delta <= 0.45,
        "anchor crossfade retained a hard edge: max_delta={max_adjacent_delta} after={after:?}"
    );
}

#[test]
fn source_monitor_transport_stop_fades_previous_cursor_without_file_start_blip() {
    let mut samples = vec![1.0; 2_000];
    samples[1_000..].fill(-1.0);
    let source = SourceAudioCache::from_interleaved_samples(
        "transport-stop.wav",
        1_000,
        1,
        samples,
    )
    .expect("source cache");
    let running = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm: 60.0,
        position_beats: 1.0,
        ..SourceMonitorRenderState::default()
    };
    let shared = SharedSourceMonitorRenderState::new(&running);
    let mut callback_state = SourceMonitorCallbackState::default();
    let mut before = vec![0.0; 100];
    let mut render = shared.snapshot();
    render.is_transport_running = true;
    render.tempo_bpm = 60.0;
    render.position_beats = 1.0;
    apply_source_monitor_policy_with_state(
        &mut before,
        1_000,
        1,
        &render,
        &mut callback_state,
    );

    let mut stopped = vec![0.0; 8];
    let mut render = shared.snapshot();
    render.is_transport_running = false;
    render.tempo_bpm = 60.0;
    render.position_beats = 1.1;
    apply_source_monitor_policy_with_state(
        &mut stopped,
        1_000,
        1,
        &render,
        &mut callback_state,
    );

    assert!(before[99] < -0.87);
    assert!(
        stopped[..4].iter().all(|sample| *sample < 0.0),
        "stop leaked file-start polarity: {stopped:?}"
    );
    assert!(
        stopped[4..]
            .iter()
            .all(|sample| sample.abs() <= f32::EPSILON)
    );
    assert!(
        (stopped[0] - before[99]).abs() <= 0.18,
        "transport stop retained a hard edge: before={} stopped={stopped:?}",
        before[99]
    );
}

#[test]
fn source_monitor_render_states_share_prepared_pcm_backing() {
    let source = SourceAudioCache::from_interleaved_samples(
        "long-source.wav",
        48_000,
        2,
        vec![0.25; 48_000 * 2],
    )
    .expect("source cache");

    let first = SourceMonitorAudioSource::from_cache(&source);
    let second = SourceMonitorAudioSource::from_cache(&source);

    assert_eq!(
        first.interleaved_samples().as_ptr(),
        source.interleaved_samples().as_ptr()
    );
    assert_eq!(
        second.interleaved_samples().as_ptr(),
        source.interleaved_samples().as_ptr()
    );
}

#[test]
fn source_monitor_resamples_44k1_source_for_48k_output_without_silence() {
    let source = SourceAudioCache::from_interleaved_samples(
        "source-44k1.wav",
        44_100,
        1,
        vec![
            0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 0.9, 0.8, 0.7,
            0.6, 0.5,
        ],
    )
    .expect("source cache");
    let generated = vec![0.1; 16];
    let render = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
        ..SourceMonitorRenderState::default()
    };

    assert_eq!(
        render.route_for_output(48_000, 2),
        SourceMonitorAudioRoute::SourceOnly
    );
    let output = render_source_monitor_mix_offline(&generated, 48_000, 2, &render);

    for (frame, expected_source) in [(0, 0.0), (1, 0.091_875), (2, 0.183_75)] {
        let expected = expected_source * 0.88;
        assert!((output[frame * 2] - expected).abs() < 1.0e-6);
        assert!((output[frame * 2 + 1] - expected).abs() < 1.0e-6);
    }
    assert!(signal_metrics(&output).rms > 0.15);
    assert!(signal_delta_metrics(&output, &generated).rms > 0.05);
}

#[test]
fn source_monitor_stops_at_source_end_without_wrapping_to_the_start() {
    let source = SourceAudioCache::from_interleaved_samples(
        "short-source.wav",
        8,
        1,
        vec![0.25, 0.5, 0.75, 1.0],
    )
    .expect("source cache");
    let generated = vec![0.4; 4];
    let render = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.5,
        ..SourceMonitorRenderState::default()
    };

    let output = render_source_monitor_mix_offline(&generated, 8, 1, &render);

    assert_eq!(output, vec![0.75 * 0.88, 1.0 * 0.88, 0.0, 0.0]);
}

#[test]
fn source_monitor_fades_the_source_tail_before_eof() {
    let source = SourceAudioCache::from_interleaved_samples(
        "tail.wav",
        1_000,
        1,
        vec![1.0; 100],
    )
    .expect("source cache");
    let generated = vec![0.0; 10];
    let render = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm: 60.0,
        position_beats: 0.095,
        ..SourceMonitorRenderState::default()
    };

    let output = render_source_monitor_mix_offline(&generated, 1_000, 1, &render);

    assert!((output[0] - 0.88).abs() < 1.0e-6);
    assert!((output[1] - 0.704).abs() < 1.0e-6);
    assert!((output[4] - 0.176).abs() < 1.0e-6);
    assert!(output[5..].iter().all(|sample| sample.abs() <= f32::EPSILON));
}

#[test]
fn source_monitor_blend_keeps_only_riotbox_after_crossing_source_end() {
    let source = SourceAudioCache::from_interleaved_samples(
        "short-source.wav",
        8,
        1,
        vec![0.25, 0.5, 0.75, 1.0],
    )
    .expect("source cache");
    let generated = vec![0.4; 4];
    let render = SourceMonitorRenderState {
        mode: SourceMonitorMode::Blend,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.5,
        ..SourceMonitorRenderState::default()
    };

    let output = render_source_monitor_mix_offline(&generated, 8, 1, &render);

    assert_eq!(
        output,
        vec![
            (0.4 * 0.62) + (0.75 * 0.62),
            (0.4 * 0.62) + (1.0 * 0.62),
            0.4 * 0.62,
            0.4 * 0.62,
        ]
    );
}

#[test]
fn source_monitor_seek_beyond_source_end_stays_silent() {
    let source = SourceAudioCache::from_interleaved_samples(
        "short-source.wav",
        8,
        1,
        vec![0.25, 0.5, 0.75, 1.0],
    )
    .expect("source cache");
    let generated = vec![0.4; 4];
    let render = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 2.0,
        ..SourceMonitorRenderState::default()
    };

    let output = render_source_monitor_mix_offline(&generated, 8, 1, &render);

    assert!(output.iter().all(|sample| sample.abs() <= f32::EPSILON));
}

#[test]
fn unavailable_blend_preserves_the_riotbox_component_as_degraded_output() {
    let generated = vec![0.25, -0.25, 0.5, -0.5];
    let render = SourceMonitorRenderState {
        mode: SourceMonitorMode::Blend,
        source: None,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
        ..SourceMonitorRenderState::default()
    };

    assert_eq!(
        render.route_for_output(48_000, 2),
        SourceMonitorAudioRoute::SourceUnavailable
    );
    let output = render_source_monitor_mix_offline(&generated, 48_000, 2, &render);

    assert_eq!(output, vec![0.155, -0.155, 0.31, -0.31]);
    assert!(signal_metrics(&output).rms > 0.1);
}

#[test]
fn source_monitor_source_mode_mutes_when_source_cache_is_absent() {
    let generated = vec![0.25, -0.25, 0.5, -0.5];
    let render = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: None,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
        ..SourceMonitorRenderState::default()
    };

    let output = render_source_monitor_mix_offline(&generated, 44_100, 2, &render);

    assert!(output.iter().all(|sample| sample.abs() <= f32::EPSILON));
}

#[test]
fn source_monitor_seeked_running_transport_changes_audible_source_excerpt() {
    let sample_rate = 480;
    let channel_count = 2;
    let tempo_bpm = 120.0;
    let frames_per_beat = 240;
    let frames_per_bar = frames_per_beat * 4;
    let source = SourceAudioCache::from_interleaved_samples(
        "source.wav",
        sample_rate,
        channel_count,
        source_with_bar_markers(frames_per_bar),
    )
    .expect("source cache");
    let generated = vec![0.0; 128];
    let before_seek = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm,
        position_beats: 0.0,
        ..SourceMonitorRenderState::default()
    };
    let after_seek = SourceMonitorRenderState {
        position_beats: 16.0,
        ..before_seek.clone()
    };

    let before_output =
        render_source_monitor_mix_offline(&generated, sample_rate, channel_count, &before_seek);
    let after_output =
        render_source_monitor_mix_offline(&generated, sample_rate, channel_count, &after_seek);
    let before_metrics = signal_metrics(&before_output);
    let after_metrics = signal_metrics(&after_output);
    let delta_metrics = signal_delta_metrics(&before_output, &after_output);

    assert!(before_seek.is_transport_running);
    assert!(after_seek.is_transport_running);
    assert!(before_metrics.rms > 0.1);
    assert!(after_metrics.rms > 0.1);
    assert!(delta_metrics.rms > 0.3);
    assert_eq!(before_output[0], 0.18 * 0.88);
    assert_eq!(after_output[0], -0.62 * 0.88);
}

#[test]
fn source_monitor_scene_anchor_repositions_source_excerpt_from_commit_boundary() {
    let sample_rate = 480;
    let channel_count = 2;
    let tempo_bpm = 120.0;
    let frames_per_beat = 240;
    let frames_per_bar = frames_per_beat * 4;
    let source = SourceAudioCache::from_interleaved_samples(
        "source.wav",
        sample_rate,
        channel_count,
        source_with_bar_markers(frames_per_bar),
    )
    .expect("source cache");
    let generated = vec![0.0; 128];
    let transport_only = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm,
        position_beats: 16.0,
        source_anchor_seconds: None,
        source_anchor_position_beats: 0.0,
    };
    let scene_anchored = SourceMonitorRenderState {
        source_anchor_seconds: Some(0.0),
        source_anchor_position_beats: 16.0,
        ..transport_only.clone()
    };

    let transport_output =
        render_source_monitor_mix_offline(&generated, sample_rate, channel_count, &transport_only);
    let anchored_output =
        render_source_monitor_mix_offline(&generated, sample_rate, channel_count, &scene_anchored);
    let delta_metrics = signal_delta_metrics(&transport_output, &anchored_output);

    assert_eq!(transport_output[0], -0.62 * 0.88);
    assert_eq!(anchored_output[0], 0.18 * 0.88);
    assert!(delta_metrics.rms > 0.4);
}

#[test]
fn source_monitor_shared_state_updates_scene_anchor_without_replacing_source() {
    let source = SourceAudioCache::from_interleaved_samples(
        "source.wav",
        480,
        2,
        source_with_bar_markers(960),
    )
    .expect("source cache");
    let render = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 16.0,
        source_anchor_seconds: Some(4.0),
        source_anchor_position_beats: 16.0,
    };
    let shared = SharedSourceMonitorRenderState::new(&render);
    let render_source_ptr = render
        .source
        .as_ref()
        .expect("render source")
        .interleaved_samples()
        .as_ptr();
    let snapshot = shared.snapshot();
    let snapshot_source = snapshot.source.expect("snapshot source");

    assert_eq!(snapshot_source.interleaved_samples().as_ptr(), render_source_ptr);
    assert_eq!(snapshot.source_anchor_seconds, Some(4.0));
    assert_eq!(snapshot.source_anchor_position_beats, 16.0);

    shared.update(&SourceMonitorRenderState {
        source: None,
        source_anchor_seconds: None,
        source_anchor_position_beats: 0.0,
        ..render
    });
    let cleared_anchor = shared.snapshot();

    assert!(cleared_anchor.source.is_some());
    assert_eq!(cleared_anchor.source_anchor_seconds, None);
    assert_eq!(cleared_anchor.source_anchor_position_beats, 0.0);
}

fn source_with_bar_markers(frames_per_bar: usize) -> Vec<f32> {
    let bar_levels = [0.18, 0.32, -0.24, 0.46, -0.62];
    let mut samples = Vec::with_capacity(frames_per_bar * bar_levels.len() * 2);
    for level in bar_levels {
        for _ in 0..frames_per_bar {
            samples.push(level);
            samples.push(-level);
        }
    }
    samples
}
