#[path = "feral_grid_pack/args.rs"]
mod args;
#[path = "feral_grid_pack/grid_bpm_decision.rs"]
mod grid_bpm_decision;
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
use grid_bpm_decision::{
    GridBpmDecision, choose_grid_bpm, grid_bpm_decision_reason_label, grid_bpm_source_label,
    source_timing_bpm_agrees,
};
use source_timing_analysis::{SourceTimingAnalysisForManifest, source_timing_analysis_for_source};
use source_timing_groove_policy::{
    Tr909GrooveTimingPolicy, apply_tr909_groove_timing, tr909_groove_timing_policy,
};
use timing_readiness_manifest::{
    ManifestSourceTimingReadiness, confidence_result_label, downbeat_evidence_status_label,
    drift_status_label, manifest_source_timing_readiness, phrase_status_label,
    readiness_status_label,
};

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

use bar_variation_metrics::{BarVariationMetrics, bar_variation_metrics};
use config::{
    CHANNEL_COUNT, DEFAULT_BEATS_PER_BAR, MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO,
    MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO, MIN_LOW_BAND_RMS, MIN_SIGNAL_RMS,
    MIN_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO, PACK_ID, PATTERN_ORIGIN_PRIMITIVE_RENDERER,
    PATTERN_ORIGIN_SOURCE_DERIVED, SAMPLE_RATE,
};
#[cfg(test)]
use grid::frames_for_beats;
use grid::{Grid, frames_for_beat_position};
use signal_filter::one_pole_lowpass;
#[cfg(test)]
use source_grid_output_drift::{
    SOURCE_GRID_OUTPUT_MAX_PEAK_OFFSET_MS, source_grid_output_drift_metrics,
};
use source_grid_output_drift::{
    SOURCE_GRID_OUTPUT_MIN_HIT_RATIO, SourceGridOutputDriftMetrics, source_grid_alignment_report,
};
use spectral_energy_metrics::{SpectralEnergyMetrics, spectral_energy_metrics};

#[path = "feral_grid_pack/manifest.rs"]
mod manifest;
#[path = "feral_grid_pack/product_stem_contributions.rs"]
mod product_stem_contributions;

use manifest::write_manifest;
#[cfg(test)]
use product_stem_contributions::{
    PRODUCT_STEM_RECONSTRUCTION_RULE, PRODUCT_STEM_RECONSTRUCTION_SCHEMA,
};
use product_stem_contributions::{
    ProductStemContributionRender, ProductStemReconstructionReport,
    render_product_stem_contributions, validate_written_product_stem_reconstruction,
};

#[cfg(test)]
use riotbox_audio::listening_manifest::LISTENING_MANIFEST_SCHEMA_VERSION;

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

use w30_slice_choice::{W30SourceSliceChoiceProof, w30_source_slice_choice_plan};
use w30_source_accent_dynamics::{W30SourceAccentDynamicsProof, w30_source_accent_dynamics_proof};
use w30_source_chop::{
    W30SourceChopProfile, W30SourceLoopClosureProof, chop_articulation_metrics,
    source_chop_preview_from_interleaved, w30_source_loop_closure_proof,
};
use w30_source_events::W30SourceTriggerEvent;
use w30_source_manifest::{
    ManifestW30SourceAccentDynamicsProof, ManifestW30SourceChopProfile,
    ManifestW30SourceLoopClosureProof, ManifestW30SourceSliceChoiceProof,
    ManifestW30SourceTriggerVariationProof, manifest_w30_source_accent_dynamics_proof,
    manifest_w30_source_chop_profile, manifest_w30_source_loop_closure_proof,
    manifest_w30_source_slice_choice_proof, manifest_w30_source_trigger_variation_proof,
};
use w30_source_playback_profile::w30_source_playback_profile;
use w30_source_trigger_policy::{
    W30SourceTriggerVariationProof, is_beat_anchor, w30_source_trigger_events_with_slice_plan,
    w30_source_trigger_variation_proof,
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

use source_aware_tr909::{SourceAwareTr909Profile, derive_source_aware_tr909_profile};
#[cfg(test)]
use tr909_kick_pressure::{
    TR909_KICK_PRESSURE_MAX_PEAK_ABS, TR909_KICK_PRESSURE_MIN_LOW_BAND_RATIO,
    TR909_SOURCE_ACCENT_MIN_ACCENT_SPAN, TR909_SOURCE_ACCENT_MIN_DISTINCT_ACCENTS,
    TR909_SOURCE_EVIDENCE_ROLE_PROFILE_AND_ACCENT_DYNAMICS, render_tr909_source_support,
    render_tr909_source_support_legacy,
};
use tr909_kick_pressure::{
    Tr909KickPressureProof, Tr909SourceAccentDynamicsProof,
    render_tr909_source_support_with_pressure_and_accents,
};
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

use source_character_window_selection::{
    SourceCharacterWindowSelection, select_source_character_window, source_character_search_window,
};

// Remaining legacy owners consume explicit compatibility imports from the real modules.
include!("feral_grid_pack/pack_builder.rs");
include!("feral_grid_pack/tr909_rendered_drum_pressure.rs");
include!("feral_grid_pack/mc202_bass_pressure.rs");
include!("feral_grid_pack/mc202_low_body_policy.rs");
include!("feral_grid_pack/mix_policy.rs");
include!("feral_grid_pack/pack_text_outputs.rs");
include!("feral_grid_pack/render_stems.rs");
include!("feral_grid_pack/manifest_assertions.rs");
include!("feral_grid_pack/manifest_mc202_assertions.rs");
include!("feral_grid_pack/manifest_mix_assertions.rs");
include!("feral_grid_pack/tests.rs");
