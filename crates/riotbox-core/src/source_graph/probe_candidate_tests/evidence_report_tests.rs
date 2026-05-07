use super::*;

#[test]
fn source_timing_probe_beat_evidence_report_summarizes_stable_candidate() {
    let report = source_timing_probe_beat_evidence_report(
        &candidate_input(
            "beat-evidence-stable-120",
            16.0,
            &even_onsets(0.0, 0.5, 32),
        ),
        focused_120_bpm_policy(),
    );

    assert_eq!(report.schema, "riotbox.source_timing_probe_beat_evidence.v1");
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.source_id, "beat-evidence-stable-120");
    assert_eq!(report.onset_count, 32);
    assert_eq!(report.status, SourceTimingProbeBeatEvidenceStatus::Stable);
    assert_bpm_close(report.primary_bpm, 120.0);
    assert!(report.primary_score.is_some_and(|score| score >= 0.95));
    assert!(report
        .primary_matched_onset_ratio
        .is_some_and(|ratio| ratio >= 0.95));
    assert!(report
        .primary_median_distance_ratio
        .is_some_and(|ratio| ratio <= 0.01));
    assert_eq!(report.alternate_candidate_count, 0);
}

#[test]
fn source_timing_probe_beat_evidence_report_summarizes_unavailable_weak_and_ambiguous_candidates()
{
    let unavailable = source_timing_probe_beat_evidence_report(
        &candidate_input("beat-evidence-weak", 4.0, &[0.0, 1.0]),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    assert_eq!(
        unavailable.status,
        SourceTimingProbeBeatEvidenceStatus::Unavailable
    );
    assert_eq!(unavailable.primary_bpm, None);
    assert_eq!(unavailable.candidate_count, 0);

    let weak = source_timing_probe_beat_evidence_report(
        &candidate_input(
            "beat-evidence-threshold-weak-120",
            16.0,
            &even_onsets(0.0, 0.5, 32),
        ),
        SourceTimingProbeBpmCandidatePolicy {
            min_beat_period_score: 1.1,
            ..focused_120_bpm_policy()
        },
    );

    assert_eq!(weak.status, SourceTimingProbeBeatEvidenceStatus::Weak);
    assert_bpm_close(weak.primary_bpm, 120.0);

    let ambiguous = source_timing_probe_beat_evidence_report(
        &candidate_input(
            "beat-evidence-ambiguous-120",
            4.0,
            &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5],
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    assert_eq!(
        ambiguous.status,
        SourceTimingProbeBeatEvidenceStatus::Ambiguous
    );
    assert_bpm_close(ambiguous.primary_bpm, 120.0);
    assert!(ambiguous.alternate_candidate_count > 0);
}

#[test]
fn source_timing_probe_downbeat_evidence_report_summarizes_stable_candidate() {
    let onsets = even_onsets(0.0, 0.5, 32);
    let report = source_timing_probe_downbeat_evidence_report(
        &weighted_candidate_input(
            "downbeat-evidence-stable-120",
            16.0,
            &onsets,
            &downbeat_strengths(onsets.len(), 4),
        ),
        120.0,
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    assert_eq!(
        report.schema,
        "riotbox.source_timing_probe_downbeat_evidence.v1"
    );
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.source_id, "downbeat-evidence-stable-120");
    assert_eq!(report.status, SourceTimingProbeDownbeatEvidenceStatus::Stable);
    assert_eq!(report.phase_count, 4);
    assert_eq!(report.primary_offset_beats, Some(0));
    assert!(report.primary_score.is_some_and(|score| score >= 0.45));
    assert_eq!(report.alternate_phase_count, 0);
}

#[test]
fn source_timing_probe_downbeat_evidence_report_summarizes_unavailable_weak_and_ambiguous_candidates(
) {
    let unavailable = source_timing_probe_downbeat_evidence_report(
        &candidate_input("downbeat-evidence-unavailable", 4.0, &[]),
        120.0,
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    assert_eq!(
        unavailable.status,
        SourceTimingProbeDownbeatEvidenceStatus::Unavailable
    );
    assert_eq!(unavailable.primary_offset_beats, None);

    let flat_onsets = even_onsets(0.0, 0.5, 32);
    let weak = source_timing_probe_downbeat_evidence_report(
        &weighted_candidate_input(
            "downbeat-evidence-weak-flat-120",
            16.0,
            &flat_onsets,
            &vec![0.5; flat_onsets.len()],
        ),
        120.0,
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    assert_eq!(weak.status, SourceTimingProbeDownbeatEvidenceStatus::Weak);
    assert!(weak.primary_score.is_some_and(|score| score < 0.4));

    let ambiguous = source_timing_probe_downbeat_evidence_report(
        &weighted_candidate_input(
            "downbeat-evidence-ambiguous-120",
            8.0,
            &[0.0, 0.5, 2.0, 2.5, 4.0, 4.5, 6.0, 6.5],
            &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        ),
        120.0,
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    assert_eq!(
        ambiguous.status,
        SourceTimingProbeDownbeatEvidenceStatus::Ambiguous
    );
    assert!(ambiguous.primary_score.is_some_and(|score| score >= 0.4));
    assert_eq!(ambiguous.alternate_phase_count, 1);
}
