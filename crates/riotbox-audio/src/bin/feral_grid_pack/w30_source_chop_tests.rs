#[cfg(test)]
mod w30_source_chop_tests {
    use super::*;
    use riotbox_audio::runtime::signal_metrics;

    #[test]
    fn source_chop_preview_selects_and_normalizes_articulate_segment() {
        let samples = delayed_pulse_source(5_000, 2_600, 0.04, 0.42);
        let (preview, profile) =
            source_chop_preview_from_interleaved(&samples, usize::from(CHANNEL_COUNT), 10, 5_010)
                .expect("preview");

        assert!(profile.selected_start_frame > 10);
        assert!(profile.preview_rms > profile.source_window_rms);
        assert!(profile.preview_rms > 0.08);
        assert!(profile.preview_peak_abs <= 0.95);
        assert_eq!(preview.sample_count, W30_PREVIEW_SAMPLE_WINDOW_LEN);
    }

    #[test]
    fn source_chop_preview_shapes_tail_into_a_chop_instead_of_a_flat_loop() {
        let samples = tone_source(180.0, 5_000, 0.22);
        let (preview, profile) =
            source_chop_preview_from_interleaved(&samples, usize::from(CHANNEL_COUNT), 0, 5_000)
                .expect("preview");

        let preview = &preview.samples[..preview.sample_count];
        let body_rms = rms(&preview[256..512]);
        let tail_rms = rms(&preview[preview.len() - 384..preview.len() - 128]);

        assert!(profile.preview_rms > MIN_SIGNAL_RMS);
        assert!(
            tail_rms < body_rms * 0.70,
            "tail {tail_rms} should decay below body {body_rms}"
        );
        assert!(profile.tail_to_body_rms_ratio < 0.70, "{profile:?}");
    }

    #[test]
    fn w30_source_chop_render_differs_from_control_and_other_source() {
        let grid = Grid::new(128.0, 4, 2).expect("grid");
        let low_source = tone_source(90.0, frames_for_beats(128.0, 8), 0.12);
        let pulse_source = delayed_pulse_source(frames_for_beats(128.0, 8), 1_200, 0.02, 0.55);
        let low_preview = source_chop_preview_from_interleaved(
            &low_source,
            usize::from(CHANNEL_COUNT),
            0,
            frames_for_beats(128.0, 8) as u64,
        )
        .expect("low preview")
        .0;
        let pulse_preview = source_chop_preview_from_interleaved(
            &pulse_source,
            usize::from(CHANNEL_COUNT),
            0,
            frames_for_beats(128.0, 8) as u64,
        )
        .expect("pulse preview")
        .0;

        let low_render = render_w30_source_chop(&grid, low_preview);
        let pulse_render = render_w30_source_chop(&grid, pulse_preview);
        let control_render = render_w30_source_chop_control(&grid);
        let low_metrics = signal_metrics(&low_render);
        let pulse_metrics = signal_metrics(&pulse_render);
        let control_metrics = signal_metrics(&control_render);

        assert_ne!(low_render, pulse_render);
        assert_ne!(low_render, control_render);
        assert_ne!(pulse_render, control_render);
        assert!((low_metrics.rms - control_metrics.rms).abs() > 0.001);
        assert!((pulse_metrics.rms - control_metrics.rms).abs() > 0.001);
        assert!(low_metrics.rms > MIN_SIGNAL_RMS);
        assert!(pulse_metrics.rms > MIN_SIGNAL_RMS);
    }

    #[test]
    fn w30_source_chop_render_lands_on_selected_source_grid() {
        let grid = Grid::new(128.0, 4, 2).expect("grid");
        let pulse_source = delayed_pulse_source(frames_for_beats(128.0, 8), 0, 0.01, 0.65);
        let pulse_preview = source_chop_preview_from_interleaved(
            &pulse_source,
            usize::from(CHANNEL_COUNT),
            0,
            frames_for_beats(128.0, 8) as u64,
        )
        .expect("pulse preview")
        .0;

        let render = render_w30_source_chop(&grid, pulse_preview);
        let metrics = source_grid_output_drift_metrics(&render, &grid);

        assert_eq!(metrics.beat_count, grid.total_beats);
        assert!(
            metrics.hit_ratio >= SOURCE_GRID_OUTPUT_MIN_HIT_RATIO,
            "W-30 hit ratio {} should stay on the selected source grid",
            metrics.hit_ratio
        );
        assert!(
            metrics.max_peak_offset_ms <= SOURCE_GRID_OUTPUT_MAX_PEAK_OFFSET_MS,
            "W-30 max peak offset {} ms should stay inside source-grid tolerance",
            metrics.max_peak_offset_ms
        );
    }

    #[test]
    fn w30_source_chop_trigger_variation_adds_offbeats_without_grid_drift() {
        let grid = Grid::new(128.0, 4, 4).expect("grid");
        let pulse_source = delayed_pulse_source(frames_for_beats(128.0, 16), 0, 0.01, 0.65);
        let pulse_preview = source_chop_preview_from_interleaved(
            &pulse_source,
            usize::from(CHANNEL_COUNT),
            0,
            frames_for_beats(128.0, 16) as u64,
        )
        .expect("pulse preview")
        .0;

        let (varied_render, proof, slice_choice) =
            render_w30_source_chop_with_variation(&grid, &pulse_preview);
        let legacy_render = render_w30_source_chop_legacy(&grid, pulse_preview);
        let varied_alignment = source_grid_output_drift_metrics(&varied_render, &grid);
        let legacy_bars = bar_variation_metrics(&legacy_render, &grid);
        let varied_bars = bar_variation_metrics(&varied_render, &grid);

        assert!(proof.applied, "{proof:?}");
        assert_eq!(proof.grid_subdivision, W30_SOURCE_TRIGGER_GRID_SUBDIVISION);
        assert!(proof.offbeat_trigger_count > 0);
        assert!(proof.distinct_bar_pattern_count >= 2);
        assert!(proof.max_quantized_offset_ms <= proof.max_allowed_quantized_offset_ms);
        assert!(slice_choice.applied, "{slice_choice:?}");
        assert!(slice_choice.unique_source_offset_count >= 4);
        assert!(slice_choice.selected_offset_span_samples > 0);
        assert!(
            varied_alignment.hit_ratio >= SOURCE_GRID_OUTPUT_MIN_HIT_RATIO,
            "{varied_alignment:?}"
        );
        assert!(
            varied_bars.bar_similarity < legacy_bars.bar_similarity,
            "varied bars {varied_bars:?} should be less static than legacy {legacy_bars:?}"
        );
    }

