#[derive(Clone, Copy, Debug, PartialEq)]
struct BeatPeriodScore {
    bpm: f32,
    period_seconds: f32,
    score: f32,
    matched_onset_ratio: f32,
    median_distance_ratio: f32,
}

fn beat_period_scores(
    input: &SourceTimingProbeBpmCandidateInput,
    policy: SourceTimingProbeBpmCandidatePolicy,
) -> Vec<BeatPeriodScore> {
    if input.duration_seconds <= 0.0 {
        return Vec::new();
    }

    let onset_times = normalized_onset_times(input);
    if onset_times.len() < policy.min_onset_count {
        return Vec::new();
    }

    let Some(reference_period) = median_adjacent_onset_delta(&onset_times) else {
        return Vec::new();
    };

    let mut periods = candidate_periods(&onset_times, policy);
    if periods.is_empty() {
        return Vec::new();
    }

    periods.sort_by(f32::total_cmp);
    periods.dedup_by(|left, right| (*left - *right).abs() <= 0.002);

    let mut scores = periods
        .into_iter()
        .filter_map(|period_seconds| {
            scored_period(&onset_times, period_seconds, reference_period, policy)
        })
        .collect::<Vec<_>>();
    scores.sort_by(|left, right| {
        period_score_order(left, right).then_with(|| left.bpm.total_cmp(&right.bpm))
    });
    scores
}

#[must_use]
pub fn source_timing_probe_beat_evidence_report(
    input: &SourceTimingProbeBpmCandidateInput,
    policy: SourceTimingProbeBpmCandidatePolicy,
) -> SourceTimingProbeBeatEvidenceReport {
    let scores = beat_period_scores(input, policy);
    let primary = scores.first().copied();
    let alternate_candidate_count = ambiguous_beat_period_scores(&scores, policy).count();
    let status = match primary {
        None => SourceTimingProbeBeatEvidenceStatus::Unavailable,
        Some(score) if score.score < policy.min_beat_period_score => {
            SourceTimingProbeBeatEvidenceStatus::Weak
        }
        Some(_) if alternate_candidate_count > 0 => SourceTimingProbeBeatEvidenceStatus::Ambiguous,
        Some(_) => SourceTimingProbeBeatEvidenceStatus::Stable,
    };

    SourceTimingProbeBeatEvidenceReport {
        schema: "riotbox.source_timing_probe_beat_evidence.v1",
        schema_version: 1,
        source_id: input.source_id.clone(),
        onset_count: normalized_onset_times(input).len(),
        candidate_count: scores.len(),
        primary_bpm: primary.map(|score| score.bpm),
        primary_period_seconds: primary.map(|score| score.period_seconds),
        primary_score: primary.map(|score| score.score),
        primary_matched_onset_ratio: primary.map(|score| score.matched_onset_ratio),
        primary_median_distance_ratio: primary.map(|score| score.median_distance_ratio),
        alternate_candidate_count,
        status,
    }
}

fn period_score_order(left: &BeatPeriodScore, right: &BeatPeriodScore) -> std::cmp::Ordering {
    // Bucket near-equal scores with a deterministic key instead of pairwise
    // fuzzy equality; `sort_by` requires this comparator to be transitive.
    period_score_bucket(right.score)
        .cmp(&period_score_bucket(left.score))
        .then_with(|| {
            left.median_distance_ratio
                .total_cmp(&right.median_distance_ratio)
        })
        .then_with(|| right.matched_onset_ratio.total_cmp(&left.matched_onset_ratio))
        .then_with(|| right.score.total_cmp(&left.score))
}

fn period_score_bucket(score: f32) -> i32 {
    const SCORE_TIE_BUCKET: f32 = 0.001;

    if !score.is_finite() {
        return i32::MIN;
    }

    (score / SCORE_TIE_BUCKET).round() as i32
}

fn ambiguous_beat_period_scores(
    scores: &[BeatPeriodScore],
    policy: SourceTimingProbeBpmCandidatePolicy,
) -> impl Iterator<Item = BeatPeriodScore> + '_ {
    let best_score = scores.first().map_or(0.0, |score| score.score);
    scores.iter().copied().skip(1).filter(move |score| {
        score.score >= policy.min_beat_period_score
            && best_score - score.score <= policy.beat_period_ambiguity_margin
    })
}

