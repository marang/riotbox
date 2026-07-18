use std::{error::Error, path::Path};

use riotbox_audio::{
    listening_manifest::{LISTENING_MANIFEST_SCHEMA_VERSION, write_manifest_json},
    runtime::{
        MasterBusLimiterReport, OfflineAudioMetrics, signal_delta_metrics, signal_metrics,
        signal_metrics_with_grid,
    },
    source_audio::{SourceAudioCache, write_interleaved_pcm16_wav},
};
use riotbox_core::action::{ActionCommand, CommitBoundary};
use serde_json::{Value, json};

use crate::{
    alpha_manifest,
    model::{
        CHANNEL_COUNT, MAX_EXACT_MIX_LIMITED_SAMPLE_COUNT, MAX_SOURCE_MONITOR_SILENCE_RATIO,
        MIN_ISOLATED_TR909_REGRESSION_RMS, MIN_MIX_RMS, MIN_MONITOR_DELTA_RMS, MONITOR_REVIEW_BARS,
        PreparedLivePath, RenderedLivePath, SAMPLE_RATE,
    },
};

#[derive(Clone, Copy)]
struct GestureQaPolicy {
    window_beats: u32,
    min_delta_rms: f32,
    min_delta_peak: f32,
    min_relative_delta_rms: f32,
    min_relevant_10ms_activity_ratio: Option<f32>,
    max_waveform_correlation: Option<f32>,
}

const PERCEPTUAL_WINDOW_MS: usize = 10;
const PERCEPTUAL_DELTA_RELATIVE_FLOOR: f32 = 0.10;
const PERCEPTUAL_DELTA_ABSOLUTE_FLOOR: f32 = 1.0e-5;
const FILL_EXIT_BOUNDARY_WINDOW_MS: usize = 10;
const MAX_FILL_EXIT_BOUNDARY_STEP: f32 = 0.20;
const MAX_FILL_EXIT_BOUNDARY_TO_LOCAL_P99_RATIO: f32 = 4.0;
const MAX_FILL_EXIT_BOUNDARY_TO_ATTACK_RMS_RATIO: f32 = 4.0;
const LIVE_SEQUENCE_ARTIFACT_PATH: &str = "gestures/06_live_sequence.wav";
const TR909_FILL_PRIMITIVE_SCHEMA: &str = "riotbox.tr909_fill_recipe.v1";

#[derive(Clone, Copy, Debug)]
struct SequenceBoundaryMetrics {
    boundary_step: f32,
    local_adjacent_step_p99: f32,
    boundary_to_local_p99_ratio: f32,
    post_boundary_attack_rms: f32,
    boundary_to_attack_rms_ratio: f32,
    window_frames: usize,
}

