//! Existing offline pack/grid gates, not a source or human qualification verdict.
use super::{
    config::{
        CHANNEL_COUNT, MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO,
        MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO, MIN_LOW_BAND_RMS, MIN_SIGNAL_RMS,
        MIN_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
    },
    grid::Grid,
    mc202_bass_pressure::Mc202PatternOrigin,
    pack_report::PackReport,
    source_grid_output_drift::SOURCE_GRID_OUTPUT_MIN_HIT_RATIO,
};

pub(super) fn assert_grid_len(name: &str, samples: &[f32], grid: &Grid) {
    assert_eq!(
        samples.len(),
        grid.total_frames.saturating_mul(usize::from(CHANNEL_COUNT)),
        "{name} must match grid length"
    );
}

pub(super) fn validate_report(report: &PackReport) -> Result<(), Box<dyn std::error::Error>> {
    let mut required_signal_metrics = vec![
        ("tr909", report.tr909),
        ("w30", report.w30),
        ("source_first_mix", report.source_first_mix),
        ("full_mix", report.full_mix),
    ];
    if report.mc202_bass_pressure.applied {
        required_signal_metrics.push(("mc202", report.mc202));
    }
    for (name, metrics) in required_signal_metrics {
        if metrics.signal.rms <= MIN_SIGNAL_RMS {
            return Err(format!("{name} rendered near silence").into());
        }
    }

    if report.source_first_generated_to_source_rms_ratio
        >= MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO
    {
        return Err(format!(
            "source-first mix generated/source RMS ratio {:.6} exceeds {:.6}",
            report.source_first_generated_to_source_rms_ratio,
            MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO
        )
        .into());
    }

    if report.support_generated_to_source_rms_ratio < MIN_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO {
        return Err(format!(
            "generated-support mix generated/source RMS ratio {:.6} is below {:.6}",
            report.support_generated_to_source_rms_ratio, MIN_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO
        )
        .into());
    }

    if report.support_generated_to_source_rms_ratio >= MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO {
        return Err(format!(
            "generated-support mix generated/source RMS ratio {:.6} exceeds {:.6}",
            report.support_generated_to_source_rms_ratio, MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO
        )
        .into());
    }

    if report.full_mix.low_band.rms <= MIN_LOW_BAND_RMS {
        return Err(format!(
            "full mix low-band support is too weak: low-band RMS {:.6}",
            report.full_mix.low_band.rms
        )
        .into());
    }

    for (name, limiter) in [
        ("source-first mix", report.source_first_master_bus_limiter),
        ("generated-support mix", report.full_mix_master_bus_limiter),
    ] {
        if limiter.post.clip_count > 0 || limiter.post.peak_abs > limiter.ceiling + 0.000_001 {
            return Err(format!(
                "{name} master bus limiter failed to control output: post peak {:.6}, post clips {}",
                limiter.post.peak_abs, limiter.post.clip_count
            )
            .into());
        }

        if limiter.post.rms <= MIN_SIGNAL_RMS {
            return Err(format!(
                "{name} master bus limiter collapsed output to near silence: post RMS {:.6}",
                limiter.post.rms
            )
            .into());
        }
    }

    if report.tr909_source_grid_alignment.hit_ratio < SOURCE_GRID_OUTPUT_MIN_HIT_RATIO {
        return Err(format!(
            "TR-909 source-grid alignment hit ratio {:.6} is below {:.6}",
            report.tr909_source_grid_alignment.hit_ratio, SOURCE_GRID_OUTPUT_MIN_HIT_RATIO
        )
        .into());
    }

    if !report.tr909_kick_pressure.applied {
        return Err(format!(
            "TR-909 kick pressure was too weak: low-band ratio {:.6}, anchors {}",
            report.tr909_kick_pressure.low_band_rms_ratio, report.tr909_kick_pressure.anchor_count
        )
        .into());
    }

    if !report.tr909_source_accent_dynamics.applied {
        return Err(format!(
            "TR-909 source-accent dynamics were too flat: distinct accents {} span {:.6}",
            report.tr909_source_accent_dynamics.distinct_accent_count,
            report.tr909_source_accent_dynamics.accent_span
        )
        .into());
    }

    if !report.tr909_rendered_drum_pressure.applied {
        return Err(format!(
            "TR-909 rendered drum pressure was too weak or masked the source: profile {}, contribution {:.6}, min {:.6}, tr909 low {:.6}, low min {:.6}, full low {:.6}, grid hit {:.6}, source-first ratio {:.6}, support ratio {:.6}, all-lane applied {}, generated/w30 {:.6}, generated/w30 min {:.6}, mc202 contribution {:.6}, lane min {:.6}",
            report.tr909_source_profile.reason,
            report.tr909_rendered_drum_pressure.support_mix_tr909_contribution_ratio,
            report
                .tr909_rendered_drum_pressure
                .min_required_support_mix_tr909_contribution_ratio,
            report.tr909_rendered_drum_pressure.tr909_low_band_rms,
            report
                .tr909_rendered_drum_pressure
                .min_required_tr909_low_band_rms,
            report.tr909_rendered_drum_pressure.full_mix_low_band_rms,
            report.tr909_rendered_drum_pressure.tr909_source_grid_hit_ratio,
            report.tr909_rendered_drum_pressure.source_first_generated_to_source_rms_ratio,
            report.tr909_rendered_drum_pressure.support_generated_to_source_rms_ratio,
            report.all_lane_mix_movement.applied,
            report
                .all_lane_mix_movement
                .generated_to_w30_contribution_ratio,
            report
                .all_lane_mix_movement
                .min_required_generated_to_w30_ratio,
            report.all_lane_mix_movement.mc202_contribution_ratio,
            report
                .all_lane_mix_movement
                .min_required_lane_contribution_ratio
        )
        .into());
    }

    if !report.all_lane_mix_movement.applied {
        return Err(format!(
            "all-lane mix movement proof failed: delta {:.6}, correlation {:.6}, tr909 {:.6}, mc202 {:.6}, w30 {:.6}, generated/w30 {:.6}",
            report.all_lane_mix_movement.source_first_to_support_rms_delta,
            report.all_lane_mix_movement.source_first_to_support_correlation,
            report.all_lane_mix_movement.tr909_contribution_ratio,
            report.all_lane_mix_movement.mc202_contribution_ratio,
            report.all_lane_mix_movement.w30_contribution_ratio,
            report.all_lane_mix_movement.generated_to_w30_contribution_ratio
        )
        .into());
    }

    if report.mc202_bass_pressure.applied
        && report.mc202_source_grid_alignment.hit_ratio < SOURCE_GRID_OUTPUT_MIN_HIT_RATIO
    {
        return Err(format!(
            "MC-202 source-grid alignment hit ratio {:.6} is below {:.6}",
            report.mc202_source_grid_alignment.hit_ratio, SOURCE_GRID_OUTPUT_MIN_HIT_RATIO
        )
        .into());
    }

    if !report.mc202_source_contour.applied {
        return Err(format!(
            "MC-202 source contour was too weak: delta RMS {:.6}",
            report.mc202_source_contour.source_contour_delta_rms
        )
        .into());
    }

    if report.mc202_bass_pressure.pattern_origin != Mc202PatternOrigin::SourceDerived
        || !report
            .mc202_bass_pressure
            .source_expression_render_plan_applied
        || report.mc202_bass_pressure.source_failure_fallback
    {
        return Err(format!(
            "MC-202 source-expression origin is unavailable: origin {}, render plan applied {}, fallback {}",
            report.mc202_bass_pressure.pattern_origin.label(),
            report
                .mc202_bass_pressure
                .source_expression_render_plan_applied,
            report.mc202_bass_pressure.source_failure_fallback
        )
        .into());
    }

    if !report.mc202_bass_pressure.applied {
        return Err(format!(
            "MC-202 bass pressure was too weak: low-band RMS {:.6}, low/mid energy ratio {:.6}, reinforcement gain {:.6}",
            report.mc202_bass_pressure.low_band_rms,
            report.mc202_bass_pressure.low_to_mid_energy_ratio,
            report.mc202_bass_pressure.pressure_reinforcement_gain
        )
        .into());
    }

    if report.w30_source_grid_alignment.hit_ratio < SOURCE_GRID_OUTPUT_MIN_HIT_RATIO {
        return Err(format!(
            "W-30 source-grid alignment hit ratio {:.6} is below {:.6}",
            report.w30_source_grid_alignment.hit_ratio, SOURCE_GRID_OUTPUT_MIN_HIT_RATIO
        )
        .into());
    }

    if !report.w30_source_loop_closure.passed {
        return Err(format!(
            "W-30 source loop closure failed: edge_delta_abs {:.6} edge_abs_max {:.6}",
            report.w30_source_loop_closure.edge_delta_abs,
            report.w30_source_loop_closure.edge_abs_max
        )
        .into());
    }

    if !report.w30_source_trigger_variation.applied {
        return Err(format!(
            "W-30 source trigger plan was not applied: beat anchors {} skipped {} distinct source offsets {} quantized offset {:.6} ms",
            report.w30_source_trigger_variation.beat_anchor_trigger_count,
            report.w30_source_trigger_variation.skipped_beat_anchor_count,
            report.w30_source_trigger_variation.distinct_bar_pattern_count,
            report.w30_source_trigger_variation.max_quantized_offset_ms
        )
        .into());
    }

    if !report.w30_source_slice_choice.applied {
        return Err(format!(
            "W-30 source slice choice was too static: unique offsets {} span {} samples",
            report.w30_source_slice_choice.unique_source_offset_count,
            report.w30_source_slice_choice.selected_offset_span_samples
        )
        .into());
    }

    if !report.w30_source_accent_dynamics.applied {
        return Err(format!(
            "W-30 source-accent dynamics were too flat: distinct velocities {} span {:.6}",
            report.w30_source_accent_dynamics.distinct_velocity_count,
            report.w30_source_accent_dynamics.velocity_span
        )
        .into());
    }

    Ok(())
}
