use riotbox_core::source_graph::{
    GrooveResidual, GrooveSubdivision, SourceTimingAnchor, SourceTimingAnchorType, TimingModel,
};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) struct ManifestSourceTimingAnchorEvidence {
    primary_anchor_count: usize,
    primary_kick_anchor_count: usize,
    primary_backbeat_anchor_count: usize,
    primary_transient_anchor_count: usize,
}

impl ManifestSourceTimingAnchorEvidence {
    pub(super) fn from_timing(timing: &TimingModel) -> Self {
        let anchors = timing
            .primary_hypothesis()
            .map_or(&[][..], |hypothesis| hypothesis.anchors.as_slice());
        Self {
            primary_anchor_count: anchors.len(),
            primary_kick_anchor_count: count_source_timing_anchor_type(
                anchors,
                SourceTimingAnchorType::Kick,
            ),
            primary_backbeat_anchor_count: count_source_timing_anchor_type(
                anchors,
                SourceTimingAnchorType::Backbeat,
            ),
            primary_transient_anchor_count: count_source_timing_anchor_type(
                anchors,
                SourceTimingAnchorType::TransientCluster,
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) struct ManifestSourceTimingGrooveEvidence {
    pub(super) primary_groove_residual_count: usize,
    pub(super) primary_max_abs_offset_ms: f32,
    pub(super) primary_groove_preview: Vec<ManifestSourceTimingGrooveResidual>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) struct ManifestSourceTimingGrooveResidual {
    pub(super) subdivision: &'static str,
    pub(super) offset_ms: f32,
    pub(super) confidence: f32,
}

impl ManifestSourceTimingGrooveEvidence {
    pub(super) fn from_timing(timing: &TimingModel) -> Self {
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
                .map(ManifestSourceTimingGrooveResidual::from_residual)
                .collect(),
        }
    }
}

impl ManifestSourceTimingGrooveResidual {
    fn from_residual(residual: &GrooveResidual) -> Self {
        Self {
            subdivision: source_timing_groove_subdivision_label(residual.subdivision),
            offset_ms: residual.offset_ms,
            confidence: residual.confidence,
        }
    }
}

fn source_timing_groove_subdivision_label(subdivision: GrooveSubdivision) -> &'static str {
    match subdivision {
        GrooveSubdivision::Eighth => "eighth",
        GrooveSubdivision::Triplet => "triplet",
        GrooveSubdivision::Sixteenth => "sixteenth",
        GrooveSubdivision::ThirtySecond => "thirty_second",
    }
}

fn count_source_timing_anchor_type(
    anchors: &[SourceTimingAnchor],
    anchor_type: SourceTimingAnchorType,
) -> usize {
    anchors
        .iter()
        .filter(|anchor| anchor.anchor_type == anchor_type)
        .count()
}
