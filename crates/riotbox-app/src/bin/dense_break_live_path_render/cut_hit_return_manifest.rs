use std::{error::Error, fs, path::Path};

use riotbox_audio::runtime::{signal_delta_metrics, signal_metrics};
use riotbox_audio::tr909::{Tr909FillRecipeId, Tr909RenderMode};
use serde_json::{Value, json};

use crate::{
    manifest::{gate_exact_mix_limiter, limiter_json, metrics_json, write_audio_artifact},
    model::{CHANNEL_COUNT, PreparedLivePath, RenderedLivePath, SAMPLE_RATE},
};

const TR909_STEPS_PER_BEAT: usize = 8;
const NEGATIVE_SPACE_START_STEP: usize = 26;
const NEGATIVE_SPACE_END_STEP: usize = 30;
const HARD_RETURN_END_STEP: usize = 31;
const MIN_CONTROL_WINDOW_RMS: f32 = 0.01;
const MIN_HARD_RETURN_RMS: f32 = 0.05;

pub(super) fn write_and_validate(
    prepared: &PreparedLivePath,
    rendered: &RenderedLivePath,
    output_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(output_dir.join("cut-hit-return"))?;
    let mut failures = Vec::new();
    let mut artifacts = Vec::<Value>::new();
    write_audio_artifact(
        output_dir,
        "cut-hit-return/00_slam_only_control.wav",
        "cut-hit-return-slam-only-control",
        "technical_control",
        &rendered.cut_hit_return_slam_only_control.samples,
        &mut artifacts,
    )?;
    write_audio_artifact(
        output_dir,
        "cut-hit-return/01_fill_only_attribution.wav",
        "cut-hit-return-fill-only-attribution",
        "attribution_control",
        &rendered.cut_hit_return_fill_only_control.samples,
        &mut artifacts,
    )?;
    write_audio_artifact(
        output_dir,
        "cut-hit-return/02_candidate.wav",
        "cut-hit-return-candidate",
        "candidate",
        &rendered.cut_hit_return_candidate.samples,
        &mut artifacts,
    )?;
    write_audio_artifact(
        output_dir,
        "cut-hit-return/03_changed_return.wav",
        "cut-hit-return-changed-return",
        "technical_return",
        &rendered.cut_hit_return_changed_return.samples,
        &mut artifacts,
    )?;

    let candidate_pause = step_window(
        &rendered.cut_hit_return_candidate.samples,
        prepared.source_timing.bpm,
        NEGATIVE_SPACE_START_STEP,
        NEGATIVE_SPACE_END_STEP,
    )?;
    let candidate_hit = step_window(
        &rendered.cut_hit_return_candidate.samples,
        prepared.source_timing.bpm,
        NEGATIVE_SPACE_END_STEP,
        HARD_RETURN_END_STEP,
    )?;
    let control_pause = step_window(
        &rendered.cut_hit_return_slam_only_control.samples,
        prepared.source_timing.bpm,
        NEGATIVE_SPACE_START_STEP,
        NEGATIVE_SPACE_END_STEP,
    )?;
    let candidate_pause_metrics = signal_metrics(candidate_pause);
    let candidate_hit_metrics = signal_metrics(candidate_hit);
    let control_pause_metrics = signal_metrics(control_pause);
    let candidate_control_delta = signal_delta_metrics(
        &rendered.cut_hit_return_slam_only_control.samples,
        &rendered.cut_hit_return_candidate.samples,
    );
    let candidate_fill_delta = signal_delta_metrics(
        &rendered.cut_hit_return_fill_only_control.samples,
        &rendered.cut_hit_return_candidate.samples,
    );

    if !candidate_pause.iter().all(|sample| *sample == 0.0) {
        failures.push("candidate negative-space window was not digitally silent".into());
    }
    if control_pause_metrics.rms < MIN_CONTROL_WINDOW_RMS {
        failures.push(format!(
            "Slam-only control was already silent in the declared cut window: rms {:.6}",
            control_pause_metrics.rms
        ));
    }
    if candidate_hit_metrics.rms < MIN_HARD_RETURN_RMS {
        failures.push(format!(
            "candidate late hit was below the inherited V2 return floor: rms {:.6}",
            candidate_hit_metrics.rms
        ));
    }
    if candidate_control_delta.active_samples == 0 {
        failures.push("candidate was byte-identical to the Slam-only control".into());
    }
    if !rendered.cut_hit_return_callback_partition_invariant {
        failures.push("candidate changed across callback partitions 128 and 257".into());
    }
    if prepared
        .cut_hit_return_proof
        .candidate_plan
        .tr909_render
        .fill_recipe_id()
        != Some(Tr909FillRecipeId::PhraseDriveBreakCutStompV2)
    {
        failures.push("candidate did not retain PhraseDriveBreakCutStompV2".into());
    }
    if prepared
        .cut_hit_return_proof
        .changed_return_plan
        .tr909_render
        .mode
        != Tr909RenderMode::BreakReinforce
        || !prepared
            .cut_hit_return_proof
            .changed_return_plan
            .tr909_render
            .slam_enabled
    {
        failures.push("next bar did not return to BreakReinforce with Slam held".into());
    }
    let source_identity_preserved = prepared
        .cut_hit_return_proof
        .candidate_plan
        .source_monitor_render
        .source
        == prepared
            .cut_hit_return_proof
            .changed_return_plan
            .source_monitor_render
            .source;
    if !source_identity_preserved {
        failures.push("source audio identity changed across the cut-hit return".into());
    }
    for (role, output) in [
        (
            "slam_only_control",
            &rendered.cut_hit_return_slam_only_control,
        ),
        (
            "fill_only_attribution",
            &rendered.cut_hit_return_fill_only_control,
        ),
        ("candidate", &rendered.cut_hit_return_candidate),
        ("changed_return", &rendered.cut_hit_return_changed_return),
    ] {
        gate_exact_mix_limiter("cut-hit-return", role, &output.limiter, &mut failures);
    }

    let result = if failures.is_empty() { "pass" } else { "fail" };
    let report = json!({
        "schema": "riotbox.tr909_cut_hit_return_exact_runtime_mix.v1",
        "ticket": "RIOTBOX-1438",
        "decision": "RBX-293",
        "contract": "docs/benchmarks/tr909_cut_hit_return_development_v1.json",
        "result": result,
        "quality_proof": false,
        "human_verdict": "unverified",
        "source": {
            "source_id": prepared.live_policy.source_id.to_string(),
            "source_character": prepared.live_policy.character.label(),
            "source_hash": prepared.state.source_graph.as_ref().map(|graph| graph.source.content_hash.as_str()),
            "source_identity_preserved_on_return": source_identity_preserved,
        },
        "gesture": {
            "key": "S",
            "fill_action_id": prepared.cut_hit_return_proof.fill_action_id,
            "slam_action_id": prepared.cut_hit_return_proof.slam_action_id,
            "commit_boundary": {
                "kind": format!("{:?}", prepared.cut_hit_return_proof.commit_boundary.kind),
                "beat_index": prepared.cut_hit_return_proof.commit_boundary.beat_index,
                "bar_index": prepared.cut_hit_return_proof.commit_boundary.bar_index,
            },
            "recipe": "phrase_drive_break_cut_stomp_v2",
            "return_mode": "break_reinforce",
            "slam_held_on_return": true,
        },
        "gates": {
            "negative_space_digitally_silent": candidate_pause.iter().all(|sample| *sample == 0.0),
            "control_non_silent_in_cut_window": control_pause_metrics.rms >= MIN_CONTROL_WINDOW_RMS,
            "late_hit_above_inherited_floor": candidate_hit_metrics.rms >= MIN_HARD_RETURN_RMS,
            "candidate_distinct_from_slam_only": candidate_control_delta.active_samples > 0,
            "callback_partition_invariant": rendered.cut_hit_return_callback_partition_invariant,
            "no_limiter_activity": failures.iter().all(|failure| !failure.contains("hot exact mix")),
        },
        "metrics": {
            "candidate_pause": metrics_json(candidate_pause_metrics),
            "slam_control_same_window": metrics_json(control_pause_metrics),
            "candidate_late_hit": metrics_json(candidate_hit_metrics),
            "candidate_vs_slam_control_delta": metrics_json(candidate_control_delta),
            "candidate_vs_fill_only_delta": metrics_json(candidate_fill_delta),
            "slam_control_limiter": limiter_json(rendered.cut_hit_return_slam_only_control.limiter),
            "fill_control_limiter": limiter_json(rendered.cut_hit_return_fill_only_control.limiter),
            "candidate_limiter": limiter_json(rendered.cut_hit_return_candidate.limiter),
            "changed_return_limiter": limiter_json(rendered.cut_hit_return_changed_return.limiter),
        },
        "artifacts": artifacts,
        "failures": failures,
    });
    fs::write(
        output_dir.join("cut-hit-return/report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    if result != "pass" {
        return Err(format!(
            "cut-hit return exact RuntimeMix gate failed: {}",
            report["failures"]
        )
        .into());
    }
    Ok(())
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
        format!("cut-hit step window {start_step}..{end_step} exceeded output").into()
    })
}
