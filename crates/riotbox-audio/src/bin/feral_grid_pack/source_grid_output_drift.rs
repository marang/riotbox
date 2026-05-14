const SOURCE_GRID_OUTPUT_MAX_PEAK_OFFSET_MS: f32 = 70.0;
const SOURCE_GRID_OUTPUT_MIN_HIT_RATIO: f32 = 0.50;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
struct SourceGridOutputDriftMetrics {
    beat_count: u32,
    hit_count: u32,
    hit_ratio: f32,
    max_peak_offset_ms: f32,
    max_allowed_peak_offset_ms: f32,
}

fn source_grid_output_drift_metrics(samples: &[f32], grid: &Grid) -> SourceGridOutputDriftMetrics {
    if samples.is_empty() || grid.total_beats == 0 {
        return SourceGridOutputDriftMetrics {
            max_allowed_peak_offset_ms: SOURCE_GRID_OUTPUT_MAX_PEAK_OFFSET_MS,
            ..SourceGridOutputDriftMetrics::default()
        };
    }

    let channels = usize::from(CHANNEL_COUNT);
    let global_peak = samples
        .chunks_exact(channels)
        .map(frame_peak_abs)
        .fold(0.0_f32, f32::max);
    if global_peak <= f32::EPSILON {
        return SourceGridOutputDriftMetrics {
            beat_count: grid.total_beats,
            max_allowed_peak_offset_ms: SOURCE_GRID_OUTPUT_MAX_PEAK_OFFSET_MS,
            ..SourceGridOutputDriftMetrics::default()
        };
    }

    let hit_threshold = (global_peak * 0.08).max(MIN_SIGNAL_RMS);
    let window_frames = ((SOURCE_GRID_OUTPUT_MAX_PEAK_OFFSET_MS / 1000.0) * SAMPLE_RATE as f32)
        .round()
        .max(1.0) as usize;
    let mut hit_count = 0;
    let mut max_peak_offset_ms = 0.0_f32;

    for beat in 0..grid.total_beats {
        let beat_frame = frames_for_beats(grid.bpm, beat);
        if let Some((offset_frames, peak)) = strongest_peak_near_frame(samples, beat_frame, window_frames)
            && peak >= hit_threshold
        {
            hit_count += 1;
            let offset_ms = offset_frames as f32 * 1000.0 / SAMPLE_RATE as f32;
            max_peak_offset_ms = max_peak_offset_ms.max(offset_ms);
        }
    }

    SourceGridOutputDriftMetrics {
        beat_count: grid.total_beats,
        hit_count,
        hit_ratio: hit_count as f32 / grid.total_beats as f32,
        max_peak_offset_ms,
        max_allowed_peak_offset_ms: SOURCE_GRID_OUTPUT_MAX_PEAK_OFFSET_MS,
    }
}

fn source_grid_alignment_report(
    tr909: &[f32],
    w30: &[f32],
    generated_support_mix: &[f32],
    grid: &Grid,
) -> SourceGridAlignmentReport {
    SourceGridAlignmentReport {
        tr909_source_grid_alignment: source_grid_output_drift_metrics(tr909, grid),
        w30_source_grid_alignment: source_grid_output_drift_metrics(w30, grid),
        source_grid_output_drift: source_grid_output_drift_metrics(generated_support_mix, grid),
    }
}

fn strongest_peak_near_frame(
    samples: &[f32],
    center_frame: usize,
    window_frames: usize,
) -> Option<(usize, f32)> {
    let channels = usize::from(CHANNEL_COUNT);
    if channels == 0 {
        return None;
    }

    let frame_count = samples.len() / channels;
    if frame_count == 0 {
        return None;
    }

    let start = center_frame.saturating_sub(window_frames);
    let end = center_frame
        .saturating_add(window_frames)
        .saturating_add(1)
        .min(frame_count);
    let mut strongest: Option<(usize, f32)> = None;

    for frame_index in start..end {
        let sample_start = frame_index * channels;
        let peak = frame_peak_abs(&samples[sample_start..sample_start + channels]);
        let offset = center_frame.abs_diff(frame_index);
        match strongest {
            Some((_, best_peak)) if peak <= best_peak => {}
            _ => strongest = Some((offset, peak)),
        }
    }

    strongest
}

fn frame_peak_abs(frame: &[f32]) -> f32 {
    frame.iter().map(|sample| sample.abs()).fold(0.0_f32, f32::max)
}
