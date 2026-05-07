use super::*;

mod confidence_report_tests;
mod evidence_report_tests;
mod readiness_report_tests;

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
    assert!(primary
        .provenance
        .contains(&"source-timing-probe.downbeat-accent-score.v0".into()));
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
        &weighted_candidate_input(
            "phase-weighted-120",
            4.0,
            &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5],
            &[1.0, 0.25, 0.25, 0.25, 1.0, 0.25, 0.25, 0.25],
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
    assert!(!has_warning(&timing, TimingWarningCode::AmbiguousDownbeat));
    assert!(!timing
        .hypotheses
        .iter()
        .any(|hypothesis| hypothesis.kind == TimingHypothesisKind::AlternateDownbeat));
}

#[test]
fn source_timing_probe_bpm_candidates_keep_ambiguous_downbeats_when_accents_are_flat() {
    let timing = timing_model_from_probe_bpm_candidates(
        &weighted_candidate_input(
            "phase-flat-120",
            4.0,
            &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5],
            &[0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    let alternate_downbeats = timing
        .hypotheses
        .iter()
        .filter(|hypothesis| hypothesis.kind == TimingHypothesisKind::AlternateDownbeat)
        .count();
    assert_eq!(alternate_downbeats, 3);
    assert!(has_warning(&timing, TimingWarningCode::AmbiguousDownbeat));
}

#[test]
fn source_timing_probe_bpm_candidates_report_stable_grid_drift() {
    let timing = timing_model_from_probe_bpm_candidates(
        &candidate_input(
            "stable-drift-120",
            16.0,
            &even_onsets(0.0, 0.5, 32),
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    let primary = timing.primary_hypothesis().expect("primary hypothesis");
    assert!(primary
        .provenance
        .contains(&"source-timing-probe.drift-report.v0".into()));
    let four_bar = primary
        .drift
        .iter()
        .find(|report| report.window_bars == 4)
        .expect("4-bar drift report");
    let eight_bar = primary
        .drift
        .iter()
        .find(|report| report.window_bars == 8)
        .expect("8-bar drift report");
    assert!(four_bar.max_drift_ms <= 0.01, "{four_bar:?}");
    assert!(eight_bar.mean_abs_drift_ms <= 0.01, "{eight_bar:?}");
    assert!(primary.drift.iter().all(|report| report.window_bars <= 8));
    assert!(!has_warning(&timing, TimingWarningCode::DriftHigh));
}

#[test]
fn source_timing_probe_bpm_candidates_report_long_grid_drift_windows() {
    let timing = timing_model_from_probe_bpm_candidates(
        &candidate_input(
            "long-stable-drift-120",
            64.0,
            &even_onsets(0.0, 0.5, 128),
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    let primary = timing.primary_hypothesis().expect("primary hypothesis");
    let reported_windows = primary
        .drift
        .iter()
        .map(|report| report.window_bars)
        .collect::<Vec<_>>();
    assert_eq!(reported_windows, vec![4, 8, 16, 32]);
    assert!(primary
        .drift
        .iter()
        .all(|report| report.max_drift_ms <= 0.01));
    assert!(!has_warning(&timing, TimingWarningCode::DriftHigh));
}

#[test]
fn source_timing_probe_bpm_candidates_add_phrase_grid_when_bar_timing_is_stable() {
    let onsets = even_onsets(0.0, 0.5, 32);
    let timing = timing_model_from_probe_bpm_candidates(
        &weighted_candidate_input(
            "stable-phrase-120",
            16.0,
            &onsets,
            &downbeat_strengths(onsets.len(), 4),
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    let primary = timing.primary_hypothesis().expect("primary hypothesis");
    assert_eq!(primary.phrase_grid.len(), 2);
    assert_eq!(timing.phrase_grid, primary.phrase_grid);
    let phrase_spans = primary
        .phrase_grid
        .iter()
        .map(|phrase| (phrase.phrase_index, phrase.start_bar, phrase.end_bar))
        .collect::<Vec<_>>();
    assert_eq!(phrase_spans, vec![(1, 1, 4), (2, 5, 8)]);
    assert!(primary
        .provenance
        .contains(&"source-timing-probe.phrase-grid.v0".into()));
    assert!(!has_warning(&timing, TimingWarningCode::PhraseUncertain));
}

#[test]
fn source_timing_probe_bpm_candidates_keep_phrase_uncertain_for_short_material() {
    let onsets = even_onsets(0.0, 0.5, 8);
    let timing = timing_model_from_probe_bpm_candidates(
        &weighted_candidate_input(
            "short-phrase-120",
            4.0,
            &onsets,
            &downbeat_strengths(onsets.len(), 4),
        ),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    assert!(timing.phrase_grid.is_empty());
    assert!(timing
        .primary_hypothesis()
        .expect("primary hypothesis")
        .phrase_grid
        .is_empty());
    assert!(has_warning(&timing, TimingWarningCode::PhraseUncertain));
}

#[test]
fn source_timing_probe_bpm_candidates_warn_when_grid_drift_is_high() {
    let mut onsets = even_onsets(0.0, 0.5, 32);
    for time_seconds in onsets.iter_mut().skip(16) {
        *time_seconds += 0.12;
    }
    let timing = timing_model_from_probe_bpm_candidates(
        &candidate_input("late-drift-120", 16.0, &onsets),
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    let primary = timing.primary_hypothesis().expect("primary hypothesis");
    assert!(primary.drift.iter().any(|report| report.max_drift_ms > 100.0));
    assert!(primary.phrase_grid.is_empty());
    assert!(has_warning(&timing, TimingWarningCode::DriftHigh));
    assert!(has_warning(&timing, TimingWarningCode::PhraseUncertain));
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
    weighted_candidate_input(
        source_id,
        duration_seconds,
        onset_times_seconds,
        &vec![1.0; onset_times_seconds.len()],
    )
}

fn weighted_candidate_input(
    source_id: &str,
    duration_seconds: f32,
    onset_times_seconds: &[f32],
    onset_strengths: &[f32],
) -> SourceTimingProbeBpmCandidateInput {
    SourceTimingProbeBpmCandidateInput {
        source_id: source_id.into(),
        duration_seconds,
        onset_times_seconds: onset_times_seconds.to_vec(),
        onset_strengths: onset_strengths.to_vec(),
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
    }
}

fn even_onsets(start_seconds: f32, period_seconds: f32, count: usize) -> Vec<f32> {
    (0..count)
        .map(|index| start_seconds + period_seconds * index as f32)
        .collect()
}

fn downbeat_strengths(count: usize, beats_per_bar: usize) -> Vec<f32> {
    (0..count)
        .map(|index| {
            if index % beats_per_bar == 0 {
                2.0
            } else {
                0.5
            }
        })
        .collect()
}

fn focused_120_bpm_policy() -> SourceTimingProbeBpmCandidatePolicy {
    SourceTimingProbeBpmCandidatePolicy {
        min_bpm: 80.0,
        max_bpm: 180.0,
        ..SourceTimingProbeBpmCandidatePolicy::default()
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
