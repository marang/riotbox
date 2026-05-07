fn probe_candidate_drift_reports(
    input: &SourceTimingProbeBpmCandidateInput,
    bpm: f32,
    confidence: Confidence,
) -> Vec<TimingDriftReport> {
    let seconds_per_beat = 60.0 / bpm.max(1.0);
    let seconds_per_bar = seconds_per_beat * f32::from(input.meter.beats_per_bar.max(1));
    if seconds_per_beat <= 0.0 || seconds_per_bar <= 0.0 {
        return Vec::new();
    }

    [4_u32, 8_u32, 16_u32, 32_u32]
        .into_iter()
        .filter_map(|window_bars| {
            let window_seconds = seconds_per_bar * window_bars as f32;
            if input.duration_seconds + seconds_per_beat < window_seconds {
                return None;
            }
            drift_report_for_window(input, seconds_per_beat, window_bars, window_seconds, confidence)
        })
        .collect()
}

fn drift_report_for_window(
    input: &SourceTimingProbeBpmCandidateInput,
    seconds_per_beat: f32,
    window_bars: u32,
    window_seconds: f32,
    confidence: Confidence,
) -> Option<TimingDriftReport> {
    let residuals = normalized_onset_times(input)
        .into_iter()
        .filter(|time_seconds| *time_seconds <= window_seconds)
        .map(|time_seconds| signed_distance_to_nearest_grid(time_seconds, seconds_per_beat))
        .collect::<Vec<_>>();
    if residuals.len() < 4 {
        return None;
    }

    let max_drift_seconds = residuals
        .iter()
        .map(|residual| residual.abs())
        .fold(0.0_f32, f32::max);
    let mean_abs_drift_seconds =
        residuals.iter().map(|residual| residual.abs()).sum::<f32>() / residuals.len() as f32;
    let end_drift_seconds = residuals.last().copied().unwrap_or(0.0);

    Some(TimingDriftReport {
        window_bars,
        max_drift_ms: max_drift_seconds * 1_000.0,
        mean_abs_drift_ms: mean_abs_drift_seconds * 1_000.0,
        end_drift_ms: end_drift_seconds * 1_000.0,
        confidence: drift_confidence(confidence, max_drift_seconds),
    })
}

fn signed_distance_to_nearest_grid(time_seconds: f32, period_seconds: f32) -> f32 {
    let lower_grid = (time_seconds / period_seconds).floor() * period_seconds;
    let upper_grid = lower_grid + period_seconds;
    let lower_distance = time_seconds - lower_grid;
    let upper_distance = time_seconds - upper_grid;
    if lower_distance.abs() <= upper_distance.abs() {
        lower_distance
    } else {
        upper_distance
    }
}

fn drift_confidence(confidence: Confidence, max_drift_seconds: f32) -> Confidence {
    let penalty = (max_drift_seconds * 1_000.0 / 70.0).clamp(0.0, 1.0);
    confidence * (1.0 - penalty)
}
