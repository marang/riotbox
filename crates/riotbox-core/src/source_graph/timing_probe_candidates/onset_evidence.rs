use crate::source_graph::timing_probe_candidates::types::SourceTimingProbeBpmCandidateInput;

pub(super) fn normalized_onset_times(input: &SourceTimingProbeBpmCandidateInput) -> Vec<f32> {
    normalized_onset_evidence(input)
        .into_iter()
        .map(|onset| onset.time_seconds)
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct NormalizedOnsetEvidence {
    pub(super) time_seconds: f32,
    pub(super) strength: f32,
}

pub(super) fn normalized_onset_evidence(
    input: &SourceTimingProbeBpmCandidateInput,
) -> Vec<NormalizedOnsetEvidence> {
    let max_time = input.duration_seconds.max(0.0);
    let mut onsets = input
        .onset_times_seconds
        .iter()
        .enumerate()
        .filter_map(|(index, time_seconds)| {
            if !time_seconds.is_finite() || *time_seconds < 0.0 || *time_seconds > max_time {
                return None;
            }
            let strength = input
                .onset_strengths
                .get(index)
                .copied()
                .filter(|strength| strength.is_finite() && *strength > 0.0)
                .unwrap_or(1.0);
            Some(NormalizedOnsetEvidence {
                time_seconds: *time_seconds,
                strength,
            })
        })
        .collect::<Vec<_>>();
    onsets.sort_by(|left, right| {
        left.time_seconds
            .total_cmp(&right.time_seconds)
            .then_with(|| right.strength.total_cmp(&left.strength))
    });
    onsets
}

pub(super) fn normalized_onset_times_and_strengths(
    input: &SourceTimingProbeBpmCandidateInput,
) -> Vec<(f32, f32)> {
    normalized_onset_evidence(input)
        .into_iter()
        .map(|onset| (onset.time_seconds, onset.strength))
        .collect()
}
