#[cfg(test)]
mod tr909_source_grid_consumer_tests {
    use super::*;

    #[test]
    fn tr909_source_support_render_lands_on_selected_source_grid() {
        let grid = Grid::new(128.0, 4, 2).expect("grid");
        let samples = render_tr909_source_support(&grid, source_locked_tr909_profile());

        let metrics = source_grid_output_drift_metrics(&samples, &grid);

        assert_eq!(metrics.beat_count, grid.total_beats);
        assert!(
            metrics.hit_ratio >= SOURCE_GRID_OUTPUT_MIN_HIT_RATIO,
            "TR-909 hit ratio {} should stay on the selected source grid",
            metrics.hit_ratio
        );
        assert!(
            metrics.max_peak_offset_ms <= SOURCE_GRID_OUTPUT_MAX_PEAK_OFFSET_MS,
            "TR-909 max peak offset {} ms should stay inside source-grid tolerance",
            metrics.max_peak_offset_ms
        );
    }

    #[test]
    fn source_grid_alignment_gate_rejects_half_beat_shift() {
        let grid = Grid::new(128.0, 4, 2).expect("grid");
        let samples = sparse_source_grid_pulses(&grid);
        let shifted = delay_by_frames(&samples, frames_for_beats(grid.bpm, 1) / 2);

        let metrics = source_grid_output_drift_metrics(&shifted, &grid);

        assert!(
            metrics.hit_ratio < SOURCE_GRID_OUTPUT_MIN_HIT_RATIO,
            "half-beat-shifted grid pulse should not pass TR-909 source-grid alignment gate"
        );
    }

    fn source_locked_tr909_profile() -> SourceAwareTr909Profile {
        SourceAwareTr909Profile {
            signal_rms: 0.10,
            low_band_rms: 0.08,
            onset_count: 8,
            event_density_per_bar: 4.0,
            low_band_energy_ratio: 0.55,
            mid_band_energy_ratio: 0.30,
            high_band_energy_ratio: 0.15,
            support_profile: Tr909SourceSupportProfile::DropDrive,
            support_context: Tr909SourceSupportContext::TransportBar,
            pattern_adoption: Tr909PatternAdoption::MainlineDrive,
            phrase_variation: Tr909PhraseVariation::PhraseDrive,
            drum_bus_level: 0.84,
            slam_intensity: 0.22,
            reason: "source_low_drive",
        }
    }

    fn delay_by_frames(samples: &[f32], frame_delay: usize) -> Vec<f32> {
        let channels = usize::from(CHANNEL_COUNT);
        let sample_delay = frame_delay.saturating_mul(channels);
        let mut shifted = vec![0.0; samples.len()];
        if sample_delay >= samples.len() {
            return shifted;
        }
        shifted[sample_delay..].copy_from_slice(&samples[..samples.len() - sample_delay]);
        shifted
    }

    fn sparse_source_grid_pulses(grid: &Grid) -> Vec<f32> {
        let channels = usize::from(CHANNEL_COUNT);
        let mut samples = vec![0.0; grid.total_frames.saturating_mul(channels)];
        for beat in 0..grid.total_beats {
            let frame = frames_for_beats(grid.bpm, beat);
            let sample_index = frame.saturating_mul(channels);
            if sample_index + 1 >= samples.len() {
                continue;
            }
            samples[sample_index] = 1.0;
            samples[sample_index + 1] = 0.95;
        }
        samples
    }
}
