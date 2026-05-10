#[derive(Clone, Debug, PartialEq)]
pub struct TimingFixtureEvaluationTarget {
    pub fixture_id: String,
    pub primary_bpm: f32,
    pub bpm_tolerance: f32,
    pub beat_hit_tolerance_ms: f32,
    pub downbeat_tolerance_ms: f32,
    pub expected_beat_count_min: u32,
    pub expected_bar_count_min: u32,
    pub expected_phrase_count_min: u32,
    pub confidence_floor: Confidence,
    pub quality: TimingQuality,
    pub degraded_policy: TimingDegradedPolicy,
    pub warnings: Vec<TimingWarningCode>,
    pub alternative_kinds: Vec<TimingHypothesisKind>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimingFixtureEvaluation {
    pub fixture_id: String,
    pub passed: bool,
    pub bpm_error: f32,
    pub beat_count: usize,
    pub bar_count: usize,
    pub phrase_count: usize,
    pub primary_confidence: Option<Confidence>,
    pub primary_max_mean_abs_drift_ms: Option<f32>,
    pub primary_max_drift_ms: Option<f32>,
    pub issues: Vec<TimingFixtureEvaluationIssue>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TimingFixtureEvaluationIssue {
    MissingBpmEstimate,
    BpmOutsideTolerance,
    BeatCountBelowMinimum,
    BarCountBelowMinimum,
    PhraseCountBelowMinimum,
    QualityMismatch,
    DegradedPolicyMismatch,
    PrimaryConfidenceBelowFloor,
    MissingTimingDrift,
    BeatDriftOutsideTolerance,
    DownbeatDriftOutsideTolerance,
    MissingWarning(TimingWarningCode),
    MissingAlternative(TimingHypothesisKind),
    MissingPrimaryHypothesis,
}

#[must_use]
pub fn evaluate_timing_fixture_output(
    timing: &TimingModel,
    target: &TimingFixtureEvaluationTarget,
) -> TimingFixtureEvaluation {
    let mut issues = Vec::new();
    let primary_hypothesis = timing.primary_hypothesis();
    let primary_confidence = primary_hypothesis.map(|primary| primary.confidence);
    let primary_max_mean_abs_drift_ms = primary_hypothesis
        .and_then(|primary| max_drift_value(primary.drift.iter().map(|drift| drift.mean_abs_drift_ms)));
    let primary_max_drift_ms = primary_hypothesis
        .and_then(|primary| max_drift_value(primary.drift.iter().map(|drift| drift.max_drift_ms)));
    let bpm_error = match timing.bpm_estimate {
        Some(bpm) => (bpm - target.primary_bpm).abs(),
        None => {
            issues.push(TimingFixtureEvaluationIssue::MissingBpmEstimate);
            f32::INFINITY
        }
    };

    if bpm_error > target.bpm_tolerance {
        issues.push(TimingFixtureEvaluationIssue::BpmOutsideTolerance);
    }
    if timing.beat_grid.len() < target.expected_beat_count_min as usize {
        issues.push(TimingFixtureEvaluationIssue::BeatCountBelowMinimum);
    }
    if timing.bar_grid.len() < target.expected_bar_count_min as usize {
        issues.push(TimingFixtureEvaluationIssue::BarCountBelowMinimum);
    }
    if timing.phrase_grid.len() < target.expected_phrase_count_min as usize {
        issues.push(TimingFixtureEvaluationIssue::PhraseCountBelowMinimum);
    }
    if timing.effective_timing_quality() != target.quality {
        issues.push(TimingFixtureEvaluationIssue::QualityMismatch);
    }
    if timing.effective_degraded_policy() != target.degraded_policy {
        issues.push(TimingFixtureEvaluationIssue::DegradedPolicyMismatch);
    }
    match primary_hypothesis {
        Some(primary) => {
            if primary.confidence < target.confidence_floor {
                issues.push(TimingFixtureEvaluationIssue::PrimaryConfidenceBelowFloor);
            }
            if target.expected_bar_count_min > 0 && primary.drift.is_empty() {
                issues.push(TimingFixtureEvaluationIssue::MissingTimingDrift);
            }
            if primary
                .drift
                .iter()
                .any(|drift| drift.mean_abs_drift_ms > target.beat_hit_tolerance_ms)
            {
                issues.push(TimingFixtureEvaluationIssue::BeatDriftOutsideTolerance);
            }
            if primary
                .drift
                .iter()
                .any(|drift| drift.max_drift_ms > target.downbeat_tolerance_ms)
            {
                issues.push(TimingFixtureEvaluationIssue::DownbeatDriftOutsideTolerance);
            }
        }
        None => issues.push(TimingFixtureEvaluationIssue::MissingPrimaryHypothesis),
    }

    for expected_warning in &target.warnings {
        if !timing
            .warnings
            .iter()
            .any(|warning| warning.code == *expected_warning)
        {
            issues.push(TimingFixtureEvaluationIssue::MissingWarning(
                *expected_warning,
            ));
        }
    }

    for alternative_kind in &target.alternative_kinds {
        if !timing
            .hypotheses
            .iter()
            .any(|hypothesis| hypothesis.kind == *alternative_kind)
        {
            issues.push(TimingFixtureEvaluationIssue::MissingAlternative(
                *alternative_kind,
            ));
        }
    }

    TimingFixtureEvaluation {
        fixture_id: target.fixture_id.clone(),
        passed: issues.is_empty(),
        bpm_error,
        beat_count: timing.beat_grid.len(),
        bar_count: timing.bar_grid.len(),
        phrase_count: timing.phrase_grid.len(),
        primary_confidence,
        primary_max_mean_abs_drift_ms,
        primary_max_drift_ms,
        issues,
    }
}

fn max_drift_value(values: impl Iterator<Item = f32>) -> Option<f32> {
    values.reduce(f32::max)
}
