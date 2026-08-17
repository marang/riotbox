use std::{env, fs, path::PathBuf};

use riotbox_app::jam_app::{JamAppState, QueueControlResult};
use riotbox_audio::{
    runtime::{
        AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan, RuntimeMixRenderSequenceStep,
        SourceMonitorRenderState,
        render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report,
        render_runtime_mix_realtime_simulation_offline, signal_delta_metrics, signal_metrics,
    },
    source_audio::{SourceAudioCache, write_interleaved_pcm16_wav},
    w30::W30PreviewRenderState,
};
use riotbox_core::{
    action::{CaptureLengthIntent, CommitBoundary, SourceMonitorMode},
    session::W30HookSelectionPolicy,
    style::PerformancePresetId,
    transport::CommitBoundaryState,
};

const SAMPLE_RATE: u32 = 48_000;
const CHANNEL_COUNT: u16 = 2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let source_path = required_path(&args, "--source")?;
    let output_dir = required_path(&args, "--output")?;
    let bpm = required_value(&args, "--bpm")?.parse::<f32>()?;
    let downbeat_seconds = optional_value(&args, "--downbeat-seconds")
        .map(str::parse::<f32>)
        .transpose()?;
    let hook_policy = parse_hook_policy(
        optional_value(&args, "--hook-policy").unwrap_or("transport_boundary_v1"),
    )?;
    let include_resample = args.iter().any(|arg| arg == "--include-resample");
    let explore_hook_turnaround = args.iter().any(|arg| arg == "--explore-hook-turnaround-v1");
    let qualify_hook_turnaround = args.iter().any(|arg| arg == "--qualify-hook-turnaround-v1");
    let prepare_hook_turnaround_review = args
        .iter()
        .any(|arg| arg == "--prepare-hook-turnaround-review");
    fs::create_dir_all(&output_dir)?;

    let session_path = output_dir.join("session.json");
    let graph_path = output_dir.join("source-graph.json");
    let mut state = JamAppState::analyze_source_file_to_json_with_source_timing_confirmation(
        &source_path,
        &session_path,
        Some(graph_path),
        "python/sidecar/json_stdio_sidecar.py",
        19,
        Some(bpm),
        downbeat_seconds,
    )?;
    state.set_transport_playing(true);
    let scene_id = state.runtime.transport.current_scene.clone();
    if state.queue_performance_preset(PerformancePresetId::FeralBreakAlphaV2, 90)
        != QueueControlResult::Enqueued
    {
        return Err("FeralBreakAlphaV2 preset activation was unavailable".into());
    }
    commit(
        &mut state,
        CommitBoundary::Immediate,
        0,
        1,
        1,
        scene_id.clone(),
        95,
    )?;
    // This diagnostic override compares the two already-frozen policies through the same
    // product capture/runtime path. The shipped preset default changes only after a winner.
    state.session.runtime_state.style.w30_hook_selection_policy = hook_policy;
    state.queue_capture_length_intent(CaptureLengthIntent::OneBar, 96);
    commit(
        &mut state,
        CommitBoundary::Immediate,
        0,
        1,
        1,
        scene_id.clone(),
        97,
    )?;

    state.queue_capture_bar(100);
    commit(
        &mut state,
        CommitBoundary::Bar,
        0,
        1,
        1,
        scene_id.clone(),
        200,
    )?;
    let capture = state
        .session
        .captures
        .last()
        .ok_or("capture commit produced no CaptureRef")?;
    let source_window = capture
        .source_window
        .as_ref()
        .ok_or("capture commit produced no source window")?;
    println!(
        "hook selection: policy={hook_policy:?} range={:.6}-{:.6}s decision={:?}",
        source_window.start_seconds, source_window.end_seconds, source_window.hook_selection
    );
    if !state.queue_promote_last_capture(210) {
        return Err("capture promotion was unavailable".into());
    }
    commit(
        &mut state,
        CommitBoundary::Bar,
        5,
        2,
        1,
        scene_id.clone(),
        300,
    )?;
    if state.queue_w30_trigger_pad(310).is_none() {
        return Err("W-30 trigger was unavailable".into());
    }
    commit(
        &mut state,
        CommitBoundary::Beat,
        6,
        2,
        1,
        scene_id.clone(),
        400,
    )?;

    print_w30_render_summary("normal", &state.runtime.w30_preview);
    let normal_render = state.runtime.w30_preview.clone();
    if qualify_hook_turnaround {
        qualify_hook_turnaround_v1(
            &mut state,
            &normal_render,
            bpm,
            &output_dir,
            scene_id,
            prepare_hook_turnaround_review,
        )?;
        state.save()?;
        return Ok(());
    }
    let normal = render_state(&state, bpm);
    if state.queue_w30_apply_damage_profile(410).is_none() {
        return Err("W-30 damage gesture was unavailable".into());
    }
    commit(
        &mut state,
        CommitBoundary::Bar,
        9,
        3,
        1,
        scene_id.clone(),
        500,
    )?;
    print_w30_render_summary("damaged", &state.runtime.w30_preview);
    let damaged = render_state(&state, bpm);

    write_interleaved_pcm16_wav(
        output_dir.join("01_w30_live_hook.wav"),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &normal,
    )?;
    write_interleaved_pcm16_wav(
        output_dir.join("02_w30_live_hook_pitch_damage.wav"),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &damaged,
    )?;
    if explore_hook_turnaround {
        let (control, candidate) = render_hook_turnaround_v1(&normal_render, bpm);
        write_interleaved_pcm16_wav(
            output_dir.join("05_w30_hook_turnaround_control.wav"),
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &control,
        )?;
        write_interleaved_pcm16_wav(
            output_dir.join("06_w30_hook_turnaround_candidate_v1.wav"),
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &candidate,
        )?;

        let control_metrics = signal_metrics(&control);
        let candidate_metrics = signal_metrics(&candidate);
        let delta = signal_delta_metrics(&control, &candidate);
        println!("hook-turnaround control: {control_metrics:?}");
        println!("hook-turnaround candidate-v1: {candidate_metrics:?}");
        println!("hook-turnaround delta: {delta:?}");
        if control_metrics.rms <= 0.001
            || candidate_metrics.rms <= 0.001
            || candidate_metrics.peak_abs >= 0.99
            || delta.rms <= 0.001
        {
            return Err(
                "W-30 hook-turnaround exploration was silent, clipped, or collapsed".into(),
            );
        }
    }
    let resample_outputs = if include_resample {
        if state.queue_w30_internal_resample(510).is_none() {
            return Err("W-30 internal resample was unavailable".into());
        }
        commit(&mut state, CommitBoundary::Phrase, 16, 4, 2, scene_id, 600)?;
        println!(
            "resample tap: mode={:?} routing={:?} availability={:?} source={:?} lineage={} generation={}",
            state.runtime.w30_resample_tap.mode,
            state.runtime.w30_resample_tap.routing,
            state.runtime.w30_resample_tap.availability,
            state.runtime.w30_resample_tap.source_capture_id,
            state.runtime.w30_resample_tap.lineage_capture_count,
            state.runtime.w30_resample_tap.generation_depth,
        );
        let tap = render_resample_state(&state, bpm);
        let mut unavailable_state = state.runtime.w30_resample_tap.clone();
        unavailable_state.source_audio = None;
        unavailable_state.availability =
            riotbox_audio::w30::W30ResampleTapAvailability::SourceAudioUnavailable;
        unavailable_state.routing = riotbox_audio::w30::W30ResampleTapRouting::Silent;
        let unavailable = render_resample_tap(&state, unavailable_state, bpm);
        write_interleaved_pcm16_wav(
            output_dir.join("03_w30_source_backed_resample_tap.wav"),
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &tap,
        )?;
        write_interleaved_pcm16_wav(
            output_dir.join("04_w30_missing_source_silence.wav"),
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &unavailable,
        )?;
        Some((tap, unavailable))
    } else {
        None
    };
    let source = SourceAudioCache::load_pcm_wav(&source_path)?;
    write_interleaved_pcm16_wav(
        output_dir.join("00_source.wav"),
        source.sample_rate,
        source.channel_count,
        source.interleaved_samples(),
    )?;
    state.save()?;

    let normal_metrics = signal_metrics(&normal);
    let damaged_metrics = signal_metrics(&damaged);
    let delta = signal_delta_metrics(&normal, &damaged);
    println!("normal: {normal_metrics:?}");
    println!("damaged: {damaged_metrics:?}");
    println!("gesture delta: {delta:?}");
    if normal_metrics.rms <= 0.001 || damaged_metrics.rms <= 0.001 || delta.rms <= 0.001 {
        return Err("live W-30 render was silent or gesture-collapsed".into());
    }
    if let Some((tap, unavailable)) = resample_outputs {
        let tap_metrics = signal_metrics(&tap);
        let unavailable_metrics = signal_metrics(&unavailable);
        println!("source-backed resample tap: {tap_metrics:?}");
        println!("missing-source control: {unavailable_metrics:?}");
        if tap_metrics.rms <= 0.001 || tap_metrics.peak_abs >= 0.99 {
            return Err("source-backed resample tap was silent or clipped".into());
        }
        if unavailable_metrics.active_samples != 0 {
            return Err("missing-source resample control emitted fallback audio".into());
        }
    }
    Ok(())
}

