use std::{error::Error, path::Path};

use riotbox_audio::{
    listening_manifest::{LISTENING_MANIFEST_SCHEMA_VERSION, write_manifest_json},
    runtime::{
        RuntimeMixRenderOutput, RuntimeMixRenderPlan, RuntimeMixRenderSequenceStep,
        render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report,
        signal_delta_metrics, signal_metrics_with_grid,
    },
    source_audio::{SourceAudioCache, write_interleaved_pcm16_wav},
};
use riotbox_core::{
    live_performance_policy::{LIVE_PERFORMANCE_POLICY_SCHEMA, LivePerformanceCharacter},
    w30_damage_policy::{
        W30_DAMAGE_PROFILE_ACTIVE_INTENSITY, W30_TRANSIENT_BITE_GATE_STEP_FRACTION,
    },
};
use serde_json::{Value, json};

use crate::{
    manifest::{gate_exact_mix_limiter, limiter_json, metrics_json},
    model::{CHANNEL_COUNT, PreparedLivePath, SAMPLE_RATE},
    rendering::{only_mc202, only_source_monitor, only_tr909, only_w30, render},
};

const HELD_BEATS: u32 = 16;
const DAMAGE_BEATS: u32 = 16;
const REENTRY_BEATS: u32 = 8;
const RESTART_BEATS: u32 = 8;
const MIN_MIX_RMS: f32 = 0.01;
const MIN_LANE_RMS: f32 = 0.005;
const MAX_STAY_OUT_RMS: f32 = 1.0e-5;
const MIN_DAMAGE_DELTA_RMS: f32 = 0.005;
const GATE_EPSILON: f32 = 1.0e-6;

