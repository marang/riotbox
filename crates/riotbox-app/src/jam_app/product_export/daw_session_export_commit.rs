use std::path::Path;

use riotbox_core::{
    TimestampMs,
    action::{ActionCommand, ActionParams, DawSessionExportBoundary},
    session::{ActionCommitRecord, ExportReceiptState},
    transport::CommitBoundaryState,
};

use crate::jam_app::{
    JamAppError, JamAppState,
    daw_session_audible_output_proof::{
        attach_daw_session_audible_output_proof_evidence_to_receipt,
        daw_session_audible_output_proof_report,
    },
    daw_session_host_import_proof::{
        attach_daw_session_host_import_proof_evidence_to_receipt,
        daw_session_host_import_proof_report,
    },
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
                mc202_source_phrase_plan: None,
            });
        update_logged_action_result(&mut self.session, action_id, result_summary);
        self.runtime.last_commit_boundary = Some(boundary);
        self.refresh_view();

        Ok(receipt)
    }

    pub fn commit_daw_session_host_import_proof_export(
        &mut self,
        proof_path: impl AsRef<Path>,
        requested_at: TimestampMs,
    ) -> Result<ExportReceiptState, JamAppError> {
        let proof_path = proof_path.as_ref();
        let action_id = match self.pending_daw_session_host_import_proof_action_id(proof_path) {
            Some(action_id) => action_id,
            None => match self.queue_daw_session_host_import_proof_export(
                requested_at,
                Some(proof_path.to_string_lossy().into_owned()),
            ) {
                DawSessionExportQueueResult::Enqueued { action_id } => action_id,
                DawSessionExportQueueResult::Rejected { reason } => {
                    return Err(JamAppError::InvalidSession(reason));
                }
                DawSessionExportQueueResult::AlreadyPending => {
                    return Err(JamAppError::InvalidSession(
                        "DAW session host-import proof action is already pending".into(),
                    ));
                }
            },
        };

        let receipt_id = match self.pending_daw_session_proof_receipt_id(
            action_id,
            DawSessionExportBoundary::HostImportProofV1,
        ) {
            Some(receipt_id) => receipt_id,
            None => {
                let message = format!(
                    "queued DAW session host-import proof action {action_id} is missing a receipt"
                );
                self.queue.reject(action_id, message.clone());
                self.refresh_view();
                return Err(JamAppError::InvalidSession(message));
            }
        };
        let receipt_index = match daw_session_receipt_index_by_id(&self.session, &receipt_id) {
            Some(receipt_index) => receipt_index,
            None => {
                let message =
                    format!("DAW session host-import proof receipt {receipt_id} is missing");
                self.queue.reject(action_id, message.clone());
                self.refresh_view();
                return Err(JamAppError::InvalidSession(message));
            }
        };
        let report = daw_session_host_import_proof_report(proof_path);
        if !report.ready_for_receipt(&self.session.export_receipts[receipt_index]) {
            let reason = format!(
                "DAW session host-import proof is not ready: {}",
                report
                    .gate_blockers_for_receipt(&self.session.export_receipts[receipt_index])
                    .join(", ")
            );
            self.queue.reject(action_id, reason.clone());
            self.refresh_view();
            return Err(JamAppError::InvalidSession(reason));
        }

        if let Err(error) = attach_daw_session_host_import_proof_evidence_to_receipt(
            &mut self.session.export_receipts[receipt_index],
            &report,
        ) {
            let message = format!("DAW session host-import proof evidence rejected: {error:?}");
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
            "committed DAW session host-import proof receipt {} proof {}",
            receipt.receipt_id,
            report.sha256.as_deref().unwrap_or("missing-proof-sha")
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
                    "queued DAW session host-import proof action {action_id} was not ready to commit"
                ))
            })?;
        let action = self
            .queue
            .history_action(committed_ref.action_id)
            .cloned()
            .ok_or_else(|| {
                JamAppError::InvalidSession(format!(
                    "committed DAW session host-import proof action {} missing from queue history",
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
                mc202_source_phrase_plan: None,
            });
        update_logged_action_result(&mut self.session, action_id, result_summary);
        self.runtime.last_commit_boundary = Some(boundary);
        self.refresh_view();

        Ok(receipt)
    }

    pub fn commit_daw_session_audible_output_proof_export(
        &mut self,
        proof_path: impl AsRef<Path>,
        requested_at: TimestampMs,
    ) -> Result<ExportReceiptState, JamAppError> {
        let proof_path = proof_path.as_ref();
        let action_id = match self.pending_daw_session_audible_output_proof_action_id(proof_path) {
            Some(action_id) => action_id,
            None => match self.queue_daw_session_audible_output_proof_export(
                requested_at,
                Some(proof_path.to_string_lossy().into_owned()),
            ) {
                DawSessionExportQueueResult::Enqueued { action_id } => action_id,
                DawSessionExportQueueResult::Rejected { reason } => {
                    return Err(JamAppError::InvalidSession(reason));
                }
                DawSessionExportQueueResult::AlreadyPending => {
                    return Err(JamAppError::InvalidSession(
                        "DAW session audible-output proof action is already pending".into(),
                    ));
                }
            },
        };

        let receipt_id = match self.pending_daw_session_proof_receipt_id(
            action_id,
            DawSessionExportBoundary::AudibleOutputProofV1,
        ) {
            Some(receipt_id) => receipt_id,
            None => {
                let message = format!(
                    "queued DAW session audible-output proof action {action_id} is missing a receipt"
                );
                self.queue.reject(action_id, message.clone());
                self.refresh_view();
                return Err(JamAppError::InvalidSession(message));
            }
        };
        let receipt_index = match daw_session_receipt_index_by_id(&self.session, &receipt_id) {
            Some(receipt_index) => receipt_index,
            None => {
                let message =
                    format!("DAW session audible-output proof receipt {receipt_id} is missing");
                self.queue.reject(action_id, message.clone());
                self.refresh_view();
                return Err(JamAppError::InvalidSession(message));
            }
        };
        let report = daw_session_audible_output_proof_report(proof_path);
        if !report.ready_for_receipt(&self.session.export_receipts[receipt_index]) {
            let reason = format!(
                "DAW session audible-output proof is not ready: {}",
                report
                    .gate_blockers_for_receipt(&self.session.export_receipts[receipt_index])
                    .join(", ")
            );
            self.queue.reject(action_id, reason.clone());
            self.refresh_view();
            return Err(JamAppError::InvalidSession(reason));
        }

        if let Err(error) = attach_daw_session_audible_output_proof_evidence_to_receipt(
            &mut self.session.export_receipts[receipt_index],
            &report,
        ) {
            let message = format!("DAW session audible-output proof evidence rejected: {error:?}");
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
            "committed DAW session audible-output proof receipt {} proof {}",
            receipt.receipt_id,
            report.sha256.as_deref().unwrap_or("missing-proof-sha")
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
                    "queued DAW session audible-output proof action {action_id} was not ready to commit"
                ))
            })?;
        let action = self
            .queue
            .history_action(committed_ref.action_id)
            .cloned()
            .ok_or_else(|| {
                JamAppError::InvalidSession(format!(
                    "committed DAW session audible-output proof action {} missing from queue history",
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
                mc202_source_phrase_plan: None,
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

    fn pending_daw_session_host_import_proof_action_id(
        &self,
        proof_path: &Path,
    ) -> Option<riotbox_core::ids::ActionId> {
        let expected_proof_path = proof_path.to_string_lossy();
        self.queue
            .pending_actions()
            .into_iter()
            .find(|action| {
                action.command == ActionCommand::ExportDawSession
                    && matches!(
                        &action.params,
                        ActionParams::DawSessionExport {
                            boundary: DawSessionExportBoundary::HostImportProofV1,
                            destination_path,
                            ..
                        } if destination_path.as_deref() == Some(expected_proof_path.as_ref())
                    )
            })
            .map(|action| action.id)
    }

    fn pending_daw_session_audible_output_proof_action_id(
        &self,
        proof_path: &Path,
    ) -> Option<riotbox_core::ids::ActionId> {
        let expected_proof_path = proof_path.to_string_lossy();
        self.queue
            .pending_actions()
            .into_iter()
            .find(|action| {
                action.command == ActionCommand::ExportDawSession
                    && matches!(
                        &action.params,
                        ActionParams::DawSessionExport {
                            boundary: DawSessionExportBoundary::AudibleOutputProofV1,
                            destination_path,
                            ..
                        } if destination_path.as_deref() == Some(expected_proof_path.as_ref())
                    )
            })
            .map(|action| action.id)
    }

    fn pending_daw_session_proof_receipt_id(
        &self,
        action_id: riotbox_core::ids::ActionId,
        expected_boundary: DawSessionExportBoundary,
    ) -> Option<String> {
        let action = self
            .queue
            .pending_actions()
            .into_iter()
            .find(|action| action.id == action_id)?;
        match &action.params {
            ActionParams::DawSessionExport {
                boundary,
                receipt_id: Some(receipt_id),
                ..
            } if *boundary == expected_boundary => Some(receipt_id.clone()),
            _ => None,
        }
    }
}

fn latest_daw_session_receipt_index(session: &riotbox_core::session::SessionFile) -> Option<usize> {
    session.export_receipts.iter().rposition(|receipt| {
        receipt.export_scope == riotbox_core::export_readiness::ExportScope::DawSession
    })
}

fn daw_session_receipt_index_by_id(
    session: &riotbox_core::session::SessionFile,
    receipt_id: &str,
) -> Option<usize> {
    session.export_receipts.iter().position(|receipt| {
        receipt.export_scope == riotbox_core::export_readiness::ExportScope::DawSession
            && receipt.receipt_id.as_str() == receipt_id
    })
}
