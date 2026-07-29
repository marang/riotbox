use std::{env, fs, path::PathBuf, time::Instant};

use riotbox_app::jam_app::JamAppState;
use riotbox_audio::{
    runtime::{
        AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan, RuntimeMixRenderSequenceStep,
        render_runtime_mix_plan_sequence_realtime_simulation_offline,
        render_runtime_mix_realtime_simulation_offline, signal_delta_metrics, signal_metrics,
    },
    source_audio::{SourceAudioCache, write_interleaved_pcm16_wav},
    w30::{
        W30_RESAMPLE_H13_MAX_BODY_GAIN, W30_RESAMPLE_H13_MIN_BODY_GAIN,
        W30_RESAMPLE_H13_MIN_IMPACT_LEVEL_COMPENSATION, W30_RESAMPLE_H13_MIN_PICKUP_GAIN,
        W30_RESAMPLE_HARD_SLICE_COUNT, W30_RESAMPLE_MIN_BODY_PRESERVATION_RATIO,
        W30ResampleHardGestureRecipe, W30ResampleLowImpactRecipe, W30ResampleTapHardPolicy,
        W30ResampleTapState,
    },
};
use riotbox_core::{
    action::{CaptureLengthIntent, CommitBoundary, SourceMonitorMode},
    transport::CommitBoundaryState,
};

#[path = "w30_live_path_render/preflight.rs"]
mod preflight;

use preflight::{
    W30ReachabilityPreflightReport, analyze_timing_reachability, write_preflight_report,
};

const SAMPLE_RATE: u32 = 48_000;
const CHANNEL_COUNT: u16 = 2;
const ATTACK_RATIO_MIN: f64 = 1.1;
const LOW_IMPACT_ATTACK_RATIO_MIN: f64 = 1.15;
const LOW_IMPACT_ATTACK_RMS_MIN: f64 = 0.0001;
const SOURCE_HIT_SHAPER_HEAD_RATIO_MIN: f64 = 1.15;
const SOURCE_HIT_SHAPER_HEAD_RMS_MIN: f64 = 0.0001;
const SOURCE_ALIGNED_IMPACT_HEAD_RATIO_MIN: f64 = 1.10;
const SOURCE_HIT_SHAPER_BODY_RATIO_MIN: f64 = 1.15;
const SOURCE_HIT_SHAPER_BODY_RMS_MIN: f64 = 0.0001;
const SOURCE_ALIGNED_IMPACT_BODY_RMS_MIN: f64 = 0.001;
const SOURCE_HIT_SHAPER_SIGNIFICANT_DELTA_MS_MIN: f64 = 25.0;
const SOURCE_ALIGNED_IMPACT_SIGNIFICANT_DELTA_MS_MIN: f64 = 6.0;
const SOURCE_ALIGNED_IMPACT_CREST_RATIO_MIN: f64 = 0.9;
const GESTURE_LEVEL_RATIO_MIN: f64 = 0.9;
const GESTURE_LEVEL_RATIO_MAX: f64 = 1.15;
const SOURCE_HIT_SHAPER_GESTURE_LEVEL_RATIO_MAX: f64 = 1.3;
const POLICY_RELATIVE_DELTA_MIN: f64 = 0.12;
// These are anti-collapse diagnostics for the versioned grit recipe, not a
// substitute for the structured human verdict that the gesture sounds harder.
const GESTURE_RMS_MATCHED_RELATIVE_DELTA_MIN: f64 = 0.2;
const GESTURE_CORRELATION_MAX: f64 = 0.98;
const H13_IMPACT_BODY_RATIO_MIN: f64 = 1.05;
const H13_PICKUP_RELATIVE_DELTA_MIN: f64 = 0.20;
const BOUNDARY_LOCAL_OUTLIER_RATIO_MAX: f64 = 1.5;
const PCM16_LSB: f64 = 1.0 / 32_768.0;

