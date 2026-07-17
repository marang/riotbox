use std::{error::Error, path::Path};

use riotbox_audio::runtime::{signal_delta_metrics, signal_metrics, signal_metrics_with_grid};
use riotbox_core::action::ActionCommand;
use serde_json::{Value, json};

use crate::{
    manifest::{
        gate_exact_mix_limiter, limiter_json, metrics_json, mono_waveform_correlation,
        waveform_is_too_similar, write_audio_artifact,
    },
    model::{
        CHANNEL_COUNT, MIN_MIX_RMS, MIN_MONITOR_DELTA_RMS, PreparedLivePath, RenderedLivePath,
        SAMPLE_RATE,
    },
};

const ALPHA_ARC_ARTIFACT_PATH: &str = "alpha/05_feral_break_alpha_eight_bar.wav";
const ALPHA_SOURCE_RAW_ARTIFACT_PATH: &str = "alpha/06_source_reference_raw.wav";
const ALPHA_CANDIDATE_MATCHED_ARTIFACT_PATH: &str = "alpha/07_candidate_loudness_matched.wav";
const ALPHA_SOURCE_MATCHED_ARTIFACT_PATH: &str = "alpha/08_source_reference_loudness_matched.wav";
const ALPHA_RESTART_RECALL_ARTIFACT_PATH: &str = "alpha/09_restart_recall_trigger.wav";
const DESTRUCTIVE_NEGATIVE_SPACE_START_STEP: usize = 26;
const DESTRUCTIVE_NEGATIVE_SPACE_END_STEP: usize = 30;
const DESTRUCTIVE_HARD_RETURN_END_STEP: usize = 31;
const TR909_STEPS_PER_BEAT: usize = 8;
const MAX_DESTRUCTIVE_NEGATIVE_SPACE_RMS: f32 = 0.001;
const MIN_DESTRUCTIVE_NEGATIVE_SPACE_SILENCE_RATIO: f32 = 0.95;
const MIN_DESTRUCTIVE_HARD_RETURN_RMS: f32 = 0.05;

pub(super) struct AlphaManifestEvidence {
    pub arc: Value,
    pub restart_recall: Value,
    pub capture_journey: Value,
}

