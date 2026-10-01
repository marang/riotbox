use super::{
    config::{CHANNEL_COUNT, SAMPLE_RATE},
    grid::Grid,
    signal_filter::one_pole_lowpass,
};
use riotbox_audio::runtime::{
    MasterBusLimiterReport, apply_master_bus_soft_limiter_with_report, signal_metrics_with_grid,
};

#[derive(Clone, Copy, Debug)]
pub(super) struct MixPolicy {
    pub(super) tr909_gain: f32,
    pub(super) tr909_low_gain: f32,
    pub(super) mc202_gain: f32,
    pub(super) mc202_low_gain: f32,
    pub(super) w30_gain: f32,
    pub(super) drive: f32,
    pub(super) output_gain: f32,
}

pub(super) fn render_mix_with_master_bus_report(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    policy: MixPolicy,
) -> (Vec<f32>, MasterBusLimiterReport) {
    let mut mix = render_mix_pre_master_bus_limiter(tr909, mc202, w30, policy);
    let report = apply_master_bus_soft_limiter_with_report(&mut mix);
    (mix, report)
}

fn render_mix_pre_master_bus_limiter(
    tr909: &[f32],
    mc202: &[f32],
    w30: &[f32],
    policy: MixPolicy,
) -> Vec<f32> {
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

pub(super) fn mix_component_rms(samples: &[f32], gain: f32, low_gain: f32, grid: &Grid) -> f32 {
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

pub(super) fn mix_source_component_rms(samples: &[f32], gain: f32, grid: &Grid) -> f32 {
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

pub(super) fn generated_to_source_rms_ratio(
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
