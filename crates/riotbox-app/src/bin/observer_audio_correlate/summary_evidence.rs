fn format_output_path_issues(summary: &CorrelationSummary) -> String {
    let failures = output_path_evidence_failures(summary);
    if failures.is_empty() {
        "none".to_string()
    } else {
        failures.join(", ")
    }
}

fn control_path_present(summary: &CorrelationSummary) -> bool {
    summary.first_commit != "none"
}

fn output_path_present(summary: &CorrelationSummary) -> bool {
    output_path_evidence_failures(summary).is_empty()
}

fn output_path_evidence_failures(summary: &CorrelationSummary) -> Vec<String> {
    let mut failures = Vec::new();

    if summary.manifest_result != "pass" {
        failures.push(format!("manifest_result={}", summary.manifest_result));
    }

    let metric_failures = if summary.pack_id == "w30-preview-smoke" {
        w30_source_preview_metric_failures(summary)
    } else if summary.pack_id == "lane-recipe-listening-pack" {
        lane_recipe_metric_failures(&summary.lane_recipe_cases, STRICT_OUTPUT_METRIC_FLOOR)
    } else {
        feral_grid_metric_failures(summary)
    };
    failures.extend(metric_failures);

    failures
}

fn feral_grid_metric_failures(summary: &CorrelationSummary) -> Vec<String> {
    let mut failures = metric_failures([
        ("full_mix_rms", summary.full_mix_rms),
        ("full_mix_low_band_rms", summary.full_mix_low_band_rms),
    ]);
    if summary.source_timing_malformed {
        failures.push("source_timing=malformed".to_string());
    }
    failures.extend(source_timing_alignment_failures(summary));
    failures.extend(source_timing_anchor_alignment_failures(summary));
    failures.extend(source_timing_groove_alignment_failures(summary));
    failures.extend(required_source_grid_alignment_failures(
        "source_grid_output_drift",
        &summary.source_grid_output_drift,
        summary.source_grid_output_drift_malformed,
    ));
    failures.extend(required_source_grid_alignment_failures(
        "tr909_source_grid_alignment",
        &summary.tr909_source_grid_alignment,
        summary.tr909_source_grid_alignment_malformed,
    ));
    failures.extend(required_source_grid_alignment_failures(
        "w30_source_grid_alignment",
        &summary.w30_source_grid_alignment,
        summary.w30_source_grid_alignment_malformed,
    ));
    failures.extend(w30_source_loop_closure_failures(summary));
    failures
}

fn source_timing_alignment_failures(summary: &CorrelationSummary) -> Vec<String> {
    let mut failures = summary
        .source_timing_alignment
        .as_ref()
        .map(|alignment| alignment.issues.clone())
        .unwrap_or_default();
    failures.extend(source_timing_policy_failures(summary));
    failures
}

fn source_timing_anchor_alignment_failures(summary: &CorrelationSummary) -> Vec<String> {
    summary
        .source_timing_anchor_alignment
        .as_ref()
        .map(|alignment| alignment.issues.clone())
        .unwrap_or_default()
}

fn source_timing_groove_alignment_failures(summary: &CorrelationSummary) -> Vec<String> {
    summary
        .source_timing_groove_alignment
        .as_ref()
        .map(|alignment| alignment.issues.clone())
        .unwrap_or_default()
}

fn required_source_grid_alignment_failures(
    metric_key: &str,
    drift: &Option<SourceGridOutputDriftEvidence>,
    malformed: bool,
) -> Vec<String> {
    if malformed {
        return vec![format!("{metric_key}=malformed")];
    }

    if drift.is_none() {
        return vec![format!("{metric_key}=missing")];
    }

    source_grid_alignment_failures(metric_key, drift, false)
}

