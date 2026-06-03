use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType,
        DawSessionExportBoundary, Quantization, TargetScope, UndoPolicy,
    },
    export_readiness::{ExportScope, ProductExportDestinationKind},
    queue::QueueEnqueueResult,
};

use crate::jam_app::JamAppState;

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
}