fn qualify_hook_turnaround_v1(
    state: &mut JamAppState,
    control_render: &W30PreviewRenderState,
    bpm: f32,
    output_dir: &std::path::Path,
    scene_id: Option<riotbox_core::ids::SceneId>,
    prepare_review: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    const START_BEAT: u64 = 8;
    let frame_count = (5.0 * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize;
    let samples_per_beat = ((60.0 / bpm * SAMPLE_RATE as f32).round() as usize)
        .saturating_mul(usize::from(CHANNEL_COUNT));
    let control_plan = isolated_w30_plan(control_render.clone(), bpm, START_BEAT as f64);
    let control = render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(
            &control_plan,
            frame_count,
        )],
        SAMPLE_RATE,
        CHANNEL_COUNT,
        128,
    )
    .pop()
    .ok_or("control render produced no output")?;

    let captures_before = state.session.captures.clone();
    let grit_before = state.session.runtime_state.macro_state.w30_grit;
    let source_monitor_before = state.session.runtime_state.source_monitor.clone();
    if state.queue_w30_hook_turnaround(410) != Some(QueueControlResult::Enqueued) {
        return Err("W-30 hook turnaround was unavailable".into());
    }
    commit(state, CommitBoundary::Bar, START_BEAT, 3, 1, scene_id, 500)?;
    let articulation = state
        .session
        .runtime_state
        .lane_state
        .w30
        .hook_articulation
        .as_ref()
        .ok_or("hook turnaround commit produced no Session articulation")?;
    if articulation.started_at_beat != START_BEAT
        || articulation.capture_id.to_string()
            != state
                .runtime
                .w30_preview
                .capture_id
                .as_deref()
                .unwrap_or_default()
    {
        return Err("hook turnaround Session target or start beat diverged".into());
    }
    if state.session.captures != captures_before
        || state.session.runtime_state.macro_state.w30_grit != grit_before
        || state.session.runtime_state.source_monitor != source_monitor_before
    {
        return Err("hook turnaround changed frozen capture, grit, or Source Monitor state".into());
    }

    let candidate_plan =
        isolated_w30_plan(state.runtime.w30_preview.clone(), bpm, START_BEAT as f64);
    let candidate = render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(
            &candidate_plan,
            frame_count,
        )],
        SAMPLE_RATE,
        CHANNEL_COUNT,
        128,
    )
    .pop()
    .ok_or("candidate render produced no output")?;
    let partition_control =
        render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
            &[RuntimeMixRenderSequenceStep::new(
                &candidate_plan,
                frame_count,
            )],
            SAMPLE_RATE,
            CHANNEL_COUNT,
            257,
        )
        .pop()
        .ok_or("partition-control render produced no output")?;

    if candidate.samples != partition_control.samples {
        return Err("hook turnaround changed across callback partitions".into());
    }
    if candidate.samples[..samples_per_beat] != control.samples[..samples_per_beat] {
        return Err("hook turnaround changed the frozen first relative beat".into());
    }
    if candidate.samples[4 * samples_per_beat..] != control.samples[4 * samples_per_beat..] {
        return Err("hook turnaround did not return sample-exactly on relative beat four".into());
    }
    let reverse_delta = signal_delta_metrics(
        &control.samples[samples_per_beat..3 * samples_per_beat],
        &candidate.samples[samples_per_beat..3 * samples_per_beat],
    );
    let choke_delta = signal_delta_metrics(
        &control.samples[3 * samples_per_beat..4 * samples_per_beat],
        &candidate.samples[3 * samples_per_beat..4 * samples_per_beat],
    );
    if reverse_delta.rms <= 0.001 || choke_delta.rms <= 0.001 {
        return Err("hook turnaround reverse or choke window collapsed into control".into());
    }
    for (label, output) in [("control", &control), ("candidate", &candidate)] {
        if output.limiter.pre.clip_count != 0
            || output.limiter.limited_sample_count != 0
            || output.limiter.post.clip_count != 0
        {
            return Err(format!("{label} render clipped or invoked the limiter").into());
        }
    }

    let mut missing_source_render = state.runtime.w30_preview.clone();
    missing_source_render.pad_playback = None;
    missing_source_render.source_window_preview = None;
    missing_source_render.routing = riotbox_audio::w30::W30PreviewRenderRouting::Silent;
    let missing_source_plan = isolated_w30_plan(missing_source_render, bpm, START_BEAT as f64);
    let missing_source = render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(
            &missing_source_plan,
            frame_count,
        )],
        SAMPLE_RATE,
        CHANNEL_COUNT,
        128,
    )
    .pop()
    .ok_or("missing-source render produced no output")?;
    if missing_source.limiter.post.active_samples != 0 {
        return Err("missing source emitted fallback audio".into());
    }

    write_interleaved_pcm16_wav(
        output_dir.join("05_w30_hook_turnaround_control.wav"),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &control.samples,
    )?;
    write_interleaved_pcm16_wav(
        output_dir.join("06_w30_hook_turnaround_candidate_v1.wav"),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &candidate.samples,
    )?;
    let review = if prepare_review {
        let review_position_beats = START_BEAT.saturating_sub(4) as f64;
        let review_frame_count = (12.0 * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize;
        let review_control_plan =
            isolated_w30_plan(control_render.clone(), bpm, review_position_beats);
        let review_candidate_plan = isolated_w30_plan(
            state.runtime.w30_preview.clone(),
            bpm,
            review_position_beats,
        );
        let review_control =
            render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
                &[RuntimeMixRenderSequenceStep::new(
                    &review_control_plan,
                    review_frame_count,
                )],
                SAMPLE_RATE,
                CHANNEL_COUNT,
                128,
            )
            .pop()
            .ok_or("review control render produced no output")?;
        let review_candidate =
            render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
                &[RuntimeMixRenderSequenceStep::new(
                    &review_candidate_plan,
                    review_frame_count,
                )],
                SAMPLE_RATE,
                CHANNEL_COUNT,
                128,
            )
            .pop()
            .ok_or("review candidate render produced no output")?;
        if review_control.limiter.pre.clip_count != 0
            || review_control.limiter.limited_sample_count != 0
            || review_control.limiter.post.clip_count != 0
            || review_candidate.limiter.pre.clip_count != 0
            || review_candidate.limiter.limited_sample_count != 0
            || review_candidate.limiter.post.clip_count != 0
        {
            return Err("formal review artifact clipped or invoked the limiter".into());
        }
        let four_beats = 4 * samples_per_beat;
        let eight_beats = 8 * samples_per_beat;
        if review_candidate.samples[..four_beats] != review_control.samples[..four_beats]
            || review_candidate.samples[eight_beats..] != review_control.samples[eight_beats..]
        {
            return Err("formal review artifact changed its pre-roll or ordinary return".into());
        }
        write_interleaved_pcm16_wav(
            output_dir.join("07_review_A_control.wav"),
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &review_control.samples,
        )?;
        write_interleaved_pcm16_wav(
            output_dir.join("08_review_B_candidate_v1.wav"),
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &review_candidate.samples,
        )?;
        Some(serde_json::json!({
            "duration_beats": 12,
            "pre_roll_beats": 4,
            "articulation_and_anchor_beats": 4,
            "ordinary_return_beats": 4,
            "control_peak_abs": review_control.limiter.post.peak_abs,
            "candidate_peak_abs": review_candidate.limiter.post.peak_abs,
            "control_rms": review_control.limiter.post.rms,
            "candidate_rms": review_candidate.limiter.post.rms,
            "pre_roll_sample_exact": true,
            "ordinary_return_sample_exact": true,
            "pre_limiter_clip_count": 0,
            "limited_sample_count": 0,
            "post_limiter_clip_count": 0
        }))
    } else {
        None
    };
    let result = serde_json::json!({
        "schema": "riotbox.w30_hook_turnaround_qualification_case.v1",
        "mechanism": "w30_hook_turnaround_v1",
        "committed_start_beat": START_BEAT,
        "isolated_contributors": ["w30_preview"],
        "control": {
            "peak_abs": control.limiter.post.peak_abs,
            "rms": control.limiter.post.rms,
            "pre_limiter_clip_count": control.limiter.pre.clip_count,
            "limited_sample_count": control.limiter.limited_sample_count,
            "post_limiter_clip_count": control.limiter.post.clip_count
        },
        "candidate": {
            "peak_abs": candidate.limiter.post.peak_abs,
            "rms": candidate.limiter.post.rms,
            "pre_limiter_clip_count": candidate.limiter.pre.clip_count,
            "limited_sample_count": candidate.limiter.limited_sample_count,
            "post_limiter_clip_count": candidate.limiter.post.clip_count
        },
        "reverse_delta_rms": reverse_delta.rms,
        "choke_delta_rms": choke_delta.rms,
        "first_relative_beat_sample_exact": true,
        "return_from_relative_beat_four_sample_exact": true,
        "callback_partition_128_vs_257_sample_exact": true,
        "capture_lineage_unchanged": true,
        "grit_unchanged": true,
        "source_monitor_unchanged": true,
        "missing_source_active_samples": missing_source.limiter.post.active_samples,
        "session_round_trip_and_replay_equivalence": "source_blind_automated_tests"
        ,"formal_review_artifact": review
    });
    fs::write(
        output_dir.join("hook-turnaround-qualification.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    println!("hook-turnaround qualification: {result}");
    Ok(())
}

fn isolated_w30_plan(
    render: W30PreviewRenderState,
    bpm: f32,
    position_beats: f64,
) -> RuntimeMixRenderPlan {
    RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: bpm,
            position_beats,
        },
        tr909_render: Default::default(),
        mc202_render: Default::default(),
        w30_preview_render: render,
        w30_resample_tap: Default::default(),
        source_monitor_render: SourceMonitorRenderState::control_only(SourceMonitorMode::Riotbox),
    }
}

