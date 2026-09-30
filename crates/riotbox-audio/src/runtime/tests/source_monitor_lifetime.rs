use crate::runtime::source_monitor::{
    SourceMonitorAudioRoute, SourceMonitorAudioSource, SourceMonitorRenderState,
    render_source_monitor_mix_offline,
};
use crate::runtime::{signal_delta_metrics, signal_metrics};
use crate::source_audio::SourceAudioCache;
use riotbox_core::action::SourceMonitorMode;

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
            0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 0.9, 0.8, 0.7, 0.6, 0.5,
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
    let source = SourceAudioCache::from_interleaved_samples("tail.wav", 1_000, 1, vec![1.0; 100])
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
    assert!(
        output[5..]
            .iter()
            .all(|sample| sample.abs() <= f32::EPSILON)
    );
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
