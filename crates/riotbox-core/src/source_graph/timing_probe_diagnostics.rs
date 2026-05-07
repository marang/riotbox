#[derive(Clone, Debug, PartialEq)]
pub struct SourceTimingProbeDiagnosticInput {
    pub source_id: String,
    pub duration_seconds: f32,
    pub peak_energy: f32,
    pub peak_positive_flux: f32,
    pub onset_count: usize,
    pub onset_density_per_second: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceTimingProbeDiagnosticPolicy {
    pub min_peak_energy: f32,
    pub min_positive_flux: f32,
    pub min_onset_count: usize,
    pub min_onset_density_per_second: f32,
}

impl Default for SourceTimingProbeDiagnosticPolicy {
    fn default() -> Self {
        Self {
            min_peak_energy: 0.02,
            min_positive_flux: 0.02,
            min_onset_count: 4,
            min_onset_density_per_second: 1.0,
        }
    }
}

#[must_use]
pub fn timing_model_from_probe_diagnostics(
    input: &SourceTimingProbeDiagnosticInput,
    policy: SourceTimingProbeDiagnosticPolicy,
) -> TimingModel {
    let has_signal =
        input.duration_seconds > 0.0 && input.peak_energy >= policy.min_peak_energy.max(0.0);
    let has_onset_evidence = input.peak_positive_flux >= policy.min_positive_flux.max(0.0)
        && input.onset_count >= policy.min_onset_count
        && input.onset_density_per_second >= policy.min_onset_density_per_second.max(0.0);

    let (quality, degraded_policy, confidence, warnings) = match (has_signal, has_onset_evidence) {
        (true, true) => (
            TimingQuality::Low,
            TimingDegradedPolicy::ManualConfirm,
            0.30,
            vec![
                TimingWarningCode::AmbiguousDownbeat,
                TimingWarningCode::PhraseUncertain,
            ],
        ),
        (true, false) => (
            TimingQuality::Low,
            TimingDegradedPolicy::Disabled,
            0.10,
            vec![
                TimingWarningCode::LowTimingConfidence,
                TimingWarningCode::WeakKickAnchor,
                TimingWarningCode::PhraseUncertain,
            ],
        ),
        (false, _) => (
            TimingQuality::Unknown,
            TimingDegradedPolicy::Disabled,
            0.0,
            vec![
                TimingWarningCode::LowTimingConfidence,
                TimingWarningCode::WeakKickAnchor,
            ],
        ),
    };

    TimingModel {
        bpm_estimate: None,
        bpm_confidence: confidence,
        meter_hint: None,
        beat_grid: Vec::new(),
        bar_grid: Vec::new(),
        phrase_grid: Vec::new(),
        hypotheses: Vec::new(),
        primary_hypothesis_id: None,
        quality,
        warnings: warnings
            .into_iter()
            .map(|code| TimingWarning {
                code,
                message: probe_timing_warning_message(code, input),
            })
            .collect(),
        degraded_policy,
    }
}

fn probe_timing_warning_message(
    code: TimingWarningCode,
    input: &SourceTimingProbeDiagnosticInput,
) -> String {
    match code {
        TimingWarningCode::LowTimingConfidence => {
            format!("probe evidence is too weak to lock timing for {}", input.source_id)
        }
        TimingWarningCode::WeakKickAnchor => {
            "probe has no trusted low-end or downbeat anchor yet".into()
        }
        TimingWarningCode::AmbiguousDownbeat => {
            "probe found onset evidence but no downbeat scoring yet".into()
        }
        TimingWarningCode::PhraseUncertain => {
            "probe found onset evidence but no phrase boundary scoring yet".into()
        }
        TimingWarningCode::WeakBackbeatAnchor => "probe has no trusted backbeat anchor yet".into(),
        TimingWarningCode::HalfTimePossible => "probe has not disambiguated half-time yet".into(),
        TimingWarningCode::DoubleTimePossible => {
            "probe has not disambiguated double-time yet".into()
        }
        TimingWarningCode::DriftHigh => "probe has no drift model yet".into(),
    }
}

#[cfg(test)]
mod probe_diagnostic_tests;
