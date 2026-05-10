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
    fn tr909_source_support_consumes_trusted_groove_offset_without_leaving_grid() {
        let grid = Grid::new(128.0, 4, 2).expect("grid");
        let policy = tr909_groove_timing_policy(
            source_timing_grid_decision(128.0),
            &groove_evidence_with_offset(12.5),
        );
        assert!(policy.applied);
        assert_eq!(policy.reason, "source_timing_groove_residual");
        assert_eq!(policy.source_subdivision, Some("eighth"));

        let base = render_tr909_source_support(&grid, source_locked_tr909_profile());
        let shifted = apply_tr909_groove_timing(&base, policy);
        let metrics = source_grid_output_drift_metrics(&shifted, &grid);

        assert_ne!(shifted, base, "groove timing should move TR-909 support");
        assert!(
            metrics.hit_ratio >= SOURCE_GRID_OUTPUT_MIN_HIT_RATIO,
            "groove-shifted TR-909 should stay inside source-grid hit tolerance"
        );
        assert!(
            metrics.max_peak_offset_ms >= 10.0,
            "expected audible timing offset, got {:.3} ms",
            metrics.max_peak_offset_ms
        );
        assert!(
            metrics.max_peak_offset_ms <= SOURCE_GRID_OUTPUT_MAX_PEAK_OFFSET_MS,
            "groove-shifted TR-909 exceeded source-grid tolerance"
        );
    }

    #[test]
    fn tr909_groove_timing_stays_inactive_without_source_timing_grid() {
        let policy = tr909_groove_timing_policy(
            static_grid_decision(128.0),
            &groove_evidence_with_offset(12.5),
        );

        assert!(!policy.applied);
        assert_eq!(policy.reason, "not_source_timing_grid");
        assert_eq!(policy.offset_ms, 0.0);
    }

    #[test]
    fn tr909_groove_timing_stays_inactive_for_cautious_source_timing_grid() {
        let policy = tr909_groove_timing_policy(
            cautious_source_timing_grid_decision(128.0),
            &groove_evidence_with_offset(12.5),
        );

        assert!(!policy.applied);
        assert_eq!(policy.reason, "source_timing_not_locked");
        assert_eq!(policy.offset_ms, 0.0);
    }

    #[test]
    fn tr909_groove_timing_accepts_early_residuals_inside_grid_tolerance() {
        let grid = Grid::new(128.0, 4, 2).expect("grid");
        let policy = tr909_groove_timing_policy(
            source_timing_grid_decision(128.0),
            &groove_evidence_with_offset(-8.0),
        );
        let shifted = apply_tr909_groove_timing(
            &render_tr909_source_support(&grid, source_locked_tr909_profile()),
            policy,
        );
        let metrics = source_grid_output_drift_metrics(&shifted, &grid);

        assert!(policy.applied);
        assert!(
            metrics.hit_ratio >= SOURCE_GRID_OUTPUT_MIN_HIT_RATIO,
            "early groove-shifted TR-909 should stay inside source-grid hit tolerance"
        );
        assert!(
            metrics.max_peak_offset_ms <= SOURCE_GRID_OUTPUT_MAX_PEAK_OFFSET_MS,
            "early groove-shifted TR-909 exceeded source-grid tolerance"
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

    fn source_timing_grid_decision(bpm: f32) -> GridBpmDecision {
        GridBpmDecision {
            bpm,
            source: GridBpmSource::SourceTiming,
            reason: GridBpmDecisionReason::SourceTimingReady,
            source_primary_bpm: Some(bpm),
            source_delta_bpm: Some(0.0),
        }
    }

    fn static_grid_decision(bpm: f32) -> GridBpmDecision {
        GridBpmDecision {
            bpm,
            source: GridBpmSource::StaticDefault,
            reason: GridBpmDecisionReason::SourceTimingRequiresManualConfirm,
            source_primary_bpm: Some(bpm),
            source_delta_bpm: Some(0.0),
        }
    }

    fn cautious_source_timing_grid_decision(bpm: f32) -> GridBpmDecision {
        GridBpmDecision {
            bpm,
            source: GridBpmSource::SourceTiming,
            reason: GridBpmDecisionReason::SourceTimingNeedsReviewManualConfirm,
            source_primary_bpm: Some(bpm),
            source_delta_bpm: Some(0.0),
        }
    }

    fn groove_evidence_with_offset(offset_ms: f32) -> ManifestSourceTimingGrooveEvidence {
        ManifestSourceTimingGrooveEvidence {
            primary_groove_residual_count: 1,
            primary_max_abs_offset_ms: offset_ms.abs(),
            primary_groove_preview: vec![ManifestSourceTimingGrooveResidual {
                subdivision: "eighth",
                offset_ms,
                confidence: 0.82,
            }],
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