pub fn write_pack(
    prepared: PreparedLivePath,
    rendered: RenderedLivePath,
    source_path: &Path,
    output_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let bpm = prepared.source_timing.bpm;
    // Source Graph is serialized directly from typed structs, while `json!`
    // first widens `f32` values inside `Value`. Preserve the Source Graph's
    // canonical f32 JSON number so the cross-artifact timing identity remains
    // exact instead of becoming a serializer-representation mismatch.
    let manifest_bpm = canonical_f32_json_number(bpm)?;
    let manifest_cli_bpm_hint = canonical_f32_json_number(prepared.source_timing.cli_bpm_hint)?;
    let mut artifacts = Vec::new();
    for (proof, samples) in prepared
        .monitor_proofs
        .iter()
        .zip(&rendered.monitor_outputs)
    {
        write_audio_artifact(
            output_dir,
            proof.artifact_path,
            proof.case_id,
            proof.case_id,
            &samples.samples,
            &mut artifacts,
        )?;
    }
    for (stage, samples) in prepared.stages.iter().zip(&rendered.stage_outputs) {
        write_audio_artifact(
            output_dir,
            stage.artifact_path,
            stage.case_id,
            "performance_stage",
            &samples.samples,
            &mut artifacts,
        )?;
    }
    let continuous = rendered
        .stage_outputs
        .iter()
        .flat_map(|output| output.samples.iter().copied())
        .collect::<Vec<_>>();
    write_audio_artifact(
        output_dir,
        LIVE_SEQUENCE_ARTIFACT_PATH,
        "live-sequence",
        "continuous_performance_sequence",
        &continuous,
        &mut artifacts,
    )?;
    let mut failures = Vec::new();
    let alpha_evidence = alpha_manifest::write_and_validate(
        &prepared,
        &rendered,
        output_dir,
        bpm,
        &mut artifacts,
        &mut failures,
    )?;
    let render_plans_match_confirmed_bpm =
        all_render_plans_match_bpm(&prepared, prepared.source_timing.bpm);
    if !render_plans_match_confirmed_bpm {
        failures.push(
            "one or more runtime render plans diverged from confirmed source hypothesis BPM".into(),
        );
    }
    let monitor_metrics = prepared
        .monitor_proofs
        .iter()
        .zip(&rendered.monitor_outputs)
        .map(|(proof, output)| {
            let metrics =
                signal_metrics_with_grid(&output.samples, SAMPLE_RATE, CHANNEL_COUNT, bpm, 4);
            let route = proof
                .plan
                .source_monitor_render
                .route_for_output(SAMPLE_RATE, usize::from(CHANNEL_COUNT));
            if route != proof.expected_route {
                failures.push(format!(
                    "{} route {} != {}",
                    proof.case_id,
                    route.label(),
                    proof.expected_route.label(),
                ));
            }
            gate_exact_mix_limiter(proof.case_id, "monitor", &output.limiter, &mut failures);
            if metrics.rms <= MIN_MIX_RMS || metrics.clip_count > 0 {
                failures.push(format!(
                    "{} silent or clipping: rms {:.6}, clips {}",
                    proof.case_id, metrics.rms, metrics.clip_count
                ));
            }
            if proof.expected_route == riotbox_audio::runtime::SourceMonitorAudioRoute::SourceOnly
        && f64::from(metrics.silence_ratio) > MAX_SOURCE_MONITOR_SILENCE_RATIO
            {
                failures.push(format!(
                    "{} source review is truncated or too silent: silence ratio {:.4}, maximum {:.4}",
                    proof.case_id, metrics.silence_ratio, MAX_SOURCE_MONITOR_SILENCE_RATIO
                ));
            }
            json!({
                "case_id": proof.case_id,
                "mode": proof.plan.source_monitor_render.mode.as_str(),
                "route": route.label(),
                "action_id": proof.action_id,
                "artifact": proof.artifact_path,
                "metrics": metrics_json(metrics),
                "limiter": limiter_json(output.limiter),
            })
        })
        .collect::<Vec<_>>();
    let source_blend_delta = signal_delta_metrics(
        &rendered.monitor_outputs[0].samples,
        &rendered.monitor_outputs[1].samples,
    );
    let blend_riotbox_delta = signal_delta_metrics(
        &rendered.monitor_outputs[1].samples,
        &rendered.monitor_outputs[2].samples,
    );
    let source_riotbox_delta = signal_delta_metrics(
        &rendered.monitor_outputs[0].samples,
        &rendered.monitor_outputs[2].samples,
    );
    for (name, metrics) in [
        ("source_vs_blend", source_blend_delta),
        ("blend_vs_riotbox", blend_riotbox_delta),
        ("source_vs_riotbox", source_riotbox_delta),
    ] {
        if metrics.rms <= MIN_MONITOR_DELTA_RMS {
            failures.push(format!("monitor {name} delta rms {:.6}", metrics.rms));
        }
    }

    let stage_manifest = prepared
        .stages
        .iter()
        .zip(&rendered.stage_outputs)
        .map(|(stage, output)| {
            let metrics =
                signal_metrics_with_grid(&output.samples, SAMPLE_RATE, CHANNEL_COUNT, bpm, 4);
            gate_exact_mix_limiter(
                stage.case_id,
                "performance_stage",
                &output.limiter,
                &mut failures,
            );
            if metrics.rms <= MIN_MIX_RMS || metrics.clip_count > 0 {
                failures.push(format!(
                    "{} silent or clipping: rms {:.6}, clips {}",
                    stage.case_id, metrics.rms, metrics.clip_count
                ));
            }
            json!({
                "case_id": stage.case_id,
                "duration_beats": stage.duration_beats,
                "key": stage.key,
                "command": stage.command.map(ActionCommand::as_str),
                "boundary": stage.boundary.map(boundary_label),
                "action_id": stage.action_id,
                "scene_id": stage.scene_id,
                "source_anchor_seconds": stage.source_anchor_seconds,
                "monitor_mode": stage.plan.source_monitor_render.mode.as_str(),
                "monitor_route": stage.plan.source_monitor_render
                    .route_for_output(SAMPLE_RATE, usize::from(CHANNEL_COUNT)).label(),
                "artifact": stage.artifact_path,
                "metrics": metrics_json(metrics),
                "limiter": limiter_json(output.limiter),
            })
        })
        .collect::<Vec<_>>();

    let fill_stage_index = prepared
        .stages
        .iter()
        .position(|stage| stage.command == Some(ActionCommand::Tr909FillNext))
        .ok_or("live sequence did not contain a committed TR-909 Fill stage")?;
    let fill_stage = &prepared.stages[fill_stage_index];
    let fill_followup_stage = prepared
        .stages
        .get(fill_stage_index + 1)
        .ok_or("live sequence did not contain a post-Fill follow-up stage")?;
    let fill_output = rendered
        .stage_outputs
        .get(fill_stage_index)
        .ok_or("exact RuntimeMix sequence omitted the committed Fill output")?;
    let fill_followup_output = rendered
        .stage_outputs
        .get(fill_stage_index + 1)
        .ok_or("exact RuntimeMix sequence omitted the post-Fill output")?;
    let fill_recipe_id = fill_stage
        .plan
        .tr909_render
        .fill_recipe_id()
        .ok_or("committed Fill stage did not select a typed Fill recipe")?;
    let fill_pattern_adoption = fill_stage
        .plan
        .tr909_render
        .pattern_adoption
        .ok_or("committed Fill stage did not carry typed pattern adoption")?;
    let fill_phrase_variation = fill_stage
        .plan
        .tr909_render
        .phrase_variation
        .ok_or("committed Fill stage did not carry typed phrase variation")?;
    let fill_transition_index = prepared
        .transitions
        .iter()
        .position(|transition| transition.command == ActionCommand::Tr909FillNext)
        .ok_or("live sequence did not contain a Fill transition proof")?;
    let fill_transition = &prepared.transitions[fill_transition_index];
    let transition_fill_recipe_id = fill_transition
        .after
        .tr909_render
        .fill_recipe_id()
        .ok_or("Fill transition candidate did not select a typed Fill recipe")?;
    if transition_fill_recipe_id != fill_recipe_id {
        return Err(format!(
            "Fill stage recipe {} diverged from transition candidate {}",
            fill_recipe_id.label(),
            transition_fill_recipe_id.label()
        )
        .into());
    }
    let fill_exit_boundary = sequence_boundary_metrics(
        &fill_output.samples,
        &fill_followup_output.samples,
        usize::from(CHANNEL_COUNT),
        (SAMPLE_RATE as usize * FILL_EXIT_BOUNDARY_WINDOW_MS / 1_000).max(1),
    )?;
    let click_like_boundary = fill_exit_boundary.boundary_step > MAX_FILL_EXIT_BOUNDARY_STEP
        && fill_exit_boundary.boundary_to_local_p99_ratio
            > MAX_FILL_EXIT_BOUNDARY_TO_LOCAL_P99_RATIO
        && fill_exit_boundary.boundary_to_attack_rms_ratio
            > MAX_FILL_EXIT_BOUNDARY_TO_ATTACK_RMS_RATIO;
    if !fill_exit_boundary.boundary_step.is_finite()
        || !fill_exit_boundary.local_adjacent_step_p99.is_finite()
        || !fill_exit_boundary.boundary_to_local_p99_ratio.is_finite()
        || !fill_exit_boundary.post_boundary_attack_rms.is_finite()
        || !fill_exit_boundary.boundary_to_attack_rms_ratio.is_finite()
        || click_like_boundary
    {
        failures.push(format!(
            "{} -> {} exact Blend boundary was click-like: step {:.6}, local p99 {:.6}, local ratio {:.3}, attack rms {:.6}, attack ratio {:.3}",
            fill_stage.case_id,
            fill_followup_stage.case_id,
            fill_exit_boundary.boundary_step,
            fill_exit_boundary.local_adjacent_step_p99,
            fill_exit_boundary.boundary_to_local_p99_ratio,
            fill_exit_boundary.post_boundary_attack_rms,
            fill_exit_boundary.boundary_to_attack_rms_ratio,
        ));
    }
    let fill_exit_boundary_manifest = json!({
        "from_case_id": fill_stage.case_id,
        "to_case_id": fill_followup_stage.case_id,
        "expected_role": "fill_release_to_break_slam_downbeat",
        "exact_runtime_mix_sequence": true,
        "window_ms": FILL_EXIT_BOUNDARY_WINDOW_MS,
        "window_frames": fill_exit_boundary.window_frames,
        "boundary_step": fill_exit_boundary.boundary_step,
        "local_adjacent_step_p99": fill_exit_boundary.local_adjacent_step_p99,
        "boundary_to_local_p99_ratio": fill_exit_boundary.boundary_to_local_p99_ratio,
        "post_boundary_attack_rms": fill_exit_boundary.post_boundary_attack_rms,
        "boundary_to_attack_rms_ratio": fill_exit_boundary.boundary_to_attack_rms_ratio,
        "thresholds": {
            "max_boundary_step": MAX_FILL_EXIT_BOUNDARY_STEP,
            "max_boundary_to_local_p99_ratio": MAX_FILL_EXIT_BOUNDARY_TO_LOCAL_P99_RATIO,
            "max_boundary_to_attack_rms_ratio": MAX_FILL_EXIT_BOUNDARY_TO_ATTACK_RMS_RATIO,
        },
    });

    let mut transition_manifest = Vec::new();
    for (index, (transition, output)) in prepared
        .transitions
        .iter()
        .zip(&rendered.transition_outputs)
        .enumerate()
    {
        let before_path = format!(
            "gestures/proofs/{:02}_{}_before.wav",
            index + 1,
            transition.key
        );
        let after_path = format!(
            "gestures/proofs/{:02}_{}_after.wav",
            index + 1,
            transition.key
        );
        write_audio_artifact(
            output_dir,
            &before_path,
            transition.case_id,
            "counterfactual",
            &output.before.samples,
            &mut artifacts,
        )?;
        write_audio_artifact(
            output_dir,
            &after_path,
            transition.case_id,
            "candidate",
            &output.after.samples,
            &mut artifacts,
        )?;
        let before_metrics =
            signal_metrics_with_grid(&output.before.samples, SAMPLE_RATE, CHANNEL_COUNT, bpm, 4);
        let after_metrics =
            signal_metrics_with_grid(&output.after.samples, SAMPLE_RATE, CHANNEL_COUNT, bpm, 4);
        let full_delta = signal_delta_metrics(&output.before.samples, &output.after.samples);
        gate_exact_mix_limiter(
            transition.case_id,
            "counterfactual",
            &output.before.limiter,
            &mut failures,
        );
        gate_exact_mix_limiter(
            transition.case_id,
            "candidate",
            &output.after.limiter,
            &mut failures,
        );
        let policy = gesture_qa_policy(transition.command)?;
        let expected_key = gesture_key(transition.command)?;
        if transition.key != expected_key {
            return Err(format!(
                "gesture key {} does not match command {} key {expected_key}",
                transition.key,
                transition.command.as_str()
            )
            .into());
        }
        let window_sample_count = gesture_window_sample_count(bpm, policy.window_beats);
        let before_window = leading_window(&output.before.samples, window_sample_count);
        let after_window = leading_window(&output.after.samples, window_sample_count);
        let window_candidate_metrics = signal_metrics(after_window);
        let delta = signal_delta_metrics(before_window, after_window);
        let relative_delta_rms = if window_candidate_metrics.rms > f32::EPSILON {
            delta.rms / window_candidate_metrics.rms
        } else {
            0.0
        };
        let relevant_10ms_activity_ratio = relevant_window_activity_ratio(
            before_window,
            after_window,
            SAMPLE_RATE,
            usize::from(CHANNEL_COUNT),
        );
        let waveform_correlation =
            mono_waveform_correlation(before_window, after_window, usize::from(CHANNEL_COUNT));
        if after_metrics.rms <= MIN_MIX_RMS || after_metrics.clip_count > 0 {
            failures.push(format!(
                "{} candidate silent or clipping: rms {:.6}, clips {}",
                transition.case_id, after_metrics.rms, after_metrics.clip_count
            ));
        }
        if delta.rms <= policy.min_delta_rms
            || delta.peak_abs <= policy.min_delta_peak
            || relative_delta_rms <= policy.min_relative_delta_rms
            || delta.active_samples == 0
        {
            failures.push(format!(
                "{} collapsed in {}-beat QA window: delta rms {:.6}, peak {:.6}, relative {:.4}, active {}",
                transition.case_id,
                policy.window_beats,
                delta.rms,
                delta.peak_abs,
                relative_delta_rms,
                delta.active_samples
            ));
        }
        if policy
            .min_relevant_10ms_activity_ratio
            .is_some_and(|minimum| relevant_10ms_activity_ratio < minimum)
        {
            failures.push(format!(
                "{} perceptual delta was too brief: 10ms activity {:.4}, required {:?}",
                transition.case_id,
                relevant_10ms_activity_ratio,
                policy.min_relevant_10ms_activity_ratio,
            ));
        }
        let absolute_waveform_correlation = waveform_correlation.abs();
        if policy
            .max_waveform_correlation
            .is_some_and(|maximum| waveform_is_too_similar(waveform_correlation, maximum))
        {
            failures.push(format!(
                "{} remained too waveform-similar: absolute correlation {:.4} (signed {:.4}), maximum {:?}",
                transition.case_id,
                absolute_waveform_correlation,
                waveform_correlation,
                policy.max_waveform_correlation,
            ));
        }
        transition_manifest.push(json!({
            "case_id": transition.case_id,
            "key": transition.key,
            "command": transition.command.as_str(),
            "actor": "performer",
            "status": "committed",
            "boundary": boundary_label(transition.boundary),
            "action_id": transition.action_id,
            "commit_boundary": {
                "beat_cursor": transition.commit_boundary.beat_index,
                "bar_index": transition.commit_boundary.bar_index,
                "phrase_index": transition.commit_boundary.phrase_index,
            },
            "control_values": {
                "tr909_mode_before": transition.before.tr909_render.mode.label(),
                "tr909_mode_after": transition.after.tr909_render.mode.label(),
                "tr909_drum_bus_level_before": transition.before.tr909_render.drum_bus_level,
                "tr909_drum_bus_level_after": transition.after.tr909_render.drum_bus_level,
                "tr909_slam_enabled_before": transition.before.tr909_render.slam_enabled,
                "tr909_slam_enabled_after": transition.after.tr909_render.slam_enabled,
                "tr909_slam_before": transition.before.tr909_render.slam_intensity,
                "tr909_slam_after": transition.after.tr909_render.slam_intensity,
            },
            "counterfactual_artifact": before_path,
            "candidate_artifact": after_path,
            "counterfactual_metrics": metrics_json(before_metrics),
            "candidate_metrics": metrics_json(after_metrics),
            "counterfactual_limiter": limiter_json(output.before.limiter),
            "candidate_limiter": limiter_json(output.after.limiter),
            "full_delta": metrics_json(full_delta),
            "qa_window_beats": policy.window_beats,
            "qa_candidate_metrics": metrics_json(window_candidate_metrics),
            "qa_thresholds": {
                "min_delta_rms": policy.min_delta_rms,
                "min_delta_peak": policy.min_delta_peak,
                "min_relative_delta_rms": policy.min_relative_delta_rms,
                "min_relevant_10ms_activity_ratio": policy.min_relevant_10ms_activity_ratio,
                "max_waveform_correlation": policy.max_waveform_correlation,
            },
            "relative_delta_rms": relative_delta_rms,
            "perceptual_delta": {
                "window_ms": PERCEPTUAL_WINDOW_MS,
                "candidate_relative_floor": PERCEPTUAL_DELTA_RELATIVE_FLOOR,
                "absolute_floor": PERCEPTUAL_DELTA_ABSOLUTE_FLOOR,
                "relevant_window_activity_ratio": relevant_10ms_activity_ratio,
                "waveform_correlation": waveform_correlation,
                "absolute_waveform_correlation": absolute_waveform_correlation,
            },
            "delta": metrics_json(delta),
        }));
    }

    for (path, case_id, role, samples) in [
        (
            "01_all_lane_hook.wav",
            "legacy-hook",
            "full_mix",
            rendered.normal.samples.as_slice(),
        ),
        (
            "02_all_lane_destructive.wav",
            "legacy-damage",
            "destructive_full_mix",
            rendered.damaged.samples.as_slice(),
        ),
        (
            "stems/01_w30_hook.wav",
            "legacy-w30",
            "w30_stem",
            rendered.w30.samples.as_slice(),
        ),
        (
            "stems/02_tr909_pressure.wav",
            "legacy-tr909",
            "tr909_stem",
            rendered.tr909.samples.as_slice(),
        ),
        (
            "stems/03_mc202_selected_role.wav",
            "legacy-mc202",
            "mc202_selected_role_stem",
            rendered.mc202_selected_role.samples.as_slice(),
        ),
    ] {
        write_audio_artifact(output_dir, path, case_id, role, samples, &mut artifacts)?;
    }
    let source = SourceAudioCache::load_pcm_wav(source_path)?;
    write_interleaved_pcm16_wav(
        output_dir.join("00_source.wav"),
        source.sample_rate,
        source.channel_count,
        source.interleaved_samples(),
    )?;
    artifacts.push(artifact_json("source", "source_reference", "00_source.wav"));

    let mix_metrics = signal_metrics(&rendered.normal.samples);
    let damage_delta = signal_delta_metrics(&rendered.normal.samples, &rendered.damaged.samples);
    let w30_metrics = signal_metrics(&rendered.w30.samples);
    let tr909_metrics = signal_metrics(&rendered.tr909.samples);
    let mc202_metrics = signal_metrics(&rendered.mc202_selected_role.samples);
    let mc202_stem_delta = signal_delta_metrics(
        &rendered.mc202_selected_role.samples,
        &rendered.direct_mc202,
    );
    for (case_id, output) in [
        ("legacy-hook", &rendered.normal),
        ("legacy-damage", &rendered.damaged),
        ("legacy-w30", &rendered.w30),
        ("legacy-tr909-pressure", &rendered.tr909),
        ("legacy-mc202", &rendered.mc202_selected_role),
    ] {
        gate_exact_mix_limiter(
            case_id,
            "legacy_lane_regression",
            &output.limiter,
            &mut failures,
        );
    }
    if mix_metrics.rms <= MIN_MIX_RMS
        || damage_delta.rms <= 0.01
        || w30_metrics.rms <= 0.005
        || tr909_metrics.rms <= MIN_ISOLATED_TR909_REGRESSION_RMS
        || mc202_metrics.rms <= 0.005
        || mc202_stem_delta.rms > 0.000_01
        || mix_metrics.clip_count > 0
    {
        failures.push("legacy dense-break lane proof was silent, collapsed, or clipping".into());
    }

    let source_descriptor = &prepared
        .state
        .source_graph
        .as_ref()
        .ok_or("source graph missing after analysis")?
        .source;
    let result = if failures.is_empty() { "pass" } else { "fail" };
    let fill_transition_artifact = transition_manifest
        .get(fill_transition_index)
        .and_then(|transition| transition.get("candidate_artifact"))
        .and_then(Value::as_str)
        .ok_or("Fill transition manifest omitted its candidate artifact")?;
    let fill_affected_artifacts = vec![
        fill_stage.artifact_path.to_owned(),
        LIVE_SEQUENCE_ARTIFACT_PATH.to_owned(),
        fill_transition_artifact.to_owned(),
    ];
    let fill_activation_ref = format!("/gesture_transitions/{fill_transition_index}");
    let fill_source_transient_backbeat = prepared
        .live_policy
        .source_transient_backbeat_evidence
        .ok_or("Fill product primitive omitted trusted transient-backbeat source evidence")?;
    let resolved_fill_drum_level = fill_stage.plan.tr909_render.drum_bus_level;
    let resolved_fill_slam_intensity = fill_stage.plan.tr909_render.slam_intensity;
    let resolved_fill_bar_anchor = fill_stage
        .plan
        .tr909_render
        .source_bar_grid_anchor_position_beats
        .ok_or("Fill product primitive omitted its confirmed source-bar phase")?;
    if resolved_fill_drum_level + f32::EPSILON < prepared.live_policy.tr909_drum_level
        || resolved_fill_slam_intensity + f32::EPSILON < prepared.live_policy.tr909_slam_floor
    {
        return Err("Fill render inputs did not preserve source-derived pressure floors".into());
    }
    if resolved_fill_bar_anchor != prepared.source_timing.primary_bar_anchor_beat_cursor as f64 {
        return Err("Fill render input drifted from the confirmed source-bar phase".into());
    }
    let pattern_provenance = json!({
        "tr909_fill": {
            "pattern_origin": "primitive_renderer",
            "source_evidence_role": "availability_timing_and_pressure_modulation",
            "source_evidence_selects_pattern": false,
            "source_evidence_modulates_output": true,
            "primitive_schema": TR909_FILL_PRIMITIVE_SCHEMA,
            "recipe_id": fill_recipe_id.label(),
            "selection_inputs": {
                "mode": fill_stage.plan.tr909_render.mode.label(),
                "routing": fill_stage.plan.tr909_render.routing.label(),
                "pattern_adoption": fill_pattern_adoption.label(),
                "phrase_variation": fill_phrase_variation.label(),
            },
            "source_modulation": {
                "schema": "riotbox.tr909_fill_source_modulation.v2",
                "source_feature_path": "session.runtime_state.lane_state.mc202.source_phrase_plan.source_expression.transient_backbeat",
                "source_feature_value": fill_source_transient_backbeat,
                "source_timing_path": "source_graph.timing.primary_hypothesis.transport_bar_grid_anchor.beat_cursor",
                "derived_policy": {
                    "tr909_drum_level": prepared.live_policy.tr909_drum_level,
                    "tr909_slam_floor": prepared.live_policy.tr909_slam_floor,
                    "source_bar_grid_anchor_beat_cursor": prepared.source_timing.primary_bar_anchor_beat_cursor,
                },
                "resolved_render_inputs": {
                    "drum_bus_level": resolved_fill_drum_level,
                    "slam_intensity": resolved_fill_slam_intensity,
                    "slam_enabled": fill_stage.plan.tr909_render.slam_enabled,
                    "source_bar_grid_anchor_position_beats": resolved_fill_bar_anchor,
                },
                "affected_runtime_parameters": [
                    "runtime_mix.tr909.drum_bus_level",
                    "runtime_mix.tr909.slam_intensity",
                    "runtime_mix.tr909.source_bar_grid_phase",
                ],
                "pattern_selection_changed": false,
            },
            "activation_ref": fill_activation_ref,
            "affected_artifacts": fill_affected_artifacts.clone(),
        },
    });
    let primitive_renderer_boundary = json!({
        "schema": "riotbox.primitive_renderer_boundary.v2",
        "evidence_role": "product_primitive_vocabulary",
        "product_output_allowed": true,
        "quality_proof": false,
        "demo_readiness": "unverified",
        "promotion_blocked": true,
        "promotion_target": "source_derived_musical_intelligence",
        "promotion_target_scope": "recipe_and_pattern_selection",
        "recipe_derivation_claimed": false,
        "pattern_selection_claimed": false,
        "source_output_modulation_claimed": true,
        "activation": {
            "kind": "explicit_committed_performer_gesture",
            "references": [fill_activation_ref],
        },
        "source_failure_fallback": false,
        "affected_paths": ["pattern_provenance.tr909_fill.pattern_origin"],
        "affected_runtime_paths": [
            "runtime_mix.tr909.fill_recipe",
            "runtime_mix.tr909.drum_bus_level",
            "runtime_mix.tr909.slam_intensity",
            "runtime_mix.tr909.source_bar_grid_phase",
            "runtime_mix.non_tr909_bed.fill_focus",
            "runtime_mix.source_monitor.blend_fill_focus",
        ],
        "affected_artifacts": fill_affected_artifacts,
        "musician_message": "The committed live Fill uses fixed versioned TR-909 instrument vocabulary and applies its recipe-owned focus hole to the non-TR-909 bed and Blend source. Source evidence gates and times the gesture and modulates its drum level and slam pressure, but it does not select or compose the fixed recipe; this is source-responsive rendering, not source-derived recipe intelligence or musical-quality proof.",
    });
    let mut manifest = json!({
        "schema_version": LISTENING_MANIFEST_SCHEMA_VERSION,
        "pack_id": "dense-break-live-path",
        "result": result,
        "evidence_role": "diagnostic",
        "source_backed": true,
        "source_timing_backed": true,
        "scripted_generation": true,
        "quality_proof": false,
        "human_verdict": "unverified",
        "evidence_boundary": {
            "schema": "riotbox.audio_qa_evidence_boundary.v1",
            "schema_version": 1,
            "evidence_role": "diagnostic",
            "source_backed": true,
            "source_timing_backed": true,
            "scripted_generation": true,
            "quality_proof": false,
            "human_verdict": "unverified",
            "notes": "Scripted exact-mixer reachability and anti-collapse diagnostic; not musical quality proof",
        },
        "product_path": "JamAppState queue/commit -> runtime projections -> exact callback-block RuntimeMix -> SourceMonitor -> master limiter",
        "exact_mixer_proof": {
            "kind": "runtime_mix_callback_block_realtime_simulation",
            "stateful_sequence": true,
            "source_monitor_included": true,
            "master_limiter_included": true,
            "pre_post_limiter_reported": true,
            "limiter_activity_gated": true,
        },
        "correlation_scope": {
            "kind": "action_contract_only",
            "shared_source_fixture": false,
            "shared_transport_timeline": false,
            "sample_exact_observer_correlation": false,
        },
        "sample_rate": SAMPLE_RATE,
        "channel_count": CHANNEL_COUNT,
        "bpm": manifest_bpm.clone(),
        "timing_identity": {
            "cli_bpm_hint": manifest_cli_bpm_hint,
            "confirmed_source_id": prepared.source_timing.source_id.to_string(),
            "confirmed_hypothesis_id": prepared.source_timing.hypothesis_id.as_str(),
            "confirmed_hypothesis_bpm": manifest_bpm.clone(),
            "beats_per_bar": prepared.source_timing.beats_per_bar,
            "primary_bar_anchor_beat_index": prepared.source_timing.primary_bar_anchor_beat_index,
            "primary_bar_anchor_beat_cursor": prepared.source_timing.primary_bar_anchor_beat_cursor,
            "primary_bar_anchor_bar_index": prepared.source_timing.primary_bar_anchor_bar_index,
            "render_plan_bpm": manifest_bpm.clone(),
            "frame_count_bpm": manifest_bpm.clone(),
            "metrics_grid_bpm": manifest_bpm,
            "all_render_plans_match_confirmed_bpm": render_plans_match_confirmed_bpm,
        },
        "source": {
            "source_id": source_descriptor.source_id.to_string(),
            "path": source_descriptor.path,
            "content_hash": source_descriptor.content_hash,
            "sample_rate": source_descriptor.sample_rate,
            "channel_count": source_descriptor.channel_count,
            "duration_seconds": source_descriptor.duration_seconds,
        },
        "thresholds": {
            "min_mix_rms": MIN_MIX_RMS,
            "min_monitor_delta_rms": MIN_MONITOR_DELTA_RMS,
            "min_isolated_tr909_regression_rms": MIN_ISOLATED_TR909_REGRESSION_RMS,
            "max_source_monitor_silence_ratio": MAX_SOURCE_MONITOR_SILENCE_RATIO,
            "max_exact_mix_limited_sample_count": MAX_EXACT_MIX_LIMITED_SAMPLE_COUNT,
        },
        "monitor_cycle": {
            "review_duration_bars": MONITOR_REVIEW_BARS,
            "keys": ["F", "M", "M", "M"],
            "action_ids": prepared.monitor_action_ids,
            "modes": monitor_metrics,
            "deltas": {
                "source_vs_blend": metrics_json(source_blend_delta),
                "blend_vs_riotbox": metrics_json(blend_riotbox_delta),
                "source_vs_riotbox": metrics_json(source_riotbox_delta),
            },
        },
        "performance_stages": stage_manifest,
        "scene_transition_proof": {
            "launch_action_id": prepared.scene_transition_proof.launch_action_id,
            "restore_action_id": prepared.scene_transition_proof.restore_action_id,
            "pre_jump_scene": prepared.scene_transition_proof.pre_jump_scene.as_str(),
            "launched_scene": prepared.scene_transition_proof.launched_scene.as_str(),
            "restored_scene": prepared.scene_transition_proof.restored_scene.as_str(),
            "pre_jump_render_anchor_seconds": prepared.scene_transition_proof.pre_jump_render_anchor_seconds,
            "expected_launch_anchor_seconds": prepared.scene_transition_proof.expected_launch_anchor_seconds,
            "expected_restore_anchor_seconds": prepared.scene_transition_proof.expected_restore_anchor_seconds,
            "launched_anchor_seconds": prepared.scene_transition_proof.launched_anchor_seconds,
            "restored_anchor_seconds": prepared.scene_transition_proof.restored_anchor_seconds,
            "mc202_plan_source_section": prepared.scene_transition_proof.mc202_plan_source_section.as_deref(),
            "launched_source_section": prepared.scene_transition_proof.launched_source_section.as_deref(),
            "launch_mc202_stayed_out_for_section_mismatch": prepared.scene_transition_proof.launch_mc202_stayed_out_for_section_mismatch,
            "restore_audio_projection_matches_pre_jump": prepared.scene_transition_proof.restore_audio_projection_matches_pre_jump,
            "restore_lane_projection_matches_pre_jump": prepared.scene_transition_proof.restore_lane_projection_matches_pre_jump,
            "launch_changed_scene": prepared.scene_transition_proof.pre_jump_scene != prepared.scene_transition_proof.launched_scene,
            "launch_anchor_matches_expected": anchors_match(
                prepared.scene_transition_proof.launched_anchor_seconds,
                prepared.scene_transition_proof.expected_launch_anchor_seconds,
            ),
            "restore_returned_to_pre_jump_scene": prepared.scene_transition_proof.restored_scene == prepared.scene_transition_proof.pre_jump_scene,
            "restore_anchor_matches_expected": anchors_match(
                prepared.scene_transition_proof.restored_anchor_seconds,
                prepared.scene_transition_proof.expected_restore_anchor_seconds,
            ),
        },
        "gesture_transitions": transition_manifest,
        "legacy_lane_regression": {
            "riotbox_monitor_action_id": prepared.legacy_riotbox_action_id,
            "frozen_before_live_fill_slam_scene_gestures": true,
            "plan": {
                "tr909_mode": prepared.normal_plan.tr909_render.mode.label(),
                "tr909_routing": prepared.normal_plan.tr909_render.routing.label(),
                "tr909_slam_intensity": prepared.normal_plan.tr909_render.slam_intensity,
                "transport_position_beats": prepared.normal_plan.transport.position_beats,
                "monitor_mode": prepared.normal_plan.source_monitor_render.mode.as_str(),
                "monitor_route": prepared.normal_plan.source_monitor_render
                    .route_for_output(SAMPLE_RATE, usize::from(CHANNEL_COUNT)).label(),
            },
            "mix": metrics_json(mix_metrics),
            "mix_limiter": limiter_json(rendered.normal.limiter),
            "damage_delta": metrics_json(damage_delta),
            "damage_limiter": limiter_json(rendered.damaged.limiter),
            "w30": metrics_json(w30_metrics),
            "w30_limiter": limiter_json(rendered.w30.limiter),
            "tr909": metrics_json(tr909_metrics),
            "tr909_limiter": limiter_json(rendered.tr909.limiter),
            "mc202_selected_role": metrics_json(mc202_metrics),
            "mc202_limiter": limiter_json(rendered.mc202_selected_role.limiter),
            "mc202_stem_direct_delta": metrics_json(mc202_stem_delta),
        },
        "artifacts": artifacts,
        "failures": failures,
    });
    manifest["pattern_provenance"] = pattern_provenance;
    manifest["primitive_renderer_boundary"] = primitive_renderer_boundary;
    manifest["fill_exit_boundary_proof"] = fill_exit_boundary_manifest;
    let preset_definition = prepared.preset_id.definition();
    manifest["performance_preset"] = json!({
        "preset_id": prepared.preset_id.contract_id(),
        "label": prepared.preset_id.label(),
        "profile_id": preset_definition.profile_id.label(),
        "activation_action_id": prepared.preset_action_id,
        "w30_role": preset_definition.w30_role.label(),
        "tr909_role": preset_definition.tr909_role.label(),
        "mc202_role": preset_definition.mc202_role.label(),
        "bass_ownership_policy": preset_definition.bass_ownership.label(),
        "actual_bass_owner": prepared.live_policy.bass_owner.label(),
    });
    manifest["feral_break_alpha_arc"] = alpha_evidence.arc;
    manifest["feral_break_alpha_restart_recall"] = alpha_evidence.restart_recall;
    manifest["feral_break_alpha_capture_journey"] = alpha_evidence.capture_journey;
    write_manifest_json(&output_dir.join("gesture-manifest.json"), &manifest)?;
    prepared.state.save()?;

    println!("exact live-path manifest: {result}");
    println!("mix: {mix_metrics:?}");
    println!("damage delta: {damage_delta:?}");
    if result != "pass" {
        return Err(format!(
            "dense-break exact live path failed: {}",
            manifest["failures"]
        )
        .into());
    }
    Ok(())
}

