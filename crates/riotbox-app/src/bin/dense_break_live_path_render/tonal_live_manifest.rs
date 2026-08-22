use std::{error::Error, path::Path};

use riotbox_audio::{
    listening_manifest::{LISTENING_MANIFEST_SCHEMA_VERSION, write_manifest_json},
    runtime::{
        RuntimeMixRenderOutput, RuntimeMixRenderSequenceStep,
        render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report,
        signal_delta_metrics, signal_metrics_with_grid,
    },
    source_audio::{SourceAudioCache, write_interleaved_pcm16_wav},
};
use riotbox_core::live_performance_policy::{
    LIVE_PERFORMANCE_POLICY_SCHEMA, LivePerformanceCharacter,
};
use serde_json::{Value, json};

use crate::{
    manifest::{gate_exact_mix_limiter, limiter_json, metrics_json},
    model::{CHANNEL_COUNT, PreparedLivePath, SAMPLE_RATE, TONAL_PITCH_DIVE_ACTIVE_BEATS},
    rendering::{only_mc202, only_tr909, only_w30, render},
};

const HELD_BEATS: u32 = 16;
const CONTRAST_BEATS: u32 = TONAL_PITCH_DIVE_ACTIVE_BEATS;
const REENTRY_BEATS: u32 = 8;
const RESTART_BEATS: u32 = 8;
const MIN_MIX_RMS: f32 = 0.01;
const MIN_LANE_RMS: f32 = 0.005;
const MAX_STAY_OUT_RMS: f32 = 1.0e-5;
const MIN_CONTRAST_DELTA_RMS: f32 = 0.005;
const MIN_PITCH_DIVE_ACTIVE_TAIL_DELTA_RMS: f32 = 0.001;

