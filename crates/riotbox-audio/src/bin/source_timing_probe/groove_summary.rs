use riotbox_core::source_graph::{GrooveResidual, GrooveSubdivision, TimingModel};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct GrooveEvidenceSummary {
    pub primary_groove_residual_count: usize,
    pub primary_max_abs_offset_ms: f32,
    pub primary_groove_preview: Vec<GrooveResidualPreview>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct GrooveResidualPreview {
    pub subdivision: &'static str,
    pub offset_ms: f32,
    pub confidence: f32,
}

impl GrooveEvidenceSummary {
    pub(crate) fn from_timing(timing: &TimingModel) -> Self {
        let groove = timing
            .primary_hypothesis()
            .map_or(&[][..], |hypothesis| hypothesis.groove.as_slice());
        Self {
            primary_groove_residual_count: groove.len(),
            primary_max_abs_offset_ms: groove
                .iter()
                .map(|residual| residual.offset_ms.abs())
                .fold(0.0_f32, f32::max),
            primary_groove_preview: groove
                .iter()
                .take(4)
                .map(GrooveResidualPreview::from_residual)
                .collect(),
        }
    }
}

impl GrooveResidualPreview {
    fn from_residual(residual: &GrooveResidual) -> Self {
        Self {
            subdivision: groove_subdivision_label(residual.subdivision),
            offset_ms: residual.offset_ms,
            confidence: residual.confidence,
        }
    }
}

pub(crate) fn groove_subdivision_label(subdivision: GrooveSubdivision) -> &'static str {
    match subdivision {
        GrooveSubdivision::Eighth => "eighth",
        GrooveSubdivision::Triplet => "triplet",
        GrooveSubdivision::Sixteenth => "sixteenth",
        GrooveSubdivision::ThirtySecond => "thirty_second",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riotbox_core::source_graph::{
        MeterHint, SourceTimingProbeBpmCandidateInput, SourceTimingProbeBpmCandidatePolicy,
        timing_model_from_probe_bpm_candidates,
    };

    #[test]
    fn summarizes_primary_groove_residual_evidence() {
        let timing = timing_model_from_probe_bpm_candidates(
            &weighted_candidate_input(
                "groove-summary",
                4.0,
                &[0.0, 0.29, 0.5, 0.79, 1.0, 1.29, 1.5, 1.79],
                &[1.0, 0.75, 1.0, 0.75, 1.0, 0.75, 1.0, 0.75],
            ),
            SourceTimingProbeBpmCandidatePolicy {
                min_bpm: 110.0,
                max_bpm: 130.0,
                ..SourceTimingProbeBpmCandidatePolicy::default()
            },
        );

        let summary = GrooveEvidenceSummary::from_timing(&timing);

        assert!(summary.primary_groove_residual_count > 0);
        assert!(summary.primary_max_abs_offset_ms > 0.0);
        assert!(summary.primary_groove_preview.iter().any(|residual| {
            residual.subdivision == "eighth"
                && residual.offset_ms > 10.0
                && residual.confidence > 0.4
        }));
    }

    fn weighted_candidate_input(
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
}