fn all_render_plans_match_bpm(prepared: &PreparedLivePath, expected_bpm: f32) -> bool {
    let matches = |actual_bpm: f32| (actual_bpm - expected_bpm).abs() <= f32::EPSILON;

    prepared
        .monitor_proofs
        .iter()
        .all(|proof| matches(proof.plan.transport.tempo_bpm))
        && prepared
            .stages
            .iter()
            .all(|stage| matches(stage.plan.transport.tempo_bpm))
        && prepared
            .alpha_arc_stages
            .iter()
            .all(|stage| matches(stage.plan.transport.tempo_bpm))
        && prepared.transitions.iter().all(|transition| {
            matches(transition.before.transport.tempo_bpm)
                && matches(transition.after.transport.tempo_bpm)
                && transition
                    .prefix
                    .iter()
                    .all(|(plan, _)| matches(plan.transport.tempo_bpm))
        })
        && matches(prepared.normal_plan.transport.tempo_bpm)
        && matches(prepared.damaged_plan.transport.tempo_bpm)
}

pub(super) fn write_audio_artifact(
    output_dir: &Path,
    relative_path: &str,
    case_id: &str,
    role: &str,
    samples: &[f32],
    artifacts: &mut Vec<Value>,
) -> Result<(), Box<dyn Error>> {
    write_interleaved_pcm16_wav(
        output_dir.join(relative_path),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        samples,
    )?;
    artifacts.push(artifact_json(case_id, role, relative_path));
    Ok(())
}