#[derive(Clone, Copy, Debug)]
struct ResampleDirectionalMetrics {
    attack_0_10ms_hard_over_base: f64,
    selected_band_attack_hard_over_base: f64,
    low_impact_attack_base_rms: Option<f64>,
    low_impact_attack_hard_rms: Option<f64>,
    low_impact_attack_hard_over_base: Option<f64>,
    source_hit_shaper_head_base_rms: Option<f64>,
    source_hit_shaper_head_hard_rms: Option<f64>,
    source_hit_shaper_head_hard_over_base: Option<f64>,
    source_hit_shaper_body_base_rms: Option<f64>,
    source_hit_shaper_body_hard_rms: Option<f64>,
    source_hit_shaper_body_hard_over_base: Option<f64>,
    source_hit_shaper_significant_delta_ms_per_hit: Option<f64>,
    h13_impact_body_hard_over_base: Option<f64>,
    h13_pickup_relative_rms_delta: Option<f64>,
    body_40_120ms_hard_over_base: f64,
    body_120_200ms_hard_over_base: f64,
    gesture_level_hard_over_base: f64,
    gesture_relative_rms_delta: f64,
    gesture_rms_matched_relative_delta: f64,
    gesture_correlation: f64,
    base_boundary_jump_max: f64,
    hard_boundary_jump_max: f64,
    hard_boundary_local_outlier_ratio_max: f64,
    base_global_frame_jump_max: f64,
    hard_global_frame_jump_max: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let source_path = required_path(&args, "--source")?;
    let output_dir = required_path(&args, "--output")?;
    let bpm = required_value(&args, "--bpm")?.parse::<f32>()?;
    let downbeat_seconds = optional_value(&args, "--downbeat-seconds")
        .map(str::parse::<f32>)
        .transpose()?;
    let include_resample = args.iter().any(|arg| arg == "--include-resample");
    let preflight_only = args.iter().any(|arg| arg == "--preflight-only");
    let require_exact_hit_shaper =
        preflight_only || args.iter().any(|arg| arg == "--require-exact-hit-shaper");
    if require_exact_hit_shaper && !include_resample {
        return Err(
            "--preflight-only/--require-exact-hit-shaper requires --include-resample".into(),
        );
    }
    fs::create_dir_all(&output_dir)?;
    let mut reachability_report = if require_exact_hit_shaper {
        let timing = analyze_timing_reachability(&source_path, bpm, downbeat_seconds)?;
        let report = W30ReachabilityPreflightReport::from_timing(&source_path, timing);
        if !report.timing_allows_product_projection() {
            write_preflight_report(&output_dir, &report)?;
            return Err("W-30 candidate WAV generation rejected by source-timing preflight".into());
        }
        Some(report)
    } else {
        None
    };

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
    if let Some(report) = reachability_report.as_mut() {
        let graph = state
            .source_graph
            .as_ref()
            .ok_or("W-30 reachability requires a Source Graph")?;
        report.record_product_timing(&graph.timing);
        if !report.timing_allows_product_projection() {
            write_preflight_report(&output_dir, report)?;
            return Err("W-30 candidate WAV generation rejected by product timing drift".into());
        }
    }
    state.set_transport_playing(true);
    let scene_id = state.runtime.transport.current_scene.clone();
    state.queue_source_monitor_mode(SourceMonitorMode::Riotbox, 90);
    commit(
        &mut state,
        CommitBoundary::Immediate,
        0,
        1,
        1,
        scene_id.clone(),
        95,
    )?;
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
        4,
        2,
        1,
        scene_id.clone(),
        200,
    )?;
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
    let normal_state = state.clone();
    let resample_outputs = if include_resample {
        if state.queue_w30_internal_resample(410).is_none() {
            return Err("W-30 internal resample was unavailable".into());
        }
        commit(
            &mut state,
            CommitBoundary::Phrase,
            16,
            4,
            2,
            scene_id.clone(),
            500,
        )?;
        println!(
            "base resample tap: mode={:?} routing={:?} availability={:?} source={:?} lineage={} generation={} variation={:?} policy={} suitability={} source_rms={:.6} active_frame_ratio={:.3}",
            state.runtime.w30_resample_tap.mode,
            state.runtime.w30_resample_tap.routing,
            state.runtime.w30_resample_tap.availability,
            state.runtime.w30_resample_tap.source_capture_id,
            state.runtime.w30_resample_tap.lineage_capture_count,
            state.runtime.w30_resample_tap.generation_depth,
            state.runtime.w30_resample_tap.variation,
            state.runtime.w30_resample_tap.hard_policy.label(),
            state
                .runtime
                .w30_resample_tap
                .hard_suitability
                .status
                .label(),
            state.runtime.w30_resample_tap.hard_suitability.source_rms,
            state
                .runtime
                .w30_resample_tap
                .hard_suitability
                .active_frame_ratio,
        );
        let base_tap_state = state.runtime.w30_resample_tap.clone();

        if state.queue_w30_apply_damage_profile(510).is_none() {
            return Err("W-30 post-resample damage gesture was unavailable".into());
        }
        commit(
            &mut state,
            CommitBoundary::Bar,
            17,
            5,
            2,
            scene_id.clone(),
            600,
        )?;
        if let Some(report) = reachability_report.as_mut() {
            report.record_projection(&state.runtime.w30_resample_tap);
            write_preflight_report(&output_dir, report)?;
            if !report.candidate_wav_generation_eligible_after_preflight() {
                return Err(
                    "W-30 candidate WAV generation rejected by Hard-recipe preflight".into(),
                );
            }
            if preflight_only {
                return Ok(());
            }
        }

        print_w30_render_summary("damaged", &state.runtime.w30_preview);
        let base_tap = render_resample_tap(&state, base_tap_state.clone(), bpm);
        let damaged = render_state(&state, bpm);
        let hard_tap_state = state.runtime.w30_resample_tap.clone();
        let hard_tap = render_resample_tap(&state, hard_tap_state.clone(), bpm);
        let mut h12_counterfactual_state = hard_tap_state.clone();
        h12_counterfactual_state.hard_gesture = Default::default();
        let h12_counterfactual_tap = render_resample_tap(&state, h12_counterfactual_state, bpm);
        let live_gesture =
            render_resample_live_gesture(&state, base_tap_state, hard_tap_state, bpm);
        println!(
            "hard resample tap: variation={:?} revision={} intensity={} policy={} suitability={} source_rms={:.6} active_frame_ratio={:.3} calibrated_output_gain={:.6} hit_window_compensation_gain={:.6} impact_body_eq_gain_db={:.3} impact_presence_gain={:.3} exact_callback_calibrated={} exact_callback_evaluated={} predicted_raw_level_ratio={:.6} predicted_compensated_level_ratio={:.6} predicted_level_matched_body_ratio={:.6} grit_recipe={} grit_effective_sample_rate_hz={:?} grit_quantization_levels={:?} trigger_mask={:08b} onset_cursors={:?} attack_lengths={:?} bite_band={} bite_input_gain={:.3} bite_output_gain={:.3} low_impact_recipe={} presence_head_wet={:.3} low_attack_share={:.3} low_attack_over_body={:.3} low_attack_over_source={:.3} hard_gesture_recipe={} impact_slot={} pickup_slot={} body_gain={:.3} impact_level_compensation={:.3} pickup_gain={:.3} selected_head_rms={:.6} selected_body_rms={:.6} transient_contrast={:.3}",
            state.runtime.w30_resample_tap.variation,
            state.runtime.w30_resample_tap.variation_revision,
            state.runtime.w30_resample_tap.variation_intensity,
            state.runtime.w30_resample_tap.hard_policy.label(),
            state
                .runtime
                .w30_resample_tap
                .hard_suitability
                .status
                .label(),
            state.runtime.w30_resample_tap.hard_suitability.source_rms,
            state
                .runtime
                .w30_resample_tap
                .hard_suitability
                .active_frame_ratio,
            state.runtime.w30_resample_tap.hard_calibration.output_gain,
            state
                .runtime
                .w30_resample_tap
                .hard_calibration
                .hit_window_compensation_gain,
            state
                .runtime
                .w30_resample_tap
                .hard_calibration
                .impact_body_eq_gain_db,
            state
                .runtime
                .w30_resample_tap
                .hard_low_impact
                .recipe
                .calibrated_presence_gain(
                    state
                        .runtime
                        .w30_resample_tap
                        .hard_calibration
                        .impact_body_eq_gain_db,
                ),
            state
                .runtime
                .w30_resample_tap
                .hard_calibration
                .exact_callback_calibrated,
            state
                .runtime
                .w30_resample_tap
                .hard_calibration
                .exact_callback_evaluated,
            state
                .runtime
                .w30_resample_tap
                .hard_calibration
                .predicted_raw_level_ratio,
            state
                .runtime
                .w30_resample_tap
                .hard_calibration
                .predicted_compensated_level_ratio,
            state
                .runtime
                .w30_resample_tap
                .hard_calibration
                .predicted_level_matched_body_ratio,
            state
                .runtime
                .w30_resample_tap
                .hard_policy
                .grit_recipe()
                .label(),
            state
                .runtime
                .w30_resample_tap
                .hard_policy
                .grit_recipe()
                .effective_sample_rate_hz(),
            state
                .runtime
                .w30_resample_tap
                .hard_policy
                .grit_recipe()
                .quantization_levels(),
            state.runtime.w30_resample_tap.hard_trigger_mask,
            state.runtime.w30_resample_tap.hard_slice_cursors,
            state.runtime.w30_resample_tap.hard_attack_lengths,
            state.runtime.w30_resample_tap.hard_attack_bite.band.label(),
            state.runtime.w30_resample_tap.hard_attack_bite.input_gain,
            state.runtime.w30_resample_tap.hard_attack_bite.output_gain,
            state
                .runtime
                .w30_resample_tap
                .hard_low_impact
                .recipe
                .label(),
            state
                .runtime
                .w30_resample_tap
                .hard_low_impact
                .presence_head_wet,
            state
                .runtime
                .w30_resample_tap
                .hard_low_impact
                .low_band_attack_share,
            state
                .runtime
                .w30_resample_tap
                .hard_low_impact
                .low_band_attack_over_body,
            state
                .runtime
                .w30_resample_tap
                .hard_low_impact
                .low_band_attack_over_source,
            state.runtime.w30_resample_tap.hard_gesture.recipe.label(),
            state.runtime.w30_resample_tap.hard_gesture.impact_slot,
            state.runtime.w30_resample_tap.hard_gesture.pickup_slot,
            state.runtime.w30_resample_tap.hard_gesture.body_gain,
            state
                .runtime
                .w30_resample_tap
                .hard_gesture
                .impact_level_compensation,
            state.runtime.w30_resample_tap.hard_gesture.pickup_gain,
            state
                .runtime
                .w30_resample_tap
                .hard_gesture
                .selected_head_rms,
            state
                .runtime
                .w30_resample_tap
                .hard_gesture
                .selected_body_rms,
            state.runtime.w30_resample_tap.hard_transient_contrast,
        );

        let mut unavailable_state = state.runtime.w30_resample_tap.clone();
        unavailable_state.source_audio = None;
        unavailable_state.availability =
            riotbox_audio::w30::W30ResampleTapAvailability::SourceAudioUnavailable;
        unavailable_state.routing = riotbox_audio::w30::W30ResampleTapRouting::Silent;
        let unavailable = render_resample_tap(&state, unavailable_state, bpm);
        write_interleaved_pcm16_wav(
            output_dir.join("02_w30_live_hook_pitch_damage.wav"),
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &damaged,
        )?;
        write_interleaved_pcm16_wav(
            output_dir.join("03_w30_source_backed_resample_tap_base.wav"),
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &base_tap,
        )?;
        write_interleaved_pcm16_wav(
            output_dir.join("04_w30_source_backed_resample_tap_hard_damage.wav"),
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &hard_tap,
        )?;
        write_interleaved_pcm16_wav(
            output_dir.join("05_w30_missing_source_silence.wav"),
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &unavailable,
        )?;
        write_interleaved_pcm16_wav(
            output_dir.join("06_w30_resample_base_to_hard_live_gesture.wav"),
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &live_gesture,
        )?;
        write_interleaved_pcm16_wav(
            output_dir.join("07_w30_h12_hard_counterfactual.wav"),
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &h12_counterfactual_tap,
        )?;
        Some((
            damaged,
            base_tap,
            hard_tap,
            unavailable,
            h12_counterfactual_tap,
        ))
    } else {
        if state.queue_w30_apply_damage_profile(410).is_none() {
            return Err("W-30 damage gesture was unavailable".into());
        }
        commit(&mut state, CommitBoundary::Bar, 9, 3, 1, scene_id, 500)?;
        print_w30_render_summary("damaged", &state.runtime.w30_preview);
        let damaged = render_state(&state, bpm);
        write_interleaved_pcm16_wav(
            output_dir.join("02_w30_live_hook_pitch_damage.wav"),
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &damaged,
        )?;
        Some((damaged, Vec::new(), Vec::new(), Vec::new(), Vec::new()))
    };
    let normal = render_state(&normal_state, bpm);
    write_interleaved_pcm16_wav(
        output_dir.join("01_w30_live_hook.wav"),
        SAMPLE_RATE,
        CHANNEL_COUNT,
        &normal,
    )?;
    let source = SourceAudioCache::load_pcm_wav(&source_path)?;
    write_interleaved_pcm16_wav(
        output_dir.join("00_source.wav"),
        source.sample_rate,
        source.channel_count,
        source.interleaved_samples(),
    )?;
    state.save()?;

    let normal_metrics = signal_metrics(&normal);
    let damaged = &resample_outputs.as_ref().expect("damaged render").0;
    let damaged_metrics = signal_metrics(damaged);
    let delta = signal_delta_metrics(&normal, damaged);
    println!("normal: {normal_metrics:?}");
    println!("damaged: {damaged_metrics:?}");
    println!("gesture delta: {delta:?}");
    if normal_metrics.rms <= 0.001 || damaged_metrics.rms <= 0.001 || delta.rms <= 0.001 {
        return Err("live W-30 render was silent or gesture-collapsed".into());
    }
    if include_resample {
        let (_, base_tap, hard_tap, unavailable, h12_counterfactual_tap) =
            resample_outputs.as_ref().expect("resample outputs");
        let tap_metrics = signal_metrics(base_tap);
        let hard_metrics = signal_metrics(hard_tap);
        let variation_delta = signal_delta_metrics(base_tap, hard_tap);
        let unavailable_metrics = signal_metrics(unavailable);
        println!("source-backed base resample tap: {tap_metrics:?}");
        println!("hard resample variation: {hard_metrics:?}");
        println!("base-to-hard variation delta: {variation_delta:?}");
        println!("missing-source control: {unavailable_metrics:?}");
        if tap_metrics.rms <= 0.001
            || hard_metrics.rms <= 0.001
            || tap_metrics.peak_abs >= 0.99
            || hard_metrics.peak_abs >= 0.99
        {
            return Err("source-backed resample tap was silent or clipped".into());
        }
        let hard_state = &state.runtime.w30_resample_tap;
        if hard_state.hard_policy == W30ResampleTapHardPolicy::Unavailable {
            if hard_state.hard_suitability.status
                == riotbox_audio::w30::W30ResampleHardSuitability::Suitable
            {
                return Err("suitable W-30 source has no assigned Hard policy".into());
            }
            if variation_delta.peak_abs > f32::EPSILON {
                return Err(format!(
                    "unavailable W-30 Hard policy changed the Base output: peak_delta={:.9}",
                    variation_delta.peak_abs
                )
                .into());
            }
            println!(
                "hard variation explicitly unavailable: suitability={} (Base output preserved)",
                hard_state.hard_suitability.status.label()
            );
        } else {
            if hard_state.hard_suitability.status
                != riotbox_audio::w30::W30ResampleHardSuitability::Suitable
            {
                return Err("audible W-30 Hard policy lacks suitable source evidence".into());
            }
            let relative_delta = f64::from(variation_delta.rms / tap_metrics.rms.max(f32::EPSILON));
            println!(
                "base-to-hard policy contrast: relative_delta={relative_delta:.6} minimum={POLICY_RELATIVE_DELTA_MIN}"
            );
            if relative_delta < POLICY_RELATIVE_DELTA_MIN {
                return Err(
                    "post-resample hard variation collapsed relative to the base tap".into(),
                );
            }
            if hard_state.hard_policy == W30ResampleTapHardPolicy::SourceTransientChop {
                let mut h12_counterfactual_state = hard_state.clone();
                h12_counterfactual_state.hard_gesture = Default::default();
                let h12_directional = resample_directional_metrics(
                    base_tap,
                    h12_counterfactual_tap,
                    &h12_counterfactual_state,
                    hard_state.tempo_bpm,
                    SAMPLE_RATE,
                    usize::from(CHANNEL_COUNT),
                )?;
                println!("base-to-H12 directional metrics: {h12_directional:?}");
                validate_resample_directional_metrics(
                    h12_directional,
                    hard_state.hard_low_impact.recipe,
                )?;
                if hard_state.hard_gesture.recipe
                    == W30ResampleHardGestureRecipe::SourceReverseIntoImpactV1
                {
                    let h13_directional = resample_directional_metrics(
                        h12_counterfactual_tap,
                        hard_tap,
                        hard_state,
                        hard_state.tempo_bpm,
                        SAMPLE_RATE,
                        usize::from(CHANNEL_COUNT),
                    )?;
                    println!("H12-to-H13 directional metrics: {h13_directional:?}");
                    println!(
                        "H12-to-H13 auxiliary metrics: raw_relative_delta={:.6} h12_boundary_jump={:.6} h13_boundary_jump={:.6}",
                        h13_directional.gesture_relative_rms_delta,
                        h13_directional.base_boundary_jump_max,
                        h13_directional.hard_boundary_jump_max,
                    );
                    validate_resample_h13_metrics(h13_directional, hard_state)?;
                } else {
                    println!(
                        "H13 unavailable: the aligned V5 impact owns the final Hard output without a delayed body gesture"
                    );
                }
            }
            validate_resample_hard_level(
                tap_metrics.rms,
                tap_metrics.peak_abs,
                hard_metrics.rms,
                hard_metrics.peak_abs,
                hard_state.hard_low_impact.recipe,
            )?;
        }
        if unavailable_metrics.active_samples != 0 {
            return Err("missing-source resample control emitted fallback audio".into());
        }
    }
    Ok(())
}

