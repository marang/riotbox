use serde::{Deserialize, Serialize};

use crate::{
    TimestampMs,
    ids::{ActionId, SourceId},
    source_graph::SourceGraphVersion,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportArtifactSourceGraphRef {
    pub source_id: SourceId,
    pub graph_version: SourceGraphVersion,
    pub graph_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportArtifactTimingGridRef {
    pub source_id: SourceId,
    #[serde(default)]
    pub hypothesis_id: Option<String>,
    pub confirmed_by_action: ActionId,
    pub confirmed_at: TimestampMs,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportArtifactFallbackComparisonEvidence {
    pub comparison_kind: ExportArtifactFallbackComparisonKind,
    pub reference_identity: String,
    #[serde(default)]
    pub rms_difference_micros: Option<u32>,
    #[serde(default)]
    pub normalized_correlation_micros: Option<i32>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportArtifactFallbackComparisonKind {
    SourceVsFallback,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportArtifactAudioMetrics {
    #[serde(default)]
    pub peak_milli_dbfs: Option<i32>,
    #[serde(default)]
    pub rms_milli_dbfs: Option<i32>,
    #[serde(default)]
    pub peak_amplitude_micros: Option<u32>,
    #[serde(default)]
    pub rms_amplitude_micros: Option<u32>,
    #[serde(default)]
    pub silent_frame_count: Option<u64>,
    #[serde(default)]
    pub total_frame_count: Option<u64>,
}
