use riotbox_core::{
    session::Mc202SourcePhraseExpressionState,
    source_graph::{PhraseSpan, SourceGraph, SourceTimingAnchor, SourceTimingAnchorType},
};

use super::super::{Mc202SourcePhraseFingerprint, feature_step};

#[derive(Copy, Clone)]
pub(super) struct SourcePhraseGrooveMap {
    pub pressure_step: usize,
    pub answer_step: usize,
    pub callback_step: usize,
    pub hook_safe_step: usize,
    pub fill_pickup_step: usize,
}

impl SourcePhraseGrooveMap {
    pub(super) fn from_graph(
        graph: &SourceGraph,
        phrase_slot: &PhraseSpan,
        expression: &Mc202SourcePhraseExpressionState,
        fingerprint: Mc202SourcePhraseFingerprint,
    ) -> Self {
        let fallback_pressure =
            feature_step(expression.bass_pressure, fingerprint.step_rotation, 0);
        let fallback_answer = feature_step(
            expression.offbeat_answer_space.max(0.25),
            fingerprint.accent_step,
            3,
        );
        let fallback_callback = feature_step(
            expression.transient_backbeat.max(expression.stab_bite),
            fingerprint.accent_step,
            2,
        );
        let fallback_fill = feature_step(expression.phrase_density, fingerprint.accent_step, 14);
        let pressure_step = strongest_anchor_step(
            graph,
            phrase_slot,
            &[
                SourceTimingAnchorType::Kick,
                SourceTimingAnchorType::TransientCluster,
            ],
        )
        .unwrap_or(fallback_pressure);
        let backbeat_step = strongest_anchor_step(
            graph,
            phrase_slot,
            &[
                SourceTimingAnchorType::Snare,
                SourceTimingAnchorType::Backbeat,
            ],
        )
        .unwrap_or_else(|| (pressure_step + 8) % 16);
        let answer_step =
            strongest_anchor_step(graph, phrase_slot, &[SourceTimingAnchorType::AnswerSlot])
                .unwrap_or_else(|| {
                    avoid_steps(fallback_answer, &[pressure_step, backbeat_step, 0, 8])
                });
        let callback_step = strongest_anchor_step(
            graph,
            phrase_slot,
            &[
                SourceTimingAnchorType::Snare,
                SourceTimingAnchorType::Backbeat,
                SourceTimingAnchorType::TransientCluster,
            ],
        )
        .map_or(fallback_callback, |step| {
            avoid_steps((step + 1) % 16, &[pressure_step])
        });
        let hook_safe_step = avoid_steps(
            feature_step(expression.hook_restraint, fingerprint.accent_step, 11),
            &[pressure_step, backbeat_step, answer_step, 0, 8],
        );
        let fill_pickup_step =
            strongest_anchor_step(graph, phrase_slot, &[SourceTimingAnchorType::Fill])
                .map_or(fallback_fill, |step| {
                    avoid_steps(step, &[pressure_step, backbeat_step])
                });

        Self {
            pressure_step: avoid_steps(pressure_step, &[backbeat_step]),
            answer_step,
            callback_step,
            hook_safe_step,
            fill_pickup_step,
        }
    }

    pub(super) fn secondary_pressure_step(self) -> usize {
        avoid_steps(
            (self.pressure_step + 8) % 16,
            &[self.answer_step, self.callback_step],
        )
    }

    pub(super) fn pressure_movement_step(self) -> usize {
        avoid_steps(
            (self.pressure_step + 12) % 16,
            &[
                self.secondary_pressure_step(),
                self.answer_step,
                self.callback_step,
            ],
        )
    }

    pub(super) fn backbeat_answer_step(self) -> usize {
        avoid_steps(
            (self.callback_step + 2) % 16,
            &[self.pressure_step, self.answer_step],
        )
    }

    pub(super) fn callback_tail_step(self) -> usize {
        avoid_steps(
            (self.callback_step + 5) % 16,
            &[self.pressure_step, self.answer_step],
        )
    }

    pub(super) fn answer_tail_step(self) -> usize {
        avoid_steps(
            (self.answer_step + 3) % 16,
            &[self.pressure_step, self.callback_step],
        )
    }

    pub(super) fn provenance_refs(self) -> Vec<String> {
        vec![
            format!("groove_pressure_step:{}", self.pressure_step),
            format!("groove_answer_step:{}", self.answer_step),
            format!("groove_callback_step:{}", self.callback_step),
            format!("groove_hook_safe_step:{}", self.hook_safe_step),
            format!("groove_fill_pickup_step:{}", self.fill_pickup_step),
            format!(
                "groove_pressure_movement_step:{}",
                self.pressure_movement_step()
            ),
        ]
    }
}

fn strongest_anchor_step(
    graph: &SourceGraph,
    phrase_slot: &PhraseSpan,
    anchor_types: &[SourceTimingAnchorType],
) -> Option<usize> {
    let hypothesis = graph.timing.primary_hypothesis()?;
    let beats_per_bar = u32::from(hypothesis.meter.beats_per_bar.max(1));

    hypothesis
        .anchors
        .iter()
        .filter(|anchor| {
            anchor_types.contains(&anchor.anchor_type)
                && anchor
                    .bar_index
                    .is_some_and(|bar| bar >= phrase_slot.start_bar && bar <= phrase_slot.end_bar)
        })
        .max_by(|left, right| {
            (left.strength * left.confidence).total_cmp(&(right.strength * right.confidence))
        })
        .and_then(|anchor| source_anchor_step(anchor, phrase_slot, beats_per_bar))
}

fn source_anchor_step(
    anchor: &SourceTimingAnchor,
    phrase_slot: &PhraseSpan,
    beats_per_bar: u32,
) -> Option<usize> {
    if let Some(beat_index) = anchor.beat_index {
        let phrase_start_beat = phrase_slot.start_bar.saturating_mul(beats_per_bar);
        let relative_beat = beat_index.saturating_sub(phrase_start_beat);
        return Some(((relative_beat * 4) as usize) % 16);
    }
    anchor.bar_index.map(|bar| {
        let relative_bar = bar.saturating_sub(phrase_slot.start_bar);
        ((relative_bar * 16) as usize) % 16
    })
}

fn avoid_steps(mut step: usize, blocked: &[usize]) -> usize {
    step %= 16;
    for offset in [0, 1, 15, 2, 14, 3, 13] {
        let candidate = (step + offset) % 16;
        if !blocked.contains(&candidate) {
            return candidate;
        }
    }
    step
}
