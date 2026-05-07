fn probe_candidate_beat_grid(
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

fn probe_candidate_bar_grid(
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
