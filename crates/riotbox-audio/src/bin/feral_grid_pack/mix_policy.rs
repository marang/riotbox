#[derive(Clone, Copy, Debug)]
struct MixPolicy {
    tr909_gain: f32,
    tr909_low_gain: f32,
    mc202_gain: f32,
    mc202_low_gain: f32,
    w30_gain: f32,
    drive: f32,
    output_gain: f32,
}

const SOURCE_FIRST_MIX_POLICY: MixPolicy = MixPolicy {
    tr909_gain: 0.075,
    tr909_low_gain: 0.030,
    mc202_gain: 0.026,
    mc202_low_gain: 0.010,
    w30_gain: 1.28,
    drive: 1.18,
    output_gain: 0.88,
};

const GENERATED_SUPPORT_MIX_POLICY: MixPolicy = MixPolicy {
    tr909_gain: 0.845,
    tr909_low_gain: 0.330,
    mc202_gain: 0.400,
    mc202_low_gain: 0.120,
    w30_gain: 1.46,
    drive: 2.18,
    output_gain: 0.94,
};

const ALL_LANE_MIX_MIN_RMS_DELTA: f32 = 0.012;
const ALL_LANE_MIX_MAX_CORRELATION: f32 = 0.999;
const ALL_LANE_MIX_MIN_LANE_CONTRIBUTION_RATIO: f32 = 0.015;
const ALL_LANE_MIX_MIN_GENERATED_TO_W30_RATIO: f32 = 0.08;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
struct AllLaneMixMovementProof {
    applied: bool,
    reason: &'static str,
    source_first_to_support_rms_delta: f32,
    source_first_to_support_correlation: f32,
    tr909_contribution_ratio: f32,
    mc202_contribution_ratio: f32,
    w30_contribution_ratio: f32,
    generated_to_w30_contribution_ratio: f32,
    min_required_rms_delta: f32,
    max_allowed_correlation: f32,
    min_required_lane_contribution_ratio: f32,
    min_required_generated_to_w30_ratio: f32,
}

fn render_source_first_mix(tr909: &[f32], mc202: &[f32], w30: &[f32]) -> Vec<f32> {
    render_mix(tr909, mc202, w30, SOURCE_FIRST_MIX_POLICY)
}

fn render_generated_support_mix(tr909: &[f32], mc202: &[f32], w30: &[f32]) -> Vec<f32> {
    render_mix(tr909, mc202, w30, GENERATED_SUPPORT_MIX_POLICY)
}

fn render_mix(tr909: &[f32], mc202: &[f32], w30: &[f32], policy: MixPolicy) -> Vec<f32> {
    debug_assert_eq!(tr909.len(), w30.len());
    debug_assert_eq!(tr909.len(), mc202.len());

    let tr909_low = one_pole_lowpass(tr909, 165.0);
    let mc202_low = one_pole_lowpass(mc202, 165.0);
    tr909
        .iter()
        .zip(tr909_low.iter())
        .zip(mc202.iter())
        .zip(mc202_low.iter())
        .zip(w30.iter())
        .map(|((((tr909, tr909_low), mc202), mc202_low), w30)| {
            let mixed = tr909 * policy.tr909_gain
                + tr909_low * policy.tr909_low_gain
                + mc202 * policy.mc202_gain
                + mc202_low * policy.mc202_low_gain
                + w30 * policy.w30_gain;
            (mixed * policy.drive).tanh() * policy.output_gain
        })
        .collect()
}

fn all_lane_mix_movement_proof(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    source_first_mix: &[f32],
    generated_support_mix: &[f32],
    grid: &Grid,
) -> AllLaneMixMovementProof {
    debug_assert_eq!(tr909.len(), mc202.len());
    debug_assert_eq!(tr909.len(), w30.len());
    debug_assert_eq!(tr909.len(), source_first_mix.len());
    debug_assert_eq!(tr909.len(), generated_support_mix.len());

    let tr909_contribution = mix_component_rms(
        tr909,
        GENERATED_SUPPORT_MIX_POLICY.tr909_gain,
        GENERATED_SUPPORT_MIX_POLICY.tr909_low_gain,
        grid,
    );
    let mc202_contribution = mix_component_rms(
        mc202,
        GENERATED_SUPPORT_MIX_POLICY.mc202_gain,
        GENERATED_SUPPORT_MIX_POLICY.mc202_low_gain,
        grid,
    );
    let w30_contribution = mix_source_component_rms(
        w30,
        GENERATED_SUPPORT_MIX_POLICY.w30_gain,
        grid,
    );
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

fn mix_component_rms(samples: &[f32], gain: f32, low_gain: f32, grid: &Grid) -> f32 {
    let low = one_pole_lowpass(samples, 165.0);
    let weighted: Vec<_> = samples
        .iter()
        .zip(low.iter())
        .map(|(sample, low)| sample * gain + low * low_gain)
        .collect();
    signal_metrics_with_grid(
        &weighted,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.bpm,
        grid.beats_per_bar,
    )
    .rms
}

fn mix_source_component_rms(samples: &[f32], gain: f32, grid: &Grid) -> f32 {
    let weighted: Vec<_> = samples.iter().map(|sample| sample * gain).collect();
    signal_metrics_with_grid(
        &weighted,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.bpm,
        grid.beats_per_bar,
    )
    .rms
}

fn rms_delta(left: &[f32], right: &[f32], grid: &Grid) -> f32 {
    let delta: Vec<_> = left
        .iter()
        .zip(right.iter())
        .map(|(left, right)| left - right)
        .collect();
    signal_metrics_with_grid(
        &delta,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.bpm,
        grid.beats_per_bar,
    )
    .rms
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

fn source_first_generated_to_source_rms_ratio(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    grid: &Grid,
) -> f32 {
    generated_to_source_rms_ratio(tr909, mc202, w30, grid, SOURCE_FIRST_MIX_POLICY)
}

fn support_generated_to_source_rms_ratio(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    grid: &Grid,
) -> f32 {
    generated_to_source_rms_ratio(tr909, mc202, w30, grid, GENERATED_SUPPORT_MIX_POLICY)
}

fn generated_to_source_rms_ratio(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    grid: &Grid,
    policy: MixPolicy,
) -> f32 {
    debug_assert_eq!(tr909.len(), w30.len());
    debug_assert_eq!(tr909.len(), mc202.len());

    let tr909_low = one_pole_lowpass(tr909, 165.0);
    let mc202_low = one_pole_lowpass(mc202, 165.0);
    let generated: Vec<_> = tr909
        .iter()
        .zip(tr909_low.iter())
        .zip(mc202.iter())
        .zip(mc202_low.iter())
        .map(|(((tr909, tr909_low), mc202), mc202_low)| {
            tr909 * policy.tr909_gain
                + tr909_low * policy.tr909_low_gain
                + mc202 * policy.mc202_gain
                + mc202_low * policy.mc202_low_gain
        })
        .collect();
    let source: Vec<_> = w30.iter().map(|w30| w30 * policy.w30_gain).collect();
    let generated_rms = signal_metrics_with_grid(
        &generated,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.bpm,
        grid.beats_per_bar,
    )
    .rms;
    let source_rms = signal_metrics_with_grid(
        &source,
        SAMPLE_RATE,
        CHANNEL_COUNT,
        grid.bpm,
        grid.beats_per_bar,
    )
    .rms;

    generated_rms / source_rms.max(f32::EPSILON)
}