fn artifact_json(case_id: &str, role: &str, path: &str) -> Value {
    json!({
        "case_id": case_id,
        "role": role,
        "kind": "audio_wav",
        "path": path,
        "metrics_path": null,
    })
}

pub(super) fn metrics_json(metrics: OfflineAudioMetrics) -> Value {
    json!({
        "active_samples": metrics.active_samples,
        "peak_abs": metrics.peak_abs,
        "clip_count": metrics.clip_count,
        "near_clip_count": metrics.near_clip_count,
        "headroom_to_full_scale": metrics.headroom_to_full_scale,
        "rms": metrics.rms,
        "sum": metrics.sum,
        "mean_abs": metrics.mean_abs,
        "zero_crossings": metrics.zero_crossings,
        "crest_factor": metrics.crest_factor,
        "active_sample_ratio": metrics.active_sample_ratio,
        "silence_ratio": metrics.silence_ratio,
        "dc_offset": metrics.dc_offset,
        "onset_count": metrics.onset_count,
        "event_density_per_bar": metrics.event_density_per_bar,
    })
}

pub(super) fn limiter_json(report: MasterBusLimiterReport) -> Value {
    json!({
        "applied": report.applied,
        "threshold": report.threshold,
        "ceiling": report.ceiling,
        "limited_sample_count": report.limited_sample_count,
        "pre": metrics_json(report.pre),
        "post": metrics_json(report.post),
    })
}

