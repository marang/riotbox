fn probe_candidate_phrase_grid(
    bar_grid: &[BarSpan],
    downbeat_score: f32,
    drift: &[TimingDriftReport],
) -> Vec<PhraseSpan> {
    const PHRASE_BARS: u32 = 4;
    const MIN_PHRASE_COUNT: u32 = 2;

    if downbeat_score < 0.30 || has_high_drift(drift) {
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

fn has_high_drift(drift: &[TimingDriftReport]) -> bool {
    drift
        .iter()
        .any(|drift| drift.max_drift_ms > 70.0 || drift.end_drift_ms.abs() > 70.0)
}
