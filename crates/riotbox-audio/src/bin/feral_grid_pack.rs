use std::env;

use riotbox_audio::runtime::MasterBusLimiterReport;

use pack_builder::render_pack;
use pack_report::PackReport;

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
use grid_bpm_decision::{GridBpmDecision, grid_bpm_decision_reason_label, grid_bpm_source_label};
use source_timing_analysis::SourceTimingAnalysisForManifest;
use source_timing_groove_policy::Tr909GrooveTimingPolicy;
use timing_readiness_manifest::{ManifestSourceTimingReadiness, manifest_source_timing_readiness};

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

use bar_variation_metrics::BarVariationMetrics;
use config::{
    CHANNEL_COUNT, MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO,
    MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO, MIN_LOW_BAND_RMS, MIN_SIGNAL_RMS,
    MIN_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO, PACK_ID, SAMPLE_RATE,
};
use grid::Grid;
use source_grid_output_drift::SourceGridOutputDriftMetrics;
use spectral_energy_metrics::SpectralEnergyMetrics;

#[path = "feral_grid_pack/manifest.rs"]
mod manifest;
#[path = "feral_grid_pack/product_stem_contributions.rs"]
mod product_stem_contributions;

use product_stem_contributions::ProductStemReconstructionReport;

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

use w30_source_manifest::{
    ManifestW30SourceAccentDynamicsProof, ManifestW30SourceChopProfile,
    ManifestW30SourceLoopClosureProof, ManifestW30SourceSliceChoiceProof,
    ManifestW30SourceTriggerVariationProof, manifest_w30_source_accent_dynamics_proof,
    manifest_w30_source_chop_profile, manifest_w30_source_loop_closure_proof,
    manifest_w30_source_slice_choice_proof, manifest_w30_source_trigger_variation_proof,
};

#[path = "feral_grid_pack/source_aware_tr909.rs"]
mod source_aware_tr909;
#[path = "feral_grid_pack/tr909_kick_pressure.rs"]
mod tr909_kick_pressure;
#[path = "feral_grid_pack/tr909_source_manifest.rs"]
mod tr909_source_manifest;

#[cfg(test)]
#[path = "feral_grid_pack/tr909_source_grid_consumer_tests.rs"]
mod tr909_source_grid_consumer_tests;

use tr909_source_manifest::{
    ManifestTr909KickPressureProof, ManifestTr909SourceAccentDynamicsProof,
    ManifestTr909SourceProfile, manifest_tr909_kick_pressure_proof,
    manifest_tr909_source_accent_dynamics_proof, manifest_tr909_source_profile,
};

#[path = "feral_grid_pack/source_character_window_selection.rs"]
mod source_character_window_selection;
#[cfg(test)]
#[path = "feral_grid_pack/source_character_window_selection_tests.rs"]
mod source_character_window_selection_tests;

use source_character_window_selection::SourceCharacterWindowSelection;

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

use mc202_source_manifest::{
    ManifestMc202BassPressureProof, ManifestMc202SourceContourProof,
    manifest_mc202_bass_pressure_proof, manifest_mc202_source_contour_proof,
};
use render_measurements::RenderMetrics;

#[path = "feral_grid_pack/mix_components.rs"]
mod mix_components;
#[path = "feral_grid_pack/mix_movement_evidence.rs"]
mod mix_movement_evidence;
#[path = "feral_grid_pack/mix_policy.rs"]
mod mix_policy;

#[cfg(test)]
use mix_components::render_mix_with_master_bus_report;
use mix_movement_evidence::AllLaneMixMovementProof;

// Remaining legacy owners consume explicit compatibility imports from the real modules.
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
#[path = "feral_grid_pack/pack_text_outputs.rs"]
mod pack_text_outputs;
#[path = "feral_grid_pack/pack_validation.rs"]
mod pack_validation;
#[path = "feral_grid_pack/verification_command.rs"]
mod verification_command;
#[path = "feral_grid_pack/w30_trigger_render.rs"]
mod w30_trigger_render;
#[cfg(test)]
use verification_command::verification_command;
#[cfg(test)]
use w30_trigger_render::{
    render_w30_source_chop, render_w30_source_chop_legacy, render_w30_source_chop_with_variation,
};
#[cfg(test)]
#[path = "feral_grid_pack/manifest_assertions.rs"]
mod manifest_assertions;
#[cfg(test)]
#[path = "feral_grid_pack/manifest_mc202_assertions.rs"]
mod manifest_mc202_assertions;
#[cfg(test)]
#[path = "feral_grid_pack/manifest_mix_assertions.rs"]
mod manifest_mix_assertions;
#[cfg(test)]
#[path = "feral_grid_pack/synthetic_test_fixtures.rs"]
mod test_fixtures;
#[cfg(test)]
#[path = "feral_grid_pack/tests.rs"]
mod tests;
