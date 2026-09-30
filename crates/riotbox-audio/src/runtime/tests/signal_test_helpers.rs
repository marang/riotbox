use crate::runtime::signal_metrics;

pub(super) fn adjacent_sample_delta_rms(samples: &[f32]) -> f32 {
    let square_sum = samples
        .windows(2)
        .map(|window| (window[1] - window[0]).powi(2))
        .sum::<f32>();
    (square_sum / samples.len().saturating_sub(1).max(1) as f32).sqrt()
}

pub(super) fn test_signal_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt()
}

pub(super) fn test_band_rms(
    samples: &[f32],
    low_hz: f32,
    high_hz: f32,
    sample_rate: u32,
    channel_count: usize,
) -> f32 {
    if samples.is_empty() || sample_rate == 0 || channel_count == 0 {
        return 0.0;
    }
    let dt = 1.0 / sample_rate as f32;
    let low_alpha = dt / (1.0 / (std::f32::consts::TAU * low_hz.max(1.0)) + dt);
    let high_alpha = dt / (1.0 / (std::f32::consts::TAU * high_hz.max(1.0)) + dt);
    let mut low_state = vec![0.0_f32; channel_count];
    let mut high_state = vec![0.0_f32; channel_count];
    let mut energy = 0.0_f32;
    for (index, sample) in samples.iter().enumerate() {
        let channel = index % channel_count;
        low_state[channel] += low_alpha * (*sample - low_state[channel]);
        high_state[channel] += high_alpha * (*sample - high_state[channel]);
        let band = high_state[channel] - low_state[channel];
        energy += band * band;
    }
    (energy / samples.len() as f32).sqrt()
}

pub(super) fn test_max_frame_delta(samples: &[f32], channel_count: usize) -> f32 {
    if channel_count == 0 {
        return 0.0;
    }
    samples
        .chunks_exact(channel_count)
        .map(|frame| frame[0])
        .collect::<Vec<_>>()
        .windows(2)
        .fold(0.0_f32, |max_delta, frame| {
            max_delta.max((frame[1] - frame[0]).abs())
        })
}

pub(super) fn tr909_low_band_rms(samples: &[f32], sample_rate: u32, channel_count: usize) -> f32 {
    signal_metrics(&tr909_one_pole_lowpass(
        samples,
        145.0,
        sample_rate,
        channel_count,
    ))
    .rms
}

pub(super) fn tr909_high_band_proxy_rms(
    samples: &[f32],
    sample_rate: u32,
    channel_count: usize,
) -> f32 {
    let low = tr909_one_pole_lowpass(samples, 1_800.0, sample_rate, channel_count);
    let high = samples
        .iter()
        .zip(low.iter())
        .map(|(sample, low)| sample - low)
        .collect::<Vec<_>>();
    signal_metrics(&high).rms
}

fn tr909_one_pole_lowpass(
    samples: &[f32],
    cutoff_hz: f32,
    sample_rate: u32,
    channel_count: usize,
) -> Vec<f32> {
    if sample_rate == 0 || channel_count == 0 {
        return vec![0.0; samples.len()];
    }
    let rc = 1.0 / (std::f32::consts::TAU * cutoff_hz.max(1.0));
    let dt = 1.0 / sample_rate as f32;
    let alpha = dt / (rc + dt);
    let mut previous = vec![0.0; channel_count];
    samples
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            let channel = index % channel_count;
            previous[channel] += alpha * (sample - previous[channel]);
            previous[channel]
        })
        .collect()
}

pub(super) fn region_delta_rms(first: &[f32], second: &[f32]) -> f32 {
    let square_sum = first
        .iter()
        .zip(second)
        .map(|(left, right)| {
            let delta = left - right;
            delta * delta
        })
        .sum::<f32>();
    (square_sum / first.len().max(1) as f32).sqrt()
}
