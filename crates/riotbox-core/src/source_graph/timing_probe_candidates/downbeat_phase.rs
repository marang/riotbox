#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct DownbeatPhaseScore {
    offset_beats: u8,
    score: f32,
}

const MIN_STABLE_DOWNBEAT_PHASE_SCORE: f32 = 0.30;

fn downbeat_phase_scores(
    input: &SourceTimingProbeBpmCandidateInput,
    bpm: f32,
) -> Vec<DownbeatPhaseScore> {
    let onsets = normalized_onset_times_and_strengths(input);
    let beats_per_bar = input.meter.beats_per_bar.max(1);
    let seconds_per_beat = 60.0 / bpm.max(1.0);
    let seconds_per_bar = seconds_per_beat * f32::from(beats_per_bar);
    if onsets.is_empty() || seconds_per_bar <= 0.0 {
        return vec![DownbeatPhaseScore::default()];
    }
    let total_strength = onsets
        .iter()
        .map(|(_, strength)| strength.max(0.0))
        .sum::<f32>()
        .max(f32::EPSILON);

    let tolerance_seconds = (seconds_per_beat * 0.2).clamp(0.02, 0.08);
    let mut scores = (0..beats_per_bar)
        .map(|offset_beats| {
            let phase_seconds = f32::from(offset_beats) * seconds_per_beat;
            let matching_strength = onsets
                .iter()
                .filter_map(|(time_seconds, strength)| {
                    (distance_to_repeating_phase(*time_seconds, phase_seconds, seconds_per_bar)
                        <= tolerance_seconds)
                        .then_some(strength.max(0.0))
                })
                .sum::<f32>();
            DownbeatPhaseScore {
                offset_beats,
                score: matching_strength / total_strength,
            }
        })
        .collect::<Vec<_>>();
    scores.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.offset_beats.cmp(&right.offset_beats))
    });
    scores
}

#[must_use]
pub fn source_timing_probe_downbeat_evidence_report(
    input: &SourceTimingProbeBpmCandidateInput,
    bpm: f32,
    policy: SourceTimingProbeBpmCandidatePolicy,
) -> SourceTimingProbeDownbeatEvidenceReport {
    let onsets = normalized_onset_times_and_strengths(input);
    let phases = if onsets.is_empty() || !bpm.is_finite() || bpm <= 0.0 {
        Vec::new()
    } else {
        downbeat_phase_scores(input, bpm)
    };
    let primary = phases.first().copied();
    let alternate_phase_count = ambiguous_downbeat_phases(&phases, policy).count();
    let status = match primary {
        None => SourceTimingProbeDownbeatEvidenceStatus::Unavailable,
        Some(phase) if phase.score < MIN_STABLE_DOWNBEAT_PHASE_SCORE => {
            SourceTimingProbeDownbeatEvidenceStatus::Weak
        }
        Some(_) if alternate_phase_count > 0 => SourceTimingProbeDownbeatEvidenceStatus::Ambiguous,
        Some(_) => SourceTimingProbeDownbeatEvidenceStatus::Stable,
    };

    SourceTimingProbeDownbeatEvidenceReport {
        schema: "riotbox.source_timing_probe_downbeat_evidence.v1",
        schema_version: 1,
        source_id: input.source_id.clone(),
        bpm,
        phase_count: phases.len(),
        primary_offset_beats: primary.map(|phase| phase.offset_beats),
        primary_score: primary.map(|phase| phase.score),
        alternate_phase_count,
        status,
    }
}

fn best_downbeat_phase(input: &SourceTimingProbeBpmCandidateInput, bpm: f32) -> DownbeatPhaseScore {
    downbeat_phase_scores(input, bpm)
        .first()
        .copied()
        .unwrap_or_default()
}

fn ambiguous_downbeat_phases(
    phases: &[DownbeatPhaseScore],
    policy: SourceTimingProbeBpmCandidatePolicy,
) -> impl Iterator<Item = DownbeatPhaseScore> + '_ {
    let best_score = phases.first().map_or(0.0, |phase| phase.score);
    phases.iter().copied().skip(1).filter(move |phase| {
        phase.score > 0.0 && best_score - phase.score <= policy.downbeat_ambiguity_margin
    })
}

fn distance_to_repeating_phase(time_seconds: f32, phase_seconds: f32, period_seconds: f32) -> f32 {
    let position = (time_seconds - phase_seconds).rem_euclid(period_seconds);
    position.min(period_seconds - position)
}
