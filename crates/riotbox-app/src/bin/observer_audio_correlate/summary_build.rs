use super::lane_recipe_output::collect_lane_recipe_cases;
use super::manifest_metrics::collect_metric_bool;
use super::manifest_metrics::collect_metric_f64;
use super::manifest_metrics::collect_metric_string;
use super::manifest_metrics::collect_source_grid_alignment;
use super::manifest_metrics::collect_source_grid_output_drift;
use super::manifest_metrics::collect_w30_source_loop_closure;
use super::manifest_source_timing::collect_source_timing;
use super::metadata_io::read_manifest;
#[cfg(test)]
use super::metadata_io::read_observer_events;
use super::observer_events::collect_commit_summary;
use super::observer_events::format_first_commit;
use super::observer_events::string_field;
use super::observer_source_timing::collect_observer_source_timing;
use super::report_model::CorrelationSummary;
use super::scene_movement::collect_observer_scene_movement;
use super::source_timing_alignment::collect_source_timing_alignment;
use super::source_timing_alignment::collect_source_timing_anchor_alignment;
use super::source_timing_alignment::collect_source_timing_groove_alignment;
use serde_json::Value;
use std::path::Path;

#[cfg(test)]
pub(super) fn build_summary(
    observer_path: &Path,
    manifest_path: &Path,
) -> Result<CorrelationSummary, Box<dyn std::error::Error>> {
    let observer_events = read_observer_events(observer_path)?;
    build_summary_from_events(&observer_events, manifest_path)
}