fn validate_resample_hard_level(
    base_rms: f32,
    base_peak: f32,
    hard_rms: f32,
    hard_peak: f32,
    low_impact_recipe: W30ResampleLowImpactRecipe,
) -> Result<(), Box<dyn std::error::Error>> {
    if !base_rms.is_finite()
        || !base_peak.is_finite()
        || !hard_rms.is_finite()
        || !hard_peak.is_finite()
        || base_rms <= f32::EPSILON
        || hard_rms <= f32::EPSILON
    {
        return Err("invalid W-30 Hard level comparison".into());
    }
    let ratio = f64::from(hard_rms / base_rms);
    let maximum = if matches!(
        low_impact_recipe,
        W30ResampleLowImpactRecipe::SourceHitShaperV3
            | W30ResampleLowImpactRecipe::SourceImpactShaperV4
    ) {
        SOURCE_HIT_SHAPER_GESTURE_LEVEL_RATIO_MAX
    } else {
        GESTURE_LEVEL_RATIO_MAX
    };
    if ratio < GESTURE_LEVEL_RATIO_MIN || ratio > maximum {
        return Err(format!(
            "hard gesture level compensation failed: {ratio:.4} outside {GESTURE_LEVEL_RATIO_MIN}..={maximum}"
        )
        .into());
    }
    println!(
        "base-to-hard level validation: ratio={ratio:.4} accepted={GESTURE_LEVEL_RATIO_MIN}..={maximum}"
    );
    if low_impact_recipe == W30ResampleLowImpactRecipe::SourceAlignedImpactV5 {
        let base_crest = f64::from(base_peak / base_rms);
        let hard_crest = f64::from(hard_peak / hard_rms);
        let crest_ratio = hard_crest / base_crest.max(f64::EPSILON);
        if crest_ratio < SOURCE_ALIGNED_IMPACT_CREST_RATIO_MIN {
            return Err(format!(
                "aligned V5 impact collapsed source crest: {crest_ratio:.4} < {SOURCE_ALIGNED_IMPACT_CREST_RATIO_MIN}"
            )
            .into());
        }
        println!(
            "aligned V5 crest validation: base={base_crest:.4} hard={hard_crest:.4} ratio={crest_ratio:.4}"
        );
    }
    Ok(())
}