pub(super) fn gate_exact_mix_limiter(
    case_id: &str,
    role: &str,
    report: &MasterBusLimiterReport,
    failures: &mut Vec<String>,
) {
    if report.pre.clip_count > 0
        || report.limited_sample_count > MAX_EXACT_MIX_LIMITED_SAMPLE_COUNT
        || report.post.clip_count > 0
    {
        failures.push(format!(
            "{case_id} {role} hid a hot exact mix: pre clips {}, limited samples {}, post clips {}",
            report.pre.clip_count, report.limited_sample_count, report.post.clip_count
        ));
    }
}

fn boundary_label(boundary: CommitBoundary) -> &'static str {
    match boundary {
        CommitBoundary::Immediate => "Immediate",
        CommitBoundary::Beat => "Beat",
        CommitBoundary::HalfBar => "HalfBar",
        CommitBoundary::Bar => "Bar",
        CommitBoundary::Phrase => "Phrase",
        CommitBoundary::Scene => "Scene",
    }
}

fn gesture_qa_policy(command: ActionCommand) -> Result<GestureQaPolicy, Box<dyn Error>> {
    let policy = match command {
        ActionCommand::W30TriggerPad => GestureQaPolicy {
            window_beats: 1,
            min_delta_rms: 0.005,
            min_delta_peak: 0.02,
            min_relative_delta_rms: 0.05,
            min_relevant_10ms_activity_ratio: None,
            max_waveform_correlation: None,
        },
        ActionCommand::Tr909FillNext => GestureQaPolicy {
            window_beats: 4,
            min_delta_rms: 0.005,
            min_delta_peak: 0.05,
            min_relative_delta_rms: 0.05,
            min_relevant_10ms_activity_ratio: Some(0.15),
            max_waveform_correlation: Some(0.99),
        },
        ActionCommand::Tr909SetSlam => GestureQaPolicy {
            window_beats: 1,
            min_delta_rms: 0.004,
            min_delta_peak: 0.05,
            min_relative_delta_rms: 0.05,
            min_relevant_10ms_activity_ratio: Some(0.10),
            max_waveform_correlation: Some(0.99),
        },
        ActionCommand::SceneLaunch | ActionCommand::SceneRestore => GestureQaPolicy {
            window_beats: 4,
            min_delta_rms: 0.03,
            min_delta_peak: 0.15,
            min_relative_delta_rms: 0.20,
            min_relevant_10ms_activity_ratio: None,
            max_waveform_correlation: None,
        },
        _ => {
            return Err(format!("missing gesture QA policy for {}", command.as_str()).into());
        }
    };
    Ok(policy)
}

