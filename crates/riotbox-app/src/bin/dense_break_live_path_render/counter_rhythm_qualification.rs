use std::{error::Error, fs, path::Path};

use riotbox_audio::{
    runtime::{
        RuntimeMixRenderOutput, RuntimeMixRenderSequenceStep,
        render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report,
        signal_delta_metrics, signal_metrics,
    },
    source_audio::write_interleaved_pcm16_wav,
    tr909::{Tr909CounterRhythmPhase, Tr909PatternAdoption, Tr909RenderMode, Tr909RenderRouting},
};
use riotbox_core::transport::TransportClockState;
use serde_json::json;

use crate::{
    live_flow,
    model::{CALLBACK_FRAME_COUNT, CHANNEL_COUNT, PreparedLivePath, SAMPLE_RATE},
};

const REVIEW_BARS: usize = 4;
const MIN_TIME_LOCAL_DELTA_RMS: f32 = 0.02;

pub fn write_case(
    prepared: PreparedLivePath,
    output_dir: &Path,
    write_wavs: bool,
) -> Result<(), Box<dyn Error>> {
    prepared
        .stages
        .iter()
        .find(|stage| stage.case_id == "after-s-slam")
        .ok_or("counter-rhythm qualification could not find committed Slam stage")?;
    let blend = prepared
        .monitor_proofs
        .iter()
        .find(|proof| proof.case_id == "monitor-blend")
        .ok_or("counter-rhythm qualification could not find committed Blend route")?;

    let graph = prepared
        .state
        .source_graph
        .as_ref()
        .ok_or("counter-rhythm qualification lost Source Graph")?;
    let source = &graph.source;
    let feature = graph
        .phrase_audio_features
        .iter()
        .min_by_key(|evidence| (evidence.phrase_index, evidence.start_bar));
    let review_feature = feature.ok_or("source graph has no phrase-audio evidence")?;
    let review_position = prepared
        .source_timing
        .bar_start_beat_cursor(u64::from(review_feature.start_bar))
        .ok_or("counter-rhythm qualification could not map first source phrase to transport")?;
    let mut review_state = prepared.state.clone();
    review_state.update_transport_clock(TransportClockState {
        is_playing: true,
        position_beats: review_position as f64,
        beat_index: review_position,
        bar_index: u64::from(review_feature.start_bar),
        phrase_index: u64::from(review_feature.phrase_index),
        current_scene: review_state.runtime.transport.current_scene.clone(),
    });
    let mut candidate = live_flow::render_plan(
        &review_state,
        prepared.source_timing.bpm,
        review_position as f64,
    );
    candidate.source_monitor_render = blend.plan.source_monitor_render.clone();
    let selected = candidate.tr909_render.counter_rhythm;
    let activation_ok = candidate.tr909_render.mode == Tr909RenderMode::BreakReinforce
        && candidate.tr909_render.routing == Tr909RenderRouting::DrumBusSupport
        && candidate.tr909_render.slam_enabled
        && candidate.tr909_render.pattern_adoption == Some(Tr909PatternAdoption::MainlineDrive)
        && selected.is_some();
    let report_path = output_dir.join("counter-rhythm-qualification.json");
    if !activation_ok {
        fs::write(
            report_path,
            serde_json::to_vec_pretty(&json!({
                "schema": "riotbox.tr909_counter_rhythm_qualification.case.v1",
                "mechanism": "tr909_counter_rhythm_slam_v3",
                "decision": "RBX-291",
                "result": "refused",
                "source": source,
                "selection": selected.map(|value| value.label()),
                "reason": "frozen_activation_or_source_evidence_gate_not_satisfied",
                "review_wavs_written": false,
            }))?,
        )?;
        println!(
            "counter-rhythm qualification: refused source={}",
            source.source_id
        );
        return Ok(());
    }

    let mut phase_control = candidate.clone();
    phase_control.tr909_render.counter_rhythm_phase = Tr909CounterRhythmPhase::PhaseControl;
    let frame_count = review_frame_count(prepared.source_timing.bpm);
    let candidate_output = render(&candidate, frame_count)?;
    let phase_control_output = render(&phase_control, frame_count)?;
    let delta = signal_delta_metrics(&candidate_output.samples, &phase_control_output.samples);
    let max_step_delta_rms = max_step_delta_rms(
        &candidate_output.samples,
        &phase_control_output.samples,
        prepared.source_timing.bpm,
    );
    let candidate_metrics = signal_metrics(&candidate_output.samples);
    let phase_control_metrics = signal_metrics(&phase_control_output.samples);
    let limiter_clean =
        limiter_is_clean(&candidate_output) && limiter_is_clean(&phase_control_output);
    let passed = max_step_delta_rms >= MIN_TIME_LOCAL_DELTA_RMS && limiter_clean;

    if write_wavs && passed {
        write_interleaved_pcm16_wav(
            output_dir.join("candidate.wav"),
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &candidate_output.samples,
        )?;
        write_interleaved_pcm16_wav(
            output_dir.join("phase-control.wav"),
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &phase_control_output.samples,
        )?;
    }

    fs::write(
        report_path,
        serde_json::to_vec_pretty(&json!({
            "schema": "riotbox.tr909_counter_rhythm_qualification.case.v1",
            "mechanism": "tr909_counter_rhythm_slam_v3",
            "decision": "RBX-291",
            "result": if passed { "pass" } else { "fail" },
            "source": source,
            "source_feature_evidence": feature.map(|evidence| json!({
                "phrase_index": evidence.phrase_index,
                "transient_density": evidence.transient_density,
                "offbeat_onset_density": evidence.offbeat_onset_density,
                "confidence": evidence.confidence,
            })),
            "selection": selected.map(|value| value.label()),
            "product_path": "JamAppState committed Slam -> render projection -> callback-block RuntimeMix -> master limiter",
            "phase_control": "same plan with only accent/donor polarity swapped",
            "contract_invariants": {
                "accent_slots_per_bar": 2,
                "donor_slots_per_bar": 2,
                "changed_slot_multiplier_sum": 4.0,
                "event_count_equal": true,
                "kick_lane_unchanged": true,
                "downbeat_and_backbeat_unchanged": true,
            },
            "metrics": {
                "candidate_rms": candidate_metrics.rms,
                "phase_control_rms": phase_control_metrics.rms,
                "whole_render_delta_rms": delta.rms,
                "maximum_step_local_delta_rms": max_step_delta_rms,
                "candidate_pre_limiter_clips": candidate_output.limiter.pre.clip_count,
                "candidate_limited_samples": candidate_output.limiter.limited_sample_count,
                "candidate_post_limiter_clips": candidate_output.limiter.post.clip_count,
                "phase_control_pre_limiter_clips": phase_control_output.limiter.pre.clip_count,
                "phase_control_limited_samples": phase_control_output.limiter.limited_sample_count,
                "phase_control_post_limiter_clips": phase_control_output.limiter.post.clip_count,
            },
            "thresholds": {
                "minimum_time_local_delta_rms": MIN_TIME_LOCAL_DELTA_RMS,
                "maximum_pre_limiter_clip_count": 0,
                "maximum_limited_sample_count": 0,
                "maximum_post_limiter_clip_count": 0,
            },
            "review_wavs_written": write_wavs && passed,
        }))?,
    )?;
    println!(
        "counter-rhythm qualification: result={} source={} policy={} local_delta_rms={:.6}",
        if passed { "pass" } else { "fail" },
        source.source_id,
        selected.expect("activation requires selection").label(),
        max_step_delta_rms,
    );
    if passed {
        Ok(())
    } else {
        Err("counter-rhythm qualification failed frozen technical gates".into())
    }
}

