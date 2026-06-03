use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType,
        LiveRecordingExportBoundary, LiveRecordingExportRole, Quantization, TargetScope,
        UndoPolicy,
    },
    export_readiness::{ExportScope, ProductExportDestinationKind},
    queue::QueueEnqueueResult,
};

use crate::jam_app::JamAppState;

pub const LIVE_RECORDING_EXPORT_RESERVED_REASON: &str = "live recording export is reserved for a future capture writer; Riotbox cannot record a live take to disk yet";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiveRecordingExportQueueResult {
    Rejected { reason: String },
    AlreadyPending,
}

impl JamAppState {
    pub fn queue_live_recording_export_reserved(
        &mut self,
        requested_at: TimestampMs,
        destination_path: Option<String>,
    ) -> LiveRecordingExportQueueResult {
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::ExportLiveRecording,
            Quantization::Immediate,
            ActionTarget {
                scope: Some(TargetScope::Session),
                ..Default::default()
            },
        );
        draft.params = ActionParams::LiveRecordingExport {
            export_scope: ExportScope::LiveRecording,
            export_role: LiveRecordingExportRole::LiveRecordingCapture,
            boundary: LiveRecordingExportBoundary::ReservedContractOnly,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
            destination_path,
            receipt_id: None,
        };
        draft.undo_policy = UndoPolicy::NotUndoable {
            reason: "live recording export would write capture files outside musical undo".into(),
        };
        draft.explanation =
            Some("reserved live recording export contract; not runnable yet".into());

        match self
            .queue
            .enqueue_if_no_pending_command(draft, requested_at)
        {
            QueueEnqueueResult::AlreadyPending { .. } => {
                LiveRecordingExportQueueResult::AlreadyPending
            }
            QueueEnqueueResult::Enqueued(action_id) => {
                let reason = LIVE_RECORDING_EXPORT_RESERVED_REASON.to_owned();
                self.queue.reject(action_id, reason.clone());
                self.refresh_view();
                LiveRecordingExportQueueResult::Rejected { reason }
            }
        }
    }
}
