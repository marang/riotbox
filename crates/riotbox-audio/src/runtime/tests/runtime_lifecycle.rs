use crate::mc202::Mc202RenderState;
use crate::runtime::shared_mc202::SharedMc202RenderState;
use crate::runtime::shared_transport_tr909::{
    AudioRuntimeShellTestParts, SharedTr909RenderState, SharedTransportTimingState,
};
use crate::runtime::shared_w30_resample_callback::{
    CallbackTimingSnapshot, SharedW30ResampleTapState,
};
use crate::runtime::source_monitor::{SharedSourceMonitorRenderState, SourceMonitorRenderState};
use crate::runtime::telemetry::RuntimeTelemetry;
use crate::runtime::tr909_tail_telemetry::mode_to_u32;
use crate::runtime::w30_preview_snapshot::SharedW30PreviewRenderState;
use crate::runtime::{
    AudioOutputInfo, AudioRuntimeLifecycle, AudioRuntimeShell, begin_coherent_snapshot_update,
    coherent_snapshot_or, finish_coherent_snapshot_update,
};
use crate::tr909::{Tr909RenderMode, Tr909RenderRouting, Tr909RenderState};
use crate::w30::{W30PreviewRenderState, W30ResampleTapState};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

fn sample_output() -> AudioOutputInfo {
    AudioOutputInfo {
        host_name: "Alsa".into(),
        device_name: "default".into(),
        sample_format: "F32".into(),
        sample_rate: 44_100,
        channel_count: 2,
        buffer_size: "Default".into(),
        supported_output_config_count: Some(12),
    }
}

fn sample_timing(position_beats: f64) -> CallbackTimingSnapshot {
    CallbackTimingSnapshot {
        is_transport_running: true,
        tempo_bpm: 128.0,
        render_position_beats: position_beats,
        completed_position_beats: position_beats,
    }
}

#[test]
fn telemetry_tracks_callback_count_and_max_gap() {
    let telemetry = RuntimeTelemetry::new();
    telemetry.record_callback_at(100, &sample_timing(16.0));
    telemetry.record_callback_at(350, &sample_timing(16.5));
    telemetry.record_callback_at(775, &sample_timing(17.0));

    let snapshot = telemetry.snapshot();

    assert_eq!(snapshot.callback_count, 3);
    assert_eq!(snapshot.max_callback_gap_micros, Some(425));
    let timing = telemetry.timing_snapshot();
    assert!(timing.is_transport_running);
    assert_eq!(timing.position_beats, 17.0);
}

#[test]
fn coherent_snapshot_or_uses_previous_snapshot_during_active_update() {
    let revision = AtomicU64::new(1);

    let snapshot = coherent_snapshot_or(&revision, &17_u32, || 23_u32);

    assert_eq!(snapshot, 17);
}

#[test]
fn tr909_snapshot_or_previous_holds_last_complete_state_during_partial_update() {
    let initial = Tr909RenderState {
        mode: Tr909RenderMode::SourceSupport,
        routing: Tr909RenderRouting::DrumBusSupport,
        drum_bus_level: 0.82,
        ..Tr909RenderState::default()
    };
    let shared = SharedTr909RenderState::new(&initial);
    let previous = shared.snapshot();

    begin_coherent_snapshot_update(&shared.revision);
    shared
        .mode
        .store(mode_to_u32(Tr909RenderMode::Takeover), Ordering::Relaxed);

    let snapshot = shared.snapshot_or_previous(&previous);

    assert_eq!(snapshot, previous);
    finish_coherent_snapshot_update(&shared.revision);
}

#[test]
fn telemetry_tracks_callback_scratch_overflow_without_stream_error() {
    let telemetry = RuntimeTelemetry::new();
    telemetry.record_callback_scratch_overflow_at(100, &sample_timing(16.0));

    let snapshot = telemetry.snapshot();

    assert_eq!(snapshot.callback_count, 1);
    assert_eq!(snapshot.callback_scratch_overflow_count, 1);
    assert_eq!(snapshot.stream_error_count, 0);
    assert!(snapshot.last_stream_error.is_none());
}

#[test]
fn callback_scratch_uses_configured_fixed_size_and_bounded_default() {
    let fixed = cpal::StreamConfig {
        channels: 2,
        sample_rate: 44_100,
        buffer_size: cpal::BufferSize::Fixed(512),
    };
    let default = cpal::StreamConfig {
        channels: 2,
        sample_rate: 44_100,
        buffer_size: cpal::BufferSize::Default,
    };

    assert_eq!(
        crate::runtime::shared_transport_tr909::callback_scratch_sample_count(&fixed, 2),
        1024
    );
    assert_eq!(
        crate::runtime::shared_transport_tr909::callback_scratch_sample_count(&default, 2),
        8192
    );
}

