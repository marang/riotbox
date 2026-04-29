#[cfg(test)]
mod tests {
    use super::*;
    use riotbox_audio::runtime::signal_metrics;

    #[test]
    fn parses_grid_controls() {
        let parsed = Args::parse([
            "--source".to_string(),
            "input.wav".to_string(),
            "--output-dir".to_string(),
            "out".to_string(),
            "--bpm".to_string(),
            "130.0".to_string(),
            "--bars".to_string(),
            "4".to_string(),
            "--source-window-seconds".to_string(),
            "0.5".to_string(),
        ])
        .expect("parse args");

        assert_eq!(parsed.source_path, PathBuf::from("input.wav"));
        assert_eq!(parsed.output_dir, Some(PathBuf::from("out")));
        assert_eq!(parsed.bpm, 130.0);
        assert_eq!(parsed.bars, 4);
        assert_eq!(parsed.source_window_seconds, 0.5);
    }

    #[test]
    fn rejects_missing_source() {
        assert!(Args::parse(Vec::<String>::new()).is_err());
    }

    #[test]
    fn rejects_single_bar_pack() {
        assert!(
            Args::parse([
                "--source".to_string(),
                "input.wav".to_string(),
                "--bars".to_string(),
                "1".to_string(),
            ])
            .is_err()
        );
    }

    #[test]
    fn grid_uses_cumulative_frame_rounding() {
        let grid = Grid::new(128.0, 4, 8).expect("grid");

        assert_eq!(grid.total_beats, 32);
        assert_eq!(grid.total_frames, frames_for_beats(128.0, 32));
        assert_eq!(grid.bar_frame_count(0), grid.bar_end_frame(0));
        assert_eq!(grid.bar_end_frame(7), grid.total_frames);
    }

    #[test]
    fn bar_variation_metrics_distinguish_identical_and_alternating_bars() {
        let grid = Grid::new(120.0, 4, 3).expect("grid");
        let identical = bar_pattern_samples(&grid, |_, frame, bar_frames| frame < bar_frames / 8);
        let alternating = bar_pattern_samples(&grid, |bar, frame, bar_frames| {
            if bar.is_multiple_of(2) {
                frame < bar_frames / 8
            } else {
                frame > bar_frames / 2 && frame < bar_frames / 2 + bar_frames / 8
            }
        });

        let identical_metrics = bar_variation_metrics(&identical, &grid);
        let alternating_metrics = bar_variation_metrics(&alternating, &grid);

        assert!(identical_metrics.bar_similarity > 0.999);
        assert_eq!(identical_metrics.identical_bar_run_length, 3);
        assert!(alternating_metrics.bar_similarity < 0.25);
        assert_eq!(alternating_metrics.identical_bar_run_length, 1);
    }

    #[test]
    fn spectral_energy_metrics_distinguish_low_and_high_content() {
        let low = tone_samples(80.0, SAMPLE_RATE as usize / 2);
        let high = tone_samples(4_200.0, SAMPLE_RATE as usize / 2);

        let low_metrics = spectral_energy_metrics(&low);
        let high_metrics = spectral_energy_metrics(&high);

        assert!(low_metrics.low_band_energy_ratio > high_metrics.low_band_energy_ratio);
        assert!(high_metrics.high_band_energy_ratio > low_metrics.high_band_energy_ratio);
    }

    #[test]
    fn renders_grid_pack_files_and_noncollapsed_audio() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source_path = temp.path().join("source.wav");
        let output_dir = temp.path().join("pack");
        write_interleaved_pcm16_wav(
            &source_path,
            SAMPLE_RATE,
            CHANNEL_COUNT,
            &synthetic_break_source(frames_for_beats(128.0, 8)),
        )
        .expect("write source");

        let args = Args {
            source_path,
            output_dir: Some(output_dir.clone()),
            date: "test".into(),
            bpm: 128.0,
            bars: 2,
            source_start_seconds: 0.0,
            source_window_seconds: 0.5,
            show_help: false,
        };

