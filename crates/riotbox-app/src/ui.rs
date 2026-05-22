// Keep the remaining TUI include shards behavior-preserving while semantic
// slices move to explicit child modules.
include!("ui/types_state.rs");

mod first_run_capture;
#[path = "ui/jam_landed_warnings_source/scene_commit_cues.rs"]
mod jam_scene_commit_cues;
mod recovery_prompt;
mod source_timing_panel;
mod source_trust_summary;

use first_run_capture::{
    FirstRunOnrampStage, capture_do_next_lines, capture_latest_lines, capture_lines,
    capture_provenance_lines, capture_readiness_lines, capture_routing_lines,
    first_run_onramp_stage, pending_capture_lines, recent_capture_items,
};
#[cfg(test)]
use first_run_capture::{capture_pending_detail_line, capture_pending_intent_line};
use jam_scene_commit_cues::{
    latest_landed_command, scene_history_trail_line, scene_post_commit_cue_line,
};
use recovery_prompt::{recovery_help_lines, recovery_warning_line};
#[cfg(test)]
use riotbox_core::view::jam::{CaptureHandoffReadinessView, CaptureTargetKindView};
use source_timing_panel::source_timing_lines;
use source_trust_summary::{
    energy_label, source_candidate_lines, source_confidence_lines, source_provenance_lines,
    source_timing_clock_compact, source_timing_clock_line, source_timing_help_line,
    source_timing_performance_rail_line, source_timing_readiness_line, source_timing_warning_line,
    source_warning_lines, trust_summary,
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
