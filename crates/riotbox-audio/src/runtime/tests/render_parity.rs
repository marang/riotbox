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
            source_profile: Some(W30ResampleTapSourceProfile::PromotedCapture),
            source_capture_id: Some("sequence-capture".into()),
            lineage_capture_count: 2,
            generation_depth: 1,
            music_bus_level: 0.34,
            grit_level: 0.48,
            is_transport_running: true,
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
fn exact_runtime_mix_transport_stop_fades_and_silences_all_w30_paths() {
    let running = RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 130.0,
            position_beats: 8.0,
        },
        w30_preview_render: W30PreviewRenderState {
            mode: W30PreviewRenderMode::RawCaptureAudition,
            routing: W30PreviewRenderRouting::MusicBusPreview,
            source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
            source_window_preview: Some(runtime_mix_parity_source_window()),
            music_bus_level: 0.58,
            grit_level: 0.4,
            is_transport_running: true,
            tempo_bpm: 130.0,
            position_beats: 8.0,
            ..W30PreviewRenderState::default()
        },
        w30_resample_tap: W30ResampleTapState {
            mode: W30ResampleTapMode::CaptureLineageReady,
            routing: W30ResampleTapRouting::InternalCaptureTap,
            source_profile: Some(W30ResampleTapSourceProfile::RawCapture),
            source_capture_id: Some("stop-proof-capture".into()),
            lineage_capture_count: 1,
            generation_depth: 0,
            music_bus_level: 0.34,
            grit_level: 0.4,
            is_transport_running: true,
        },
        ..RuntimeMixRenderPlan::default()
    };
    let mut stopped = running.clone();
    stopped.transport.is_transport_running = false;
    stopped.transport.position_beats = 8.25;
    stopped.w30_preview_render.is_transport_running = false;
    stopped.w30_preview_render.position_beats = 8.25;
    stopped.w30_resample_tap.is_transport_running = false;

    let segments = render_runtime_mix_plan_sequence_realtime_simulation_offline(
        &[
            RuntimeMixRenderSequenceStep::new(&running, 512),
            RuntimeMixRenderSequenceStep::new(&stopped, 1_024),
        ],
        44_100,
        2,
        128,
    );

    assert!(segments[0].iter().any(|sample| sample.abs() > 0.0001));
    let fade_sample_count = usize::try_from(44_100 / 200).unwrap() * 2;
    assert!(segments[1][..fade_sample_count]
        .iter()
        .any(|sample| sample.abs() > 0.0001));
    assert!(segments[1][fade_sample_count..]
        .iter()
        .all(|sample| sample.abs() <= f32::EPSILON));
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
fn fill_focus_is_typed_to_fill_support_and_the_drum_owned_half_bar_phase() {
    let render = RealtimeTr909RenderState {
        mode: Tr909RenderMode::Fill,
        routing: Tr909RenderRouting::DrumBusSupport,
        source_support_profile: None,
        source_support_context: None,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
        takeover_profile: None,
        drum_bus_level: 0.80,
        slam_enabled: false,
        slam_intensity: 0.66,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 1.93,
        source_bar_grid_anchor_position_beats: None,
    };
    let focus = FillFocusRenderState::from_tr909(&render);
    let frames_per_beat = 24_000;
    let frames_to_bar_wrap = frames_per_beat * 207 / 100;

    assert_eq!(focus.gain_at_frame(48_000, 0), 1.0);
    assert_eq!(focus.gain_at_frame(48_000, 1_680), 0.0);
    assert_eq!(
        focus.gain_at_frame(48_000, frames_per_beat / 2),
        0.0
    );
    let late_stomp_frame = frames_per_beat * 179 / 100;
    let release_frame = frames_per_beat * 204 / 100;
    assert_eq!(focus.gain_at_frame(48_000, late_stomp_frame), 0.0);
    assert!(focus.gain_at_frame(48_000, release_frame) > 0.0);
    assert_eq!(focus.gain_at_frame(48_000, frames_to_bar_wrap), 1.0);

    let non_fill = FillFocusRenderState::from_tr909(&RealtimeTr909RenderState {
        mode: Tr909RenderMode::BreakReinforce,
        ..render
    });
    let wrong_route = FillFocusRenderState::from_tr909(&RealtimeTr909RenderState {
        routing: Tr909RenderRouting::SourceOnly,
        ..render
    });
    let stopped = FillFocusRenderState::from_tr909(&RealtimeTr909RenderState {
        is_transport_running: false,
        ..render
    });
    let silent_fill = FillFocusRenderState::from_tr909(&RealtimeTr909RenderState {
        drum_bus_level: 0.0,
        ..render
    });
    let non_signature_fill = FillFocusRenderState::from_tr909(&RealtimeTr909RenderState {
        phrase_variation: Some(Tr909PhraseVariation::PhraseLift),
        ..render
    });
    assert_eq!(non_fill.gain_at_frame(48_000, frames_per_beat / 2), 1.0);
    assert_eq!(wrong_route.gain_at_frame(48_000, frames_per_beat / 2), 1.0);
    assert_eq!(stopped.gain_at_frame(48_000, frames_per_beat / 2), 1.0);
    assert_eq!(silent_fill.gain_at_frame(48_000, frames_per_beat / 2), 1.0);
    assert_eq!(
        non_signature_fill.gain_at_frame(48_000, frames_per_beat / 2),
        1.0
    );

    let mut previous = focus.gain_at_frame(48_000, 0);
    let mut max_adjacent_delta = 0.0_f32;
    for frame in 1..=frames_to_bar_wrap {
        let current = focus.gain_at_frame(48_000, frame);
        max_adjacent_delta = max_adjacent_delta.max((current - previous).abs());
        previous = current;
    }
    assert!(
        max_adjacent_delta < 0.001,
        "fill-focus envelope stepped too abruptly: {max_adjacent_delta}"
    );
}

