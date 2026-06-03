use super::product_export::DawSessionExportQueueResult;

#[test]
fn reserved_daw_session_export_queue_attempt_is_rejected_without_side_effects() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("daw-session-export");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    let contract = ExportReadinessContract {
        schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: PRODUCT_EXPORT_PROOF_SCHEMA.into(),
        export_scope: ExportScope::DawSession,
        boundary: ProductExportBoundary::ArrangementDawPlacementContractV1,
        pack_id: ARRANGEMENT_DAW_PLACEMENT_PACK_ID.into(),
        export_role: ProductExportRole::ArrangementManifest,
        export_artifact: "exports/arrangement_manifest.json".into(),
        source_sha256: "source-sha".into(),
        export_sha256: "manifest-sha".into(),
        normalized_manifest_sha256: "normalized-manifest-sha".into(),
        unsupported_scopes: Vec::new(),
    };
    let receipt = ExportReceiptState::from_readiness_contract(
        ActionId(42),
        91_000,
        &contract,
        "exports/arrangement_manifest.json",
        "exports/proof.json",
        Some("exports/arrangement_manifest.json".into()),
    );
    let receipt_id = receipt.receipt_id.as_str().to_owned();
    session.export_receipts.push(receipt);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let result = state.queue_daw_session_export_reserved(
        960,
        Some(destination.to_string_lossy().into_owned()),
    );

    let reason = match result {
        DawSessionExportQueueResult::Rejected { reason } => reason,
        other => panic!("expected reserved DAW session export rejection, got {other:?}"),
    };
    assert!(reason.contains("DAW session export is developer proof only"));
    assert!(reason.contains("DAW JSON package evidence is missing"));
    assert!(reason.contains("DAW project/session writer is missing"));
    assert!(state.queue.pending_actions().is_empty());
    assert!(!destination.exists());
    assert_eq!(state.session.export_receipts.len(), 1);
    assert!(
        state
            .session
            .action_log
            .actions
            .iter()
            .all(|action| action.command != ActionCommand::ExportDawSession)
    );

    let rejected = state
        .queue
        .history()
        .iter()
        .find(|action| action.command == ActionCommand::ExportDawSession)
        .expect("reserved DAW session action recorded in queue history");
    assert_eq!(rejected.status, ActionStatus::Rejected);
    assert_eq!(
        rejected.result.as_ref().map(|result| result.summary.as_str()),
        Some(reason.as_str())
    );
    assert!(matches!(rejected.undo_policy, UndoPolicy::NotUndoable { .. }));
    assert_eq!(rejected.target.scope, Some(TargetScope::Session));
    match &rejected.params {
        ActionParams::DawSessionExport {
            export_scope,
            boundary,
            include_manifest,
            destination_kind,
            destination_path,
            receipt_id: action_receipt_id,
        } => {
            assert_eq!(*export_scope, ExportScope::DawSession);
            assert_eq!(
                *boundary,
                riotbox_core::action::DawSessionExportBoundary::ReservedContractOnly
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
            assert_eq!(action_receipt_id.as_deref(), Some(receipt_id.as_str()));
        }
        other => panic!("expected DAW session params, got {other:?}"),
    }
}