pub fn write_pack(
    mut prepared: Box<PreparedLivePath>,
    source_path: &Path,
    output_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    if prepared.live_policy.character != LivePerformanceCharacter::SparsePressure {
        return Err(format!(
            "sparse live review requires sparse_pressure policy, got {}",
            prepared.live_policy.character.label()
        )
        .into());
    }
    let journey = prepared
        .sparse_journey
        .take()
        .ok_or("sparse live review omitted its performer journey")?;
    let bpm = prepared.source_timing.bpm;
    let plans = [
        journey.held_plan.as_ref(),
        journey.damage_plan.as_ref(),
        journey.reentry_plan.as_ref(),
    ];
    let beats = [HELD_BEATS, DAMAGE_BEATS, REENTRY_BEATS];
    let callback_128 = render_sequence(&plans, &beats, bpm, 128);
    let callback_257 = render_sequence(&plans, &beats, bpm, 257);
    if callback_128.len() != 3 {
        return Err("sparse journey did not render exactly three live stages".into());
    }
    let callback_partition_sample_exact = outputs_sample_exact(&callback_128, &callback_257);
    let held = &callback_128[0];
    let damage = &callback_128[1];
    let reentry = &callback_128[2];
    let restart = render(
        &journey.restart_recall_plan,
        frames_for_beats(bpm, RESTART_BEATS),
    )?;

    let w30 = render_isolated_sequence(&plans, &beats, bpm, only_w30);
    let tr909 = render_isolated_sequence(&plans, &beats, bpm, only_tr909);
    let mc202 = render_isolated_sequence(&plans, &beats, bpm, only_mc202);
    let source_monitor = render_isolated_sequence(&plans, &beats, bpm, only_source_monitor);
    let restart_w30 = render(
        &only_w30(&journey.restart_recall_plan),
        frames_for_beats(bpm, RESTART_BEATS),
    )?;
    let restart_tr909 = render(
        &only_tr909(&journey.restart_recall_plan),
        frames_for_beats(bpm, RESTART_BEATS),
    )?;
    let restart_mc202 = render(
        &only_mc202(&journey.restart_recall_plan),
        frames_for_beats(bpm, RESTART_BEATS),
    )?;
    let restart_source_monitor = render(
        &only_source_monitor(&journey.restart_recall_plan),
        frames_for_beats(bpm, RESTART_BEATS),
    )?;

    let held_metrics = metrics(held, bpm);
    let damage_metrics = metrics(damage, bpm);
    let reentry_metrics = metrics(reentry, bpm);
    let restart_metrics = metrics(&restart, bpm);
    let w30_held_metrics = metrics(&w30[0], bpm);
    let w30_damage_metrics = metrics(&w30[1], bpm);
    let w30_reentry_metrics = metrics(&w30[2], bpm);
    let tr909_held_metrics = metrics(&tr909[0], bpm);
    let mc202_held_metrics = metrics(&mc202[0], bpm);
    let mix_damage_delta = signal_delta_metrics(&held.samples, &damage.samples);
    let w30_damage_delta = signal_delta_metrics(&w30[0].samples, &w30[1].samples);
    let w30_reentry_delta = signal_delta_metrics(&w30[1].samples, &w30[2].samples);
    let expected_gate = W30_DAMAGE_PROFILE_ACTIVE_INTENSITY * W30_TRANSIENT_BITE_GATE_STEP_FRACTION;

    let mut failures = Vec::new();
    for (case_id, output) in [
        ("held-sparse-pressure", held),
        ("transient-bite-damage", damage),
        ("ordinary-reentry", reentry),
        ("restart-recall-trigger", &restart),
        ("w30-held", &w30[0]),
        ("w30-damage", &w30[1]),
        ("w30-reentry", &w30[2]),
        ("tr909-held", &tr909[0]),
        ("mc202-held", &mc202[0]),
    ] {
        gate_exact_mix_limiter(
            case_id,
            "sparse_live_review",
            &output.limiter,
            &mut failures,
        );
    }
    if !callback_partition_sample_exact {
        failures.push("128- and 257-frame callback partitions diverged".into());
    }
    for (stage, rms) in [
        ("held", held_metrics.rms),
        ("damage", damage_metrics.rms),
        ("re-entry", reentry_metrics.rms),
        ("restart", restart_metrics.rms),
    ] {
        if rms < MIN_MIX_RMS {
            failures.push(format!("sparse {stage} mix was too quiet: rms {rms:.6}"));
        }
    }
    for (lane, outputs) in [("w30", &w30), ("tr909", &tr909), ("mc202", &mc202)] {
        let min_rms = minimum_stage_rms(outputs, bpm);
        if min_rms < MIN_LANE_RMS {
            failures.push(format!(
                "required sparse {lane} lane collapsed: min stage rms {min_rms:.6}"
            ));
        }
    }
    for (lane, output) in [
        ("w30", &restart_w30),
        ("tr909", &restart_tr909),
        ("mc202", &restart_mc202),
    ] {
        let rms = metrics(output, bpm).rms;
        if rms < MIN_LANE_RMS {
            failures.push(format!(
                "required sparse restart {lane} lane collapsed: rms {rms:.6}"
            ));
        }
    }
    let source_monitor_max_rms =
        maximum_stage_rms(&source_monitor, bpm).max(metrics(&restart_source_monitor, bpm).rms);
    if source_monitor_max_rms > MAX_STAY_OUT_RMS {
        failures.push(format!(
            "source monitor leaked into sparse journey: max rms {source_monitor_max_rms:.8}"
        ));
    }
    if mix_damage_delta.rms < MIN_DAMAGE_DELTA_RMS
        || w30_damage_delta.rms < MIN_DAMAGE_DELTA_RMS
        || w30_reentry_delta.rms < MIN_DAMAGE_DELTA_RMS
    {
        failures.push(format!(
            "sparse damage/re-entry contrast collapsed: mix damage {:.6}, isolated damage {:.6}, isolated re-entry {:.6}",
            mix_damage_delta.rms, w30_damage_delta.rms, w30_reentry_delta.rms
        ));
    }
    if tr909_held_metrics.peak_abs <= w30_held_metrics.peak_abs
        || tr909_held_metrics.crest_factor <= w30_held_metrics.crest_factor
    {
        failures.push(format!(
            "sparse TR-909 did not own the hardest transient: tr909 peak {:.6}/crest {:.3}, w30 peak {:.6}/crest {:.3}",
            tr909_held_metrics.peak_abs,
            tr909_held_metrics.crest_factor,
            w30_held_metrics.peak_abs,
            w30_held_metrics.crest_factor,
        ));
    }
    if (journey.proof.damage_gate_step_fraction - expected_gate).abs() > GATE_EPSILON
        || journey.proof.reentry_gate_step_fraction != 0.0
        || restart_gate_step_fraction(&journey.restart_recall_plan)? != 0.0
    {
        failures.push(format!(
            "damage gate state diverged: damage {:.6} expected {:.6}, re-entry {:.6}, restart {:.6}",
            journey.proof.damage_gate_step_fraction,
            expected_gate,
            journey.proof.reentry_gate_step_fraction,
            restart_gate_step_fraction(&journey.restart_recall_plan)?,
        ));
    }
    if journey.proof.damage_intensity != W30_DAMAGE_PROFILE_ACTIVE_INTENSITY
        || journey.proof.bypass_intensity != 0.0
    {
        failures.push(format!(
            "damage action intensities diverged: apply {:.6}, bypass {:.6}",
            journey.proof.damage_intensity, journey.proof.bypass_intensity
        ));
    }
    if !journey.restart_recall_proof.preset_survived_restart {
        failures.push("performance preset did not survive restart".into());
    }

    let source = SourceAudioCache::load_pcm_wav(source_path)?;
    let source_context_frame_count = (source.frame_count() as f64 * f64::from(SAMPLE_RATE)
        / f64::from(source.sample_rate.max(1)))
    .round() as usize;
    let source_context_plan = prepared
        .monitor_proofs
        .first()
        .ok_or("sparse live review omitted source context")?;
    let source_context = render(&source_context_plan.plan, source_context_frame_count)?;
    gate_exact_mix_limiter(
        "source-context",
        "sparse_live_review",
        &source_context.limiter,
        &mut failures,
    );

    let mut artifacts = Vec::new();
    write_artifact(
        output_dir,
        "sparse/00_source_context.wav",
        "source-context",
        "source_reference_resampled",
        &source_context.samples,
        &mut artifacts,
    )?;
    for (path, case_id, role, output) in [
        (
            "sparse/01_held.wav",
            "held-sparse-pressure",
            "candidate",
            held,
        ),
        (
            "sparse/02_transient_bite.wav",
            "transient-bite-damage",
            "candidate_variation",
            damage,
        ),
        (
            "sparse/03_ordinary_reentry.wav",
            "ordinary-reentry",
            "candidate_reentry",
            reentry,
        ),
        (
            "sparse/04_restart_recall.wav",
            "restart-recall-trigger",
            "candidate_restart_recall",
            &restart,
        ),
    ] {
        write_artifact(
            output_dir,
            path,
            case_id,
            role,
            &output.samples,
            &mut artifacts,
        )?;
    }
    for (path, case_id, role, output) in [
        (
            "stems/01_w30_held.wav",
            "w30-held",
            "w30_source_stem",
            &w30[0],
        ),
        (
            "stems/02_tr909_held.wav",
            "tr909-held",
            "tr909_transient_stem",
            &tr909[0],
        ),
        (
            "stems/03_mc202_held.wav",
            "mc202-held",
            "mc202_punctuation_stem",
            &mc202[0],
        ),
        (
            "stems/04_w30_damage.wav",
            "w30-damage",
            "w30_damage_stem",
            &w30[1],
        ),
    ] {
        write_artifact(
            output_dir,
            path,
            case_id,
            role,
            &output.samples,
            &mut artifacts,
        )?;
    }
    let live_journey = callback_128
        .iter()
        .chain(std::iter::once(&restart))
        .flat_map(|output| output.samples.iter().copied())
        .collect::<Vec<_>>();
    write_artifact(
        output_dir,
        "sparse/05_live_journey.wav",
        "sparse-live-journey",
        "candidate_sequence",
        &live_journey,
        &mut artifacts,
    )?;
    let silence = vec![0.0; SAMPLE_RATE as usize * usize::from(CHANNEL_COUNT)];
    let human_review_sequence = [
        source_context.samples.as_slice(),
        silence.as_slice(),
        held.samples.as_slice(),
        silence.as_slice(),
        damage.samples.as_slice(),
        reentry.samples.as_slice(),
        restart.samples.as_slice(),
        silence.as_slice(),
    ]
    .concat();
    write_artifact(
        output_dir,
        "sparse/06_human_review_sequence.wav",
        "sparse-human-review-sequence",
        "human_review_sequence",
        &human_review_sequence,
        &mut artifacts,
    )?;
    write_interleaved_pcm16_wav(
        output_dir.join("00_source.wav"),
        source.sample_rate,
        source.channel_count,
        source.interleaved_samples(),
    )?;
    artifacts.push(artifact("source", "source_reference", "00_source.wav"));

    let source_descriptor = &prepared
        .state
        .source_graph
        .as_ref()
        .ok_or("sparse journey lost Source Graph")?
        .source;
    let result = if failures.is_empty() { "pass" } else { "fail" };
    let manifest = json!({
        "schema_version": LISTENING_MANIFEST_SCHEMA_VERSION,
        "pack_id": "sparse-pressure-live-journey",
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
            "notes": "Exact Development-only sparse live journey; musical fitness remains a human decision"
        },
        "product_path": "JamAppState queue/commit -> Session/runtime projection -> exact callback-block RuntimeMix -> master limiter",
        "sample_rate": SAMPLE_RATE,
        "channel_count": CHANNEL_COUNT,
        "bpm": bpm,
        "source": {
            "source_id": source_descriptor.source_id.to_string(),
            "path": source_descriptor.path,
            "content_hash": source_descriptor.content_hash,
        },
        "timing_identity": {
            "confirmed_source_id": prepared.source_timing.source_id.to_string(),
            "confirmed_hypothesis_id": prepared.source_timing.hypothesis_id,
            "manual_bpm": prepared.source_timing.bpm,
            "manual_downbeat_seconds": 0.0,
        },
        "capture_journey": {
            "capture_action_id": prepared.capture_journey_proof.capture_action_id,
            "raw_audition_action_id": prepared.capture_journey_proof.raw_audition_action_id,
            "promotion_action_id": prepared.capture_journey_proof.promotion_action_id,
        },
        "journey": {
            "held_beats": HELD_BEATS,
            "damage_command": "w30.apply_damage_profile",
            "damage_action_id": journey.proof.damage_action_id,
            "damage_intensity": journey.proof.damage_intensity,
            "damage_beats": DAMAGE_BEATS,
            "bypass_action_id": journey.proof.bypass_action_id,
            "bypass_intensity": journey.proof.bypass_intensity,
            "ordinary_reentry_beats": REENTRY_BEATS,
            "saved_before_restart": true,
            "restart_preset_survived": journey.restart_recall_proof.preset_survived_restart,
            "restart_capture_id": journey.restart_recall_proof.capture_id,
            "restart_recall_action_id": journey.restart_recall_proof.recall_action_id,
            "restart_trigger_action_id": journey.restart_recall_proof.trigger_action_id,
            "restart_review_beats": RESTART_BEATS,
        },
        "source_character_policy": {
            "schema": LIVE_PERFORMANCE_POLICY_SCHEMA,
            "character": prepared.live_policy.character.label(),
            "destructive_intent": prepared.live_policy.destructive_intent.label(),
            "lead": prepared.live_policy.lead.label(),
            "bass_owner": prepared.live_policy.bass_owner.label(),
            "tr909_intent": prepared.live_policy.tr909_intent.label(),
            "mc202_intent": prepared.live_policy.mc202_intent.label(),
        },
        "exact_mixer_proof": {
            "kind": "runtime_mix_callback_block_realtime_simulation",
            "callback_partitions_sample_exact": callback_partition_sample_exact,
            "source_monitor_included": false,
            "source_monitor_max_rms": source_monitor_max_rms,
            "master_limiter_included": true,
            "limiter_activity_gated": true,
            "damage_gate_step_fraction": journey.proof.damage_gate_step_fraction,
            "expected_damage_gate_step_fraction": expected_gate,
            "reentry_gate_step_fraction": journey.proof.reentry_gate_step_fraction,
            "restart_gate_step_fraction": restart_gate_step_fraction(&journey.restart_recall_plan)?,
            "human_review_sequence_duration_seconds": human_review_sequence.len() as f64 / f64::from(CHANNEL_COUNT) / f64::from(SAMPLE_RATE),
        },
        "metrics": {
            "held": metrics_json(held_metrics),
            "held_limiter": limiter_json(held.limiter),
            "damage": metrics_json(damage_metrics),
            "damage_delta": metrics_json(mix_damage_delta),
            "damage_limiter": limiter_json(damage.limiter),
            "ordinary_reentry": metrics_json(reentry_metrics),
            "ordinary_reentry_limiter": limiter_json(reentry.limiter),
            "restart_recall": metrics_json(restart_metrics),
            "restart_recall_limiter": limiter_json(restart.limiter),
            "w30_held": metrics_json(w30_held_metrics),
            "w30_damage": metrics_json(w30_damage_metrics),
            "w30_reentry": metrics_json(w30_reentry_metrics),
            "w30_damage_delta": metrics_json(w30_damage_delta),
            "w30_reentry_delta": metrics_json(w30_reentry_delta),
            "tr909_held": metrics_json(tr909_held_metrics),
            "mc202_held": metrics_json(mc202_held_metrics),
        },
        "thresholds": {
            "min_mix_rms": MIN_MIX_RMS,
            "min_lane_rms": MIN_LANE_RMS,
            "max_source_monitor_rms": MAX_STAY_OUT_RMS,
            "min_damage_delta_rms": MIN_DAMAGE_DELTA_RMS,
            "max_limited_sample_count": 0,
        },
        "artifacts": artifacts,
        "failures": failures,
    });
    write_manifest_json(&output_dir.join("sparse-live-manifest.json"), &manifest)?;
    println!(
        "sparse live journey: result={result} held_rms={:.6} damage_delta_rms={:.6} reentry_rms={:.6} restart_rms={:.6}",
        held_metrics.rms, mix_damage_delta.rms, reentry_metrics.rms, restart_metrics.rms
    );
    if result != "pass" {
        return Err(format!("sparse live journey failed: {failures:?}").into());
    }
    Ok(())
}

