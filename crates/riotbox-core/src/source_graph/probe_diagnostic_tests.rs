use super::*;

#[test]
fn source_timing_probe_diagnostics_degrade_silence_to_disabled_unknown_timing() {
    let timing = timing_model_from_probe_diagnostics(
        &SourceTimingProbeDiagnosticInput {
            source_id: "silence".into(),
            duration_seconds: 2.0,
            peak_energy: 0.0,
            peak_positive_flux: 0.0,
            onset_count: 0,
            onset_density_per_second: 0.0,
        },
        SourceTimingProbeDiagnosticPolicy::default(),
    );

    assert_eq!(timing.bpm_estimate, None);
    assert_eq!(timing.effective_timing_quality(), TimingQuality::Unknown);
    assert_eq!(
        timing.effective_degraded_policy(),
        TimingDegradedPolicy::Disabled
    );
    assert!(timing.beat_grid.is_empty());
    assert!(timing.primary_hypothesis().is_none());
    assert!(has_warning(&timing, TimingWarningCode::LowTimingConfidence));
    assert!(has_warning(&timing, TimingWarningCode::WeakKickAnchor));
}

#[test]
fn source_timing_probe_diagnostics_keep_sparse_onsets_low_confidence() {
    let timing = timing_model_from_probe_diagnostics(
        &SourceTimingProbeDiagnosticInput {
            source_id: "sparse".into(),
            duration_seconds: 4.0,
            peak_energy: 0.25,
            peak_positive_flux: 0.30,
            onset_count: 1,
            onset_density_per_second: 0.25,
        },
        SourceTimingProbeDiagnosticPolicy::default(),
    );

    assert_eq!(timing.bpm_estimate, None);
    assert_eq!(timing.effective_timing_quality(), TimingQuality::Low);
    assert_eq!(
        timing.effective_degraded_policy(),
        TimingDegradedPolicy::Disabled
    );
    assert!(timing.bpm_confidence < 0.5);
    assert!(has_warning(&timing, TimingWarningCode::LowTimingConfidence));
}

#[test]
fn source_timing_probe_diagnostics_require_manual_confirm_for_rich_onsets() {
    let timing = timing_model_from_probe_diagnostics(
        &SourceTimingProbeDiagnosticInput {
            source_id: "impulses".into(),
            duration_seconds: 2.0,
            peak_energy: 0.50,
            peak_positive_flux: 0.45,
            onset_count: 6,
            onset_density_per_second: 3.0,
        },
        SourceTimingProbeDiagnosticPolicy::default(),
    );

    assert_eq!(timing.bpm_estimate, None);
    assert_eq!(timing.effective_timing_quality(), TimingQuality::Low);
    assert_eq!(
        timing.effective_degraded_policy(),
        TimingDegradedPolicy::ManualConfirm
    );
    assert!(timing.bpm_confidence < 0.5);
    assert!(timing.beat_grid.is_empty());
    assert!(has_warning(&timing, TimingWarningCode::AmbiguousDownbeat));
    assert!(has_warning(&timing, TimingWarningCode::PhraseUncertain));
}

fn has_warning(timing: &TimingModel, expected: TimingWarningCode) -> bool {
    timing
        .warnings
        .iter()
        .any(|warning| warning.code == expected)
}
