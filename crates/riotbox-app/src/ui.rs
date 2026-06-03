// Keep the remaining TUI include shards behavior-preserving while semantic
// slices move to explicit child modules.
include!("ui/types_state.rs");

mod first_run_capture;
#[path = "ui/footer_help_perform_lines/lane_perform_lines.rs"]
mod footer_lane_perform_lines;
mod footer_renderer;
#[path = "ui/jam_landed_warnings_source/scene_commit_cues.rs"]
mod jam_scene_commit_cues;
mod recovery_prompt;
#[path = "ui/scene_timing_rail/labels.rs"]
mod scene_timing_labels;
mod source_timing_panel;
mod source_trust_summary;
#[path = "ui/diagnostics_mc202_w30_logs/stem_package_export.rs"]
mod stem_package_export_inspect;
#[path = "ui/capture_log_source_lists/w30_cue_labels.rs"]
mod w30_cue_labels;
#[path = "ui/diagnostics_mc202_w30_logs/w30_preview.rs"]
mod w30_preview_labels;
#[path = "ui/diagnostics_mc202_w30_logs/w30_resample.rs"]
mod w30_resample_labels;
#[path = "ui/w30_capture_source_helpers/slice_pool.rs"]
mod w30_slice_pool_helpers;

use first_run_capture::{
    FirstRunOnrampStage, capture_do_next_lines, capture_latest_lines, capture_lines,
    capture_provenance_lines, capture_readiness_lines, capture_routing_lines,
    first_run_onramp_stage, pending_capture_lines, recent_capture_items,
};
#[cfg(test)]
use first_run_capture::{capture_pending_detail_line, capture_pending_intent_line};
use footer_lane_perform_lines::{
    mc202_perform_lines, tr909_inspect_lines, tr909_perform_lines, w30_perform_lines,
};
#[cfg(test)]
use footer_renderer::{
    footer_advanced_line, footer_keys_line, footer_lane_ops_line, footer_primary_line,
    footer_scene_line,
};
use footer_renderer::{render_footer, render_help_primary_gesture_items};
use jam_scene_commit_cues::{
    latest_landed_command, scene_history_trail_line, scene_post_commit_cue_line,
};
use recovery_prompt::{recovery_help_lines, recovery_warning_line};
use riotbox_core::view::jam::ArrangementSceneContractReadinessView;
#[cfg(test)]
use riotbox_core::view::jam::{CaptureHandoffReadinessView, CaptureTargetKindView};
use scene_timing_labels::{
    compact_energy_delta_label, compact_scene_label, current_scene_compact_label,
    current_scene_target_compact_label, energy_delta_label, next_action_line,
    next_scene_jump_suggestion, next_scene_target_compact_label, now_line,
    quantization_boundary_label, restore_scene_energy_direction_label, restore_scene_label,
    restore_scene_now_compact_label, restore_scene_target_compact_label,
    scene_energy_label_for_scene_id, scene_restore_contrast_line,
};
use source_timing_panel::{source_map_lines, source_timing_lines};
use source_trust_summary::{
    arrangement_inspect_lines, arrangement_proof_line, arrangement_taste_line, energy_label,
    source_candidate_lines, source_confidence_lines, source_provenance_lines,
    source_timing_clock_compact, source_timing_clock_line, source_timing_grid_confirmed,
    source_timing_help_line, source_timing_performance_rail_line, source_timing_readiness_line,
    source_timing_warning_line, source_warning_lines, trust_summary,
};
use stem_package_export_inspect::stem_package_export_receipt_lines;
use w30_cue_labels::{last_committed_w30_action, short_w30_action_label, w30_pending_cue_label};
use w30_preview_labels::{
    w30_preview_log_compact, w30_preview_mode_profile_compact, w30_preview_source_readiness,
};
use w30_resample_labels::{
    w30_capture_lineage_compact, w30_resample_lineage_active, w30_resample_log_focus_compact,
    w30_resample_mix_log_compact, w30_resample_route_compact, w30_resample_source_compact,
    w30_resample_tap_compact,
};
use w30_slice_pool_helpers::{
    w30_slice_pool_compact, w30_slice_pool_log_compact, w30_slice_pool_relevant,
};

include!("ui/shell_render_root.rs");
include!("ui/jam_perform_layout.rs");
include!("ui/screen_bodies_footer_start.rs");
include!("ui/footer_help_perform_lines.rs");
include!("ui/jam_landed_warnings_source.rs");
include!("ui/diagnostics_mc202_w30_logs.rs");
include!("ui/w30_capture_source_helpers.rs");
include!("ui/scene_timing_rail.rs");
include!("ui/capture_log_source_lists.rs");

#[cfg(test)]
mod tests;
