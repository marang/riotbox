//! Action queue, publication and Session commit orchestration for live-master DAW handoff.
//! Admission and document construction have separate owners; archive mechanics
//! remain shared with the W-30 exporter.

use crate::jam_app::{
    JamAppError, JamAppState,
    dawproject_archive::{
        DawprojectArchiveAudio, DawprojectArchivePayload, remove_owned_destination,
        validate_destination, write_dawproject_archive,
    },
    product_export::DawSessionExportQueueResult,
};
use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType,
        DawSessionExportBoundary, Quantization, TargetScope, UndoPolicy,
    },
    export_readiness::{ExportScope, ProductExportDestinationKind},
    ids::{ActionId, ExportReceiptId},
    queue::QueueEnqueueResult,
    session::{ExportReceiptState, SessionFile},
};
use std::path::Path;

mod document;
mod input;
pub use document::LiveMasterDawprojectProof;
use input::{latest_live_master_receipt, validate_queue_source_receipt};

pub const LIVE_MASTER_DAWPROJECT_ACTION_BOUNDARY_ID: &str = "live_master_dawproject_v1";
pub const LIVE_MASTER_DAWPROJECT_PROOF_SCHEMA: &str = "riotbox.live_master_dawproject.v1";
const EMBEDDED_AUDIO_PATH: &str = "audio/live_master.wav";