fn gesture_key(command: ActionCommand) -> Result<&'static str, Box<dyn Error>> {
    match command {
        ActionCommand::W30TriggerPad => Ok("w"),
        ActionCommand::Tr909FillNext => Ok("f"),
        ActionCommand::Tr909SetSlam => Ok("s"),
        ActionCommand::SceneLaunch => Ok("y"),
        ActionCommand::SceneRestore => Ok("Y"),
        _ => Err(format!("missing gesture key for {}", command.as_str()).into()),
    }
}

fn gesture_window_sample_count(bpm: f32, beats: u32) -> usize {
    let frames =
        (f64::from(SAMPLE_RATE) * 60.0 / f64::from(bpm) * f64::from(beats)).round() as usize;
    frames.saturating_mul(usize::from(CHANNEL_COUNT))
}

fn leading_window(samples: &[f32], sample_count: usize) -> &[f32] {
    &samples[..samples.len().min(sample_count)]
}

fn sequence_boundary_metrics(
    before: &[f32],
    after: &[f32],
    channel_count: usize,
    requested_window_frames: usize,
) -> Result<SequenceBoundaryMetrics, Box<dyn Error>> {
    if channel_count == 0
        || before.len() < channel_count.saturating_mul(2)
        || after.len() < channel_count.saturating_mul(2)
    {
        return Err("sequence boundary requires two complete frames on both sides".into());
    }
    let before_frames = before.len() / channel_count;
    let after_frames = after.len() / channel_count;
    let window_frames = requested_window_frames
        .min(before_frames)
        .min(after_frames)
        .max(2);
    let before_last = &before[(before_frames - 1) * channel_count..before_frames * channel_count];
    let after_first = &after[..channel_count];
    let before_window = &before[(before_frames - window_frames) * channel_count..];
    let after_window = &after[..window_frames * channel_count];
    if before_window
        .iter()
        .chain(after_window)
        .any(|sample| !sample.is_finite())
    {
        return Err("sequence boundary contains non-finite samples".into());
    }
    let boundary_step = frame_step(before_last, after_first);

    let mut local_steps = Vec::with_capacity(window_frames.saturating_mul(2));
    collect_adjacent_frame_steps(before_window, channel_count, &mut local_steps);
    collect_adjacent_frame_steps(after_window, channel_count, &mut local_steps);
    local_steps.sort_by(f32::total_cmp);
    let p99_index = ((local_steps.len() as f64 * 0.99).ceil() as usize)
        .saturating_sub(1)
        .min(local_steps.len().saturating_sub(1));
    let local_adjacent_step_p99 = local_steps[p99_index];
    let boundary_to_local_p99_ratio = boundary_step / local_adjacent_step_p99.max(f32::EPSILON);
    // A hard Fill release is allowed to land on a real downbeat transient. A discontinuity only
    // becomes click-like when it is both locally anomalous and unsupported by the following
    // two-millisecond attack body.
    let attack_frames = (window_frames / 5).max(2);
    let post_boundary_attack_rms =
        frame_window_rms(&after_window[..attack_frames.min(window_frames) * channel_count]);
    let boundary_to_attack_rms_ratio = boundary_step / post_boundary_attack_rms.max(f32::EPSILON);

    Ok(SequenceBoundaryMetrics {
        boundary_step,
        local_adjacent_step_p99,
        boundary_to_local_p99_ratio,
        post_boundary_attack_rms,
        boundary_to_attack_rms_ratio,
        window_frames,
    })
}

