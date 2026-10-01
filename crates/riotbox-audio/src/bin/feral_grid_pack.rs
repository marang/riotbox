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

use bar_variation_metrics::{BarVariationMetrics, bar_variation_metrics};
use config::{
    CHANNEL_COUNT, DEFAULT_BARS, DEFAULT_BEATS_PER_BAR, DEFAULT_BPM, DEFAULT_DATE,
    DEFAULT_SOURCE_START_SECONDS, DEFAULT_SOURCE_WINDOW_SECONDS,
    MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO, MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
    MIN_BARS, MIN_LOW_BAND_RMS, MIN_SIGNAL_RMS, MIN_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO, PACK_ID,
    PATTERN_ORIGIN_PRIMITIVE_RENDERER, PATTERN_ORIGIN_SOURCE_DERIVED, SAMPLE_RATE,
    SOURCE_TIMING_BPM_MATCH_TOLERANCE,
};
use grid::Grid;
#[cfg(test)]
use grid::frames_for_beats;
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

// Remaining legacy owners consume explicit compatibility imports from the real modules.
include!("feral_grid_pack/pack_builder.rs");
include!("feral_grid_pack/source_aware_tr909.rs");
include!("feral_grid_pack/tr909_kick_pressure.rs");
include!("feral_grid_pack/tr909_rendered_drum_pressure.rs");
include!("feral_grid_pack/mc202_bass_pressure.rs");
include!("feral_grid_pack/mc202_low_body_policy.rs");
include!("feral_grid_pack/w30_source_chop.rs");
include!("feral_grid_pack/source_character_window_selection.rs");
include!("feral_grid_pack/w30_slice_choice.rs");
include!("feral_grid_pack/w30_source_accent_dynamics.rs");
include!("feral_grid_pack/w30_source_playback_profile.rs");
include!("feral_grid_pack/mix_policy.rs");
include!("feral_grid_pack/grid_bpm_decision.rs");
include!("feral_grid_pack/source_timing_policy_profile.rs");
include!("feral_grid_pack/timing_readiness_manifest.rs");
include!("feral_grid_pack/source_timing_groove_policy.rs");
include!("feral_grid_pack/pack_text_outputs.rs");
include!("feral_grid_pack/render_stems.rs");
include!("feral_grid_pack/manifest_assertions.rs");
include!("feral_grid_pack/manifest_mc202_assertions.rs");
include!("feral_grid_pack/manifest_mix_assertions.rs");
include!("feral_grid_pack/tests.rs");
include!("feral_grid_pack/bpm_decision_tests.rs");
include!("feral_grid_pack/w30_source_chop_tests.rs");
include!("feral_grid_pack/tr909_source_grid_consumer_tests.rs");