        render_pack(&args).expect("render pack");

        assert!(
            output_dir
                .join("stems/01_mc202_question_answer.wav")
                .is_file()
        );
        assert!(output_dir.join("stems/02_tr909_beat_fill.wav").is_file());
        assert!(
            output_dir
                .join("stems/03_w30_feral_source_chop.wav")
                .is_file()
        );
        assert!(output_dir.join("04_riotbox_grid_feral_mix.wav").is_file());
        assert!(output_dir.join("grid-report.md").is_file());
        assert!(output_dir.join("manifest.json").is_file());

        let mc202 =
            SourceAudioCache::load_pcm_wav(output_dir.join("stems/01_mc202_question_answer.wav"))
                .expect("load mc202");
        let full_mix =
            SourceAudioCache::load_pcm_wav(output_dir.join("04_riotbox_grid_feral_mix.wav"))
                .expect("load full mix");
        let grid = Grid::new(128.0, 4, 2).expect("grid");

        assert_eq!(mc202.frame_count(), grid.total_frames);
        assert_eq!(full_mix.frame_count(), grid.total_frames);
        assert!(signal_metrics(full_mix.interleaved_samples()).rms > MIN_SIGNAL_RMS);
        assert!(
            mc202_first_question_answer_delta(mc202.interleaved_samples(), &grid).rms
                > MIN_MC202_BAR_DELTA_RMS
        );
        assert!(
            signal_metrics(&one_pole_lowpass(full_mix.interleaved_samples(), 165.0)).rms
                > MIN_LOW_BAND_RMS
        );