fn resample_directional_metrics(
    base: &[f32],
    hard: &[f32],
    hard_state: &W30ResampleTapState,
    bpm: f32,
    sample_rate: u32,
    channel_count: usize,
) -> Result<ResampleDirectionalMetrics, Box<dyn std::error::Error>> {
    if base.len() != hard.len() || channel_count == 0 || bpm <= 0.0 || !bpm.is_finite() {
        return Err("invalid W-30 directional comparison inputs".into());
    }

    let frames_per_step = f64::from(sample_rate) * 60.0 / f64::from(bpm) / 2.0;
    let frame_count = base.len() / channel_count;
    let step_count = (frame_count as f64 / frames_per_step).ceil() as usize;
    let attack = window_energy_pair(
        base,
        hard,
        hard_state.hard_trigger_mask,
        step_count,
        frames_per_step,
        sample_rate,
        channel_count,
        0.0,
        0.010,
    );
    let early_body = window_energy_pair(
        base,
        hard,
        hard_state.hard_trigger_mask,
        step_count,
        frames_per_step,
        sample_rate,
        channel_count,
        0.040,
        0.120,
    );
    let late_body = window_energy_pair(
        base,
        hard,
        hard_state.hard_trigger_mask,
        step_count,
        frames_per_step,
        sample_rate,
        channel_count,
        0.120,
        0.200,
    );
    let selected_band = if matches!(
        hard_state.hard_low_impact.recipe,
        W30ResampleLowImpactRecipe::SourceHitShaperV3
            | W30ResampleLowImpactRecipe::SourceImpactShaperV4
            | W30ResampleLowImpactRecipe::SourceAlignedImpactV5
    ) {
        hard_state.hard_low_impact.recipe.presence_cutoff_hz()
    } else {
        hard_state.hard_attack_bite.band.cutoff_hz()
    };
    let selected_band_attack_hard_over_base = selected_band.map_or(1.0, |(low_hz, high_hz)| {
        let filtered_base = bandpass_interleaved(base, sample_rate, channel_count, low_hz, high_hz);
        let filtered_hard = bandpass_interleaved(hard, sample_rate, channel_count, low_hz, high_hz);
        rms_ratio(source_attack_window_energy_pair(
            &filtered_base,
            &filtered_hard,
            hard_state,
            hard_state.hard_trigger_mask,
            step_count,
            frames_per_step,
            sample_rate,
            channel_count,
        ))
    });
    let (low_impact_attack_base_rms, low_impact_attack_hard_rms) = hard_state
        .hard_low_impact
        .recipe
        .cutoff_hz()
        .map_or((None, None), |(low_hz, high_hz)| {
            let filtered_base =
                bandpass_interleaved(base, sample_rate, channel_count, low_hz, high_hz);
            let filtered_hard =
                bandpass_interleaved(hard, sample_rate, channel_count, low_hz, high_hz);
            let (base_rms, hard_rms) = rms_values(source_attack_window_energy_pair(
                &filtered_base,
                &filtered_hard,
                hard_state,
                hard_state.hard_trigger_mask,
                step_count,
                frames_per_step,
                sample_rate,
                channel_count,
            ));
            (Some(base_rms), Some(hard_rms))
        });
    let low_impact_attack_hard_over_base = low_impact_attack_base_rms
        .zip(low_impact_attack_hard_rms)
        .map(|(base_rms, hard_rms)| hard_rms / base_rms.max(f64::EPSILON));
    let (
        source_hit_shaper_head_base_rms,
        source_hit_shaper_head_hard_rms,
        source_hit_shaper_head_hard_over_base,
        source_hit_shaper_body_base_rms,
        source_hit_shaper_body_hard_rms,
        source_hit_shaper_body_hard_over_base,
        source_hit_shaper_significant_delta_ms_per_hit,
    ) = if matches!(
        hard_state.hard_low_impact.recipe,
        W30ResampleLowImpactRecipe::SourceHitShaperV3
            | W30ResampleLowImpactRecipe::SourceImpactShaperV4
            | W30ResampleLowImpactRecipe::SourceAlignedImpactV5
    ) {
        let (head_low_hz, head_high_hz) = hard_state
            .hard_low_impact
            .recipe
            .presence_cutoff_hz()
            .ok_or("source-hit shaper has no declared head band")?;
        let (body_low_hz, body_high_hz) = hard_state
            .hard_low_impact
            .recipe
            .cutoff_hz()
            .ok_or("source-hit shaper has no declared body band")?;
        let filtered_base_head =
            bandpass_interleaved(base, sample_rate, channel_count, head_low_hz, head_high_hz);
        let filtered_hard_head =
            bandpass_interleaved(hard, sample_rate, channel_count, head_low_hz, head_high_hz);
        let (head_base_rms, head_hard_rms) = rms_values(window_energy_pair(
            &filtered_base_head,
            &filtered_hard_head,
            hard_state.hard_trigger_mask,
            step_count,
            frames_per_step,
            sample_rate,
            channel_count,
            0.0,
            0.020,
        ));
        let filtered_base_body =
            bandpass_interleaved(base, sample_rate, channel_count, body_low_hz, body_high_hz);
        let filtered_hard_body =
            bandpass_interleaved(hard, sample_rate, channel_count, body_low_hz, body_high_hz);
        let (body_base_rms, body_hard_rms) = rms_values(window_energy_pair(
            &filtered_base_body,
            &filtered_hard_body,
            hard_state.hard_trigger_mask,
            step_count,
            frames_per_step,
            sample_rate,
            channel_count,
            0.020,
            0.100,
        ));
        (
            Some(head_base_rms),
            Some(head_hard_rms),
            Some(head_hard_rms / head_base_rms.max(f64::EPSILON)),
            Some(body_base_rms),
            Some(body_hard_rms),
            Some(body_hard_rms / body_base_rms.max(f64::EPSILON)),
            Some(significant_delta_ms_per_selected_hit(
                base,
                hard,
                hard_state.hard_trigger_mask,
                step_count,
                frames_per_step,
                sample_rate,
                channel_count,
                0.120,
                0.01,
            )),
        )
    } else {
        (None, None, None, None, None, None, None)
    };
    let (h13_impact_body_hard_over_base, h13_pickup_relative_rms_delta) = if hard_state
        .hard_gesture
        .recipe
        == W30ResampleHardGestureRecipe::SourceReverseIntoImpactV1
    {
        let impact_mask = 1_u8 << hard_state.hard_gesture.impact_slot.min(7);
        let pickup_mask = 1_u8 << hard_state.hard_gesture.pickup_slot.min(7);
        let step_seconds = 30.0 / f64::from(bpm);
        let pickup_seconds = f64::from(
            hard_state
                .hard_gesture
                .recipe
                .pickup_duration_frames(sample_rate),
        ) / f64::from(sample_rate.max(1));
        (
            Some(rms_ratio(window_energy_pair(
                base,
                hard,
                impact_mask,
                step_count,
                frames_per_step,
                sample_rate,
                channel_count,
                0.020,
                0.100,
            ))),
            Some(window_relative_rms_delta(
                base,
                hard,
                pickup_mask,
                step_count,
                frames_per_step,
                sample_rate,
                channel_count,
                (step_seconds - pickup_seconds).max(0.0),
                step_seconds,
            )),
        )
    } else {
        (None, None)
    };
    let base_rms = signal_rms(base);
    let hard_rms = signal_rms(hard);
    let hard_match_gain = base_rms / hard_rms.max(f64::EPSILON);

    Ok(ResampleDirectionalMetrics {
        attack_0_10ms_hard_over_base: rms_ratio(attack),
        selected_band_attack_hard_over_base,
        low_impact_attack_base_rms,
        low_impact_attack_hard_rms,
        low_impact_attack_hard_over_base,
        source_hit_shaper_head_base_rms,
        source_hit_shaper_head_hard_rms,
        source_hit_shaper_head_hard_over_base,
        source_hit_shaper_body_base_rms,
        source_hit_shaper_body_hard_rms,
        source_hit_shaper_body_hard_over_base,
        source_hit_shaper_significant_delta_ms_per_hit,
        h13_impact_body_hard_over_base,
        h13_pickup_relative_rms_delta,
        body_40_120ms_hard_over_base: rms_ratio(early_body),
        body_120_200ms_hard_over_base: rms_ratio(late_body),
        gesture_level_hard_over_base: hard_rms / base_rms.max(f64::EPSILON),
        gesture_relative_rms_delta: signal_delta_rms(base, hard) / base_rms.max(f64::EPSILON),
        gesture_rms_matched_relative_delta: signal_scaled_delta_rms(base, hard, hard_match_gain)
            / base_rms.max(f64::EPSILON),
        gesture_correlation: signal_correlation(base, hard),
        base_boundary_jump_max: maximum_trigger_boundary_jump(
            base,
            hard_state.hard_trigger_mask,
            step_count,
            frames_per_step,
            channel_count,
        ),
        hard_boundary_jump_max: maximum_trigger_boundary_jump(
            hard,
            hard_state.hard_trigger_mask,
            step_count,
            frames_per_step,
            channel_count,
        ),
        hard_boundary_local_outlier_ratio_max: maximum_trigger_boundary_local_outlier_ratio(
            hard,
            hard_state.hard_trigger_mask,
            step_count,
            frames_per_step,
            sample_rate,
            channel_count,
        ),
        base_global_frame_jump_max: maximum_frame_jump(base, channel_count),
        hard_global_frame_jump_max: maximum_frame_jump(hard, channel_count),
    })
}

fn signal_rms(samples: &[f32]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples
        .iter()
        .map(|sample| f64::from(*sample).powi(2))
        .sum::<f64>()
        / samples.len() as f64)
        .sqrt()
}

fn signal_delta_rms(left: &[f32], right: &[f32]) -> f64 {
    signal_scaled_delta_rms(left, right, 1.0)
}

fn signal_scaled_delta_rms(left: &[f32], right: &[f32], right_gain: f64) -> f64 {
    let sample_count = left.len().min(right.len());
    if sample_count == 0 {
        return 0.0;
    }
    (left[..sample_count]
        .iter()
        .zip(&right[..sample_count])
        .map(|(left, right)| (f64::from(*left) - f64::from(*right) * right_gain).powi(2))
        .sum::<f64>()
        / sample_count as f64)
        .sqrt()
}