impl JamAppState {
    pub fn queue_live_master_dawproject_export(
        &mut self,
        requested_at: TimestampMs,
        destination_path: Option<String>,
    ) -> DawSessionExportQueueResult {
        let receipt_id = latest_live_master_receipt(&self.session)
            .map(|receipt| receipt.receipt_id.as_str().to_owned());
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::ExportDawSession,
            Quantization::Immediate,
            ActionTarget {
                scope: Some(TargetScope::Session),
                ..Default::default()
            },
        );
        draft.params = ActionParams::DawSessionExport {
            export_scope: ExportScope::DawSession,
            boundary: DawSessionExportBoundary::LiveMasterDawprojectV1,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalFilePath,
            destination_path: destination_path.clone(),
            receipt_id,
        };
        draft.undo_policy = UndoPolicy::NotUndoable {
            reason: "DAWproject export writes a musician file outside musical undo".into(),
        };
        draft.explanation = Some("place the committed live master as one two-bar DAW clip".into());
        match self
            .queue
            .enqueue_if_no_pending_command(draft, requested_at)
        {
            QueueEnqueueResult::AlreadyPending { .. } => {
                DawSessionExportQueueResult::AlreadyPending
            }
            QueueEnqueueResult::Enqueued(action_id) => {
                let preflight = destination_path
                    .as_deref()
                    .ok_or_else(|| {
                        JamAppError::InvalidSession(
                            "live-master DAWproject export requires an explicit destination file"
                                .into(),
                        )
                    })
                    .and_then(|destination| validate_destination(Path::new(destination)))
                    .and_then(|_| validate_queue_source_receipt(&self.session));
                match preflight {
                    Ok(_) => {
                        self.refresh_view();
                        DawSessionExportQueueResult::Enqueued { action_id }
                    }
                    Err(error) => {
                        let reason = error.to_string();
                        self.queue.reject(action_id, reason.clone());
                        self.refresh_view();
                        DawSessionExportQueueResult::Rejected { reason }
                    }
                }
            }
        }
    }

    pub fn commit_live_master_dawproject_export(
        &mut self,
        destination_path: impl AsRef<Path>,
        requested_at: TimestampMs,
    ) -> Result<ExportReceiptState, JamAppError> {
        let destination_path = destination_path.as_ref();
        let action_id = match self.pending_live_master_dawproject_action_id(destination_path) {
            Some(action_id) => action_id,
            None => match self.queue_live_master_dawproject_export(
                requested_at,
                Some(destination_path.to_string_lossy().into_owned()),
            ) {
                DawSessionExportQueueResult::Enqueued { action_id } => action_id,
                DawSessionExportQueueResult::Rejected { reason } => {
                    return Err(JamAppError::InvalidSession(reason));
                }
                DawSessionExportQueueResult::AlreadyPending => {
                    return Err(JamAppError::InvalidSession(
                        "another DAW session export is already pending".into(),
                    ));
                }
            },
        };
        let source_receipt_id = self.pending_live_master_dawproject_source_receipt_id(action_id)?;
        let receipt = match write_live_master_dawproject(
            &self.session,
            self.session_base_dir(),
            destination_path,
            action_id,
            &source_receipt_id,
            requested_at,
        ) {
            Ok(receipt) => receipt,
            Err(error) => {
                self.queue.reject(action_id, error.to_string());
                self.refresh_view();
                return Err(error);
            }
        };
        let summary = format!(
            "exported live-master DAWproject receipt {} sha256 {}",
            receipt.receipt_id, receipt.export_hash
        );
        if let Err(error) = self.commit_export_receipt_after_side_effect(
            action_id,
            requested_at,
            receipt.clone(),
            summary,
        ) {
            remove_owned_destination(destination_path, &receipt.export_hash);
            return Err(error);
        }
        Ok(receipt)
    }

    pub fn commit_and_save_live_master_dawproject_export(
        &mut self,
        destination_path: impl AsRef<Path>,
        requested_at: TimestampMs,
    ) -> Result<ExportReceiptState, JamAppError> {
        let destination_path = destination_path.as_ref();
        let action_id = match self.pending_live_master_dawproject_action_id(destination_path) {
            Some(action_id) => action_id,
            None => match self.queue_live_master_dawproject_export(
                requested_at,
                Some(destination_path.to_string_lossy().into_owned()),
            ) {
                DawSessionExportQueueResult::Enqueued { action_id } => action_id,
                DawSessionExportQueueResult::Rejected { reason } => {
                    return Err(JamAppError::InvalidSession(reason));
                }
                DawSessionExportQueueResult::AlreadyPending => {
                    return Err(JamAppError::InvalidSession(
                        "another DAW session export is already pending".into(),
                    ));
                }
            },
        };
        let session_before = self.session.clone();
        let queue_before = self.queue.clone();
        let boundary_before = self.runtime.last_commit_boundary.clone();
        let receipt = self.commit_live_master_dawproject_export(destination_path, requested_at)?;
        if let Err(save_error) = self.save_session_without_source_graph_write() {
            remove_owned_destination(destination_path, &receipt.export_hash);
            let cleanup_incomplete = destination_path.exists();
            self.session = session_before;
            self.queue = queue_before;
            self.runtime.last_commit_boundary = boundary_before;
            let reason = if cleanup_incomplete {
                format!(
                    "live-master DAWproject Session save failed and owned archive cleanup was incomplete: {save_error}"
                )
            } else {
                format!("live-master DAWproject Session save failed: {save_error}")
            };
            debug_assert_eq!(receipt.created_by_action, action_id);
            self.queue.reject(action_id, reason.clone());
            self.refresh_view();
            return Err(JamAppError::InvalidSession(reason));
        }
        Ok(receipt)
    }

    fn pending_live_master_dawproject_action_id(&self, destination: &Path) -> Option<ActionId> {
        let destination = destination.to_string_lossy();
        self.queue
            .pending_actions()
            .into_iter()
            .find(|action| {
                action.command == ActionCommand::ExportDawSession
                    && matches!(&action.params,
                ActionParams::DawSessionExport {
                    boundary: DawSessionExportBoundary::LiveMasterDawprojectV1,
                    destination_path: Some(path), ..
                } if path == destination.as_ref())
            })
            .map(|action| action.id)
    }

    fn pending_live_master_dawproject_source_receipt_id(
        &self,
        action_id: ActionId,
    ) -> Result<ExportReceiptId, JamAppError> {
        self.queue
            .pending_actions()
            .into_iter()
            .find(|action| action.id == action_id)
            .and_then(|action| match &action.params {
                ActionParams::DawSessionExport {
                    boundary: DawSessionExportBoundary::LiveMasterDawprojectV1,
                    receipt_id: Some(receipt_id),
                    ..
                } => Some(ExportReceiptId::new(receipt_id.clone())),
                _ => None,
            })
            .ok_or_else(|| {
                JamAppError::InvalidSession(
                    "queued live-master DAWproject action is missing its pinned V2 receipt ID"
                        .into(),
                )
            })
    }

    fn session_base_dir(&self) -> Option<&Path> {
        self.files
            .as_ref()
            .and_then(|files| files.session_path.parent())
    }
}

fn write_live_master_dawproject(
    session: &SessionFile,
    session_base_dir: Option<&Path>,
    destination: &Path,
    action_id: ActionId,
    source_receipt_id: &ExportReceiptId,
    created_at: TimestampMs,
) -> Result<ExportReceiptState, JamAppError> {
    validate_destination(destination)?;
    let input = input::prepare_input(session, session_base_dir, source_receipt_id)?;
    let proof = document::build_proof(&input);
    let proof_json = serde_json::to_vec_pretty(&proof)?;
    let metadata = document::build_metadata();
    let project = document::build_project(&input.proof);
    let archive = write_dawproject_archive(
        destination,
        DawprojectArchivePayload {
            metadata: &metadata,
            project: &project,
            audio: DawprojectArchiveAudio {
                path: EMBEDDED_AUDIO_PATH,
                bytes: &input.source_wav_bytes,
                sha256: &input.source_wav_sha256,
            },
            proof_json: &proof_json,
        },
    )?;
    match document::build_receipt(destination, action_id, created_at, &input, &archive) {
        Ok(receipt) => Ok(receipt),
        Err(error) => {
            remove_owned_destination(destination, &archive.archive_sha256);
            Err(error)
        }
    }
}