pub(super) fn write_and_validate(
    prepared: &PreparedLivePath,
    rendered: &RenderedLivePath,
    output_dir: &Path,
    bpm: f32,
    artifacts: &mut Vec<Value>,
    failures: &mut Vec<String>,
) -> Result<AlphaManifestEvidence, Box<dyn Error>> {
    if prepared.alpha_arc_stages.len() != 5
        || rendered.alpha_arc_outputs.len() != prepared.alpha_arc_stages.len()
    {
        return Err(format!(
            "Feral Break Alpha evidence requires five matched stages, got {} plans and {} outputs",
            prepared.alpha_arc_stages.len(),
            rendered.alpha_arc_outputs.len()
        )
        .into());
    }

    for (stage, samples) in prepared
        .alpha_arc_stages
        .iter()
        .zip(&rendered.alpha_arc_outputs)
    {
        write_audio_artifact(
            output_dir,
            stage.artifact_path,
            stage.case_id,
            "feral_break_alpha_arc_stage",
            &samples.samples,
            artifacts,
        )?;
    }
    let alpha_arc_continuous = rendered
        .alpha_arc_outputs
        .iter()
        .flat_map(|output| output.samples.iter().copied())
        .collect::<Vec<_>>();
    write_audio_artifact(
        output_dir,
        ALPHA_ARC_ARTIFACT_PATH,
        "feral-break-alpha-eight-bar",
        "continuous_eight_bar_golden_path_candidate",
        &alpha_arc_continuous,
        artifacts,
    )?;
    write_audio_artifact(
        output_dir,
        ALPHA_SOURCE_RAW_ARTIFACT_PATH,
        "feral-break-alpha-source-reference-raw",
        "bounded_source_reference_raw_level",
        &rendered.alpha_source_reference.samples,
        artifacts,
    )?;
    let alpha_raw_metrics = signal_metrics(&alpha_arc_continuous);
    let alpha_source_raw_metrics = signal_metrics(&rendered.alpha_source_reference.samples);
    let loudness_match_target_rms = alpha_raw_metrics.rms.min(alpha_source_raw_metrics.rms);
    let alpha_candidate_matched = scale_to_rms(&alpha_arc_continuous, loudness_match_target_rms);
    let alpha_source_matched = scale_to_rms(
        &rendered.alpha_source_reference.samples,
        loudness_match_target_rms,
    );
    write_audio_artifact(
        output_dir,
        ALPHA_CANDIDATE_MATCHED_ARTIFACT_PATH,
        "feral-break-alpha-candidate-loudness-matched",
        "eight_bar_candidate_loudness_matched",
        &alpha_candidate_matched,
        artifacts,
    )?;
    write_audio_artifact(
        output_dir,
        ALPHA_SOURCE_MATCHED_ARTIFACT_PATH,
        "feral-break-alpha-source-reference-loudness-matched",
        "bounded_source_reference_loudness_matched",
        &alpha_source_matched,
        artifacts,
    )?;
    let alpha_candidate_matched_metrics = signal_metrics(&alpha_candidate_matched);
    let alpha_source_matched_metrics = signal_metrics(&alpha_source_matched);
    write_audio_artifact(
        output_dir,
        ALPHA_RESTART_RECALL_ARTIFACT_PATH,
        "feral-break-alpha-restart-recall-trigger",
        "restart_recall_trigger_exact_mix",
        &rendered.restart_recall_output.samples,
        artifacts,
    )?;
    let restart_recall_metrics = signal_metrics_with_grid(
        &rendered.restart_recall_output.samples,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        bpm,
        4,
    );

    if (alpha_candidate_matched_metrics.rms - alpha_source_matched_metrics.rms).abs() > 1.0e-5 {
        failures.push(format!(
            "Feral Break Alpha loudness match diverged: candidate {:.6}, source {:.6}",
            alpha_candidate_matched_metrics.rms, alpha_source_matched_metrics.rms
        ));
    }
    gate_exact_mix_limiter(
        "feral-break-alpha-restart-recall-trigger",
        "restart_recall_trigger",
        &rendered.restart_recall_output.limiter,
        failures,
    );
    if restart_recall_metrics.rms <= MIN_MIX_RMS || restart_recall_metrics.clip_count > 0 {
        failures.push(format!(
            "Feral Break Alpha restart recall trigger silent or clipping: rms {:.6}, clips {}",
            restart_recall_metrics.rms, restart_recall_metrics.clip_count
        ));
    }
    let alpha_arc_total_beats = prepared
        .alpha_arc_stages
        .iter()
        .map(|stage| stage.duration_beats)
        .sum::<u32>();
    if alpha_arc_total_beats != 32 {
        failures.push(format!(
            "Feral Break Alpha arc is {alpha_arc_total_beats} beats, expected 32"
        ));
    }
    let destructive_fill_samples = &rendered.alpha_arc_outputs[2].samples;
    let destructive_negative_space = step_window(
        destructive_fill_samples,
        bpm,
        DESTRUCTIVE_NEGATIVE_SPACE_START_STEP,
        DESTRUCTIVE_NEGATIVE_SPACE_END_STEP,
    )?;
    let destructive_hard_return = step_window(
        destructive_fill_samples,
        bpm,
        DESTRUCTIVE_NEGATIVE_SPACE_END_STEP,
        DESTRUCTIVE_HARD_RETURN_END_STEP,
    )?;
    let destructive_negative_space_metrics = signal_metrics(destructive_negative_space);
    let destructive_hard_return_metrics = signal_metrics(destructive_hard_return);
    if destructive_negative_space_metrics.rms > MAX_DESTRUCTIVE_NEGATIVE_SPACE_RMS
        || destructive_negative_space_metrics.silence_ratio
            < MIN_DESTRUCTIVE_NEGATIVE_SPACE_SILENCE_RATIO
    {
        failures.push(format!(
            "Feral Break Alpha destructive pause did not create negative space: rms {:.6}, silence ratio {:.6}",
            destructive_negative_space_metrics.rms,
            destructive_negative_space_metrics.silence_ratio
        ));
    }
    if destructive_hard_return_metrics.rms < MIN_DESTRUCTIVE_HARD_RETURN_RMS {
        failures.push(format!(
            "Feral Break Alpha destructive pause did not return hard enough: rms {:.6}",
            destructive_hard_return_metrics.rms
        ));
    }
    let alpha_arc_stage_manifest = prepared
        .alpha_arc_stages
        .iter()
        .zip(&rendered.alpha_arc_outputs)
        .map(|(stage, output)| {
            let metrics =
                signal_metrics_with_grid(&output.samples, SAMPLE_RATE, CHANNEL_COUNT, bpm, 4);
            gate_exact_mix_limiter(
                stage.case_id,
                "feral_break_alpha_arc_stage",
                &output.limiter,
                failures,
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
                "action_id": stage.action_id,
                "scene_id": stage.scene_id,
                "tr909_mode": stage.plan.tr909_render.mode.label(),
                "tr909_fill_recipe_id": stage
                    .plan
                    .tr909_render
                    .fill_recipe_id()
                    .map(|recipe| recipe.label()),
                "artifact": stage.artifact_path,
                "metrics": metrics_json(metrics),
                "limiter": limiter_json(output.limiter),
            })
        })
        .collect::<Vec<_>>();
    let alpha_hook_pressure_delta = signal_delta_metrics(
        &rendered.alpha_arc_outputs[0].samples,
        &rendered.alpha_arc_outputs[1].samples,
    );
    let alpha_hook_return_delta = signal_delta_metrics(
        &rendered.alpha_arc_outputs[0].samples,
        &rendered.alpha_arc_outputs[4].samples,
    );
    let alpha_hook_return_correlation = mono_waveform_correlation(
        &rendered.alpha_arc_outputs[0].samples,
        &rendered.alpha_arc_outputs[4].samples,
        usize::from(CHANNEL_COUNT),
    );
    if alpha_hook_pressure_delta.rms <= MIN_MONITOR_DELTA_RMS {
        failures.push(format!(
            "Feral Break Alpha pressure lift delta rms {:.6}",
            alpha_hook_pressure_delta.rms
        ));
    }
    if alpha_hook_return_delta.rms <= MIN_MONITOR_DELTA_RMS
        || waveform_is_too_similar(alpha_hook_return_correlation, 0.985)
    {
        failures.push(format!(
            "Feral Break Alpha return did not materially change: delta rms {:.6}, correlation {:.6}",
            alpha_hook_return_delta.rms, alpha_hook_return_correlation
        ));
    }

    Ok(AlphaManifestEvidence {
        arc: json!({
            "artifact": ALPHA_ARC_ARTIFACT_PATH,
            "duration_beats": alpha_arc_total_beats,
            "duration_bars": alpha_arc_total_beats / 4,
            "stages": alpha_arc_stage_manifest,
            "actions": {
                "hook": prepared.alpha_arc_proof.hook_action_id,
                "pressure_lift": prepared.alpha_arc_proof.pressure_action_id,
                "destructive_fill": prepared.alpha_arc_proof.destructive_fill_action_id,
                "role_swap": prepared.alpha_arc_proof.role_swap_action_id,
                "return": prepared.alpha_arc_proof.return_action_id,
                "return_damage": prepared.alpha_arc_proof.return_damage_action_id,
            },
            "scenes": {
                "original": prepared.alpha_arc_proof.original_scene,
                "contrast": prepared.alpha_arc_proof.contrast_scene,
                "returned": prepared.alpha_arc_proof.returned_scene,
            },
            "hook_to_pressure_delta": metrics_json(alpha_hook_pressure_delta),
            "hook_to_changed_return_delta": metrics_json(alpha_hook_return_delta),
            "hook_to_changed_return_correlation": alpha_hook_return_correlation,
            "destructive_negative_space": {
                "window": {
                    "start_step": DESTRUCTIVE_NEGATIVE_SPACE_START_STEP,
                    "end_step_exclusive": DESTRUCTIVE_NEGATIVE_SPACE_END_STEP,
                    "steps_per_beat": TR909_STEPS_PER_BEAT,
                },
                "metrics": metrics_json(destructive_negative_space_metrics),
                "thresholds": {
                    "max_rms": MAX_DESTRUCTIVE_NEGATIVE_SPACE_RMS,
                    "min_silence_ratio": MIN_DESTRUCTIVE_NEGATIVE_SPACE_SILENCE_RATIO,
                },
                "hard_return": {
                    "start_step": DESTRUCTIVE_NEGATIVE_SPACE_END_STEP,
                    "end_step_exclusive": DESTRUCTIVE_HARD_RETURN_END_STEP,
                    "metrics": metrics_json(destructive_hard_return_metrics),
                    "min_rms": MIN_DESTRUCTIVE_HARD_RETURN_RMS,
                },
            },
            "typed_bass_owner": prepared.live_policy.bass_owner.label(),
            "raw_level_ab": {
                "candidate_artifact": ALPHA_ARC_ARTIFACT_PATH,
                "source_artifact": ALPHA_SOURCE_RAW_ARTIFACT_PATH,
                "candidate_metrics": metrics_json(alpha_raw_metrics),
                "source_metrics": metrics_json(alpha_source_raw_metrics),
            },
            "loudness_matched_ab": {
                "target_rms": loudness_match_target_rms,
                "candidate_artifact": ALPHA_CANDIDATE_MATCHED_ARTIFACT_PATH,
                "source_artifact": ALPHA_SOURCE_MATCHED_ARTIFACT_PATH,
                "candidate_metrics": metrics_json(alpha_candidate_matched_metrics),
                "source_metrics": metrics_json(alpha_source_matched_metrics),
            },
            "human_verdict": "unverified",
        }),
        restart_recall: json!({
            "preset_survived_restart": prepared.restart_recall_proof.preset_survived_restart,
            "capture_id": prepared.restart_recall_proof.capture_id.as_str(),
            "recall_action_id": prepared.restart_recall_proof.recall_action_id,
            "trigger_action_id": prepared.restart_recall_proof.trigger_action_id,
            "artifact": ALPHA_RESTART_RECALL_ARTIFACT_PATH,
            "monitor_mode": prepared.restart_recall_plan.source_monitor_render.mode.as_str(),
            "w30_routing": prepared.restart_recall_plan.w30_preview_render.routing.label(),
            "metrics": metrics_json(restart_recall_metrics),
            "limiter": limiter_json(rendered.restart_recall_output.limiter),
        }),
        capture_journey: json!({
            "capture_action_id": prepared.capture_journey_proof.capture_action_id,
            "raw_audition_action_id": prepared.capture_journey_proof.raw_audition_action_id,
            "promotion_action_id": prepared.capture_journey_proof.promotion_action_id,
            "saved_before_restart": true,
            "restart_recall_action_id": prepared.restart_recall_proof.recall_action_id,
            "restart_trigger_action_id": prepared.restart_recall_proof.trigger_action_id,
            "sequence": [
                "capture",
                "raw_audition",
                "promote_to_pad",
                "save",
                "restart",
                "live_recall",
                "trigger",
            ],
        }),
    })
}

fn step_window(
    samples: &[f32],
    bpm: f32,
    start_step: usize,
    end_step: usize,
) -> Result<&[f32], Box<dyn Error>> {
    let frames_per_beat = (60.0 / bpm * SAMPLE_RATE as f32).round() as usize;
    let start_frame = frames_per_beat.saturating_mul(start_step) / TR909_STEPS_PER_BEAT;
    let end_frame = frames_per_beat.saturating_mul(end_step) / TR909_STEPS_PER_BEAT;
    let channels = usize::from(CHANNEL_COUNT);
    let start_sample = start_frame.saturating_mul(channels);
    let end_sample = end_frame.saturating_mul(channels);
    samples.get(start_sample..end_sample).ok_or_else(|| {
        format!(
            "Feral Break Alpha step window {start_step}..{end_step} exceeded {} samples",
            samples.len()
        )
        .into()
    })
}

fn scale_to_rms(samples: &[f32], target_rms: f32) -> Vec<f32> {
    let current_rms = signal_metrics(samples).rms;
    if target_rms <= 0.0 || current_rms <= f32::EPSILON {
        return samples.to_vec();
    }
    let gain = (target_rms / current_rms).min(1.0);
    samples.iter().map(|sample| sample * gain).collect()
}
