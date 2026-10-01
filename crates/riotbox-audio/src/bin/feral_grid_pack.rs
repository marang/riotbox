use std::env;

use pack_builder::render_pack;

#[path = "feral_grid_pack/args.rs"]
mod args;
#[path = "feral_grid_pack/grid_bpm_decision.rs"]
mod grid_bpm_decision;
#[path = "feral_grid_pack/output_paths.rs"]
mod output_paths;
#[path = "qa_source_safety/mod.rs"]
mod qa_source_safety;
#[path = "feral_grid_pack/source_timing_analysis.rs"]
mod source_timing_analysis;
#[path = "feral_grid_pack/source_timing_groove_policy.rs"]
mod source_timing_groove_policy;
#[path = "feral_grid_pack/source_timing_policy_profile.rs"]
mod source_timing_policy_profile;
#[path = "feral_grid_pack/timing_evidence.rs"]
mod timing_evidence;
#[path = "feral_grid_pack/timing_readiness_manifest.rs"]
mod timing_readiness_manifest;

#[cfg(test)]
#[path = "feral_grid_pack/bpm_decision_tests.rs"]
mod bpm_decision_tests;

use args::{Args, print_help};

#[path = "feral_grid_pack/bar_variation_metrics.rs"]
mod bar_variation_metrics;
#[path = "feral_grid_pack/config.rs"]
mod config;
#[path = "feral_grid_pack/grid.rs"]
mod grid;
#[path = "feral_grid_pack/signal_filter.rs"]
mod signal_filter;
#[path = "feral_grid_pack/source_grid_output_drift/mod.rs"]
mod source_grid_output_drift;
#[path = "feral_grid_pack/spectral_energy_metrics.rs"]
mod spectral_energy_metrics;

#[cfg(test)]
#[path = "feral_grid_pack/verification_command_tests.rs"]
mod verification_command_tests;

#[path = "feral_grid_pack/manifest.rs"]
mod manifest;
#[path = "feral_grid_pack/product_stem_contributions.rs"]
mod product_stem_contributions;

#[path = "feral_grid_pack/sample_measurements.rs"]
mod sample_measurements;
#[path = "feral_grid_pack/w30_slice_choice.rs"]
mod w30_slice_choice;
#[path = "feral_grid_pack/w30_source_accent_dynamics.rs"]
mod w30_source_accent_dynamics;
#[path = "feral_grid_pack/w30_source_chop.rs"]
mod w30_source_chop;
#[path = "feral_grid_pack/w30_source_events.rs"]
mod w30_source_events;
#[path = "feral_grid_pack/w30_source_manifest.rs"]
mod w30_source_manifest;
#[path = "feral_grid_pack/w30_source_playback_profile.rs"]
mod w30_source_playback_profile;
#[path = "feral_grid_pack/w30_source_trigger_policy.rs"]
mod w30_source_trigger_policy;

#[cfg(test)]
#[path = "feral_grid_pack/w30_source_chop_tests.rs"]
mod w30_source_chop_tests;

#[path = "feral_grid_pack/source_aware_tr909.rs"]
mod source_aware_tr909;
#[path = "feral_grid_pack/tr909_kick_pressure.rs"]
mod tr909_kick_pressure;
#[path = "feral_grid_pack/tr909_source_manifest.rs"]
mod tr909_source_manifest;

#[cfg(test)]
#[path = "feral_grid_pack/tr909_source_grid_consumer_tests.rs"]
mod tr909_source_grid_consumer_tests;

#[path = "feral_grid_pack/source_character_window_selection.rs"]
mod source_character_window_selection;
#[cfg(test)]
#[path = "feral_grid_pack/source_character_window_selection_tests.rs"]
mod source_character_window_selection_tests;

#[path = "feral_grid_pack/mc202_bass_pressure.rs"]
mod mc202_bass_pressure;
#[path = "feral_grid_pack/mc202_low_body_policy.rs"]
mod mc202_low_body_policy;
#[path = "feral_grid_pack/mc202_source_contour.rs"]
mod mc202_source_contour;
#[path = "feral_grid_pack/mc202_source_manifest.rs"]
mod mc202_source_manifest;
#[path = "feral_grid_pack/mc202_source_phrase.rs"]
mod mc202_source_phrase;
#[path = "feral_grid_pack/render_measurements.rs"]
mod render_measurements;

#[path = "feral_grid_pack/mix_components.rs"]
mod mix_components;
#[path = "feral_grid_pack/mix_movement_evidence.rs"]
mod mix_movement_evidence;
#[path = "feral_grid_pack/mix_policy.rs"]
mod mix_policy;

// Binary composition only; consumers import their actual dependency owners.
#[path = "feral_grid_pack/pack_builder.rs"]
mod pack_builder;
#[path = "feral_grid_pack/pack_report.rs"]
mod pack_report;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(env::args().skip(1))?;
    if args.show_help {
        print_help();
        return Ok(());
    }

    render_pack(&args)?;
    println!("wrote {}", args.output_dir().display());
    Ok(())
}
#[path = "feral_grid_pack/tr909_rendered_drum_pressure.rs"]
mod tr909_rendered_drum_pressure;
#[cfg(test)]
#[path = "feral_grid_pack/tr909_rendered_drum_pressure_tests.rs"]
mod tr909_rendered_drum_pressure_tests;

#[path = "feral_grid_pack/artifact_io.rs"]
mod artifact_io;
#[cfg(test)]
#[path = "feral_grid_pack/manifest_assertions.rs"]
mod manifest_assertions;
#[cfg(test)]
#[path = "feral_grid_pack/manifest_mc202_assertions.rs"]
mod manifest_mc202_assertions;
#[cfg(test)]
#[path = "feral_grid_pack/manifest_mix_assertions.rs"]
mod manifest_mix_assertions;
#[path = "feral_grid_pack/pack_text_outputs.rs"]
mod pack_text_outputs;
#[path = "feral_grid_pack/pack_validation.rs"]
mod pack_validation;
#[cfg(test)]
#[path = "feral_grid_pack/synthetic_test_fixtures.rs"]
mod test_fixtures;
#[cfg(test)]
#[path = "feral_grid_pack/tests.rs"]
mod tests;
#[path = "feral_grid_pack/verification_command.rs"]
mod verification_command;
#[path = "feral_grid_pack/w30_trigger_render.rs"]
mod w30_trigger_render;
