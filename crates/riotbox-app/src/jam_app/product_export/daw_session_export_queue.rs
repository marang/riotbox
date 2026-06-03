use std::{fmt::Write as _, path::Path};

use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType,
        DawSessionExportBoundary, Quantization, TargetScope, UndoPolicy,
    },
    export_readiness::{ExportScope, ProductExportDestinationKind},
    queue::QueueEnqueueResult,
    session::{DAW_SESSION_JSON_PACKAGE_QA_GATE_ID, ExportReceiptQaGateStatus, ExportReceiptState},
};

use crate::jam_app::{
    JamAppState,
    daw_session_audible_output_proof::daw_session_audible_output_proof_report,
    daw_session_host_import_proof::daw_session_host_import_proof_report,
    daw_session_package_report::daw_session_json_package_report,
    daw_session_writer_plan::{DawSessionWriterPlanBlocker, daw_session_writer_plan},
};

use super::DawSessionExportQueueResult;

impl JamAppState {
    pub fn queue_daw_session_export_reserved(
        &mut self,
        requested_at: TimestampMs,
        destination_path: Option<String>,
    ) -> DawSessionExportQueueResult {
        let receipt_id = self
            .session
            .export_receipts
            .iter()
            .rev()
            .find(|receipt| receipt.export_scope == ExportScope::DawSession)
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
            boundary: DawSessionExportBoundary::ReservedContractOnly,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
            destination_path,
            receipt_id,
        };
        draft.undo_policy = UndoPolicy::NotUndoable {
            reason: "DAW session export writes files outside musical undo".into(),
        };
        draft.explanation = Some("reserved DAW session export contract; not runnable yet".into());

        match self
            .queue
            .enqueue_if_no_pending_command(draft, requested_at)
        {
            QueueEnqueueResult::AlreadyPending { .. } => {
                DawSessionExportQueueResult::AlreadyPending
            }
            QueueEnqueueResult::Enqueued(action_id) => {
                let reason = self.daw_session_export_surface_gate().musician_summary();
                self.queue.reject(action_id, reason.clone());
                self.refresh_view();
                DawSessionExportQueueResult::Rejected { reason }
            }
        }
    }

    pub fn queue_daw_session_writer_export(
        &mut self,
        requested_at: TimestampMs,
        session_base_dir: Option<&Path>,
        destination_path: Option<String>,
    ) -> DawSessionExportQueueResult {
        let receipt_id = latest_daw_session_receipt(&self.session)
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
            boundary: DawSessionExportBoundary::LocalProjectWriterV1,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
            destination_path: destination_path.clone(),
            receipt_id,
        };
        draft.undo_policy = UndoPolicy::NotUndoable {
            reason: "DAW session export writes files outside musical undo".into(),
        };
        draft.explanation =
            Some("commit local DAW session writer proof through export.daw_session".into());

        match self
            .queue
            .enqueue_if_no_pending_command(draft, requested_at)
        {
            QueueEnqueueResult::AlreadyPending { .. } => {
                DawSessionExportQueueResult::AlreadyPending
            }
            QueueEnqueueResult::Enqueued(action_id) => {
                let blockers = daw_session_writer_queue_blockers(
                    &self.session,
                    session_base_dir,
                    destination_path.as_deref(),
                );
                if blockers.is_empty() {
                    self.refresh_view();
                    return DawSessionExportQueueResult::Enqueued { action_id };
                }

                let reason = daw_session_writer_rejection_reason(&blockers);
                self.queue.reject(action_id, reason.clone());
                self.refresh_view();
                DawSessionExportQueueResult::Rejected { reason }
            }
        }
    }

    pub fn queue_daw_session_host_import_proof_export(
        &mut self,
        requested_at: TimestampMs,
        proof_path: Option<String>,
    ) -> DawSessionExportQueueResult {
        let receipt_id = latest_daw_session_receipt(&self.session)
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
            boundary: DawSessionExportBoundary::HostImportProofV1,
            include_manifest: false,
            destination_kind: ProductExportDestinationKind::LocalFilePath,
            destination_path: proof_path.clone(),
            receipt_id,
        };
        draft.undo_policy = UndoPolicy::NotUndoable {
            reason:
                "DAW session host-import proof mutates export receipt evidence outside musical undo"
                    .into(),
        };
        draft.explanation = Some("commit DAW host-import proof through export.daw_session".into());

        match self
            .queue
            .enqueue_if_no_pending_command(draft, requested_at)
        {
            QueueEnqueueResult::AlreadyPending { .. } => {
                DawSessionExportQueueResult::AlreadyPending
            }
            QueueEnqueueResult::Enqueued(action_id) => {
                let blockers =
                    daw_session_host_import_queue_blockers(&self.session, proof_path.as_deref());
                if blockers.is_empty() {
                    self.refresh_view();
                    return DawSessionExportQueueResult::Enqueued { action_id };
                }

                let reason = daw_session_host_import_rejection_reason(&blockers);
                self.queue.reject(action_id, reason.clone());
                self.refresh_view();
                DawSessionExportQueueResult::Rejected { reason }
            }
        }
    }

    pub fn queue_daw_session_audible_output_proof_export(
        &mut self,
        requested_at: TimestampMs,
        proof_path: Option<String>,
    ) -> DawSessionExportQueueResult {
        let receipt_id = latest_daw_session_receipt(&self.session)
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
            boundary: DawSessionExportBoundary::AudibleOutputProofV1,
            include_manifest: false,
            destination_kind: ProductExportDestinationKind::LocalFilePath,
            destination_path: proof_path.clone(),
            receipt_id,
        };
        draft.undo_policy = UndoPolicy::NotUndoable {
            reason:
                "DAW session audible-output proof mutates export receipt evidence outside musical undo"
                    .into(),
        };
        draft.explanation =
            Some("commit DAW audible-output proof through export.daw_session".into());

        match self
            .queue
            .enqueue_if_no_pending_command(draft, requested_at)
        {
            QueueEnqueueResult::AlreadyPending { .. } => {
                DawSessionExportQueueResult::AlreadyPending
            }
            QueueEnqueueResult::Enqueued(action_id) => {
                let blockers =
                    daw_session_audible_output_queue_blockers(&self.session, proof_path.as_deref());
                if blockers.is_empty() {
                    self.refresh_view();
                    return DawSessionExportQueueResult::Enqueued { action_id };
                }

                let reason = daw_session_audible_output_rejection_reason(&blockers);
                self.queue.reject(action_id, reason.clone());
                self.refresh_view();
                DawSessionExportQueueResult::Rejected { reason }
            }
        }
    }
}