#[test]
fn health_snapshot_reflects_faulted_runtime_state() {
    let telemetry = Arc::new(RuntimeTelemetry::new());
    let tr909_render_state = Arc::new(SharedTr909RenderState::new(&Tr909RenderState::default()));
    let mc202_render_state = Arc::new(SharedMc202RenderState::new(&Mc202RenderState::default()));
    let w30_preview_state = Arc::new(SharedW30PreviewRenderState::new(
        &W30PreviewRenderState::default(),
    ));
    let w30_resample_tap_state = Arc::new(SharedW30ResampleTapState::new(
        &W30ResampleTapState::default(),
    ));
    let source_monitor_state = Arc::new(SharedSourceMonitorRenderState::new(
        &SourceMonitorRenderState::default(),
    ));
    let transport = Arc::new(SharedTransportTimingState::new(false, 128.0, 0.0));
    telemetry.record_callback_at(100, &sample_timing(12.0));
    telemetry.record_callback_at(240, &sample_timing(12.25));
    telemetry.record_stream_error("stream stalled".into());

    let shell = AudioRuntimeShell::from_test_parts(AudioRuntimeShellTestParts {
        lifecycle: AudioRuntimeLifecycle::Running,
        output: Some(sample_output()),
        telemetry,
        transport,
        tr909_render: tr909_render_state,
        mc202_render: mc202_render_state,
        w30_preview: w30_preview_state,
        w30_resample_tap: w30_resample_tap_state,
        source_monitor: source_monitor_state,
    });

    let snapshot = shell.health_snapshot();

    assert_eq!(snapshot.lifecycle, AudioRuntimeLifecycle::Faulted);
    assert_eq!(snapshot.callback_count, 2);
    assert_eq!(snapshot.max_callback_gap_micros, Some(140));
    assert_eq!(snapshot.callback_scratch_overflow_count, 0);
    assert_eq!(snapshot.stream_error_count, 1);
    assert_eq!(
        snapshot.last_stream_error.as_deref(),
        Some("stream stalled")
    );
}

#[test]
fn stop_transitions_runtime_to_stopped() {
    let telemetry = Arc::new(RuntimeTelemetry::new());
    let tr909_render_state = Arc::new(SharedTr909RenderState::new(&Tr909RenderState::default()));
    let mc202_render_state = Arc::new(SharedMc202RenderState::new(&Mc202RenderState::default()));
    let w30_preview_state = Arc::new(SharedW30PreviewRenderState::new(
        &W30PreviewRenderState::default(),
    ));
    let w30_resample_tap_state = Arc::new(SharedW30ResampleTapState::new(
        &W30ResampleTapState::default(),
    ));
    let source_monitor_state = Arc::new(SharedSourceMonitorRenderState::new(
        &SourceMonitorRenderState::default(),
    ));
    let transport = Arc::new(SharedTransportTimingState::new(false, 128.0, 0.0));
    let mut shell = AudioRuntimeShell::from_test_parts(AudioRuntimeShellTestParts {
        lifecycle: AudioRuntimeLifecycle::Running,
        output: Some(sample_output()),
        telemetry,
        transport,
        tr909_render: tr909_render_state,
        mc202_render: mc202_render_state,
        w30_preview: w30_preview_state,
        w30_resample_tap: w30_resample_tap_state,
        source_monitor: source_monitor_state,
    });

    shell.stop();

    assert_eq!(shell.lifecycle(), AudioRuntimeLifecycle::Stopped);
}

#[test]
fn timing_snapshot_reflects_callback_owned_transport_progress() {
    let telemetry = Arc::new(RuntimeTelemetry::new());
    let transport = Arc::new(SharedTransportTimingState::new(true, 126.0, 32.0));
    let shell = AudioRuntimeShell::from_test_parts(AudioRuntimeShellTestParts {
        lifecycle: AudioRuntimeLifecycle::Running,
        output: Some(sample_output()),
        telemetry: Arc::clone(&telemetry),
        transport: Arc::clone(&transport),
        tr909_render: Arc::new(SharedTr909RenderState::new(&Tr909RenderState::default())),
        mc202_render: Arc::new(SharedMc202RenderState::new(&Mc202RenderState::default())),
        w30_preview: Arc::new(SharedW30PreviewRenderState::new(
            &W30PreviewRenderState::default(),
        )),
        w30_resample_tap: Arc::new(SharedW30ResampleTapState::new(
            &W30ResampleTapState::default(),
        )),
        source_monitor: Arc::new(SharedSourceMonitorRenderState::new(
            &SourceMonitorRenderState::default(),
        )),
    });

    telemetry.record_callback_at(
        100,
        &CallbackTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 126.0,
            render_position_beats: 32.0,
            completed_position_beats: 32.5,
        },
    );

    let snapshot = shell.timing_snapshot();
    assert!(snapshot.is_transport_running);
    assert_eq!(snapshot.tempo_bpm, 126.0);
    assert_eq!(snapshot.position_beats, 32.5);
}
