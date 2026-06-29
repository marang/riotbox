#[path = "feral_grid_pack/manifest.rs"]
mod manifest;

use manifest::write_manifest;

#[cfg(test)]
use riotbox_audio::listening_manifest::LISTENING_MANIFEST_SCHEMA_VERSION;

// Most shards are still textual includes; manifest ownership is now a real module.
include!("feral_grid_pack/pack_builder.rs");
include!("feral_grid_pack/bar_variation_metrics.rs");
include!("feral_grid_pack/spectral_energy_metrics.rs");
include!("feral_grid_pack/source_aware_tr909.rs");
include!("feral_grid_pack/tr909_kick_pressure.rs");
include!("feral_grid_pack/tr909_rendered_drum_pressure.rs");
include!("feral_grid_pack/mc202_bass_pressure.rs");
include!("feral_grid_pack/mc202_low_body_policy.rs");
include!("feral_grid_pack/w30_source_chop.rs");
include!("feral_grid_pack/source_character_window_selection.rs");
include!("feral_grid_pack/w30_slice_choice.rs");
include!("feral_grid_pack/w30_source_accent_dynamics.rs");
include!("feral_grid_pack/mix_policy.rs");
include!("feral_grid_pack/grid_bpm_decision.rs");
include!("feral_grid_pack/source_timing_policy_profile.rs");
include!("feral_grid_pack/timing_readiness_manifest.rs");
include!("feral_grid_pack/source_timing_groove_policy.rs");
include!("feral_grid_pack/source_grid_output_drift.rs");
include!("feral_grid_pack/pack_text_outputs.rs");
include!("feral_grid_pack/render_stems.rs");
include!("feral_grid_pack/manifest_assertions.rs");
include!("feral_grid_pack/manifest_mc202_assertions.rs");
include!("feral_grid_pack/manifest_mix_assertions.rs");
include!("feral_grid_pack/tests.rs");
include!("feral_grid_pack/bpm_decision_tests.rs");
include!("feral_grid_pack/w30_source_chop_tests.rs");
include!("feral_grid_pack/source_grid_output_drift_tests.rs");
include!("feral_grid_pack/tr909_source_grid_consumer_tests.rs");
