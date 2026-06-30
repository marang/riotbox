fn synthetic_break_source(frame_count: usize) -> Vec<f32> {
    let mut samples = Vec::with_capacity(frame_count * usize::from(CHANNEL_COUNT));
    for frame in 0..frame_count {
        let phase = frame as f32 / SAMPLE_RATE as f32;
        let bar_pulse = frame % frames_for_beats(128.0, 1);
        let kick = if bar_pulse < 1_200 {
            ((1.0 - bar_pulse as f32 / 1_200.0).max(0.0) * 0.9)
                * (phase * 74.0 * std::f32::consts::TAU).sin()
        } else {
            0.0
        };
        let grit = (phase * 510.0 * std::f32::consts::TAU).sin() * 0.08;
        let sample = (kick + grit) * 1.01;
        samples.push(sample);
        samples.push(sample * 0.97);
    }
    samples
}

fn bar_pattern_samples(
    grid: &Grid,
    is_active_frame: impl Fn(u32, usize, usize) -> bool,
) -> Vec<f32> {
    let mut samples = Vec::with_capacity(grid.total_frames * usize::from(CHANNEL_COUNT));
    for bar in 0..grid.bars {
        let bar_frames = grid.bar_frame_count(bar);
        for frame in 0..bar_frames {
            let sample = if is_active_frame(bar, frame, bar_frames) {
                0.5
            } else {
                0.0
            };
            samples.push(sample);
            samples.push(sample);
        }
    }
    samples
}

fn tone_samples(frequency_hz: f32, frame_count: usize) -> Vec<f32> {
    let mut samples = Vec::with_capacity(frame_count * usize::from(CHANNEL_COUNT));
    for frame in 0..frame_count {
        let phase = frame as f32 / SAMPLE_RATE as f32;
        let sample = (phase * frequency_hz * std::f32::consts::TAU).sin() * 0.5;
        samples.push(sample);
        samples.push(sample);
    }
    samples
}

fn high_click_source(grid: &Grid) -> Vec<f32> {
    let mut samples = vec![0.0; grid.total_frames * usize::from(CHANNEL_COUNT)];
    let click_spacing = (grid.bar_frame_count(0) / 8).max(1);
    for frame in (0..grid.total_frames).step_by(click_spacing) {
        for offset in 0..96 {
            let target = frame + offset;
            if target >= grid.total_frames {
                break;
            }
            let envelope = 1.0 - offset as f32 / 96.0;
            let sample = if offset.is_multiple_of(2) {
                0.55 * envelope
            } else {
                -0.55 * envelope
            };
            let index = target * usize::from(CHANNEL_COUNT);
            samples[index] = sample;
            samples[index + 1] = sample;
        }
    }
    samples
}
