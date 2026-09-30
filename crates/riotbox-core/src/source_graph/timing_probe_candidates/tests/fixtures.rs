use crate::source_graph::timing_probe_candidates::types::{
    SourceTimingProbeBpmCandidateInput, SourceTimingProbeBpmCandidatePolicy,
};
use crate::source_graph::{MeterHint, TimingModel, TimingWarningCode};

pub(super) fn candidate_input(
    source_id: &str,
    duration_seconds: f32,
    onset_times_seconds: &[f32],
) -> SourceTimingProbeBpmCandidateInput {
    weighted_candidate_input(
        source_id,
        duration_seconds,
        onset_times_seconds,
        &vec![1.0; onset_times_seconds.len()],
    )
}

pub(super) fn weighted_candidate_input(
    source_id: &str,
    duration_seconds: f32,
    onset_times_seconds: &[f32],
    onset_strengths: &[f32],
) -> SourceTimingProbeBpmCandidateInput {
    SourceTimingProbeBpmCandidateInput {
        source_id: source_id.into(),
        duration_seconds,
        onset_times_seconds: onset_times_seconds.to_vec(),
        onset_strengths: onset_strengths.to_vec(),
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
    }
}

pub(super) fn even_onsets(start_seconds: f32, period_seconds: f32, count: usize) -> Vec<f32> {
    (0..count)
        .map(|index| start_seconds + period_seconds * index as f32)
        .collect()
}

pub(super) fn downbeat_strengths(count: usize, beats_per_bar: usize) -> Vec<f32> {
    (0..count)
        .map(|index| if index % beats_per_bar == 0 { 2.0 } else { 0.5 })
        .collect()
}

pub(super) fn moderate_downbeat_strengths(count: usize, beats_per_bar: usize) -> Vec<f32> {
    (0..count)
        .map(|index| if index % beats_per_bar == 0 { 0.8 } else { 0.5 })
        .collect()
}

pub(super) fn focused_120_bpm_policy() -> SourceTimingProbeBpmCandidatePolicy {
    SourceTimingProbeBpmCandidatePolicy {
        min_bpm: 80.0,
        max_bpm: 180.0,
        ..SourceTimingProbeBpmCandidatePolicy::default()
    }
}

pub(super) fn assert_bpm_close(actual: Option<f32>, expected: f32) {
    let actual = actual.expect("bpm estimate");
    assert!((actual - expected).abs() <= 0.01, "{actual} != {expected}");
}

pub(super) fn has_warning(timing: &TimingModel, expected: TimingWarningCode) -> bool {
    timing
        .warnings
        .iter()
        .any(|warning| warning.code == expected)
}
