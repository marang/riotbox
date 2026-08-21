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
    action::{ActionCommand, CaptureLengthIntent, CommitBoundary, SourceMonitorMode},
    session::{W30HookArticulationProfileState, W30HookSelectionPolicy},
    style::PerformancePresetId,
    transport::CommitBoundaryState,
};

mod w30_live_path_render_modules;

use w30_live_path_render_modules::{
    filter_slam_qualification::qualify_filter_slam_v1,
    pitch_dive_qualification::qualify_pitch_dive_v1,
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
    let qualify_pitch_dive = args.iter().any(|arg| arg == "--qualify-pitch-dive-v1");
    let qualify_filter_slam = args.iter().any(|arg| arg == "--qualify-filter-slam-v1");
    let qualify_gesture_vocabulary = args
        .iter()
        .any(|arg| arg == "--qualify-gesture-vocabulary-v1");
    let prepare_hook_turnaround_review = args
        .iter()
        .any(|arg| arg == "--prepare-hook-turnaround-review");
    let prepare_pitch_dive_review = args.iter().any(|arg| arg == "--prepare-pitch-dive-review");
    let prepare_filter_slam_review = args.iter().any(|arg| arg == "--prepare-filter-slam-review");
    if [
        qualify_hook_turnaround,
        qualify_pitch_dive,
        qualify_filter_slam,
        qualify_gesture_vocabulary,
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count()
        > 1
    {
        return Err("select only one W-30 qualification mode".into());
    }
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
    if qualify_gesture_vocabulary {
        let exact_product_bpm = normal_render.tempo_bpm;
        if !exact_product_bpm.is_finite() || exact_product_bpm <= 0.0 {
            return Err("W-30 qualification has no positive finite product tempo".into());
        }
        qualify_gesture_vocabulary_v1(
            &mut state,
            &normal_render,
            exact_product_bpm,
            &output_dir,
            scene_id,
        )?;
        state.save()?;
        return Ok(());
    }
    if qualify_filter_slam {
        let exact_product_bpm = normal_render.tempo_bpm;
        if !exact_product_bpm.is_finite() || exact_product_bpm <= 0.0 {
            return Err("W-30 qualification has no positive finite product tempo".into());
        }
        qualify_filter_slam_v1(
            &mut state,
            &normal_render,
            exact_product_bpm,
            &output_dir,
            scene_id,
            prepare_filter_slam_review,
            8,
        )?;
        state.save()?;
        return Ok(());
    }
    if qualify_pitch_dive {
        let exact_product_bpm = normal_render.tempo_bpm;
        if !exact_product_bpm.is_finite() || exact_product_bpm <= 0.0 {
            return Err("W-30 qualification has no positive finite product tempo".into());
        }
        qualify_pitch_dive_v1(
            &mut state,
            &normal_render,
            exact_product_bpm,
            &output_dir,
            scene_id,
            prepare_pitch_dive_review,
            8,
        )?;
        state.save()?;
        return Ok(());
    }
    if qualify_hook_turnaround {
        let exact_product_bpm = normal_render.tempo_bpm;
        if !exact_product_bpm.is_finite() || exact_product_bpm <= 0.0 {
            return Err("W-30 qualification has no positive finite product tempo".into());
        }
        qualify_hook_turnaround_v1(
            &mut state,
            &normal_render,
            exact_product_bpm,
            &output_dir,
            scene_id,
            prepare_hook_turnaround_review,
            8,
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
    start_beat: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let frame_count = (5.0 * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize;
    let beat_one = qualification_sample_offset_at_beat_boundary(
        start_beat as f64,
        start_beat as f64 + 1.0,
        bpm,
    )?;
    let beat_three = qualification_sample_offset_at_beat_boundary(
        start_beat as f64,
        start_beat as f64 + 3.0,
        bpm,
    )?;
    let beat_four = qualification_sample_offset_at_beat_boundary(
        start_beat as f64,
        start_beat as f64 + 4.0,
        bpm,
    )?;
    let control_plan = isolated_w30_plan(control_render.clone(), bpm, start_beat as f64);
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
    let requested_at = 400 + start_beat * 10;
    if state.queue_w30_hook_turnaround(requested_at) != Some(QueueControlResult::Enqueued) {
        return Err("W-30 hook turnaround was unavailable".into());
    }
    commit(
        state,
        CommitBoundary::Bar,
        start_beat,
        start_beat / 4 + 1,
        start_beat / 16 + 1,
        scene_id,
        requested_at + 1,
    )?;
    let articulation = state
        .session
        .runtime_state
        .lane_state
        .w30
        .hook_articulation
        .as_ref()
        .ok_or("hook turnaround commit produced no Session articulation")?;
    if articulation.profile != W30HookArticulationProfileState::TurnaroundV1
        || articulation.started_at_beat != start_beat
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
        isolated_w30_plan(state.runtime.w30_preview.clone(), bpm, start_beat as f64);
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
    if candidate.samples[..beat_one] != control.samples[..beat_one] {
        return Err("hook turnaround changed the frozen first relative beat".into());
    }
    if candidate.samples[beat_four..] != control.samples[beat_four..] {
        return Err("hook turnaround did not return sample-exactly on relative beat four".into());
    }
    let reverse_delta = signal_delta_metrics(
        &control.samples[beat_one..beat_three],
        &candidate.samples[beat_one..beat_three],
    );
    let choke_delta = signal_delta_metrics(
        &control.samples[beat_three..beat_four],
        &candidate.samples[beat_three..beat_four],
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
    let missing_source_plan = isolated_w30_plan(missing_source_render, bpm, start_beat as f64);
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
        let review_position_beats = start_beat.saturating_sub(4) as f64;
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
        let four_beats = qualification_sample_offset_at_beat_boundary(
            review_position_beats,
            review_position_beats + 4.0,
            bpm,
        )?;
        let eight_beats = qualification_sample_offset_at_beat_boundary(
            review_position_beats,
            review_position_beats + 8.0,
            bpm,
        )?;
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
        "exact_product_tempo_bpm": bpm,
        "committed_start_beat": start_beat,
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

fn qualify_gesture_vocabulary_v1(
    state: &mut JamAppState,
    initial_control: &W30PreviewRenderState,
    bpm: f32,
    output_dir: &std::path::Path,
    scene_id: Option<riotbox_core::ids::SceneId>,
) -> Result<(), Box<dyn std::error::Error>> {
    let replay_base = state.session.clone();
    let suffix_start = state.session.action_log.actions.len();

    qualify_hook_turnaround_v1(
        state,
        initial_control,
        bpm,
        output_dir,
        scene_id.clone(),
        false,
        8,
    )?;
    let hook_render = state.runtime.w30_preview.clone();
    let hook_reentry = qualify_ordinary_w30_reentry(
        state,
        bpm,
        output_dir,
        scene_id.clone(),
        13,
        "hook_turnaround",
    )?;
    let after_hook = state.runtime.w30_preview.clone();

    qualify_pitch_dive_v1(
        state,
        &after_hook,
        bpm,
        output_dir,
        scene_id.clone(),
        false,
        16,
    )?;
    let pitch_render = state.runtime.w30_preview.clone();
    let pitch_reentry =
        qualify_ordinary_w30_reentry(state, bpm, output_dir, scene_id.clone(), 29, "pitch_dive")?;
    let after_pitch = state.runtime.w30_preview.clone();

    qualify_filter_slam_v1(
        state,
        &after_pitch,
        bpm,
        output_dir,
        scene_id.clone(),
        false,
        32,
    )?;
    let filter_render = state.runtime.w30_preview.clone();
    let filter_reentry =
        qualify_ordinary_w30_reentry(state, bpm, output_dir, scene_id, 41, "filter_slam")?;
    let after_filter = state.runtime.w30_preview.clone();

    let journey_plans = [
        isolated_w30_plan(hook_render, bpm, 8.0),
        isolated_w30_plan(after_hook.clone(), bpm, 13.0),
        isolated_w30_plan(pitch_render, bpm, 16.0),
        isolated_w30_plan(after_pitch.clone(), bpm, 29.0),
        isolated_w30_plan(filter_render, bpm, 32.0),
        isolated_w30_plan(after_filter, bpm, 41.0),
    ];
    let journey_beats = [5.0_f32, 3.0, 13.0, 3.0, 9.0, 4.0];
    let journey_steps = journey_plans
        .iter()
        .zip(journey_beats)
        .map(|(plan, beats)| {
            RuntimeMixRenderSequenceStep::new(
                plan,
                (beats * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize,
            )
        })
        .collect::<Vec<_>>();
    let journey_outputs = render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &journey_steps,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        128,
    );
    let journey_partition_outputs =
        render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
            &journey_steps,
            SAMPLE_RATE,
            CHANNEL_COUNT,
            257,
        );
    if journey_outputs.len() != journey_steps.len()
        || journey_partition_outputs.len() != journey_steps.len()
    {
        return Err("gesture-vocabulary journey omitted a sequence step".into());
    }
    let journey_samples = journey_outputs
        .iter()
        .flat_map(|output| output.samples.iter().copied())
        .collect::<Vec<_>>();
    let journey_partition_samples = journey_partition_outputs
        .iter()
        .flat_map(|output| output.samples.iter().copied())
        .collect::<Vec<_>>();
    if journey_samples != journey_partition_samples {
        return Err("gesture-vocabulary journey changed across callback partitions".into());
    }
    let journey_metrics = signal_metrics(&journey_samples);
    let pre_limiter_clip_count = journey_outputs
        .iter()
        .map(|output| output.limiter.pre.clip_count)
        .sum::<usize>();
    let limited_sample_count = journey_outputs
        .iter()
        .map(|output| output.limiter.limited_sample_count)
        .sum::<usize>();
    let post_limiter_clip_count = journey_outputs
        .iter()
        .map(|output| output.limiter.post.clip_count)
        .sum::<usize>();
    if journey_metrics.active_samples == 0
        || journey_metrics.rms <= 0.001
        || pre_limiter_clip_count != 0
        || limited_sample_count != 0
        || post_limiter_clip_count != 0
    {
        return Err("gesture-vocabulary journey was silent, clipped, or limited".into());
    }
    let journey_path = output_dir.join("08_w30_gesture_vocabulary_journey.wav");
    write_interleaved_pcm16_wav(&journey_path, SAMPLE_RATE, CHANNEL_COUNT, &journey_samples)?;

    let expected_order = [
        ActionCommand::W30HookTurnaround,
        ActionCommand::W30TriggerPad,
        ActionCommand::W30PitchDive,
        ActionCommand::W30TriggerPad,
        ActionCommand::W30FilterSlam,
        ActionCommand::W30TriggerPad,
    ];
    let suffix = &state.session.action_log.actions[suffix_start..];
    let actual_order = suffix
        .iter()
        .map(|action| action.command)
        .collect::<Vec<_>>();
    if actual_order != expected_order {
        return Err(format!("gesture-vocabulary action order diverged: {actual_order:?}").into());
    }

    let serialized = serde_json::to_vec(&state.session)?;
    let restored: riotbox_core::session::SessionFile = serde_json::from_slice(&serialized)?;
    if restored != state.session {
        return Err("gesture-vocabulary Session round trip diverged".into());
    }

    let suffix_action_ids = suffix.iter().map(|action| action.id).collect::<Vec<_>>();
    let mut suffix_log = state.session.action_log.clone();
    suffix_log.actions = suffix.to_vec();
    suffix_log
        .commit_records
        .retain(|record| suffix_action_ids.contains(&record.action_id));
    let replay_plan = riotbox_core::replay::build_committed_replay_plan(&suffix_log)
        .map_err(|error| format!("gesture-vocabulary replay plan failed: {error:?}"))?;
    let mut replayed = replay_base;
    replayed.action_log = suffix_log.clone();
    riotbox_core::replay::apply_replay_plan_to_session(&mut replayed, &replay_plan)
        .map_err(|error| format!("gesture-vocabulary replay execution failed: {error:?}"))?;
    if replayed.runtime_state.lane_state.w30 != state.session.runtime_state.lane_state.w30
        || replayed.runtime_state.macro_state.w30_grit
            != state.session.runtime_state.macro_state.w30_grit
        || replayed.runtime_state.source_monitor != state.session.runtime_state.source_monitor
        || replayed.captures != state.session.captures
    {
        return Err(
            "gesture-vocabulary suffix replay diverged from committed Session state".into(),
        );
    }

    let result = serde_json::json!({
        "schema": "riotbox.w30_gesture_vocabulary_golden_path_qualification_case.v1",
        "exact_product_tempo_bpm": bpm,
        "isolated_contributors": ["w30_preview"],
        "action_order": actual_order.iter().map(|command| command.as_str()).collect::<Vec<_>>(),
        "roles": [
            {"role": "phrase_variation", "mechanism": "w30_hook_turnaround_v1", "committed_start_beat": 8},
            {"role": "destructive_exit", "mechanism": "w30_pitch_dive_v1", "committed_start_beat": 16},
            {"role": "long_build_and_return", "mechanism": "w30_filter_slam_v1", "committed_start_beat": 32}
        ],
        "ordinary_reentries": [hook_reentry, pitch_reentry, filter_reentry],
        "continuous_journey": {
            "path": journey_path.file_name().and_then(|name| name.to_str()),
            "start_beat": 8,
            "end_beat": 45,
            "duration_beats": 37,
            "peak_abs": journey_metrics.peak_abs,
            "rms": journey_metrics.rms,
            "active_samples": journey_metrics.active_samples,
            "pre_limiter_clip_count": pre_limiter_clip_count,
            "limited_sample_count": limited_sample_count,
            "post_limiter_clip_count": post_limiter_clip_count,
            "callback_partition_128_vs_257_sample_exact": true
        },
        "session_round_trip_exact": true,
        "suffix_replay_equivalent": true,
        "capture_lineage_unchanged": true,
        "source_monitor_unchanged": true,
        "final_articulation_cleared": state.session.runtime_state.lane_state.w30.hook_articulation.is_none()
    });
    fs::write(
        output_dir.join("gesture-vocabulary-qualification.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    println!("gesture-vocabulary qualification: {result}");
    Ok(())
}

fn qualify_ordinary_w30_reentry(
    state: &mut JamAppState,
    bpm: f32,
    output_dir: &std::path::Path,
    scene_id: Option<riotbox_core::ids::SceneId>,
    beat_index: u64,
    after_role: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    if state.queue_w30_trigger_pad(400 + beat_index * 10) != Some(QueueControlResult::Enqueued) {
        return Err(format!("ordinary W-30 re-entry after {after_role} was unavailable").into());
    }
    commit(
        state,
        CommitBoundary::Beat,
        beat_index,
        beat_index / 4 + 1,
        beat_index / 16 + 1,
        scene_id,
        401 + beat_index * 10,
    )?;
    if state
        .session
        .runtime_state
        .lane_state
        .w30
        .hook_articulation
        .is_some()
        || state
            .runtime
            .w30_preview
            .pad_playback
            .as_ref()
            .and_then(|pad| pad.hook_articulation)
            .is_some()
    {
        return Err(
            format!("ordinary W-30 re-entry after {after_role} kept old articulation").into(),
        );
    }

    let frame_count = (2.0 * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize;
    let plan = isolated_w30_plan(state.runtime.w30_preview.clone(), bpm, beat_index as f64);
    let output = render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(&plan, frame_count)],
        SAMPLE_RATE,
        CHANNEL_COUNT,
        128,
    )
    .pop()
    .ok_or("ordinary W-30 re-entry produced no output")?;
    if output.limiter.post.active_samples == 0
        || output.limiter.post.rms <= 0.001
        || output.limiter.pre.clip_count != 0
        || output.limiter.limited_sample_count != 0
        || output.limiter.post.clip_count != 0
    {
        return Err(
            format!("ordinary W-30 re-entry after {after_role} was silent or limited").into(),
        );
    }
    write_interleaved_pcm16_wav(
        output_dir.join(format!("07_reentry_after_{after_role}.wav")),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &output.samples,
    )?;
    Ok(serde_json::json!({
        "after_role": after_role,
        "committed_beat": beat_index,
        "articulation_cleared": true,
        "active_samples": output.limiter.post.active_samples,
        "peak_abs": output.limiter.post.peak_abs,
        "rms": output.limiter.post.rms,
        "pre_limiter_clip_count": output.limiter.pre.clip_count,
        "limited_sample_count": output.limiter.limited_sample_count,
        "post_limiter_clip_count": output.limiter.post.clip_count
    }))
}

/// Return the interleaved-sample offset of the callback frame at a transport beat boundary.
///
/// The runtime advances its `f64` transport position once per rendered frame and snaps values
/// already within `1e-9` beat of an integer articulation boundary. Repeating that operation here
/// avoids multiplying a separately rounded one-beat length, which drifts by several frames at
/// non-integer product tempos.
fn qualification_sample_offset_at_beat_boundary(
    start_position_beats: f64,
    target_position_beats: f64,
    bpm: f32,
) -> Result<usize, Box<dyn std::error::Error>> {
    if !start_position_beats.is_finite()
        || !target_position_beats.is_finite()
        || target_position_beats < start_position_beats
        || !bpm.is_finite()
        || bpm <= 0.0
    {
        return Err("invalid hook-turnaround qualification beat boundary".into());
    }

    const BOUNDARY_SNAP_BEATS: f64 = 1.0e-9;
    let beats_per_frame = f64::from(bpm) / 60.0 / f64::from(SAMPLE_RATE);
    let maximum_frames =
        ((target_position_beats - start_position_beats) / beats_per_frame).ceil() as usize + 2;
    let mut position_beats = start_position_beats;
    for frame in 0..=maximum_frames {
        if position_beats >= target_position_beats
            || (position_beats - target_position_beats).abs() <= BOUNDARY_SNAP_BEATS
        {
            return Ok(frame.saturating_mul(usize::from(CHANNEL_COUNT)));
        }
        position_beats += beats_per_frame;
    }

    Err("hook-turnaround qualification beat boundary was unreachable".into())
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
