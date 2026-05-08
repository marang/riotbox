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
        assert!(parsed.bpm_overridden);
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
    fn mix_balance_gate_rejects_old_drum_dominant_policy() {
        let grid = Grid::new(128.0, 4, 2).expect("grid");
        let sample_count = grid.total_frames * usize::from(CHANNEL_COUNT);
        let tr909 = vec![0.20; sample_count];
        let w30 = vec![0.20; sample_count];
        let old_drum_dominant_policy = MixPolicy {
            tr909_gain: 10.0,
            tr909_low_gain: 18.0,
            w30_gain: 0.94,
            drive: 1.7,
            output_gain: 0.92,
        };

        let old_ratio =
            generated_to_source_rms_ratio(&tr909, &w30, &grid, old_drum_dominant_policy);
        let source_first_ratio = source_first_generated_to_source_rms_ratio(&tr909, &w30, &grid);
        let support_ratio = support_generated_to_source_rms_ratio(&tr909, &w30, &grid);

        assert!(old_ratio >= MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO);
        assert!(source_first_ratio < MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO);
        assert!(support_ratio < MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO);
    }

    #[test]
    fn source_aware_tr909_profile_changes_for_same_bpm_sources() {
        let grid = Grid::new(128.0, 4, 2).expect("grid");
        let low_source = tone_samples(80.0, frames_for_beats(128.0, 8));
        let high_source = tone_samples(4_200.0, frames_for_beats(128.0, 8));

        let low_profile = derive_source_aware_tr909_profile(&low_source, &grid);
        let high_profile = derive_source_aware_tr909_profile(&high_source, &grid);
        let low_render = render_tr909_source_support(&grid, low_profile);
        let high_render = render_tr909_source_support(&grid, high_profile);
        let low_render_repeat = render_tr909_source_support(&grid, low_profile);

        assert_eq!(low_profile.support_profile, Tr909SourceSupportProfile::DropDrive);
        assert_eq!(high_profile.support_profile, Tr909SourceSupportProfile::BreakLift);
        assert_ne!(low_profile.pattern_adoption, high_profile.pattern_adoption);
        assert_ne!(low_render, high_render);
        assert_eq!(low_render, low_render_repeat);
        assert!(signal_metrics(&low_render).rms > MIN_SIGNAL_RMS);
        assert!(signal_metrics(&high_render).rms > MIN_SIGNAL_RMS);
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
            bpm_overridden: true,
            bars: 2,
            source_start_seconds: 0.0,
            source_window_seconds: 0.5,
            show_help: false,
        };

        render_pack(&args).expect("render pack");

        assert!(output_dir.join("stems/01_tr909_beat_fill.wav").is_file());
        assert!(
            output_dir
                .join("stems/02_w30_feral_source_chop.wav")
                .is_file()
        );
        assert!(output_dir.join("03_riotbox_source_first_mix.wav").is_file());
        assert!(
            output_dir
                .join("04_riotbox_generated_support_mix.wav")
                .is_file()
        );
        assert!(output_dir.join("grid-report.md").is_file());
        assert!(output_dir.join("manifest.json").is_file());

        let source_first_mix =
            SourceAudioCache::load_pcm_wav(output_dir.join("03_riotbox_source_first_mix.wav"))
                .expect("load full mix");
        let full_mix =
            SourceAudioCache::load_pcm_wav(output_dir.join("04_riotbox_generated_support_mix.wav"))
                .expect("load generated-support mix");
        let grid = Grid::new(128.0, 4, 2).expect("grid");

        assert_eq!(source_first_mix.frame_count(), grid.total_frames);
        assert_eq!(full_mix.frame_count(), grid.total_frames);
        assert!(signal_metrics(source_first_mix.interleaved_samples()).rms > MIN_SIGNAL_RMS);
        assert!(signal_metrics(full_mix.interleaved_samples()).rms > MIN_SIGNAL_RMS);
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
        assert_eq!(manifest["grid_bpm_source"], "user_override");
        assert!(manifest["source_timing_bpm_delta"].is_number());
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
        let source_timing = &manifest["source_timing"];
        assert_eq!(source_timing["schema"], "riotbox.source_timing_probe_readiness.v1");
        assert_eq!(source_timing["schema_version"], 1);
        assert!(source_timing["source_id"]
            .as_str()
            .is_some_and(|value| value.ends_with("source.wav")));
        assert_eq!(
            source_timing["policy_profile"],
            SourceTimingProbeBpmCandidatePolicy::DANCE_LOOP_AUTO_READINESS_PROFILE
        );
        let readiness = source_timing["readiness"].as_str();
        assert!(readiness.is_some_and(|value| matches!(
            value,
            "unavailable" | "weak" | "needs_review" | "ready"
        )));
        assert!(source_timing["requires_manual_confirm"].is_boolean());
        assert!(source_timing["bpm_agrees_with_grid"].is_boolean());
        assert!(source_timing["warning_codes"].is_array());
        assert_eq!(
            manifest["feral_scorecard"]["lane_gestures"]
                .as_array()
                .expect("lane gestures")
                .len(),
            2
        );
        assert_manifest_f32(
            &manifest["thresholds"]["min_signal_rms"],
            MIN_SIGNAL_RMS,
            "min_signal_rms",
        );
        assert_manifest_f32(
            &manifest["thresholds"]["min_low_band_rms"],
            MIN_LOW_BAND_RMS,
            "min_low_band_rms",
        );
        assert_manifest_f32(
            &manifest["thresholds"]["max_source_first_generated_to_source_rms_ratio"],
            MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO,
            "max_source_first_generated_to_source_rms_ratio",
        );
        assert_manifest_f32(
            &manifest["thresholds"]["max_support_generated_to_source_rms_ratio"],
            MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
            "max_support_generated_to_source_rms_ratio",
        );

        let artifacts = manifest["artifacts"].as_array().expect("artifacts");
        assert_eq!(artifacts.len(), 6);
        assert_manifest_artifact(
            artifacts,
            "tr909_beat_fill",
            "audio_wav",
            output_dir.join("stems/01_tr909_beat_fill.wav"),
            Some(output_dir.join("stems/01_tr909_beat_fill.metrics.md")),
        );
        assert_manifest_artifact(
            artifacts,
            "w30_feral_source_chop",
            "audio_wav",
            output_dir.join("stems/02_w30_feral_source_chop.wav"),
            Some(output_dir.join("stems/02_w30_feral_source_chop.metrics.md")),
        );
        assert_manifest_artifact(
            artifacts,
            "source_first_mix",
            "audio_wav",
            output_dir.join("03_riotbox_source_first_mix.wav"),
            Some(output_dir.join("03_riotbox_source_first_mix.metrics.md")),
        );
        assert_manifest_artifact(
            artifacts,
            "full_grid_mix",
            "audio_wav",
            output_dir.join("04_riotbox_generated_support_mix.wav"),
            Some(output_dir.join("04_riotbox_generated_support_mix.metrics.md")),
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
            manifest["metrics"]["source_first_mix"]["signal"]["rms"]
                .as_f64()
                .expect("source-first mix rms")
                > f64::from(MIN_SIGNAL_RMS)
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
        assert!(manifest["metrics"]["mc202_question_answer_delta"].is_null());
        assert!(manifest["metrics"]["mc202_question_answer"].is_null());
        assert_eq!(
            manifest["metrics"]["tr909_source_profile"]["support_context"],
            "transport_bar"
        );
        assert!(
            manifest["metrics"]["tr909_source_profile"]["reason"]
                .as_str()
                .expect("tr909 source reason")
                .starts_with("source_")
        );
        assert!(
            manifest["metrics"]["tr909_source_profile"]["signal_rms"]
                .as_f64()
                .expect("tr909 source signal rms")
                > 0.0
        );
        assert!(
            manifest["metrics"]["w30_source_chop_profile"]["preview_rms"]
                .as_f64()
                .expect("w30 source chop preview rms")
                > 0.0
        );
        assert!(
            manifest["metrics"]["w30_source_chop_profile"]["gain"]
                .as_f64()
                .expect("w30 source chop gain")
                >= 0.85
        );
        assert!(
            manifest["metrics"]["mix_balance"]["source_first_generated_to_source_rms_ratio"]
                .as_f64()
                .expect("source-first generated/source ratio")
                < f64::from(MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO)
        );
        assert!(
            manifest["metrics"]["mix_balance"]["support_generated_to_source_rms_ratio"]
                .as_f64()
                .expect("support generated/source ratio")
                < f64::from(MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO)
        );
        let output_drift = &manifest["metrics"]["source_grid_output_drift"];
        let hit_ratio = output_drift["hit_ratio"].as_f64().expect("hit ratio");
        assert!(hit_ratio >= f64::from(SOURCE_GRID_OUTPUT_MIN_HIT_RATIO));
        assert!(
            manifest["metrics"]["bar_variation"]["source_first_mix"]["bar_similarity"]
                .as_f64()
                .expect("source-first bar similarity")
                <= 1.0
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
        assert_spectral_sum(&manifest["metrics"]["spectral_energy"]["source_first_mix"]);
        assert_spectral_sum(&manifest["metrics"]["spectral_energy"]["full_grid_mix"]);
    }

    fn assert_spectral_sum(spectral: &serde_json::Value) {
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
