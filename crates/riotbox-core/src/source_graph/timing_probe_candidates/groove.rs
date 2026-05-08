const GROOVE_MIN_ONSET_COUNT: usize = 4;
const GROOVE_MAX_RESIDUAL_MS: f32 = 80.0;
const GROOVE_MIN_HIT_RATIO: f32 = 0.75;
const GROOVE_MIN_WEIGHTED_OFFSET_MS: f32 = 1.0;

fn probe_candidate_groove_residuals(
    input: &SourceTimingProbeBpmCandidateInput,
    bpm: f32,
    confidence: Confidence,
) -> Vec<GrooveResidual> {
    let onsets = normalized_onset_evidence(input);
    if onsets.len() < GROOVE_MIN_ONSET_COUNT || !bpm.is_finite() || bpm <= 0.0 {
        return Vec::new();
    }

    [
        GrooveSubdivision::Eighth,
        GrooveSubdivision::Sixteenth,
        GrooveSubdivision::Triplet,
        GrooveSubdivision::ThirtySecond,
    ]
    .into_iter()
    .filter_map(|subdivision| groove_residual_for_subdivision(&onsets, bpm, confidence, subdivision))
    .collect()
}

fn groove_residual_for_subdivision(
    onsets: &[NormalizedOnsetEvidence],
    bpm: f32,
    confidence: Confidence,
    subdivision: GrooveSubdivision,
) -> Option<GrooveResidual> {
    let period_seconds = groove_subdivision_period_seconds(bpm, subdivision)?;
    let max_residual_ms = max_groove_residual_ms(period_seconds);
    let mut hit_count = 0_usize;
    let mut weighted_offset_ms = 0.0_f32;
    let mut weight_sum = 0.0_f32;

    for onset in onsets {
        let residual_ms = nearest_grid_residual_seconds(onset.time_seconds, period_seconds) * 1000.0;
        if residual_ms.abs() > max_residual_ms {
            continue;
        }
        let weight = onset.strength.max(0.01);
        hit_count += 1;
        weighted_offset_ms += residual_ms * weight;
        weight_sum += weight;
    }

    let hit_ratio = hit_count as f32 / onsets.len() as f32;
    if hit_count < GROOVE_MIN_ONSET_COUNT || hit_ratio < GROOVE_MIN_HIT_RATIO || weight_sum <= 0.0 {
        return None;
    }
    let offset_ms = weighted_offset_ms / weight_sum;
    if offset_ms.abs() < GROOVE_MIN_WEIGHTED_OFFSET_MS {
        return None;
    }

    Some(GrooveResidual {
        subdivision,
        offset_ms,
        confidence: (confidence * hit_ratio).clamp(0.0, 1.0),
    })
}

fn groove_subdivision_period_seconds(
    bpm: f32,
    subdivision: GrooveSubdivision,
) -> Option<f32> {
    let beat_seconds = 60.0 / bpm.max(1.0);
    let division = match subdivision {
        GrooveSubdivision::Eighth => 2.0,
        GrooveSubdivision::Triplet => 3.0,
        GrooveSubdivision::Sixteenth => 4.0,
        GrooveSubdivision::ThirtySecond => 8.0,
    };
    let period = beat_seconds / division;
    period.is_finite().then_some(period).filter(|period| *period > 0.0)
}

fn nearest_grid_residual_seconds(time_seconds: f32, period_seconds: f32) -> f32 {
    let nearest = (time_seconds / period_seconds).round() * period_seconds;
    time_seconds - nearest
}

fn max_groove_residual_ms(period_seconds: f32) -> f32 {
    (period_seconds * 450.0).min(GROOVE_MAX_RESIDUAL_MS)
}