pub(super) fn build_summary_from_events(
    observer_events: &[Value],
    manifest_path: &Path,
) -> Result<CorrelationSummary, Box<dyn std::error::Error>> {
    let manifest = read_manifest(manifest_path)?;

    let launch = observer_events
        .iter()
        .find(|event| event["event"] == "observer_started");
    let audio_runtime = observer_events
        .iter()
        .rev()
        .find(|event| event["event"] == "audio_runtime");
    let key_outcomes = observer_events
        .iter()
        .filter(|event| event["event"] == "key_outcome")
        .map(|event| {
            format!(
                "{} -> {}",
                string_field(event, "key"),
                string_field(event, "outcome")
            )
        })
        .collect::<Vec<_>>();
    let first_commit = observer_events
        .iter()
        .find(|event| event["event"] == "transport_commit")
        .and_then(format_first_commit)
        .unwrap_or_else(|| "none".to_string());
    let (commit_count, commit_boundaries) = collect_commit_summary(observer_events);
    let (observer_source_timing, observer_source_timing_malformed) =
        collect_observer_source_timing(observer_events);
    let (observer_scene_movement, observer_scene_movement_malformed) =
        collect_observer_scene_movement(observer_events);

    let (source_grid_output_drift, source_grid_output_drift_malformed) =
        collect_source_grid_output_drift(&manifest);
    let (tr909_source_grid_alignment, tr909_source_grid_alignment_malformed) =
        collect_source_grid_alignment(&manifest, "tr909_source_grid_alignment");
    let (mc202_source_grid_alignment, mc202_source_grid_alignment_malformed) =
        collect_source_grid_alignment(&manifest, "mc202_source_grid_alignment");
    let mc202_bass_pressure_pattern_origin =
        collect_metric_string(&manifest, "mc202_bass_pressure", "pattern_origin")
            .unwrap_or_else(|| "unknown".to_string());
    let mc202_bass_pressure_applied =
        collect_metric_bool(&manifest, "mc202_bass_pressure", "applied");
    let mc202_source_expression_render_plan_applied = collect_metric_bool(
        &manifest,
        "mc202_bass_pressure",
        "source_expression_render_plan_applied",
    );
    let mc202_source_expression_role =
        collect_metric_string(&manifest, "mc202_bass_pressure", "source_expression_role")
            .unwrap_or_else(|| "unknown".to_string());
    let mc202_source_failure_fallback =
        collect_metric_bool(&manifest, "mc202_bass_pressure", "source_failure_fallback");
    let mc202_source_contour_pattern_origin =
        collect_metric_string(&manifest, "mc202_source_contour", "pattern_origin")
            .unwrap_or_else(|| "unknown".to_string());
    let mc202_source_contour_applied =
        collect_metric_bool(&manifest, "mc202_source_contour", "applied");
    let mc202_source_contour_delta_rms = collect_metric_f64(
        &manifest,
        "mc202_source_contour",
        "source_contour_delta_rms",
    );
    let mc202_source_contour_min_required_delta_rms =
        collect_metric_f64(&manifest, "mc202_source_contour", "min_required_delta_rms");
    let (w30_source_grid_alignment, w30_source_grid_alignment_malformed) =
        collect_source_grid_alignment(&manifest, "w30_source_grid_alignment");
    let (w30_source_loop_closure, w30_source_loop_closure_malformed) =
        collect_w30_source_loop_closure(&manifest);
    let (source_timing, source_timing_malformed) = collect_source_timing(&manifest);
    let source_timing_alignment = collect_source_timing_alignment(
        observer_source_timing.as_ref(),
        source_timing.as_ref(),
        observer_source_timing_malformed,
        source_timing_malformed,
    );
    let source_timing_anchor_alignment = collect_source_timing_anchor_alignment(
        observer_source_timing.as_ref(),
        source_timing.as_ref(),
        observer_source_timing_malformed,
        source_timing_malformed,
    );
    let source_timing_groove_alignment = collect_source_timing_groove_alignment(
        observer_source_timing.as_ref(),
        source_timing.as_ref(),
        observer_source_timing_malformed,
        source_timing_malformed,
    );

    Ok(CorrelationSummary {
        observer_schema: launch
            .and_then(|event| event["schema"].as_str())
            .unwrap_or("unknown")
            .to_string(),
        launch_mode: launch
            .and_then(|event| event["launch"]["mode"].as_str())
            .unwrap_or("unknown")
            .to_string(),
        audio_runtime_status: audio_runtime
            .and_then(|event| event["status"].as_str())
            .unwrap_or("unknown")
            .to_string(),
        key_outcomes,
        first_commit,
        commit_count,
        commit_boundaries,
        observer_source_timing,
        observer_source_timing_malformed,
        observer_scene_movement,
        observer_scene_movement_malformed,
        pack_id: manifest["pack_id"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        manifest_result: manifest["result"].as_str().unwrap_or("unknown").to_string(),
        artifact_count: manifest["artifacts"].as_array().map_or(0, Vec::len),
        grid_bpm_source: manifest["grid_bpm_source"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        grid_bpm_decision_reason: manifest["grid_bpm_decision_reason"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        source_timing_bpm_delta: manifest["source_timing_bpm_delta"].as_f64(),
        full_mix_rms: manifest["metrics"]["full_grid_mix"]["signal"]["rms"].as_f64(),
        full_mix_low_band_rms: manifest["metrics"]["full_grid_mix"]["low_band"]["rms"].as_f64(),
        mc202_question_answer_delta_rms: manifest["metrics"]["mc202_question_answer_delta"]["rms"]
            .as_f64(),
        w30_candidate_rms: manifest["metrics"]["candidate"]["rms"].as_f64(),
        w30_candidate_active_sample_ratio: manifest["metrics"]["candidate"]["active_sample_ratio"]
            .as_f64(),
        w30_rms_delta: manifest["metrics"]["deltas"]["rms"].as_f64(),
        source_timing,
        source_timing_malformed,
        source_timing_alignment,
        source_timing_anchor_alignment,
        source_timing_groove_alignment,
        source_grid_output_drift,
        source_grid_output_drift_malformed,
        tr909_source_grid_alignment,
        tr909_source_grid_alignment_malformed,
        mc202_source_grid_alignment,
        mc202_source_grid_alignment_malformed,
        mc202_bass_pressure_pattern_origin,
        mc202_bass_pressure_applied,
        mc202_source_expression_render_plan_applied,
        mc202_source_expression_role,
        mc202_source_failure_fallback,
        mc202_source_contour_pattern_origin,
        mc202_source_contour_applied,
        mc202_source_contour_delta_rms,
        mc202_source_contour_min_required_delta_rms,
        w30_source_grid_alignment,
        w30_source_grid_alignment_malformed,
        w30_source_loop_closure,
        w30_source_loop_closure_malformed,
        lane_recipe_cases: collect_lane_recipe_cases(&manifest),
    })
}
