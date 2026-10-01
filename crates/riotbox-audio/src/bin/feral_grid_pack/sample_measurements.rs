//! Shared scalar sample measurements for bounded offline QA controls.

pub(super) fn mono_frames(samples: &[f32], channel_count: usize) -> Vec<f32> {
    let channel_count = channel_count.max(1);
    samples
        .chunks_exact(channel_count)
        .map(|frame| frame.iter().sum::<f32>() / channel_count as f32)
        .collect()
}

pub(super) fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt()
}

pub(super) fn peak_abs(samples: &[f32]) -> f32 {
    samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0_f32, f32::max)
}

pub(super) fn positive_abs_delta(samples: &[f32]) -> f32 {
    if samples.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0;
    for pair in samples.windows(2) {
        total += (pair[1].abs() - pair[0].abs()).max(0.0);
    }
    total / (samples.len() - 1) as f32
}
