#[cfg(test)]
mod tests {
    use super::*;

    const METRICS_MARKDOWN: &str = "\
# W-30 Preview Smoke Metrics

- Pack: `w30-preview-smoke`
- Case: `raw_capture_source_window_preview`
- Role: `candidate`
- Output: `candidate.wav`
- Sample rate: `44100`
- Channels: `2`
- Duration seconds: `0.100`
- Samples: `8820`
- Active samples: `512`
- Peak abs: `0.115204`
- RMS: `0.038331`
- Sum: `4.750000`
- Mean abs: `0.021000`
- Zero crossings: `128`
- Crest factor: `3.005000`
- Active sample ratio: `0.058050`
- Silence ratio: `0.941950`
- DC offset: `0.000539`
- Onset count: `8`
- Event density per bar: `16.000000`
";

    const fn sample_metrics() -> SmokeMetrics {
        SmokeMetrics {
            active_samples: 512,
            peak_abs: 0.115204,
            rms: 0.038331,
            sum: 4.75,
            mean_abs: 0.021,
            zero_crossings: 128,
            crest_factor: 3.005,
            active_sample_ratio: 0.05805,
            silence_ratio: 0.94195,
            dc_offset: 0.000539,
            onset_count: 8,
            event_density_per_bar: 16.0,
        }
    }

    #[test]
    fn parses_default_args() {
        assert_eq!(
            Args::parse(Vec::<String>::new()).expect("parse args"),
            Args {
                baseline_metrics_path: PathBuf::from(
                    "artifacts/audio_qa/local/w30-preview-smoke/raw_capture_source_window_preview/baseline.metrics.md",
                ),
                candidate_metrics_path: PathBuf::from(
                    "artifacts/audio_qa/local/w30-preview-smoke/raw_capture_source_window_preview/candidate.metrics.md",
                ),
                report_path: PathBuf::from(
                    "artifacts/audio_qa/local/w30-preview-smoke/raw_capture_source_window_preview/comparison.md",
                ),
                limits: DriftLimits::default(),
                show_help: false,
            }
        );
    }

    #[test]
    fn parses_convention_date_and_overrides() {
        let args = Args::parse([
            "--date".to_string(),
            "2026-04-26".to_string(),
            "--baseline".to_string(),
            "tmp/base.md".to_string(),
            "--candidate".to_string(),
            "tmp/candidate.md".to_string(),
            "--report".to_string(),
            "tmp/comparison.md".to_string(),
            "--max-active-samples-delta".to_string(),
            "2".to_string(),
            "--min-active-samples-delta".to_string(),
            "1".to_string(),
            "--max-peak-delta".to_string(),
            "0.01".to_string(),
            "--min-peak-delta".to_string(),
            "0.001".to_string(),
            "--max-rms-delta".to_string(),
            "0.02".to_string(),
            "--min-rms-delta".to_string(),
            "0.002".to_string(),
            "--max-sum-delta".to_string(),
            "0.03".to_string(),
            "--min-sum-delta".to_string(),
            "0.003".to_string(),
        ])
        .expect("parse args");

        assert_eq!(args.baseline_metrics_path, PathBuf::from("tmp/base.md"));
        assert_eq!(
            args.candidate_metrics_path,
            PathBuf::from("tmp/candidate.md")
        );
        assert_eq!(args.report_path, PathBuf::from("tmp/comparison.md"));
        assert_eq!(args.limits.min_active_samples_delta, 1);
        assert_eq!(args.limits.max_active_samples_delta, 2);
        assert_eq!(args.limits.min_peak_delta, 0.001);
        assert_eq!(args.limits.max_peak_delta, 0.01);
        assert_eq!(args.limits.min_rms_delta, 0.002);
        assert_eq!(args.limits.max_rms_delta, 0.02);
        assert_eq!(args.limits.min_sum_delta, 0.003);
        assert_eq!(args.limits.max_sum_delta, 0.03);
    }

    #[test]
    fn derives_paths_from_date() {
        assert_eq!(
            Args::parse(["--date".to_string(), "2026-04-26".to_string()])
                .expect("parse args")
                .candidate_metrics_path,
            PathBuf::from(
                "artifacts/audio_qa/2026-04-26/w30-preview-smoke/raw_capture_source_window_preview/candidate.metrics.md",
            )
        );
    }

    #[test]
    fn derives_report_path_from_date() {
        assert_eq!(
            Args::parse(["--date".to_string(), "2026-04-26".to_string()])
                .expect("parse args")
                .report_path,
            PathBuf::from(
                "artifacts/audio_qa/2026-04-26/w30-preview-smoke/raw_capture_source_window_preview/comparison.md",
            )
        );
    }

