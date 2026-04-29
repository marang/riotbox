#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
struct SpectralEnergyMetrics {
    low_band_energy_ratio: f32,
    mid_band_energy_ratio: f32,
    high_band_energy_ratio: f32,
}

fn spectral_energy_metrics(samples: &[f32]) -> SpectralEnergyMetrics {
    let total_energy = signal_energy(samples);
    if total_energy <= f32::EPSILON {
        return SpectralEnergyMetrics::default();
    }

    let low = one_pole_lowpass(samples, 180.0);
    let treble_floor = one_pole_lowpass(samples, 2_800.0);
    let high = samples
        .iter()
        .zip(treble_floor.iter())
        .map(|(sample, floor)| sample - floor)
        .collect::<Vec<_>>();
    let mid = samples
        .iter()
        .zip(low.iter())
        .zip(high.iter())
        .map(|((sample, low), high)| sample - low - high)
        .collect::<Vec<_>>();

    let low_ratio = signal_energy(&low) / total_energy;
    let mid_ratio = signal_energy(&mid) / total_energy;
    let high_ratio = signal_energy(&high) / total_energy;
    let ratio_sum = low_ratio + mid_ratio + high_ratio;
    if ratio_sum <= f32::EPSILON {
        return SpectralEnergyMetrics::default();
    }

    SpectralEnergyMetrics {
        low_band_energy_ratio: low_ratio / ratio_sum,
        mid_band_energy_ratio: mid_ratio / ratio_sum,
        high_band_energy_ratio: high_ratio / ratio_sum,
    }
}

fn signal_energy(samples: &[f32]) -> f32 {
    samples.iter().map(|sample| sample * sample).sum()
}
