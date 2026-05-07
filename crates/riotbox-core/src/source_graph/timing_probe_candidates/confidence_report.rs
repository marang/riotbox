#[must_use]
pub fn source_timing_candidate_confidence_report(
    timing: &TimingModel,
) -> SourceTimingCandidateConfidenceReport {
    let alternate_downbeat_count = count_hypotheses(timing, TimingHypothesisKind::AlternateDownbeat);
    let half_time_count = count_hypotheses(timing, TimingHypothesisKind::HalfTime);
    let double_time_count = count_hypotheses(timing, TimingHypothesisKind::DoubleTime);
    let warning_codes = timing
        .warnings
        .iter()
        .map(|warning| warning.code)
        .collect::<Vec<_>>();
    let primary_downbeat_confidence = timing
        .primary_hypothesis()
        .and_then(|hypothesis| hypothesis.bar_grid.first())
        .map(|bar| bar.downbeat_confidence);
    let degraded_policy = timing.effective_degraded_policy();
    let requires_manual_confirm = degraded_policy != TimingDegradedPolicy::Locked
        || !warning_codes.is_empty()
        || alternate_downbeat_count > 0
        || half_time_count > 0
        || double_time_count > 0;
    let result = if timing.bpm_estimate.is_none() || timing.primary_hypothesis().is_none() {
        SourceTimingCandidateConfidenceResult::Degraded
    } else if alternate_downbeat_count > 0
        || half_time_count > 0
        || double_time_count > 0
        || warning_codes.contains(&TimingWarningCode::AmbiguousDownbeat)
    {
        SourceTimingCandidateConfidenceResult::CandidateAmbiguous
    } else {
        SourceTimingCandidateConfidenceResult::CandidateCautious
    };

    SourceTimingCandidateConfidenceReport {
        schema: "riotbox.source_timing_candidate_confidence.v1",
        schema_version: 1,
        primary_bpm: timing.bpm_estimate,
        bpm_confidence: timing.bpm_confidence,
        timing_quality: timing.effective_timing_quality(),
        degraded_policy,
        hypothesis_count: timing.hypotheses.len(),
        alternate_downbeat_count,
        half_time_count,
        double_time_count,
        primary_downbeat_confidence,
        warning_codes,
        requires_manual_confirm,
        result,
    }
}

fn count_hypotheses(timing: &TimingModel, kind: TimingHypothesisKind) -> usize {
    timing
        .hypotheses
        .iter()
        .filter(|hypothesis| hypothesis.kind == kind)
        .count()
}
