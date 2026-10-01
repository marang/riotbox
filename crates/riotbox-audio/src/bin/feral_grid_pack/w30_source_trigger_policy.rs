//! Existing beat-anchor trigger policy and quantization evidence for offline QA.

use super::grid::Grid;
use super::w30_slice_choice::W30SourceSliceChoicePlan;
use super::w30_source_accent_dynamics::w30_source_accent_features;
use super::w30_source_events::W30SourceTriggerEvent;
use riotbox_audio::w30::W30PreviewSampleWindow;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct W30SourceTriggerVariationProof {
    pub(super) applied: bool,
    pub(super) grid_subdivision: u32,
    pub(super) trigger_count: u32,
    pub(super) beat_anchor_trigger_count: u32,
    pub(super) offbeat_trigger_count: u32,
    pub(super) skipped_beat_anchor_count: u32,
    pub(super) distinct_bar_pattern_count: usize,
    pub(super) max_quantized_offset_ms: f32,
    pub(super) max_allowed_quantized_offset_ms: f32,
    pub(super) reason: &'static str,
}

pub(super) const W30_SOURCE_TRIGGER_GRID_SUBDIVISION: u32 = 2;

const W30_SOURCE_TRIGGER_MAX_QUANTIZED_OFFSET_MS: f32 = 0.01;

pub(super) fn w30_source_trigger_events_with_slice_plan(
    grid: &Grid,
    source_window_preview: &W30PreviewSampleWindow,
    slice_plan: &W30SourceSliceChoicePlan,
) -> Vec<W30SourceTriggerEvent> {
    let mut events = Vec::with_capacity(grid.total_beats as usize + grid.bars as usize);

    for bar in 0..grid.bars {
        let bar_start = bar.saturating_mul(grid.beats_per_bar) as f32;
        for beat_offset in 0..grid.beats_per_bar {
            let source_stride = (bar as usize)
                .saturating_mul(grid.beats_per_bar as usize)
                .saturating_add(beat_offset as usize);
            let source_offset_samples = slice_plan.offset_for_stride(source_stride);
            let accent = w30_source_accent_features(source_window_preview, source_offset_samples);
            events.push(W30SourceTriggerEvent {
                beat_position: bar_start + beat_offset as f32,
                velocity: accent.velocity,
                source_energy_score: accent.source_energy_score,
                source_offset_samples,
            });
        }
    }

    events
}

pub(super) fn w30_source_trigger_variation_proof(
    grid: &Grid,
    events: &[W30SourceTriggerEvent],
) -> W30SourceTriggerVariationProof {
    let beat_anchor_trigger_count = events
        .iter()
        .filter(|event| is_beat_anchor(event.beat_position))
        .count() as u32;
    let offbeat_trigger_count = events.len() as u32 - beat_anchor_trigger_count;
    let skipped_beat_anchor_count = grid
        .total_beats
        .saturating_sub(beat_anchor_trigger_count.min(grid.total_beats));
    let distinct_bar_pattern_count = events
        .iter()
        .map(|event| event.source_offset_samples)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let max_quantized_offset_ms = events
        .iter()
        .map(|event| quantized_offset_ms(event.beat_position, grid.bpm))
        .fold(0.0_f32, f32::max);
    let applied = beat_anchor_trigger_count == grid.total_beats
        && skipped_beat_anchor_count == 0
        && distinct_bar_pattern_count > 1
        && max_quantized_offset_ms <= W30_SOURCE_TRIGGER_MAX_QUANTIZED_OFFSET_MS;

    W30SourceTriggerVariationProof {
        applied,
        grid_subdivision: W30_SOURCE_TRIGGER_GRID_SUBDIVISION,
        trigger_count: events.len() as u32,
        beat_anchor_trigger_count,
        offbeat_trigger_count,
        skipped_beat_anchor_count,
        distinct_bar_pattern_count,
        max_quantized_offset_ms,
        max_allowed_quantized_offset_ms: W30_SOURCE_TRIGGER_MAX_QUANTIZED_OFFSET_MS,
        reason: if applied {
            "source_grid_locked_beat_anchor_triggers"
        } else {
            "source_trigger_variation_not_applied"
        },
    }
}

pub(super) fn is_beat_anchor(beat_position: f32) -> bool {
    (beat_position - beat_position.round()).abs() <= f32::EPSILON
}

fn quantized_offset_ms(beat_position: f32, bpm: f32) -> f32 {
    let subdivision = W30_SOURCE_TRIGGER_GRID_SUBDIVISION as f32;
    let quantized = (beat_position * subdivision).round() / subdivision;
    (beat_position - quantized).abs() * 60_000.0 / bpm.max(f32::EPSILON)
}
