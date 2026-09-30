use crate::runtime::source_monitor::{
    SourceMonitorAudioRoute, SourceMonitorAudioSource, SourceMonitorRenderState,
    render_source_monitor_mix_offline,
};
use crate::runtime::{signal_delta_metrics, signal_metrics};
use crate::source_audio::SourceAudioCache;
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
    let source =
        SourceAudioCache::from_interleaved_samples("hot-source.wav", 48_000, 1, vec![1.0; 16])
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

    assert!(
        pre_limiter
            .iter()
            .all(|sample| (*sample - 1.24).abs() < 1.0e-6)
    );
    assert!(signal_metrics(&pre_limiter).peak_abs > 1.0);
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
