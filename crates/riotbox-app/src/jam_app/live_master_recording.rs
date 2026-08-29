use std::path::{Path, PathBuf};

use riotbox_audio::runtime::{
    AudioOutputInfo, AudioRuntimeHealth, LiveMasterCaptureOutcome, LiveMasterCaptureRequest,
};
use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType,
        LiveRecordingExportBoundary, LiveRecordingExportRole, Quantization, TargetScope,
        UndoPolicy,
    },
    export_readiness::{ExportScope, ProductExportDestinationKind},
    ids::{ActionId, CaptureId, ExportReceiptId, SceneId},
    queue::QueueEnqueueResult,
    session::{ExportArtifactSourceGraphRef, ExportArtifactTimingGridRef, ExportReceiptState},
};
use serde::{Deserialize, Serialize};

use super::{JamAppError, JamAppState};

mod artifact;
mod session_identity;

use artifact::{
    build_recording_receipt, prepare_validated_recording, publish_recording, remove_file_if_hash,
    remove_owned_recording,
};
use session_identity::prepare_recording_plan_input;

pub const LIVE_MASTER_RECORDING_DURATION_BEATS: u32 = 8;
pub const LIVE_MASTER_RECORDING_PROOF_SCHEMA: &str =
    "riotbox.live_recording_runtime_master_bar_window.v2";

pub fn live_master_recording_proof_path(
    destination_path: impl AsRef<Path>,
) -> Result<PathBuf, JamAppError> {
    session_identity::proof_path_for(destination_path.as_ref())
}

