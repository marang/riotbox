use super::product_export::{
    LIVE_RECORDING_EXPORT_RESERVED_REASON, LiveRecordingExportQueueResult,
};

#[test]
fn reserved_live_recording_export_queue_attempt_is_rejected_without_side_effects() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("live-recording-export");
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let result = state.queue_live_recording_export_reserved(
        970,
        Some(destination.to_string_lossy().into_owned()),
    );

    let reason = match result {
        LiveRecordingExportQueueResult::Rejected { reason } => reason,
        other => panic!("expected reserved live recording export rejection, got {other:?}"),
    };
    assert_eq!(reason, LIVE_RECORDING_EXPORT_RESERVED_REASON);
    assert!(reason.contains("future capture writer"));
    assert!(state.queue.pending_actions().is_empty());
    assert!(!destination.exists());
    assert!(state.session.export_receipts.is_empty());
    assert!(
        state
            .session
            .action_log
            .actions
            .iter()
            .all(|action| action.command != ActionCommand::ExportLiveRecording)
    );

    let rejected = state
        .queue
        .history()
        .iter()
        .find(|action| action.command == ActionCommand::ExportLiveRecording)
        .expect("reserved live recording action recorded in queue history");
    assert_eq!(rejected.status, ActionStatus::Rejected);
    assert_eq!(
        rejected.result.as_ref().map(|result| result.summary.as_str()),
        Some(reason.as_str())
    );
    assert!(matches!(rejected.undo_policy, UndoPolicy::NotUndoable { .. }));
    assert_eq!(rejected.target.scope, Some(TargetScope::Session));
    match &rejected.params {
        ActionParams::LiveRecordingExport {
            export_scope,
            export_role,
            boundary,
            include_manifest,
            destination_kind,
            destination_path,
            receipt_id,
        } => {
            assert_eq!(*export_scope, ExportScope::LiveRecording);
            assert_eq!(
                *export_role,
                riotbox_core::action::LiveRecordingExportRole::LiveRecordingCapture
            );
            assert_eq!(
                *boundary,
                riotbox_core::action::LiveRecordingExportBoundary::ReservedContractOnly
            );
            assert!(*include_manifest);
            assert_eq!(
                *destination_kind,
                ProductExportDestinationKind::LocalArtifactDirectory
            );
            assert_eq!(
                destination_path.as_deref(),
                Some(destination.to_string_lossy().as_ref())
            );
            assert_eq!(receipt_id, &None);
        }
        other => panic!("expected live recording params, got {other:?}"),
    }
}
