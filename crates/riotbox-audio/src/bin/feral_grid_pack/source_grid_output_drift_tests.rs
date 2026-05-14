#[cfg(test)]
mod source_grid_output_drift_tests {
    use super::*;

    #[test]
    fn source_grid_output_drift_accepts_grid_locked_pulses() {
        let grid = Grid::new(120.0, 4, 2).expect("grid");
        let samples = pulse_train(&grid, 0);

        let metrics = source_grid_output_drift_metrics(&samples, &grid);

        assert_eq!(metrics.beat_count, 8);
        assert_eq!(metrics.hit_count, 8);
        assert_eq!(metrics.hit_ratio, 1.0);
        assert_eq!(metrics.max_peak_offset_ms, 0.0);
    }

    #[test]
    fn source_grid_output_drift_flags_obvious_half_beat_shift() {
        let grid = Grid::new(120.0, 4, 2).expect("grid");
        let half_beat_frames = frames_for_beats(grid.bpm, 1) / 2;
        let samples = pulse_train(&grid, half_beat_frames);

        let metrics = source_grid_output_drift_metrics(&samples, &grid);

        assert_eq!(metrics.beat_count, 8);
        assert_eq!(metrics.hit_count, 0);
        assert_eq!(metrics.hit_ratio, 0.0);
    }

    #[test]
    fn pack_level_source_grid_output_drift_uses_generated_support_mix() {
        let grid = Grid::new(120.0, 4, 2).expect("grid");
        let half_beat_frames = frames_for_beats(grid.bpm, 1) / 2;
        let tr909 = pulse_train(&grid, 0);
        let w30 = pulse_train(&grid, half_beat_frames);
        let generated_support_mix = pulse_train(&grid, half_beat_frames);

        let report = source_grid_alignment_report(&tr909, &w30, &generated_support_mix, &grid);

        assert_eq!(report.tr909_source_grid_alignment.hit_ratio, 1.0);
        assert_eq!(report.w30_source_grid_alignment.hit_ratio, 0.0);
        assert_eq!(report.source_grid_output_drift.hit_ratio, 0.0);
    }

    fn pulse_train(grid: &Grid, frame_offset: usize) -> Vec<f32> {
        let mut samples = vec![0.0; grid.total_frames * usize::from(CHANNEL_COUNT)];
        for beat in 0..grid.total_beats {
            let frame = frames_for_beats(grid.bpm, beat).saturating_add(frame_offset);
            if frame >= grid.total_frames {
                continue;
            }
            let sample_index = frame * usize::from(CHANNEL_COUNT);
            samples[sample_index] = 1.0;
            samples[sample_index + 1] = 0.95;
        }
        samples
    }
}