fn render_isolated_sequence(
    plans: &[&RuntimeMixRenderPlan; 3],
    beats: &[u32; 3],
    bpm: f32,
    isolate: fn(&RuntimeMixRenderPlan) -> RuntimeMixRenderPlan,
) -> Vec<RuntimeMixRenderOutput> {
    let isolated = plans.map(isolate);
    let refs = isolated.each_ref();
    render_sequence(&refs, beats, bpm, 128)
}

fn render_sequence(
    plans: &[&RuntimeMixRenderPlan; 3],
    beats: &[u32; 3],
    bpm: f32,
    callback_frames: usize,
) -> Vec<RuntimeMixRenderOutput> {
    let steps = plans
        .iter()
        .zip(beats)
        .map(|(plan, beat_count)| {
            RuntimeMixRenderSequenceStep::new(plan, frames_for_beats(bpm, *beat_count))
        })
        .collect::<Vec<_>>();
    render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &steps,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        callback_frames,
    )
}

fn frames_for_beats(bpm: f32, beats: u32) -> usize {
    (beats as f32 * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize
}

fn outputs_sample_exact(
    first: &[RuntimeMixRenderOutput],
    second: &[RuntimeMixRenderOutput],
) -> bool {
    first.len() == second.len()
        && first
            .iter()
            .zip(second)
            .all(|(left, right)| left.samples == right.samples)
}

fn metrics(
    output: &RuntimeMixRenderOutput,
    bpm: f32,
) -> riotbox_audio::runtime::OfflineAudioMetrics {
    signal_metrics_with_grid(&output.samples, SAMPLE_RATE, CHANNEL_COUNT, bpm, 4)
}

fn minimum_stage_rms(outputs: &[RuntimeMixRenderOutput], bpm: f32) -> f32 {
    outputs
        .iter()
        .map(|output| metrics(output, bpm).rms)
        .fold(f32::INFINITY, f32::min)
}

fn maximum_stage_rms(outputs: &[RuntimeMixRenderOutput], bpm: f32) -> f32 {
    outputs
        .iter()
        .map(|output| metrics(output, bpm).rms)
        .fold(0.0, f32::max)
}

fn restart_gate_step_fraction(plan: &RuntimeMixRenderPlan) -> Result<f32, Box<dyn Error>> {
    plan.w30_preview_render
        .pad_playback
        .as_ref()
        .map(|playback| playback.gate_step_fraction)
        .ok_or_else(|| "sparse restart lost W-30 pad playback".into())
}

fn write_artifact(
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
    artifacts.push(artifact(case_id, role, relative_path));
    Ok(())
}

fn artifact(case_id: &str, role: &str, path: &str) -> Value {
    json!({
        "case_id": case_id,
        "role": role,
        "kind": "audio_wav",
        "path": path,
        "metrics_path": null,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_review_lengths_are_exact_at_120_bpm() {
        assert_eq!(frames_for_beats(120.0, HELD_BEATS), 384_000);
        assert_eq!(frames_for_beats(120.0, REENTRY_BEATS), 192_000);
    }

    #[test]
    fn frozen_transient_bite_gate_is_exact() {
        assert!(
            (W30_DAMAGE_PROFILE_ACTIVE_INTENSITY * W30_TRANSIENT_BITE_GATE_STEP_FRACTION - 0.3608)
                .abs()
                < GATE_EPSILON
        );
    }
}
