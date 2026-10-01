use super::report_model::CorrelationSummary;
use super::report_model::SourceGridOutputDriftEvidence;
use super::report_model::SourceTimingAnchorEvidence;
use super::report_model::SourceTimingGrooveEvidence;
use super::source_timing_labels::source_timing_readiness_actionability;
use super::source_timing_labels::source_timing_readiness_cue;
use super::summary_evidence::control_path_present;
use super::summary_evidence::output_path_evidence_failures;
use super::summary_evidence::output_path_present;
use super::summary_evidence::scene_movement_audio_evidence_failures;

fn format_observer_source_timing(summary: &CorrelationSummary) -> String {
    if summary.observer_source_timing_malformed {
        return "malformed".to_string();
    }
    summary.observer_source_timing.as_ref().map_or_else(
        || "unknown".to_string(),
        |timing| {
            format!(
                "{} cue={} actionability={} grid_use={} quality={} policy={} bpm={} confidence={:.3} beat={}({}) downbeat={}({}) offset={} downbeat_score={} downbeat_gap={} downbeat_alts={} phrase={}({}) anchors={} anchor_cue=\"{}\" groove={} warning={}",
                timing.source_id,
                timing.cue,
                timing.actionability,
                timing.grid_use,
                timing.quality,
                timing.degraded_policy,
                format_optional_f64(timing.bpm_estimate),
                timing.bpm_confidence,
                timing.beat_status,
                timing.beat_count,
                timing.downbeat_status,
                timing.bar_count,
                timing
                    .primary_downbeat_offset_beats
                    .map_or_else(|| "none".to_string(), |offset| offset.to_string()),
                format_optional_f64(timing.primary_downbeat_score),
                format_optional_f64(timing.primary_downbeat_score_gap),
                timing.alternate_downbeat_phase_count,
                timing.phrase_status,
                timing.phrase_count,
                format_source_timing_anchor_counts(timing.anchor_evidence.as_ref()),
                timing.primary_anchor_cue,
                format_source_timing_groove_counts(timing.groove_evidence.as_ref()),
                timing
                    .primary_warning_code
                    .as_deref()
                    .unwrap_or("none")
            )
        },
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

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

pub(super) fn render_markdown(summary: &CorrelationSummary) -> String {
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
         - Observer scene movement: `{}`\n\n\
         ## Output Path\n\n\
         - Pack id: `{}`\n\
         - Manifest result: `{}`\n\
         - Artifact count: `{}`\n\
         - Grid BPM source: `{}`\n\
         - Grid BPM decision reason: `{}`\n\
         - Source timing BPM delta: `{}`\n\
         - Source timing BPM agrees with grid: `{}`\n\
         - Full mix RMS: `{}`\n\
         - Full mix low-band RMS: `{}`\n\n\
         - Source timing readiness: `{}`\n\
         - Source timing grid use: `{}`\n\
         - Source timing downbeat: `{}`\n\
         - Source timing phrase: `{}`\n\n\
         - Source timing alignment: `{}`\n\n\
         - Source timing anchor alignment: `{}`\n\n\
         - Source timing groove alignment: `{}`\n\n\
         - Source-grid output hit ratio: `{}`\n\
         - Source-grid output max peak offset: `{}`\n\
         - Source-grid output max allowed offset: `{}`\n\n\
         - TR-909 source-grid alignment: `{}`\n\
         - MC-202 source-grid alignment: `{}`\n\
         - MC-202 bass-pressure origin: `{}` applied `{}`\n\
         - MC-202 source-expression: plan `{}` role `{}` fallback `{}` contour `{}` applied `{}` delta `{}` min `{}`\n\
         - W-30 source-grid alignment: `{}`\n\n\
         - W-30 source-loop closure: `{}`\n\n\
         - Scene movement/audio evidence: `{}`\n\n\
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
        format_observer_scene_movement(summary),
        summary.pack_id,
        summary.manifest_result,
        summary.artifact_count,
        summary.grid_bpm_source,
        summary.grid_bpm_decision_reason,
        format_optional_f64(summary.source_timing_bpm_delta),
        format_source_timing_bpm_agreement(summary),
        format_optional_f64(summary.full_mix_rms),
        format_optional_f64(summary.full_mix_low_band_rms),
        format_source_timing_readiness(summary),
        format_source_timing_grid_use(summary),
        format_source_timing_downbeat(summary),
        format_source_timing_phrase(summary),
        format_source_timing_alignment(summary),
        format_source_timing_anchor_alignment(summary),
        format_source_timing_groove_alignment(summary),
        format_source_grid_hit_ratio(summary),
        format_source_grid_max_peak_offset(summary),
        format_source_grid_max_allowed_offset(summary),
        format_source_grid_alignment(&summary.tr909_source_grid_alignment),
        format_source_grid_alignment(&summary.mc202_source_grid_alignment),
        summary.mc202_bass_pressure_pattern_origin,
        format_optional_bool(summary.mc202_bass_pressure_applied),
        format_optional_bool(summary.mc202_source_expression_render_plan_applied),
        summary.mc202_source_expression_role,
        format_optional_bool(summary.mc202_source_failure_fallback),
        summary.mc202_source_contour_pattern_origin,
        format_optional_bool(summary.mc202_source_contour_applied),
        format_optional_f64(summary.mc202_source_contour_delta_rms),
        format_optional_f64(summary.mc202_source_contour_min_required_delta_rms),
        format_source_grid_alignment(&summary.w30_source_grid_alignment),
        format_w30_source_loop_closure(summary),
        format_scene_movement_audio_evidence(summary),
        format_optional_f64(summary.w30_candidate_rms),
        format_optional_f64(summary.w30_candidate_active_sample_ratio),
        format_optional_f64(summary.w30_rms_delta),
        yes_no(control_path_present(summary)),
        yes_no(output_path_present(summary)),
        format_output_path_issues(summary)
    )
}

fn format_optional_f64(value: Option<f64>) -> String {
    value.map_or_else(|| "unknown".to_string(), |value| format!("{value:.6}"))
}

fn format_optional_bool(value: Option<bool>) -> String {
    value.map_or_else(|| "unknown".to_string(), |value| yes_no(value).to_string())
}

fn format_source_timing_bpm_agreement(summary: &CorrelationSummary) -> String {
    if summary.source_timing_malformed {
        return "malformed".to_string();
    }
    summary
        .source_timing
        .as_ref()
        .and_then(|timing| timing.bpm_agrees_with_grid)
        .map_or_else(|| "unknown".to_string(), |value| yes_no(value).to_string())
}

fn format_observer_scene_movement(summary: &CorrelationSummary) -> String {
    if summary.observer_scene_movement_malformed {
        return "malformed".to_string();
    }
    summary.observer_scene_movement.as_ref().map_or_else(
        || "none".to_string(),
        |movement| {
            format!(
                "{} {} -> {} direction={} 909={} 202={} w30={} intensity={:.3} bar={} phrase={} source_locked={} source_anchor={}",
                movement.kind,
                movement.from_scene.as_deref().unwrap_or("none"),
                movement.to_scene,
                movement.direction,
                movement.tr909_intent,
                movement.mc202_intent,
                movement.w30_intent,
                movement.intensity,
                movement.committed_bar_index,
                movement.committed_phrase_index,
                yes_no(movement.can_use_source_locked_scene_movement),
                format_optional_f64(movement.source_anchor_seconds)
            )
        },
    )
}

fn format_scene_movement_audio_evidence(summary: &CorrelationSummary) -> String {
    let failures = scene_movement_audio_evidence_failures(summary);
    if summary.observer_scene_movement.is_none() && !summary.observer_scene_movement_malformed {
        return "none".to_string();
    }
    if failures.is_empty() {
        "pass".to_string()
    } else {
        failures.join(", ")
    }
}

fn format_source_timing_readiness(summary: &CorrelationSummary) -> String {
    if summary.source_timing_malformed {
        return "malformed".to_string();
    }
    summary.source_timing.as_ref().map_or_else(
        || "unknown".to_string(),
        |timing| {
            format!(
                "{} actionability={} readiness={} manual_confirm={}",
                source_timing_readiness_cue(timing),
                source_timing_readiness_actionability(timing),
                timing.readiness,
                yes_no(timing.requires_manual_confirm)
            )
        },
    )
}

fn format_source_timing_grid_use(summary: &CorrelationSummary) -> String {
    if summary.source_timing_malformed {
        return "malformed".to_string();
    }
    summary
        .source_timing
        .as_ref()
        .and_then(|timing| timing.grid_use.as_deref())
        .unwrap_or("unknown")
        .to_string()
}

fn format_source_timing_downbeat(summary: &CorrelationSummary) -> String {
    if summary.source_timing_malformed {
        return "malformed".to_string();
    }
    summary.source_timing.as_ref().map_or_else(
        || "unknown".to_string(),
        |timing| {
            format!(
                "{} offset={} score={} margin={} alts={}",
                timing.downbeat_status,
                timing
                    .primary_downbeat_offset_beats
                    .map_or_else(|| "unknown".to_string(), |value| value.to_string()),
                format_optional_f64(timing.primary_downbeat_score),
                format_optional_f64(timing.primary_downbeat_margin),
                timing
                    .alternate_downbeat_phase_count
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
                "{} phrases={} bars={} confidence={} drift={} alternates={}",
                timing.phrase_status,
                timing.primary_phrase_count,
                timing.primary_phrase_bar_count,
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
                "{} bpm_delta={} tolerance={:.6} grid_use={} observer_grid_use={} manifest_grid_use={} downbeat_offset={} downbeat_ambiguity={} observer_downbeat_offset={} manifest_downbeat_offset={} warning_overlap={} issues={}",
                alignment.status,
                format_optional_f64(alignment.bpm_delta),
                alignment.bpm_tolerance,
                alignment.grid_use_compatibility,
                alignment.observer_grid_use,
                alignment
                    .manifest_grid_use
                    .as_deref()
                    .unwrap_or("unknown"),
                alignment.downbeat_offset_compatibility,
                alignment.downbeat_ambiguity_compatibility,
                alignment
                    .observer_downbeat_offset_beats
                    .map_or_else(|| "unknown".to_string(), |value| value.to_string()),
                alignment
                    .manifest_downbeat_offset_beats
                    .map_or_else(|| "unknown".to_string(), |value| value.to_string()),
                warnings,
                issues
            )
        },
    )
}

fn format_source_timing_anchor_alignment(summary: &CorrelationSummary) -> String {
    summary.source_timing_anchor_alignment.as_ref().map_or_else(
        || "unknown".to_string(),
        |alignment| {
            let issues = if alignment.issues.is_empty() {
                "none".to_string()
            } else {
                alignment.issues.join(",")
            };
            format!(
                "{} observer={} manifest={} issues={}",
                alignment.status,
                format_source_timing_anchor_counts(alignment.observer.as_ref()),
                format_source_timing_anchor_counts(alignment.manifest.as_ref()),
                issues
            )
        },
    )
}

fn format_source_timing_anchor_counts(evidence: Option<&SourceTimingAnchorEvidence>) -> String {
    evidence.map_or_else(
        || "missing".to_string(),
        |evidence| {
            format!(
                "{}(kick={} backbeat={} transient={})",
                evidence.primary_anchor_count,
                evidence.primary_kick_anchor_count,
                evidence.primary_backbeat_anchor_count,
                evidence.primary_transient_anchor_count
            )
        },
    )
}

fn format_source_timing_groove_alignment(summary: &CorrelationSummary) -> String {
    summary.source_timing_groove_alignment.as_ref().map_or_else(
        || "unknown".to_string(),
        |alignment| {
            let issues = if alignment.issues.is_empty() {
                "none".to_string()
            } else {
                alignment.issues.join(",")
            };
            format!(
                "{} observer={} manifest={} issues={}",
                alignment.status,
                format_source_timing_groove_counts(alignment.observer.as_ref()),
                format_source_timing_groove_counts(alignment.manifest.as_ref()),
                issues
            )
        },
    )
}

fn format_source_timing_groove_counts(evidence: Option<&SourceTimingGrooveEvidence>) -> String {
    evidence.map_or_else(
        || "missing".to_string(),
        |evidence| {
            format!(
                "{}(max_abs_ms={:.3})",
                evidence.primary_groove_residual_count, evidence.primary_max_abs_offset_ms
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

fn format_source_grid_alignment(drift: &Option<SourceGridOutputDriftEvidence>) -> String {
    drift.as_ref().map_or_else(
        || "unknown".to_string(),
        |drift| {
            format!(
                "hit_ratio={} max_peak_offset_ms={} max_allowed_peak_offset_ms={}",
                format_optional_f64(Some(drift.hit_ratio)),
                format_optional_f64(Some(drift.max_peak_offset_ms)),
                format_optional_f64(Some(drift.max_allowed_peak_offset_ms))
            )
        },
    )
}

fn format_w30_source_loop_closure(summary: &CorrelationSummary) -> String {
    if summary.w30_source_loop_closure_malformed {
        return "malformed".to_string();
    }
    summary.w30_source_loop_closure.as_ref().map_or_else(
        || "unknown".to_string(),
        |proof| {
            format!(
                "passed={} preview_rms={} edge_delta_abs={} max_allowed_edge_delta_abs={} edge_abs_max={} max_allowed_edge_abs={} source_contains_selection={}",
                yes_no(proof.passed),
                format_optional_f64(Some(proof.preview_rms)),
                format_optional_f64(Some(proof.edge_delta_abs)),
                format_optional_f64(Some(proof.max_allowed_edge_delta_abs)),
                format_optional_f64(Some(proof.edge_abs_max)),
                format_optional_f64(Some(proof.max_allowed_edge_abs)),
                yes_no(proof.source_contains_selection)
            )
        },
    )
}
