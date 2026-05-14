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
    tr909_gain: 0.12,
    tr909_low_gain: 0.05,
    mc202_gain: 0.04,
    mc202_low_gain: 0.02,
    w30_gain: 1.18,
    drive: 1.18,
    output_gain: 0.88,
};

const GENERATED_SUPPORT_MIX_POLICY: MixPolicy = MixPolicy {
    tr909_gain: 0.58,
    tr909_low_gain: 0.26,
    mc202_gain: 0.30,
    mc202_low_gain: 0.10,
    w30_gain: 1.36,
    drive: 2.20,
    output_gain: 0.94,
};

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