    #[test]
    fn derives_audio_path_from_metrics_path() {
        assert_eq!(
            audio_path_for_metrics_path(Path::new("out/baseline.metrics.md")),
            PathBuf::from("out/baseline.wav")
        );
    }

    #[test]
    fn rejects_negative_float_limits() {
        assert!(Args::parse(["--max-rms-delta".to_string(), "-0.1".to_string()]).is_err());
    }

    #[test]
    fn rejects_non_finite_float_limits() {
        assert!(Args::parse(["--max-rms-delta".to_string(), "NaN".to_string()]).is_err());
        assert!(Args::parse(["--max-rms-delta".to_string(), "inf".to_string()]).is_err());
    }

    #[test]
    fn parses_metrics_markdown() {
        assert_eq!(
            SmokeMetrics::parse_markdown(METRICS_MARKDOWN).expect("parse metrics"),
            SmokeMetrics {
                active_samples: 512,
                peak_abs: 0.115204,
                rms: 0.038331,
                sum: 4.75,
                mean_abs: 0.021,
                zero_crossings: 128,
                crest_factor: 3.005,
                active_sample_ratio: 0.05805,
                silence_ratio: 0.94195,
                dc_offset: 0.000539,
                onset_count: 8,
                event_density_per_bar: 16.0,
            }
        );
    }

    #[test]
    fn rejects_missing_metrics() {
        assert!(SmokeMetrics::parse_markdown("- RMS: `0.1`").is_err());
    }

    #[test]
    fn rejects_non_finite_metrics() {
        assert!(
            SmokeMetrics::parse_markdown(&METRICS_MARKDOWN.replace("0.038331", "NaN")).is_err()
        );
    }

    #[test]
    fn comparison_passes_within_limits() {
        let baseline = SmokeMetrics {
            active_samples: 512,
            peak_abs: 0.115204,
            rms: 0.038331,
            sum: 4.75,
            ..sample_metrics()
        };
        let candidate = SmokeMetrics {
            active_samples: 513,
            peak_abs: 0.115205,
            rms: 0.038330,
            sum: 4.750001,
            mean_abs: 0.021001,
            zero_crossings: 130,
            crest_factor: 3.004,
            active_sample_ratio: 0.058163,
            silence_ratio: 0.941837,
            dc_offset: 0.000538,
            onset_count: 9,
            event_density_per_bar: 18.0,
        };
        let limits = DriftLimits {
            min_active_samples_delta: 0,
            max_active_samples_delta: 1,
            min_peak_delta: 0.0,
            max_peak_delta: 0.000001,
            min_rms_delta: 0.0,
            max_rms_delta: 0.000001,
            min_sum_delta: 0.0,
            max_sum_delta: 0.000001,
        };

        assert!(!compare_metrics(&baseline, &candidate, &limits).has_failures());
    }

    #[test]
    fn comparison_can_require_candidate_to_differ_from_baseline() {
        let baseline = SmokeMetrics {
            active_samples: 512,
            peak_abs: 0.115204,
            rms: 0.038331,
            sum: 4.75,
            ..sample_metrics()
        };
        let candidate = SmokeMetrics {
            active_samples: 512,
            peak_abs: 0.125204,
            rms: 0.041331,
            sum: 5.15,
            mean_abs: 0.024,
            zero_crossings: 150,
            crest_factor: 3.21,
            active_sample_ratio: 0.061,
            silence_ratio: 0.939,
            dc_offset: 0.001,
            onset_count: 10,
            event_density_per_bar: 20.0,
        };
        let limits = DriftLimits {
            min_active_samples_delta: 0,
            max_active_samples_delta: 0,
            min_peak_delta: 0.005,
            max_peak_delta: 0.02,
            min_rms_delta: 0.002,
            max_rms_delta: 0.01,
            min_sum_delta: 0.2,
            max_sum_delta: 1.0,
        };

        assert!(!compare_metrics(&baseline, &candidate, &limits).has_failures());

        let too_similar = SmokeMetrics {
            peak_abs: 0.115304,
            rms: 0.038431,
            sum: 4.751,
            ..candidate
        };

        assert!(compare_metrics(&baseline, &too_similar, &limits).has_failures());
    }

