#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_args() {
        assert_eq!(
            Args::parse(Vec::<String>::new()).expect("parse args"),
            Args {
                output_path: PathBuf::from(
                    "artifacts/audio_qa/local/w30-preview-smoke/raw_capture_source_window_preview/candidate.wav",
                ),
                duration_seconds: DEFAULT_DURATION_SECONDS,
                date: DEFAULT_DATE.to_string(),
                role: RenderRole::Candidate,
                source_path: None,
                source_start_seconds: DEFAULT_SOURCE_START_SECONDS,
                source_duration_seconds: DEFAULT_SOURCE_DURATION_SECONDS,
                show_help: false,
            }
        );
    }

    #[test]
    fn parses_custom_output_and_duration() {
        assert_eq!(
            Args::parse([
                "--out".to_string(),
                "tmp/render.wav".to_string(),
                "--date".to_string(),
                "2026-04-26".to_string(),
                "--role".to_string(),
                "baseline".to_string(),
                "--source".to_string(),
                "data/test_audio/examples/DH_BeatC_120-01.wav".to_string(),
                "--source-start-seconds".to_string(),
                "0.25".to_string(),
                "--source-duration-seconds".to_string(),
                "0.75".to_string(),
                "--duration-seconds".to_string(),
                "0.5".to_string(),
            ])
            .expect("parse args"),
            Args {
                output_path: PathBuf::from("tmp/render.wav"),
                duration_seconds: 0.5,
                date: "2026-04-26".to_string(),
                role: RenderRole::Baseline,
                source_path: Some(PathBuf::from(
                    "data/test_audio/examples/DH_BeatC_120-01.wav"
                )),
                source_start_seconds: 0.25,
                source_duration_seconds: 0.75,
                show_help: false,
            }
        );
    }

    #[test]
    fn derives_convention_path_from_date_and_role() {
        assert_eq!(
            Args::parse([
                "--date".to_string(),
                "2026-04-26".to_string(),
                "--role".to_string(),
                "baseline".to_string(),
            ])
            .expect("parse args")
            .output_path,
            PathBuf::from(
                "artifacts/audio_qa/2026-04-26/w30-preview-smoke/raw_capture_source_window_preview/baseline.wav",
            )
        );
    }

    #[test]
    fn rejects_unknown_roles() {
        assert!(Args::parse(["--role".to_string(), "review".to_string()]).is_err());
    }

    #[test]
    fn rejects_invalid_source_window_seconds() {
        assert!(Args::parse(["--source-start-seconds".to_string(), "-0.1".to_string()]).is_err());
        assert!(Args::parse(["--source-duration-seconds".to_string(), "0".to_string()]).is_err());
    }

    #[test]
    fn averages_interleaved_source_frames_into_preview() {
        let preview =
            source_preview_from_interleaved(&[1.0, 3.0, 5.0, 7.0], 2, 10, 12).expect("preview");

        assert_eq!(preview.source_start_frame, 10);
        assert_eq!(preview.source_end_frame, 12);
        assert_eq!(preview.sample_count, 2);
        assert_eq!(preview.samples[0], 2.0);
        assert_eq!(preview.samples[1], 6.0);
    }

    #[test]
    fn rejects_unknown_args() {
        assert!(Args::parse(["--unknown".to_string()]).is_err());
    }

    #[test]
    fn derives_sibling_metrics_path() {
        assert_eq!(
            metrics_path_for(Path::new("out/candidate.wav")),
            PathBuf::from("out/candidate.metrics.md")
        );
    }
}
