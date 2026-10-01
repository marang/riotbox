use super::lane_recipe_output::lane_recipe_cases_json;
use super::report_model::CorrelationSummary;
use super::report_model::ObserverSceneMovementEvidence;
use super::report_model::SourceGridOutputDriftEvidence;
use super::report_model::W30SourceLoopClosureEvidence;
use super::source_timing_anchor_evidence::source_timing_anchor_evidence_json;
use super::source_timing_groove_evidence::source_timing_groove_evidence_json;
use super::source_timing_labels::source_timing_readiness_actionability;
use super::source_timing_labels::source_timing_readiness_cue;
use super::summary_evidence::control_path_present;
use super::summary_evidence::output_path_evidence_failures;
use super::summary_evidence::output_path_present;
use super::summary_evidence::scene_movement_audio_evidence_failures;

pub(super) const SUMMARY_SCHEMA: &str = "riotbox.observer_audio_summary.v1";

pub(super) const SUMMARY_SCHEMA_VERSION: u32 = 1;

pub(super) fn render_json(summary: &CorrelationSummary) -> Result<String, serde_json::Error> {
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
                "mc202_source_expression_render_plan_applied": summary.mc202_source_expression_render_plan_applied,
                "mc202_source_expression_role": &summary.mc202_source_expression_role,
                "mc202_source_failure_fallback": summary.mc202_source_failure_fallback,
                "mc202_source_contour_pattern_origin": &summary.mc202_source_contour_pattern_origin,
                "mc202_source_contour_applied": summary.mc202_source_contour_applied,
                "mc202_source_contour_delta_rms": summary.mc202_source_contour_delta_rms,
                "mc202_source_contour_min_required_delta_rms": summary.mc202_source_contour_min_required_delta_rms,
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
        "w30_intent": &movement.w30_intent,
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

fn source_grid_alignment_json(drift: &SourceGridOutputDriftEvidence) -> serde_json::Value {
    serde_json::json!({
        "hit_ratio": drift.hit_ratio,
        "max_peak_offset_ms": drift.max_peak_offset_ms,
        "max_allowed_peak_offset_ms": drift.max_allowed_peak_offset_ms,
    })
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
