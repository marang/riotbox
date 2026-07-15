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
    let full_metrics = signal_metrics(&full_block);
    let delta = signal_delta_metrics(&full_block, &realtime_simulated);

    assert!(full_metrics.active_samples > 1_000);
    assert!(full_metrics.rms > 0.001);
    assert_eq!(full_block.len(), realtime_simulated.len());
    assert_eq!(delta.active_samples, 0);
    assert_eq!(delta.rms, 0.0);
}

#[test]
fn realtime_transport_progress_reaches_late_mc202_source_steps() {
    let mut source_plan = runtime_mix_parity_source_plan();
    source_plan.active_mask = 1_u16 << 7;
    source_plan.accent_mask = 1_u16 << 7;
    source_plan.semitones[7] = -12;
    let plan = RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 130.0,
            position_beats: 0.0,
        },
        mc202_render: Mc202RenderState {
            mode: Mc202RenderMode::Pressure,
            routing: Mc202RenderRouting::MusicBusBass,
            source_phrase_plan: Some(source_plan),
            touch: 0.84,
            music_bus_level: 0.82,
            ..Mc202RenderState::default()
        },
        source_monitor_render: SourceMonitorRenderState::control_only(
            riotbox_core::action::SourceMonitorMode::Riotbox,
        ),
        ..RuntimeMixRenderPlan::default()
    };
    let frame_count = 48_000;

    let rendered =
        render_runtime_mix_realtime_simulation_offline(&plan, 48_000, 2, frame_count, 128);
    let metrics = signal_metrics(&rendered);

    assert!(metrics.active_samples > 1_000);
    assert!(metrics.rms > 0.005);
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
}

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
    let metrics = signal_metrics(&full_block);
    let delta = signal_delta_metrics(&full_block, &realtime_simulated);

    assert!(metrics.active_samples > 1_000);
    assert!(metrics.rms > 0.05);
    assert_eq!(metrics.clip_count, 0);
    assert!(metrics.peak_abs <= master_bus_limiter_ceiling() + 0.000_001);
    assert_eq!(delta.active_samples, 0);
}

fn runtime_mix_parity_source_plan() -> Mc202SourcePhraseRenderPlan {
    Mc202SourcePhraseRenderPlan {
        active_mask: 0b0001_0001_0010_0101,
        semitones: [-12, 0, -7, 0, 0, -5, 0, 0, -10, 0, 0, 0, -3, 0, 0, 0],
        accent_mask: 0b0001_0000_0000_0001,
        destructive_mask: 0b0000_0000_0001_0000,
        pressure: 0.70,
        contrast: 0.56,
        bass_weight: 0.72,
        stab_bite: 0.26,
        gate_snap: 0.22,
    }
}

fn runtime_mix_parity_source_window() -> W30PreviewSampleWindow {
    let mut window = W30PreviewSampleWindow {
        source_start_frame: 4_096,
        source_end_frame: 4_096 + W30_PREVIEW_SAMPLE_WINDOW_LEN as u64,
        sample_count: W30_PREVIEW_SAMPLE_WINDOW_LEN,
        samples: [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN],
    };
    for (index, sample) in window.samples.iter_mut().enumerate() {
        let phase = index as f32 / W30_PREVIEW_SAMPLE_WINDOW_LEN as f32;
        *sample = ((phase * std::f32::consts::TAU * 3.0).sin() * 0.45)
            + ((phase * std::f32::consts::TAU * 11.0).sin() * 0.15);
    }
    window
}

fn runtime_mix_duration_pad() -> W30PadPlaybackSampleWindow {
    let mut samples = [0.0; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN];
    for (index, sample) in samples.iter_mut().enumerate() {
        let phase = index as f32 / W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN as f32;
        let transient = if index % 2_048 < 96 { 0.5 } else { 0.0 };
        *sample = ((phase * std::f32::consts::TAU * 7.0).sin() * 0.42) + transient;
    }
    W30PadPlaybackSampleWindow {
        source_start_frame: 0,
        source_end_frame: 96_000,
        source_sample_rate: 48_000,
        playback_frame_count: 96_000,
        sample_count: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN,
        loop_enabled: true,
        playback_rate: 1.0,
        reverse: false,
        loop_crossfade_sample_count: 128,
        chop_slice_count: 0,
        chop_slice_starts: [0; W30_PAD_CHOP_SLICE_COUNT],
        samples,
    }
}
