use crate::source_graph::timing_probe_candidates::MIN_STABLE_DOWNBEAT_PHASE_SCORE;
use crate::source_graph::timing_probe_candidates::drift::has_high_drift;
use crate::source_graph::{
    BarSpan, BeatPoint, Confidence, MeterHint, PhraseSpan, TimingDriftReport,
};

pub(super) fn probe_candidate_phrase_grid(
    bar_grid: &[BarSpan],
    downbeat_score: f32,
    drift: &[TimingDriftReport],
) -> Vec<PhraseSpan> {
    const PHRASE_BARS: u32 = 4;
    const MIN_PHRASE_COUNT: u32 = 2;

    if downbeat_score < MIN_STABLE_DOWNBEAT_PHASE_SCORE || has_high_drift(drift) {
        return Vec::new();
    }
    let bar_count = u32::try_from(bar_grid.len()).unwrap_or(u32::MAX);
    if bar_count < PHRASE_BARS * MIN_PHRASE_COUNT {
        return Vec::new();
    }

    (0..(bar_count / PHRASE_BARS))
        .map(|phrase_index| PhraseSpan {
            phrase_index: phrase_index + 1,
            start_bar: phrase_index * PHRASE_BARS + 1,
            end_bar: (phrase_index + 1) * PHRASE_BARS,
            confidence: downbeat_score.clamp(0.0, 1.0),
        })
        .collect()
}

pub(super) fn probe_candidate_beat_grid(
    duration_seconds: f32,
    bpm: f32,
    confidence: Confidence,
) -> Vec<BeatPoint> {
    let seconds_per_beat = 60.0 / bpm.max(1.0);
    let mut beat_grid = Vec::new();
    let mut time_seconds = 0.0_f32;
    while time_seconds <= duration_seconds.max(0.0) {
        beat_grid.push(BeatPoint {
            beat_index: u32::try_from(beat_grid.len() + 1).unwrap_or(u32::MAX),
            time_seconds,
            confidence,
        });
        time_seconds += seconds_per_beat;
    }
    beat_grid
}

pub(super) fn probe_candidate_bar_grid(
    duration_seconds: f32,
    bpm: f32,
    confidence: Confidence,
    meter: MeterHint,
    downbeat_offset_beats: u8,
    downbeat_score: f32,
) -> Vec<BarSpan> {
    let seconds_per_beat = 60.0 / bpm.max(1.0);
    let seconds_per_bar = seconds_per_beat * f32::from(meter.beats_per_bar.max(1));
    let mut bar_grid = Vec::new();
    let mut start_seconds = f32::from(downbeat_offset_beats) * seconds_per_beat;
    while start_seconds < duration_seconds.max(0.0) {
        bar_grid.push(BarSpan {
            bar_index: u32::try_from(bar_grid.len() + 1).unwrap_or(u32::MAX),
            start_seconds,
            end_seconds: (start_seconds + seconds_per_bar).min(duration_seconds.max(0.0)),
            downbeat_confidence: confidence * downbeat_score.clamp(0.0, 1.0),
            phrase_index: None,
        });
        start_seconds += seconds_per_bar;
    }
    bar_grid
}
