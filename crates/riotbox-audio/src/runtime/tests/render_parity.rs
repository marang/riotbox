use crate::mc202::{Mc202PhraseShape, Mc202RenderMode, Mc202RenderRouting, Mc202RenderState};
use crate::runtime::source_monitor::{SourceMonitorAudioSource, SourceMonitorRenderState};
use crate::runtime::tests::mix_plan_fixtures::{
    runtime_mix_duration_pad, runtime_mix_parity_source_plan, runtime_mix_parity_source_window,
    runtime_mix_resample_source,
};
use crate::runtime::{
    AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan, RuntimeMixRenderSequenceStep,
    render_runtime_mix_offline, render_runtime_mix_plan_sequence_realtime_simulation_offline,
    render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report,
    render_runtime_mix_realtime_simulation_offline, signal_delta_metrics, signal_metrics,
};
use crate::source_audio::SourceAudioCache;
use crate::tr909::{
    Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
    Tr909RenderState,
};
use crate::w30::{
    W30PreviewRenderMode, W30PreviewRenderRouting, W30PreviewRenderState, W30PreviewSourceProfile,
    W30ResampleTapAvailability, W30ResampleTapMode, W30ResampleTapRouting,
    W30ResampleTapSourceProfile, W30ResampleTapState,
};

#[test]
fn runtime_mix_realtime_simulation_matches_full_block_offline_render() {
    let frame_count = 2_048;
    let plan = RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 32.0,
        },
        tr909_render: Tr909RenderState {
            mode: Tr909RenderMode::Fill,
            routing: Tr909RenderRouting::DrumBusSupport,
            pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
            phrase_variation: Some(Tr909PhraseVariation::PhraseLift),
            drum_bus_level: 0.68,
            slam_intensity: 0.54,
            ..Tr909RenderState::default()
        },
        mc202_render: Mc202RenderState {
            mode: Mc202RenderMode::Instigator,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::InstigatorSpike,
            source_phrase_plan: Some(runtime_mix_parity_source_plan()),
            touch: 0.86,
            music_bus_level: 0.56,
            ..Mc202RenderState::default()
        },
        w30_preview_render: W30PreviewRenderState {
            mode: W30PreviewRenderMode::RawCaptureAudition,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
            source_window_preview: Some(runtime_mix_parity_source_window()),
            music_bus_level: 0.34,
            grit_level: 0.18,
            ..W30PreviewRenderState::default()
        },
        w30_resample_tap: W30ResampleTapState::default(),
        source_monitor_render: SourceMonitorRenderState::control_only(
            riotbox_core::action::SourceMonitorMode::Riotbox,
        ),
    };

    let full_block = render_runtime_mix_offline(&plan, 44_100, 2, frame_count);
    let realtime_simulated =
        render_runtime_mix_realtime_simulation_offline(&plan, 44_100, 2, frame_count, 128);
    let reported = render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(&plan, frame_count)],
        44_100,
        2,
        128,
    )
    .pop()
    .expect("reported exact-mix render");
    let full_metrics = signal_metrics(&full_block);
    let delta = signal_delta_metrics(&full_block, &realtime_simulated);
    let report_delta = signal_delta_metrics(&realtime_simulated, &reported.samples);

    assert!(full_metrics.active_samples > 1_000);
    assert!(full_metrics.rms > 0.001);
    assert_eq!(full_block.len(), realtime_simulated.len());
    assert_eq!(delta.active_samples, 0);
    assert_eq!(delta.rms, 0.0);
    assert_eq!(report_delta.active_samples, 0);
    assert!(!reported.limiter.applied);
    assert_eq!(reported.limiter.limited_sample_count, 0);
    assert_eq!(reported.limiter.pre, reported.limiter.post);
}

