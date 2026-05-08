use riotbox_core::source_graph::{SourceTimingAnchor, SourceTimingAnchorType, TimingModel};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct AnchorEvidenceSummary {
    pub primary_anchor_count: usize,
    pub primary_kick_anchor_count: usize,
    pub primary_backbeat_anchor_count: usize,
    pub primary_transient_anchor_count: usize,
    pub primary_anchor_preview: Vec<AnchorEvidencePreview>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct AnchorEvidencePreview {
    pub anchor_type: &'static str,
    pub time_seconds: f32,
    pub bar_index: Option<u32>,
    pub beat_index: Option<u32>,
    pub confidence: f32,
    pub strength: f32,
    pub tags: Vec<String>,
}

impl AnchorEvidenceSummary {
    pub(crate) fn from_timing(timing: &TimingModel) -> Self {
        let anchors = timing
            .primary_hypothesis()
            .map_or(&[][..], |hypothesis| hypothesis.anchors.as_slice());
        Self {
            primary_anchor_count: anchors.len(),
            primary_kick_anchor_count: count_anchor_type(anchors, SourceTimingAnchorType::Kick),
            primary_backbeat_anchor_count: count_anchor_type(
                anchors,
                SourceTimingAnchorType::Backbeat,
            ),
            primary_transient_anchor_count: count_anchor_type(
                anchors,
                SourceTimingAnchorType::TransientCluster,
            ),
            primary_anchor_preview: anchors
                .iter()
                .take(8)
                .map(AnchorEvidencePreview::from_anchor)
                .collect(),
        }
    }
}

impl AnchorEvidencePreview {
    fn from_anchor(anchor: &SourceTimingAnchor) -> Self {
        Self {
            anchor_type: source_timing_anchor_type_label(anchor.anchor_type),
            time_seconds: anchor.time_seconds,
            bar_index: anchor.bar_index,
            beat_index: anchor.beat_index,
            confidence: anchor.confidence,
            strength: anchor.strength,
            tags: anchor.tags.clone(),
        }
    }
}

pub(crate) fn source_timing_anchor_type_label(anchor_type: SourceTimingAnchorType) -> &'static str {
    match anchor_type {
        SourceTimingAnchorType::Kick => "kick",
        SourceTimingAnchorType::Snare => "snare",
        SourceTimingAnchorType::Backbeat => "backbeat",
        SourceTimingAnchorType::Fill => "fill",
        SourceTimingAnchorType::LoopWindow => "loop_window",
        SourceTimingAnchorType::AnswerSlot => "answer_slot",
        SourceTimingAnchorType::CaptureCandidate => "capture_candidate",
        SourceTimingAnchorType::TransientCluster => "transient_cluster",
    }
}

fn count_anchor_type(anchors: &[SourceTimingAnchor], anchor_type: SourceTimingAnchorType) -> usize {
    anchors
        .iter()
        .filter(|anchor| anchor.anchor_type == anchor_type)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use riotbox_core::source_graph::{
        MeterHint, SourceTimingProbeBpmCandidateInput, SourceTimingProbeBpmCandidatePolicy,
        timing_model_from_probe_bpm_candidates,
    };

    #[test]
    fn summarizes_classified_primary_anchor_evidence() {
        let timing = timing_model_from_probe_bpm_candidates(
            &weighted_candidate_input(
                "classified-anchor-summary",
                4.0,
                &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5],
                &[1.0, 0.7, 0.35, 0.8, 1.1, 0.75, 0.3, 0.85],
            ),
            SourceTimingProbeBpmCandidatePolicy::default(),
        );

        let summary = AnchorEvidenceSummary::from_timing(&timing);

        assert_eq!(summary.primary_anchor_count, 8);
        assert_eq!(summary.primary_kick_anchor_count, 2);
        assert_eq!(summary.primary_backbeat_anchor_count, 4);
        assert_eq!(summary.primary_transient_anchor_count, 2);
        assert!(summary.primary_anchor_preview.iter().any(|anchor| {
            anchor.anchor_type == "kick"
                && anchor.bar_index == Some(1)
                && anchor.beat_index == Some(1)
                && anchor.tags.contains(&"kick_anchor".into())
        }));
    }

    #[test]
    fn keeps_ambiguous_anchor_evidence_generic() {
        let timing = timing_model_from_probe_bpm_candidates(
            &weighted_candidate_input(
                "ambiguous-anchor-summary",
                4.0,
                &[0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5],
                &[0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            ),
            SourceTimingProbeBpmCandidatePolicy::default(),
        );

        let summary = AnchorEvidenceSummary::from_timing(&timing);

        assert_eq!(summary.primary_anchor_count, 8);
        assert_eq!(summary.primary_kick_anchor_count, 0);
        assert_eq!(summary.primary_backbeat_anchor_count, 0);
        assert_eq!(summary.primary_transient_anchor_count, 8);
        assert!(
            summary
                .primary_anchor_preview
                .iter()
                .all(|anchor| anchor.anchor_type == "transient_cluster")
        );
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
