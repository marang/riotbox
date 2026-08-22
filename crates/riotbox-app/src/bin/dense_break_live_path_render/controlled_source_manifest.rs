use std::{error::Error, path::Path};

use riotbox_audio::{
    listening_manifest::{LISTENING_MANIFEST_SCHEMA_VERSION, write_manifest_json},
    runtime::{signal_delta_metrics, signal_metrics_with_grid},
    source_audio::{SourceAudioCache, write_interleaved_pcm16_wav},
};
use riotbox_core::{
    action::ActionCommand,
    live_performance_policy::{
        LIVE_PERFORMANCE_CHARACTER_CONTRAST_MARGIN, LIVE_PERFORMANCE_POLICY_SCHEMA,
        LivePerformanceCharacter,
    },
};
use serde_json::{Value, json};

use crate::{
    manifest::{gate_exact_mix_limiter, limiter_json, metrics_json},
    model::{CHANNEL_COUNT, PreparedLivePath, RenderedLivePath, SAMPLE_RATE},
};

const REVIEW_BARS: usize = 4;
const MIN_HELD_MIX_RMS: f32 = 0.01;
const MIN_AUDIBLE_LANE_RMS: f32 = 0.005;
const MAX_INTENTIONAL_STAY_OUT_RMS: f32 = 1.0e-5;
const MIN_DESTRUCTIVE_DELTA_RMS: f32 = 0.01;