fn render(
    plan: &riotbox_audio::runtime::RuntimeMixRenderPlan,
    frame_count: usize,
) -> Result<RuntimeMixRenderOutput, Box<dyn Error>> {
    render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(plan, frame_count)],
        SAMPLE_RATE,
        CHANNEL_COUNT,
        CALLBACK_FRAME_COUNT,
    )
    .pop()
    .ok_or_else(|| "counter-rhythm exact-mix render produced no output".into())
}

fn review_frame_count(bpm: f32) -> usize {
    (REVIEW_BARS as f32 * 4.0 * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize
}

fn max_step_delta_rms(candidate: &[f32], control: &[f32], bpm: f32) -> f32 {
    let step_frames = (60.0 / bpm * SAMPLE_RATE as f32 / 4.0).round() as usize;
    let step_samples = step_frames
        .saturating_mul(usize::from(CHANNEL_COUNT))
        .max(1);
    candidate
        .chunks(step_samples)
        .zip(control.chunks(step_samples))
        .map(|(left, right)| signal_delta_metrics(left, right).rms)
        .fold(0.0, f32::max)
}

fn limiter_is_clean(output: &RuntimeMixRenderOutput) -> bool {
    output.limiter.pre.clip_count == 0
        && output.limiter.limited_sample_count == 0
        && output.limiter.post.clip_count == 0
}
