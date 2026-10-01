use super::{
    bar_variation_metrics::{BarVariationMetrics, bar_variation_metrics},
    config::{CHANNEL_COUNT, SAMPLE_RATE},
    grid::Grid,
    signal_filter::one_pole_lowpass,
    spectral_energy_metrics::{SpectralEnergyMetrics, spectral_energy_metrics},
};
use riotbox_audio::runtime::{OfflineAudioMetrics, signal_metrics_with_grid};

#[derive(Clone, Copy, Debug)]
pub(super) struct RenderMetrics {
    pub(super) signal: OfflineAudioMetrics,
    pub(super) low_band: OfflineAudioMetrics,
    pub(super) bar_variation: BarVariationMetrics,
    pub(super) spectral_energy: SpectralEnergyMetrics,
}

pub(super) fn render_metrics(samples: &[f32], grid: &Grid) -> RenderMetrics {
    RenderMetrics {
        signal: signal_metrics_with_grid(
            samples,
            SAMPLE_RATE,
            CHANNEL_COUNT,
            grid.bpm,
            grid.beats_per_bar,
        ),
        low_band: signal_metrics_with_grid(
            &one_pole_lowpass(samples, 165.0),
            SAMPLE_RATE,
            CHANNEL_COUNT,
            grid.bpm,
            grid.beats_per_bar,
        ),
        bar_variation: bar_variation_metrics(samples, grid),
        spectral_energy: spectral_energy_metrics(samples),
    }
}

pub(super) fn rms_delta(left: &[f32], right: &[f32], grid: &Grid) -> f32 {
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
