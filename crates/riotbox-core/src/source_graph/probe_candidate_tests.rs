use super::*;

#[test]
fn source_timing_probe_bpm_candidates_estimate_clean_synthetic_spacing() {
    let timing = timing_model_from_probe_bpm_candidates(
        &candidate_input("clean-120", 4.0, &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5]),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    assert_bpm_close(timing.bpm_estimate, 120.0);
    assert_eq!(timing.effective_timing_quality(), TimingQuality::Medium);
    assert_eq!(
        timing.effective_degraded_policy(),
        TimingDegradedPolicy::Cautious
    );
    assert_eq!(
        timing.primary_hypothesis().map(|hypothesis| hypothesis.kind),
        Some(TimingHypothesisKind::Primary)
    );
    assert!(!timing.beat_grid.is_empty());
    assert!(has_warning(&timing, TimingWarningCode::AmbiguousDownbeat));
    assert!(has_warning(&timing, TimingWarningCode::PhraseUncertain));
    let primary = timing.primary_hypothesis().expect("primary hypothesis");
    assert!(primary.score > 0.0);
    assert!(primary
        .provenance
        .contains(&"source-timing-probe.beat-period-score.v0".into()));
}

#[test]
fn source_timing_probe_bpm_candidates_preserve_half_and_double_time_ambiguity() {
    let timing = timing_model_from_probe_bpm_candidates(
        &candidate_input("ambiguous-120", 4.0, &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5]),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    assert!(timing
        .hypotheses
        .iter()
        .any(|hypothesis| hypothesis.kind == TimingHypothesisKind::HalfTime));
    assert!(timing
        .hypotheses
        .iter()
        .any(|hypothesis| hypothesis.kind == TimingHypothesisKind::DoubleTime));
    assert!(has_warning(&timing, TimingWarningCode::HalfTimePossible));
    assert!(has_warning(&timing, TimingWarningCode::DoubleTimePossible));
}

#[test]
fn source_timing_probe_bpm_candidates_preserve_period_score_ambiguity() {
    let timing = timing_model_from_probe_bpm_candidates(
        &candidate_input(
            "period-ambiguous-120",
            4.0,
            &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5],
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    assert_bpm_close(timing.bpm_estimate, 120.0);
    assert!(timing
        .hypotheses
        .iter()
        .any(|hypothesis| hypothesis.kind == TimingHypothesisKind::HalfTime));
    assert!(timing
        .hypotheses
        .iter()
        .any(|hypothesis| hypothesis.kind == TimingHypothesisKind::DoubleTime));
    assert!(timing.hypotheses.iter().all(|hypothesis| hypothesis
        .provenance
        .contains(&"source-timing-probe.beat-period-score.v0".into())));
}

#[test]
fn source_timing_probe_bpm_candidates_score_uneven_onsets_without_collapsing() {
    let timing = timing_model_from_probe_bpm_candidates(
        &candidate_input(
            "uneven-120",
            4.0,
            &[0.0, 0.5, 1.01, 1.52, 2.0, 2.49, 3.01, 3.48],
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    assert_bpm_close(timing.bpm_estimate, 120.0);
    let primary = timing.primary_hypothesis().expect("primary hypothesis");
    assert_eq!(primary.kind, TimingHypothesisKind::Primary);
    assert!(primary.score > 0.0);
    assert!(!primary.beat_grid.is_empty());
    assert!(has_warning(&timing, TimingWarningCode::AmbiguousDownbeat));
}

#[test]
fn source_timing_probe_bpm_candidates_preserve_alternate_downbeat_phases() {
    let timing = timing_model_from_probe_bpm_candidates(
        &candidate_input(
            "phase-ambiguous-120",
            4.0,
            &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5],
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    let alternate_downbeats = timing
        .hypotheses
        .iter()
        .filter(|hypothesis| hypothesis.kind == TimingHypothesisKind::AlternateDownbeat)
        .collect::<Vec<_>>();
    assert_eq!(alternate_downbeats.len(), 3);
    assert!(has_warning(&timing, TimingWarningCode::AmbiguousDownbeat));
    assert!(alternate_downbeats.iter().all(|hypothesis| hypothesis
        .bar_grid
        .first()
        .is_some_and(|bar| bar.start_seconds > 0.0)));
}

#[test]
fn source_timing_probe_bpm_candidates_keep_primary_bar_grid_phase_when_clearer() {
    let timing = timing_model_from_probe_bpm_candidates(
        &candidate_input(
            "phase-weighted-120",
            4.0,
            &[0.0, 0.0, 0.5, 1.0, 1.5, 2.0, 2.0, 2.5, 3.0, 3.5],
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    let primary = timing.primary_hypothesis().expect("primary hypothesis");
    assert_eq!(
        primary.bar_grid.first().map(|bar| bar.start_seconds),
        Some(0.0)
    );
    assert!(primary
        .bar_grid
        .first()
        .is_some_and(|bar| bar.downbeat_confidence > 0.0));
    assert!(!timing
        .hypotheses
        .iter()
        .any(|hypothesis| hypothesis.kind == TimingHypothesisKind::AlternateDownbeat));
}

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
    assert_eq!(report.degraded_policy, TimingDegradedPolicy::Disabled);
    assert!(report.requires_manual_confirm);
    assert!(report
        .warning_codes
        .contains(&TimingWarningCode::LowTimingConfidence));
}

#[test]
fn source_timing_probe_bpm_candidates_sort_and_filter_public_input() {
    let timing = timing_model_from_probe_bpm_candidates(
        &candidate_input(
            "messy-120",
            4.0,
            &[2.0, f32::NAN, 0.0, 9.0, 1.5, -1.0, 0.5, 1.0],
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    assert_bpm_close(timing.bpm_estimate, 120.0);
    assert_eq!(
        timing
            .primary_hypothesis()
            .expect("primary hypothesis")
            .anchors
            .first()
            .map(|anchor| anchor.time_seconds),
        Some(0.0)
    );
}

#[test]
fn source_timing_probe_bpm_candidates_degrade_insufficient_onsets() {
    let timing = timing_model_from_probe_bpm_candidates(
        &candidate_input("too-sparse", 4.0, &[0.0, 1.0]),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    assert_eq!(timing.bpm_estimate, None);
    assert_eq!(timing.effective_timing_quality(), TimingQuality::Unknown);
    assert_eq!(
        timing.effective_degraded_policy(),
        TimingDegradedPolicy::Disabled
    );
    assert!(has_warning(&timing, TimingWarningCode::LowTimingConfidence));
}

fn candidate_input(
    source_id: &str,
    duration_seconds: f32,
    onset_times_seconds: &[f32],
) -> SourceTimingProbeBpmCandidateInput {
    SourceTimingProbeBpmCandidateInput {
        source_id: source_id.into(),
        duration_seconds,
        onset_times_seconds: onset_times_seconds.to_vec(),
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
    }
}

fn assert_bpm_close(actual: Option<f32>, expected: f32) {
    let actual = actual.expect("bpm estimate");
    assert!((actual - expected).abs() <= 0.01, "{actual} != {expected}");
}

fn has_warning(timing: &TimingModel, expected: TimingWarningCode) -> bool {
    timing
        .warnings
        .iter()
        .any(|warning| warning.code == expected)
}