fn signal_correlation(left: &[f32], right: &[f32]) -> f64 {
    let sample_count = left.len().min(right.len());
    if sample_count == 0 {
        return 0.0;
    }
    let left_mean = left[..sample_count]
        .iter()
        .map(|sample| f64::from(*sample))
        .sum::<f64>()
        / sample_count as f64;
    let right_mean = right[..sample_count]
        .iter()
        .map(|sample| f64::from(*sample))
        .sum::<f64>()
        / sample_count as f64;
    let (cross, left_energy, right_energy) = left[..sample_count]
        .iter()
        .zip(&right[..sample_count])
        .fold((0.0, 0.0, 0.0), |acc, (left, right)| {
            let left = f64::from(*left) - left_mean;
            let right = f64::from(*right) - right_mean;
            (
                acc.0 + left * right,
                acc.1 + left * left,
                acc.2 + right * right,
            )
        });
    cross / (left_energy * right_energy).sqrt().max(f64::EPSILON)
}

#[allow(clippy::too_many_arguments)]
fn window_energy_pair(
    base: &[f32],
    hard: &[f32],
    trigger_mask: u8,
    step_count: usize,
    frames_per_step: f64,
    sample_rate: u32,
    channel_count: usize,
    start_seconds: f64,
    end_seconds: f64,
) -> ((f64, usize), (f64, usize)) {
    let mut base_energy = 0.0;
    let mut hard_energy = 0.0;
    let mut sample_count = 0;
    let start_offset = (start_seconds * f64::from(sample_rate)).round() as usize;
    let end_offset = (end_seconds * f64::from(sample_rate)).round() as usize;
    let available_frames = base.len().min(hard.len()) / channel_count;

    for step in 0..step_count {
        let slot = step % 8;
        if trigger_mask & (1_u8 << slot) == 0 {
            continue;
        }
        let onset = (step as f64 * frames_per_step).round() as usize;
        let start = onset.saturating_add(start_offset).min(available_frames);
        let end = onset.saturating_add(end_offset).min(available_frames);
        for frame in start..end {
            let first_sample = frame * channel_count;
            for channel in 0..channel_count {
                let index = first_sample + channel;
                base_energy += f64::from(base[index]).powi(2);
                hard_energy += f64::from(hard[index]).powi(2);
                sample_count += 1;
            }
        }
    }

    ((base_energy, sample_count), (hard_energy, sample_count))
}

#[allow(clippy::too_many_arguments)]
fn window_relative_rms_delta(
    base: &[f32],
    hard: &[f32],
    trigger_mask: u8,
    step_count: usize,
    frames_per_step: f64,
    sample_rate: u32,
    channel_count: usize,
    start_seconds: f64,
    end_seconds: f64,
) -> f64 {
    let mut base_energy = 0.0;
    let mut delta_energy = 0.0;
    let mut sample_count = 0_usize;
    let start_offset = (start_seconds * f64::from(sample_rate)).round() as usize;
    let end_offset = (end_seconds * f64::from(sample_rate)).round() as usize;
    let available_frames = base.len().min(hard.len()) / channel_count.max(1);
    for step in 0..step_count {
        let slot = step % W30_RESAMPLE_HARD_SLICE_COUNT;
        if trigger_mask & (1_u8 << slot) == 0 {
            continue;
        }
        let onset = (step as f64 * frames_per_step).round() as usize;
        let start = onset.saturating_add(start_offset).min(available_frames);
        let end = onset.saturating_add(end_offset).min(available_frames);
        for frame in start..end {
            let first_sample = frame * channel_count;
            for channel in 0..channel_count {
                let index = first_sample + channel;
                let base_sample = f64::from(base[index]);
                base_energy += base_sample * base_sample;
                delta_energy += (f64::from(hard[index]) - base_sample).powi(2);
                sample_count += 1;
            }
        }
    }
    if sample_count == 0 {
        return 0.0;
    }
    let base_rms = (base_energy / sample_count as f64).sqrt();
    let delta_rms = (delta_energy / sample_count as f64).sqrt();
    delta_rms / base_rms.max(f64::EPSILON)
}

#[allow(clippy::too_many_arguments)]
fn source_attack_window_energy_pair(
    base: &[f32],
    hard: &[f32],
    hard_state: &W30ResampleTapState,
    trigger_mask: u8,
    step_count: usize,
    frames_per_step: f64,
    sample_rate: u32,
    channel_count: usize,
) -> ((f64, usize), (f64, usize)) {
    let mut base_energy = 0.0;
    let mut hard_energy = 0.0;
    let mut sample_count = 0;
    let available_frames = base.len().min(hard.len()) / channel_count;
    let Some(source) = hard_state.source_audio.as_deref() else {
        return ((0.0, 0), (0.0, 0));
    };
    let source_duration_seconds =
        source.source_frame_count.max(1) as f64 / f64::from(source.source_sample_rate.max(1));
    let source_duration_beats =
        source_duration_seconds * f64::from(hard_state.tempo_bpm.max(1.0)) / 60.0;
    let aligned_beats = source_duration_beats.round().clamp(1.0, 64.0);
    let cycle_output_frames =
        aligned_beats * 60.0 / f64::from(hard_state.tempo_bpm.max(1.0)) * f64::from(sample_rate);
    let cursor_increment = source.sample_count.max(1) as f64 / cycle_output_frames.max(1.0);

    for step in 0..step_count {
        let slot = step % 8;
        if trigger_mask & (1_u8 << slot) == 0 {
            continue;
        }
        let onset = (step as f64 * frames_per_step).round() as usize;
        let attack_frames = (f64::from(hard_state.hard_attack_lengths[slot].max(1))
            / cursor_increment)
            .round()
            .clamp(1.0, f64::from(sample_rate) * 0.08) as usize;
        let start = onset.min(available_frames);
        let end = onset.saturating_add(attack_frames).min(available_frames);
        for frame in start..end {
            let first_sample = frame * channel_count;
            for channel in 0..channel_count {
                let index = first_sample + channel;
                base_energy += f64::from(base[index]).powi(2);
                hard_energy += f64::from(hard[index]).powi(2);
                sample_count += 1;
            }
        }
    }

    ((base_energy, sample_count), (hard_energy, sample_count))
}

fn rms_ratio(energies: ((f64, usize), (f64, usize))) -> f64 {
    let (base_rms, hard_rms) = rms_values(energies);
    if base_rms <= f64::EPSILON {
        if hard_rms <= f64::EPSILON {
            1.0
        } else {
            f64::INFINITY
        }
    } else {
        hard_rms / base_rms
    }
}

fn rms_values(energies: ((f64, usize), (f64, usize))) -> (f64, f64) {
    let ((base_energy, base_count), (hard_energy, hard_count)) = energies;
    (
        (base_energy / base_count.max(1) as f64).sqrt(),
        (hard_energy / hard_count.max(1) as f64).sqrt(),
    )
}

#[allow(clippy::too_many_arguments)]
fn significant_delta_ms_per_selected_hit(
    base: &[f32],
    hard: &[f32],
    trigger_mask: u8,
    step_count: usize,
    frames_per_step: f64,
    sample_rate: u32,
    channel_count: usize,
    window_seconds: f64,
    threshold: f32,
) -> f64 {
    if channel_count == 0 || sample_rate == 0 {
        return 0.0;
    }
    let available_frames = base.len().min(hard.len()) / channel_count;
    let window_frames = (window_seconds * f64::from(sample_rate)).round() as usize;
    let mut changed_frames = vec![false; available_frames];
    let mut selected_hit_count = 0_usize;
    for step in 0..step_count {
        let slot = step % 8;
        if trigger_mask & (1_u8 << slot) == 0 {
            continue;
        }
        let onset = (step as f64 * frames_per_step).round() as usize;
        if onset >= available_frames {
            continue;
        }
        selected_hit_count += 1;
        let end = onset.saturating_add(window_frames).min(available_frames);
        for (frame, changed) in changed_frames.iter_mut().enumerate().take(end).skip(onset) {
            let first_sample = frame * channel_count;
            *changed = (0..channel_count).any(|channel| {
                (hard[first_sample + channel] - base[first_sample + channel]).abs() >= threshold
            });
        }
    }
    if selected_hit_count == 0 {
        return 0.0;
    }
    let changed_frame_count = changed_frames
        .into_iter()
        .filter(|changed| *changed)
        .count();
    changed_frame_count as f64 * 1_000.0 / f64::from(sample_rate) / selected_hit_count as f64
}

fn maximum_frame_jump(samples: &[f32], channel_count: usize) -> f64 {
    if channel_count == 0 {
        return 0.0;
    }
    samples
        .chunks_exact(channel_count)
        .zip(samples.chunks_exact(channel_count).skip(1))
        .flat_map(|(previous, current)| previous.iter().zip(current))
        .map(|(previous, current)| f64::from((current - previous).abs()))
        .fold(0.0, f64::max)
}