fn source_grid_alignment_failures(
    metric_key: &str,
    drift: &Option<SourceGridOutputDriftEvidence>,
    malformed: bool,
) -> Vec<String> {
    if malformed {
        return vec![format!("{metric_key}=malformed")];
    }

    let Some(drift) = drift else {
        return Vec::new();
    };

    let mut failures = Vec::new();
    if drift.hit_ratio < SOURCE_GRID_OUTPUT_MIN_HIT_RATIO {
        failures.push(format!(
            "{metric_key}.hit_ratio={:.6} < floor {:.6}",
            drift.hit_ratio, SOURCE_GRID_OUTPUT_MIN_HIT_RATIO
        ));
    }
    if drift.max_peak_offset_ms > drift.max_allowed_peak_offset_ms {
        failures.push(format!(
            "{metric_key}.max_peak_offset_ms={:.6} > allowed {:.6}",
            drift.max_peak_offset_ms, drift.max_allowed_peak_offset_ms
        ));
    }
    failures
}

fn w30_source_preview_metric_failures(summary: &CorrelationSummary) -> Vec<String> {
    metric_failures([
        ("w30_candidate_rms", summary.w30_candidate_rms),
        (
            "w30_candidate_active_sample_ratio",
            summary.w30_candidate_active_sample_ratio,
        ),
        ("w30_rms_delta", summary.w30_rms_delta),
    ])
}

fn w30_source_loop_closure_failures(summary: &CorrelationSummary) -> Vec<String> {
    if summary.w30_source_loop_closure_malformed {
        return vec!["w30_source_loop_closure=malformed".to_string()];
    }

    let Some(proof) = &summary.w30_source_loop_closure else {
        return vec!["w30_source_loop_closure=missing".to_string()];
    };

    let mut failures = Vec::new();
    if !proof.passed {
        failures.push("w30_source_loop_closure.passed=false".to_string());
    }
    if proof.preview_rms <= STRICT_OUTPUT_METRIC_FLOOR {
        failures.push(format!(
            "w30_source_loop_closure.preview_rms={:.6} <= floor {:.6}",
            proof.preview_rms, STRICT_OUTPUT_METRIC_FLOOR
        ));
    }
    if !proof.source_contains_selection {
        failures.push("w30_source_loop_closure.source_contains_selection=false".to_string());
    }
    if proof.edge_delta_abs > proof.max_allowed_edge_delta_abs {
        failures.push(format!(
            "w30_source_loop_closure.edge_delta_abs={:.6} > allowed {:.6}",
            proof.edge_delta_abs, proof.max_allowed_edge_delta_abs
        ));
    }
    if proof.edge_abs_max > proof.max_allowed_edge_abs {
        failures.push(format!(
            "w30_source_loop_closure.edge_abs_max={:.6} > allowed {:.6}",
            proof.edge_abs_max, proof.max_allowed_edge_abs
        ));
    }
    failures
}

fn metric_failures(metrics: impl IntoIterator<Item = (&'static str, Option<f64>)>) -> Vec<String> {
    let mut failures = Vec::new();
    for (name, metric) in metrics {
        if let Some(failure) = output_metric_failure(name, metric) {
            failures.push(failure);
        }
    }

    failures
}

fn output_metric_failure(name: &str, metric: Option<f64>) -> Option<String> {
    match metric {
        Some(_) if metric_is_noncollapsed(metric) => None,
        Some(value) => Some(format!(
            "{name}={value:.6} <= floor {STRICT_OUTPUT_METRIC_FLOOR:.6}"
        )),
        None => Some(format!("{name}=missing")),
    }
}

fn metric_is_noncollapsed(metric: Option<f64>) -> bool {
    metric.is_some_and(|value| value > STRICT_OUTPUT_METRIC_FLOOR)
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn validate_required_evidence(summary: &CorrelationSummary) -> Result<(), io::Error> {
    if !control_path_present(summary) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "observer/audio correlation is missing committed control-path evidence",
        ));
    }
    if summary.observer_source_timing_malformed {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "observer/audio correlation has malformed observer source timing evidence",
        ));
    }

    let output_failures = output_path_evidence_failures(summary);
    if !output_failures.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "observer/audio correlation is missing passing output-path manifest evidence: {}",
                output_failures.join(", ")
            ),
        ));
    }

    Ok(())
}
