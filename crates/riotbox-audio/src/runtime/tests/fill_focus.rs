use crate::mc202::{Mc202PhraseShape, Mc202RenderMode, Mc202RenderRouting, Mc202RenderState};
use crate::runtime::fill_focus::{FillFocusRenderState, apply_fill_focus_to_non_tr909_bed};
use crate::runtime::shared_transport_tr909::RealtimeTr909RenderState;
use crate::runtime::source_monitor::{SourceMonitorAudioSource, SourceMonitorRenderState};
use crate::runtime::tests::mix_plan_fixtures::runtime_mix_parity_source_plan;
use crate::runtime::{
    AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan, render_runtime_mix_offline,
    render_runtime_mix_realtime_simulation_offline, signal_delta_metrics, signal_metrics,
};
use crate::source_audio::SourceAudioCache;
use crate::tr909::{
    Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
    Tr909RenderState,
};

#[test]
fn fill_focus_is_typed_to_fill_support_and_the_drum_owned_full_bar_phase() {
    let render = RealtimeTr909RenderState {
        mode: Tr909RenderMode::Fill,
        routing: Tr909RenderRouting::DrumBusSupport,
        source_support_profile: None,
        source_support_context: None,
        pattern_adoption: Some(Tr909PatternAdoption::MainlineDrive),
        phrase_variation: Some(Tr909PhraseVariation::PhraseDriveHardCut),
        takeover_profile: None,
        drum_bus_level: 0.80,
        slam_enabled: false,
        slam_intensity: 0.66,
        is_transport_running: true,
        tempo_bpm: 120.0,
        position_beats: 0.0,
        source_bar_grid_anchor_position_beats: None,
    };
    let focus = FillFocusRenderState::from_tr909(&render);
    let frames_per_beat = 24_000;
    let frames_to_bar_wrap = frames_per_beat * 4;

    assert_eq!(focus.gain_at_frame(48_000, 0), 1.0);
    assert_eq!(focus.gain_at_frame(48_000, 1_680), 0.0);
    assert_eq!(focus.gain_at_frame(48_000, frames_per_beat / 2), 0.0);
    let late_stomp_frame = frames_per_beat * 15 / 4;
    let release_frame = frames_per_beat * 395 / 100;
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
    let fill = fill_focus_test_plan(Tr909RenderMode::Fill, source_monitor_render.clone(), 3.0);
    let control = fill_focus_test_plan(Tr909RenderMode::BreakReinforce, source_monitor_render, 3.0);

    let fill_output =
        render_runtime_mix_realtime_simulation_offline(&fill, sample_rate, 2, frame_count, 128);
    let control_output =
        render_runtime_mix_realtime_simulation_offline(&control, sample_rate, 2, frame_count, 128);

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
        phrase_variation: Some(Tr909PhraseVariation::PhraseDriveHardCut),
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
        phrase_variation: Some(Tr909PhraseVariation::PhraseDriveHardCut),
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
    let source_monitor_render =
        SourceMonitorRenderState::control_only(riotbox_core::action::SourceMonitorMode::Riotbox);
    let fill = fill_focus_test_plan(Tr909RenderMode::Fill, source_monitor_render.clone(), 0.0);
    let mut tr909_only = fill.clone();
    tr909_only.mc202_render = Mc202RenderState::default();
    let mut bed_control = fill.clone();
    bed_control.tr909_render = Tr909RenderState::default();

    let fill_output =
        render_runtime_mix_realtime_simulation_offline(&fill, sample_rate, 2, frame_count, 128);
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
    let focused_first_half_start = (frames_per_beat / 4) * 2;
    let focused_first_half_end = (frames_per_beat * 7 / 4) * 2;
    let middle_last_beat_start = (frames_per_beat * 3 + frames_per_beat / 4) * 2;
    let middle_last_beat_end = (frames_per_beat * 3 + frames_per_beat * 3 / 4) * 2;
    let focused_bed = fill_output
        .iter()
        .zip(&tr909_only_output)
        .map(|(full, drums)| full - drums)
        .collect::<Vec<_>>();
    let focused_first_half =
        signal_metrics(&focused_bed[focused_first_half_start..focused_first_half_end]);
    let unfocused_first_half =
        signal_metrics(&bed_control_output[focused_first_half_start..focused_first_half_end]);
    let focused = signal_metrics(&focused_bed[middle_last_beat_start..middle_last_beat_end]);
    let unfocused =
        signal_metrics(&bed_control_output[middle_last_beat_start..middle_last_beat_end]);

    assert!(
        focused_first_half.rms < unfocused_first_half.rms * 0.14,
        "non-TR909 bed was not removed from the first half: focused={focused_first_half:?} control={unfocused_first_half:?}"
    );
    assert!(
        unfocused.rms > 0.01,
        "control bed was not audible: {unfocused:?}"
    );
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
    let fill = fill_focus_test_plan(Tr909RenderMode::Fill, source_monitor_render.clone(), 0.0);
    let mut tr909_only = fill.clone();
    tr909_only.mc202_render = Mc202RenderState::default();
    tr909_only.source_monitor_render.source = None;
    let mut bed_source_control = fill.clone();
    bed_source_control.tr909_render = Tr909RenderState::default();

    let realtime =
        render_runtime_mix_realtime_simulation_offline(&fill, sample_rate, 2, frame_count, 128);
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
    let focused_first_half_start = (frames_per_beat / 4) * 2;
    let focused_first_half_end = (frames_per_beat * 7 / 4) * 2;
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
    let focused_first_half =
        signal_metrics(&focused_bed_and_source[focused_first_half_start..focused_first_half_end]);
    let unfocused_first_half = signal_metrics(
        &bed_source_control_output[focused_first_half_start..focused_first_half_end],
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
    assert!(
        focused_first_half.rms < unfocused_first_half.rms * 0.14,
        "blend bed was not removed from the first half: focused={focused_first_half:?} control={unfocused_first_half:?}"
    );
    assert!(
        unfocused.rms > 0.02,
        "blend control was not audible: {unfocused:?}"
    );
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
            phrase_variation: Some(Tr909PhraseVariation::PhraseDriveHardCut),
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
