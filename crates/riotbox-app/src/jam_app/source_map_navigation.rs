use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType, Quantization,
        TargetScope, UndoPolicy,
    },
    source_graph::{BarSpan, PhraseSpan, SourceGraph},
};

use super::JamAppState;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SourceMapNavigationIntent {
    PreviousBar,
    NextBar,
    PreviousPhrase,
    NextPhrase,
}

impl SourceMapNavigationIntent {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::PreviousBar => "previous bar",
            Self::NextBar => "next bar",
            Self::PreviousPhrase => "previous phrase",
            Self::NextPhrase => "next phrase",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceMapNavigationResult {
    Enqueued {
        target_position_beats: u64,
        target_label: String,
    },
    AlreadyPending,
    AlreadyAtBoundary {
        target_label: String,
    },
    Unavailable {
        reason: &'static str,
    },
}

impl JamAppState {
    pub fn queue_source_map_navigation(
        &mut self,
        intent: SourceMapNavigationIntent,
        requested_at: TimestampMs,
    ) -> SourceMapNavigationResult {
        if self
            .queue
            .pending_actions()
            .iter()
            .any(|action| action.command == ActionCommand::TransportSeek)
        {
            return SourceMapNavigationResult::AlreadyPending;
        }

        let Some(target) = source_map_navigation_target(
            self.source_graph.as_ref(),
            self.runtime.transport.position_beats,
            intent,
        ) else {
            return SourceMapNavigationResult::Unavailable {
                reason: "source map timing grid unavailable",
            };
        };

        let current_position = self.runtime.transport.position_beats.floor().max(0.0) as u64;
        if target.position_beats == current_position {
            return SourceMapNavigationResult::AlreadyAtBoundary {
                target_label: target.label,
            };
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::TransportSeek,
            Quantization::Immediate,
            ActionTarget {
                scope: Some(TargetScope::Session),
                object_id: Some("source-map-navigator".into()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Transport {
            position_beats: Some(target.position_beats),
        };
        draft.undo_policy = UndoPolicy::NotUndoable {
            reason: "source map navigation is explicit transport position selection".into(),
        };
        draft.explanation = Some(format!("source map {} to {}", intent.label(), target.label));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        SourceMapNavigationResult::Enqueued {
            target_position_beats: target.position_beats,
            target_label: target.label,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SourceMapNavigationTarget {
    position_beats: u64,
    label: String,
}

fn source_map_navigation_target(
    graph: Option<&SourceGraph>,
    position_beats: f64,
    intent: SourceMapNavigationIntent,
) -> Option<SourceMapNavigationTarget> {
    let graph = graph?;
    let beats_per_bar = source_map_beats_per_bar(graph)?;
    let bars = source_map_navigation_bars(graph);
    if bars.is_empty() {
        return None;
    }
    let current_bar = current_source_map_bar(graph, position_beats, beats_per_bar, &bars)?;

    match intent {
        SourceMapNavigationIntent::PreviousBar => {
            let target_bar = bars
                .iter()
                .rev()
                .find(|bar| bar.bar_index < current_bar)
                .or_else(|| bars.first())?;
            source_map_bar_target(graph, target_bar.bar_index, beats_per_bar)
        }
        SourceMapNavigationIntent::NextBar => {
            let target_bar = bars
                .iter()
                .find(|bar| bar.bar_index > current_bar)
                .or_else(|| bars.last())?;
            source_map_bar_target(graph, target_bar.bar_index, beats_per_bar)
        }
        SourceMapNavigationIntent::PreviousPhrase => {
            let phrases = source_map_navigation_phrases(graph, &bars);
            let current_phrase = current_source_map_phrase(current_bar, &phrases)?;
            let target_phrase = phrases
                .iter()
                .rev()
                .find(|phrase| phrase.start_bar < current_phrase.start_bar)
                .or_else(|| phrases.first())?;
            source_map_phrase_target(graph, target_phrase, beats_per_bar)
        }
        SourceMapNavigationIntent::NextPhrase => {
            let phrases = source_map_navigation_phrases(graph, &bars);
            let current_phrase = current_source_map_phrase(current_bar, &phrases)?;
            let target_phrase = phrases
                .iter()
                .find(|phrase| phrase.start_bar > current_phrase.start_bar)
                .or_else(|| phrases.last())?;
            source_map_phrase_target(graph, target_phrase, beats_per_bar)
        }
    }
}

fn source_map_beats_per_bar(graph: &SourceGraph) -> Option<u64> {
    graph
        .timing
        .primary_hypothesis()
        .map(|hypothesis| u64::from(hypothesis.meter.beats_per_bar))
        .or_else(|| {
            graph
                .timing
                .meter_hint
                .map(|meter| u64::from(meter.beats_per_bar))
        })
        .filter(|beats| *beats > 0)
}

fn source_map_navigation_bars(graph: &SourceGraph) -> Vec<BarSpan> {
    let mut bars = graph
        .timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.bar_grid.clone())
        .filter(|bars| !bars.is_empty())
        .unwrap_or_else(|| graph.timing.bar_grid.clone());
    bars.sort_by_key(|bar| bar.bar_index);
    bars
}

fn source_map_navigation_phrases(graph: &SourceGraph, bars: &[BarSpan]) -> Vec<PhraseSpan> {
    let mut phrases = graph
        .timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.phrase_grid.clone())
        .filter(|phrases| !phrases.is_empty())
        .unwrap_or_else(|| graph.timing.phrase_grid.clone());
    if phrases.is_empty() {
        phrases = phrases_from_bar_markers(bars);
    }
    phrases.sort_by_key(|phrase| (phrase.start_bar, phrase.end_bar, phrase.phrase_index));
    phrases
}

fn phrases_from_bar_markers(bars: &[BarSpan]) -> Vec<PhraseSpan> {
    let mut phrases = Vec::new();
    let mut active: Option<PhraseSpan> = None;
    for bar in bars.iter().filter(|bar| bar.phrase_index.is_some()) {
        let phrase_index = bar.phrase_index.expect("filtered phrase index");
        match active.as_mut() {
            Some(phrase) if phrase.phrase_index == phrase_index => {
                phrase.end_bar = bar.bar_index;
            }
            Some(_) => {
                if let Some(phrase) = active.replace(PhraseSpan {
                    phrase_index,
                    start_bar: bar.bar_index,
                    end_bar: bar.bar_index,
                    confidence: bar.downbeat_confidence,
                }) {
                    phrases.push(phrase);
                }
            }
            None => {
                active = Some(PhraseSpan {
                    phrase_index,
                    start_bar: bar.bar_index,
                    end_bar: bar.bar_index,
                    confidence: bar.downbeat_confidence,
                });
            }
        }
    }
    if let Some(phrase) = active {
        phrases.push(phrase);
    }
    phrases
}

fn current_source_map_bar(
    graph: &SourceGraph,
    position_beats: f64,
    beats_per_bar: u64,
    bars: &[BarSpan],
) -> Option<u32> {
    let raw_bar = if let Some(primary) = graph.timing.primary_hypothesis()
        && !primary.bar_grid.is_empty()
    {
        primary.transport_grid_position(position_beats)?.bar_index
    } else {
        ((position_beats.floor().max(0.0) as u64) / beats_per_bar).saturating_add(1)
    };
    let first = bars.first().map_or(1, |bar| bar.bar_index);
    let last = bars.last().map_or(first, |bar| bar.bar_index);
    Some(
        u32::try_from(raw_bar)
            .unwrap_or(u32::MAX)
            .clamp(first, last),
    )
}

fn current_source_map_phrase(current_bar: u32, phrases: &[PhraseSpan]) -> Option<&PhraseSpan> {
    phrases
        .iter()
        .find(|phrase| current_bar >= phrase.start_bar && current_bar <= phrase.end_bar)
        .or_else(|| {
            phrases
                .iter()
                .rev()
                .find(|phrase| phrase.start_bar <= current_bar)
        })
        .or_else(|| phrases.first())
}

fn source_map_bar_target(
    graph: &SourceGraph,
    bar_index: u32,
    beats_per_bar: u64,
) -> Option<SourceMapNavigationTarget> {
    let position_beats = source_map_bar_start_cursor(graph, bar_index, beats_per_bar)?;
    Some(SourceMapNavigationTarget {
        position_beats,
        label: format!("bar {bar_index}"),
    })
}

fn source_map_phrase_target(
    graph: &SourceGraph,
    phrase: &PhraseSpan,
    beats_per_bar: u64,
) -> Option<SourceMapNavigationTarget> {
    let position_beats = source_map_bar_start_cursor(graph, phrase.start_bar, beats_per_bar)?;
    Some(SourceMapNavigationTarget {
        position_beats,
        label: format!("phrase {} bar {}", phrase.phrase_index, phrase.start_bar),
    })
}

fn source_map_bar_start_cursor(
    graph: &SourceGraph,
    bar_index: u32,
    beats_per_bar: u64,
) -> Option<u64> {
    if let Some(primary) = graph.timing.primary_hypothesis()
        && !primary.bar_grid.is_empty()
    {
        return primary.bar_start_beat_cursor(u64::from(bar_index));
    }
    Some(u64::from(bar_index.saturating_sub(1)) * beats_per_bar)
}
