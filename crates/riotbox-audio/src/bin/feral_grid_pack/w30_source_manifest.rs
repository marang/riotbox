//! JSON presentation only; the policy owners never depend on these serializer types.

use super::config::PATTERN_ORIGIN_SOURCE_DERIVED;
use super::w30_slice_choice::W30SourceSliceChoiceProof;
use super::w30_source_accent_dynamics::W30SourceAccentDynamicsProof;
use super::w30_source_chop::{W30SourceChopProfile, W30SourceLoopClosureProof};
use super::w30_source_trigger_policy::W30SourceTriggerVariationProof;
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct ManifestW30SourceChopProfile {
    source_window_rms: f32,
    selected_rms_before_gain: f32,
    preview_rms: f32,
    preview_peak_abs: f32,
    body_rms: f32,
    tail_rms: f32,
    tail_to_body_rms_ratio: f32,
    selected_start_frame: u64,
    selected_frame_count: usize,
    gain: f32,
    reason: &'static str,
}

#[derive(Serialize)]
pub(super) struct ManifestW30SourceLoopClosureProof {
    passed: bool,
    selected_frame_count: usize,
    preview_rms: f32,
    edge_delta_abs: f32,
    max_allowed_edge_delta_abs: f32,
    edge_abs_max: f32,
    max_allowed_edge_abs: f32,
    source_contains_selection: bool,
    reason: &'static str,
}

#[derive(Serialize)]
pub(super) struct ManifestW30SourceTriggerVariationProof {
    pattern_origin: &'static str,
    applied: bool,
    grid_subdivision: u32,
    trigger_count: u32,
    beat_anchor_trigger_count: u32,
    offbeat_trigger_count: u32,
    skipped_beat_anchor_count: u32,
    distinct_bar_pattern_count: usize,
    max_quantized_offset_ms: f32,
    max_allowed_quantized_offset_ms: f32,
    reason: &'static str,
}

pub(super) fn manifest_w30_source_chop_profile(
    profile: W30SourceChopProfile,
) -> ManifestW30SourceChopProfile {
    ManifestW30SourceChopProfile {
        source_window_rms: profile.source_window_rms,
        selected_rms_before_gain: profile.selected_rms_before_gain,
        preview_rms: profile.preview_rms,
        preview_peak_abs: profile.preview_peak_abs,
        body_rms: profile.body_rms,
        tail_rms: profile.tail_rms,
        tail_to_body_rms_ratio: profile.tail_to_body_rms_ratio,
        selected_start_frame: profile.selected_start_frame,
        selected_frame_count: profile.selected_frame_count,
        gain: profile.gain,
        reason: profile.reason,
    }
}

pub(super) fn manifest_w30_source_loop_closure_proof(
    proof: W30SourceLoopClosureProof,
) -> ManifestW30SourceLoopClosureProof {
    ManifestW30SourceLoopClosureProof {
        passed: proof.passed,
        selected_frame_count: proof.selected_frame_count,
        preview_rms: proof.preview_rms,
        edge_delta_abs: proof.edge_delta_abs,
        max_allowed_edge_delta_abs: proof.max_allowed_edge_delta_abs,
        edge_abs_max: proof.edge_abs_max,
        max_allowed_edge_abs: proof.max_allowed_edge_abs,
        source_contains_selection: proof.source_contains_selection,
        reason: proof.reason,
    }
}

pub(super) fn manifest_w30_source_trigger_variation_proof(
    proof: W30SourceTriggerVariationProof,
) -> ManifestW30SourceTriggerVariationProof {
    ManifestW30SourceTriggerVariationProof {
        pattern_origin: PATTERN_ORIGIN_SOURCE_DERIVED,
        applied: proof.applied,
        grid_subdivision: proof.grid_subdivision,
        trigger_count: proof.trigger_count,
        beat_anchor_trigger_count: proof.beat_anchor_trigger_count,
        offbeat_trigger_count: proof.offbeat_trigger_count,
        skipped_beat_anchor_count: proof.skipped_beat_anchor_count,
        distinct_bar_pattern_count: proof.distinct_bar_pattern_count,
        max_quantized_offset_ms: proof.max_quantized_offset_ms,
        max_allowed_quantized_offset_ms: proof.max_allowed_quantized_offset_ms,
        reason: proof.reason,
    }
}

#[derive(Serialize)]
pub(super) struct ManifestW30SourceSliceChoiceProof {
    applied: bool,
    candidate_count: usize,
    unique_source_offset_count: usize,
    selected_offset_span_samples: usize,
    min_selected_offset_samples: usize,
    max_selected_offset_samples: usize,
    reason: &'static str,
}

pub(super) fn manifest_w30_source_slice_choice_proof(
    proof: W30SourceSliceChoiceProof,
) -> ManifestW30SourceSliceChoiceProof {
    ManifestW30SourceSliceChoiceProof {
        applied: proof.applied,
        candidate_count: proof.candidate_count,
        unique_source_offset_count: proof.unique_source_offset_count,
        selected_offset_span_samples: proof.selected_offset_span_samples,
        min_selected_offset_samples: proof.min_selected_offset_samples,
        max_selected_offset_samples: proof.max_selected_offset_samples,
        reason: proof.reason,
    }
}

#[derive(Serialize)]
pub(super) struct ManifestW30SourceAccentDynamicsProof {
    pattern_origin: &'static str,
    applied: bool,
    trigger_count: u32,
    distinct_velocity_count: usize,
    min_velocity: f32,
    max_velocity: f32,
    velocity_span: f32,
    min_required_velocity_span: f32,
    source_energy_span: f32,
    reason: &'static str,
}

pub(super) fn manifest_w30_source_accent_dynamics_proof(
    proof: W30SourceAccentDynamicsProof,
) -> ManifestW30SourceAccentDynamicsProof {
    ManifestW30SourceAccentDynamicsProof {
        pattern_origin: PATTERN_ORIGIN_SOURCE_DERIVED,
        applied: proof.applied,
        trigger_count: proof.trigger_count,
        distinct_velocity_count: proof.distinct_velocity_count,
        min_velocity: proof.min_velocity,
        max_velocity: proof.max_velocity,
        velocity_span: proof.velocity_span,
        min_required_velocity_span: proof.min_required_velocity_span,
        source_energy_span: proof.source_energy_span,
        reason: proof.reason,
    }
}
