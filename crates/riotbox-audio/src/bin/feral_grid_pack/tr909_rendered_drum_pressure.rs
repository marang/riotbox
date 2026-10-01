use serde::Serialize;

use riotbox_audio::tr909::Tr909SourceSupportProfile;

use super::{
    config::{
        MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO, MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
        MIN_LOW_BAND_RMS, PATTERN_ORIGIN_PRIMITIVE_RENDERER, PATTERN_ORIGIN_SOURCE_DERIVED,
    },
    mix_movement_evidence::AllLaneMixMovementProof,
    render_measurements::RenderMetrics,
    source_aware_tr909::SourceAwareTr909Profile,
    source_grid_output_drift::{SOURCE_GRID_OUTPUT_MIN_HIT_RATIO, SourceGridOutputDriftMetrics},
    tr909_kick_pressure::{Tr909KickPressureProof, Tr909SourceAccentDynamicsProof},
};

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub(super) struct Tr909RenderedDrumPressureProof {
    pub(super) applied: bool,
    pub(super) reason: &'static str,
    pub(super) pattern_origin: &'static str,
    pub(super) source_evidence_role: &'static str,
    pub(super) support_mix_tr909_contribution_ratio: f32,
    pub(super) support_generated_to_source_rms_ratio: f32,
    pub(super) source_first_generated_to_source_rms_ratio: f32,
    pub(super) source_first_masking_headroom: f32,
    pub(super) tr909_low_band_rms: f32,
    pub(super) full_mix_low_band_rms: f32,
    pub(super) tr909_source_grid_hit_ratio: f32,
    max_tr909_source_grid_peak_offset_ms: f32,
    pub(super) min_required_support_mix_tr909_contribution_ratio: f32,
    pub(super) min_required_tr909_low_band_rms: f32,
    max_source_first_generated_to_source_rms_ratio: f32,
    max_support_generated_to_source_rms_ratio: f32,
}

pub(super) const TR909_RENDERED_DRUM_PRESSURE_MIN_SUPPORT_CONTRIBUTION_RATIO: f32 = 0.050;

pub(super) const TR909_RENDERED_DRUM_PRESSURE_MIN_LOW_BAND_RMS: f32 = 0.0030;

pub(super) const TR909_RENDERED_DRUM_PRESSURE_MIN_STEADY_LOW_BAND_RMS: f32 = 0.0017;

pub(super) const TR909_RENDERED_DRUM_PRESSURE_SOURCE_EVIDENCE_ROLE: &str =
    "tr909_source_profile_accent_dynamics_and_rendered_mix_pressure";

const TR909_RENDERED_DRUM_PRESSURE_PRIMITIVE_EVIDENCE_ROLE: &str = "tr909_primitive_control_only";

#[derive(Clone, Copy, Debug)]
pub(super) struct Tr909RenderedDrumPressureInput {
    pub(super) source_profile: SourceAwareTr909Profile,
    pub(super) tr909_metrics: RenderMetrics,
    pub(super) full_mix_metrics: RenderMetrics,
    pub(super) kick_pressure: Tr909KickPressureProof,
    pub(super) accent_dynamics: Tr909SourceAccentDynamicsProof,
    pub(super) all_lane_mix_movement: AllLaneMixMovementProof,
    pub(super) tr909_source_grid_alignment: SourceGridOutputDriftMetrics,
    pub(super) source_first_generated_to_source_rms_ratio: f32,
    pub(super) support_generated_to_source_rms_ratio: f32,
}

pub(super) fn tr909_rendered_drum_pressure_proof(
    input: Tr909RenderedDrumPressureInput,
) -> Tr909RenderedDrumPressureProof {
    let source_first_masking_headroom = MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO
        - input.source_first_generated_to_source_rms_ratio;
    let min_required_support_mix_tr909_contribution_ratio =
        tr909_rendered_drum_pressure_min_support_contribution(input.source_profile);
    let min_required_tr909_low_band_rms =
        tr909_rendered_drum_pressure_min_low_band_rms(input.source_profile);
    let support_mix_tr909_contribution_ratio = input.all_lane_mix_movement.tr909_contribution_ratio;
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
        max_tr909_source_grid_peak_offset_ms: input.tr909_source_grid_alignment.max_peak_offset_ms,
        min_required_support_mix_tr909_contribution_ratio,
        min_required_tr909_low_band_rms,
        max_source_first_generated_to_source_rms_ratio:
            MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO,
        max_support_generated_to_source_rms_ratio: MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
    }
}

fn tr909_rendered_drum_pressure_min_support_contribution(profile: SourceAwareTr909Profile) -> f32 {
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
