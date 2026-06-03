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

#[test]
fn daw_session_writer_export_commits_local_proof_without_enabling_surface() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("daw-session-export");
    let mut state = daw_session_writer_export_state(temp.path(), &destination, true);

    let queued = state.queue_daw_session_writer_export(
        960,
        Some(temp.path()),
        Some(destination.to_string_lossy().into_owned()),
    );

    let action_id = match queued {
        DawSessionExportQueueResult::Enqueued { action_id } => action_id,
        other => panic!("expected DAW session writer export to enqueue, got {other:?}"),
    };
    assert_eq!(state.queue.pending_actions().len(), 1);
    let pending = state.queue.pending_actions()[0];
    assert_eq!(pending.command, ActionCommand::ExportDawSession);
    match &pending.params {
        ActionParams::DawSessionExport {
            boundary,
            destination_path,
            receipt_id,
            ..
        } => {
            assert_eq!(
                *boundary,
                riotbox_core::action::DawSessionExportBoundary::LocalProjectWriterV1
            );
            assert_eq!(
                destination_path.as_deref(),
                Some(destination.to_string_lossy().as_ref())
            );
            assert_eq!(receipt_id.as_deref(), Some("export-receipt-a-0042"));
        }
        other => panic!("expected DAW session writer params, got {other:?}"),
    }

    let committed_receipt = state
        .commit_daw_session_writer_export(Some(temp.path()), &destination, 980)
        .expect("commit DAW session writer proof action");

    assert!(state.queue.pending_actions().is_empty());
    assert!(
        destination
            .join("daw_session_writer/local_project_skeleton.json")
            .exists()
    );
    assert!(
        destination
            .join("daw_session_writer/writer_proof.json")
            .exists()
    );
    assert_eq!(state.session.export_receipts.len(), 1);
    let saved_receipt = &state.session.export_receipts[0];
    assert_eq!(saved_receipt, &committed_receipt);
    assert!(
        saved_receipt
            .artifact_set
            .iter()
            .any(|artifact| artifact.role == ExportArtifactRole::DawSessionWriterProof)
    );
    let writer_gate = saved_receipt
        .qa_gates
        .iter()
        .find(|gate| gate.gate_id == riotbox_core::session::DAW_SESSION_WRITER_QA_GATE_ID)
        .expect("writer proof gate");
    assert_eq!(writer_gate.status, ExportReceiptQaGateStatus::Passed);
    assert_eq!(
        writer_gate.artifact_roles,
        vec![ExportArtifactRole::DawSessionWriterProof]
    );
    assert!(
        saved_receipt
            .qa_gates
            .iter()
            .all(|gate| gate.gate_id != riotbox_core::session::DAW_SESSION_HOST_IMPORT_QA_GATE_ID
                && gate.gate_id
                    != riotbox_core::session::DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID)
    );

    let action = state
        .session
        .action_log
        .actions
        .iter()
        .find(|action| action.id == action_id)
        .expect("committed DAW session writer action");
    assert_eq!(action.command, ActionCommand::ExportDawSession);
    assert_eq!(action.status, ActionStatus::Committed);
    assert!(
        action
            .result
            .as_ref()
            .expect("action result")
            .summary
            .contains("wrote local DAW session writer proof")
    );
    assert_eq!(state.session.action_log.commit_records.len(), 1);
    assert_eq!(state.session.action_log.commit_records[0].action_id, action_id);

    let surface_gate = state.daw_session_export_surface_gate();
    assert_eq!(
        surface_gate
            .blockers
            .iter()
            .map(|blocker| blocker.as_str())
            .collect::<Vec<_>>(),
        vec![
            "developer_proof_only",
            "daw_host_import_proof_missing",
            "audible_output_proof_missing",
        ]
    );
}

#[test]
fn daw_session_writer_export_queue_rejects_without_json_package_evidence() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("daw-session-export");
    let mut state = daw_session_writer_export_state(temp.path(), &destination, false);

    let result = state.queue_daw_session_writer_export(
        960,
        Some(temp.path()),
        Some(destination.to_string_lossy().into_owned()),
    );

    let reason = match result {
        DawSessionExportQueueResult::Rejected { reason } => reason,
        other => panic!("expected DAW session writer export rejection, got {other:?}"),
    };
    assert!(reason.contains("json_package_evidence_missing"));
    assert!(state.queue.pending_actions().is_empty());
    assert!(!destination.join("daw_session_writer").exists());
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
        .expect("rejected DAW session writer action");
    assert_eq!(rejected.status, ActionStatus::Rejected);
    match &rejected.params {
        ActionParams::DawSessionExport { boundary, .. } => assert_eq!(
            *boundary,
            riotbox_core::action::DawSessionExportBoundary::LocalProjectWriterV1
        ),
        other => panic!("expected DAW session writer params, got {other:?}"),
    }
}

