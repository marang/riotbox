use super::mc202_bass_pressure::{
    MC202_BASS_PRESSURE_MAX_BAR_SIMILARITY, MC202_BASS_PRESSURE_MIN_LOW_TO_MID_ENERGY_RATIO,
    Mc202BassPressureProof, Mc202SourceContourProof,
};
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct ManifestMc202BassPressureProof {
    pattern_origin: &'static str,
    applied: bool,
    pressure_role: &'static str,
    source_expression_render_plan_applied: bool,
    source_expression_role: &'static str,
    source_failure_fallback: bool,
    mode: &'static str,
    phrase_shape: &'static str,
    note_budget: &'static str,
    phrase_variation_applied: bool,
    distinct_bar_profile_count: usize,
    bar_similarity: f32,
    identical_bar_run_length: usize,
    max_bar_similarity: f32,
    touch: f32,
    music_bus_level: f32,
    low_body_emphasis: f32,
    pressure_reinforcement_gain: f32,
    signal_rms: f32,
    low_band_rms: f32,
    low_to_mid_energy_ratio: f32,
    low_to_high_energy_ratio: f32,
    min_low_to_mid_energy_ratio: f32,
    active_sample_ratio: f32,
    peak_abs: f32,
    reason: &'static str,
}

#[derive(Serialize)]
pub(super) struct ManifestMc202SourceContourProof {
    pattern_origin: &'static str,
    applied: bool,
    contour_hint: &'static str,
    note_budget: &'static str,
    touch_boost: f32,
    music_bus_boost: f32,
    low_band_energy_ratio: f32,
    mid_band_energy_ratio: f32,
    high_band_energy_ratio: f32,
    event_density_per_bar: f32,
    source_contour_delta_rms: f32,
    min_required_delta_rms: f32,
    reason: &'static str,
}

pub(super) const MC202_PATTERN_ORIGIN_SOURCE_DERIVED_CONTOUR: &str = "source_derived_contour";

pub(super) fn manifest_mc202_bass_pressure_proof(
    proof: Mc202BassPressureProof,
) -> ManifestMc202BassPressureProof {
    ManifestMc202BassPressureProof {
        pattern_origin: proof.pattern_origin.label(),
        applied: proof.applied,
        pressure_role: proof.pressure_role,
        source_expression_render_plan_applied: proof.source_expression_render_plan_applied,
        source_expression_role: proof.source_expression_role,
        source_failure_fallback: proof.source_failure_fallback,
        mode: proof.mode.label(),
        phrase_shape: proof.phrase_shape.label(),
        note_budget: proof.note_budget.label(),
        phrase_variation_applied: proof.phrase_variation_applied,
        distinct_bar_profile_count: proof.distinct_bar_profile_count,
        bar_similarity: proof.bar_similarity,
        identical_bar_run_length: proof.identical_bar_run_length,
        max_bar_similarity: MC202_BASS_PRESSURE_MAX_BAR_SIMILARITY,
        touch: proof.touch,
        music_bus_level: proof.music_bus_level,
        low_body_emphasis: proof.low_body_emphasis,
        pressure_reinforcement_gain: proof.pressure_reinforcement_gain,
        signal_rms: proof.signal_rms,
        low_band_rms: proof.low_band_rms,
        low_to_mid_energy_ratio: proof.low_to_mid_energy_ratio,
        low_to_high_energy_ratio: proof.low_to_high_energy_ratio,
        min_low_to_mid_energy_ratio: MC202_BASS_PRESSURE_MIN_LOW_TO_MID_ENERGY_RATIO,
        active_sample_ratio: proof.active_sample_ratio,
        peak_abs: proof.peak_abs,
        reason: proof.reason,
    }
}

pub(super) fn manifest_mc202_source_contour_proof(
    proof: Mc202SourceContourProof,
) -> ManifestMc202SourceContourProof {
    ManifestMc202SourceContourProof {
        pattern_origin: MC202_PATTERN_ORIGIN_SOURCE_DERIVED_CONTOUR,
        applied: proof.applied,
        contour_hint: proof.contour_hint.label(),
        note_budget: proof.note_budget.label(),
        touch_boost: proof.touch_boost,
        music_bus_boost: proof.music_bus_boost,
        low_band_energy_ratio: proof.low_band_energy_ratio,
        mid_band_energy_ratio: proof.mid_band_energy_ratio,
        high_band_energy_ratio: proof.high_band_energy_ratio,
        event_density_per_bar: proof.event_density_per_bar,
        source_contour_delta_rms: proof.source_contour_delta_rms,
        min_required_delta_rms: proof.min_required_delta_rms,
        reason: proof.reason,
    }
}
