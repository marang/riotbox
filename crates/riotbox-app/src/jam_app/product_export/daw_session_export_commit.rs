use std::path::Path;

use riotbox_core::{
    TimestampMs,
    action::{ActionCommand, ActionParams, DawSessionExportBoundary},
    session::{ActionCommitRecord, ExportReceiptState},
    transport::CommitBoundaryState,
};

use crate::jam_app::{
    JamAppError, JamAppState,
    daw_session_writer_proof::{
        attach_daw_session_writer_proof_evidence_to_receipt, daw_session_writer_proof_report,
        write_daw_session_writer_proof_skeleton,
    },
    helpers::update_logged_action_result,
};

use super::DawSessionExportQueueResult;

impl JamAppState {
    pub fn commit_daw_session_writer_export(
        &mut self,
        session_base_dir: Option<&Path>,
        destination_dir: impl AsRef<Path>,
        requested_at: TimestampMs,
    ) -> Result<ExportReceiptState, JamAppError> {
        let destination_dir = destination_dir.as_ref();
        let action_id = match self.pending_daw_session_writer_export_action_id(destination_dir) {
            Some(action_id) => action_id,
            None => match self.queue_daw_session_writer_export(
                requested_at,
                session_base_dir,
                Some(destination_dir.to_string_lossy().into_owned()),
            ) {
                DawSessionExportQueueResult::Enqueued { action_id } => action_id,
                DawSessionExportQueueResult::Rejected { reason } => {
                    return Err(JamAppError::InvalidSession(reason));
                }
                DawSessionExportQueueResult::AlreadyPending => {
                    return Err(JamAppError::InvalidSession(
                        "DAW session writer export is already pending".into(),
                    ));
                }
            },
        };

        if let Err(error) = write_daw_session_writer_proof_skeleton(
            &self.session,
            session_base_dir,
            destination_dir,
        ) {
            self.queue.reject(action_id, error.to_string());
            self.refresh_view();
            return Err(error);
        }

        let report = daw_session_writer_proof_report(destination_dir);
        let receipt_index = latest_daw_session_receipt_index(&self.session).ok_or_else(|| {
            JamAppError::InvalidSession(
                "DAW session writer export requires a DAW session receipt".into(),
            )
        })?;
        if let Err(error) = attach_daw_session_writer_proof_evidence_to_receipt(
            &mut self.session.export_receipts[receipt_index],
            &report,
        ) {
            let message = format!("DAW session writer proof evidence rejected: {error:?}");
            self.queue.reject(action_id, message.clone());
            self.refresh_view();
            return Err(JamAppError::InvalidSession(message));
        }

        let receipt = self.session.export_receipts[receipt_index].clone();
        let boundary = CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Immediate,
            beat_index: self.runtime.transport.beat_index,
            bar_index: self.runtime.transport.bar_index,
            phrase_index: self.runtime.transport.phrase_index,
            scene_id: self.runtime.transport.current_scene.clone(),
        };
        let result_summary = format!(
            "wrote local DAW session writer proof receipt {} proof {}",
            receipt.receipt_id,
            report
                .proof_sha256
                .as_deref()
                .unwrap_or("missing-proof-sha")
        );
        let committed_ref = self
            .queue
            .commit_pending_after_side_effect(
                action_id,
                boundary.clone(),
                requested_at,
                result_summary.clone(),
            )
            .ok_or_else(|| {
                JamAppError::InvalidSession(format!(
                    "queued DAW session writer action {action_id} was not ready to commit"
                ))
            })?;
        let action = self
            .queue
            .history_action(committed_ref.action_id)
            .cloned()
            .ok_or_else(|| {
                JamAppError::InvalidSession(format!(
                    "committed DAW session writer action {} missing from queue history",
                    committed_ref.action_id
                ))
            })?;

        self.session.action_log.actions.push(action);
        self.session
            .action_log
            .commit_records
            .push(ActionCommitRecord {
                action_id,
                boundary: committed_ref.boundary,
                commit_sequence: committed_ref.commit_sequence,
                committed_at: requested_at,
            });
        update_logged_action_result(&mut self.session, action_id, result_summary);
        self.runtime.last_commit_boundary = Some(boundary);
        self.refresh_view();

        Ok(receipt)
    }

    fn pending_daw_session_writer_export_action_id(
        &self,
        destination_dir: &Path,
    ) -> Option<riotbox_core::ids::ActionId> {
        let expected_destination = destination_dir.to_string_lossy();
        self.queue
            .pending_actions()
            .into_iter()
            .find(|action| {
                action.command == ActionCommand::ExportDawSession
                    && matches!(
                        &action.params,
                        ActionParams::DawSessionExport {
                            boundary: DawSessionExportBoundary::LocalProjectWriterV1,
                            destination_path,
                            ..
                        } if destination_path.as_deref() == Some(expected_destination.as_ref())
                    )
            })
            .map(|action| action.id)
    }
}

fn latest_daw_session_receipt_index(session: &riotbox_core::session::SessionFile) -> Option<usize> {
    session.export_receipts.iter().rposition(|receipt| {
        receipt.export_scope == riotbox_core::export_readiness::ExportScope::DawSession
    })
}
