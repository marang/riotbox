use crate::mc202::{Mc202PhraseShape, Mc202RenderMode, Mc202RenderRouting, Mc202RenderState};
use crate::runtime::tests::mix_plan_fixtures::{
    runtime_mix_parity_source_plan, runtime_mix_parity_source_window,
};
use crate::runtime::{
    AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan, RuntimeMixRenderSequenceStep,
    master_bus_limiter_ceiling, master_bus_limiter_threshold, render_runtime_mix_offline,
    render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report,
    render_runtime_mix_realtime_simulation_offline, signal_delta_metrics, signal_metrics,
};
use crate::tr909::{Tr909RenderMode, Tr909RenderRouting, Tr909RenderState};
use crate::w30::{
    W30PreviewRenderMode, W30PreviewRenderRouting, W30PreviewRenderState, W30PreviewSourceProfile,
};

#[test]
fn runtime_mix_plan_default_keeps_riotbox_output_enabled_without_source() {
    let plan = RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
        },
        tr909_render: Tr909RenderState {
            mode: Tr909RenderMode::Fill,
            routing: Tr909RenderRouting::DrumBusSupport,
            drum_bus_level: 0.72,
            ..Tr909RenderState::default()
        },
        ..RuntimeMixRenderPlan::default()
    };

    let output = render_runtime_mix_offline(&plan, 44_100, 2, 1_024);
    let metrics = signal_metrics(&output);

    assert!(metrics.active_samples > 100);
    assert!(metrics.rms > 0.001);
}

#[test]
fn runtime_mix_master_bus_limiter_controls_hot_product_mix() {
    let plan = RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
        },
        tr909_render: Tr909RenderState {
            mode: Tr909RenderMode::Fill,
            routing: Tr909RenderRouting::DrumBusSupport,
            drum_bus_level: 4.0,
            slam_intensity: 2.5,
            ..Tr909RenderState::default()
        },
        mc202_render: Mc202RenderState {
            mode: Mc202RenderMode::Instigator,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::InstigatorSpike,
            source_phrase_plan: Some(runtime_mix_parity_source_plan()),
            touch: 3.0,
            music_bus_level: 4.0,
            ..Mc202RenderState::default()
        },
        w30_preview_render: W30PreviewRenderState {
            mode: W30PreviewRenderMode::RawCaptureAudition,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
            source_window_preview: Some(runtime_mix_parity_source_window()),
            music_bus_level: 4.0,
            grit_level: 1.0,
            ..W30PreviewRenderState::default()
        },
        ..RuntimeMixRenderPlan::default()
    };

    let full_block = render_runtime_mix_offline(&plan, 44_100, 2, 2_048);
    let realtime_simulated =
        render_runtime_mix_realtime_simulation_offline(&plan, 44_100, 2, 2_048, 128);
    let reported = render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(&plan, 2_048)],
        44_100,
        2,
        128,
    )
    .pop()
    .expect("reported hot exact-mix render");
    let metrics = signal_metrics(&full_block);
    let delta = signal_delta_metrics(&full_block, &realtime_simulated);
    let report_delta = signal_delta_metrics(&realtime_simulated, &reported.samples);

    assert!(metrics.active_samples > 1_000);
    assert!(metrics.rms > 0.05);
    assert_eq!(metrics.clip_count, 0);
    assert!(metrics.peak_abs <= master_bus_limiter_ceiling() + 0.000_001);
    assert_eq!(delta.active_samples, 0);
    assert_eq!(report_delta.active_samples, 0);
    assert!(reported.limiter.applied);
    assert!(reported.limiter.limited_sample_count > 0);
    assert!(reported.limiter.pre.peak_abs > master_bus_limiter_threshold());
    assert!(reported.limiter.pre.clip_count > 0);
    assert_eq!(reported.limiter.post.clip_count, 0);
    assert!(reported.limiter.post.peak_abs <= master_bus_limiter_ceiling() + 0.000_001);
}
