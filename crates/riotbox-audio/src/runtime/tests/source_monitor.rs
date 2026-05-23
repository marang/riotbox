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
    };

    let output = render_source_monitor_mix_offline(&generated, 44_100, 2, &render);

    assert!(output.iter().all(|sample| *sample > 0.25));
    assert!(signal_metrics(&output).rms > signal_metrics(&generated).rms);
}

#[test]
fn source_monitor_falls_back_to_riotbox_when_source_cache_is_absent() {
    let generated = vec![0.25, -0.25, 0.5, -0.5];
    let render = SourceMonitorRenderState {
        mode: SourceMonitorMode::Source,
        source: None,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
    };

    let output = render_source_monitor_mix_offline(&generated, 44_100, 2, &render);

    assert_eq!(output, generated);
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
