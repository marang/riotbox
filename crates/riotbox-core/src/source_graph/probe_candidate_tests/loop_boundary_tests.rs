use super::*;

#[test]
fn repeated_full_bar_loop_prior_prefers_file_boundary_without_claiming_stability() {
    let input = weighted_candidate_input(
        "repeated-boundary-120",
        4.0,
        &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5],
        &[0.8, 0.3, 0.9, 0.3, 0.8, 0.3, 0.9, 0.3],
    );
    let policy = SourceTimingProbeBpmCandidatePolicy::default();
    let timing = timing_model_from_probe_bpm_candidates(&input, policy);
    let evidence = source_timing_probe_downbeat_evidence_report(&input, 120.0, policy);

    let primary = timing.primary_hypothesis().expect("primary hypothesis");
    assert_eq!(
        primary.bar_grid.first().map(|bar| bar.start_seconds),
        Some(0.0)
    );
    assert!(
        primary
            .provenance
            .contains(&"source-timing-probe.repeated-loop-boundary-prior.v1".into()),
        "{primary:?}"
    );
    assert!(has_warning(&timing, TimingWarningCode::AmbiguousDownbeat));
    assert!(timing.warnings.iter().any(|warning| {
        warning.code == TimingWarningCode::AmbiguousDownbeat
            && warning.message
                == "repeated full-bar loop suggests the file boundary, but alternate bar starts require confirmation"
    }));
    assert_eq!(evidence.primary_offset_beats, Some(0));
    assert_eq!(
        evidence.status,
        SourceTimingProbeDownbeatEvidenceStatus::Ambiguous
    );
    assert!(evidence.alternate_phase_count > 0);
}

#[test]
fn repeated_loop_prior_does_not_apply_when_duration_is_not_full_bars() {
    let input = weighted_candidate_input(
        "partial-boundary-120",
        3.75,
        &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5],
        &[0.8, 0.3, 0.9, 0.3, 0.8, 0.3, 0.9, 0.3],
    );
    let timing = timing_model_from_probe_bpm_candidates(
        &input,
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    let primary = timing.primary_hypothesis().expect("primary hypothesis");
    assert_eq!(
        primary.bar_grid.first().map(|bar| bar.start_seconds),
        Some(1.0)
    );
    assert!(
        !primary
            .provenance
            .contains(&"source-timing-probe.repeated-loop-boundary-prior.v1".into()),
        "{primary:?}"
    );
}

#[test]
fn repeated_loop_prior_does_not_override_clear_accent_phase() {
    let input = weighted_candidate_input(
        "clear-non-boundary-120",
        4.0,
        &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5],
        &[0.2, 0.2, 2.0, 0.2, 0.2, 0.2, 2.0, 0.2],
    );
    let timing = timing_model_from_probe_bpm_candidates(
        &input,
        SourceTimingProbeBpmCandidatePolicy::default(),
    );

    let primary = timing.primary_hypothesis().expect("primary hypothesis");
    assert_eq!(
        primary.bar_grid.first().map(|bar| bar.start_seconds),
        Some(1.0)
    );
    assert!(
        !primary
            .provenance
            .contains(&"source-timing-probe.repeated-loop-boundary-prior.v1".into()),
        "{primary:?}"
    );
}

#[test]
fn repeated_loop_prior_requires_an_onset_at_the_file_boundary() {
    let input = weighted_candidate_input(
        "no-boundary-onset-120",
        4.0,
        &[0.25, 0.5, 1.0, 1.5, 2.25, 2.5, 3.0, 3.5],
        &[0.8, 0.3, 0.9, 0.3, 0.8, 0.3, 0.9, 0.3],
    );

    assert!(!repeated_full_bar_loop_supports_file_boundary(
        &input, 120.0
    ));
}

#[test]
fn repeated_loop_prior_rejects_a_nonrepeating_second_bar() {
    let input = weighted_candidate_input(
        "nonrepeating-bars-120",
        4.0,
        &[0.0, 0.5, 1.0, 1.5, 2.0, 2.25, 2.75, 3.5],
        &[0.8, 0.3, 0.9, 0.3, 0.8, 0.3, 0.9, 0.3],
    );

    assert!(!repeated_full_bar_loop_supports_file_boundary(
        &input, 120.0
    ));
}

#[test]
fn repeated_loop_prior_rejects_mismatched_repeated_accent_strengths() {
    let input = weighted_candidate_input(
        "mismatched-bar-accents-120",
        4.0,
        &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5],
        &[1.0, 0.2, 1.0, 0.2, 0.2, 1.0, 0.2, 1.0],
    );

    assert!(!repeated_full_bar_loop_supports_file_boundary(
        &input, 120.0
    ));
}
