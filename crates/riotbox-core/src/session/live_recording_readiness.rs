use serde::{Deserialize, Serialize};

use crate::export_readiness::{ExportScope, UnsupportedExportScope};

use super::{
    export_types::ExportReceiptState, live_recording_host_audio::ExportLiveRecordingHostAudioRef,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveRecordingHostAudioReadinessReport {
    pub status: LiveRecordingHostAudioReadinessStatus,
    pub blockers: Vec<LiveRecordingHostAudioReadinessBlocker>,
}

impl LiveRecordingHostAudioReadinessReport {
    #[must_use]
    pub fn ready(&self) -> bool {
        self.status == LiveRecordingHostAudioReadinessStatus::Ready
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiveRecordingHostAudioReadinessStatus {
    Ready,
    Blocked,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiveRecordingHostAudioReadinessBlocker {
    NotLiveRecordingScope,
    UnsupportedScopeFlagPresent,
    MissingHostAudioEvidence,
    BlankHost,
    BlankDevice,
    ZeroRecordingDuration,
    CallbackGapOverThreshold,
    StreamErrorReported,
}

#[must_use]
pub fn validate_live_recording_host_audio_readiness(
    receipt: &ExportReceiptState,
) -> LiveRecordingHostAudioReadinessReport {
    let mut blockers = Vec::new();

    if receipt.export_scope != ExportScope::LiveRecording {
        push_blocker(
            &mut blockers,
            LiveRecordingHostAudioReadinessBlocker::NotLiveRecordingScope,
        );
    }
    if receipt
        .unsupported_scopes
        .contains(&UnsupportedExportScope::LiveRecording)
    {
        push_blocker(
            &mut blockers,
            LiveRecordingHostAudioReadinessBlocker::UnsupportedScopeFlagPresent,
        );
    }
    if receipt.live_recording_host_audio_refs.is_empty() {
        push_blocker(
            &mut blockers,
            LiveRecordingHostAudioReadinessBlocker::MissingHostAudioEvidence,
        );
    }

    for evidence in &receipt.live_recording_host_audio_refs {
        push_host_audio_ref_blockers(evidence, &mut blockers);
    }

    let status = if blockers.is_empty() {
        LiveRecordingHostAudioReadinessStatus::Ready
    } else {
        LiveRecordingHostAudioReadinessStatus::Blocked
    };

    LiveRecordingHostAudioReadinessReport { status, blockers }
}

fn push_host_audio_ref_blockers(
    evidence: &ExportLiveRecordingHostAudioRef,
    blockers: &mut Vec<LiveRecordingHostAudioReadinessBlocker>,
) {
    if evidence.host.trim().is_empty() {
        push_blocker(blockers, LiveRecordingHostAudioReadinessBlocker::BlankHost);
    }
    if evidence.device.trim().is_empty() {
        push_blocker(
            blockers,
            LiveRecordingHostAudioReadinessBlocker::BlankDevice,
        );
    }
    if evidence.recording_duration_ms == 0 {
        push_blocker(
            blockers,
            LiveRecordingHostAudioReadinessBlocker::ZeroRecordingDuration,
        );
    }
    if evidence.callback_gap_summary.over_threshold_count > 0 {
        push_blocker(
            blockers,
            LiveRecordingHostAudioReadinessBlocker::CallbackGapOverThreshold,
        );
    }
    if evidence.stream_error_summary.error_count > 0 {
        push_blocker(
            blockers,
            LiveRecordingHostAudioReadinessBlocker::StreamErrorReported,
        );
    }
}

fn push_blocker(
    blockers: &mut Vec<LiveRecordingHostAudioReadinessBlocker>,
    blocker: LiveRecordingHostAudioReadinessBlocker,
) {
    if !blockers.contains(&blocker) {
        blockers.push(blocker);
    }
}