fn render_state(state: &JamAppState, bpm: f32) -> Vec<f32> {
    let bars = 8.0_f32;
    let frame_count = (bars * 4.0 * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize;
    let plan = RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: bpm,
            position_beats: 0.0,
        },
        tr909_render: Default::default(),
        mc202_render: Default::default(),
        w30_preview_render: state.runtime.w30_preview.clone(),
        w30_resample_tap: Default::default(),
        source_monitor_render: state.source_monitor_render_state(),
    };
    render_runtime_mix_realtime_simulation_offline(
        &plan,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        frame_count,
        128,
    )
}

/// Development-only sampler articulation for RIOTBOX-1440.
///
/// The control and candidate use identical segment boundaries so reset behavior cannot explain
/// their difference. The candidate establishes one full source hook, anchors the next downbeat,
/// turns the source around for two beats, chokes one beat of forward source attacks, and then
/// returns to the unmodified hook. Only the W-30 lane is audible.
fn render_hook_turnaround_v1(render: &W30PreviewRenderState, bpm: f32) -> (Vec<f32>, Vec<f32>) {
    let segments = [
        (0.0_f64, 4.0_f32),
        (4.0, 1.0),
        (5.0, 2.0),
        (7.0, 1.0),
        (8.0, 4.0),
    ];
    let mut control = Vec::new();
    let mut candidate = Vec::new();

    for (index, (position_beats, duration_beats)) in segments.into_iter().enumerate() {
        control.extend(render_w30_only_segment(
            render,
            bpm,
            position_beats,
            duration_beats,
        ));

        let mut articulated = render.clone();
        if let Some(pad) = articulated.pad_playback.as_mut() {
            match index {
                2 => {
                    pad.reverse = true;
                    pad.gate_step_fraction = 0.68;
                }
                3 => {
                    pad.reverse = false;
                    pad.gate_step_fraction = 0.34;
                }
                _ => {}
            }
        }
        candidate.extend(render_w30_only_segment(
            &articulated,
            bpm,
            position_beats,
            duration_beats,
        ));
    }

    (control, candidate)
}

