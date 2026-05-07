#[cfg(test)]
mod timing_tests {
    use super::*;

    const TIMING_FIXTURE_CATALOG: &str =
        include_str!("../../tests/fixtures/source_timing/timing_fixture_catalog.json");

    #[test]
    fn source_timing_fixture_catalog_maps_to_core_timing_contract() {
        let catalog: serde_json::Value =
            serde_json::from_str(TIMING_FIXTURE_CATALOG).expect("parse timing fixture catalog");
        let cases = catalog
            .get("cases")
            .and_then(serde_json::Value::as_array)
            .expect("catalog cases");

        assert!(cases.len() >= 5);

        let clean_case = case_by_id(cases, "fx_timing_clean_128_4x4");
        let weak_case = case_by_id(cases, "fx_timing_weak_noisy_123");
        let ambiguous_case = case_by_id(cases, "fx_timing_halftime_140_ambiguous");

        let clean_timing = analyze_source_timing_seed(&analysis_seed_from_case(clean_case));
        let clean_evaluation =
            evaluate_timing_fixture_output(&clean_timing, &evaluation_target_from_case(clean_case));
        assert!(clean_evaluation.passed, "{clean_evaluation:?}");
        assert_eq!(clean_timing.effective_timing_quality(), TimingQuality::High);
        assert_eq!(
            clean_timing.effective_degraded_policy(),
            TimingDegradedPolicy::Locked
        );
        assert_eq!(clean_timing.beat_grid.len(), 32);
        assert_eq!(clean_timing.bar_grid.len(), 8);
        assert_eq!(clean_timing.phrase_grid.len(), 2);
        assert_eq!(clean_timing.hypotheses[0].anchors.len(), 1);

        let weak_timing = analyze_source_timing_seed(&analysis_seed_from_case(weak_case));
        let weak_evaluation =
            evaluate_timing_fixture_output(&weak_timing, &evaluation_target_from_case(weak_case));
        assert!(weak_evaluation.passed, "{weak_evaluation:?}");
        assert_eq!(weak_timing.effective_timing_quality(), TimingQuality::Low);
        assert_eq!(
            weak_timing.effective_degraded_policy(),
            TimingDegradedPolicy::ManualConfirm
        );
        assert!(weak_timing.phrase_grid.is_empty());
        assert!(
            weak_timing
                .warnings
                .iter()
                .any(|warning| warning.code == TimingWarningCode::LowTimingConfidence)
        );

        let ambiguous_timing = analyze_source_timing_seed(&analysis_seed_from_case(ambiguous_case));
        let ambiguous_evaluation = evaluate_timing_fixture_output(
            &ambiguous_timing,
            &evaluation_target_from_case(ambiguous_case),
        );
        assert!(ambiguous_evaluation.passed, "{ambiguous_evaluation:?}");
        assert_eq!(
            ambiguous_timing.primary_hypothesis().map(|hypothesis| {
                (
                    hypothesis.kind,
                    hypothesis.bpm,
                    hypothesis.warnings[0].code,
                )
            }),
            Some((
                TimingHypothesisKind::Primary,
                140.0,
                TimingWarningCode::HalfTimePossible,
            ))
        );
        assert_eq!(ambiguous_timing.hypotheses.len(), 2);
        assert_eq!(ambiguous_timing.hypotheses[1].kind, TimingHypothesisKind::HalfTime);
        assert_eq!(ambiguous_timing.hypotheses[1].bpm, 70.0);
        assert!(
            ambiguous_timing.hypotheses[1]
                .beat_grid
                .windows(2)
                .all(|window| window[0].time_seconds < window[1].time_seconds)
        );
    }