fn bandpass_interleaved(
    samples: &[f32],
    sample_rate: u32,
    channel_count: usize,
    low_hz: f32,
    high_hz: f32,
) -> Vec<f32> {
    if channel_count == 0 {
        return Vec::new();
    }
    let sample_rate = sample_rate.max(1) as f32;
    let low_alpha = 1.0 - (-std::f32::consts::TAU * low_hz / sample_rate).exp();
    let high_alpha = 1.0 - (-std::f32::consts::TAU * high_hz / sample_rate).exp();
    let mut low_state = vec![0.0_f32; channel_count];
    let mut high_state = vec![0.0_f32; channel_count];
    let mut initialized = vec![false; channel_count];
    let mut output = Vec::with_capacity(samples.len());
    for frame in samples.chunks_exact(channel_count) {
        for (channel, sample) in frame.iter().copied().enumerate() {
            if !initialized[channel] {
                low_state[channel] = sample;
                high_state[channel] = sample;
                initialized[channel] = true;
            }
            low_state[channel] += low_alpha * (sample - low_state[channel]);
            high_state[channel] += high_alpha * (sample - high_state[channel]);
            output.push(high_state[channel] - low_state[channel]);
        }
    }
    output
}

fn maximum_trigger_boundary_jump(
    samples: &[f32],
    trigger_mask: u8,
    step_count: usize,
    frames_per_step: f64,
    channel_count: usize,
) -> f64 {
    if channel_count == 0 {
        return 0.0;
    }
    let frame_count = samples.len() / channel_count;
    let mut maximum = 0.0_f64;
    for step in 1..step_count {
        if trigger_mask & (1_u8 << (step % 8)) == 0 {
            continue;
        }
        let frame = (step as f64 * frames_per_step).round() as usize;
        if frame == 0 || frame >= frame_count {
            continue;
        }
        let previous = (frame - 1) * channel_count;
        let current = frame * channel_count;
        for channel in 0..channel_count {
            maximum = maximum.max(f64::from(
                (samples[current + channel] - samples[previous + channel]).abs(),
            ));
        }
    }
    maximum
}

fn maximum_trigger_boundary_local_outlier_ratio(
    samples: &[f32],
    trigger_mask: u8,
    step_count: usize,
    frames_per_step: f64,
    sample_rate: u32,
    channel_count: usize,
) -> f64 {
    let frame_count = samples.len() / channel_count;
    let neighborhood_frames = (sample_rate as usize / 200).max(1);
    let mut maximum_ratio = 0.0_f64;
    for step in 0..step_count {
        let slot = step % 8;
        if trigger_mask & (1_u8 << slot) == 0 {
            continue;
        }
        let frame = (step as f64 * frames_per_step).round() as usize;
        if frame == 0 || frame >= frame_count {
            continue;
        }
        let boundary_jump = frame_jump(samples, frame, channel_count);
        let start = frame.saturating_sub(neighborhood_frames).max(1);
        let end = frame
            .saturating_add(neighborhood_frames)
            .min(frame_count.saturating_sub(1));
        let local_max = (start..=end)
            .filter(|candidate| *candidate != frame)
            .map(|candidate| frame_jump(samples, candidate, channel_count))
            .fold(0.0_f64, f64::max);
        maximum_ratio = maximum_ratio.max(boundary_jump / (local_max + PCM16_LSB));
    }
    maximum_ratio
}

fn frame_jump(samples: &[f32], frame: usize, channel_count: usize) -> f64 {
    let previous = (frame - 1) * channel_count;
    let current = frame * channel_count;
    (0..channel_count)
        .map(|channel| f64::from((samples[current + channel] - samples[previous + channel]).abs()))
        .fold(0.0_f64, f64::max)
}

fn validate_resample_directional_metrics(
    metrics: ResampleDirectionalMetrics,
    low_impact_recipe: W30ResampleLowImpactRecipe,
) -> Result<(), Box<dyn std::error::Error>> {
    if !metrics.base_global_frame_jump_max.is_finite()
        || !metrics.hard_global_frame_jump_max.is_finite()
    {
        return Err("non-finite W-30 frame slew metric".into());
    }
    if metrics.attack_0_10ms_hard_over_base < ATTACK_RATIO_MIN {
        return Err(format!(
            "hard attack lift failed: {:.4} < {ATTACK_RATIO_MIN}",
            metrics.attack_0_10ms_hard_over_base
        )
        .into());
    }
    if !metrics.selected_band_attack_hard_over_base.is_finite() {
        return Err("non-finite W-30 selected-band attack metric".into());
    }
    if let (Some(base_rms), Some(hard_rms), Some(ratio)) = (
        metrics.low_impact_attack_base_rms,
        metrics.low_impact_attack_hard_rms,
        metrics.low_impact_attack_hard_over_base,
    ) {
        if !base_rms.is_finite() || !hard_rms.is_finite() || !ratio.is_finite() {
            return Err("non-finite W-30 low-impact attack metric".into());
        }
        if hard_rms < LOW_IMPACT_ATTACK_RMS_MIN {
            return Err(format!(
                "assigned low-impact attack lacks absolute energy: {hard_rms:.6} < {LOW_IMPACT_ATTACK_RMS_MIN}"
            )
            .into());
        }
        if low_impact_recipe != W30ResampleLowImpactRecipe::SourceAlignedImpactV5
            && ratio < LOW_IMPACT_ATTACK_RATIO_MIN
        {
            return Err(format!(
                "assigned low-impact attack ratio failed: {ratio:.4} < {LOW_IMPACT_ATTACK_RATIO_MIN}"
            )
            .into());
        }
    }
    if matches!(
        low_impact_recipe,
        W30ResampleLowImpactRecipe::SourceHitShaperV3
            | W30ResampleLowImpactRecipe::SourceImpactShaperV4
            | W30ResampleLowImpactRecipe::SourceAlignedImpactV5
    ) {
        let (Some(head_base_rms), Some(head_hard_rms), Some(head_ratio)) = (
            metrics.source_hit_shaper_head_base_rms,
            metrics.source_hit_shaper_head_hard_rms,
            metrics.source_hit_shaper_head_hard_over_base,
        ) else {
            return Err("source-hit shaper head metrics are missing".into());
        };
        let (Some(body_base_rms), Some(body_hard_rms), Some(body_ratio)) = (
            metrics.source_hit_shaper_body_base_rms,
            metrics.source_hit_shaper_body_hard_rms,
            metrics.source_hit_shaper_body_hard_over_base,
        ) else {
            return Err("source-hit shaper body metrics are missing".into());
        };
        let Some(significant_delta_ms_per_hit) =
            metrics.source_hit_shaper_significant_delta_ms_per_hit
        else {
            return Err("source-hit shaper delta-duration metric is missing".into());
        };
        if !head_base_rms.is_finite()
            || !head_hard_rms.is_finite()
            || !head_ratio.is_finite()
            || !body_base_rms.is_finite()
            || !body_hard_rms.is_finite()
            || !body_ratio.is_finite()
            || !significant_delta_ms_per_hit.is_finite()
        {
            return Err("non-finite source-hit shaper metric".into());
        }
        let minimum_head_ratio =
            if low_impact_recipe == W30ResampleLowImpactRecipe::SourceAlignedImpactV5 {
                SOURCE_ALIGNED_IMPACT_HEAD_RATIO_MIN
            } else {
                SOURCE_HIT_SHAPER_HEAD_RATIO_MIN
            };
        if head_hard_rms < SOURCE_HIT_SHAPER_HEAD_RMS_MIN || head_ratio < minimum_head_ratio {
            return Err(format!(
                "source-hit shaper head lift failed: rms={head_hard_rms:.6} min={SOURCE_HIT_SHAPER_HEAD_RMS_MIN} ratio={head_ratio:.4} min_ratio={minimum_head_ratio}"
            )
            .into());
        }
        let body_contract_pass =
            if low_impact_recipe == W30ResampleLowImpactRecipe::SourceAlignedImpactV5 {
                body_hard_rms >= SOURCE_ALIGNED_IMPACT_BODY_RMS_MIN
            } else {
                body_hard_rms >= SOURCE_HIT_SHAPER_BODY_RMS_MIN
                    && body_ratio >= SOURCE_HIT_SHAPER_BODY_RATIO_MIN
            };
        if !body_contract_pass {
            return Err(format!(
                "source-hit shaper body contract failed: rms={body_hard_rms:.6} ratio={body_ratio:.4} recipe={}",
                low_impact_recipe.label()
            )
            .into());
        }
        let minimum_delta_ms =
            if low_impact_recipe == W30ResampleLowImpactRecipe::SourceAlignedImpactV5 {
                SOURCE_ALIGNED_IMPACT_SIGNIFICANT_DELTA_MS_MIN
            } else {
                SOURCE_HIT_SHAPER_SIGNIFICANT_DELTA_MS_MIN
            };
        if significant_delta_ms_per_hit < minimum_delta_ms {
            return Err(format!(
                "source-hit shaper changed too little of each selected hit: {significant_delta_ms_per_hit:.2}ms < {minimum_delta_ms:.2}ms"
            )
            .into());
        }
    }
    let gesture_level_ratio_max = if matches!(
        low_impact_recipe,
        W30ResampleLowImpactRecipe::SourceHitShaperV3
            | W30ResampleLowImpactRecipe::SourceImpactShaperV4
    ) {
        SOURCE_HIT_SHAPER_GESTURE_LEVEL_RATIO_MAX
    } else {
        GESTURE_LEVEL_RATIO_MAX
    };
    if metrics.gesture_level_hard_over_base < GESTURE_LEVEL_RATIO_MIN
        || metrics.gesture_level_hard_over_base > gesture_level_ratio_max
    {
        return Err(format!(
            "hard gesture level compensation failed: {:.4} outside {GESTURE_LEVEL_RATIO_MIN}..={gesture_level_ratio_max}",
            metrics.gesture_level_hard_over_base
        )
        .into());
    }
    if metrics.gesture_rms_matched_relative_delta < GESTURE_RMS_MATCHED_RELATIVE_DELTA_MIN {
        return Err(format!(
            "versioned hard transform collapsed after RMS matching: {:.4} < {GESTURE_RMS_MATCHED_RELATIVE_DELTA_MIN}",
            metrics.gesture_rms_matched_relative_delta
        )
        .into());
    }
    if metrics.gesture_correlation > GESTURE_CORRELATION_MAX {
        return Err(format!(
            "versioned hard transform remained indistinguishable after level compensation: {:.4} > {GESTURE_CORRELATION_MAX}",
            metrics.gesture_correlation
        )
        .into());
    }
    if low_impact_recipe != W30ResampleLowImpactRecipe::SourceAlignedImpactV5
        && (metrics.body_40_120ms_hard_over_base
            < f64::from(W30_RESAMPLE_MIN_BODY_PRESERVATION_RATIO)
            || metrics.body_120_200ms_hard_over_base
                < f64::from(W30_RESAMPLE_MIN_BODY_PRESERVATION_RATIO))
    {
        return Err(format!(
            "hard body preservation failed: early={:.4} late={:.4} min={W30_RESAMPLE_MIN_BODY_PRESERVATION_RATIO}",
            metrics.body_40_120ms_hard_over_base, metrics.body_120_200ms_hard_over_base
        )
        .into());
    }
    if metrics.hard_boundary_local_outlier_ratio_max > BOUNDARY_LOCAL_OUTLIER_RATIO_MAX {
        return Err(format!(
            "hard boundary continuity failed: local_outlier_ratio={:.4} max={BOUNDARY_LOCAL_OUTLIER_RATIO_MAX}",
            metrics.hard_boundary_local_outlier_ratio_max
        )
        .into());
    }
    Ok(())
}

