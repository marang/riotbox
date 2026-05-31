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
                "cue": &timing.cue,
                "actionability": &timing.actionability,
                "bpm_estimate": timing.bpm_estimate,
                "bpm_confidence": timing.bpm_confidence,
                "quality": &timing.quality,
                "degraded_policy": &timing.degraded_policy,
                "grid_use": &timing.grid_use,
                "beat_status": &timing.beat_status,
                "beat_count": timing.beat_count,
                "downbeat_status": &timing.downbeat_status,
                "primary_downbeat_offset_beats": timing.primary_downbeat_offset_beats,
                "primary_downbeat_score": timing.primary_downbeat_score,
                "primary_downbeat_score_gap": timing.primary_downbeat_score_gap,
                "alternate_downbeat_phase_count": timing.alternate_downbeat_phase_count,
                "bar_count": timing.bar_count,
                "phrase_status": &timing.phrase_status,
                "phrase_count": timing.phrase_count,
                "primary_hypothesis_id": &timing.primary_hypothesis_id,
                "hypothesis_count": timing.hypothesis_count,
                "anchor_evidence": timing.anchor_evidence.as_ref().map(source_timing_anchor_evidence_json),
                "primary_anchor_cue": &timing.primary_anchor_cue,
                "groove_evidence": timing.groove_evidence.as_ref().map(source_timing_groove_evidence_json),
                "primary_warning_code": &timing.primary_warning_code,
                "warning_codes": &timing.warning_codes,
            })),
            "observer_scene_movement": summary.observer_scene_movement.as_ref().map(observer_scene_movement_json),
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
                "actionability": source_timing_readiness_actionability(timing),
                "policy_profile": &timing.policy_profile,
                "grid_use": &timing.grid_use,
                "readiness": &timing.readiness,
                "requires_manual_confirm": timing.requires_manual_confirm,
                "primary_bpm": timing.primary_bpm,
                "bpm_agrees_with_grid": timing.bpm_agrees_with_grid,
                "beat_status": &timing.beat_status,
                "downbeat_status": &timing.downbeat_status,
                "primary_downbeat_offset_beats": timing.primary_downbeat_offset_beats,
                "primary_downbeat_score": timing.primary_downbeat_score,
                "primary_downbeat_margin": timing.primary_downbeat_margin,
                "alternate_downbeat_phase_count": timing.alternate_downbeat_phase_count,
                "confidence_result": &timing.confidence_result,
                "drift_status": &timing.drift_status,
                "phrase_status": &timing.phrase_status,
                "primary_phrase_count": timing.primary_phrase_count,
                "primary_phrase_bar_count": timing.primary_phrase_bar_count,
                "alternate_evidence_count": timing.alternate_evidence_count,
                "anchor_evidence": timing.anchor_evidence.as_ref().map(source_timing_anchor_evidence_json),
                "groove_evidence": timing.groove_evidence.as_ref().map(source_timing_groove_evidence_json),
                "warning_codes": &timing.warning_codes,
            })),
            "source_timing_alignment": summary.source_timing_alignment.as_ref().map(|alignment| serde_json::json!({
                "status": &alignment.status,
                "bpm_delta": alignment.bpm_delta,
                "bpm_tolerance": alignment.bpm_tolerance,
                "observer_grid_use": &alignment.observer_grid_use,
                "manifest_grid_use": &alignment.manifest_grid_use,
                "grid_use_compatibility": &alignment.grid_use_compatibility,
                "observer_downbeat_offset_beats": alignment.observer_downbeat_offset_beats,
                "manifest_downbeat_offset_beats": alignment.manifest_downbeat_offset_beats,
                "downbeat_offset_compatibility": &alignment.downbeat_offset_compatibility,
                "downbeat_ambiguity_compatibility": &alignment.downbeat_ambiguity_compatibility,
                "warning_overlap": &alignment.warning_overlap,
                "issues": &alignment.issues,
            })),
            "source_timing_anchor_alignment": summary.source_timing_anchor_alignment.as_ref().map(|alignment| serde_json::json!({
                "status": &alignment.status,
                "observer": alignment.observer.as_ref().map(source_timing_anchor_evidence_json),
                "manifest": alignment.manifest.as_ref().map(source_timing_anchor_evidence_json),
                "issues": &alignment.issues,
            })),
            "source_timing_groove_alignment": summary.source_timing_groove_alignment.as_ref().map(|alignment| serde_json::json!({
                "status": &alignment.status,
                "observer": alignment.observer.as_ref().map(source_timing_groove_evidence_json),
                "manifest": alignment.manifest.as_ref().map(source_timing_groove_evidence_json),
                "issues": &alignment.issues,
            })),
            "lane_recipe_cases": lane_recipe_cases_json(&summary.lane_recipe_cases),
            "scene_movement_audio_evidence": serde_json::json!({
                "present": summary.observer_scene_movement.is_some(),
                "issues": scene_movement_audio_evidence_failures(summary),
            }),
            "metrics": {
                "full_mix_rms": summary.full_mix_rms,
                "full_mix_low_band_rms": summary.full_mix_low_band_rms,
                "mc202_question_answer_delta_rms": summary.mc202_question_answer_delta_rms,
                "source_grid_output_drift": summary.source_grid_output_drift.as_ref().map(|drift| serde_json::json!({
                    "hit_ratio": drift.hit_ratio,
                    "max_peak_offset_ms": drift.max_peak_offset_ms,
                    "max_allowed_peak_offset_ms": drift.max_allowed_peak_offset_ms,
                })),
                "tr909_source_grid_alignment": summary.tr909_source_grid_alignment.as_ref().map(source_grid_alignment_json),
                "mc202_source_grid_alignment": summary.mc202_source_grid_alignment.as_ref().map(source_grid_alignment_json),
                "mc202_bass_pressure_pattern_origin": &summary.mc202_bass_pressure_pattern_origin,
                "mc202_bass_pressure_applied": summary.mc202_bass_pressure_applied,
                "w30_source_grid_alignment": summary.w30_source_grid_alignment.as_ref().map(source_grid_alignment_json),
                "w30_source_loop_closure": summary.w30_source_loop_closure.as_ref().map(w30_source_loop_closure_json),
                "w30_candidate_rms": summary.w30_candidate_rms,
                "w30_candidate_active_sample_ratio": summary.w30_candidate_active_sample_ratio,
                "w30_rms_delta": summary.w30_rms_delta,
            },
        },
        "needs_human_listening": true,
    }))
    .map(|json| json + "\n")
}

