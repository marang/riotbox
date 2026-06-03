use riotbox_core::{
    export_readiness::{
        ARRANGEMENT_DAW_PLACEMENT_PACK_ID, EXPORT_READINESS_CONTRACT_SCHEMA,
        ExportReadinessContract, ExportReadinessStatus, ExportScope, PRODUCT_EXPORT_PROOF_SCHEMA,
        ProductExportBoundary, ProductExportRole,
    },
    session::{
        ExportArrangementPlacementRef, ExportArtifactSetEntry, ExportDawTempoMapRef,
        ExportReceiptState,
    },
};
use riotbox_app::jam_app::{
    attach_daw_session_json_package_evidence_to_receipt, daw_session_json_package_report,
    write_daw_session_json_package,
};

#[test]
fn observer_snapshot_reports_committed_daw_session_writer_lifecycle() {
    let temp = tempfile::tempdir().expect("tempdir");
    let destination = temp.path().join("daw-session-export");
    let mut state = daw_session_writer_export_observer_state(temp.path(), &destination, true);

    let receipt = state
        .commit_daw_session_writer_export(Some(temp.path()), &destination, 980)
        .expect("commit DAW session writer proof");
    let action_id = state
        .session
        .action_log
        .actions
        .iter()
        .find(|action| action.command == ActionCommand::ExportDawSession)
        .expect("committed DAW session action")
        .id;

    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    let snapshot = observer_snapshot(&shell);
    let lifecycle = snapshot["export"]["lifecycle"]
        .as_array()
        .expect("lifecycle array");

    assert_eq!(lifecycle.len(), 3);
    assert_eq!(lifecycle[0]["stage"], "requested");
    assert_eq!(lifecycle[1]["stage"], "started");
    assert_eq!(lifecycle[2]["stage"], "completed");
    assert_eq!(lifecycle[2]["command"], "export.daw_session");
    assert_eq!(lifecycle[2]["action_id"], action_id.0);
    assert_eq!(
        lifecycle[2]["receipt"]["receipt_id"],
        receipt.receipt_id.to_string()
    );
    assert_eq!(lifecycle[2]["receipt"]["export_scope"], "daw_session");
    assert_eq!(
        lifecycle[2]["receipt"]["proof_gates"]["json_package_integrity"]["status"],
        "passed"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["proof_gates"]["writer_proof"]["status"],
        "passed"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["proof_gates"]["writer_proof"]["artifact_available"],
        true
    );
    assert_eq!(
        lifecycle[2]["receipt"]["proof_gates"]["host_import_proof"]["status"],
        "missing"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["proof_gates"]["audible_output_proof"]["status"],
        "missing"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["artifact_set"][3]["role"],
        "daw_session_writer_proof"
    );
    assert_eq!(
        snapshot["export"]["daw_session_receipt"]["receipt_id"],
        receipt.receipt_id.to_string()
    );
    assert_eq!(
        snapshot["export"]["daw_session_surface_gate"]["status"],
        "disabled"
    );
    assert_eq!(
        snapshot["export"]["daw_session_surface_gate"]["blockers"],
        serde_json::json!([
            "developer_proof_only",
            "daw_host_import_proof_missing",
            "audible_output_proof_missing"
        ])
    );
}

