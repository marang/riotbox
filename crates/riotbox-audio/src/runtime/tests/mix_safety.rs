use crate::mc202::{Mc202PhraseShape, Mc202RenderMode, Mc202RenderRouting, Mc202RenderState};
use crate::runtime::source_monitor::{SourceMonitorAudioSource, SourceMonitorRenderState};
use crate::runtime::tests::mix_plan_fixtures::{
    runtime_mix_parity_source_plan, runtime_mix_parity_source_window, runtime_mix_resample_source,
};
use crate::runtime::{
    AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan, RuntimeMixRenderSequenceStep,
    master_bus_limiter_ceiling, master_bus_limiter_threshold, render_runtime_mix_offline,
    render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report,
    render_runtime_mix_realtime_simulation_offline, signal_delta_metrics, signal_metrics,
};
use crate::source_audio::SourceAudioCache;
use crate::tr909::{Tr909RenderMode, Tr909RenderRouting, Tr909RenderState};
use crate::w30::{
    W30PreviewRenderMode, W30PreviewRenderRouting, W30PreviewRenderState, W30PreviewSourceProfile,
    W30ResampleTapAvailability, W30ResampleTapMode, W30ResampleTapRouting,
    W30ResampleTapSourceProfile, W30ResampleTapState,
};
use riotbox_core::action::SourceMonitorMode;

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

#[test]
fn runtime_mix_limiter_baseline_names_five_sounding_owners_and_honest_hot_sum() {
    #[derive(Clone, Copy, Debug)]
    enum Owner {
        Tr909,
        Mc202,
        W30Preview,
        W30Resample,
        SourceMonitor,
    }
    let source = SourceAudioCache::from_interleaved_samples(
        "synthetic-limiter-monitor.wav",
        44_100,
        1,
        vec![1.0; 2_048],
    )
    .expect("in-memory synthetic cache, no file access");
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
        w30_resample_tap: W30ResampleTapState {
            mode: W30ResampleTapMode::CaptureLineageReady,
            routing: W30ResampleTapRouting::InternalCaptureTap,
            availability: W30ResampleTapAvailability::SourceAudioReady,
            source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
            source_capture_id: Some("synthetic-limiter-capture".into()),
            source_audio: Some(Box::new(runtime_mix_resample_source())),
            lineage_capture_count: 2,
            generation_depth: 1,
            music_bus_level: 4.0,
            grit_level: 1.0,
            ..W30ResampleTapState::default()
        },
        source_monitor_render: SourceMonitorRenderState {
            mode: SourceMonitorMode::Blend,
            source: Some(SourceMonitorAudioSource::from_cache(&source)),
            source_anchor_seconds: Some(0.0),
            source_anchor_position_beats: 32.0,
            ..SourceMonitorRenderState::default()
        },
    };
    let render = |plan: &RuntimeMixRenderPlan| {
        render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
            &[RuntimeMixRenderSequenceStep::new(plan, 2_048)],
            44_100,
            2,
            128,
        )
        .pop()
        .expect("exact synthetic RuntimeMix output")
    };
    let composite = render(&plan);
    eprintln!("five_owner_hot: {:?}", composite.limiter);
    assert_eq!(composite.samples.len(), 4_096);
    assert!(composite.samples.iter().all(|sample| sample.is_finite()));
    assert!(composite.limiter.pre.peak_abs > 1.0);
    assert!(composite.limiter.pre.clip_count > 0);
    assert!(composite.limiter.applied);
    assert!(composite.limiter.limited_sample_count > 0);
    assert_eq!(composite.limiter.post.clip_count, 0);
    assert!(composite.limiter.post.peak_abs <= master_bus_limiter_ceiling());
    assert_eq!(composite.limiter.post, signal_metrics(&composite.samples));
    let repeated = render(&plan);
    eprintln!("five_owner_repeat: {:?}", repeated.limiter);
    assert_eq!(repeated, composite);

    let silent = RuntimeMixRenderPlan {
        transport: plan.transport,
        ..RuntimeMixRenderPlan::default()
    };
    let silent_output = render(&silent);
    eprintln!("silent_routes: {:?}", silent_output.limiter);
    assert!(silent_output.samples.iter().all(|sample| *sample == 0.0));
    assert!(!silent_output.limiter.applied);
    for owner in [
        Owner::Tr909,
        Owner::Mc202,
        Owner::W30Preview,
        Owner::W30Resample,
        Owner::SourceMonitor,
    ] {
        let mut isolated = silent.clone();
        let mut without = plan.clone();
        match owner {
            Owner::Tr909 => {
                isolated.tr909_render = plan.tr909_render.clone();
                without.tr909_render = Tr909RenderState::default();
            }
            Owner::Mc202 => {
                isolated.mc202_render = plan.mc202_render;
                without.mc202_render = Mc202RenderState::default();
            }
            Owner::W30Preview => {
                isolated.w30_preview_render = plan.w30_preview_render.clone();
                without.w30_preview_render = W30PreviewRenderState::default();
            }
            Owner::W30Resample => {
                isolated.w30_resample_tap = plan.w30_resample_tap.clone();
                without.w30_resample_tap = W30ResampleTapState::default();
            }
            Owner::SourceMonitor => {
                isolated.source_monitor_render = plan.source_monitor_render.clone();
                isolated.source_monitor_render.mode = SourceMonitorMode::Source;
                // Retain Blend's generated gain; remove only the source contribution.
                without.source_monitor_render.source = None;
            }
        }
        let isolated_output = render(&isolated);
        let without_output = render(&without);
        eprintln!("isolated_{owner:?}: {:?}", isolated_output.limiter);
        eprintln!("without_{owner:?}: {:?}", without_output.limiter);
        assert!(
            isolated_output.limiter.post.rms > 0.0,
            "{owner:?} must sound"
        );
        assert_ne!(
            without_output.limiter.pre, composite.limiter.pre,
            "{owner:?} must contribute"
        );
    }
}
