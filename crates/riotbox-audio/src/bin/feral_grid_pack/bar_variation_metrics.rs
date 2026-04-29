#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
struct BarVariationMetrics {
    bar_similarity: f32,
    identical_bar_run_length: usize,
}

fn bar_variation_metrics(samples: &[f32], grid: &Grid) -> BarVariationMetrics {
    if grid.bars < 2 || samples.is_empty() {
        return BarVariationMetrics::default();
    }

    let mut similarity_sum = 0.0;
    let mut similarity_count = 0;
    let mut current_identical_run = 1;
    let mut max_identical_run = 1;

    for bar in 0..grid.bars - 1 {
        let left = bar_slice(samples, grid, bar);
        let right = bar_slice(samples, grid, bar + 1);
        let similarity = bar_similarity(left, right);
        similarity_sum += similarity;
        similarity_count += 1;

        if similarity >= 0.999 {
            current_identical_run += 1;
            max_identical_run = max_identical_run.max(current_identical_run);
        } else {
            current_identical_run = 1;
        }
    }

    BarVariationMetrics {
        bar_similarity: if similarity_count == 0 {
            0.0
        } else {
            similarity_sum / similarity_count as f32
        },
        identical_bar_run_length: max_identical_run,
    }
}

fn bar_slice<'samples>(samples: &'samples [f32], grid: &Grid, bar: u32) -> &'samples [f32] {
    let channels = usize::from(CHANNEL_COUNT);
    let start = grid.bar_start_frame(bar).saturating_mul(channels);
    let end = grid
        .bar_end_frame(bar)
        .saturating_mul(channels)
        .min(samples.len());
    if start >= end || start >= samples.len() {
        &[]
    } else {
        &samples[start..end]
    }
}

fn bar_similarity(left: &[f32], right: &[f32]) -> f32 {
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
