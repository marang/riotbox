use super::config::{CHANNEL_COUNT, SAMPLE_RATE};

pub(super) fn one_pole_lowpass(samples: &[f32], cutoff_hz: f32) -> Vec<f32> {
    let dt = 1.0 / SAMPLE_RATE as f32;
    let rc = 1.0 / (std::f32::consts::TAU * cutoff_hz.max(1.0));
    let alpha = dt / (rc + dt);
    let mut state = [0.0_f32; CHANNEL_COUNT as usize];
    let mut output = Vec::with_capacity(samples.len());

    for frame in samples.as_chunks::<{ CHANNEL_COUNT as usize }>().0.iter() {
        for (channel, sample) in frame.iter().enumerate() {
            state[channel] += alpha * (*sample - state[channel]);
            output.push(state[channel]);
        }
    }

    output
}