#[test]
fn fill_focus_leaves_source_only_sample_exact() {
    let sample_rate = 48_000;
    let frame_count = 24_000;
    let source = SourceAudioCache::from_interleaved_samples(
        "fill-focus-source-only.wav",
        sample_rate,
        1,
        (0..sample_rate * 3)
            .map(|frame| ((frame as f32 * 0.013).sin() * 0.35) + 0.05)
            .collect(),
    )
    .expect("source cache");
    let source_monitor_render = SourceMonitorRenderState {
        mode: riotbox_core::action::SourceMonitorMode::Source,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 3.0,
        ..SourceMonitorRenderState::default()
    };
    let fill = fill_focus_test_plan(
        Tr909RenderMode::Fill,
        source_monitor_render.clone(),
        3.0,
    );
    let control = fill_focus_test_plan(
        Tr909RenderMode::BreakReinforce,
        source_monitor_render,
        3.0,
    );

    let fill_output = render_runtime_mix_realtime_simulation_offline(
        &fill,
        sample_rate,
        2,
        frame_count,
        128,
    );
    let control_output = render_runtime_mix_realtime_simulation_offline(
        &control,
        sample_rate,
        2,
        frame_count,
        128,
    );

    assert_eq!(fill_output, control_output);
}

#[test]
fn fill_focus_uses_the_same_confirmed_source_bar_phase_as_the_fill_recipe() {
    let zero_phase = RealtimeTr909RenderState {
        mode: Tr909RenderMode::Fill,
        routing: Tr909RenderRouting::DrumBusSupport,
        source_support_profile: None,
        source_support_context: None,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
        takeover_profile: None,
        drum_bus_level: 0.80,
        slam_enabled: false,
        slam_intensity: 0.66,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 2.92,
        source_bar_grid_anchor_position_beats: None,
    };
    let source_aligned = RealtimeTr909RenderState {
        position_beats: 5.92,
        source_bar_grid_anchor_position_beats: Some(3.0),
        ..zero_phase
    };
    let zero_focus = FillFocusRenderState::from_tr909(&zero_phase);
    let aligned_focus = FillFocusRenderState::from_tr909(&source_aligned);

    for frame in [0, 1_920, 10_920, 19_920, 25_920] {
        assert_eq!(
            aligned_focus.gain_at_frame(48_000, frame),
            zero_focus.gain_at_frame(48_000, frame),
            "FillFocus drifted from the confirmed source phase at frame {frame}"
        );
    }
}

