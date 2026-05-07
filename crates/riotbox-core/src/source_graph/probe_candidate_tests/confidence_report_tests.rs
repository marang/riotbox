use super::*;

#[test]
fn source_timing_candidate_confidence_report_summarizes_ambiguous_candidate() {
    let timing = timing_model_from_probe_bpm_candidates(
        &candidate_input(
            "report-ambiguous-120",
            4.0,
            &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5],
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    let report = source_timing_candidate_confidence_report(&timing);

    assert_eq!(report.schema, "riotbox.source_timing_candidate_confidence.v1");
    assert_eq!(report.schema_version, 1);
    assert_bpm_close(report.primary_bpm, 120.0);
    assert_eq!(
        report.result,
        SourceTimingCandidateConfidenceResult::CandidateAmbiguous
    );
    assert_eq!(report.alternate_downbeat_count, 3);
    assert_eq!(report.half_time_count, 1);
    assert_eq!(report.double_time_count, 1);
    assert_eq!(
        report.primary_drift_status,
        SourceTimingCandidateDriftStatus::NotEnoughMaterial
    );
    assert_eq!(
        report.primary_phrase_status,
        SourceTimingCandidatePhraseStatus::NotEnoughMaterial
    );
    assert_eq!(report.primary_phrase_count, 0);
    assert!(report.primary_phrase_confidence.is_none());
    assert_eq!(report.primary_drift_window_count, 0);
    assert_eq!(report.primary_drift_max_ms, None);
    assert!(report.requires_manual_confirm);
    assert!(report.primary_downbeat_confidence.is_some_and(|value| value > 0.0));
    assert!(report
        .warning_codes
        .contains(&TimingWarningCode::AmbiguousDownbeat));
}

#[test]
fn source_timing_candidate_confidence_report_summarizes_degraded_probe() {
    let timing = timing_model_from_probe_bpm_candidates(
        &candidate_input("report-sparse", 4.0, &[0.0, 1.0]),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    let report = source_timing_candidate_confidence_report(&timing);

    assert_eq!(report.primary_bpm, None);
    assert_eq!(report.hypothesis_count, 0);
    assert_eq!(
        report.result,
        SourceTimingCandidateConfidenceResult::Degraded
    );
    assert_eq!(
        report.primary_drift_status,
        SourceTimingCandidateDriftStatus::Unavailable
    );
    assert_eq!(
        report.primary_phrase_status,
        SourceTimingCandidatePhraseStatus::Unavailable
    );
    assert_eq!(report.primary_phrase_count, 0);
    assert_eq!(report.primary_phrase_bar_count, 0);
    assert_eq!(report.degraded_policy, TimingDegradedPolicy::Disabled);
    assert!(report.requires_manual_confirm);
    assert!(report
        .warning_codes
        .contains(&TimingWarningCode::LowTimingConfidence));
}

#[test]
fn source_timing_candidate_confidence_report_summarizes_primary_drift() {
    let stable = source_timing_candidate_confidence_report(&timing_model_from_probe_bpm_candidates(
        &candidate_input(
            "report-stable-drift-120",
            16.0,
            &even_onsets(0.0, 0.5, 32),
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    ));

    assert_eq!(
        stable.primary_drift_status,
        SourceTimingCandidateDriftStatus::Stable
    );
    assert_eq!(stable.primary_drift_window_count, 2);
    assert!(stable.primary_drift_max_ms.is_some_and(|value| value <= 0.01));
    assert!(stable
        .primary_drift_mean_abs_ms
        .is_some_and(|value| value <= 0.01));
    assert!(stable.primary_drift_end_ms.is_some_and(|value| value <= 0.01));
    assert!(stable
        .primary_drift_confidence
        .is_some_and(|value| value > 0.0));

    let mut drifting_onsets = even_onsets(0.0, 0.5, 32);
    for time_seconds in drifting_onsets.iter_mut().skip(16) {
        *time_seconds += 0.12;
    }
    let drifting =
        source_timing_candidate_confidence_report(&timing_model_from_probe_bpm_candidates(
            &candidate_input("report-high-drift-120", 16.0, &drifting_onsets),
            SourceTimingProbeBpmCandidatePolicy::default(),
        ));

    assert_eq!(
        drifting.primary_drift_status,
        SourceTimingCandidateDriftStatus::High
    );
    assert!(drifting
        .primary_drift_max_ms
        .is_some_and(|value| value > 100.0));
    assert!(drifting.warning_codes.contains(&TimingWarningCode::DriftHigh));
    assert_eq!(
        drifting.primary_phrase_status,
        SourceTimingCandidatePhraseStatus::HighDrift
    );
    assert!(drifting.requires_manual_confirm);
}

#[test]
fn source_timing_candidate_confidence_report_summarizes_primary_phrase_grid() {
    let onsets = even_onsets(0.0, 0.5, 32);
    let stable = source_timing_candidate_confidence_report(&timing_model_from_probe_bpm_candidates(
        &weighted_candidate_input(
            "report-stable-phrase-120",
            16.0,
            &onsets,
            &downbeat_strengths(onsets.len(), 4),
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    ));

    assert_eq!(
        stable.primary_phrase_status,
        SourceTimingCandidatePhraseStatus::Stable
    );
    assert_eq!(stable.primary_phrase_count, 2);
    assert_eq!(stable.primary_phrase_bar_count, 8);
    assert!(stable
        .primary_phrase_confidence
        .is_some_and(|value| value >= 0.4));

    let ambiguous = source_timing_candidate_confidence_report(
        &timing_model_from_probe_bpm_candidates(
            &weighted_candidate_input(
                "report-ambiguous-phrase-120",
                16.0,
                &onsets,
                &vec![0.5; onsets.len()],
            ),
            SourceTimingProbeBpmCandidatePolicy::default(),
        ),
    );

    assert_eq!(
        ambiguous.primary_phrase_status,
        SourceTimingCandidatePhraseStatus::AmbiguousDownbeat
    );
    assert!(ambiguous
        .warning_codes
        .contains(&TimingWarningCode::AmbiguousDownbeat));
}
