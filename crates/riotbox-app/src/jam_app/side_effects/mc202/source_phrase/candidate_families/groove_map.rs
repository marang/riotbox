use riotbox_core::{
    session::Mc202SourcePhraseExpressionState,
    source_graph::{
        PhraseSpan, SourceGraph, SourceTimingAnchor, SourceTimingAnchorType, TimingHypothesis,
    },
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
        .and_then(|anchor| source_anchor_step(anchor, phrase_slot, hypothesis))
}

fn source_anchor_step(
    anchor: &SourceTimingAnchor,
    phrase_slot: &PhraseSpan,
    hypothesis: &TimingHypothesis,
) -> Option<usize> {
    if let Some(beat_index) = anchor.beat_index {
        let beats_per_bar = u32::from(hypothesis.meter.beats_per_bar.max(1));
        let phrase_start_beat = if hypothesis.bar_grid.is_empty() {
            phrase_slot
                .start_bar
                .saturating_sub(1)
                .saturating_mul(beats_per_bar)
                .saturating_add(1)
        } else {
            hypothesis
                .bar_start_beat_point(phrase_slot.start_bar)?
                .beat_index
        };
        let anchor_beat = hypothesis
            .beat_grid
            .iter()
            .find(|beat| beat.beat_index == beat_index);
        if !hypothesis.bar_grid.is_empty() && anchor_beat.is_none() {
            return None;
        }
        let relative_beat = beat_index.checked_sub(phrase_start_beat)?;
        let coarse_step = i32::try_from(relative_beat.saturating_mul(4)).unwrap_or(i32::MAX);
        if hypothesis.bpm.is_finite()
            && hypothesis.bpm > 0.0
            && anchor.time_seconds.is_finite()
            && let Some(beat) = anchor_beat.filter(|beat| beat.time_seconds.is_finite())
        {
            let seconds_per_step = (60.0 / hypothesis.bpm) / 4.0;
            if seconds_per_step.is_finite() && seconds_per_step > 0.0 {
                let subbeat_step =
                    ((anchor.time_seconds - beat.time_seconds) / seconds_per_step).round() as i32;
                return Some((coarse_step + subbeat_step).rem_euclid(16) as usize);
            }
        }
        return Some(coarse_step.rem_euclid(16) as usize);
    }
    let bar_index = anchor.bar_index?;
    if hypothesis.bar_grid.is_empty() {
        let relative_bar = bar_index.saturating_sub(phrase_slot.start_bar);
        return Some(((relative_bar * 16) as usize) % 16);
    }

    let phrase_start_beat = hypothesis
        .bar_start_beat_point(phrase_slot.start_bar)?
        .beat_index;
    let anchor_bar_start_beat = hypothesis.bar_start_beat_point(bar_index)?.beat_index;
    let relative_beat = anchor_bar_start_beat.checked_sub(phrase_start_beat)?;
    Some((relative_beat.saturating_mul(4) as usize) % 16)
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

#[cfg(test)]
mod tests {
    use super::source_anchor_step;
    use riotbox_core::source_graph::{
        BarSpan, BeatPoint, MeterHint, PhraseSpan, SourceTimingAnchor, SourceTimingAnchorType,
        TimingHypothesis, TimingHypothesisKind, TimingQuality,
    };

    fn anchor_at_beat(beat_index: u32) -> SourceTimingAnchor {
        anchor_at_beat_and_time(beat_index, 0.0)
    }

    fn anchor_at_beat_and_time(beat_index: u32, time_seconds: f32) -> SourceTimingAnchor {
        SourceTimingAnchor {
            anchor_id: format!("beat-{beat_index}"),
            anchor_type: SourceTimingAnchorType::Kick,
            time_seconds,
            bar_index: Some(8),
            beat_index: Some(beat_index),
            confidence: 1.0,
            strength: 1.0,
            tags: Vec::new(),
        }
    }

    fn timing_hypothesis(beat_grid: Vec<BeatPoint>, bar_grid: Vec<BarSpan>) -> TimingHypothesis {
        TimingHypothesis {
            hypothesis_id: "primary".into(),
            kind: TimingHypothesisKind::Primary,
            bpm: 120.0,
            meter: MeterHint {
                beats_per_bar: 4,
                beat_unit: 4,
            },
            confidence: 1.0,
            score: 1.0,
            beat_grid,
            bar_grid,
            phrase_grid: Vec::new(),
            anchors: Vec::new(),
            drift: Vec::new(),
            groove: Vec::new(),
            quality: TimingQuality::High,
            warnings: Vec::new(),
            provenance: Vec::new(),
        }
    }

    #[test]
    fn phrase_start_bar_eight_maps_one_based_beats_to_sixteenth_steps() {
        let phrase = PhraseSpan {
            phrase_index: 1,
            start_bar: 8,
            end_bar: 8,
            confidence: 1.0,
        };
        let hypothesis = timing_hypothesis(Vec::new(), Vec::new());

        for (beat_index, expected_step) in [(29, 0), (30, 4), (31, 8), (32, 12)] {
            assert_eq!(
                source_anchor_step(&anchor_at_beat(beat_index), &phrase, &hypothesis),
                Some(expected_step)
            );
        }
    }

    #[test]
    fn selected_primary_downbeat_phase_maps_bar_eight_beat_thirty_to_step_zero() {
        let phrase = PhraseSpan {
            phrase_index: 1,
            start_bar: 8,
            end_bar: 8,
            confidence: 1.0,
        };
        let beat_grid = (30..=33)
            .map(|beat_index| BeatPoint {
                beat_index,
                time_seconds: 14.5 + (beat_index - 30) as f32 * 0.5,
                confidence: 1.0,
            })
            .collect();
        let bar_grid = vec![BarSpan {
            bar_index: 8,
            start_seconds: 14.5,
            end_seconds: 16.5,
            downbeat_confidence: 1.0,
            phrase_index: Some(1),
        }];
        let hypothesis = timing_hypothesis(beat_grid, bar_grid);

        assert_eq!(
            source_anchor_step(&anchor_at_beat_and_time(30, 14.5), &phrase, &hypothesis),
            Some(0)
        );
        assert_eq!(
            source_anchor_step(&anchor_at_beat_and_time(31, 15.0), &phrase, &hypothesis),
            Some(4)
        );
    }

    #[test]
    fn present_bar_grid_does_not_fall_back_when_beat_grid_is_missing() {
        let phrase = PhraseSpan {
            phrase_index: 1,
            start_bar: 8,
            end_bar: 8,
            confidence: 1.0,
        };
        let hypothesis = timing_hypothesis(
            Vec::new(),
            vec![BarSpan {
                bar_index: 8,
                start_seconds: 14.5,
                end_seconds: 16.5,
                downbeat_confidence: 1.0,
                phrase_index: Some(1),
            }],
        );

        assert_eq!(
            source_anchor_step(&anchor_at_beat(30), &phrase, &hypothesis),
            None
        );
    }

    #[test]
    fn zero_or_pre_phrase_beats_do_not_collapse_onto_phrase_start() {
        let phrase = PhraseSpan {
            phrase_index: 1,
            start_bar: 8,
            end_bar: 8,
            confidence: 1.0,
        };
        let hypothesis = timing_hypothesis(Vec::new(), Vec::new());

        assert_eq!(
            source_anchor_step(&anchor_at_beat(0), &phrase, &hypothesis),
            None
        );
        assert_eq!(
            source_anchor_step(&anchor_at_beat(28), &phrase, &hypothesis),
            None
        );
    }
}