pub fn write_pack(
    prepared: Box<PreparedLivePath>,
    rendered: RenderedLivePath,
    source_path: &Path,
    output_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let character_evidence = prepared
        .live_policy
        .character_evidence
        .as_ref()
        .ok_or("controlled source review omitted measured phrase-audio character evidence")?;
    let review_sample_count =
        review_frame_count(prepared.source_timing.bpm).saturating_mul(usize::from(CHANNEL_COUNT));
    let held = leading_samples(&rendered.normal.samples, review_sample_count)?;
    let destructive = leading_samples(&rendered.damaged.samples, review_sample_count)?;
    let w30 = leading_samples(&rendered.w30.samples, review_sample_count)?;
    let tr909 = leading_samples(&rendered.tr909.samples, review_sample_count)?;
    let mc202 = leading_samples(&rendered.mc202_selected_role.samples, review_sample_count)?;

    let mut artifacts = Vec::new();
    write_artifact(
        output_dir,
        "controlled/01_held_character_loop.wav",
        "held-character-loop",
        "candidate",
        held,
        &mut artifacts,
    )?;
    write_artifact(
        output_dir,
        "controlled/02_destructive_variation.wav",
        "destructive-variation",
        "candidate_variation",
        destructive,
        &mut artifacts,
    )?;
    write_artifact(
        output_dir,
        "stems/01_w30_source_hook.wav",
        "w30-source-hook",
        "w30_source_stem",
        w30,
        &mut artifacts,
    )?;
    write_artifact(
        output_dir,
        "stems/02_tr909_support.wav",
        "tr909-character-support",
        "tr909_support_stem",
        tr909,
        &mut artifacts,
    )?;
    write_artifact(
        output_dir,
        "stems/03_mc202_selected_role.wav",
        "mc202-character-role",
        "mc202_selected_role_stem",
        mc202,
        &mut artifacts,
    )?;
    let source = SourceAudioCache::load_pcm_wav(source_path)?;
    write_interleaved_pcm16_wav(
        output_dir.join("00_source.wav"),
        source.sample_rate,
        source.channel_count,
        source.interleaved_samples(),
    )?;
    artifacts.push(artifact("source", "source_reference", "00_source.wav"));

    let bpm = prepared.source_timing.bpm;
    let held_metrics = signal_metrics_with_grid(held, SAMPLE_RATE, CHANNEL_COUNT, bpm, 4);
    let destructive_metrics =
        signal_metrics_with_grid(destructive, SAMPLE_RATE, CHANNEL_COUNT, bpm, 4);
    let destructive_delta = signal_delta_metrics(held, destructive);
    let w30_metrics = signal_metrics_with_grid(w30, SAMPLE_RATE, CHANNEL_COUNT, bpm, 4);
    let tr909_metrics = signal_metrics_with_grid(tr909, SAMPLE_RATE, CHANNEL_COUNT, bpm, 4);
    let mc202_metrics = signal_metrics_with_grid(mc202, SAMPLE_RATE, CHANNEL_COUNT, bpm, 4);
    let mut failures = Vec::new();
    for (case_id, output) in [
        ("held-character-loop", &rendered.normal),
        ("destructive-variation", &rendered.damaged),
        ("w30-source-hook", &rendered.w30),
        ("tr909-character-support", &rendered.tr909),
        ("mc202-character-role", &rendered.mc202_selected_role),
    ] {
        gate_exact_mix_limiter(
            case_id,
            "controlled_source_review",
            &output.limiter,
            &mut failures,
        );
    }
    if held_metrics.rms < MIN_HELD_MIX_RMS || held_metrics.clip_count > 0 {
        failures.push(format!(
            "held character loop was silent or clipping: rms {:.6}, clips {}",
            held_metrics.rms, held_metrics.clip_count
        ));
    }
    if destructive_delta.rms < MIN_DESTRUCTIVE_DELTA_RMS || destructive_metrics.clip_count > 0 {
        failures.push(format!(
            "destructive variation collapsed: delta rms {:.6}, clips {}",
            destructive_delta.rms, destructive_metrics.clip_count
        ));
    }
    if w30_metrics.rms < MIN_AUDIBLE_LANE_RMS {
        failures.push(format!(
            "required W-30 role was inaudible: rms {:.6}",
            w30_metrics.rms
        ));
    }
    match prepared.live_policy.character {
        LivePerformanceCharacter::TonalHook => {
            if mc202_metrics.rms > MAX_INTENTIONAL_STAY_OUT_RMS {
                failures.push(format!(
                    "tonal MC-202 stay-out leaked audio: rms {:.8}",
                    mc202_metrics.rms
                ));
            }
            if tr909_metrics.rms > MAX_INTENTIONAL_STAY_OUT_RMS {
                failures.push(format!(
                    "tonal TR-909 stay-out leaked audio: rms {:.8}",
                    tr909_metrics.rms
                ));
            }
        }
        LivePerformanceCharacter::SparsePressure => {
            if tr909_metrics.rms < MIN_AUDIBLE_LANE_RMS {
                failures.push(format!(
                    "sparse TR-909 transient layer was inaudible: rms {:.6}",
                    tr909_metrics.rms
                ));
            }
            if mc202_metrics.rms < MIN_AUDIBLE_LANE_RMS {
                failures.push(format!(
                    "sparse MC-202 punctuation was inaudible: rms {:.6}",
                    mc202_metrics.rms
                ));
            }
            if tr909_metrics.peak_abs <= w30_metrics.peak_abs
                || tr909_metrics.crest_factor <= w30_metrics.crest_factor
            {
                failures.push(format!(
                    "sparse TR-909 transient layer did not lead W-30 support: tr909 peak {:.6}/crest {:.3}, w30 peak {:.6}/crest {:.3}",
                    tr909_metrics.peak_abs,
                    tr909_metrics.crest_factor,
                    w30_metrics.peak_abs,
                    w30_metrics.crest_factor,
                ));
            }
        }
        LivePerformanceCharacter::DenseBreak => {
            if tr909_metrics.rms < MIN_AUDIBLE_LANE_RMS {
                failures.push(format!(
                    "dense TR-909 pressure was inaudible: rms {:.6}",
                    tr909_metrics.rms
                ));
            }
            if mc202_metrics.rms < MIN_AUDIBLE_LANE_RMS {
                failures.push(format!(
                    "dense MC-202 instigator was inaudible: rms {:.6}",
                    mc202_metrics.rms
                ));
            }
        }
    }
    if prepared.live_policy.bass_owner.label() != "unassigned" {
        failures.push("controlled 1404 sources unexpectedly assigned bass ownership".into());
    }

    let trigger_action_id = prepared
        .stages
        .iter()
        .find(|stage| stage.command == Some(ActionCommand::W30TriggerPad))
        .and_then(|stage| stage.action_id)
        .ok_or("controlled source review omitted committed W-30 trigger")?;
    let source_descriptor = &prepared
        .state
        .source_graph
        .as_ref()
        .ok_or("controlled source review lost Source Graph")?
        .source;
    let result = if failures.is_empty() { "pass" } else { "fail" };
    let manifest = json!({
        "schema_version": LISTENING_MANIFEST_SCHEMA_VERSION,
        "pack_id": "controlled-source-live-path",
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
            "notes": "Exact callback-path held-loop and isolated-role diagnostic; human musical verdict remains required",
        },
        "product_path": "JamAppState queue/commit -> runtime projections -> exact callback-block RuntimeMix -> master limiter",
        "exact_mixer_proof": {
            "kind": "runtime_mix_callback_block_realtime_simulation",
            "held_state": true,
            "source_monitor_included": false,
            "master_limiter_included": true,
            "limiter_activity_gated": true,
        },
        "sample_rate": SAMPLE_RATE,
        "channel_count": CHANNEL_COUNT,
        "bpm": bpm,
        "review_duration_bars": REVIEW_BARS,
        "source": {
            "source_id": source_descriptor.source_id.to_string(),
            "path": source_descriptor.path,
            "content_hash": source_descriptor.content_hash,
        },
        "timing_identity": {
            "confirmed_source_id": prepared.source_timing.source_id.to_string(),
            "confirmed_hypothesis_id": prepared.source_timing.hypothesis_id,
            "confirmed_bpm": prepared.source_timing.bpm,
            "source_bar_grid_anchor_beat_cursor": prepared.live_policy.source_bar_grid_anchor_beat_cursor,
        },
        "source_character_policy": {
            "schema": LIVE_PERFORMANCE_POLICY_SCHEMA,
            "character": prepared.live_policy.character.label(),
            "destructive_intent": prepared.live_policy.destructive_intent.label(),
            "lead": prepared.live_policy.lead.label(),
            "bass_owner": prepared.live_policy.bass_owner.label(),
            "mc202_intent": prepared.live_policy.mc202_intent.label(),
            "tr909_intent": prepared.live_policy.tr909_intent.label(),
            "classification_margin": LIVE_PERFORMANCE_CHARACTER_CONTRAST_MARGIN,
            "source_evidence": {
                "feature_path": "source_graph.phrase_audio_features",
                "phrase_index": character_evidence.phrase_index,
                "spectral_brightness": character_evidence.spectral_brightness,
                "low_mid_ratio": character_evidence.low_mid_ratio,
                "offbeat_onset_density": character_evidence.offbeat_onset_density,
                "hook_restraint_hint": character_evidence.hook_restraint_hint,
                "confidence": character_evidence.confidence,
            },
            "resolved_live_defaults": {
                "w30_music_level": prepared.live_policy.w30_music_level,
                "tr909_drum_level": prepared.live_policy.tr909_drum_level,
                "tr909_slam_floor": prepared.live_policy.tr909_slam_floor,
                "tr909_pattern_adoption": prepared.live_policy.tr909_pattern_adoption.map(|value| value.label()),
                "tr909_phrase_variation": prepared.live_policy.tr909_phrase_variation.map(|value| value.label()),
                "mc202_music_level": prepared.live_policy.mc202_music_level,
                "mc202_touch_floor": prepared.live_policy.mc202_touch_floor,
                "damage_playback_rate": prepared.damaged_plan.w30_preview_render.pad_playback.as_ref().map(|window| window.playback_rate),
                "damage_gate_step_fraction": prepared.damaged_plan.w30_preview_render.pad_playback.as_ref().map(|window| window.gate_step_fraction),
            },
        },
        "activation": {
            "preset_action_id": prepared.preset_action_id,
            "w30_trigger_action_id": trigger_action_id,
            "performer_loopable_elements": true,
            "fixed_composition_claimed": false,
        },
        "role_expectations": {
            "bass": "unassigned; absent bass pressure is not a failure",
            "tonal_hook": "W-30 source hook leads alone; TR-909 and MC-202 stay out unless an explicit performer override owns them",
            "sparse_pressure": "TR-909 drum/transient impact must lead; W-30 preserves source rhythm; MC-202 only punctuates",
        },
        "metrics": {
            "held_loop": metrics_json(held_metrics),
            "held_loop_limiter": limiter_json(rendered.normal.limiter),
            "destructive_variation": metrics_json(destructive_metrics),
            "destructive_delta": metrics_json(destructive_delta),
            "destructive_limiter": limiter_json(rendered.damaged.limiter),
            "w30": metrics_json(w30_metrics),
            "tr909": metrics_json(tr909_metrics),
            "mc202": metrics_json(mc202_metrics),
        },
        "thresholds": {
            "min_held_mix_rms": MIN_HELD_MIX_RMS,
            "min_audible_lane_rms": MIN_AUDIBLE_LANE_RMS,
            "max_intentional_stay_out_rms": MAX_INTENTIONAL_STAY_OUT_RMS,
            "min_destructive_delta_rms": MIN_DESTRUCTIVE_DELTA_RMS,
            "max_exact_mix_limited_sample_count": 0,
        },
        "artifacts": artifacts,
        "failures": failures,
    });
    write_manifest_json(
        &output_dir.join("controlled-source-manifest.json"),
        &manifest,
    )?;
    prepared.state.save()?;
    println!(
        "controlled source review: result={result} character={} held_rms={:.6} w30_rms={:.6} tr909_rms={:.6} mc202_rms={:.6}",
        prepared.live_policy.character.label(),
        held_metrics.rms,
        w30_metrics.rms,
        tr909_metrics.rms,
        mc202_metrics.rms,
    );
    if result != "pass" {
        return Err(format!("controlled source live path failed: {failures:?}").into());
    }
    Ok(())
}

fn review_frame_count(bpm: f32) -> usize {
    (REVIEW_BARS as f32 * 4.0 * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize
}

fn leading_samples(samples: &[f32], sample_count: usize) -> Result<&[f32], Box<dyn Error>> {
    samples
        .get(..sample_count)
        .ok_or_else(|| "exact render was shorter than controlled review window".into())
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