    #[test]
    fn source_timing_fixture_evaluator_rejects_out_of_tolerance_timing() {
        let catalog: serde_json::Value =
            serde_json::from_str(TIMING_FIXTURE_CATALOG).expect("parse timing fixture catalog");
        let cases = catalog
            .get("cases")
            .and_then(serde_json::Value::as_array)
            .expect("catalog cases");
        let clean_case = case_by_id(cases, "fx_timing_clean_128_4x4");
        let mut timing = analyze_source_timing_seed(&analysis_seed_from_case(clean_case));
        timing.bpm_estimate = Some(118.0);
        timing.beat_grid.truncate(4);

        let evaluation =
            evaluate_timing_fixture_output(&timing, &evaluation_target_from_case(clean_case));

        assert!(!evaluation.passed);
        assert!(evaluation
            .issues
            .contains(&TimingFixtureEvaluationIssue::BpmOutsideTolerance));
        assert!(evaluation
            .issues
            .contains(&TimingFixtureEvaluationIssue::BeatCountBelowMinimum));
    }

    fn case_by_id<'a>(
        cases: &'a [serde_json::Value],
        fixture_id: &str,
    ) -> &'a serde_json::Value {
        cases
            .iter()
            .find(|case| {
                case.get("fixture_id").and_then(serde_json::Value::as_str) == Some(fixture_id)
            })
            .expect("fixture case")
    }

    fn analysis_seed_from_case(case: &serde_json::Value) -> SourceTimingAnalysisSeed {
        let expected = case.get("expected").expect("expected timing contract");
        let meter = MeterHint {
            beats_per_bar: expected
                .pointer("/meter/beats_per_bar")
                .and_then(serde_json::Value::as_u64)
                .expect("beats_per_bar") as u8,
            beat_unit: expected
                .pointer("/meter/beat_unit")
                .and_then(serde_json::Value::as_u64)
                .expect("beat_unit") as u8,
        };
        let primary_bpm = expected
            .get("primary_bpm")
            .and_then(serde_json::Value::as_f64)
            .expect("primary_bpm") as f32;

        SourceTimingAnalysisSeed {
            fixture_id: case
                .get("fixture_id")
                .and_then(serde_json::Value::as_str)
                .expect("fixture_id")
                .into(),
            duration_seconds: case
                .get("duration_seconds")
                .and_then(serde_json::Value::as_f64)
                .expect("duration_seconds") as f32,
            primary_bpm,
            meter,
            quality: timing_quality_from_label(
                expected
                    .get("timing_quality")
                    .and_then(serde_json::Value::as_str),
            ),
            degraded_policy: degraded_policy_from_label(
                expected
                    .get("degraded_policy")
                    .and_then(serde_json::Value::as_str),
            ),
            beat_hit_tolerance_ms: expected
                .get("beat_hit_tolerance_ms")
                .and_then(serde_json::Value::as_f64)
                .expect("beat_hit_tolerance_ms") as f32,
            downbeat_tolerance_ms: expected
                .get("downbeat_tolerance_ms")
                .and_then(serde_json::Value::as_f64)
                .expect("downbeat_tolerance_ms") as f32,
            expected_beat_count_min: u32_from_expected(expected, "expected_beat_count_min"),
            expected_bar_count_min: u32_from_expected(expected, "expected_bar_count_min"),
            expected_phrase_count_min: u32_from_expected(expected, "expected_phrase_count_min"),
            confidence_floor: expected
                .get("confidence_floor")
                .and_then(serde_json::Value::as_f64)
                .expect("confidence_floor") as f32,
            warnings: timing_warning_codes_from_expected(expected),
            alternatives: timing_alternatives_from_expected(expected),
        }
    }

    fn evaluation_target_from_case(case: &serde_json::Value) -> TimingFixtureEvaluationTarget {
        let expected = case.get("expected").expect("expected timing contract");
        TimingFixtureEvaluationTarget {
            fixture_id: case
                .get("fixture_id")
                .and_then(serde_json::Value::as_str)
                .expect("fixture_id")
                .into(),
            primary_bpm: expected
                .get("primary_bpm")
                .and_then(serde_json::Value::as_f64)
                .expect("primary_bpm") as f32,
            bpm_tolerance: expected
                .get("bpm_tolerance")
                .and_then(serde_json::Value::as_f64)
                .expect("bpm_tolerance") as f32,
            expected_beat_count_min: u32_from_expected(expected, "expected_beat_count_min"),
            expected_bar_count_min: u32_from_expected(expected, "expected_bar_count_min"),
            expected_phrase_count_min: u32_from_expected(expected, "expected_phrase_count_min"),
            quality: timing_quality_from_label(
                expected
                    .get("timing_quality")
                    .and_then(serde_json::Value::as_str),
            ),
            degraded_policy: degraded_policy_from_label(
                expected
                    .get("degraded_policy")
                    .and_then(serde_json::Value::as_str),
            ),
            warnings: timing_warning_codes_from_expected(expected),
            alternative_kinds: timing_alternatives_from_expected(expected)
                .into_iter()
                .map(|alternative| alternative.kind)
                .collect(),
        }
    }

    fn u32_from_expected(expected: &serde_json::Value, field: &str) -> u32 {
        expected
            .get(field)
            .and_then(serde_json::Value::as_u64)
            .expect(field) as u32
    }

    fn timing_quality_from_label(label: Option<&str>) -> TimingQuality {
        match label {
            Some("low") => TimingQuality::Low,
            Some("medium") => TimingQuality::Medium,
            Some("high") => TimingQuality::High,
            _ => TimingQuality::Unknown,
        }
    }

    fn degraded_policy_from_label(label: Option<&str>) -> TimingDegradedPolicy {
        match label {
            Some("locked") => TimingDegradedPolicy::Locked,
            Some("cautious") => TimingDegradedPolicy::Cautious,
            Some("manual_confirm") => TimingDegradedPolicy::ManualConfirm,
            Some("fallback_grid") => TimingDegradedPolicy::FallbackGrid,
            Some("disabled") => TimingDegradedPolicy::Disabled,
            _ => TimingDegradedPolicy::Unknown,
        }
    }

    fn timing_hypothesis_kind_from_label(label: Option<&str>) -> TimingHypothesisKind {
        match label {
            Some("half_time") => TimingHypothesisKind::HalfTime,
            Some("double_time") => TimingHypothesisKind::DoubleTime,
            Some("alternate_downbeat") => TimingHypothesisKind::AlternateDownbeat,
            _ => TimingHypothesisKind::Ambiguous,
        }
    }

    fn timing_alternatives_from_expected(
        expected: &serde_json::Value,
    ) -> Vec<SourceTimingAlternativeSeed> {
        expected
            .get("alternatives")
            .and_then(serde_json::Value::as_array)
            .map(|alternatives| {
                alternatives
                    .iter()
                    .map(|alternative| SourceTimingAlternativeSeed {
                        kind: timing_hypothesis_kind_from_label(
                            alternative.get("kind").and_then(serde_json::Value::as_str),
                        ),
                        bpm: alternative
                            .get("bpm")
                            .and_then(serde_json::Value::as_f64)
                            .expect("alternative bpm") as f32,
                        confidence_floor: alternative
                            .get("confidence_floor")
                            .and_then(serde_json::Value::as_f64)
                            .expect("alternative confidence_floor") as f32,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn timing_warning_codes_from_expected(expected: &serde_json::Value) -> Vec<TimingWarningCode> {
        expected
            .get("warnings")
            .and_then(serde_json::Value::as_array)
            .map(|warnings| {
                warnings
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(timing_warning_code_from_label)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn timing_warning_code_from_label(label: &str) -> TimingWarningCode {
        match label {
            "weak_kick_anchor" => TimingWarningCode::WeakKickAnchor,
            "weak_backbeat_anchor" => TimingWarningCode::WeakBackbeatAnchor,
            "ambiguous_downbeat" => TimingWarningCode::AmbiguousDownbeat,
            "half_time_possible" => TimingWarningCode::HalfTimePossible,
            "double_time_possible" => TimingWarningCode::DoubleTimePossible,
            "drift_high" => TimingWarningCode::DriftHigh,
            "phrase_uncertain" => TimingWarningCode::PhraseUncertain,
            _ => TimingWarningCode::LowTimingConfidence,
        }
    }
}