#[test]
fn fill_focus_envelope_is_sample_exact_across_callback_partitions() {
    let sample_rate = 48_000;
    let frame_count = 48_000 * 2;
    let render = RealtimeTr909RenderState {
        mode: Tr909RenderMode::Fill,
        routing: Tr909RenderRouting::DrumBusSupport,
        source_support_profile: None,
        source_support_context: None,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
        takeover_profile: None,
        drum_bus_level: 0.80,
        slam_enabled: false,
        slam_intensity: 0.66,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
        source_bar_grid_anchor_position_beats: None,
    };
    let mut full_block = vec![1.0_f32; frame_count * 2];
    apply_fill_focus_to_non_tr909_bed(
        &mut full_block,
        sample_rate,
        2,
        FillFocusRenderState::from_tr909(&render),
    );

    let mut partitioned = Vec::with_capacity(full_block.len());
    let beats_per_frame = 120.0_f64 / 60.0 / f64::from(sample_rate);
    for frame_offset in (0..frame_count).step_by(127) {
        let block_frames = (frame_count - frame_offset).min(127);
        let mut block = vec![1.0_f32; block_frames * 2];
        let block_render = RealtimeTr909RenderState {
            position_beats: frame_offset as f64 * beats_per_frame,
            ..render
        };
        apply_fill_focus_to_non_tr909_bed(
            &mut block,
            sample_rate,
            2,
            FillFocusRenderState::from_tr909(&block_render),
        );
        partitioned.extend(block);
    }

    assert_eq!(full_block, partitioned);
}

#[test]
fn fill_focus_ducks_the_non_tr909_riotbox_bed_without_boosting_the_drum_lane() {
    let sample_rate = 48_000;
    let frames_per_beat = 24_000;
    let frame_count = frames_per_beat * 4;
    let source_monitor_render = SourceMonitorRenderState::control_only(
        riotbox_core::action::SourceMonitorMode::Riotbox,
    );
    let fill = fill_focus_test_plan(
        Tr909RenderMode::Fill,
        source_monitor_render.clone(),
        0.0,
    );
    let mut tr909_only = fill.clone();
    tr909_only.mc202_render = Mc202RenderState::default();
    let mut bed_control = fill.clone();
    bed_control.tr909_render = Tr909RenderState::default();

    let fill_output = render_runtime_mix_realtime_simulation_offline(
        &fill,
        sample_rate,
        2,
        frame_count,
        128,
    );
    let tr909_only_output = render_runtime_mix_realtime_simulation_offline(
        &tr909_only,
        sample_rate,
        2,
        frame_count,
        128,
    );
    let bed_control_output = render_runtime_mix_realtime_simulation_offline(
        &bed_control,
        sample_rate,
        2,
        frame_count,
        128,
    );
    let pre_focus_samples = (frames_per_beat * 193 / 100) * 2;
    let middle_last_beat_start = (frames_per_beat * 3 + frames_per_beat / 4) * 2;
    let middle_last_beat_end = (frames_per_beat * 3 + frames_per_beat * 3 / 4) * 2;
    let focused_bed = fill_output
        .iter()
        .zip(&tr909_only_output)
        .map(|(full, drums)| full - drums)
        .collect::<Vec<_>>();
    let early_delta = signal_delta_metrics(
        &focused_bed[..pre_focus_samples],
        &bed_control_output[..pre_focus_samples],
    );
    let focused = signal_metrics(&focused_bed[middle_last_beat_start..middle_last_beat_end]);
    let unfocused =
        signal_metrics(&bed_control_output[middle_last_beat_start..middle_last_beat_end]);

    assert_eq!(early_delta.active_samples, 0);
    assert!(unfocused.rms > 0.01, "control bed was not audible: {unfocused:?}");
    assert!(
        focused.rms < unfocused.rms * 0.14,
        "non-TR909 bed did not make decisive room: focused={focused:?} control={unfocused:?}"
    );
    assert_eq!(signal_metrics(&fill_output).clip_count, 0);
}