    #[test]
    fn comparison_fails_outside_limits() {
        let baseline = SmokeMetrics {
            active_samples: 512,
            peak_abs: 0.115204,
            rms: 0.038331,
            sum: 4.75,
            ..sample_metrics()
        };
        let candidate = SmokeMetrics {
            active_samples: 514,
            peak_abs: 0.2,
            rms: 0.1,
            sum: 6.0,
            mean_abs: 0.07,
            zero_crossings: 160,
            crest_factor: 2.0,
            active_sample_ratio: 0.07,
            silence_ratio: 0.93,
            dc_offset: 0.01,
            onset_count: 12,
            event_density_per_bar: 24.0,
        };

        assert!(compare_metrics(&baseline, &candidate, &DriftLimits::default()).has_failures());
    }

    #[test]
    fn writes_manifest_for_existing_smoke_artifacts() {
        let temp = tempfile::tempdir().expect("tempdir");
        let case_dir = temp.path().join(CASE_ID);
        fs::create_dir_all(&case_dir).expect("case dir");
        let baseline_metrics_path = case_dir.join("baseline.metrics.md");
        let candidate_metrics_path = case_dir.join("candidate.metrics.md");
        let report_path = case_dir.join("comparison.md");
        fs::write(case_dir.join("baseline.wav"), b"baseline").expect("baseline wav");
        fs::write(case_dir.join("candidate.wav"), b"candidate").expect("candidate wav");
        fs::write(&baseline_metrics_path, METRICS_MARKDOWN).expect("baseline metrics");
        fs::write(
            &candidate_metrics_path,
            METRICS_MARKDOWN.replace("candidate", "baseline"),
        )
        .expect("candidate metrics");

        let args = Args {
            baseline_metrics_path,
            candidate_metrics_path,
            report_path,
            limits: DriftLimits::default(),
            show_help: false,
        };
        let baseline = SmokeMetrics::read_from_path(&args.baseline_metrics_path).expect("baseline");
        let candidate =
            SmokeMetrics::read_from_path(&args.candidate_metrics_path).expect("candidate");
        let report = compare_metrics(&baseline, &candidate, &args.limits);
        write_report_markdown(
            &args.report_path,
            &render_report(
                &args.baseline_metrics_path,
                &args.candidate_metrics_path,
                &report,
            ),
        )
        .expect("report");

        write_manifest(&args, baseline, candidate, &report).expect("manifest");

        let manifest =
            fs::read_to_string(manifest_path_for_report_path(&args.report_path)).expect("manifest");
        let manifest: serde_json::Value = serde_json::from_str(&manifest).expect("parse manifest");
        assert_eq!(
            manifest["schema_version"],
            LISTENING_MANIFEST_SCHEMA_VERSION
        );
        assert_eq!(manifest["pack_id"], PACK_ID);
        assert_eq!(manifest["case_id"], CASE_ID);
        assert_eq!(manifest["result"], "pass");
        assert_eq!(
            manifest["artifacts"].as_array().expect("artifacts").len(),
            3
        );
        assert_eq!(manifest["metrics"]["baseline"]["rms"], 0.038331);
        assert_eq!(manifest["metrics"]["baseline"]["zero_crossings"], 128);
        assert_eq!(manifest["metrics"]["baseline"]["active_sample_ratio"], 0.05805);
        assert_eq!(manifest["metrics"]["baseline"]["onset_count"], 8);
        assert_eq!(manifest["metrics"]["deltas"]["rms"], 0.0);
        assert_eq!(manifest["metrics"]["deltas"]["zero_crossings"], 0);
        assert_eq!(manifest["metrics"]["deltas"]["dc_offset"], 0.0);
        assert_eq!(manifest["metrics"]["deltas"]["event_density_per_bar"], 0.0);
    }

    #[test]
    fn manifest_rejects_missing_audio_artifacts() {
        let temp = tempfile::tempdir().expect("tempdir");
        let case_dir = temp.path().join(CASE_ID);
        fs::create_dir_all(&case_dir).expect("case dir");
        let baseline_metrics_path = case_dir.join("baseline.metrics.md");
        let candidate_metrics_path = case_dir.join("candidate.metrics.md");
        let report_path = case_dir.join("comparison.md");
        fs::write(&baseline_metrics_path, METRICS_MARKDOWN).expect("baseline metrics");
        fs::write(&candidate_metrics_path, METRICS_MARKDOWN).expect("candidate metrics");
        fs::write(&report_path, "comparison").expect("report");

        let args = Args {
            baseline_metrics_path,
            candidate_metrics_path,
            report_path,
            limits: DriftLimits::default(),
            show_help: false,
        };
        let metrics = SmokeMetrics::read_from_path(&args.baseline_metrics_path).expect("metrics");
        let report = compare_metrics(&metrics, &metrics, &args.limits);

        assert!(write_manifest(&args, metrics, metrics, &report).is_err());
    }
}
