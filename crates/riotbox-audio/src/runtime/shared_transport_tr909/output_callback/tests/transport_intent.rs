//! Position intent through the exact processor installed in the CPAL callback.
//! Generated PCM and callback-sized buffers only; never opens a device or file.

use super::{prepare_output_callback, shared};
use crate::{
    runtime::{
        AudioRuntimeLifecycle, AudioRuntimeShell, AudioTransportPositionRequest,
        AudioTransportPositionUpdate, SourceMonitorAudioSource, SourceMonitorRenderState,
        shared_transport_tr909::{AudioRuntimeSharedState, AudioRuntimeShellTestParts},
    },
    source_audio::SourceAudioCache,
    test_heap::{HeapCounts, measure},
};
use riotbox_core::action::SourceMonitorMode;
use std::time::Instant;

fn reposition(position: f64) -> AudioTransportPositionUpdate {
    AudioTransportPositionUpdate::Reposition(AudioTransportPositionRequest::from_beats(position))
}

fn shell(state: &AudioRuntimeSharedState) -> AudioRuntimeShell {
    AudioRuntimeShell::from_test_parts(AudioRuntimeShellTestParts {
        lifecycle: AudioRuntimeLifecycle::Running,
        output: None,
        telemetry: state.telemetry.clone(),
        transport: state.transport.clone(),
        tr909_render: state.tr909_render.clone(),
        mc202_render: state.mc202_render.clone(),
        w30_preview: state.w30_preview.clone(),
        w30_resample_tap: state.w30_resample_tap.clone(),
        source_monitor: state.source_monitor.clone(),
    })
}

fn config() -> cpal::StreamConfig {
    cpal::StreamConfig {
        channels: 2,
        sample_rate: 48_000,
        buffer_size: cpal::BufferSize::Fixed(480),
    }
}

fn check_position(shell: &AudioRuntimeShell, expected: f64) {
    let actual = shell.timing_snapshot().unwrap().position_beats;
    assert!((actual - expected).abs() < 1e-11, "{actual} != {expected}");
}

#[test]
fn delayed_observed_progress_does_not_rewind_the_actual_callback() {
    let config = cpal::StreamConfig {
        channels: 2,
        sample_rate: 48_000,
        buffer_size: cpal::BufferSize::Fixed(480),
    };
    let state = shared();
    let source = SourceAudioCache::from_interleaved_samples(
        "generated-transport-ramp-no-file.wav",
        48_000,
        2,
        (0..48_000)
            .flat_map(|frame| [frame as f32 / 48_000.0 * 0.1; 2])
            .collect(),
    )
    .unwrap();
    state
        .source_monitor
        .replace_source_and_controls(&SourceMonitorRenderState {
            mode: SourceMonitorMode::Source,
            source: Some(SourceMonitorAudioSource::from_cache(&source)),
            ..SourceMonitorRenderState::default()
        });
    state
        .transport
        .update(
            true,
            120.0,
            crate::runtime::AudioTransportPositionUpdate::Reposition(
                crate::runtime::AudioTransportPositionRequest::from_beats(0.0),
            ),
        )
        .unwrap();
    let mut callback =
        prepare_output_callback::<f32>(&config, state.clone(), Instant::now()).unwrap();
    let mut output = [0.0; 960];
    for _ in 0..10 {
        let (_, heap) = measure(|| callback(&mut output));
        assert_eq!(heap, HeapCounts::default());
    }
    let observed = state.telemetry.timing_snapshot().unwrap();
    assert!((observed.position_beats - 0.2).abs() < 1e-12);
    // Control work is delayed by ten otherwise healthy 10-ms callbacks.
    for _ in 0..10 {
        callback(&mut output);
    }
    assert!((state.telemetry.timing_snapshot().unwrap().position_beats - 0.4).abs() < 1e-12);
    state
        .transport
        .update(
            true,
            120.0,
            crate::runtime::AudioTransportPositionUpdate::Preserve,
        )
        .unwrap();
    let (_, heap) = measure(|| callback(&mut output));
    assert_eq!(heap, HeapCounts::default());
    let actual = state.telemetry.timing_snapshot().unwrap().position_beats;
    assert!(
        (actual - 0.42).abs() < 1e-12,
        "observed progress rewound callback to {actual}"
    );
    // The exact callback must continue through the generated ramp, not replay
    // the earlier .2-beat excerpt. Limiter is inactive for this quiet signal.
    assert!(output[958] > 0.018, "source playback unexpectedly rewound");
}