fn validate_resample_h13_metrics(
    metrics: ResampleDirectionalMetrics,
    hard_state: &W30ResampleTapState,
) -> Result<(), Box<dyn std::error::Error>> {
    if hard_state.hard_gesture.recipe == W30ResampleHardGestureRecipe::Unavailable {
        return Ok(());
    }
    if hard_state.hard_gesture.recipe != W30ResampleHardGestureRecipe::SourceReverseIntoImpactV1 {
        return Err("unknown W-30 H13 gesture recipe".into());
    }
    let expected_pickup =
        (hard_state.hard_gesture.impact_slot + W30_RESAMPLE_HARD_SLICE_COUNT as u8 - 1)
            % W30_RESAMPLE_HARD_SLICE_COUNT as u8;
    if hard_state.hard_gesture.pickup_slot != expected_pickup
        || hard_state.hard_gesture.pickup_slot == hard_state.hard_gesture.impact_slot
    {
        return Err("H13 pickup/impact slot contract is invalid".into());
    }
    if !(W30_RESAMPLE_H13_MIN_BODY_GAIN..=W30_RESAMPLE_H13_MAX_BODY_GAIN)
        .contains(&hard_state.hard_gesture.body_gain)
        || !(W30_RESAMPLE_H13_MIN_IMPACT_LEVEL_COMPENSATION..=1.0)
            .contains(&hard_state.hard_gesture.impact_level_compensation)
        || !(W30_RESAMPLE_H13_MIN_PICKUP_GAIN..=1.0).contains(&hard_state.hard_gesture.pickup_gain)
        || hard_state.hard_gesture.selected_head_rms <= 0.0
        || hard_state.hard_gesture.selected_body_rms <= 0.0
    {
        return Err("H13 source-relative body evidence is invalid".into());
    }
    let Some(body_ratio) = metrics.h13_impact_body_hard_over_base else {
        return Err("H13 impact-body metric is missing".into());
    };
    if !body_ratio.is_finite() || body_ratio < H13_IMPACT_BODY_RATIO_MIN {
        return Err(format!(
            "H13 selected impact body failed: {body_ratio:.4} < {H13_IMPACT_BODY_RATIO_MIN}"
        )
        .into());
    }
    let Some(pickup_delta) = metrics.h13_pickup_relative_rms_delta else {
        return Err("H13 reverse-pickup metric is missing".into());
    };
    if !pickup_delta.is_finite() || pickup_delta < H13_PICKUP_RELATIVE_DELTA_MIN {
        return Err(format!(
            "H13 reverse pickup collapsed: {pickup_delta:.4} < {H13_PICKUP_RELATIVE_DELTA_MIN}"
        )
        .into());
    }
    if !metrics.hard_boundary_local_outlier_ratio_max.is_finite()
        || metrics.hard_boundary_local_outlier_ratio_max > BOUNDARY_LOCAL_OUTLIER_RATIO_MAX
    {
        return Err(format!(
            "H13 boundary continuity failed: local_outlier_ratio={:.4} max={BOUNDARY_LOCAL_OUTLIER_RATIO_MAX}",
            metrics.hard_boundary_local_outlier_ratio_max
        )
        .into());
    }
    println!(
        "H13 source reverse-into-impact validation: impact_slot={} pickup_slot={} body_ratio={body_ratio:.4} pickup_relative_delta={pickup_delta:.4}",
        hard_state.hard_gesture.impact_slot, hard_state.hard_gesture.pickup_slot,
    );
    Ok(())
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

fn render_resample_live_gesture(
    _state: &JamAppState,
    base: riotbox_audio::w30::W30ResampleTapState,
    hard: riotbox_audio::w30::W30ResampleTapState,
    bpm: f32,
) -> Vec<f32> {
    let segment_frame_count = (2.0 * 4.0 * 60.0 / bpm * SAMPLE_RATE as f32).round() as usize;
    let plan = |position_beats, tap| RuntimeMixRenderPlan {
        transport: AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: bpm,
            position_beats,
        },
        tr909_render: Default::default(),
        mc202_render: Default::default(),
        w30_preview_render: Default::default(),
        w30_resample_tap: tap,
        source_monitor_render: riotbox_audio::runtime::SourceMonitorRenderState::control_only(
            SourceMonitorMode::Riotbox,
        ),
    };
    let base_plan = plan(0.0, base);
    let hard_plan = plan(8.0, hard);
    render_runtime_mix_plan_sequence_realtime_simulation_offline(
        &[
            RuntimeMixRenderSequenceStep::new(&base_plan, segment_frame_count),
            RuntimeMixRenderSequenceStep::new(&hard_plan, segment_frame_count),
        ],
        SAMPLE_RATE,
        CHANNEL_COUNT,
        128,
    )
    .into_iter()
    .flatten()
    .collect()
}