fn render_w30_only_segment(
    render: &W30PreviewRenderState,
    bpm: f32,
    position_beats: f64,
    duration_beats: f32,
) -> Vec<f32> {
    let frame_count = (duration_beats * 60.0 / bpm * SAMPLE_RATE as f32)
        .round()
        .max(1.0) as usize;
    let plan = RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: bpm,
            position_beats,
        },
        tr909_render: Default::default(),
        mc202_render: Default::default(),
        w30_preview_render: render.clone(),
        w30_resample_tap: Default::default(),
        source_monitor_render: SourceMonitorRenderState::control_only(SourceMonitorMode::Riotbox),
    };
    render_runtime_mix_realtime_simulation_offline(
        &plan,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        frame_count,
        128,
    )
}

fn render_resample_state(state: &JamAppState, bpm: f32) -> Vec<f32> {
    render_resample_tap(state, state.runtime.w30_resample_tap.clone(), bpm)
}

fn render_resample_tap(
    state: &JamAppState,
    tap: riotbox_audio::w30::W30ResampleTapState,
    bpm: f32,
) -> Vec<f32> {
    let bars = 4.0_f32;
    let frame_count = (bars * 4.0 * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize;
    let plan = RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: bpm,
            position_beats: 0.0,
        },
        tr909_render: Default::default(),
        mc202_render: Default::default(),
        w30_preview_render: Default::default(),
        w30_resample_tap: tap,
        source_monitor_render: state.source_monitor_render_state(),
    };
    render_runtime_mix_realtime_simulation_offline(
        &plan,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        frame_count,
        128,
    )
}

