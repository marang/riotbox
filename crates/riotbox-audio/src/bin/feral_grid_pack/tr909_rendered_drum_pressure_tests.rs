use riotbox_audio::tr909::{
    Tr909PatternAdoption, Tr909PhraseVariation, Tr909SourceSupportContext,
    Tr909SourceSupportProfile,
};

use super::{
    config::{CHANNEL_COUNT, PATTERN_ORIGIN_SOURCE_DERIVED, SAMPLE_RATE},
    grid::{Grid, frames_for_beats},
    mix_movement_evidence::{
        ALL_LANE_MIX_MAX_CORRELATION, ALL_LANE_MIX_MIN_GENERATED_TO_W30_RATIO,
        ALL_LANE_MIX_MIN_LANE_CONTRIBUTION_RATIO, ALL_LANE_MIX_MIN_RMS_DELTA,
        AllLaneMixMovementProof,
    },
    render_measurements::render_metrics,
    source_aware_tr909::SourceAwareTr909Profile,
    source_grid_output_drift::{
        SOURCE_GRID_OUTPUT_MAX_PEAK_OFFSET_MS, SourceGridOutputDriftMetrics,
    },
    tr909_kick_pressure::{
        TR909_SOURCE_ACCENT_MIN_ACCENT_SPAN,
        TR909_SOURCE_EVIDENCE_ROLE_PROFILE_AND_ACCENT_DYNAMICS, Tr909KickPressureProof,
        Tr909SourceAccentDynamicsProof,
    },
    tr909_rendered_drum_pressure::{
        TR909_RENDERED_DRUM_PRESSURE_MIN_LOW_BAND_RMS,
        TR909_RENDERED_DRUM_PRESSURE_MIN_STEADY_LOW_BAND_RMS,
        TR909_RENDERED_DRUM_PRESSURE_MIN_SUPPORT_CONTRIBUTION_RATIO,
        Tr909RenderedDrumPressureInput, tr909_rendered_drum_pressure_proof,
    },
};

#[test]
fn rendered_drum_pressure_accepts_source_derived_pressure_that_survives_mix() {
    let grid = Grid::new(128.0, 4, 2).expect("grid");
    let samples = low_pulse_samples(&grid, 0.040);
    let proof = tr909_rendered_drum_pressure_proof(Tr909RenderedDrumPressureInput {
        source_profile: source_profile(Tr909SourceSupportProfile::DropDrive),
        tr909_metrics: render_metrics(&samples, &grid),
        full_mix_metrics: render_metrics(&samples, &grid),
        kick_pressure: source_derived_kick_pressure(true),
        accent_dynamics: source_derived_accent_dynamics(true),
        all_lane_mix_movement: all_lane_mix_movement_with_tr909(0.060, true),
        tr909_source_grid_alignment: grid_alignment(1.0),
        source_first_generated_to_source_rms_ratio: 0.030,
        support_generated_to_source_rms_ratio: 0.180,
    });

    assert!(proof.applied, "{proof:?}");
    assert_eq!(proof.pattern_origin, PATTERN_ORIGIN_SOURCE_DERIVED);
    assert!(proof.support_mix_tr909_contribution_ratio >= 0.050);
}

#[test]
fn rendered_drum_pressure_rejects_source_derived_pressure_buried_in_support_mix() {
    let grid = Grid::new(128.0, 4, 2).expect("grid");
    let samples = low_pulse_samples(&grid, 0.040);
    let proof = tr909_rendered_drum_pressure_proof(Tr909RenderedDrumPressureInput {
        source_profile: source_profile(Tr909SourceSupportProfile::DropDrive),
        tr909_metrics: render_metrics(&samples, &grid),
        full_mix_metrics: render_metrics(&samples, &grid),
        kick_pressure: source_derived_kick_pressure(true),
        accent_dynamics: source_derived_accent_dynamics(true),
        all_lane_mix_movement: all_lane_mix_movement_with_tr909(0.020, true),
        tr909_source_grid_alignment: grid_alignment(1.0),
        source_first_generated_to_source_rms_ratio: 0.030,
        support_generated_to_source_rms_ratio: 0.180,
    });

    assert!(!proof.applied, "{proof:?}");
    assert_eq!(
        proof.reason,
        "tr909_rendered_drum_pressure_too_weak_or_masks_source"
    );
}

#[test]
fn rendered_drum_pressure_requires_steady_pulse_support_to_hit_normal_floor() {
    let grid = Grid::new(128.0, 4, 2).expect("grid");
    let samples = low_pulse_samples(&grid, 0.040);
    let proof = tr909_rendered_drum_pressure_proof(Tr909RenderedDrumPressureInput {
        source_profile: source_profile(Tr909SourceSupportProfile::SteadyPulse),
        tr909_metrics: render_metrics(&samples, &grid),
        full_mix_metrics: render_metrics(&samples, &grid),
        kick_pressure: source_derived_kick_pressure(true),
        accent_dynamics: source_derived_accent_dynamics(true),
        all_lane_mix_movement: all_lane_mix_movement_with_tr909(0.049, true),
        tr909_source_grid_alignment: grid_alignment(1.0),
        source_first_generated_to_source_rms_ratio: 0.030,
        support_generated_to_source_rms_ratio: 0.180,
    });

    assert!(!proof.applied, "{proof:?}");
    assert_eq!(
        proof.min_required_support_mix_tr909_contribution_ratio,
        TR909_RENDERED_DRUM_PRESSURE_MIN_SUPPORT_CONTRIBUTION_RATIO
    );
    assert_eq!(
        proof.min_required_tr909_low_band_rms,
        TR909_RENDERED_DRUM_PRESSURE_MIN_STEADY_LOW_BAND_RMS
    );
}