fn render_resample_tap(
    _state: &JamAppState,
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
        source_monitor_render: riotbox_audio::runtime::SourceMonitorRenderState::control_only(
            SourceMonitorMode::Riotbox,
        ),
    };
    let started = Instant::now();
    let rendered = render_runtime_mix_realtime_simulation_offline(
        &plan,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        frame_count,
        128,
    );
    let elapsed = started.elapsed();
    let audio_seconds = frame_count as f64 / f64::from(SAMPLE_RATE);
    println!(
        "resample callback simulation: variation={} audio_seconds={audio_seconds:.3} wall_ms={:.3} realtime_factor={:.1}",
        plan.w30_resample_tap.variation.label(),
        elapsed.as_secs_f64() * 1_000.0,
        audio_seconds / elapsed.as_secs_f64().max(f64::EPSILON),
    );
    rendered
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directional_metrics_measure_attack_lift_without_hiding_body_loss() {
        let sample_rate = 1_000;
        let channel_count = 2;
        let mut base = vec![0.25_f32; 1_000 * channel_count];
        let mut hard = base.clone();
        for frame in 0..10 {
            for channel in 0..channel_count {
                hard[frame * channel_count + channel] = 0.5;
            }
        }
        for frame in 120..200 {
            for channel in 0..channel_count {
                hard[frame * channel_count + channel] = 0.125;
            }
        }
        // Keep the synthetic edge metric focused on adjacent-frame changes, not interleaving.
        base[500 * channel_count] = 0.26;
        hard[500 * channel_count] = 0.26;

        let metrics = resample_directional_metrics(
            &base,
            &hard,
            &W30ResampleTapState {
                hard_trigger_mask: 0b0000_0001,
                ..Default::default()
            },
            30.0,
            sample_rate,
            channel_count,
        )
        .expect("metrics");

        assert!((metrics.attack_0_10ms_hard_over_base - 2.0).abs() < 0.001);
        assert!((metrics.body_40_120ms_hard_over_base - 1.0).abs() < 0.001);
        assert!((metrics.body_120_200ms_hard_over_base - 0.5).abs() < 0.001);
        assert!(
            validate_resample_directional_metrics(
                metrics,
                W30ResampleLowImpactRecipe::Unavailable,
            )
            .is_err()
        );
    }

    #[test]
    fn directional_gate_accepts_lifted_attack_and_preserved_body() {
        let metrics = ResampleDirectionalMetrics {
            attack_0_10ms_hard_over_base: ATTACK_RATIO_MIN,
            selected_band_attack_hard_over_base: 1.0,
            low_impact_attack_base_rms: None,
            low_impact_attack_hard_rms: None,
            low_impact_attack_hard_over_base: None,
            source_hit_shaper_head_base_rms: None,
            source_hit_shaper_head_hard_rms: None,
            source_hit_shaper_head_hard_over_base: None,
            source_hit_shaper_body_base_rms: None,
            source_hit_shaper_body_hard_rms: None,
            source_hit_shaper_body_hard_over_base: None,
            source_hit_shaper_significant_delta_ms_per_hit: None,
            h13_impact_body_hard_over_base: None,
            h13_pickup_relative_rms_delta: None,
            body_40_120ms_hard_over_base: f64::from(W30_RESAMPLE_MIN_BODY_PRESERVATION_RATIO),
            body_120_200ms_hard_over_base: f64::from(W30_RESAMPLE_MIN_BODY_PRESERVATION_RATIO),
            gesture_level_hard_over_base: 1.0,
            gesture_relative_rms_delta: 0.5,
            gesture_rms_matched_relative_delta: GESTURE_RMS_MATCHED_RELATIVE_DELTA_MIN,
            gesture_correlation: GESTURE_CORRELATION_MAX,
            base_boundary_jump_max: 0.1,
            hard_boundary_jump_max: 0.15,
            hard_boundary_local_outlier_ratio_max: BOUNDARY_LOCAL_OUTLIER_RATIO_MAX,
            base_global_frame_jump_max: 0.1,
            hard_global_frame_jump_max: 0.2,
        };

        validate_resample_directional_metrics(metrics, W30ResampleLowImpactRecipe::Unavailable)
            .expect("directional gate");
    }

    #[test]
    fn directional_gate_rejects_a_level_only_hard_change() {
        let metrics = ResampleDirectionalMetrics {
            attack_0_10ms_hard_over_base: 1.1,
            selected_band_attack_hard_over_base: 1.1,
            low_impact_attack_base_rms: None,
            low_impact_attack_hard_rms: None,
            low_impact_attack_hard_over_base: None,
            source_hit_shaper_head_base_rms: None,
            source_hit_shaper_head_hard_rms: None,
            source_hit_shaper_head_hard_over_base: None,
            source_hit_shaper_body_base_rms: None,
            source_hit_shaper_body_hard_rms: None,
            source_hit_shaper_body_hard_over_base: None,
            source_hit_shaper_significant_delta_ms_per_hit: None,
            h13_impact_body_hard_over_base: None,
            h13_pickup_relative_rms_delta: None,
            body_40_120ms_hard_over_base: 1.1,
            body_120_200ms_hard_over_base: 1.1,
            gesture_level_hard_over_base: 1.1,
            gesture_relative_rms_delta: 0.1,
            gesture_rms_matched_relative_delta: 0.0,
            gesture_correlation: 1.0,
            base_boundary_jump_max: 0.1,
            hard_boundary_jump_max: 0.11,
            hard_boundary_local_outlier_ratio_max: 1.0,
            base_global_frame_jump_max: 0.1,
            hard_global_frame_jump_max: 0.11,
        };

        assert!(
            validate_resample_directional_metrics(
                metrics,
                W30ResampleLowImpactRecipe::Unavailable,
            )
            .is_err()
        );
    }

    #[test]
    fn directional_gate_requires_absolute_and_relative_low_impact_lift_when_assigned() {
        let metrics = ResampleDirectionalMetrics {
            attack_0_10ms_hard_over_base: ATTACK_RATIO_MIN,
            selected_band_attack_hard_over_base: 1.0,
            low_impact_attack_base_rms: Some(0.01),
            low_impact_attack_hard_rms: Some(0.011),
            low_impact_attack_hard_over_base: Some(1.1),
            source_hit_shaper_head_base_rms: None,
            source_hit_shaper_head_hard_rms: None,
            source_hit_shaper_head_hard_over_base: None,
            source_hit_shaper_body_base_rms: None,
            source_hit_shaper_body_hard_rms: None,
            source_hit_shaper_body_hard_over_base: None,
            source_hit_shaper_significant_delta_ms_per_hit: None,
            h13_impact_body_hard_over_base: None,
            h13_pickup_relative_rms_delta: None,
            body_40_120ms_hard_over_base: f64::from(W30_RESAMPLE_MIN_BODY_PRESERVATION_RATIO),
            body_120_200ms_hard_over_base: f64::from(W30_RESAMPLE_MIN_BODY_PRESERVATION_RATIO),
            gesture_level_hard_over_base: 1.0,
            gesture_relative_rms_delta: 0.5,
            gesture_rms_matched_relative_delta: GESTURE_RMS_MATCHED_RELATIVE_DELTA_MIN,
            gesture_correlation: GESTURE_CORRELATION_MAX,
            base_boundary_jump_max: 0.1,
            hard_boundary_jump_max: 0.15,
            hard_boundary_local_outlier_ratio_max: BOUNDARY_LOCAL_OUTLIER_RATIO_MAX,
            base_global_frame_jump_max: 0.1,
            hard_global_frame_jump_max: 0.2,
        };

        assert!(
            validate_resample_directional_metrics(
                metrics,
                W30ResampleLowImpactRecipe::SourceLowTransientPunchV1,
            )
            .is_err()
        );
    }

    #[test]
    fn source_hit_shaper_gate_requires_audible_head_body_and_delta_duration() {
        let mut metrics = ResampleDirectionalMetrics {
            attack_0_10ms_hard_over_base: ATTACK_RATIO_MIN,
            selected_band_attack_hard_over_base: SOURCE_HIT_SHAPER_HEAD_RATIO_MIN,
            low_impact_attack_base_rms: Some(0.01),
            low_impact_attack_hard_rms: Some(0.012),
            low_impact_attack_hard_over_base: Some(1.2),
            source_hit_shaper_head_base_rms: Some(0.01),
            source_hit_shaper_head_hard_rms: Some(0.012),
            source_hit_shaper_head_hard_over_base: Some(1.2),
            source_hit_shaper_body_base_rms: Some(0.01),
            source_hit_shaper_body_hard_rms: Some(0.012),
            source_hit_shaper_body_hard_over_base: Some(1.2),
            source_hit_shaper_significant_delta_ms_per_hit: Some(
                SOURCE_HIT_SHAPER_SIGNIFICANT_DELTA_MS_MIN,
            ),
            h13_impact_body_hard_over_base: None,
            h13_pickup_relative_rms_delta: None,
            body_40_120ms_hard_over_base: f64::from(W30_RESAMPLE_MIN_BODY_PRESERVATION_RATIO),
            body_120_200ms_hard_over_base: f64::from(W30_RESAMPLE_MIN_BODY_PRESERVATION_RATIO),
            gesture_level_hard_over_base: 1.0,
            gesture_relative_rms_delta: 0.5,
            gesture_rms_matched_relative_delta: GESTURE_RMS_MATCHED_RELATIVE_DELTA_MIN,
            gesture_correlation: GESTURE_CORRELATION_MAX,
            base_boundary_jump_max: 0.1,
            hard_boundary_jump_max: 0.15,
            hard_boundary_local_outlier_ratio_max: BOUNDARY_LOCAL_OUTLIER_RATIO_MAX,
            base_global_frame_jump_max: 0.1,
            hard_global_frame_jump_max: 0.2,
        };

        validate_resample_directional_metrics(
            metrics,
            W30ResampleLowImpactRecipe::SourceHitShaperV3,
        )
        .expect("source-hit shaper gate");

        metrics.source_hit_shaper_significant_delta_ms_per_hit =
            Some(SOURCE_HIT_SHAPER_SIGNIFICANT_DELTA_MS_MIN - 0.1);
        assert!(
            validate_resample_directional_metrics(
                metrics,
                W30ResampleLowImpactRecipe::SourceHitShaperV3,
            )
            .is_err()
        );
    }
}