fn candidate_periods(
    onset_times: &[f32],
    policy: SourceTimingProbeBpmCandidatePolicy,
) -> Vec<f32> {
    let bounded_onsets = onset_times.iter().copied().take(128).collect::<Vec<_>>();
    let mut periods = Vec::new();
    for (left_index, left) in bounded_onsets.iter().enumerate() {
        for right in bounded_onsets.iter().skip(left_index + 1) {
            let delta = right - left;
            if !delta.is_finite() || delta < 0.05 {
                continue;
            }

            for subdivision in 1..=4 {
                if let Some(period) = normalize_period(delta / subdivision as f32, policy) {
                    periods.push(period);
                }
            }
        }
    }
    periods
}

fn scored_period(
    onset_times: &[f32],
    period_seconds: f32,
    reference_period: f32,
    policy: SourceTimingProbeBpmCandidatePolicy,
) -> Option<BeatPeriodScore> {
    let bpm = 60.0 / period_seconds;
    if bpm < policy.min_bpm || bpm > policy.max_bpm || period_seconds <= 0.0 {
        return None;
    }

    let tolerance_seconds = beat_period_tolerance(period_seconds);
    let mut best_matched = 0_usize;
    let mut best_mean_distance_ratio = 1.0_f32;
    for phase_seconds in onset_times
        .iter()
        .map(|time_seconds| time_seconds.rem_euclid(period_seconds))
    {
        let distances = onset_times
            .iter()
            .map(|time_seconds| {
                distance_to_repeating_beat(*time_seconds, phase_seconds, period_seconds)
            })
            .collect::<Vec<_>>();
        let matched = distances
            .iter()
            .filter(|distance| **distance <= tolerance_seconds)
            .count();
        let mean_distance_ratio = distances
            .iter()
            .map(|distance| (distance / tolerance_seconds).min(1.0))
            .sum::<f32>()
            / distances.len().max(1) as f32;

        if matched > best_matched
            || (matched == best_matched && mean_distance_ratio < best_mean_distance_ratio)
        {
            best_matched = matched;
            best_mean_distance_ratio = mean_distance_ratio;
        }
    }

    let matched_onset_ratio = best_matched as f32 / onset_times.len() as f32;
    let median_distance_ratio = if reference_period > 0.0 {
        ((period_seconds - reference_period).abs() / reference_period).min(1.0)
    } else {
        1.0
    };
    let alignment_score = (1.0 - best_mean_distance_ratio).clamp(0.0, 1.0);
    let score = (matched_onset_ratio * 0.85 + alignment_score * 0.15).clamp(0.0, 1.0);

    Some(BeatPeriodScore {
        bpm,
        period_seconds,
        score,
        matched_onset_ratio,
        median_distance_ratio,
    })
}

fn median_adjacent_onset_delta(onset_times: &[f32]) -> Option<f32> {
    let mut deltas = onset_times
        .windows(2)
        .filter_map(|times| {
            let delta = times[1] - times[0];
            (delta.is_finite() && delta >= 0.05).then_some(delta)
        })
        .collect::<Vec<_>>();
    if deltas.is_empty() {
        return None;
    }

    deltas.sort_by(f32::total_cmp);
    Some(deltas[deltas.len() / 2])
}

fn normalize_period(
    period_seconds: f32,
    policy: SourceTimingProbeBpmCandidatePolicy,
) -> Option<f32> {
    if !period_seconds.is_finite() || period_seconds <= 0.0 {
        return None;
    }

    let mut normalized = period_seconds;
    let mut bpm = 60.0 / normalized;
    while bpm < policy.min_bpm {
        normalized /= 2.0;
        bpm = 60.0 / normalized;
    }
    while bpm > policy.max_bpm {
        normalized *= 2.0;
        bpm = 60.0 / normalized;
    }

    (bpm >= policy.min_bpm && bpm <= policy.max_bpm).then_some(normalized)
}

fn beat_period_tolerance(period_seconds: f32) -> f32 {
    (period_seconds * 0.18).clamp(0.02, 0.08)
}

fn distance_to_repeating_beat(
    time_seconds: f32,
    phase_seconds: f32,
    period_seconds: f32,
) -> f32 {
    let position = (time_seconds - phase_seconds).rem_euclid(period_seconds);
    position.min(period_seconds - position)
}
