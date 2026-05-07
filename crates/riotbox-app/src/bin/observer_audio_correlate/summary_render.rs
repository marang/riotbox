fn render_markdown(summary: &CorrelationSummary) -> String {
    format!(
        "# Observer / Audio QA Correlation Summary\n\n\
         ## Control Path\n\n\
         - Observer schema: `{}`\n\
         - Launch mode: `{}`\n\
         - Audio runtime status: `{}`\n\
         - Key outcomes: `{}`\n\
         - First commit: `{}`\n\n\
         - Commit count: `{}`\n\
         - Commit boundaries: `{}`\n\n\
         ## Output Path\n\n\
         - Pack id: `{}`\n\
         - Manifest result: `{}`\n\
         - Artifact count: `{}`\n\
         - Full mix RMS: `{}`\n\
         - Full mix low-band RMS: `{}`\n\n\
         - W-30 candidate RMS: `{}`\n\
         - W-30 candidate active-sample ratio: `{}`\n\
         - W-30 RMS delta: `{}`\n\n\
         ## Correlation Verdict\n\n\
         - Control path present: `{}`\n\
         - Output path present: `{}`\n\
         - Output path issues: `{}`\n\
         - Needs human listening: `yes`\n",
        summary.observer_schema,
        summary.launch_mode,
        summary.audio_runtime_status,
        if summary.key_outcomes.is_empty() {
            "none".to_string()
        } else {
            summary.key_outcomes.join(", ")
        },
        summary.first_commit,
        summary.commit_count,
        if summary.commit_boundaries.is_empty() {
            "none".to_string()
        } else {
            summary.commit_boundaries.join(", ")
        },
        summary.pack_id,
        summary.manifest_result,
        summary.artifact_count,
        format_optional_f64(summary.full_mix_rms),
        format_optional_f64(summary.full_mix_low_band_rms),
        format_optional_f64(summary.w30_candidate_rms),
        format_optional_f64(summary.w30_candidate_active_sample_ratio),
        format_optional_f64(summary.w30_rms_delta),
        yes_no(control_path_present(summary)),
        yes_no(output_path_present(summary)),
        format_output_path_issues(summary)
    )
}

fn render_json(summary: &CorrelationSummary) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": SUMMARY_SCHEMA,
        "schema_version": SUMMARY_SCHEMA_VERSION,
        "control_path": {
            "present": control_path_present(summary),
            "observer_schema": &summary.observer_schema,
            "launch_mode": &summary.launch_mode,
            "audio_runtime_status": &summary.audio_runtime_status,
            "key_outcomes": &summary.key_outcomes,
            "first_commit": &summary.first_commit,
            "commit_count": summary.commit_count,
            "commit_boundaries": &summary.commit_boundaries,
        },
        "output_path": {
            "present": output_path_present(summary),
            "issues": output_path_evidence_failures(summary),
            "pack_id": &summary.pack_id,
            "manifest_result": &summary.manifest_result,
            "artifact_count": summary.artifact_count,
            "metrics": {
                "full_mix_rms": summary.full_mix_rms,
                "full_mix_low_band_rms": summary.full_mix_low_band_rms,
                "mc202_question_answer_delta_rms": summary.mc202_question_answer_delta_rms,
                "w30_candidate_rms": summary.w30_candidate_rms,
                "w30_candidate_active_sample_ratio": summary.w30_candidate_active_sample_ratio,
                "w30_rms_delta": summary.w30_rms_delta,
            },
        },
        "needs_human_listening": true,
    }))
    .map(|json| json + "\n")
}

fn format_optional_f64(value: Option<f64>) -> String {
    value.map_or_else(|| "unknown".to_string(), |value| format!("{value:.6}"))
}

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
    metric_failures([
        ("full_mix_rms", summary.full_mix_rms),
        ("full_mix_low_band_rms", summary.full_mix_low_band_rms),
    ])
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
