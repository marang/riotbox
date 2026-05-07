#[derive(Clone, Debug, PartialEq)]
pub struct TimingFixtureEvaluationTarget {
    pub fixture_id: String,
    pub primary_bpm: f32,
    pub bpm_tolerance: f32,
    pub expected_beat_count_min: u32,
    pub expected_bar_count_min: u32,
    pub expected_phrase_count_min: u32,
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
    if timing.primary_hypothesis().is_none() {
        issues.push(TimingFixtureEvaluationIssue::MissingPrimaryHypothesis);
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
        issues,
    }
}