#[test]
fn fill_focus_blend_is_callback_size_invariant_and_time_locally_decisive() {
    let sample_rate = 48_000;
    let frames_per_beat = 24_000;
    let frame_count = frames_per_beat * 4;
    let source = SourceAudioCache::from_interleaved_samples(
        "fill-focus-blend.wav",
        sample_rate,
        1,
        vec![0.28; (sample_rate * 3) as usize],
    )
    .expect("source cache");
    let source_monitor_render = SourceMonitorRenderState {
        mode: riotbox_core::action::SourceMonitorMode::Blend,
        source: Some(SourceMonitorAudioSource::from_cache(&source)),
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
        ..SourceMonitorRenderState::default()
    };
    let fill = fill_focus_test_plan(
        Tr909RenderMode::Fill,
        source_monitor_render.clone(),
        0.0,
    );
    let mut tr909_only = fill.clone();
    tr909_only.mc202_render = Mc202RenderState::default();
    tr909_only.source_monitor_render.source = None;
    let mut bed_source_control = fill.clone();
    bed_source_control.tr909_render = Tr909RenderState::default();

    let realtime = render_runtime_mix_realtime_simulation_offline(
        &fill,
        sample_rate,
        2,
        frame_count,
        128,
    );
    let tr909_only_output = render_runtime_mix_realtime_simulation_offline(
        &tr909_only,
        sample_rate,
        2,
        frame_count,
        128,
    );
    let bed_source_control_output = render_runtime_mix_realtime_simulation_offline(
        &bed_source_control,
        sample_rate,
        2,
        frame_count,
        128,
    );
    let mut parity_fill = fill.clone();
    parity_fill.mc202_render = Mc202RenderState::default();
    let parity_full = render_runtime_mix_offline(&parity_fill, sample_rate, 2, frame_count);
    let parity_full_drums = render_runtime_mix_offline(&tr909_only, sample_rate, 2, frame_count);
    let parity_realtime = render_runtime_mix_realtime_simulation_offline(
        &parity_fill,
        sample_rate,
        2,
        frame_count,
        128,
    );
    let parity_full_source = parity_full
        .iter()
        .zip(&parity_full_drums)
        .map(|(full, drums)| full - drums)
        .collect::<Vec<_>>();
    let parity_realtime_source = parity_realtime
        .iter()
        .zip(&tr909_only_output)
        .map(|(full, drums)| full - drums)
        .collect::<Vec<_>>();
    let pre_focus_samples = (frames_per_beat * 193 / 100) * 2;
    let middle_last_beat_start = (frames_per_beat * 3 + frames_per_beat / 4) * 2;
    let middle_last_beat_end = (frames_per_beat * 3 + frames_per_beat * 3 / 4) * 2;
    let focused_bed_and_source = realtime
        .iter()
        .zip(&tr909_only_output)
        .map(|(full, drums)| full - drums)
        .collect::<Vec<_>>();
    let unfocused_counterfactual = tr909_only_output
        .iter()
        .zip(&bed_source_control_output)
        .map(|(drums, bed_and_source)| drums + bed_and_source)
        .collect::<Vec<_>>();
    let early_delta = signal_delta_metrics(
        &focused_bed_and_source[..pre_focus_samples],
        &bed_source_control_output[..pre_focus_samples],
    );
    let focused =
        signal_metrics(&focused_bed_and_source[middle_last_beat_start..middle_last_beat_end]);
    let unfocused =
        signal_metrics(&bed_source_control_output[middle_last_beat_start..middle_last_beat_end]);
    let local_delta = signal_delta_metrics(
        &realtime[middle_last_beat_start..middle_last_beat_end],
        &unfocused_counterfactual[middle_last_beat_start..middle_last_beat_end],
    );

    assert_eq!(parity_full_source, parity_realtime_source);
    assert_eq!(early_delta.active_samples, 0);
    assert!(unfocused.rms > 0.02, "blend control was not audible: {unfocused:?}");
    assert!(
        focused.rms < unfocused.rms * 0.14,
        "blend bed remained masked instead of making room: focused={focused:?} control={unfocused:?}"
    );
    assert!(
        local_delta.rms > unfocused.rms * 0.80,
        "fill focus stayed too close to the counterfactual: {local_delta:?}"
    );
    assert_eq!(signal_metrics(&realtime).clip_count, 0);
}

fn fill_focus_test_plan(
    tr909_mode: Tr909RenderMode,
    source_monitor_render: SourceMonitorRenderState,
    position_beats: f64,
) -> RuntimeMixRenderPlan {
    RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 120.0,
            position_beats,
        },
        tr909_render: Tr909RenderState {
            mode: tr909_mode,
            routing: Tr909RenderRouting::DrumBusSupport,
            pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
            phrase_variation: Some(Tr909PhraseVariation::PhraseDrive),
            drum_bus_level: 0.12,
            slam_enabled: false,
            slam_intensity: 0.0,
            ..Tr909RenderState::default()
        },
        mc202_render: Mc202RenderState {
            mode: Mc202RenderMode::Pressure,
            routing: Mc202RenderRouting::MusicBusBass,
            phrase_shape: Mc202PhraseShape::InstigatorSpike,
            source_phrase_plan: Some(runtime_mix_parity_source_plan()),
            touch: 0.84,
            music_bus_level: 0.34,
            ..Mc202RenderState::default()
        },
        source_monitor_render,
        ..RuntimeMixRenderPlan::default()
    }
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