fn frame_window_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples
        .iter()
        .map(|sample| f64::from(*sample) * f64::from(*sample))
        .sum::<f64>()
        / samples.len() as f64)
        .sqrt() as f32
}

fn collect_adjacent_frame_steps(samples: &[f32], channel_count: usize, output: &mut Vec<f32>) {
    let mut frames = samples.chunks_exact(channel_count);
    let Some(mut previous) = frames.next() else {
        return;
    };
    for current in frames {
        output.push(frame_step(previous, current));
        previous = current;
    }
}

fn frame_step(before: &[f32], after: &[f32]) -> f32 {
    before
        .iter()
        .zip(after)
        .map(|(before, after)| (after - before).abs())
        .fold(0.0, f32::max)
}

fn relevant_window_activity_ratio(
    before: &[f32],
    after: &[f32],
    sample_rate: u32,
    channel_count: usize,
) -> f32 {
    if channel_count == 0 {
        return 0.0;
    }
    let frame_count = before.len().min(after.len()) / channel_count;
    let window_frames = ((sample_rate as usize * PERCEPTUAL_WINDOW_MS) / 1_000).max(1);
    let window_count = frame_count / window_frames;
    if window_count == 0 {
        return 0.0;
    }

    let relevant = (0..window_count)
        .filter(|window| {
            let start = window * window_frames;
            let end = start + window_frames;
            let mut candidate_square_sum = 0.0_f64;
            let mut delta_square_sum = 0.0_f64;
            for frame in start..end {
                let sample_start = frame * channel_count;
                let before_mono = before[sample_start..sample_start + channel_count]
                    .iter()
                    .copied()
                    .sum::<f32>()
                    / channel_count as f32;
                let after_mono = after[sample_start..sample_start + channel_count]
                    .iter()
                    .copied()
                    .sum::<f32>()
                    / channel_count as f32;
                candidate_square_sum += f64::from(after_mono) * f64::from(after_mono);
                let delta = after_mono - before_mono;
                delta_square_sum += f64::from(delta) * f64::from(delta);
            }
            let denominator = window_frames as f64;
            let candidate_rms = (candidate_square_sum / denominator).sqrt() as f32;
            let delta_rms = (delta_square_sum / denominator).sqrt() as f32;
            delta_rms
                > (candidate_rms * PERCEPTUAL_DELTA_RELATIVE_FLOOR)
                    .max(PERCEPTUAL_DELTA_ABSOLUTE_FLOOR)
        })
        .count();

    relevant as f32 / window_count as f32
}