        let manifest = fs::read_to_string(output_dir.join("manifest.json")).expect("manifest");
        let manifest: serde_json::Value = serde_json::from_str(&manifest).expect("parse manifest");
        assert_manifest_smoke_gate(&manifest, &output_dir);
    }

    fn assert_manifest_smoke_gate(manifest: &serde_json::Value, output_dir: &Path) {
        assert_eq!(
            manifest["schema_version"],
            LISTENING_MANIFEST_SCHEMA_VERSION
        );
        assert_eq!(manifest["pack_id"], PACK_ID);
        assert_eq!(manifest["result"], "pass");
        assert_eq!(manifest["bars"], 2);
        assert_eq!(manifest["feral_scorecard"]["readiness"], "ready");
        assert_eq!(
            manifest["feral_scorecard"]["break_rebuild_potential"],
            "high"
        );
        assert_eq!(manifest["feral_scorecard"]["source_backed"], true);
        assert_eq!(manifest["feral_scorecard"]["fallback_like"], false);
        assert_eq!(
            manifest["feral_scorecard"]["top_reason"],
            "grid-locked generated feral QA pack"
        );
        assert_eq!(
            manifest["feral_scorecard"]["lane_gestures"]
                .as_array()
                .expect("lane gestures")
                .len(),
            3
        );
        assert_manifest_f32(
            &manifest["thresholds"]["min_signal_rms"],
            MIN_SIGNAL_RMS,
            "min_signal_rms",
        );
        assert_manifest_f32(
            &manifest["thresholds"]["min_mc202_bar_delta_rms"],
            MIN_MC202_BAR_DELTA_RMS,
            "min_mc202_bar_delta_rms",
        );
        assert_manifest_f32(
            &manifest["thresholds"]["min_low_band_rms"],
            MIN_LOW_BAND_RMS,
            "min_low_band_rms",
        );

        let artifacts = manifest["artifacts"].as_array().expect("artifacts");
        assert_eq!(artifacts.len(), 6);
        assert_manifest_artifact(
            artifacts,
            "mc202_question_answer",
            "audio_wav",
            output_dir.join("stems/01_mc202_question_answer.wav"),
            Some(output_dir.join("stems/01_mc202_question_answer.metrics.md")),
        );
        assert_manifest_artifact(
            artifacts,
            "tr909_beat_fill",
            "audio_wav",
            output_dir.join("stems/02_tr909_beat_fill.wav"),
            Some(output_dir.join("stems/02_tr909_beat_fill.metrics.md")),
        );
        assert_manifest_artifact(
            artifacts,
            "w30_feral_source_chop",
            "audio_wav",
            output_dir.join("stems/03_w30_feral_source_chop.wav"),
            Some(output_dir.join("stems/03_w30_feral_source_chop.metrics.md")),
        );
        assert_manifest_artifact(
            artifacts,
            "full_grid_mix",
            "audio_wav",
            output_dir.join("04_riotbox_grid_feral_mix.wav"),
            Some(output_dir.join("04_riotbox_grid_feral_mix.metrics.md")),
        );
        assert_manifest_artifact(
            artifacts,
            "grid_report",
            "markdown_report",
            output_dir.join("grid-report.md"),
            None,
        );
        assert_manifest_artifact(
            artifacts,
            "readme",
            "markdown_readme",
            output_dir.join("README.md"),
            None,
        );

        assert!(
            manifest["metrics"]["full_grid_mix"]["signal"]["rms"]
                .as_f64()
                .expect("full mix rms")
                > f64::from(MIN_SIGNAL_RMS)
        );
        assert!(
            manifest["metrics"]["full_grid_mix"]["signal"]["event_density_per_bar"]
                .as_f64()
                .expect("full mix event density")
                > 0.0
        );
        assert!(
            manifest["metrics"]["full_grid_mix"]["low_band"]["rms"]
                .as_f64()
                .expect("low-band rms")
                > f64::from(MIN_LOW_BAND_RMS)
        );
        assert!(
            manifest["metrics"]["mc202_question_answer_delta"]["rms"]
                .as_f64()
                .expect("delta rms")
                > f64::from(MIN_MC202_BAR_DELTA_RMS)
        );
        assert!(
            manifest["metrics"]["bar_variation"]["full_grid_mix"]["bar_similarity"]
                .as_f64()
                .expect("bar similarity")
                <= 1.0
        );
        assert!(
            manifest["metrics"]["bar_variation"]["full_grid_mix"]["identical_bar_run_length"]
                .as_u64()
                .expect("identical bar run")
                >= 1
        );
        let spectral = &manifest["metrics"]["spectral_energy"]["full_grid_mix"];
        let spectral_sum = spectral["low_band_energy_ratio"]
            .as_f64()
            .expect("low energy")
            + spectral["mid_band_energy_ratio"].as_f64().expect("mid energy")
            + spectral["high_band_energy_ratio"]
                .as_f64()
                .expect("high energy");
        assert!((spectral_sum - 1.0).abs() < 0.000_001);
    }

    fn assert_manifest_artifact(
        artifacts: &[serde_json::Value],
        role: &str,
        kind: &str,
        path: PathBuf,
        metrics_path: Option<PathBuf>,
    ) {
        let artifact = artifacts
            .iter()
            .find(|artifact| artifact["role"] == role)
            .unwrap_or_else(|| panic!("missing artifact role {role}"));

        assert_eq!(artifact["kind"], kind);
        assert_eq!(artifact["path"], path.display().to_string());
        assert!(path.is_file(), "manifest artifact should exist: {path:?}");

        match metrics_path {
            Some(metrics_path) => {
                assert_eq!(artifact["metrics_path"], metrics_path.display().to_string());
                assert!(
                    metrics_path.is_file(),
                    "manifest metrics artifact should exist: {metrics_path:?}"
                );
            }
            None => assert!(artifact["metrics_path"].is_null()),
        }
    }

    fn assert_manifest_f32(value: &serde_json::Value, expected: f32, name: &str) {
        let actual = value.as_f64().unwrap_or_else(|| panic!("{name} missing"));
        assert!(
            (actual - f64::from(expected)).abs() < 0.000_001,
            "{name} expected {expected}, got {actual}"
        );
    }

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
            let sample = kick + grit;
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
}
