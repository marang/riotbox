//! TR-909 QA JSON presentation; policies never depend on these serializer types.

use super::source_aware_tr909::SourceAwareTr909Profile;
use super::tr909_kick_pressure::{Tr909KickPressureProof, Tr909SourceAccentDynamicsProof};
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct ManifestTr909SourceProfile {
    signal_rms: f32,
    low_band_rms: f32,
    onset_count: usize,
    event_density_per_bar: f32,
    low_band_energy_ratio: f32,
    mid_band_energy_ratio: f32,
    high_band_energy_ratio: f32,
    support_profile: &'static str,
    support_context: &'static str,
    pattern_adoption: &'static str,
    phrase_variation: &'static str,
    drum_bus_level: f32,
    slam_intensity: f32,
    reason: &'static str,
}

pub(super) fn manifest_tr909_source_profile(
    profile: SourceAwareTr909Profile,
) -> ManifestTr909SourceProfile {
    ManifestTr909SourceProfile {
        signal_rms: profile.signal_rms,
        low_band_rms: profile.low_band_rms,
        onset_count: profile.onset_count,
        event_density_per_bar: profile.event_density_per_bar,
        low_band_energy_ratio: profile.low_band_energy_ratio,
        mid_band_energy_ratio: profile.mid_band_energy_ratio,
        high_band_energy_ratio: profile.high_band_energy_ratio,
        support_profile: profile.support_profile.label(),
        support_context: profile.support_context.label(),
        pattern_adoption: profile.pattern_adoption.label(),
        phrase_variation: profile.phrase_variation.label(),
        drum_bus_level: profile.drum_bus_level,
        slam_intensity: profile.slam_intensity,
        reason: profile.reason,
    }
}

#[derive(Serialize)]
pub(super) struct ManifestTr909KickPressureProof {
    pattern_origin: &'static str,
    source_evidence_role: &'static str,
    source_profile_reason: &'static str,
    applied: bool,
    anchor_count: usize,
    pressure_gain: f32,
    pre_low_band_rms: f32,
    post_low_band_rms: f32,
    low_band_rms_delta: f32,
    low_band_rms_ratio: f32,
    post_peak_abs: f32,
    reason: &'static str,
}

#[derive(Serialize)]
pub(super) struct ManifestTr909SourceAccentDynamicsProof {
    pattern_origin: &'static str,
    applied: bool,
    anchor_count: usize,
    distinct_accent_count: usize,
    min_accent: f32,
    max_accent: f32,
    accent_span: f32,
    min_required_accent_span: f32,
    source_energy_span: f32,
    reason: &'static str,
}

pub(super) fn manifest_tr909_kick_pressure_proof(
    proof: Tr909KickPressureProof,
) -> ManifestTr909KickPressureProof {
    ManifestTr909KickPressureProof {
        pattern_origin: proof.pattern_origin,
        source_evidence_role: proof.source_evidence_role,
        source_profile_reason: proof.source_profile_reason,
        applied: proof.applied,
        anchor_count: proof.anchor_count,
        pressure_gain: proof.pressure_gain,
        pre_low_band_rms: proof.pre_low_band_rms,
        post_low_band_rms: proof.post_low_band_rms,
        low_band_rms_delta: proof.low_band_rms_delta,
        low_band_rms_ratio: proof.low_band_rms_ratio,
        post_peak_abs: proof.post_peak_abs,
        reason: proof.reason,
    }
}

pub(super) fn manifest_tr909_source_accent_dynamics_proof(
    proof: Tr909SourceAccentDynamicsProof,
) -> ManifestTr909SourceAccentDynamicsProof {
    ManifestTr909SourceAccentDynamicsProof {
        pattern_origin: proof.pattern_origin,
        applied: proof.applied,
        anchor_count: proof.anchor_count,
        distinct_accent_count: proof.distinct_accent_count,
        min_accent: proof.min_accent,
        max_accent: proof.max_accent,
        accent_span: proof.accent_span,
        min_required_accent_span: proof.min_required_accent_span,
        source_energy_span: proof.source_energy_span,
        reason: proof.reason,
    }
}