#[test]
fn daw_session_writer_export_commit_rejects_pending_destination_mismatch() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("daw-session-export");
    let other_destination = temp.path().join("other-daw-session-export");
    let mut state = daw_session_writer_export_state(temp.path(), &destination, true);

    let queued = state.queue_daw_session_writer_export(
        960,
        Some(temp.path()),
        Some(destination.to_string_lossy().into_owned()),
    );
    assert!(matches!(
        queued,
        DawSessionExportQueueResult::Enqueued { .. }
    ));

    let error = state
        .commit_daw_session_writer_export(Some(temp.path()), &other_destination, 980)
        .expect_err("pending destination mismatch should not commit");

    assert!(error.to_string().contains("already pending"));
    assert_eq!(state.queue.pending_actions().len(), 1);
    assert!(!other_destination.join("daw_session_writer").exists());
    assert!(state.session.action_log.actions.is_empty());
    assert!(state.session.action_log.commit_records.is_empty());
}

fn daw_session_writer_export_state(
    base_dir: &Path,
    destination: &Path,
    attach_json_evidence: bool,
) -> JamAppState {
    let manifest_path = base_dir.join("exports/arrangement_manifest.json");
    let proof_path = base_dir.join("exports/proof.json");
    fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("create exports");
    fs::write(&manifest_path, "{}").expect("write manifest");
    fs::write(&proof_path, "{}").expect("write proof");
    let mut session = SessionFile::new(
        "daw-session-writer-action-test",
        "riotbox-test",
        "2026-06-03T22:05:00Z",
    );
    let mut receipt = daw_session_writer_receipt(
        "exports/arrangement_manifest.json",
        "exports/proof.json",
    );
    attach_ready_daw_writer_refs(&mut receipt);
    session.export_receipts.push(receipt);
    if attach_json_evidence {
        write_daw_session_json_package(&session, Some(base_dir), destination)
            .expect("write DAW session JSON package");
        let mut evidenced_receipt = session.export_receipts[0].clone();
        attach_daw_session_json_package_evidence_to_receipt(
            &mut evidenced_receipt,
            &daw_session_json_package_report(destination),
        )
        .expect("attach DAW JSON package evidence");
        session.export_receipts[0] = evidenced_receipt;
    }

    JamAppState::from_parts(session, None, ActionQueue::new())
}

fn daw_session_writer_receipt(artifact_path: &str, proof_path: &str) -> ExportReceiptState {
    let contract = ExportReadinessContract {
        schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: PRODUCT_EXPORT_PROOF_SCHEMA.into(),
        export_scope: ExportScope::DawSession,
        boundary: ProductExportBoundary::ArrangementDawPlacementContractV1,
        pack_id: ARRANGEMENT_DAW_PLACEMENT_PACK_ID.into(),
        export_role: ProductExportRole::ArrangementManifest,
        export_artifact: artifact_path.into(),
        source_sha256: "source-sha".into(),
        export_sha256: "manifest-sha".into(),
        normalized_manifest_sha256: "normalized-manifest-sha".into(),
        unsupported_scopes: Vec::new(),
    };
    let mut receipt = ExportReceiptState::from_readiness_contract(
        ActionId(42),
        91_000,
        &contract,
        artifact_path,
        proof_path,
        Some(artifact_path.into()),
    );
    receipt.artifact_set = vec![
        ExportArtifactSetEntry::export_manifest(artifact_path, "manifest-sha"),
        ExportArtifactSetEntry::product_export_proof(proof_path, "proof-sha"),
    ];
    receipt
}

fn attach_ready_daw_writer_refs(receipt: &mut ExportReceiptState) {
    receipt
        .arrangement_placement_refs
        .push(ExportArrangementPlacementRef::scene_range(
            "scene-a",
            Some(SourceId::from("src-1")),
            1,
            4,
            0,
            16,
        ));
    receipt.daw_tempo_map_ref = Some(ExportDawTempoMapRef::confirmed_grid(
        "src-1",
        Some("primary-grid".into()),
        ActionId(8),
        880,
        0,
        16,
        128_000_000,
    ));
}
