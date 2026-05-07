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
    let primary_drift = primary_drift_summary(timing);
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
        primary_drift_status: primary_drift.status,
        primary_drift_window_count: primary_drift.window_count,
        primary_drift_max_ms: primary_drift.max_ms,
        primary_drift_mean_abs_ms: primary_drift.mean_abs_ms,
        primary_drift_end_ms: primary_drift.end_ms,
        primary_drift_confidence: primary_drift.confidence,
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

struct PrimaryDriftSummary {
    status: SourceTimingCandidateDriftStatus,
    window_count: usize,
    max_ms: Option<f32>,
    mean_abs_ms: Option<f32>,
    end_ms: Option<f32>,
    confidence: Option<Confidence>,
}

fn primary_drift_summary(timing: &TimingModel) -> PrimaryDriftSummary {
    let Some(primary) = timing.primary_hypothesis() else {
        return PrimaryDriftSummary {
            status: SourceTimingCandidateDriftStatus::Unavailable,
            window_count: 0,
            max_ms: None,
            mean_abs_ms: None,
            end_ms: None,
            confidence: None,
        };
    };
    if primary.drift.is_empty() {
        return PrimaryDriftSummary {
            status: SourceTimingCandidateDriftStatus::NotEnoughMaterial,
            window_count: 0,
            max_ms: None,
            mean_abs_ms: None,
            end_ms: None,
            confidence: None,
        };
    }

    let max_ms = primary
        .drift
        .iter()
        .map(|drift| drift.max_drift_ms)
        .fold(0.0_f32, f32::max);
    let mean_abs_ms = primary
        .drift
        .iter()
        .map(|drift| drift.mean_abs_drift_ms)
        .sum::<f32>()
        / primary.drift.len() as f32;
    let end_ms = primary
        .drift
        .iter()
        .map(|drift| drift.end_drift_ms)
        .max_by(|left, right| left.abs().total_cmp(&right.abs()))
        .unwrap_or(0.0);
    let confidence = primary
        .drift
        .iter()
        .map(|drift| drift.confidence)
        .fold(1.0_f32, f32::min);

    PrimaryDriftSummary {
        status: if max_ms > 70.0 || end_ms.abs() > 70.0 {
            SourceTimingCandidateDriftStatus::High
        } else {
            SourceTimingCandidateDriftStatus::Stable
        },
        window_count: primary.drift.len(),
        max_ms: Some(max_ms),
        mean_abs_ms: Some(mean_abs_ms),
        end_ms: Some(end_ms),
        confidence: Some(confidence),
    }
}
