#[cfg(test)]
use super::mix_policy::GENERATED_SUPPORT_MIX_POLICY;
use super::{
    config::{CHANNEL_COUNT, SAMPLE_RATE},
    grid::Grid,
    mc202_source_contour::Mc202SourceContourProfile,
    mix_components::{MixPolicy, mix_component_rms, mix_source_component_rms},
    mix_policy::generated_support_mix_policy_for_source_contour_and_stems,
    render_measurements::rms_delta,
};
use riotbox_audio::runtime::signal_metrics_with_grid;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub(super) struct AllLaneMixMovementProof {
    pub(super) applied: bool,
    pub(super) reason: &'static str,
    pub(super) source_first_to_support_rms_delta: f32,
    pub(super) source_first_to_support_correlation: f32,
    pub(super) tr909_contribution_ratio: f32,
    pub(super) mc202_contribution_ratio: f32,
    pub(super) w30_contribution_ratio: f32,
    pub(super) generated_to_w30_contribution_ratio: f32,
    pub(super) min_required_rms_delta: f32,
    pub(super) max_allowed_correlation: f32,
    pub(super) min_required_lane_contribution_ratio: f32,
    pub(super) min_required_generated_to_w30_ratio: f32,
}

pub(super) const ALL_LANE_MIX_MIN_RMS_DELTA: f32 = 0.012;

pub(super) const ALL_LANE_MIX_MAX_CORRELATION: f32 = 0.999;

pub(super) const ALL_LANE_MIX_MIN_LANE_CONTRIBUTION_RATIO: f32 = 0.020;

pub(super) const ALL_LANE_MIX_MIN_GENERATED_TO_W30_RATIO: f32 = 0.145;

#[cfg(test)]
pub(super) fn all_lane_mix_movement_proof(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    source_first_mix: &[f32],
    generated_support_mix: &[f32],
    grid: &Grid,
) -> AllLaneMixMovementProof {
    all_lane_mix_movement_proof_with_policy(
        tr909,
        mc202,
        w30,
        source_first_mix,
        generated_support_mix,
        grid,
        GENERATED_SUPPORT_MIX_POLICY,
    )
}

pub(super) fn all_lane_mix_movement_proof_for_source_contour(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    source_first_mix: &[f32],
    generated_support_mix: &[f32],
    grid: &Grid,
    source_contour: Mc202SourceContourProfile,
) -> AllLaneMixMovementProof {
    all_lane_mix_movement_proof_with_policy(
        tr909,
        mc202,
        w30,
        source_first_mix,
        generated_support_mix,
        grid,
        generated_support_mix_policy_for_source_contour_and_stems(
            tr909,
            mc202,
            w30,
            grid,
            source_contour,
        ),
    )
}

fn all_lane_mix_movement_proof_with_policy(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    source_first_mix: &[f32],
    generated_support_mix: &[f32],
    grid: &Grid,
    generated_support_policy: MixPolicy,
) -> AllLaneMixMovementProof {
    debug_assert_eq!(tr909.len(), mc202.len());
    debug_assert_eq!(tr909.len(), w30.len());
    debug_assert_eq!(tr909.len(), source_first_mix.len());
    debug_assert_eq!(tr909.len(), generated_support_mix.len());

    let tr909_contribution = mix_component_rms(
        tr909,
        generated_support_policy.tr909_gain,
        generated_support_policy.tr909_low_gain,
        grid,
    );
    let mc202_contribution = mix_component_rms(
        mc202,
        generated_support_policy.mc202_gain,
        generated_support_policy.mc202_low_gain,
        grid,
    );
    let w30_contribution = mix_source_component_rms(w30, generated_support_policy.w30_gain, grid);
    let support_mix_rms = signal_metrics_with_grid(
        generated_support_mix,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.bpm,
        grid.beats_per_bar,
    )
    .rms
    .max(f32::EPSILON);
    let source_first_to_support_rms_delta =
        rms_delta(source_first_mix, generated_support_mix, grid);
    let source_first_to_support_correlation =
        sample_correlation(source_first_mix, generated_support_mix);
    let generated_to_w30_contribution_ratio =
        (tr909_contribution + mc202_contribution) / w30_contribution.max(f32::EPSILON);

    let applied = source_first_to_support_rms_delta >= ALL_LANE_MIX_MIN_RMS_DELTA
        && source_first_to_support_correlation <= ALL_LANE_MIX_MAX_CORRELATION
        && tr909_contribution / support_mix_rms >= ALL_LANE_MIX_MIN_LANE_CONTRIBUTION_RATIO
        && mc202_contribution / support_mix_rms >= ALL_LANE_MIX_MIN_LANE_CONTRIBUTION_RATIO
        && w30_contribution / support_mix_rms >= ALL_LANE_MIX_MIN_LANE_CONTRIBUTION_RATIO
        && generated_to_w30_contribution_ratio >= ALL_LANE_MIX_MIN_GENERATED_TO_W30_RATIO;

    AllLaneMixMovementProof {
        applied,
        reason: if applied {
            "all_lane_mix_movement_proof"
        } else {
            "all_lane_mix_movement_too_weak"
        },
        source_first_to_support_rms_delta,
        source_first_to_support_correlation,
        tr909_contribution_ratio: tr909_contribution / support_mix_rms,
        mc202_contribution_ratio: mc202_contribution / support_mix_rms,
        w30_contribution_ratio: w30_contribution / support_mix_rms,
        generated_to_w30_contribution_ratio,
        min_required_rms_delta: ALL_LANE_MIX_MIN_RMS_DELTA,
        max_allowed_correlation: ALL_LANE_MIX_MAX_CORRELATION,
        min_required_lane_contribution_ratio: ALL_LANE_MIX_MIN_LANE_CONTRIBUTION_RATIO,
        min_required_generated_to_w30_ratio: ALL_LANE_MIX_MIN_GENERATED_TO_W30_RATIO,
    }
}

fn sample_correlation(left: &[f32], right: &[f32]) -> f32 {
    let sample_count = left.len().min(right.len());
    if sample_count == 0 {
        return 0.0;
    }

    let mut dot = 0.0;
    let mut left_energy = 0.0;
    let mut right_energy = 0.0;
    for index in 0..sample_count {
        let left = left[index];
        let right = right[index];
        dot += left * right;
        left_energy += left * left;
        right_energy += right * right;
    }

    let denominator = (left_energy * right_energy).sqrt();
    if denominator <= f32::EPSILON {
        0.0
    } else {
        (dot / denominator).abs().clamp(0.0, 1.0)
    }
}
