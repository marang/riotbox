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

        let clean_timing = timing_model_from_case(clean_case);
        assert_eq!(clean_timing.effective_timing_quality(), TimingQuality::High);
        assert_eq!(
            clean_timing.effective_degraded_policy(),
            TimingDegradedPolicy::Locked
        );

        let weak_timing = timing_model_from_case(weak_case);
        assert_eq!(weak_timing.effective_timing_quality(), TimingQuality::Low);
        assert_eq!(
            weak_timing.effective_degraded_policy(),
            TimingDegradedPolicy::ManualConfirm
        );

        let ambiguous_timing = timing_model_from_case(ambiguous_case);
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

    fn timing_model_from_case(case: &serde_json::Value) -> TimingModel {
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
        let quality = timing_quality_from_label(
            expected
                .get("timing_quality")
                .and_then(serde_json::Value::as_str),
        );
        let policy = degraded_policy_from_label(
            expected
                .get("degraded_policy")
                .and_then(serde_json::Value::as_str),
        );
        let warnings = timing_warnings_from_expected(expected);
        let mut hypotheses = vec![TimingHypothesis {
            hypothesis_id: "primary".into(),
            kind: TimingHypothesisKind::Primary,
            bpm: primary_bpm,
            meter,
            confidence: expected
                .get("confidence_floor")
                .and_then(serde_json::Value::as_f64)
                .expect("confidence_floor") as f32,
            score: 1.0,
            beat_grid: Vec::new(),
            bar_grid: Vec::new(),
            phrase_grid: Vec::new(),
            anchors: Vec::new(),
            drift: Vec::new(),
            groove: Vec::new(),
            quality,
            warnings: warnings.clone(),
            provenance: vec!["fixture-catalog".into()],
        }];

        if let Some(alternatives) = expected
            .get("alternatives")
            .and_then(serde_json::Value::as_array)
        {
            for alternative in alternatives {
                hypotheses.push(TimingHypothesis {
                    hypothesis_id: alternative
                        .get("kind")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("ambiguous")
                        .into(),
                    kind: timing_hypothesis_kind_from_label(
                        alternative.get("kind").and_then(serde_json::Value::as_str),
                    ),
                    bpm: alternative
                        .get("bpm")
                        .and_then(serde_json::Value::as_f64)
                        .expect("alternative bpm") as f32,
                    meter,
                    confidence: alternative
                        .get("confidence_floor")
                        .and_then(serde_json::Value::as_f64)
                        .expect("alternative confidence_floor") as f32,
                    score: 0.5,
                    beat_grid: Vec::new(),
                    bar_grid: Vec::new(),
                    phrase_grid: Vec::new(),
                    anchors: Vec::new(),
                    drift: Vec::new(),
                    groove: Vec::new(),
                    quality: TimingQuality::Low,
                    warnings: warnings.clone(),
                    provenance: vec!["fixture-catalog-alternative".into()],
                });
            }
        }

        TimingModel {
            bpm_estimate: Some(primary_bpm),
            bpm_confidence: hypotheses[0].confidence,
            meter_hint: Some(meter),
            beat_grid: Vec::new(),
            bar_grid: Vec::new(),
            phrase_grid: Vec::new(),
            hypotheses,
            primary_hypothesis_id: Some("primary".into()),
            quality,
            warnings,
            degraded_policy: policy,
        }
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

    fn timing_warnings_from_expected(expected: &serde_json::Value) -> Vec<TimingWarning> {
        expected
            .get("warnings")
            .and_then(serde_json::Value::as_array)
            .map(|warnings| {
                warnings
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(|warning| TimingWarning {
                        code: timing_warning_code_from_label(warning),
                        message: warning.replace('_', " "),
                    })
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
