//! Shell presentation and input routing; Core/Session retain product truth.

mod action_log;
mod capture_cues;
mod export_inspect;
mod first_run_capture;
mod footer_cues;
mod footer_renderer;
mod gestures;
mod help;
mod jam_layout;
mod lane_diagnostics;
mod lane_perform;
pub mod perform_risk_cue_contract;
mod performer_cues;
mod recovery_prompt;
mod render;
mod scene_commit_cues;
mod scene_labels;
mod scene_timing;
mod screens;
mod shell_labels;
mod shell_state;
mod source_details;
mod source_timing_panel;
mod source_trust_summary;
mod stem_package_inspect;
mod styles;
mod w30_cue_labels;
mod w30_operations;
mod w30_preview_labels;
mod w30_resample_labels;
mod w30_slice_pool;
mod warnings;

pub use render::{render_jam_shell, render_jam_shell_snapshot};
pub use shell_state::{JamShellState, JamViewMode, ShellKeyOutcome, ShellLaunchMode, ShellScreen};

#[cfg(test)]
mod tests;
