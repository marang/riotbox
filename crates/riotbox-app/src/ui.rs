// Keep the TUI surface textually included in one module for now so the split is
// behavior-preserving and does not mix file-size cleanup with visibility churn.
include!("ui/types_state.rs");
include!("ui/shell_render_root.rs");
include!("ui/jam_perform_layout.rs");
include!("ui/screen_bodies_footer_start.rs");
include!("ui/recovery_prompt.rs");
include!("ui/footer_help_perform_lines.rs");
include!("ui/jam_landed_warnings_source.rs");
include!("ui/diagnostics_mc202_w30_logs.rs");
include!("ui/w30_capture_source_helpers.rs");
include!("ui/scene_timing_rail.rs");
include!("ui/first_run_capture.rs");
include!("ui/source_timing_panel.rs");
include!("ui/capture_log_source_lists.rs");
include!("ui/source_trust_summary.rs");

#[cfg(test)]
mod tests;
