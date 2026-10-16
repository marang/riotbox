//! Shared generated W-30 setup for exact callback transport regressions.
use std::time::Instant;

use riotbox_core::action::SourceMonitorMode;

use super::{prepare_output_callback, shared};
use crate::{
    runtime::{
        AudioRuntimeLifecycle, AudioRuntimeShell, AudioTransportPositionRequest,
        AudioTransportPositionUpdate, SourceMonitorRenderState,
        shared_transport_tr909::{AudioRuntimeSharedState, AudioRuntimeShellTestParts},
    },
    test_heap::{HeapCounts, measure},
    w30::{
        W30_PAD_CHOP_SLICE_COUNT, W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN, W30HookArticulationProfile,
        W30HookArticulationRenderState, W30PadPlaybackSampleWindow, W30PreviewRenderMode,
        W30PreviewRenderRouting, W30PreviewRenderState, W30PreviewSourceProfile,
        W30ResampleTapState,
    },
};

pub(super) fn reposition(position: f64) -> AudioTransportPositionUpdate {
    AudioTransportPositionUpdate::Reposition(AudioTransportPositionRequest::from_beats(position))
}

pub(super) fn generated_preview(pitch_dive: bool) -> W30PreviewRenderState {
    W30PreviewRenderState {
        mode: W30PreviewRenderMode::LiveRecall,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
        trigger_revision: 1,
        trigger_velocity: 0.8,
        pad_playback: Some(W30PadPlaybackSampleWindow {
            source_start_frame: 0,
            source_end_frame: 1000,
            source_sample_rate: 1000,
            playback_frame_count: 1000,
            sample_count: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN,
            loop_enabled: true,
            playback_rate: 1.0,
            reverse: false,
            gate_step_fraction: 0.0,
            loop_crossfade_sample_count: 0,
            chop_slice_count: 0,
            chop_slice_starts: [0; W30_PAD_CHOP_SLICE_COUNT],
            hook_articulation: pitch_dive.then_some(W30HookArticulationRenderState {
                profile: W30HookArticulationProfile::PitchDiveV1,
                started_at_beat: 0,
            }),
            samples: [0.2; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
        }),
        music_bus_level: 0.6,
        grit_level: 0.0,
        ..W30PreviewRenderState::default()
    }
}

pub(super) fn marked_preview() -> W30PreviewRenderState {
    let mut preview = generated_preview(false);
    for (index, sample) in preview
        .pad_playback
        .as_mut()
        .unwrap()
        .samples
        .iter_mut()
        .enumerate()
    {
        *sample = 0.08 + (index % 67) as f32 * 0.001;
    }
    preview
}

pub(super) fn prepared(
    pitch_dive: bool,
) -> (
    AudioRuntimeSharedState,
    AudioRuntimeShell,
    impl FnMut(&mut [f32]),
) {
    prepared_with_preview(generated_preview(pitch_dive))
}

pub(super) fn prepared_with_preview(
    preview: W30PreviewRenderState,
) -> (
    AudioRuntimeSharedState,
    AudioRuntimeShell,
    impl FnMut(&mut [f32]),
) {
    prepared_with_w30(preview, W30ResampleTapState::default())
}

pub(super) fn prepared_with_resample(
    resample: W30ResampleTapState,
) -> (
    AudioRuntimeSharedState,
    AudioRuntimeShell,
    impl FnMut(&mut [f32]),
) {
    prepared_with_w30(W30PreviewRenderState::default(), resample)
}

fn prepared_with_w30(
    preview: W30PreviewRenderState,
    resample: W30ResampleTapState,
) -> (
    AudioRuntimeSharedState,
    AudioRuntimeShell,
    impl FnMut(&mut [f32]),
) {
    let state = shared();
    state
        .source_monitor
        .update_controls(&SourceMonitorRenderState::control_only(
            SourceMonitorMode::Riotbox,
        ));
    state.w30_preview.update(&preview);
    state.w30_resample_tap.update(&resample);
    let shell = AudioRuntimeShell::from_test_parts(AudioRuntimeShellTestParts {
        lifecycle: AudioRuntimeLifecycle::Running,
        output: None,
        telemetry: state.telemetry.clone(),
        transport: state.transport.clone(),
        tr909_render: state.tr909_render.clone(),
        mc202_render: state.mc202_render.clone(),
        w30_preview: state.w30_preview.clone(),
        w30_resample_tap: state.w30_resample_tap.clone(),
        source_monitor: state.source_monitor.clone(),
    });
    shell
        .update_transport_state(true, 120.0, reposition(0.0))
        .unwrap();
    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: 1000,
        buffer_size: cpal::BufferSize::Fixed(50),
    };
    let callback = prepare_output_callback::<f32>(&config, state.clone(), Instant::now()).unwrap();
    // Multiple exact processors are compared in one ordinary test thread.
    // Keep their large prepared captures off that thread's stack; this test-
    // control allocation occurs before every callback heap measurement.
    (state, shell, Box::new(callback))
}

pub(super) fn process(callback: &mut impl FnMut(&mut [f32]), output: &mut [f32]) {
    let (_, heap) = measure(|| callback(output));
    assert_eq!(heap, HeapCounts::default());
    assert!(output.iter().all(|sample| sample.is_finite()));
}

pub(super) fn assert_pcm_eq(actual: &[f32], expected: &[f32]) {
    assert_eq!(actual.len(), expected.len());
    for (frame, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "PCM differs at frame{frame}"
        );
    }
}

pub(super) fn warm(shell: &AudioRuntimeShell, callback: &mut impl FnMut(&mut [f32])) {
    let mut output = [0.0; 50];
    for _ in 0..15 {
        process(callback, &mut output);
    }
    assert!((shell.timing_snapshot().unwrap().position_beats - 1.5).abs() < 1e-11);
}
