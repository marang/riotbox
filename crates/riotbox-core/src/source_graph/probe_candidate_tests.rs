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