pub(super) fn mono_waveform_correlation(
    before: &[f32],
    after: &[f32],
    channel_count: usize,
) -> f32 {
    if channel_count == 0 {
        return 1.0;
    }
    let frame_count = before.len().min(after.len()) / channel_count;
    if frame_count < 2 {
        return 1.0;
    }

    let mono = |samples: &[f32], frame: usize| {
        let start = frame * channel_count;
        samples[start..start + channel_count]
            .iter()
            .copied()
            .sum::<f32>() as f64
            / channel_count as f64
    };
    let mut before_sum = 0.0_f64;
    let mut after_sum = 0.0_f64;
    for frame in 0..frame_count {
        before_sum += mono(before, frame);
        after_sum += mono(after, frame);
    }
    let before_mean = before_sum / frame_count as f64;
    let after_mean = after_sum / frame_count as f64;
    let mut covariance = 0.0_f64;
    let mut before_variance = 0.0_f64;
    let mut after_variance = 0.0_f64;
    for frame in 0..frame_count {
        let before_centered = mono(before, frame) - before_mean;
        let after_centered = mono(after, frame) - after_mean;
        covariance += before_centered * after_centered;
        before_variance += before_centered * before_centered;
        after_variance += after_centered * after_centered;
    }
    let denominator = (before_variance * after_variance).sqrt();
    if denominator <= f64::EPSILON {
        return if before_variance <= f64::EPSILON && after_variance <= f64::EPSILON {
            1.0
        } else {
            0.0
        };
    }
    (covariance / denominator).clamp(-1.0, 1.0) as f32
}

pub(super) fn waveform_is_too_similar(correlation: f32, maximum: f32) -> bool {
    correlation.abs() > maximum
}

fn anchors_match(actual: Option<f64>, expected: Option<f64>) -> bool {
    match (actual, expected) {
        (Some(actual), Some(expected)) => (actual - expected).abs() <= 1.0e-6,
        _ => false,
    }
}

fn canonical_f32_json_number(value: f32) -> Result<Value, serde_json::Error> {
    serde_json::from_str(&serde_json::to_string(&value)?)
}

#[cfg(test)]
mod tests {
    use super::{
        anchors_match, canonical_f32_json_number, mono_waveform_correlation,
        relevant_window_activity_ratio, sequence_boundary_metrics, waveform_is_too_similar,
    };

    #[test]
    fn timing_identity_uses_the_canonical_source_graph_f32_number() {
        let bpm = canonical_f32_json_number(131.878_f32).expect("finite BPM");

        assert_eq!(bpm.to_string(), "131.878");
    }

    #[test]
    fn positive_scene_anchor_match_requires_present_grid_evidence() {
        assert!(anchors_match(Some(3.5), Some(3.5)));
        assert!(!anchors_match(None, None));
        assert!(!anchors_match(Some(3.5), None));
    }

    #[test]
    fn perceptual_delta_rejects_one_brief_spike_but_accepts_sustained_change() {
        let sample_rate = 1_000;
        let mut before = vec![0.2; 200];
        let mut brief = before.clone();
        brief[0] = 0.8;
        let sustained = vec![0.3; 200];

        assert!(
            relevant_window_activity_ratio(&before, &brief, sample_rate, 1) < 0.10,
            "one sample must not masquerade as a sustained gesture"
        );
        assert!(
            relevant_window_activity_ratio(&before, &sustained, sample_rate, 1) > 0.99,
            "a sustained audible change should cover the review window"
        );
        before.fill(0.0);
        assert_eq!(mono_waveform_correlation(&before, &sustained, 1), 1.0);
    }

    #[test]
    fn waveform_correlation_distinguishes_identical_and_opposed_shapes() {
        let signal = [-1.0, -0.5, 0.5, 1.0];
        let inverted = [1.0, 0.5, -0.5, -1.0];

        let identical_correlation = mono_waveform_correlation(&signal, &signal, 1);
        let inverted_correlation = mono_waveform_correlation(&signal, &inverted, 1);

        assert!((identical_correlation - 1.0).abs() < 1.0e-6);
        assert!((inverted_correlation + 1.0).abs() < 1.0e-6);
        assert!(waveform_is_too_similar(identical_correlation, 0.99));
        assert!(
            waveform_is_too_similar(inverted_correlation, 0.99),
            "polarity inversion must not masquerade as a different gesture"
        );
    }

    #[test]
    fn sequence_boundary_metrics_compare_the_join_with_local_waveform_steps() {
        let before = (0..100)
            .map(|frame| frame as f32 * 0.01)
            .collect::<Vec<_>>();
        let after = (0..100)
            .map(|frame| 1.14 + frame as f32 * 0.01)
            .collect::<Vec<_>>();

        let metrics = sequence_boundary_metrics(&before, &after, 1, 100).expect("boundary");

        assert!((metrics.boundary_step - 0.15).abs() < 1.0e-5);
        assert!((metrics.local_adjacent_step_p99 - 0.01).abs() < 1.0e-5);
        assert!((metrics.boundary_to_local_p99_ratio - 15.0).abs() < 0.01);
        assert!(metrics.post_boundary_attack_rms > 1.0);
        assert!(metrics.boundary_to_attack_rms_ratio < 1.0);
    }

    #[test]
    fn sequence_boundary_metrics_distinguish_a_supported_downbeat_from_an_isolated_click() {
        let before = vec![0.0; 500];
        let supported = (0..500)
            .map(|frame| 0.28 * (-frame as f32 / 80.0).exp())
            .collect::<Vec<_>>();
        let mut click = vec![0.0; 500];
        click[0] = 0.28;

        let supported_metrics =
            sequence_boundary_metrics(&before, &supported, 1, 500).expect("supported");
        let click_metrics = sequence_boundary_metrics(&before, &click, 1, 500).expect("click");

        assert!(supported_metrics.boundary_to_attack_rms_ratio < 4.0);
        assert!(click_metrics.boundary_to_attack_rms_ratio > 4.0);
    }

    #[test]
    fn sequence_boundary_metrics_reject_non_finite_samples() {
        let before = vec![0.0; 100];
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut after = vec![0.0; 100];
            after[0] = invalid;
            assert!(
                sequence_boundary_metrics(&before, &after, 1, 100).is_err(),
                "non-finite boundary sample {invalid:?} must fail closed"
            );
        }
    }
}