fn latest_daw_session_receipt(
    session: &riotbox_core::session::SessionFile,
) -> Option<&ExportReceiptState> {
    session
        .export_receipts
        .iter()
        .rev()
        .find(|receipt| receipt.export_scope == ExportScope::DawSession)
}

fn daw_session_writer_queue_blockers(
    session: &riotbox_core::session::SessionFile,
    session_base_dir: Option<&Path>,
    destination_path: Option<&str>,
) -> Vec<String> {
    let destination_path = destination_path.unwrap_or("");
    let plan = daw_session_writer_plan(session, session_base_dir, Path::new(destination_path));
    let mut blockers = plan
        .readiness_blockers
        .iter()
        .map(|blocker| daw_session_writer_plan_blocker_label(*blocker).to_owned())
        .collect::<Vec<_>>();
    match latest_daw_session_receipt(session) {
        Some(receipt) if daw_json_package_gate_passed(receipt) => {
            let report = daw_session_json_package_report(Path::new(destination_path));
            if !report.ready {
                blockers.extend(
                    report
                        .blockers
                        .iter()
                        .map(|blocker| format!("json_package_{}", blocker.as_str())),
                );
            }
        }
        Some(_) => blockers.push("json_package_evidence_missing".into()),
        None => blockers.push("no_daw_session_receipt".into()),
    }
    blockers.sort();
    blockers.dedup();
    blockers
}

fn daw_json_package_gate_passed(receipt: &ExportReceiptState) -> bool {
    receipt.qa_gates.iter().any(|gate| {
        gate.gate_id == DAW_SESSION_JSON_PACKAGE_QA_GATE_ID
            && gate.status == ExportReceiptQaGateStatus::Passed
    })
}

fn daw_session_host_import_queue_blockers(
    session: &riotbox_core::session::SessionFile,
    proof_path: Option<&str>,
) -> Vec<String> {
    let Some(proof_path) = proof_path.filter(|path| !path.trim().is_empty()) else {
        return vec!["host_import_proof_path_missing".into()];
    };

    let Some(receipt) = latest_daw_session_receipt(session) else {
        return vec!["no_daw_session_receipt".into()];
    };

    let report = daw_session_host_import_proof_report(Path::new(proof_path));
    let mut blockers = report.gate_blockers_for_receipt(receipt);
    blockers.sort();
    blockers.dedup();
    blockers
}

fn daw_session_host_import_rejection_reason(blockers: &[String]) -> String {
    let mut reason = String::from("DAW session host-import proof is not ready");
    if !blockers.is_empty() {
        let _ = write!(reason, ": {}", blockers.join(", "));
    }
    reason
}

fn daw_session_audible_output_queue_blockers(
    session: &riotbox_core::session::SessionFile,
    proof_path: Option<&str>,
) -> Vec<String> {
    let Some(proof_path) = proof_path.filter(|path| !path.trim().is_empty()) else {
        return vec!["audible_output_proof_path_missing".into()];
    };

    let Some(receipt) = latest_daw_session_receipt(session) else {
        return vec!["no_daw_session_receipt".into()];
    };

    let report = daw_session_audible_output_proof_report(Path::new(proof_path));
    let mut blockers = report.gate_blockers_for_receipt(receipt);
    blockers.sort();
    blockers.dedup();
    blockers
}

fn daw_session_audible_output_rejection_reason(blockers: &[String]) -> String {
    let mut reason = String::from("DAW session audible-output proof is not ready");
    if !blockers.is_empty() {
        let _ = write!(reason, ": {}", blockers.join(", "));
    }
    reason
}

fn daw_session_writer_rejection_reason(blockers: &[String]) -> String {
    let mut reason = "DAW session writer export is not ready".to_owned();
    if !blockers.is_empty() {
        let _ = write!(reason, "; blockers: {}", blockers.join(", "));
    }
    reason
}

const fn daw_session_writer_plan_blocker_label(
    blocker: DawSessionWriterPlanBlocker,
) -> &'static str {
    match blocker {
        DawSessionWriterPlanBlocker::NoDawSessionReceipt => "no_daw_session_receipt",
        DawSessionWriterPlanBlocker::MissingDestinationRoot => "missing_destination_root",
        DawSessionWriterPlanBlocker::UnsupportedCommandBoundary => "unsupported_command_boundary",
        DawSessionWriterPlanBlocker::ArrangementPlacementBlocked => "arrangement_placement_blocked",
        DawSessionWriterPlanBlocker::DawTempoMapBlocked => "daw_tempo_map_blocked",
        DawSessionWriterPlanBlocker::MissingArtifactIdentity => "missing_artifact_identity",
        DawSessionWriterPlanBlocker::MissingLocalFiles => "missing_local_files",
        DawSessionWriterPlanBlocker::UnreadableLocalFiles => "unreadable_local_files",
    }
}