#[test]
fn runtime_mix_plan_sequence_preserves_callback_state_and_segment_lengths() {
    let segment_frame_count = 1_024;
    let plan = RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 127.0,
            position_beats: 4.0,
        },
        tr909_render: Tr909RenderState {
            mode: Tr909RenderMode::Fill,
            routing: Tr909RenderRouting::DrumBusSupport,
            drum_bus_level: 0.68,
            slam_intensity: 0.54,
            ..Tr909RenderState::default()
        },
        w30_preview_render: W30PreviewRenderState {
            mode: W30PreviewRenderMode::LiveRecall,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
            trigger_revision: 4,
            trigger_velocity: 0.86,
            pad_playback: Some(runtime_mix_duration_pad()),
            music_bus_level: 0.58,
            grit_level: 0.31,
            ..W30PreviewRenderState::default()
        },
        w30_resample_tap: W30ResampleTapState {
            mode: W30ResampleTapMode::CaptureLineageReady,
            routing: W30ResampleTapRouting::InternalCaptureTap,
            availability: W30ResampleTapAvailability::SourceAudioReady,
            source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
            source_capture_id: Some("sequence-capture".into()),
            source_audio: Some(Box::new(runtime_mix_resample_source())),
            lineage_capture_count: 2,
            generation_depth: 1,
            music_bus_level: 0.34,
            grit_level: 0.48,
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 0.0,
        },
        ..RuntimeMixRenderPlan::default()
    };

    let sequence = render_runtime_mix_plan_sequence_realtime_simulation_offline(
        &[
            RuntimeMixRenderSequenceStep::new(&plan, segment_frame_count),
            RuntimeMixRenderSequenceStep::new(&plan, segment_frame_count),
        ],
        48_000,
        2,
        128,
    );
    let continuous = render_runtime_mix_realtime_simulation_offline(
        &plan,
        48_000,
        2,
        segment_frame_count * 2,
        128,
    );
    let flattened = sequence.iter().flatten().copied().collect::<Vec<_>>();

    assert_eq!(sequence.len(), 2);
    assert_eq!(sequence[0].len(), segment_frame_count * 2);
    assert_eq!(sequence[1].len(), segment_frame_count * 2);
    assert_ne!(sequence[0], sequence[1]);
    assert_eq!(flattened, continuous);
}

#[test]
fn exact_runtime_mix_sequence_replaces_source_pcm_without_stale_playback() {
    let source_a =
        SourceAudioCache::from_interleaved_samples("source-a.wav", 1_000, 1, vec![0.25; 1_024])
            .expect("source A cache");
    let source_b =
        SourceAudioCache::from_interleaved_samples("source-b.wav", 1_000, 1, vec![-0.5; 1_024])
            .expect("source B cache");
    let source_a_plan = RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 60.0,
            position_beats: 0.0,
        },
        source_monitor_render: SourceMonitorRenderState {
            mode: riotbox_core::action::SourceMonitorMode::Source,
            source: Some(SourceMonitorAudioSource::from_cache(&source_a)),
            is_transport_running: true,
            tempo_bpm: 60.0,
            position_beats: 0.0,
            ..SourceMonitorRenderState::default()
        },
        ..RuntimeMixRenderPlan::default()
    };
    let mut source_b_plan = source_a_plan.clone();
    source_b_plan.source_monitor_render.source =
        Some(SourceMonitorAudioSource::from_cache(&source_b));

    let segments = render_runtime_mix_plan_sequence_realtime_simulation_offline(
        &[
            RuntimeMixRenderSequenceStep::new(&source_a_plan, 128),
            RuntimeMixRenderSequenceStep::new(&source_b_plan, 128),
        ],
        1_000,
        1,
        64,
    );

    assert!(
        segments[0]
            .iter()
            .all(|sample| (*sample - 0.25 * 0.88).abs() < 1.0e-6)
    );
    assert!(
        segments[1]
            .iter()
            .all(|sample| (*sample + 0.5 * 0.88).abs() < 1.0e-6)
    );
    assert!(signal_delta_metrics(&segments[0], &segments[1]).rms > 0.6);
}

