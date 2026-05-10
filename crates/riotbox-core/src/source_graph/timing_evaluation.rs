#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

pub fn source_timing_analysis_seed_from_fixture_case(
    case: &serde_json::Value,
) -> Result<SourceTimingAnalysisSeed, String> {
    let expected = fixture_expected(case)?;
    let fixture_id = fixture_id_from_case(case)?;
    let primary_bpm = required_f32(expected, "primary_bpm")?;
    Ok(SourceTimingAnalysisSeed {
        fixture_id: fixture_id.clone(),
        duration_seconds: required_f32(case, "duration_seconds")?,
        primary_bpm,
        meter: MeterHint {
            beats_per_bar: required_u64_at(expected, "/meter/beats_per_bar")? as u8,
            beat_unit: required_u64_at(expected, "/meter/beat_unit")? as u8,
        },
        quality: timing_quality_from_label(&fixture_id, required_str(expected, "timing_quality")?)?,
        degraded_policy: timing_degraded_policy_from_label(
            &fixture_id,
            required_str(expected, "degraded_policy")?,
        )?,
        beat_hit_tolerance_ms: required_f32(expected, "beat_hit_tolerance_ms")?,
        downbeat_tolerance_ms: required_f32(expected, "downbeat_tolerance_ms")?,
        expected_beat_count_min: required_u32(expected, "expected_beat_count_min")?,
        expected_bar_count_min: required_u32(expected, "expected_bar_count_min")?,
        expected_phrase_count_min: required_u32(expected, "expected_phrase_count_min")?,
        confidence_floor: required_f32(expected, "confidence_floor")?,
        warnings: timing_warning_codes_from_expected(&fixture_id, expected)?,
        alternatives: source_timing_alternatives_from_expected(&fixture_id, expected)?,
        drift: source_timing_drift_from_expected(&fixture_id, expected)?,
    })
}

pub fn timing_fixture_evaluation_target_from_fixture_case(
    case: &serde_json::Value,
) -> Result<TimingFixtureEvaluationTarget, String> {
    let expected = fixture_expected(case)?;
    let fixture_id = fixture_id_from_case(case)?;
    Ok(TimingFixtureEvaluationTarget {
        fixture_id: fixture_id.clone(),
        primary_bpm: required_f32(expected, "primary_bpm")?,
        bpm_tolerance: required_f32(expected, "bpm_tolerance")?,
        beat_hit_tolerance_ms: required_f32(expected, "beat_hit_tolerance_ms")?,
        downbeat_tolerance_ms: required_f32(expected, "downbeat_tolerance_ms")?,
        expected_beat_count_min: required_u32(expected, "expected_beat_count_min")?,
        expected_bar_count_min: required_u32(expected, "expected_bar_count_min")?,
        expected_phrase_count_min: required_u32(expected, "expected_phrase_count_min")?,
        confidence_floor: required_f32(expected, "confidence_floor")?,
        quality: timing_quality_from_label(&fixture_id, required_str(expected, "timing_quality")?)?,
        degraded_policy: timing_degraded_policy_from_label(
            &fixture_id,
            required_str(expected, "degraded_policy")?,
        )?,
        warnings: timing_warning_codes_from_expected(&fixture_id, expected)?,
        alternative_kinds: source_timing_alternatives_from_expected(&fixture_id, expected)?
            .into_iter()
            .map(|alternative| alternative.kind)
            .collect(),
    })
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

fn fixture_expected(case: &serde_json::Value) -> Result<&serde_json::Value, String> {
    case.get("expected")
        .ok_or_else(|| "fixture case missing expected timing contract".into())
}

fn fixture_id_from_case(case: &serde_json::Value) -> Result<String, String> {
    required_str(case, "fixture_id").map(Into::into)
}

fn required_str<'a>(value: &'a serde_json::Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("{field} must be a string"))
}

fn required_f32(value: &serde_json::Value, field: &str) -> Result<f32, String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_f64)
        .map(|number| number as f32)
        .ok_or_else(|| format!("{field} must be a number"))
}

fn required_u32(value: &serde_json::Value, field: &str) -> Result<u32, String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_u64)
        .map(|number| number as u32)
        .ok_or_else(|| format!("{field} must be an unsigned integer"))
}

fn required_u64_at(value: &serde_json::Value, pointer: &str) -> Result<u64, String> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| format!("{pointer} must be an unsigned integer"))
}