    #[test]
    fn w30_source_loop_closure_proves_repeat_safe_faded_chop_window() {
        let pulse_source = delayed_pulse_source(frames_for_beats(128.0, 8), 1_200, 0.02, 0.55);
        let (preview, profile) = source_chop_preview_from_interleaved(
            &pulse_source,
            usize::from(CHANNEL_COUNT),
            0,
            frames_for_beats(128.0, 8) as u64,
        )
        .expect("pulse preview");

        let proof = w30_source_loop_closure_proof(&preview, profile);

        assert!(proof.passed, "{proof:?}");
        assert_eq!(proof.selected_frame_count, W30_PREVIEW_SAMPLE_WINDOW_LEN);
        assert!(proof.preview_rms > MIN_SIGNAL_RMS);
        assert!(proof.source_contains_selection);
        assert!(proof.edge_delta_abs <= proof.max_allowed_edge_delta_abs);
        assert!(proof.edge_abs_max <= proof.max_allowed_edge_abs);
    }

    #[test]
    fn w30_source_loop_closure_rejects_unfaded_loud_edges() {
        let mut preview = W30PreviewSampleWindow {
            source_start_frame: 0,
            source_end_frame: W30_PREVIEW_SAMPLE_WINDOW_LEN as u64,
            sample_count: W30_PREVIEW_SAMPLE_WINDOW_LEN,
            samples: [0.0; W30_PREVIEW_SAMPLE_WINDOW_LEN],
        };
        preview.samples[0] = 0.5;
        preview.samples[W30_PREVIEW_SAMPLE_WINDOW_LEN - 1] = -0.5;
        let profile = W30SourceChopProfile {
            source_window_rms: 0.2,
            selected_rms_before_gain: 0.2,
            preview_rms: 0.2,
            preview_peak_abs: 0.5,
            body_rms: 0.2,
            tail_rms: 0.2,
            tail_to_body_rms_ratio: 1.0,
            selected_start_frame: 0,
            selected_frame_count: W30_PREVIEW_SAMPLE_WINDOW_LEN,
            gain: 1.0,
            reason: "test_unfaded_edges",
        };

        let proof = w30_source_loop_closure_proof(&preview, profile);

        assert!(!proof.passed);
        assert!(proof.edge_delta_abs > proof.max_allowed_edge_delta_abs);
        assert!(proof.edge_abs_max > proof.max_allowed_edge_abs);
    }

    fn render_w30_source_chop_control(grid: &Grid) -> Vec<f32> {
        render_w30_preview_offline(
            &W30PreviewRenderState {
                mode: W30PreviewRenderMode::RawCaptureAudition,
                routing: W30PreviewRenderRouting::MusicBusPreview,
                source_profile: Some(W30PreviewSourceProfile::RawCaptureAudition),
                active_bank_id: Some("bank-a".into()),
                focused_pad_id: Some("pad-01".into()),
                capture_id: Some("cap-feral-grid-control".into()),
                trigger_revision: 1,
                trigger_velocity: 0.82,
                source_window_preview: None,
                pad_playback: None,
                music_bus_level: 0.72,
                grit_level: 0.46,
                is_transport_running: true,
                tempo_bpm: grid.bpm,
                position_beats: 0.0,
            },
            SAMPLE_RATE,
            CHANNEL_COUNT,
            grid.total_frames,
        )
    }

    fn delayed_pulse_source(
        frame_count: usize,
        pulse_start: usize,
        bed_level: f32,
        pulse_level: f32,
    ) -> Vec<f32> {
        let mut samples = Vec::with_capacity(frame_count * usize::from(CHANNEL_COUNT));
        for frame in 0..frame_count {
            let phase = frame as f32 / SAMPLE_RATE as f32;
            let bed = (phase * 180.0 * std::f32::consts::TAU).sin() * bed_level;
            let pulse_frame = frame.saturating_sub(pulse_start);
            let pulse = if frame >= pulse_start && pulse_frame < 500 {
                let decay = 1.0 - pulse_frame as f32 / 500.0;
                (phase * 1_600.0 * std::f32::consts::TAU).sin() * pulse_level * decay
            } else {
                0.0
            };
            let sample = bed + pulse;
            samples.push(sample);
            samples.push(sample * 0.98);
        }
        samples
    }

    fn tone_source(frequency_hz: f32, frame_count: usize, level: f32) -> Vec<f32> {
        let mut samples = Vec::with_capacity(frame_count * usize::from(CHANNEL_COUNT));
        for frame in 0..frame_count {
            let phase = frame as f32 / SAMPLE_RATE as f32;
            let sample = (phase * frequency_hz * std::f32::consts::TAU).sin() * level;
            samples.push(sample);
            samples.push(sample);
        }
        samples
    }
}
