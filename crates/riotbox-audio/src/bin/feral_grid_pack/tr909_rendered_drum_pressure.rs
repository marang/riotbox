#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
struct Tr909RenderedDrumPressureProof {
    applied: bool,
    reason: &'static str,
    pattern_origin: &'static str,
    source_evidence_role: &'static str,
    support_mix_tr909_contribution_ratio: f32,
    support_generated_to_source_rms_ratio: f32,
    source_first_generated_to_source_rms_ratio: f32,
    source_first_masking_headroom: f32,
    tr909_low_band_rms: f32,
    full_mix_low_band_rms: f32,
    tr909_source_grid_hit_ratio: f32,
    max_tr909_source_grid_peak_offset_ms: f32,
    min_required_support_mix_tr909_contribution_ratio: f32,
    min_required_tr909_low_band_rms: f32,
    max_source_first_generated_to_source_rms_ratio: f32,
    max_support_generated_to_source_rms_ratio: f32,
}

const TR909_RENDERED_DRUM_PRESSURE_MIN_SUPPORT_CONTRIBUTION_RATIO: f32 = 0.050;
const TR909_RENDERED_DRUM_PRESSURE_MIN_LOW_BAND_RMS: f32 = 0.0030;
const TR909_RENDERED_DRUM_PRESSURE_MIN_STEADY_LOW_BAND_RMS: f32 = 0.0017;
const TR909_RENDERED_DRUM_PRESSURE_SOURCE_EVIDENCE_ROLE: &str =
    "tr909_source_profile_accent_dynamics_and_rendered_mix_pressure";
const TR909_RENDERED_DRUM_PRESSURE_PRIMITIVE_EVIDENCE_ROLE: &str =
    "tr909_primitive_control_only";

#[derive(Clone, Copy, Debug)]
struct Tr909RenderedDrumPressureInput {
    source_profile: SourceAwareTr909Profile,
    tr909_metrics: RenderMetrics,
    full_mix_metrics: RenderMetrics,
    kick_pressure: Tr909KickPressureProof,
    accent_dynamics: Tr909SourceAccentDynamicsProof,
    all_lane_mix_movement: AllLaneMixMovementProof,
    tr909_source_grid_alignment: SourceGridOutputDriftMetrics,
    source_first_generated_to_source_rms_ratio: f32,
    support_generated_to_source_rms_ratio: f32,
}

fn tr909_rendered_drum_pressure_proof(
    input: Tr909RenderedDrumPressureInput,
) -> Tr909RenderedDrumPressureProof {
    let source_first_masking_headroom = MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO
        - input.source_first_generated_to_source_rms_ratio;
    let min_required_support_mix_tr909_contribution_ratio =
        tr909_rendered_drum_pressure_min_support_contribution(input.source_profile);
    let min_required_tr909_low_band_rms =
        tr909_rendered_drum_pressure_min_low_band_rms(input.source_profile);
    let support_mix_tr909_contribution_ratio =
        input.all_lane_mix_movement.tr909_contribution_ratio;
    let source_derived = input.kick_pressure.pattern_origin == PATTERN_ORIGIN_SOURCE_DERIVED
        && input.accent_dynamics.pattern_origin == PATTERN_ORIGIN_SOURCE_DERIVED;
    let applied = source_derived
        && input.kick_pressure.applied
        && input.accent_dynamics.applied
        && input.all_lane_mix_movement.applied
        && support_mix_tr909_contribution_ratio
            >= min_required_support_mix_tr909_contribution_ratio
        && input.tr909_metrics.low_band.rms >= min_required_tr909_low_band_rms
        && input.full_mix_metrics.low_band.rms >= MIN_LOW_BAND_RMS
        && input.tr909_source_grid_alignment.hit_ratio >= SOURCE_GRID_OUTPUT_MIN_HIT_RATIO
        && input.source_first_generated_to_source_rms_ratio
            <= MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO
        && input.support_generated_to_source_rms_ratio <= MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO;

    Tr909RenderedDrumPressureProof {
        applied,
        reason: if applied {
            "tr909_rendered_drum_pressure_survives_source_first_mix"
        } else {
            "tr909_rendered_drum_pressure_too_weak_or_masks_source"
        },
        pattern_origin: if source_derived {
            PATTERN_ORIGIN_SOURCE_DERIVED
        } else {
            PATTERN_ORIGIN_PRIMITIVE_RENDERER
        },
        source_evidence_role: if source_derived {
            TR909_RENDERED_DRUM_PRESSURE_SOURCE_EVIDENCE_ROLE
        } else {
            TR909_RENDERED_DRUM_PRESSURE_PRIMITIVE_EVIDENCE_ROLE
        },
        support_mix_tr909_contribution_ratio,
        support_generated_to_source_rms_ratio: input.support_generated_to_source_rms_ratio,
        source_first_generated_to_source_rms_ratio: input
            .source_first_generated_to_source_rms_ratio,
        source_first_masking_headroom,
        tr909_low_band_rms: input.tr909_metrics.low_band.rms,
        full_mix_low_band_rms: input.full_mix_metrics.low_band.rms,
        tr909_source_grid_hit_ratio: input.tr909_source_grid_alignment.hit_ratio,
        max_tr909_source_grid_peak_offset_ms: input
            .tr909_source_grid_alignment
            .max_peak_offset_ms,
        min_required_support_mix_tr909_contribution_ratio,
        min_required_tr909_low_band_rms,
        max_source_first_generated_to_source_rms_ratio:
            MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO,
        max_support_generated_to_source_rms_ratio: MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
    }
}

fn tr909_rendered_drum_pressure_min_support_contribution(
    profile: SourceAwareTr909Profile,
) -> f32 {
    match profile.support_profile {
        Tr909SourceSupportProfile::DropDrive
        | Tr909SourceSupportProfile::BreakLift
        | Tr909SourceSupportProfile::SteadyPulse => {
            TR909_RENDERED_DRUM_PRESSURE_MIN_SUPPORT_CONTRIBUTION_RATIO
        }
    }
}

fn tr909_rendered_drum_pressure_min_low_band_rms(profile: SourceAwareTr909Profile) -> f32 {
    match profile.support_profile {
        Tr909SourceSupportProfile::DropDrive | Tr909SourceSupportProfile::BreakLift => {
            TR909_RENDERED_DRUM_PRESSURE_MIN_LOW_BAND_RMS
        }
        Tr909SourceSupportProfile::SteadyPulse => {
            TR909_RENDERED_DRUM_PRESSURE_MIN_STEADY_LOW_BAND_RMS
        }
    }
}

#[cfg(test)]
mod tr909_rendered_drum_pressure_tests {
    use super::*;

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
                let value = (frame_offset as f32 * 55.0 / SAMPLE_RATE as f32
                    * std::f32::consts::TAU)
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
}