pub fn write_pack(
    mut prepared: Box<PreparedLivePath>,
    source_path: &Path,
    output_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    if prepared.live_policy.character != LivePerformanceCharacter::TonalHook {
        return Err(format!(
            "tonal live review requires tonal_hook policy, got {}",
            prepared.live_policy.character.label()
        )
        .into());
    }
    let journey = prepared
        .tonal_journey
        .take()
        .ok_or("tonal live review omitted its performer journey")?;
    journey.state.save()?;

    let bpm = prepared.source_timing.bpm;
    let plans = [
        journey.held_plan.as_ref(),
        journey.contrast_plan.as_ref(),
        journey.reentry_plan.as_ref(),
    ];
    let beats = [HELD_BEATS, CONTRAST_BEATS, REENTRY_BEATS];
    let callback_128 = render_sequence(&plans, beats, bpm, 128);
    let callback_257 = render_sequence(&plans, beats, bpm, 257);
    let callback_partition_sample_exact = outputs_sample_exact(&callback_128, &callback_257);
    if callback_128.len() != 3 {
        return Err("tonal journey did not render exactly three stages".into());
    }
    let held = &callback_128[0];
    let contrast = &callback_128[1];
    let reentry = &callback_128[2];
    let restart = render(
        &journey.restart_recall_plan,
        frames_for_beats(bpm, RESTART_BEATS),
    )?;
    let source = SourceAudioCache::load_pcm_wav(source_path)?;
    let source_context_frame_count = (source.frame_count() as f64 * f64::from(SAMPLE_RATE)
        / f64::from(source.sample_rate.max(1)))
    .round() as usize;
    let source_context_plan = prepared
        .monitor_proofs
        .first()
        .ok_or("tonal live review omitted source monitor context")?;
    let source_context = render(&source_context_plan.plan, source_context_frame_count)?;
    let w30 = render(
        &only_w30(&journey.held_plan),
        frames_for_beats(bpm, HELD_BEATS),
    )?;
    let tr909 = render(
        &only_tr909(&journey.held_plan),
        frames_for_beats(bpm, HELD_BEATS),
    )?;
    let mc202 = render(
        &only_mc202(&journey.held_plan),
        frames_for_beats(bpm, HELD_BEATS),
    )?;
    let w30_contrast = render(
        &only_w30(&journey.contrast_plan),
        frames_for_beats(bpm, CONTRAST_BEATS),
    )?;

    let held_metrics = signal_metrics_with_grid(&held.samples, SAMPLE_RATE, CHANNEL_COUNT, bpm, 4);
    let contrast_metrics =
        signal_metrics_with_grid(&contrast.samples, SAMPLE_RATE, CHANNEL_COUNT, bpm, 4);
    let contrast_delta = signal_delta_metrics(&held.samples, &contrast.samples);
    let reentry_metrics =
        signal_metrics_with_grid(&reentry.samples, SAMPLE_RATE, CHANNEL_COUNT, bpm, 4);
    let restart_metrics =
        signal_metrics_with_grid(&restart.samples, SAMPLE_RATE, CHANNEL_COUNT, bpm, 4);
    let w30_metrics = signal_metrics_with_grid(&w30.samples, SAMPLE_RATE, CHANNEL_COUNT, bpm, 4);
    let tr909_metrics =
        signal_metrics_with_grid(&tr909.samples, SAMPLE_RATE, CHANNEL_COUNT, bpm, 4);
    let mc202_metrics =
        signal_metrics_with_grid(&mc202.samples, SAMPLE_RATE, CHANNEL_COUNT, bpm, 4);
    let first_half_sample_count = frames_for_beats(bpm, 8) * usize::from(CHANNEL_COUNT);
    let active_tail_sample_count =
        frames_for_beats(bpm, CONTRAST_BEATS - 8) * usize::from(CHANNEL_COUNT);
    let w30_pitch_dive_first_half_sample_exact =
        w30.samples[..first_half_sample_count] == w30_contrast.samples[..first_half_sample_count];
    let w30_pitch_dive_active_tail_delta = signal_delta_metrics(
        &w30.samples[first_half_sample_count..first_half_sample_count + active_tail_sample_count],
        &w30_contrast.samples[first_half_sample_count..],
    );

    let mut failures = Vec::new();
    for (case_id, output) in [
        ("held-tonal-hook", held),
        ("pitch-dive-contrast", contrast),
        ("ordinary-reentry", reentry),
        ("restart-recall-trigger", &restart),
        ("w30-tonal-lead", &w30),
        ("tr909-tonal-support", &tr909),
        ("mc202-stay-out", &mc202),
        ("w30-pitch-dive", &w30_contrast),
        ("source-context", &source_context),
    ] {
        gate_exact_mix_limiter(case_id, "tonal_live_review", &output.limiter, &mut failures);
    }
    if !callback_partition_sample_exact {
        failures.push("128- and 257-frame callback partitions diverged".into());
    }
    if held_metrics.rms < MIN_MIX_RMS
        || reentry_metrics.rms < MIN_MIX_RMS
        || restart_metrics.rms < MIN_MIX_RMS
    {
        failures.push(format!(
            "held/re-entry/restart mix was too quiet: {:.6}/{:.6}/{:.6}",
            held_metrics.rms, reentry_metrics.rms, restart_metrics.rms
        ));
    }
    if contrast_delta.rms < MIN_CONTRAST_DELTA_RMS {
        failures.push(format!(
            "Pitch Dive contrast collapsed against held tonal hook: delta rms {:.6}",
            contrast_delta.rms
        ));
    }
    if !w30_pitch_dive_first_half_sample_exact {
        failures.push("Pitch Dive changed the isolated W-30 before its eight-beat onset".into());
    }
    if w30_pitch_dive_active_tail_delta.rms < MIN_PITCH_DIVE_ACTIVE_TAIL_DELTA_RMS {
        failures.push(format!(
            "Pitch Dive isolated active-tail delta collapsed: rms {:.6}",
            w30_pitch_dive_active_tail_delta.rms
        ));
    }
    if w30_metrics.rms < MIN_LANE_RMS {
        failures.push(format!(
            "tonal W-30 hook was inaudible: rms {:.6}",
            w30_metrics.rms
        ));
    }
    if tr909_metrics.rms > MAX_STAY_OUT_RMS {
        failures.push(format!(
            "tonal TR-909 stay-out leaked audio: rms {:.8}",
            tr909_metrics.rms
        ));
    }
    if mc202_metrics.rms > MAX_STAY_OUT_RMS {
        failures.push(format!(
            "tonal MC-202 stay-out leaked audio: rms {:.8}",
            mc202_metrics.rms
        ));
    }
    if !journey.proof.ordinary_reentry_cleared_articulation {
        failures.push("ordinary re-entry retained the Pitch Dive articulation".into());
    }
    if !journey.restart_recall_proof.preset_survived_restart {
        failures.push("performance preset did not survive restart".into());
    }

    let mut artifacts = Vec::new();
    write_artifact(
        output_dir,
        "tonal/00_source_context.wav",
        "source-context",
        "source_reference_resampled",
        &source_context.samples,
        &mut artifacts,
    )?;
    write_artifact(
        output_dir,
        "tonal/01_held_hook.wav",
        "held-tonal-hook",
        "candidate",
        &held.samples,
        &mut artifacts,
    )?;
    write_artifact(
        output_dir,
        "stems/04_w30_pitch_dive.wav",
        "w30-pitch-dive",
        "w30_contrast_stem",
        &w30_contrast.samples,
        &mut artifacts,
    )?;
    write_artifact(
        output_dir,
        "tonal/02_pitch_dive.wav",
        "pitch-dive-contrast",
        "candidate_variation",
        &contrast.samples,
        &mut artifacts,
    )?;
    write_artifact(
        output_dir,
        "tonal/03_ordinary_reentry.wav",
        "ordinary-reentry",
        "candidate_reentry",
        &reentry.samples,
        &mut artifacts,
    )?;
    write_artifact(
        output_dir,
        "tonal/04_restart_recall.wav",
        "restart-recall-trigger",
        "candidate_restart_recall",
        &restart.samples,
        &mut artifacts,
    )?;
    write_artifact(
        output_dir,
        "stems/01_w30_source_hook.wav",
        "w30-tonal-lead",
        "w30_source_stem",
        &w30.samples,
        &mut artifacts,
    )?;
    write_artifact(
        output_dir,
        "stems/02_tr909_support.wav",
        "tr909-tonal-support",
        "tr909_support_stem",
        &tr909.samples,
        &mut artifacts,
    )?;
    write_artifact(
        output_dir,
        "stems/03_mc202_stay_out.wav",
        "mc202-stay-out",
        "mc202_selected_role_stem",
        &mc202.samples,
        &mut artifacts,
    )?;
    let live_journey = callback_128
        .iter()
        .chain(std::iter::once(&restart))
        .flat_map(|output| output.samples.iter().copied())
        .collect::<Vec<_>>();
    write_artifact(
        output_dir,
        "tonal/05_live_journey.wav",
        "tonal-live-journey",
        "candidate_sequence",
        &live_journey,
        &mut artifacts,
    )?;
    let one_second_silence = vec![0.0; SAMPLE_RATE as usize * usize::from(CHANNEL_COUNT)];
    let human_review_sequence = [
        source_context.samples.as_slice(),
        one_second_silence.as_slice(),
        held.samples.as_slice(),
        one_second_silence.as_slice(),
        contrast.samples.as_slice(),
        reentry.samples.as_slice(),
        one_second_silence.as_slice(),
        restart.samples.as_slice(),
        one_second_silence.as_slice(),
    ]
    .concat();
    write_artifact(
        output_dir,
        "tonal/06_human_review_sequence.wav",
        "tonal-human-review-sequence",
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

    let source_descriptor = &journey
        .state
        .source_graph
        .as_ref()
        .ok_or("tonal journey lost Source Graph")?
        .source;
    let result = if failures.is_empty() { "pass" } else { "fail" };
    let manifest = json!({
        "schema_version": LISTENING_MANIFEST_SCHEMA_VERSION,
        "pack_id": "tonal-hook-live-journey",
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
            "notes": "Exact Development-only tonal live journey; musical fitness remains a human decision"
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
        "journey": {
            "held_beats": HELD_BEATS,
            "contrast": "w30.pitch_dive",
            "contrast_beats": CONTRAST_BEATS,
            "contrast_action_id": journey.proof.contrast_action_id,
            "ordinary_reentry_action_id": journey.proof.reentry_action_id,
            "ordinary_reentry_beats": REENTRY_BEATS,
            "ordinary_reentry_cleared_articulation": journey.proof.ordinary_reentry_cleared_articulation,
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
            "lead": prepared.live_policy.lead.label(),
            "bass_owner": prepared.live_policy.bass_owner.label(),
            "tr909_intent": prepared.live_policy.tr909_intent.label(),
            "mc202_intent": prepared.live_policy.mc202_intent.label(),
        },
        "exact_mixer_proof": {
            "kind": "runtime_mix_callback_block_realtime_simulation",
            "callback_partitions_sample_exact": callback_partition_sample_exact,
            "source_monitor_included": false,
            "master_limiter_included": true,
            "limiter_activity_gated": true,
            "pitch_dive_first_eight_beats_sample_exact_to_held_w30": w30_pitch_dive_first_half_sample_exact,
            "pitch_dive_active_tail_delta_rms": w30_pitch_dive_active_tail_delta.rms,
            "human_review_sequence_duration_seconds": human_review_sequence.len() as f64 / f64::from(CHANNEL_COUNT) / f64::from(SAMPLE_RATE),
        },
        "metrics": {
            "held": metrics_json(held_metrics),
            "held_limiter": limiter_json(held.limiter),
            "pitch_dive": metrics_json(contrast_metrics),
            "pitch_dive_delta": metrics_json(contrast_delta),
            "pitch_dive_limiter": limiter_json(contrast.limiter),
            "ordinary_reentry": metrics_json(reentry_metrics),
            "ordinary_reentry_limiter": limiter_json(reentry.limiter),
            "restart_recall": metrics_json(restart_metrics),
            "restart_recall_limiter": limiter_json(restart.limiter),
            "w30": metrics_json(w30_metrics),
            "tr909": metrics_json(tr909_metrics),
            "mc202": metrics_json(mc202_metrics),
            "w30_pitch_dive_active_tail_delta": metrics_json(w30_pitch_dive_active_tail_delta),
        },
        "thresholds": {
            "min_mix_rms": MIN_MIX_RMS,
            "min_lane_rms": MIN_LANE_RMS,
            "max_mc202_stay_out_rms": MAX_STAY_OUT_RMS,
            "max_tr909_stay_out_rms": MAX_STAY_OUT_RMS,
            "min_pitch_dive_delta_rms": MIN_CONTRAST_DELTA_RMS,
            "min_isolated_pitch_dive_active_tail_delta_rms": MIN_PITCH_DIVE_ACTIVE_TAIL_DELTA_RMS,
            "max_limited_sample_count": 0,
        },
        "artifacts": artifacts,
        "failures": failures,
    });
    write_manifest_json(&output_dir.join("tonal-live-manifest.json"), &manifest)?;
    println!(
        "tonal live journey: result={result} held_rms={:.6} pitch_delta_rms={:.6} reentry_rms={:.6} restart_rms={:.6}",
        held_metrics.rms, contrast_delta.rms, reentry_metrics.rms, restart_metrics.rms
    );
    if result != "pass" {
        return Err(format!("tonal live journey failed: {failures:?}").into());
    }
    Ok(())
}

fn render_sequence(
    plans: &[&riotbox_audio::runtime::RuntimeMixRenderPlan; 3],
    beats: [u32; 3],
    bpm: f32,
    callback_frames: usize,
) -> Vec<RuntimeMixRenderOutput> {
    let steps = plans
        .iter()
        .zip(beats)
        .map(|(plan, beat_count)| {
            RuntimeMixRenderSequenceStep::new(plan, frames_for_beats(bpm, beat_count))
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
    fn tonal_review_lengths_are_exact_at_120_bpm() {
        assert_eq!(frames_for_beats(120.0, HELD_BEATS), 384_000);
        assert_eq!(frames_for_beats(120.0, REENTRY_BEATS), 192_000);
    }
}
