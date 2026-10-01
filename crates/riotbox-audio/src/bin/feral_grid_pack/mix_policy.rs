use super::{
    config::{
        MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO, MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
    },
    grid::Grid,
    mc202_source_contour::Mc202SourceContourProfile,
    mix_components::{MixPolicy, generated_to_source_rms_ratio, render_mix_with_master_bus_report},
};
use riotbox_audio::{mc202::Mc202ContourHint, runtime::MasterBusLimiterReport};

const SOURCE_FIRST_MIX_POLICY: MixPolicy = MixPolicy {
    tr909_gain: 0.075,
    tr909_low_gain: 0.030,
    mc202_gain: 0.026,
    mc202_low_gain: 0.010,
    w30_gain: 1.28,
    drive: 1.18,
    output_gain: 0.88,
};

pub(super) const GENERATED_SUPPORT_MIX_POLICY: MixPolicy = MixPolicy {
    tr909_gain: 1.150,
    tr909_low_gain: 0.460,
    mc202_gain: 0.210,
    mc202_low_gain: 0.065,
    w30_gain: 1.50,
    drive: 2.18,
    output_gain: 0.94,
};

const DROP_SUPPORT_TARGET_GENERATED_TO_SOURCE_RMS_RATIO: f32 = 0.147;

#[cfg(test)]
pub(super) fn render_source_first_mix(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    grid: &Grid,
) -> Vec<f32> {
    render_mix_with_master_bus_report(
        tr909,
        mc202,
        w30,
        source_first_mix_policy_for_stems(tr909, mc202, w30, grid),
    )
    .0
}

pub(super) fn render_source_first_mix_with_master_bus_report(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    grid: &Grid,
) -> (Vec<f32>, MasterBusLimiterReport) {
    render_mix_with_master_bus_report(
        tr909,
        mc202,
        w30,
        source_first_mix_policy_for_stems(tr909, mc202, w30, grid),
    )
}

#[cfg(test)]
pub(super) fn render_generated_support_mix(tr909: &[f32], mc202: &[f32], w30: &[f32]) -> Vec<f32> {
    render_mix_with_master_bus_report(tr909, mc202, w30, GENERATED_SUPPORT_MIX_POLICY).0
}

pub(super) fn generated_support_mix_policy_for_source_contour_and_stems(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    grid: &Grid,
    source_contour: Mc202SourceContourProfile,
) -> MixPolicy {
    let mut policy = GENERATED_SUPPORT_MIX_POLICY;
    match source_contour.contour_hint {
        Mc202ContourHint::Drop => {
            policy = boost_generated_support_to_floor(
                tr909,
                mc202,
                w30,
                grid,
                policy,
                DROP_SUPPORT_TARGET_GENERATED_TO_SOURCE_RMS_RATIO,
            );
        }
        Mc202ContourHint::Lift => {
            policy.mc202_gain *= 1.16;
            policy.mc202_low_gain *= 1.10;
            policy.w30_gain *= 0.96;
        }
        Mc202ContourHint::Hold | Mc202ContourHint::Neutral => {
            policy.tr909_gain *= 1.18;
            policy.tr909_low_gain *= 1.12;
            policy.mc202_gain *= 1.38;
            policy.mc202_low_gain *= 1.22;
            policy.w30_gain *= 0.86;
        }
    }
    cap_generated_support_to_ceiling(
        tr909,
        mc202,
        w30,
        grid,
        policy,
        MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO * 0.98,
    )
}

fn boost_generated_support_to_floor(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    grid: &Grid,
    mut policy: MixPolicy,
    target_ratio: f32,
) -> MixPolicy {
    let current_ratio = generated_to_source_rms_ratio(tr909, mc202, w30, grid, policy);
    if current_ratio <= f32::EPSILON || current_ratio >= target_ratio {
        return policy;
    }

    let multiplier = target_ratio / current_ratio;
    policy.tr909_gain *= multiplier;
    policy.tr909_low_gain *= multiplier;
    policy.mc202_gain *= multiplier;
    policy.mc202_low_gain *= multiplier;
    policy
}

fn source_first_mix_policy_for_stems(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    grid: &Grid,
) -> MixPolicy {
    cap_generated_support_to_ceiling(
        tr909,
        mc202,
        w30,
        grid,
        SOURCE_FIRST_MIX_POLICY,
        MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO * 0.95,
    )
}

fn cap_generated_support_to_ceiling(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    grid: &Grid,
    mut policy: MixPolicy,
    target_ratio: f32,
) -> MixPolicy {
    let current_ratio = generated_to_source_rms_ratio(tr909, mc202, w30, grid, policy);
    if current_ratio <= target_ratio || current_ratio <= f32::EPSILON {
        return policy;
    }

    let multiplier = target_ratio / current_ratio;
    policy.tr909_gain *= multiplier;
    policy.tr909_low_gain *= multiplier;
    policy.mc202_gain *= multiplier;
    policy.mc202_low_gain *= multiplier;
    policy
}

pub(super) fn source_first_generated_to_source_rms_ratio(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    grid: &Grid,
) -> f32 {
    generated_to_source_rms_ratio(
        tr909,
        mc202,
        w30,
        grid,
        source_first_mix_policy_for_stems(tr909, mc202, w30, grid),
    )
}

#[cfg(test)]
pub(super) fn support_generated_to_source_rms_ratio(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    grid: &Grid,
) -> f32 {
    generated_to_source_rms_ratio(tr909, mc202, w30, grid, GENERATED_SUPPORT_MIX_POLICY)
}

pub(super) fn support_generated_to_source_rms_ratio_for_source_contour(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    grid: &Grid,
    source_contour: Mc202SourceContourProfile,
) -> f32 {
    generated_to_source_rms_ratio(
        tr909,
        mc202,
        w30,
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
