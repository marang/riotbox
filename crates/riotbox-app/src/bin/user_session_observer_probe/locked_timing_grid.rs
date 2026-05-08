use riotbox_core::source_graph::{BarSpan, BeatPoint, MeterHint, PhraseSpan, SourceGraph};

pub(super) fn attach_locked_timing_grid(graph: &mut SourceGraph, bpm: f32) {
    let beat_seconds = 60.0 / bpm;
    graph.timing.meter_hint = Some(MeterHint {
        beats_per_bar: 4,
        beat_unit: 4,
    });
    graph.timing.beat_grid = (0..16)
        .map(|beat_index| BeatPoint {
            beat_index,
            time_seconds: beat_index as f32 * beat_seconds,
            confidence: 0.92,
        })
        .collect();
    graph.timing.bar_grid = (0..4)
        .map(|bar_index| {
            let start_beat = bar_index * 4;
            BarSpan {
                bar_index,
                start_seconds: start_beat as f32 * beat_seconds,
                end_seconds: (start_beat + 4) as f32 * beat_seconds,
                downbeat_confidence: 0.92,
                phrase_index: Some(0),
            }
        })
        .collect();
    graph.timing.phrase_grid = vec![PhraseSpan {
        phrase_index: 0,
        start_bar: 0,
        end_bar: 4,
        confidence: 0.92,
    }];
}