fn print_w30_render_summary(label: &str, render: &riotbox_audio::w30::W30PreviewRenderState) {
    println!(
        "{label} render: mode={:?} routing={:?} bus={} running={} tempo={} capture={:?} pad={:?}",
        render.mode,
        render.routing,
        render.music_bus_level,
        render.is_transport_running,
        render.tempo_bpm,
        render.capture_id,
        render.pad_playback.as_ref().map(|pad| (
            pad.sample_count,
            pad.playback_frame_count,
            pad.playback_rate,
            pad.reverse,
        )),
    );
}

fn commit(
    state: &mut JamAppState,
    kind: CommitBoundary,
    beat_index: u64,
    bar_index: u64,
    phrase_index: u64,
    scene_id: Option<riotbox_core::ids::SceneId>,
    timestamp: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind,
            beat_index,
            bar_index,
            phrase_index,
            scene_id,
        },
        timestamp,
    );
    if committed.len() != 1 {
        return Err(format!("expected one {kind:?} commit, got {}", committed.len()).into());
    }
    Ok(())
}

fn required_path(args: &[String], flag: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(required_value(args, flag)?))
}

fn required_value<'a>(
    args: &'a [String],
    flag: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    let index = args
        .iter()
        .position(|arg| arg == flag)
        .ok_or(flag.to_string())?;
    args.get(index + 1)
        .map(String::as_str)
        .ok_or_else(|| format!("missing value for {flag}").into())
}

fn optional_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}

fn parse_hook_policy(value: &str) -> Result<W30HookSelectionPolicy, Box<dyn std::error::Error>> {
    match value {
        "transport_boundary_v1" => Ok(W30HookSelectionPolicy::TransportBoundaryV1),
        "attack_body_contrast_v1" => Ok(W30HookSelectionPolicy::AttackBodyContrastV1),
        "repetition_salience_v1" => Ok(W30HookSelectionPolicy::RepetitionSalienceV1),
        _ => Err(format!("unsupported --hook-policy {value}").into()),
    }
}