#[test]
fn public_observations_wait_for_each_exact_seek_even_at_the_same_numeric_target() {
    let state = shared();
    let shell = shell(&state);
    shell
        .update_transport_state(true, 120.0, reposition(12.5))
        .unwrap();
    let mut callback =
        prepare_output_callback::<f32>(&config(), state.clone(), Instant::now()).unwrap();
    let mut output = [0.0; 960];
    assert!(shell.timing_snapshot().is_none());
    callback(&mut output);
    check_position(&shell, 12.52);
    for target in [12.525, 12.525, 20.0, 2.0, 0.0, 0.0] {
        shell
            .update_transport_state(true, 120.0, reposition(target))
            .unwrap();
        // The old observation can have exactly the requested numeric position;
        // identity, not value/tolerance, is the admission criterion.
        assert!(shell.timing_snapshot().is_none());
        shell
            .update_transport_state(true, 120.0, AudioTransportPositionUpdate::Preserve)
            .unwrap();
        assert!(shell.timing_snapshot().is_none());
        let (_, heap) = measure(|| callback(&mut output));
        assert_eq!(heap, HeapCounts::default());
        check_position(&shell, target + 0.02);
    }
    // Superseding a request must not make an earlier callback acknowledgment valid.
    shell
        .update_transport_state(false, 120.0, reposition(4.0))
        .unwrap();
    callback(&mut output);
    check_position(&shell, 4.0);
    shell
        .update_transport_state(false, 120.0, reposition(4.0))
        .unwrap();
    assert!(shell.timing_snapshot().is_none());
    shell
        .update_transport_state(false, 120.0, reposition(8.0))
        .unwrap();
    assert!(shell.timing_snapshot().is_none());
    callback(&mut output);
    check_position(&shell, 8.0);
}

#[test]
fn pause_resume_and_tempo_refresh_preserve_the_actual_not_the_last_observed_cursor() {
    let state = shared();
    let shell = shell(&state);
    shell
        .update_transport_state(false, 120.0, reposition(12.5))
        .unwrap();
    let mut callback = prepare_output_callback::<f32>(&config(), state, Instant::now()).unwrap();
    let mut output = [0.0; 960];
    callback(&mut output);
    check_position(&shell, 12.5);
    shell
        .update_transport_state(true, 120.0, AudioTransportPositionUpdate::Preserve)
        .unwrap();
    for _ in 0..20 {
        callback(&mut output);
    }
    check_position(&shell, 12.9);
    shell
        .update_transport_state(false, 120.0, AudioTransportPositionUpdate::Preserve)
        .unwrap();
    for _ in 0..3 {
        callback(&mut output);
    }
    check_position(&shell, 12.9);
    assert!(!shell.timing_snapshot().unwrap().is_transport_running);
    shell
        .update_transport_state(true, 60.0, AudioTransportPositionUpdate::Preserve)
        .unwrap();
    callback(&mut output);
    check_position(&shell, 12.91);
    shell
        .update_transport_state(false, 60.0, reposition(0.0))
        .unwrap();
    assert!(shell.timing_snapshot().is_none());
    callback(&mut output);
    check_position(&shell, 0.0);
    assert!(!shell.timing_snapshot().unwrap().is_transport_running);
}

#[test]
fn restored_nonzero_constructor_cursor_is_observed_only_after_the_first_callback() {
    use crate::runtime::shared_transport_tr909::SharedTransportTimingState;
    use std::sync::Arc;
    for running in [false, true] {
        let mut state = shared();
        state.transport = Arc::new(SharedTransportTimingState::new(running, 120.0, 12.5));
        let shell = shell(&state);
        shell
            .update_transport_state(running, 120.0, AudioTransportPositionUpdate::Preserve)
            .unwrap();
        assert!(shell.timing_snapshot().is_none());
        let mut callback =
            prepare_output_callback::<f32>(&config(), state, Instant::now()).unwrap();
        let mut output = [0.0; 960];
        let (_, heap) = measure(|| callback(&mut output));
        assert_eq!(heap, HeapCounts::default());
        check_position(&shell, if running { 12.52 } else { 12.5 });
    }
}

#[test]
fn a_sub_step_repeated_seek_retriggers_the_actual_fill_not_only_telemetry() {
    use crate::tr909::{Tr909RenderMode, Tr909RenderRouting, Tr909RenderState};
    let state = shared();
    let shell = shell(&state);
    state.tr909_render.update(&Tr909RenderState {
        mode: Tr909RenderMode::Fill,
        routing: Tr909RenderRouting::DrumBusTakeover,
        drum_bus_level: 0.8,
        ..Default::default()
    });
    state
        .source_monitor
        .update_controls(&SourceMonitorRenderState::control_only(
            SourceMonitorMode::Riotbox,
        ));
    shell
        .update_transport_state(true, 120.0, reposition(0.0))
        .unwrap();
    let mut callback = prepare_output_callback::<f32>(&config(), state, Instant::now()).unwrap();
    let mut first = [0.0; 960];
    callback(&mut first);
    assert!(first.iter().any(|sample| sample.abs() > 0.001));
    for _ in 0..3 {
        // .02 beats is below both the old transport tolerance and one fill step.
        shell
            .update_transport_state(true, 120.0, reposition(0.0))
            .unwrap();
        let mut repeated = [0.0; 960];
        let (_, heap) = measure(|| callback(&mut repeated));
        assert_eq!(heap, HeapCounts::default());
        assert_eq!(
            repeated, first,
            "explicit seek was ignored by the drum renderer"
        );
        check_position(&shell, 0.02);
    }
}
