#[test]
fn daw_session_host_import_proof_commits_through_export_action_without_enabling_surface() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("daw-session-export");
    let proof_path = temp.path().join("host_import_proof.json");
    let mut state = daw_session_writer_export_state(temp.path(), &destination, true);
    state
        .commit_daw_session_writer_export(Some(temp.path()), &destination, 980)
        .expect("commit writer proof prerequisite");
    write_host_import_proof(&proof_path, true, &[]);

    let queued = state.queue_daw_session_host_import_proof_export(
        990,
        Some(proof_path.to_string_lossy().into_owned()),
    );

    let action_id = match queued {
        DawSessionExportQueueResult::Enqueued { action_id } => action_id,
        other => panic!("expected host-import proof action to enqueue, got {other:?}"),
    };
    let pending = state.queue.pending_actions()[0];
    assert_eq!(pending.command, ActionCommand::ExportDawSession);
    match &pending.params {
        ActionParams::DawSessionExport {
            boundary,
            include_manifest,
            destination_kind,
            destination_path,
            receipt_id,
            ..
        } => {
            assert_eq!(
                *boundary,
                riotbox_core::action::DawSessionExportBoundary::HostImportProofV1
            );
            assert!(!include_manifest);
            assert_eq!(
                *destination_kind,
                ProductExportDestinationKind::LocalFilePath
            );
            assert_eq!(
                destination_path.as_deref(),
                Some(proof_path.to_string_lossy().as_ref())
            );
            assert_eq!(receipt_id.as_deref(), Some("export-receipt-a-0042"));
        }
        other => panic!("expected DAW session params, got {other:?}"),
    }

    let committed_receipt = state
        .commit_daw_session_host_import_proof_export(&proof_path, 1_000)
        .expect("commit DAW session host-import proof action");

    assert!(state.queue.pending_actions().is_empty());
    let saved_receipt = &state.session.export_receipts[0];
    assert_eq!(saved_receipt, &committed_receipt);
    let host_gate = saved_receipt
        .qa_gates
        .iter()
        .find(|gate| gate.gate_id == riotbox_core::session::DAW_SESSION_HOST_IMPORT_QA_GATE_ID)
        .expect("host-import proof gate");
    assert_eq!(host_gate.status, ExportReceiptQaGateStatus::Passed);
    assert!(
        saved_receipt
            .qa_gates
            .iter()
            .all(|gate| gate.gate_id
                != riotbox_core::session::DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID)
    );

    let action = state
        .session
        .action_log
        .actions
        .iter()
        .find(|action| action.id == action_id)
        .expect("committed host-import proof action");
    assert_eq!(action.command, ActionCommand::ExportDawSession);
    assert_eq!(action.status, ActionStatus::Committed);
    assert!(
        action
            .result
            .as_ref()
            .expect("action result")
            .summary
            .contains("committed DAW session host-import proof")
    );
    assert_eq!(state.session.action_log.commit_records.len(), 2);
    assert_eq!(state.session.action_log.commit_records[1].action_id, action_id);

    let surface_gate = state.daw_session_export_surface_gate();
    assert_eq!(
        surface_gate
            .blockers
            .iter()
            .map(|blocker| blocker.as_str())
            .collect::<Vec<_>>(),
        vec!["developer_proof_only", "audible_output_proof_missing"]
    );
}

#[test]
fn daw_session_host_import_proof_commits_to_queued_receipt_not_latest_receipt() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("daw-session-export");
    let proof_path = temp.path().join("host_import_proof.json");
    let mut state = daw_session_writer_export_state(temp.path(), &destination, true);
    state
        .commit_daw_session_writer_export(Some(temp.path()), &destination, 980)
        .expect("commit writer proof prerequisite");
    write_host_import_proof(&proof_path, true, &[]);

    let queued = state.queue_daw_session_host_import_proof_export(
        990,
        Some(proof_path.to_string_lossy().into_owned()),
    );
    assert!(matches!(
        queued,
        DawSessionExportQueueResult::Enqueued { .. }
    ));

    let mut later_receipt = state.session.export_receipts[0].clone();
    later_receipt.receipt_id = ExportReceiptId::from("export-receipt-a-0099");
    later_receipt.created_by_action = ActionId(99);
    state.session.export_receipts.push(later_receipt);

    let committed_receipt = state
        .commit_daw_session_host_import_proof_export(&proof_path, 1_000)
        .expect("commit DAW session host-import proof action");

    assert_eq!(committed_receipt.receipt_id.as_str(), "export-receipt-a-0042");
    assert!(
        state.session.export_receipts[0]
            .qa_gates
            .iter()
            .any(|gate| gate.gate_id
                == riotbox_core::session::DAW_SESSION_HOST_IMPORT_QA_GATE_ID
                && gate.status == ExportReceiptQaGateStatus::Passed)
    );
    assert!(
        state.session.export_receipts[1]
            .qa_gates
            .iter()
            .all(|gate| gate.gate_id
                != riotbox_core::session::DAW_SESSION_HOST_IMPORT_QA_GATE_ID)
    );
}

