use super::lane_recipe_output::LaneRecipeCaseEvidence;

#[derive(Debug, PartialEq)]
pub(super) struct ObserverSourceTimingReadiness {
    pub(super) source_id: String,
    pub(super) cue: String,
    pub(super) actionability: String,
    pub(super) bpm_estimate: Option<f64>,
    pub(super) bpm_confidence: f64,
    pub(super) quality: String,
    pub(super) degraded_policy: String,
    pub(super) grid_use: String,
    pub(super) beat_status: String,
    pub(super) beat_count: u64,
    pub(super) downbeat_status: String,
    pub(super) primary_downbeat_offset_beats: Option<u64>,
    pub(super) primary_downbeat_score: Option<f64>,
    pub(super) primary_downbeat_score_gap: Option<f64>,
    pub(super) alternate_downbeat_phase_count: u64,
    pub(super) has_alternate_downbeat_phase_count: bool,
    pub(super) bar_count: u64,
    pub(super) phrase_status: String,
    pub(super) phrase_count: u64,
    pub(super) primary_hypothesis_id: Option<String>,
    pub(super) hypothesis_count: u64,
    pub(super) anchor_evidence: Option<SourceTimingAnchorEvidence>,
    pub(super) primary_anchor_cue: String,
    pub(super) groove_evidence: Option<SourceTimingGrooveEvidence>,
    pub(super) primary_warning_code: Option<String>,
    pub(super) warning_codes: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub(super) struct SourceTimingAlignmentEvidence {
    pub(super) status: String,
    pub(super) bpm_delta: Option<f64>,
    pub(super) bpm_tolerance: f64,
    pub(super) observer_grid_use: String,
    pub(super) manifest_grid_use: Option<String>,
    pub(super) grid_use_compatibility: String,
    pub(super) observer_downbeat_offset_beats: Option<u64>,
    pub(super) manifest_downbeat_offset_beats: Option<u64>,
    pub(super) downbeat_offset_compatibility: String,
    pub(super) downbeat_ambiguity_compatibility: String,
    pub(super) warning_overlap: Vec<String>,
    pub(super) issues: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub(super) struct SourceTimingAnchorAlignmentEvidence {
    pub(super) status: String,
    pub(super) observer: Option<SourceTimingAnchorEvidence>,
    pub(super) manifest: Option<SourceTimingAnchorEvidence>,
    pub(super) issues: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub(super) struct SourceTimingGrooveAlignmentEvidence {
    pub(super) status: String,
    pub(super) observer: Option<SourceTimingGrooveEvidence>,
    pub(super) manifest: Option<SourceTimingGrooveEvidence>,
    pub(super) issues: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SourceTimingAnchorEvidence {
    pub(super) primary_anchor_count: u64,
    pub(super) primary_kick_anchor_count: u64,
    pub(super) primary_backbeat_anchor_count: u64,
    pub(super) primary_transient_anchor_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SourceTimingGrooveEvidence {
    pub(super) primary_groove_residual_count: u64,
    pub(super) primary_max_abs_offset_ms: f64,
    pub(super) primary_groove_preview: Vec<SourceTimingGrooveResidualEvidence>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SourceTimingGrooveResidualEvidence {
    pub(super) subdivision: String,
    pub(super) offset_ms: f64,
    pub(super) confidence: f64,
}

#[derive(Debug, PartialEq)]
pub(super) struct CorrelationSummary {
    pub(super) observer_schema: String,
    pub(super) launch_mode: String,
    pub(super) audio_runtime_status: String,
    pub(super) key_outcomes: Vec<String>,
    pub(super) first_commit: String,
    pub(super) commit_count: usize,
    pub(super) commit_boundaries: Vec<String>,
    pub(super) observer_source_timing: Option<ObserverSourceTimingReadiness>,
    pub(super) observer_source_timing_malformed: bool,
    pub(super) observer_scene_movement: Option<ObserverSceneMovementEvidence>,
    pub(super) observer_scene_movement_malformed: bool,
    pub(super) pack_id: String,
    pub(super) manifest_result: String,
    pub(super) artifact_count: usize,
    pub(super) grid_bpm_source: String,
    pub(super) grid_bpm_decision_reason: String,
    pub(super) source_timing_bpm_delta: Option<f64>,
    pub(super) full_mix_rms: Option<f64>,
    pub(super) full_mix_low_band_rms: Option<f64>,
    pub(super) mc202_question_answer_delta_rms: Option<f64>,
    pub(super) w30_candidate_rms: Option<f64>,
    pub(super) w30_candidate_active_sample_ratio: Option<f64>,
    pub(super) w30_rms_delta: Option<f64>,
    pub(super) source_timing: Option<SourceTimingEvidence>,
    pub(super) source_timing_malformed: bool,
    pub(super) source_timing_alignment: Option<SourceTimingAlignmentEvidence>,
    pub(super) source_timing_anchor_alignment: Option<SourceTimingAnchorAlignmentEvidence>,
    pub(super) source_timing_groove_alignment: Option<SourceTimingGrooveAlignmentEvidence>,
    pub(super) source_grid_output_drift: Option<SourceGridOutputDriftEvidence>,
    pub(super) source_grid_output_drift_malformed: bool,
    pub(super) tr909_source_grid_alignment: Option<SourceGridOutputDriftEvidence>,
    pub(super) tr909_source_grid_alignment_malformed: bool,
    pub(super) mc202_source_grid_alignment: Option<SourceGridOutputDriftEvidence>,
    pub(super) mc202_source_grid_alignment_malformed: bool,
    pub(super) mc202_bass_pressure_pattern_origin: String,
    pub(super) mc202_bass_pressure_applied: Option<bool>,
    pub(super) mc202_source_expression_render_plan_applied: Option<bool>,
    pub(super) mc202_source_expression_role: String,
    pub(super) mc202_source_failure_fallback: Option<bool>,
    pub(super) mc202_source_contour_pattern_origin: String,
    pub(super) mc202_source_contour_applied: Option<bool>,
    pub(super) mc202_source_contour_delta_rms: Option<f64>,
    pub(super) mc202_source_contour_min_required_delta_rms: Option<f64>,
    pub(super) w30_source_grid_alignment: Option<SourceGridOutputDriftEvidence>,
    pub(super) w30_source_grid_alignment_malformed: bool,
    pub(super) w30_source_loop_closure: Option<W30SourceLoopClosureEvidence>,
    pub(super) w30_source_loop_closure_malformed: bool,
    pub(super) lane_recipe_cases: Vec<LaneRecipeCaseEvidence>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ObserverSceneMovementEvidence {
    pub(super) active_scene: Option<String>,
    pub(super) kind: String,
    pub(super) direction: String,
    pub(super) tr909_intent: String,
    pub(super) mc202_intent: String,
    pub(super) w30_intent: String,
    pub(super) intensity: f64,
    pub(super) from_scene: Option<String>,
    pub(super) to_scene: String,
    pub(super) committed_bar_index: u64,
    pub(super) committed_phrase_index: u64,
    pub(super) can_use_source_locked_scene_movement: bool,
    pub(super) source_anchor_seconds: Option<f64>,
    pub(super) source_anchor_position_beats: Option<f64>,
}

#[derive(Debug, PartialEq)]
pub(super) struct SourceTimingEvidence {
    pub(super) source_id: String,
    pub(super) policy_profile: String,
    pub(super) actionability: Option<String>,
    pub(super) grid_use: Option<String>,
    pub(super) readiness: String,
    pub(super) requires_manual_confirm: bool,
    pub(super) primary_bpm: Option<f64>,
    pub(super) bpm_agrees_with_grid: Option<bool>,
    pub(super) beat_status: String,
    pub(super) downbeat_status: String,
    pub(super) primary_downbeat_offset_beats: Option<u64>,
    pub(super) primary_downbeat_score: Option<f64>,
    pub(super) primary_downbeat_margin: Option<f64>,
    pub(super) alternate_downbeat_phase_count: Option<u64>,
    pub(super) confidence_result: String,
    pub(super) drift_status: String,
    pub(super) phrase_status: String,
    pub(super) primary_phrase_count: u64,
    pub(super) primary_phrase_bar_count: u64,
    pub(super) alternate_evidence_count: u64,
    pub(super) anchor_evidence: Option<SourceTimingAnchorEvidence>,
    pub(super) groove_evidence: Option<SourceTimingGrooveEvidence>,
    pub(super) warning_codes: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub(super) struct SourceGridOutputDriftEvidence {
    pub(super) hit_ratio: f64,
    pub(super) max_peak_offset_ms: f64,
    pub(super) max_allowed_peak_offset_ms: f64,
}

#[derive(Debug, PartialEq)]
pub(super) struct W30SourceLoopClosureEvidence {
    pub(super) passed: bool,
    pub(super) preview_rms: f64,
    pub(super) edge_delta_abs: f64,
    pub(super) max_allowed_edge_delta_abs: f64,
    pub(super) edge_abs_max: f64,
    pub(super) max_allowed_edge_abs: f64,
    pub(super) source_contains_selection: bool,
}
