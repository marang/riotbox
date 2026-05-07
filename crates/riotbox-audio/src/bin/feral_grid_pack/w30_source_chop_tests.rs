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