fn timing_quality_from_label(fixture_id: &str, label: &str) -> Result<TimingQuality, String> {
    match label {
        "low" => Ok(TimingQuality::Low),
        "medium" => Ok(TimingQuality::Medium),
        "high" => Ok(TimingQuality::High),
        "unknown" => Ok(TimingQuality::Unknown),
        _ => Err(format!("{fixture_id}: unknown timing_quality {label:?}")),
    }
}

fn timing_degraded_policy_from_label(
    fixture_id: &str,
    label: &str,
) -> Result<TimingDegradedPolicy, String> {
    match label {
        "locked" => Ok(TimingDegradedPolicy::Locked),
        "cautious" => Ok(TimingDegradedPolicy::Cautious),
        "manual_confirm" => Ok(TimingDegradedPolicy::ManualConfirm),
        "fallback_grid" => Ok(TimingDegradedPolicy::FallbackGrid),
        "disabled" => Ok(TimingDegradedPolicy::Disabled),
        "unknown" => Ok(TimingDegradedPolicy::Unknown),
        _ => Err(format!("{fixture_id}: unknown degraded_policy {label:?}")),
    }
}

fn timing_hypothesis_kind_from_label(fixture_id: &str, label: &str) -> Result<TimingHypothesisKind, String> {
    match label {
        "half_time" => Ok(TimingHypothesisKind::HalfTime),
        "double_time" => Ok(TimingHypothesisKind::DoubleTime),
        "alternate_downbeat" => Ok(TimingHypothesisKind::AlternateDownbeat),
        "ambiguous" => Ok(TimingHypothesisKind::Ambiguous),
        _ => Err(format!("{fixture_id}: unknown alternative kind {label:?}")),
    }
}

fn source_timing_alternatives_from_expected(
    fixture_id: &str,
    expected: &serde_json::Value,
) -> Result<Vec<SourceTimingAlternativeSeed>, String> {
    let Some(alternatives) = expected.get("alternatives").and_then(serde_json::Value::as_array)
    else {
        return Ok(Vec::new());
    };

    alternatives
        .iter()
        .map(|alternative| {
            Ok(SourceTimingAlternativeSeed {
                kind: timing_hypothesis_kind_from_label(
                    fixture_id,
                    required_str(alternative, "kind")?,
                )?,
                bpm: required_f32(alternative, "bpm")?,
                confidence_floor: required_f32(alternative, "confidence_floor")?,
            })
        })
        .collect()
}

fn source_timing_drift_from_expected(
    fixture_id: &str,
    expected: &serde_json::Value,
) -> Result<Option<SourceTimingDriftSeed>, String> {
    let Some(drift) = expected.get("drift") else {
        return Ok(None);
    };
    if !drift.is_object() {
        return Err(format!("{fixture_id}: drift must be an object"));
    }

    Ok(Some(SourceTimingDriftSeed {
        window_bars: required_u32(drift, "window_bars")?,
        max_drift_ms: required_f32(drift, "max_drift_ms")?,
        mean_abs_drift_ms: required_f32(drift, "mean_abs_drift_ms")?,
        end_drift_ms: required_f32(drift, "end_drift_ms")?,
    }))
}

fn timing_warning_codes_from_expected(
    fixture_id: &str,
    expected: &serde_json::Value,
) -> Result<Vec<TimingWarningCode>, String> {
    let Some(warnings) = expected.get("warnings") else {
        return Ok(Vec::new());
    };
    let warnings = warnings
        .as_array()
        .ok_or_else(|| format!("{fixture_id}: warnings must be an array"))?;
    warnings
        .iter()
        .enumerate()
        .map(|(index, warning)| {
            let label = warning
                .as_str()
                .ok_or_else(|| format!("{fixture_id}: warning {index} must be a string"))?;
            timing_warning_code_from_label(fixture_id, label)
        })
        .collect()
}

fn timing_warning_code_from_label(
    fixture_id: &str,
    label: &str,
) -> Result<TimingWarningCode, String> {
    match label {
        "weak_kick_anchor" => Ok(TimingWarningCode::WeakKickAnchor),
        "weak_backbeat_anchor" => Ok(TimingWarningCode::WeakBackbeatAnchor),
        "ambiguous_downbeat" => Ok(TimingWarningCode::AmbiguousDownbeat),
        "half_time_possible" => Ok(TimingWarningCode::HalfTimePossible),
        "double_time_possible" => Ok(TimingWarningCode::DoubleTimePossible),
        "drift_high" => Ok(TimingWarningCode::DriftHigh),
        "phrase_uncertain" => Ok(TimingWarningCode::PhraseUncertain),
        "low_timing_confidence" => Ok(TimingWarningCode::LowTimingConfidence),
        _ => Err(format!("{fixture_id}: unknown warning label {label:?}")),
    }
}
