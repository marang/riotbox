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
         - Observer source timing: `{}`\n\n\
         ## Output Path\n\n\
         - Pack id: `{}`\n\
         - Manifest result: `{}`\n\
         - Artifact count: `{}`\n\
         - Grid BPM source: `{}`\n\
         - Grid BPM decision reason: `{}`\n\
         - Source timing BPM delta: `{}`\n\
         - Full mix RMS: `{}`\n\
         - Full mix low-band RMS: `{}`\n\n\
         - Source timing readiness: `{}`\n\
         - Source timing downbeat: `{}`\n\
         - Source timing phrase: `{}`\n\n\
         - Source timing alignment: `{}`\n\n\
         - Source-grid output hit ratio: `{}`\n\
         - Source-grid output max peak offset: `{}`\n\
         - Source-grid output max allowed offset: `{}`\n\n\
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
        format_observer_source_timing(summary),
        summary.pack_id,
        summary.manifest_result,
        summary.artifact_count,
        summary.grid_bpm_source,
        summary.grid_bpm_decision_reason,
        format_optional_f64(summary.source_timing_bpm_delta),
        format_optional_f64(summary.full_mix_rms),
        format_optional_f64(summary.full_mix_low_band_rms),
        format_source_timing_readiness(summary),
        format_source_timing_downbeat(summary),
        format_source_timing_phrase(summary),
        format_source_timing_alignment(summary),
        format_source_grid_hit_ratio(summary),
        format_source_grid_max_peak_offset(summary),
        format_source_grid_max_allowed_offset(summary),
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
            "observer_source_timing": summary.observer_source_timing.as_ref().map(|timing| serde_json::json!({
                "source_id": &timing.source_id,
                "cue": observer_source_timing_cue(timing),
                "bpm_estimate": timing.bpm_estimate,
                "bpm_confidence": timing.bpm_confidence,
                "quality": &timing.quality,
                "degraded_policy": &timing.degraded_policy,
                "primary_hypothesis_id": &timing.primary_hypothesis_id,
                "hypothesis_count": timing.hypothesis_count,
                "primary_warning_code": &timing.primary_warning_code,
                "warning_codes": &timing.warning_codes,
            })),
        },
        "output_path": {
            "present": output_path_present(summary),
            "issues": output_path_evidence_failures(summary),
            "pack_id": &summary.pack_id,
            "manifest_result": &summary.manifest_result,
            "artifact_count": summary.artifact_count,
            "grid_bpm_source": &summary.grid_bpm_source,
            "grid_bpm_decision_reason": &summary.grid_bpm_decision_reason,
            "source_timing_bpm_delta": summary.source_timing_bpm_delta,
            "source_timing": summary.source_timing.as_ref().map(|timing| serde_json::json!({
                "source_id": &timing.source_id,
                "cue": source_timing_readiness_cue(timing),
                "policy_profile": &timing.policy_profile,
                "readiness": &timing.readiness,
                "requires_manual_confirm": timing.requires_manual_confirm,
                "primary_bpm": timing.primary_bpm,
                "bpm_agrees_with_grid": timing.bpm_agrees_with_grid,
                "beat_status": &timing.beat_status,
                "downbeat_status": &timing.downbeat_status,
                "primary_downbeat_offset_beats": timing.primary_downbeat_offset_beats,
                "confidence_result": &timing.confidence_result,
                "drift_status": &timing.drift_status,
                "phrase_status": &timing.phrase_status,
                "alternate_evidence_count": timing.alternate_evidence_count,
                "warning_codes": &timing.warning_codes,
            })),
            "source_timing_alignment": summary.source_timing_alignment.as_ref().map(|alignment| serde_json::json!({
                "status": &alignment.status,
                "bpm_delta": alignment.bpm_delta,
                "bpm_tolerance": alignment.bpm_tolerance,
                "warning_overlap": &alignment.warning_overlap,
                "issues": &alignment.issues,
            })),
            "metrics": {
                "full_mix_rms": summary.full_mix_rms,
                "full_mix_low_band_rms": summary.full_mix_low_band_rms,
                "mc202_question_answer_delta_rms": summary.mc202_question_answer_delta_rms,
                "source_grid_output_drift": summary.source_grid_output_drift.as_ref().map(|drift| serde_json::json!({
                    "hit_ratio": drift.hit_ratio,
                    "max_peak_offset_ms": drift.max_peak_offset_ms,
                    "max_allowed_peak_offset_ms": drift.max_allowed_peak_offset_ms,
                })),
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

fn format_observer_source_timing(summary: &CorrelationSummary) -> String {
    if summary.observer_source_timing_malformed {
        return "malformed".to_string();
    }
    summary.observer_source_timing.as_ref().map_or_else(
        || "unknown".to_string(),
        |timing| {
            format!(
                "{} cue={} quality={} policy={} bpm={} confidence={:.3} warning={}",
                timing.source_id,
                observer_source_timing_cue(timing),
                timing.quality,
                timing.degraded_policy,
                format_optional_f64(timing.bpm_estimate),
                timing.bpm_confidence,
                timing
                    .primary_warning_code
                    .as_deref()
                    .unwrap_or("none")
            )
        },
    )
}

fn format_source_timing_readiness(summary: &CorrelationSummary) -> String {
    if summary.source_timing_malformed {
        return "malformed".to_string();
    }
    summary.source_timing.as_ref().map_or_else(
        || "unknown".to_string(),
        |timing| {
            format!(
                "{} readiness={} manual_confirm={}",
                source_timing_readiness_cue(timing),
                timing.readiness,
                yes_no(timing.requires_manual_confirm)
            )
        },
    )
}

fn observer_source_timing_cue(timing: &ObserverSourceTimingReadiness) -> &'static str {
    riotbox_app::source_timing_cues::source_timing_policy_cue_label(&timing.degraded_policy)
}

fn source_timing_readiness_cue(timing: &SourceTimingEvidence) -> &'static str {
    riotbox_app::source_timing_cues::source_timing_readiness_cue_label(
        &timing.readiness,
        timing.requires_manual_confirm,
    )
}

