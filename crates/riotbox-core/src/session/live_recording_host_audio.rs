use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportLiveRecordingHostAudioRef {
    pub host: String,
    pub device: String,
    pub recording_duration_ms: u64,
    pub callback_gap_summary: ExportLiveRecordingCallbackGapSummary,
    pub stream_error_summary: ExportLiveRecordingStreamErrorSummary,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportLiveRecordingCallbackGapSummary {
    #[serde(default)]
    pub max_gap_ms: Option<u64>,
    pub over_threshold_count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportLiveRecordingStreamErrorSummary {
    pub error_count: u32,
    #[serde(default)]
    pub last_error: Option<String>,
}