#[test]
fn rendered_drum_pressure_uses_break_lift_low_band_floor() {
    let grid = Grid::new(128.0, 4, 2).expect("grid");
    let samples = low_pulse_samples(&grid, 0.030);
    let proof = tr909_rendered_drum_pressure_proof(Tr909RenderedDrumPressureInput {
        source_profile: source_profile(Tr909SourceSupportProfile::BreakLift),
        tr909_metrics: render_metrics(&samples, &grid),
        full_mix_metrics: render_metrics(&low_pulse_samples(&grid, 0.060), &grid),
        kick_pressure: source_derived_kick_pressure(true),
        accent_dynamics: source_derived_accent_dynamics(true),
        all_lane_mix_movement: all_lane_mix_movement_with_tr909(0.055, true),
        tr909_source_grid_alignment: grid_alignment(1.0),
        source_first_generated_to_source_rms_ratio: 0.030,
        support_generated_to_source_rms_ratio: 0.180,
    });

    assert!(proof.applied, "{proof:?}");
    assert_eq!(
        proof.min_required_tr909_low_band_rms,
        TR909_RENDERED_DRUM_PRESSURE_MIN_LOW_BAND_RMS
    );
}

fn low_pulse_samples(grid: &Grid, amp: f32) -> Vec<f32> {
    let mut samples = vec![0.0; grid.total_frames * usize::from(CHANNEL_COUNT)];
    for beat in 0..grid.total_beats {
        let start = frames_for_beats(grid.bpm, beat);
        for frame_offset in 0..1600 {
            let frame = start + frame_offset;
            if frame >= grid.total_frames {
                break;
            }
            let envelope = (1.0 - frame_offset as f32 / 1600.0).max(0.0);
            let value = (frame_offset as f32 * 55.0 / SAMPLE_RATE as f32 * std::f32::consts::TAU)
                .sin()
                * amp
                * envelope;
            let index = frame * usize::from(CHANNEL_COUNT);
            samples[index] = value;
            samples[index + 1] = value;
        }
    }
    samples
}

fn source_derived_kick_pressure(applied: bool) -> Tr909KickPressureProof {
    Tr909KickPressureProof {
        pattern_origin: PATTERN_ORIGIN_SOURCE_DERIVED,
        source_evidence_role: TR909_SOURCE_EVIDENCE_ROLE_PROFILE_AND_ACCENT_DYNAMICS,
        source_profile_reason: "source_low_drive",
        applied,
        anchor_count: 8,
        pressure_gain: 0.018,
        pre_low_band_rms: 0.0020,
        post_low_band_rms: 0.0034,
        low_band_rms_delta: 0.0014,
        low_band_rms_ratio: 1.70,
        post_peak_abs: 0.10,
        reason: "tr909_low_drive_pressure",
    }
}

fn source_profile(support_profile: Tr909SourceSupportProfile) -> SourceAwareTr909Profile {
    SourceAwareTr909Profile {
        signal_rms: 0.2,
        low_band_rms: 0.1,
        onset_count: 8,
        event_density_per_bar: 4.0,
        low_band_energy_ratio: 0.2,
        mid_band_energy_ratio: 0.4,
        high_band_energy_ratio: 0.4,
        support_profile,
        support_context: Tr909SourceSupportContext::TransportBar,
        pattern_adoption: Tr909PatternAdoption::SupportPulse,
        phrase_variation: Tr909PhraseVariation::PhraseAnchor,
        drum_bus_level: 0.70,
        slam_intensity: 0.16,
        reason: "source_test_profile",
    }
}

fn source_derived_accent_dynamics(applied: bool) -> Tr909SourceAccentDynamicsProof {
    Tr909SourceAccentDynamicsProof {
        pattern_origin: PATTERN_ORIGIN_SOURCE_DERIVED,
        applied,
        anchor_count: 8,
        distinct_accent_count: 3,
        min_accent: 0.2,
        max_accent: 1.1,
        accent_span: 0.9,
        min_required_accent_span: TR909_SOURCE_ACCENT_MIN_ACCENT_SPAN,
        source_energy_span: 0.5,
        reason: "tr909_source_accented_support_dynamics",
    }
}

fn all_lane_mix_movement_with_tr909(
    tr909_contribution_ratio: f32,
    applied: bool,
) -> AllLaneMixMovementProof {
    AllLaneMixMovementProof {
        applied,
        reason: "all_lane_mix_movement_proof",
        source_first_to_support_rms_delta: 0.030,
        source_first_to_support_correlation: 0.950,
        tr909_contribution_ratio,
        mc202_contribution_ratio: 0.060,
        w30_contribution_ratio: 0.480,
        generated_to_w30_contribution_ratio: 0.240,
        min_required_rms_delta: ALL_LANE_MIX_MIN_RMS_DELTA,
        max_allowed_correlation: ALL_LANE_MIX_MAX_CORRELATION,
        min_required_lane_contribution_ratio: ALL_LANE_MIX_MIN_LANE_CONTRIBUTION_RATIO,
        min_required_generated_to_w30_ratio: ALL_LANE_MIX_MIN_GENERATED_TO_W30_RATIO,
    }
}

fn grid_alignment(hit_ratio: f32) -> SourceGridOutputDriftMetrics {
    SourceGridOutputDriftMetrics {
        beat_count: 8,
        hit_count: (8.0 * hit_ratio) as u32,
        hit_ratio,
        max_peak_offset_ms: 5.0,
        max_allowed_peak_offset_ms: SOURCE_GRID_OUTPUT_MAX_PEAK_OFFSET_MS,
    }
}
