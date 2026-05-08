#[cfg(test)]
mod bpm_decision_tests {
    use super::*;

    #[test]
    fn bpm_grid_decision_prefers_ready_source_timing_without_override() {
        let args = bpm_decision_args(DEFAULT_BPM, false);
        let ready = readiness_report(Some(130.0), SourceTimingProbeReadinessStatus::Ready);
        let weak = readiness_report(Some(130.0), SourceTimingProbeReadinessStatus::Weak);

        let source_timing = choose_grid_bpm(&args, &ready);
        assert_eq!(source_timing.source, GridBpmSource::SourceTiming);
        assert_eq!(
            source_timing.reason,
            GridBpmDecisionReason::SourceTimingReady
        );
        assert!((source_timing.bpm - 130.0).abs() < 0.0001);
        assert_eq!(source_timing.source_delta_bpm, Some(0.0));

        let fallback = choose_grid_bpm(&args, &weak);
        assert_eq!(fallback.source, GridBpmSource::StaticDefault);
        assert_eq!(
            fallback.reason,
            GridBpmDecisionReason::SourceTimingNotReady
        );
        assert!((fallback.bpm - DEFAULT_BPM).abs() < 0.0001);
        assert_eq!(fallback.source_delta_bpm, Some(2.0));
    }

    #[test]
    fn bpm_grid_decision_does_not_auto_trust_manual_confirm_source_timing() {
        let args = bpm_decision_args(DEFAULT_BPM, false);
        let mut manual_confirm_ready =
            readiness_report(Some(130.0), SourceTimingProbeReadinessStatus::Ready);
        manual_confirm_ready.requires_manual_confirm = true;

        let decision = choose_grid_bpm(&args, &manual_confirm_ready);

        assert_eq!(decision.source, GridBpmSource::StaticDefault);
        assert_eq!(
            decision.reason,
            GridBpmDecisionReason::SourceTimingRequiresManualConfirm
        );
        assert!((decision.bpm - DEFAULT_BPM).abs() < 0.0001);
        assert_eq!(decision.source_primary_bpm, Some(130.0));
        assert_eq!(decision.source_delta_bpm, Some(2.0));
    }

    #[test]
    fn bpm_grid_decision_keeps_explicit_bpm_and_reports_source_delta() {
        let args = bpm_decision_args(128.0, true);
        let ready = readiness_report(Some(130.0), SourceTimingProbeReadinessStatus::Ready);

        let decision = choose_grid_bpm(&args, &ready);
        assert_eq!(decision.source, GridBpmSource::UserOverride);
        assert_eq!(decision.reason, GridBpmDecisionReason::UserOverride);
        assert!((decision.bpm - 128.0).abs() < 0.0001);
        assert_eq!(decision.source_primary_bpm, Some(130.0));
        assert_eq!(decision.source_delta_bpm, Some(2.0));
        assert_eq!(source_timing_bpm_agrees(decision.source_delta_bpm), Some(false));
    }

    #[test]
    fn bpm_grid_decision_labels_missing_or_invalid_source_bpm() {
        let args = bpm_decision_args(DEFAULT_BPM, false);

        let missing = choose_grid_bpm(
            &args,
            &readiness_report(None, SourceTimingProbeReadinessStatus::Ready),
        );
        assert_eq!(missing.source, GridBpmSource::StaticDefault);
        assert_eq!(
            missing.reason,
            GridBpmDecisionReason::SourceTimingMissingBpm
        );
        assert_eq!(missing.source_delta_bpm, None);

        let invalid = choose_grid_bpm(
            &args,
            &readiness_report(Some(0.0), SourceTimingProbeReadinessStatus::Ready),
        );
        assert_eq!(invalid.source, GridBpmSource::StaticDefault);
        assert_eq!(
            invalid.reason,
            GridBpmDecisionReason::SourceTimingInvalidBpm
        );
        assert_eq!(invalid.source_delta_bpm, None);
    }

    #[test]
    fn default_args_leave_bpm_available_for_source_timing() {
        let parsed = Args::parse(["--source".to_string(), "source.wav".to_string()])
            .expect("parse args");

        assert_eq!(parsed.bpm, DEFAULT_BPM);
        assert!(!parsed.bpm_overridden);
    }

    #[test]
    fn verification_command_preserves_auto_or_explicit_bpm_mode() {
        let grid = Grid::new(130.0, DEFAULT_BEATS_PER_BAR, 2).expect("grid");
        let auto = bpm_decision_args(DEFAULT_BPM, false);
        let explicit = bpm_decision_args(130.0, true);

        assert!(verification_command(&auto, &grid, 0.5).contains(" auto "));
        assert!(verification_command(&explicit, &grid, 0.5).contains(" 130.000 "));
    }

    fn bpm_decision_args(bpm: f32, bpm_overridden: bool) -> Args {
        Args {
            source_path: PathBuf::from("source.wav"),
            output_dir: None,
            date: "test".into(),
            bpm,
            bpm_overridden,
            bars: 2,
            source_start_seconds: 0.0,
            source_window_seconds: 0.5,
            show_help: false,
        }
    }

    fn readiness_report(
        primary_bpm: Option<f32>,
        readiness: SourceTimingProbeReadinessStatus,
    ) -> SourceTimingProbeReadinessReport {
        SourceTimingProbeReadinessReport {
            schema: "riotbox.source_timing_probe_readiness.v1",
            schema_version: 1,
            source_id: "source.wav".into(),
            primary_bpm,
            primary_downbeat_offset_beats: Some(0),
            beat_status: SourceTimingProbeBeatEvidenceStatus::Stable,
            downbeat_status: SourceTimingProbeDownbeatEvidenceStatus::Stable,
            confidence_result: SourceTimingCandidateConfidenceResult::CandidateCautious,
            drift_status: SourceTimingCandidateDriftStatus::Stable,
            phrase_status: SourceTimingCandidatePhraseStatus::Stable,
            alternate_evidence_count: 0,
            warning_codes: Vec::new(),
            requires_manual_confirm: false,
            readiness,
        }
    }
}