fn format_source_timing_downbeat(summary: &CorrelationSummary) -> String {
    if summary.source_timing_malformed {
        return "malformed".to_string();
    }
    summary.source_timing.as_ref().map_or_else(
        || "unknown".to_string(),
        |timing| {
            format!(
                "{} offset={}",
                timing.downbeat_status,
                timing
                    .primary_downbeat_offset_beats
                    .map_or_else(|| "unknown".to_string(), |value| value.to_string())
            )
        },
    )
}

fn format_source_timing_phrase(summary: &CorrelationSummary) -> String {
    if summary.source_timing_malformed {
        return "malformed".to_string();
    }
    summary.source_timing.as_ref().map_or_else(
        || "unknown".to_string(),
        |timing| {
            format!(
                "{} confidence={} drift={} alternates={}",
                timing.phrase_status,
                timing.confidence_result,
                timing.drift_status,
                timing.alternate_evidence_count
            )
        },
    )
}

fn format_source_timing_alignment(summary: &CorrelationSummary) -> String {
    summary.source_timing_alignment.as_ref().map_or_else(
        || "unknown".to_string(),
        |alignment| {
            let warnings = if alignment.warning_overlap.is_empty() {
                "none".to_string()
            } else {
                alignment.warning_overlap.join("+")
            };
            let issues = if alignment.issues.is_empty() {
                "none".to_string()
            } else {
                alignment.issues.join(",")
            };
            format!(
                "{} bpm_delta={} tolerance={:.6} warning_overlap={} issues={}",
                alignment.status,
                format_optional_f64(alignment.bpm_delta),
                alignment.bpm_tolerance,
                warnings,
                issues
            )
        },
    )
}

fn format_source_grid_hit_ratio(summary: &CorrelationSummary) -> String {
    format_optional_f64(
        summary
            .source_grid_output_drift
            .as_ref()
            .map(|drift| drift.hit_ratio),
    )
}

fn format_source_grid_max_peak_offset(summary: &CorrelationSummary) -> String {
    format_optional_f64(
        summary
            .source_grid_output_drift
            .as_ref()
            .map(|drift| drift.max_peak_offset_ms),
    )
}

fn format_source_grid_max_allowed_offset(summary: &CorrelationSummary) -> String {
    format_optional_f64(
        summary
            .source_grid_output_drift
            .as_ref()
            .map(|drift| drift.max_allowed_peak_offset_ms),
    )
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
    let mut failures = metric_failures([
        ("full_mix_rms", summary.full_mix_rms),
        ("full_mix_low_band_rms", summary.full_mix_low_band_rms),
    ]);
    if summary.source_timing_malformed {
        failures.push("source_timing=malformed".to_string());
    }
    failures.extend(source_timing_alignment_failures(summary));
    failures.extend(source_grid_output_drift_failures(summary));
    failures
}

fn source_timing_alignment_failures(summary: &CorrelationSummary) -> Vec<String> {
    summary
        .source_timing_alignment
        .as_ref()
        .map(|alignment| alignment.issues.clone())
        .unwrap_or_default()
}

fn source_grid_output_drift_failures(summary: &CorrelationSummary) -> Vec<String> {
    if summary.source_grid_output_drift_malformed {
        return vec!["source_grid_output_drift=malformed".to_string()];
    }

    let Some(drift) = &summary.source_grid_output_drift else {
        return Vec::new();
    };

    let mut failures = Vec::new();
    if drift.hit_ratio < SOURCE_GRID_OUTPUT_MIN_HIT_RATIO {
        failures.push(format!(
            "source_grid_output_drift.hit_ratio={:.6} < floor {:.6}",
            drift.hit_ratio, SOURCE_GRID_OUTPUT_MIN_HIT_RATIO
        ));
    }
    if drift.max_peak_offset_ms > drift.max_allowed_peak_offset_ms {
        failures.push(format!(
            "source_grid_output_drift.max_peak_offset_ms={:.6} > allowed {:.6}",
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