fn observer_scene_movement_json(movement: &ObserverSceneMovementEvidence) -> serde_json::Value {
    serde_json::json!({
        "active_scene": &movement.active_scene,
        "kind": &movement.kind,
        "direction": &movement.direction,
        "tr909_intent": &movement.tr909_intent,
        "mc202_intent": &movement.mc202_intent,
        "intensity": movement.intensity,
        "from_scene": &movement.from_scene,
        "to_scene": &movement.to_scene,
        "committed_bar_index": movement.committed_bar_index,
        "committed_phrase_index": movement.committed_phrase_index,
        "can_use_source_locked_scene_movement": movement.can_use_source_locked_scene_movement,
        "source_anchor_seconds": movement.source_anchor_seconds,
        "source_anchor_position_beats": movement.source_anchor_position_beats,
    })
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
                "{} {} -> {} direction={} 909={} 202={} intensity={:.3} bar={} phrase={} source_locked={} source_anchor={}",
                movement.kind,
                movement.from_scene.as_deref().unwrap_or("none"),
                movement.to_scene,
                movement.direction,
                movement.tr909_intent,
                movement.mc202_intent,
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

fn source_timing_readiness_cue(timing: &SourceTimingEvidence) -> &'static str {
    riotbox_app::source_timing_cues::source_timing_readiness_cue_label(
        &timing.readiness,
        timing.requires_manual_confirm,
    )
}

fn source_timing_readiness_actionability(timing: &SourceTimingEvidence) -> &str {
    timing.actionability.as_deref().unwrap_or_else(|| {
        riotbox_app::source_timing_cues::source_timing_readiness_actionability_label(
            &timing.readiness,
            timing.requires_manual_confirm,
        )
    })
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
                evidence.primary_groove_residual_count,
                evidence.primary_max_abs_offset_ms
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

fn source_grid_alignment_json(drift: &SourceGridOutputDriftEvidence) -> serde_json::Value {
    serde_json::json!({
        "hit_ratio": drift.hit_ratio,
        "max_peak_offset_ms": drift.max_peak_offset_ms,
        "max_allowed_peak_offset_ms": drift.max_allowed_peak_offset_ms,
    })
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

fn w30_source_loop_closure_json(proof: &W30SourceLoopClosureEvidence) -> serde_json::Value {
    serde_json::json!({
        "passed": proof.passed,
        "preview_rms": proof.preview_rms,
        "edge_delta_abs": proof.edge_delta_abs,
        "max_allowed_edge_delta_abs": proof.max_allowed_edge_delta_abs,
        "edge_abs_max": proof.edge_abs_max,
        "max_allowed_edge_abs": proof.max_allowed_edge_abs,
        "source_contains_selection": proof.source_contains_selection,
    })
}