#[test]
fn observer_snapshot_reports_rejected_reserved_daw_session_lifecycle_without_receipt() {
    let temp = tempfile::tempdir().expect("tempdir");
    let destination = temp.path().join("daw-session-export");
    let mut state = daw_session_writer_export_observer_state(temp.path(), &destination, false);

    state.queue_daw_session_export_reserved(990, Some(destination.to_string_lossy().into_owned()));

    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    let snapshot = observer_snapshot(&shell);
    let lifecycle = snapshot["export"]["lifecycle"]
        .as_array()
        .expect("lifecycle array");

    assert_eq!(lifecycle.len(), 3);
    assert_eq!(lifecycle[0]["stage"], "requested");
    assert_eq!(lifecycle[1]["stage"], "started");
    assert_eq!(lifecycle[2]["stage"], "failed");
    assert_eq!(lifecycle[2]["command"], "export.daw_session");
    assert_eq!(lifecycle[2]["receipt"], serde_json::Value::Null);
    assert!(
        lifecycle[2]["failure_reason"]
            .as_str()
            .expect("failure reason")
            .contains("DAW session export is developer proof only")
    );
    assert!(
        lifecycle[2]["failure_reason"]
            .as_str()
            .expect("failure reason")
            .contains("DAW JSON package evidence is missing")
    );
    assert_eq!(
        snapshot["export"]["daw_session_surface_gate"]["status"],
        "disabled"
    );
}

#[test]
fn observer_snapshot_reports_committed_daw_session_host_import_lifecycle() {
    let temp = tempfile::tempdir().expect("tempdir");
    let destination = temp.path().join("daw-session-export");
    let proof_path = temp.path().join("host_import_proof.json");
    let mut state = daw_session_writer_export_observer_state(temp.path(), &destination, true);
    state
        .commit_daw_session_writer_export(Some(temp.path()), &destination, 980)
        .expect("commit writer proof prerequisite");
    write_host_import_observer_proof(&proof_path, true, &[]);
    let receipt = state
        .commit_daw_session_host_import_proof_export(&proof_path, 1_000)
        .expect("commit host-import proof");
    let action_id = state
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .find(|action| action.command == ActionCommand::ExportDawSession)
        .expect("committed DAW session host-import action")
        .id;

    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    let snapshot = observer_snapshot(&shell);
    let lifecycle = snapshot["export"]["lifecycle"]
        .as_array()
        .expect("lifecycle array");

    assert_eq!(lifecycle.len(), 6);
    assert_eq!(lifecycle[3]["stage"], "requested");
    assert_eq!(lifecycle[4]["stage"], "started");
    assert_eq!(lifecycle[5]["stage"], "completed");
    assert_eq!(lifecycle[5]["command"], "export.daw_session");
    assert_eq!(lifecycle[5]["action_id"], action_id.0);
    assert_eq!(
        lifecycle[5]["receipt"]["receipt_id"],
        receipt.receipt_id.to_string()
    );
    assert_eq!(
        lifecycle[5]["receipt"]["proof_gates"]["writer_proof"]["status"],
        "passed"
    );
    assert_eq!(
        lifecycle[5]["receipt"]["proof_gates"]["host_import_proof"]["status"],
        "passed"
    );
    assert_eq!(
        lifecycle[5]["receipt"]["proof_gates"]["audible_output_proof"]["status"],
        "missing"
    );
    assert_eq!(
        snapshot["export"]["daw_session_surface_gate"]["blockers"],
        serde_json::json!(["developer_proof_only", "audible_output_proof_missing"])
    );
}

fn daw_session_writer_export_observer_state(
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
        "observer-daw-session-writer-action",
        "riotbox-test",
        "2026-06-03T22:45:00Z",
    );
    let mut receipt = daw_session_writer_export_observer_receipt(
        "exports/arrangement_manifest.json",
        "exports/proof.json",
    );
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

fn daw_session_writer_export_observer_receipt(
    artifact_path: &str,
    proof_path: &str,
) -> ExportReceiptState {
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

fn write_host_import_observer_proof(path: &Path, imported: bool, blockers: &[&str]) {
    fs::write(
        path,
        serde_json::json!({
            "schema_id": riotbox_app::jam_app::DAW_SESSION_HOST_IMPORT_PROOF_SCHEMA_ID,
            "schema_version": riotbox_app::jam_app::DAW_SESSION_HOST_IMPORT_PROOF_SCHEMA_VERSION,
            "package_dir": "exports/daw-package/daw_session",
            "imported": imported,
            "blockers": blockers,
        })
        .to_string(),
    )
    .expect("write host-import proof");
}
