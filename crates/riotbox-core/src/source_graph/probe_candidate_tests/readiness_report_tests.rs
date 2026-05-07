use super::*;

#[test]
fn source_timing_probe_readiness_report_summarizes_ready_candidate() {
    let onsets = even_onsets(0.0, 0.5, 32);
    let report = source_timing_probe_readiness_report(
        &weighted_candidate_input(
            "readiness-ready-120",
            16.0,
            &onsets,
            &downbeat_strengths(onsets.len(), 4),
        ),
        focused_120_bpm_policy(),
    );

    assert_eq!(report.schema, "riotbox.source_timing_probe_readiness.v1");
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.source_id, "readiness-ready-120");
    assert_bpm_close(report.primary_bpm, 120.0);
    assert_eq!(report.primary_downbeat_offset_beats, Some(0));
    assert_eq!(report.beat_status, SourceTimingProbeBeatEvidenceStatus::Stable);
    assert_eq!(
        report.downbeat_status,
        SourceTimingProbeDownbeatEvidenceStatus::Stable
    );
    assert_eq!(
        report.confidence_result,
        SourceTimingCandidateConfidenceResult::CandidateCautious
    );
    assert_eq!(report.drift_status, SourceTimingCandidateDriftStatus::Stable);
    assert_eq!(
        report.phrase_status,
        SourceTimingCandidatePhraseStatus::Stable
    );
    assert_eq!(report.alternate_evidence_count, 0);
    assert!(report.requires_manual_confirm);
    assert_eq!(report.readiness, SourceTimingProbeReadinessStatus::Ready);
}

#[test]
fn source_timing_probe_readiness_report_summarizes_unavailable_weak_and_review_candidates() {
    let unavailable = source_timing_probe_readiness_report(
        &candidate_input("readiness-unavailable", 4.0, &[0.0, 1.0]),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    assert_eq!(
        unavailable.readiness,
        SourceTimingProbeReadinessStatus::Unavailable
    );
    assert_eq!(
        unavailable.beat_status,
        SourceTimingProbeBeatEvidenceStatus::Unavailable
    );
    assert!(unavailable.requires_manual_confirm);

    let flat_onsets = even_onsets(0.0, 0.5, 32);
    let weak = source_timing_probe_readiness_report(
        &weighted_candidate_input(
            "readiness-weak-downbeat-120",
            16.0,
            &flat_onsets,
            &vec![0.5; flat_onsets.len()],
        ),
        focused_120_bpm_policy(),
    );

    assert_eq!(weak.readiness, SourceTimingProbeReadinessStatus::Weak);
    assert_eq!(
        weak.downbeat_status,
        SourceTimingProbeDownbeatEvidenceStatus::Weak
    );
    assert!(weak.requires_manual_confirm);

    let review_onsets = even_onsets(0.0, 0.5, 8);
    let needs_review = source_timing_probe_readiness_report(
        &weighted_candidate_input(
            "readiness-ambiguous-120",
            4.0,
            &review_onsets,
            &downbeat_strengths(review_onsets.len(), 4),
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    assert_eq!(
        needs_review.readiness,
        SourceTimingProbeReadinessStatus::NeedsReview
    );
    assert_eq!(
        needs_review.confidence_result,
        SourceTimingCandidateConfidenceResult::CandidateAmbiguous
    );
    assert!(needs_review.alternate_evidence_count > 0);
    assert!(needs_review.requires_manual_confirm);
    assert!(
        needs_review
            .warning_codes
            .contains(&TimingWarningCode::HalfTimePossible)
            || needs_review
                .warning_codes
                .contains(&TimingWarningCode::DoubleTimePossible)
    );
}