#[test]
fn daw_session_host_import_proof_rejects_when_queued_receipt_disappears() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("daw-session-export");
    let proof_path = temp.path().join("host_import_proof.json");
    let mut state = daw_session_writer_export_state(temp.path(), &destination, true);
    state
        .commit_daw_session_writer_export(Some(temp.path()), &destination, 980)
        .expect("commit writer proof prerequisite");
    write_host_import_proof(&proof_path, true, &[]);

    let queued = state.queue_daw_session_host_import_proof_export(
        990,
        Some(proof_path.to_string_lossy().into_owned()),
    );
    assert!(matches!(
        queued,
        DawSessionExportQueueResult::Enqueued { .. }
    ));
    state.session.export_receipts.clear();

    let error = state
        .commit_daw_session_host_import_proof_export(&proof_path, 1_000)
        .expect_err("stale queued receipt should reject");

    assert!(
        error
            .to_string()
            .contains("DAW session host-import proof receipt export-receipt-a-0042 is missing")
    );
    assert!(state.queue.pending_actions().is_empty());
    let rejected = state
        .queue
        .history()
        .iter()
        .find(|action| {
            action.command == ActionCommand::ExportDawSession
                && action.status == ActionStatus::Rejected
        })
        .expect("rejected host-import proof action");
    assert_eq!(rejected.status, ActionStatus::Rejected);
}

#[test]
fn daw_session_host_import_proof_rejects_without_writer_proof_before_mutation() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("daw-session-export");
    let proof_path = temp.path().join("host_import_proof.json");
    let mut state = daw_session_writer_export_state(temp.path(), &destination, true);
    write_host_import_proof(&proof_path, true, &[]);

    let error = state
        .commit_daw_session_host_import_proof_export(&proof_path, 990)
        .expect_err("host-import proof without writer proof should reject");

    assert!(error.to_string().contains("daw_writer_proof_missing"));
    assert!(state.queue.pending_actions().is_empty());
    assert!(state.session.action_log.actions.is_empty());
    assert!(state.session.action_log.commit_records.is_empty());
    assert!(
        state.session.export_receipts[0]
            .qa_gates
            .iter()
            .all(|gate| gate.gate_id
                != riotbox_core::session::DAW_SESSION_HOST_IMPORT_QA_GATE_ID)
    );
    let rejected = state
        .queue
        .history()
        .iter()
        .find(|action| action.command == ActionCommand::ExportDawSession)
        .expect("rejected host-import proof action");
    assert_eq!(rejected.status, ActionStatus::Rejected);
    match &rejected.params {
        ActionParams::DawSessionExport { boundary, .. } => assert_eq!(
            *boundary,
            riotbox_core::action::DawSessionExportBoundary::HostImportProofV1
        ),
        other => panic!("expected DAW session params, got {other:?}"),
    }
}

fn write_host_import_proof(path: &Path, imported: bool, blockers: &[&str]) {
    fs::write(
        path,
        serde_json::json!({
            "schema_id": crate::jam_app::DAW_SESSION_HOST_IMPORT_PROOF_SCHEMA_ID,
            "schema_version": crate::jam_app::DAW_SESSION_HOST_IMPORT_PROOF_SCHEMA_VERSION,
            "package_dir": "exports/daw-package/daw_session",
            "imported": imported,
            "blockers": blockers,
        })
        .to_string(),
    )
    .expect("write host-import proof");
}