#[test]
fn runtime_mix_plan_sequence_observes_w30_trigger_revision_between_callbacks() {
    let ready = RuntimeMixRenderPlan {
        w30_preview_render: W30PreviewRenderState {
            mode: W30PreviewRenderMode::LiveRecall,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
            trigger_revision: 9,
            trigger_velocity: 0.82,
            pad_playback: Some(runtime_mix_duration_pad()),
            music_bus_level: 0.62,
            grit_level: 0.28,
            ..W30PreviewRenderState::default()
        },
        ..RuntimeMixRenderPlan::default()
    };
    let mut triggered = ready.clone();
    triggered.w30_preview_render.trigger_revision += 1;
    let segment_frame_count = 512;

    let unchanged = render_runtime_mix_plan_sequence_realtime_simulation_offline(
        &[
            RuntimeMixRenderSequenceStep::new(&ready, segment_frame_count),
            RuntimeMixRenderSequenceStep::new(&ready, segment_frame_count),
        ],
        48_000,
        2,
        128,
    );
    let with_trigger = render_runtime_mix_plan_sequence_realtime_simulation_offline(
        &[
            RuntimeMixRenderSequenceStep::new(&ready, segment_frame_count),
            RuntimeMixRenderSequenceStep::new(&triggered, segment_frame_count),
        ],
        48_000,
        2,
        128,
    );
    let trigger_delta = signal_delta_metrics(&unchanged[1], &with_trigger[1]);

    assert_eq!(unchanged.len(), 2);
    assert_eq!(with_trigger.len(), 2);
    assert_eq!(unchanged[0], with_trigger[0]);
    assert!(trigger_delta.active_samples > 500);
    assert!(trigger_delta.rms > 0.01);
}

#[test]
fn duration_aware_w30_pad_matches_exact_realtime_mix_path() {
    let frame_count = 48_000;
    let w30_render = W30PreviewRenderState {
        mode: W30PreviewRenderMode::LiveRecall,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile: Some(W30PreviewSourceProfile::PromotedRecall),
        trigger_revision: 7,
        trigger_velocity: 0.84,
        pad_playback: Some(runtime_mix_duration_pad()),
        music_bus_level: 0.64,
        grit_level: 0.38,
        is_transport_running: true,
        tempo_bpm: 130.0,
        ..W30PreviewRenderState::default()
    };
    let plan = RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 130.0,
            position_beats: 0.0,
        },
        w30_preview_render: w30_render.clone(),
        ..RuntimeMixRenderPlan::default()
    };
    let silent_plan = RuntimeMixRenderPlan {
        w30_preview_render: W30PreviewRenderState::default(),
        ..plan.clone()
    };

    let full_block = render_runtime_mix_offline(&plan, 48_000, 2, frame_count);
    let realtime_simulated =
        render_runtime_mix_realtime_simulation_offline(&plan, 48_000, 2, frame_count, 128);
    let silence = render_runtime_mix_offline(&silent_plan, 48_000, 2, frame_count);
    let metrics = signal_metrics(&full_block);
    let parity_delta = signal_delta_metrics(&full_block, &realtime_simulated);
    let source_delta = signal_delta_metrics(&full_block, &silence);

    assert!(metrics.active_samples > 80_000);
    assert!(metrics.rms > 0.01);
    assert_eq!(metrics.clip_count, 0);
    assert_eq!(parity_delta.active_samples, 0);
    assert_eq!(parity_delta.rms, 0.0);
    assert!(source_delta.rms > 0.01);

    let mut gated_plan = plan;
    gated_plan
        .w30_preview_render
        .pad_playback
        .as_mut()
        .expect("duration pad")
        .gate_step_fraction = 0.36;
    let gated_full = render_runtime_mix_offline(&gated_plan, 48_000, 2, frame_count);
    let gated_realtime =
        render_runtime_mix_realtime_simulation_offline(&gated_plan, 48_000, 2, frame_count, 128);
    let gated_metrics = signal_metrics(&gated_full);
    let gated_parity_delta = signal_delta_metrics(&gated_full, &gated_realtime);

    assert!(gated_metrics.active_samples > 20_000);
    assert!(gated_metrics.active_samples < metrics.active_samples);
    assert!(gated_metrics.rms > 0.01);
    assert_eq!(gated_metrics.clip_count, 0);
    assert_eq!(gated_parity_delta.active_samples, 0);
    assert_eq!(gated_parity_delta.rms, 0.0);
}
