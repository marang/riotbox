//! Frozen public numeric ingress: no implicit live W-30 reposition semantics.
//! Generated in-memory PCM only; no source path, device or file access.
use crate::{
    runtime::{
        AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan, RuntimeMixRenderSequenceStep,
        render_runtime_mix_plan_sequence_realtime_simulation_offline,
    },
    w30::{
        W30_PREVIEW_SAMPLE_WINDOW_LEN, W30_RESAMPLE_SOURCE_WINDOW_LEN, W30PreviewRenderMode,
        W30PreviewRenderRouting, W30PreviewSampleWindow, W30PreviewSourceProfile,
        W30ResampleSourceWindow, W30ResampleTapAvailability, W30ResampleTapMode,
        W30ResampleTapRouting, W30ResampleTapSourceProfile, W30ResampleTapState,
    },
};

fn preview(position: f64) -> RuntimeMixRenderPlan {
    let mut plan = RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 120.0,
            position_beats: position,
        },
        ..RuntimeMixRenderPlan::default()
    };
    plan.w30_preview_render.mode = W30PreviewRenderMode::LiveRecall;
    plan.w30_preview_render.routing = W30PreviewRenderRouting::MusicBusPreview;
    plan.w30_preview_render.source_profile = Some(W30PreviewSourceProfile::PromotedRecall);
    plan.w30_preview_render.music_bus_level = 0.5;
    let mut samples = [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN];
    for (index, sample) in samples.iter_mut().enumerate() {
        *sample = 0.02 + 0.1 * index as f32 / W30_PREVIEW_SAMPLE_WINDOW_LEN as f32;
    }
    plan.w30_preview_render.source_window_preview = Some(W30PreviewSampleWindow {
        source_start_frame: 0,
        source_end_frame: W30_PREVIEW_SAMPLE_WINDOW_LEN as u64,
        sample_count: W30_PREVIEW_SAMPLE_WINDOW_LEN,
        samples,
    });
    plan
}

fn after_warmup(warm: &RuntimeMixRenderPlan, next: &RuntimeMixRenderPlan) -> Vec<f32> {
    render_runtime_mix_plan_sequence_realtime_simulation_offline(
        &[
            RuntimeMixRenderSequenceStep::new(warm, 750),
            RuntimeMixRenderSequenceStep::new(next, 1000),
        ],
        1000,
        1,
        50,
    )
    .pop()
    .unwrap()
}

#[test]
fn public_numeric_preview_forward_and_backward_changes_keep_historical_sample_bytes() {
    // These controls preserve the independently executed1596 witness exactly.
    // They intentionally do not assert the new typed-live behavior on an old
    // public numeric interface, nor require a seek to cold-retrigger a voice.
    assert_numeric_compatibility(preview);
}

fn resample(position: f64) -> RuntimeMixRenderPlan {
    let mut samples = [0.0; W30_RESAMPLE_SOURCE_WINDOW_LEN];
    for (index, sample) in samples.iter_mut().enumerate() {
        *sample = 0.02 + 0.1 * index as f32 / W30_RESAMPLE_SOURCE_WINDOW_LEN as f32;
    }
    RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 120.0,
            position_beats: position,
        },
        w30_resample_tap: W30ResampleTapState {
            mode: W30ResampleTapMode::CaptureLineageReady,
            routing: W30ResampleTapRouting::InternalCaptureTap,
            availability: W30ResampleTapAvailability::SourceAudioReady,
            source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
            source_audio: Some(Box::new(W30ResampleSourceWindow {
                source_start_frame: 0,
                source_sample_rate: 1000,
                source_frame_count: W30_RESAMPLE_SOURCE_WINDOW_LEN as u64,
                sample_count: W30_RESAMPLE_SOURCE_WINDOW_LEN,
                samples,
            })),
            music_bus_level: 0.5,
            ..W30ResampleTapState::default()
        },
        ..RuntimeMixRenderPlan::default()
    }
}

#[test]
fn public_numeric_resample_forward_and_backward_changes_keep_historical_sample_bytes() {
    assert_numeric_compatibility(resample);
}

fn assert_numeric_compatibility(plan: fn(f64) -> RuntimeMixRenderPlan) {
    let forward = after_warmup(&plan(0.0), &plan(4.0));
    let forward_control = after_warmup(&plan(0.0), &plan(0.0));
    let backward = after_warmup(&plan(8.0), &plan(4.0));
    let backward_control = after_warmup(&plan(8.0), &plan(8.0));
    let target = plan(4.0);
    let cold = render_runtime_mix_plan_sequence_realtime_simulation_offline(
        &[RuntimeMixRenderSequenceStep::new(&target, 1000)],
        1000,
        1,
        50,
    )
    .pop()
    .unwrap();
    for samples in [
        &forward,
        &forward_control,
        &backward,
        &backward_control,
        &cold,
    ] {
        assert!(samples.iter().all(|sample| sample.is_finite()));
        assert!(samples.iter().any(|sample| sample.abs() > 0.0001));
    }
    for (actual, expected) in [(&forward, &forward_control), (&backward, &backward_control)] {
        assert_eq!(actual.len(), expected.len());
        for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
            assert_eq!(actual.to_bits(), expected.to_bits(), "PCM bits at {index}");
        }
    }
    assert_ne!(
        forward, cold,
        "positive fixture must distinguish target-phase cold output"
    );
}