#[derive(Clone, Debug, PartialEq)]
pub struct LiveMasterRecordingPlan {
    pub action_id: ActionId,
    pub request: LiveMasterCaptureRequest,
    pub destination_path: PathBuf,
    pub proof_path: PathBuf,
    pub confirmed_bpm: f32,
    pub beats_per_bar: u8,
    pub bar_grid_anchor_beat_cursor: u64,
    pub requested_start_position_beats: f64,
    pub scene_id: SceneId,
    pub output: AudioOutputInfo,
    pub session_id: String,
    pub session_pre_capture_sha256: String,
    pub source_graph_ref: Option<ExportArtifactSourceGraphRef>,
    pub timing_grid_ref: Option<ExportArtifactTimingGridRef>,
    pub source_capture_refs: Vec<CaptureId>,
    pub lineage_capture_refs: Vec<CaptureId>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LiveMasterRecordingQueueResult {
    Enqueued(Box<LiveMasterRecordingPlan>),
    Rejected { reason: String },
    AlreadyPending,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LiveMasterRecordingProof {
    pub schema: String,
    pub receipt_id: ExportReceiptId,
    pub action_id: ActionId,
    pub session_id: String,
    pub session_pre_capture_sha256: String,
    pub scene_id: SceneId,
    pub confirmed_bpm_micros: u64,
    pub duration_beats: u32,
    pub beats_per_bar: u8,
    pub bar_grid_anchor_position_microbeats: u64,
    pub beat_span_per_frame_nanobeats: u64,
    pub requested_start_position_microbeats: u64,
    pub captured_start_position_microbeats: u64,
    pub captured_end_position_microbeats: u64,
    pub start_alignment_error_frame_micros: u64,
    pub duration_error_frame_micros: u64,
    pub host: String,
    pub device: String,
    pub device_sample_format: String,
    pub wav_sample_format: String,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub frame_count: u64,
    pub callback_count: u64,
    pub max_callback_gap_micros: Option<u64>,
    pub callback_gap_over_threshold_count: u64,
    pub callback_scratch_overflow_count: u64,
    pub stream_error_count: u64,
    pub transport_mismatch_count: u64,
    pub tempo_mismatch_count: u64,
    pub timing_window_mismatch_count: u64,
    pub armed_callback_count: u64,
    pub active_sample_count: u64,
    pub peak_amplitude_micros: u32,
    pub rms_amplitude_micros: u32,
    pub clip_count: u64,
    pub sample_payload_sha256: String,
    pub wav_sha256: String,
    pub source_graph_ref: Option<ExportArtifactSourceGraphRef>,
    pub timing_grid_ref: Option<ExportArtifactTimingGridRef>,
    pub source_capture_refs: Vec<CaptureId>,
    pub lineage_capture_refs: Vec<CaptureId>,
}

impl JamAppState {
    pub fn queue_live_master_recording(
        &mut self,
        requested_at: TimestampMs,
        output: &AudioOutputInfo,
        destination_path: impl AsRef<Path>,
    ) -> LiveMasterRecordingQueueResult {
        let destination_path = destination_path.as_ref();
        let target_scene = self.session.runtime_state.scene_state.active_scene.clone();

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::ExportLiveRecording,
            Quantization::Immediate,
            ActionTarget {
                scope: Some(TargetScope::Session),
                scene_id: target_scene,
                ..Default::default()
            },
        );
        draft.params = ActionParams::LiveRecordingExport {
            export_scope: ExportScope::LiveRecording,
            export_role: LiveRecordingExportRole::LiveRecordingCapture,
            boundary: LiveRecordingExportBoundary::RuntimeMasterBarWindowV2,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalFilePath,
            destination_path: Some(destination_path.to_string_lossy().into_owned()),
            receipt_id: None,
        };
        draft.undo_policy = UndoPolicy::NotUndoable {
            reason: "live master recording writes host-audio files outside musical undo".into(),
        };
        draft.explanation = Some(
            "record the next exact two-bar 4/4 window from the real post-limiter runtime master callback"
                .into(),
        );

        match self
            .queue
            .enqueue_if_no_pending_command(draft, requested_at)
        {
            QueueEnqueueResult::AlreadyPending { .. } => {
                LiveMasterRecordingQueueResult::AlreadyPending
            }
            QueueEnqueueResult::Enqueued(action_id) => {
                match prepare_recording_plan_input(self, output, destination_path) {
                    Ok(prepared) => {
                        let identity = prepared.identity;
                        self.refresh_view();
                        LiveMasterRecordingQueueResult::Enqueued(Box::new(
                            LiveMasterRecordingPlan {
                                action_id,
                                request: LiveMasterCaptureRequest {
                                    target_frame_count: prepared.target_frame_count,
                                    channel_count: output.channel_count,
                                    expected_tempo_bpm: identity.confirmed_bpm,
                                    start_position_beats: Some(prepared.start_position_beats),
                                },
                                destination_path: destination_path.to_owned(),
                                proof_path: prepared.proof_path,
                                confirmed_bpm: identity.confirmed_bpm,
                                beats_per_bar: identity.beats_per_bar,
                                bar_grid_anchor_beat_cursor: identity.bar_grid_anchor_beat_cursor,
                                requested_start_position_beats: prepared.start_position_beats,
                                scene_id: identity.scene_id,
                                output: output.clone(),
                                session_id: self.session.session_id.clone(),
                                session_pre_capture_sha256: prepared.session_sha256,
                                source_graph_ref: identity.source_graph_ref,
                                timing_grid_ref: identity.timing_grid_ref,
                                source_capture_refs: identity.source_capture_refs,
                                lineage_capture_refs: identity.lineage_capture_refs,
                            },
                        ))
                    }
                    Err(error) => {
                        let reason = error.to_string();
                        self.queue.reject(action_id, reason.clone());
                        self.refresh_view();
                        LiveMasterRecordingQueueResult::Rejected { reason }
                    }
                }
            }
        }
    }

    pub fn reject_live_master_recording(&mut self, action_id: ActionId, reason: impl Into<String>) {
        self.queue.reject(action_id, reason.into());
        self.refresh_view();
    }

    pub fn commit_live_master_recording(
        &mut self,
        plan: &LiveMasterRecordingPlan,
        outcome: &LiveMasterCaptureOutcome,
        final_health: &AudioRuntimeHealth,
        committed_at: TimestampMs,
    ) -> Result<ExportReceiptState, JamAppError> {
        let written = match prepare_validated_recording(self, plan, outcome, final_health) {
            Ok(written) => written,
            Err(error) => {
                self.reject_live_master_recording(plan.action_id, error.to_string());
                return Err(error);
            }
        };
        if let Err(error) = publish_recording(plan, &written) {
            self.reject_live_master_recording(plan.action_id, error.to_string());
            return Err(error);
        }

        let receipt = match build_recording_receipt(self, plan, &written, committed_at) {
            Ok(receipt) => receipt,
            Err(error) => {
                remove_owned_recording(plan, &written);
                self.reject_live_master_recording(plan.action_id, error.to_string());
                return Err(error);
            }
        };
        if !receipt.live_recording_host_audio_readiness_report().ready() {
            remove_owned_recording(plan, &written);
            let error = JamAppError::InvalidSession(
                "live master recording receipt failed host-audio readiness".into(),
            );
            self.reject_live_master_recording(plan.action_id, error.to_string());
            return Err(error);
        }
        let result_summary = format!(
            "recorded bar-aligned two-bar live runtime master receipt {} sha256 {}",
            receipt.receipt_id, receipt.export_hash
        );
        if let Err(error) = self.commit_export_receipt_after_side_effect(
            plan.action_id,
            committed_at,
            receipt.clone(),
            result_summary,
        ) {
            remove_owned_recording(plan, &written);
            return Err(error);
        }
        Ok(receipt)
    }

    pub fn commit_and_save_live_master_recording(
        &mut self,
        plan: &LiveMasterRecordingPlan,
        outcome: &LiveMasterCaptureOutcome,
        final_health: &AudioRuntimeHealth,
        committed_at: TimestampMs,
    ) -> Result<ExportReceiptState, JamAppError> {
        let session_before_commit = self.session.clone();
        let queue_before_commit = self.queue.clone();
        let last_boundary_before_commit = self.runtime.last_commit_boundary.clone();
        let receipt =
            self.commit_live_master_recording(plan, outcome, final_health, committed_at)?;

        if let Err(save_error) = self.save_session_without_source_graph_write() {
            remove_file_if_hash(&plan.proof_path, &receipt.normalized_manifest_hash);
            remove_file_if_hash(&plan.destination_path, &receipt.export_hash);
            let cleanup_incomplete = plan.proof_path.exists() || plan.destination_path.exists();
            self.session = session_before_commit;
            self.queue = queue_before_commit;
            self.runtime.last_commit_boundary = last_boundary_before_commit;
            let reason = if cleanup_incomplete {
                format!(
                    "live master recording Session save failed and owned artifact cleanup was incomplete: {save_error}"
                )
            } else {
                format!("live master recording Session save failed: {save_error}")
            };
            self.reject_live_master_recording(plan.action_id, reason.clone());
            return Err(JamAppError::InvalidSession(reason));
        }

        Ok(receipt)
    }
}
