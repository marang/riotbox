use std::{fs, process::Command};

use riotbox_app::jam_app::{
    attach_daw_session_json_package_evidence_to_receipt, daw_session_json_package_report,
    write_daw_session_json_package, write_daw_session_writer_proof_skeleton,
};
use riotbox_core::{
    action::{ActionCommand, ActionParams, ActionStatus, DawSessionExportBoundary},
    export_readiness::{
        ARRANGEMENT_DAW_PLACEMENT_PACK_ID, EXPORT_READINESS_CONTRACT_SCHEMA,
        ExportReadinessContract, ExportReadinessStatus, ExportScope, PRODUCT_EXPORT_PROOF_SCHEMA,
        ProductExportBoundary, ProductExportRole,
    },
    ids::{ActionId, SourceId},
    persistence::save_session_json,
    session::{
        DAW_SESSION_WRITER_QA_GATE_ID, ExportArrangementPlacementRef, ExportArtifactRole,
        ExportArtifactSetEntry, ExportDawTempoMapRef, ExportReceiptQaGateStatus,
        ExportReceiptState, SessionFile,
    },
};
use serde_json::Value;

#[test]
fn daw_session_writer_proof_smoke_writes_and_applies_only_writer_gate() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("daw-out");
    let manifest_path = temp.path().join("exports/arrangement_manifest.json");
    let proof_path = temp.path().join("exports/proof.json");
    fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("create exports");
    fs::write(&manifest_path, "{}").expect("write manifest");
    fs::write(&proof_path, "{}").expect("write proof");
    let mut session = SessionFile::new(
        "daw-session-writer-proof-smoke",
        "riotbox-test",
        "2026-06-03T20:25:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    session.export_receipts.push(receipt);
    write_daw_session_json_package(&session, Some(temp.path()), &destination_path)
        .expect("write DAW session JSON package");
    let mut evidenced_receipt = session.export_receipts[0].clone();
    attach_daw_session_json_package_evidence_to_receipt(
        &mut evidenced_receipt,
        &daw_session_json_package_report(&destination_path),
    )
    .expect("attach DAW JSON package evidence");
    session.export_receipts[0] = evidenced_receipt;
    save_session_json(&session_path, &session).expect("save smoke session");
    let before_execute = fs::read(&session_path).expect("read session before writer execute");

    let execute_summary = run_riotbox_app_json([
        "--daw-session-writer-proof-execute",
        "--session",
        session_path.to_str().expect("session path"),
        "--daw-session-destination",
        destination_path.to_str().expect("destination path"),
    ]);

    assert_eq!(execute_summary["mode"], "daw_session_writer_proof_execute");
    assert_eq!(execute_summary["status"], "ready");
    assert_eq!(execute_summary["writes_files"], true);
    assert_eq!(execute_summary["mutates_session"], false);
    assert_eq!(execute_summary["observer_events"], false);
    assert_eq!(
        execute_summary["boundary"],
        "daw_session.local_project_writer_v1"
    );
    assert_eq!(
        execute_summary["proof_report"]["receipt_id"],
        "export-receipt-a-0042"
    );
    assert!(
        destination_path
            .join("daw_session_writer/local_project_skeleton.json")
            .exists()
    );
    assert!(
        destination_path
            .join("daw_session_writer/writer_proof.json")
            .exists()
    );
    assert_eq!(
        fs::read(&session_path).expect("read session after writer execute"),
        before_execute
    );

    let apply_summary = run_riotbox_app_json([
        "--daw-session-writer-proof-apply",
        "--session",
        session_path.to_str().expect("session path"),
        "--daw-session-destination",
        destination_path.to_str().expect("destination path"),
    ]);

    assert_eq!(apply_summary["mode"], "daw_session_writer_proof_apply");
    assert_eq!(apply_summary["status"], "ready");
    assert_eq!(apply_summary["writes_files"], false);
    assert_eq!(apply_summary["mutates_session"], true);
    assert_eq!(apply_summary["observer_events"], false);
    assert_eq!(
        apply_summary["daw_session_surface_gate"]["blockers"],
        Value::Array(vec![
            "developer_proof_only".into(),
            "daw_host_import_proof_missing".into(),
            "audible_output_proof_missing".into(),
        ])
    );

    let saved_session =
        riotbox_core::persistence::load_session_json(&session_path).expect("load applied session");
    let saved_receipt = &saved_session.export_receipts[0];
    assert!(
        saved_receipt
            .artifact_set
            .iter()
            .any(|artifact| artifact.role == ExportArtifactRole::DawSessionWriterProof)
    );
    let writer_gate = saved_receipt
        .qa_gates
        .iter()
        .find(|gate| gate.gate_id == DAW_SESSION_WRITER_QA_GATE_ID)
        .expect("writer gate");
    assert_eq!(writer_gate.status, ExportReceiptQaGateStatus::Passed);
    assert_eq!(
        writer_gate.artifact_roles,
        vec![ExportArtifactRole::DawSessionWriterProof]
    );

    let report_summary = run_riotbox_app_json([
        "--daw-export-readiness-report",
        "--session",
        session_path.to_str().expect("session path"),
    ]);
    assert_eq!(
        report_summary["proof_gates"]["writer_proof"]["gate_id"],
        DAW_SESSION_WRITER_QA_GATE_ID
    );
    assert_eq!(
        report_summary["proof_gates"]["writer_proof"]["status"],
        "passed"
    );
    assert_eq!(
        report_summary["proof_gates"]["writer_proof"]["artifact_available"],
        true
    );
    assert_eq!(
        report_summary["proof_gates"]["writer_proof"]["artifact_roles"],
        Value::Array(vec!["daw_session_writer_proof".into()])
    );
    assert_eq!(
        report_summary["proof_gates"]["writer_proof"]["artifacts"][0]["role"],
        "daw_session_writer_proof"
    );
    assert_eq!(
        report_summary["daw_session_surface_gate"]["blockers"],
        Value::Array(vec![
            "developer_proof_only".into(),
            "daw_host_import_proof_missing".into(),
            "audible_output_proof_missing".into(),
        ])
    );
    assert_eq!(
        report_summary["release_blockers"],
        Value::Array(vec![
            "developer_proof_only".into(),
            "daw_host_import_proof_missing".into(),
            "audible_output_proof_missing".into(),
        ])
    );

    let existing = run_riotbox_app_json([
        "--daw-session-writer-proof-execute",
        "--session",
        session_path.to_str().expect("session path"),
        "--daw-session-destination",
        destination_path.to_str().expect("destination path"),
    ]);
    assert_eq!(existing["status"], "blocked");
    assert_eq!(existing["writes_files"], false);
}

#[test]
fn daw_session_writer_export_execute_smoke_commits_action_and_observer_lifecycle() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("daw-out");
    let observer_path = temp.path().join("observer.ndjson");
    let manifest_path = temp.path().join("exports/arrangement_manifest.json");
    let proof_path = temp.path().join("exports/proof.json");
    fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("create exports");
    fs::write(&manifest_path, "{}").expect("write manifest");
    fs::write(&proof_path, "{}").expect("write proof");
    let mut session = SessionFile::new(
        "daw-session-writer-export-smoke",
        "riotbox-test",
        "2026-06-04T01:05:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    session.export_receipts.push(receipt);
    write_daw_session_json_package(&session, Some(temp.path()), &destination_path)
        .expect("write DAW session JSON package");
    attach_daw_session_json_package_evidence_to_receipt(
        &mut session.export_receipts[0],
        &daw_session_json_package_report(&destination_path),
    )
    .expect("attach DAW JSON package evidence");
    save_session_json(&session_path, &session).expect("save smoke session");

    let summary = run_riotbox_app_json([
        "--daw-session-writer-export-execute",
        "--session",
        session_path.to_str().expect("session path"),
        "--daw-session-destination",
        destination_path.to_str().expect("destination path"),
        "--observer",
        observer_path.to_str().expect("observer path"),
    ]);

    assert_eq!(summary["mode"], "daw_session_writer_export_execute");
    assert_eq!(summary["status"], "ready");
    assert_eq!(summary["writes_files"], true);
    assert_eq!(summary["mutates_session"], true);
    assert_eq!(summary["observer_events"], true);
    assert_eq!(summary["boundary"], "daw_session.local_project_writer_v1");
    assert_eq!(
        summary["daw_session_surface_gate"]["blockers"],
        Value::Array(vec![
            "developer_proof_only".into(),
            "daw_host_import_proof_missing".into(),
            "audible_output_proof_missing".into(),
        ])
    );
    assert_eq!(
        summary["commit_records"].as_array().expect("records").len(),
        1
    );
    assert!(
        destination_path
            .join("daw_session_writer/local_project_skeleton.json")
            .exists()
    );
    assert!(
        destination_path
            .join("daw_session_writer/writer_proof.json")
            .exists()
    );

    let saved_session =
        riotbox_core::persistence::load_session_json(&session_path).expect("load saved session");
    let saved_receipt = &saved_session.export_receipts[0];
    assert!(
        saved_receipt
            .artifact_set
            .iter()
            .any(|artifact| artifact.role == ExportArtifactRole::DawSessionWriterProof)
    );
    let writer_gate = saved_receipt
        .qa_gates
        .iter()
        .find(|gate| gate.gate_id == DAW_SESSION_WRITER_QA_GATE_ID)
        .expect("writer gate");
    assert_eq!(writer_gate.status, ExportReceiptQaGateStatus::Passed);
    assert!(saved_receipt.qa_gates.iter().all(|gate| gate.gate_id
        != riotbox_core::session::DAW_SESSION_HOST_IMPORT_QA_GATE_ID
        && gate.gate_id != riotbox_core::session::DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID));

    let action = saved_session
        .action_log
        .actions
        .iter()
        .find(|action| action.command == ActionCommand::ExportDawSession)
        .expect("committed DAW session action");
    assert_eq!(action.status, ActionStatus::Committed);
    match &action.params {
        ActionParams::DawSessionExport {
            boundary,
            destination_path: action_destination,
            receipt_id,
            ..
        } => {
            assert_eq!(*boundary, DawSessionExportBoundary::LocalProjectWriterV1);
            assert_eq!(
                action_destination.as_deref(),
                Some(destination_path.to_string_lossy().as_ref())
            );
            assert_eq!(
                receipt_id.as_deref(),
                Some(saved_receipt.receipt_id.as_str())
            );
        }
        other => panic!("expected DAW session export params, got {other:?}"),
    }
    assert_eq!(saved_session.action_log.commit_records.len(), 1);
    assert_eq!(
        saved_session.action_log.commit_records[0].action_id,
        action.id
    );

    let observer_line = fs::read_to_string(&observer_path)
        .expect("read observer")
        .lines()
        .next()
        .expect("observer line")
        .to_owned();
    let observer: Value = serde_json::from_str(&observer_line).expect("observer json");
    assert_eq!(observer["event"], "daw_session_writer_export_execute");
    let lifecycle = observer["snapshot"]["export"]["lifecycle"]
        .as_array()
        .expect("observer lifecycle");
    assert_eq!(lifecycle.len(), 3);
    assert_eq!(lifecycle[0]["stage"], "requested");
    assert_eq!(lifecycle[1]["stage"], "started");
    assert_eq!(lifecycle[2]["stage"], "completed");
    assert_eq!(lifecycle[2]["command"], "export.daw_session");
    assert_eq!(
        lifecycle[2]["receipt"]["proof_gates"]["writer_proof"]["status"],
        "passed"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["proof_stack"]["missing_layers"],
        serde_json::json!(["host_import_proof", "audible_output_proof"])
    );
}

#[test]
fn daw_session_writer_proof_requires_json_package_evidence_gate() {
    let temp = tempfile::tempdir().expect("tempdir");
    let destination_path = temp.path().join("daw-out");
    let manifest_path = temp.path().join("exports/arrangement_manifest.json");
    let proof_path = temp.path().join("exports/proof.json");
    fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("create exports");
    fs::write(&manifest_path, "{}").expect("write manifest");
    fs::write(&proof_path, "{}").expect("write proof");
    let mut session = SessionFile::new(
        "daw-session-writer-proof-blocked",
        "riotbox-test",
        "2026-06-03T20:35:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    session.export_receipts.push(receipt);

    let blocked =
        write_daw_session_writer_proof_skeleton(&session, Some(temp.path()), &destination_path)
            .expect_err("missing JSON package gate should block writer proof");

    assert!(
        blocked
            .to_string()
            .contains("requires passed JSON package evidence")
    );
    assert!(!destination_path.exists());
}

#[test]
fn daw_session_writer_proof_requires_json_package_evidence_to_match_destination() {
    let temp = tempfile::tempdir().expect("tempdir");
    let attached_destination = temp.path().join("attached-daw-out");
    let other_destination = temp.path().join("other-daw-out");
    let manifest_path = temp.path().join("exports/arrangement_manifest.json");
    let proof_path = temp.path().join("exports/proof.json");
    fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("create exports");
    fs::write(&manifest_path, "{}").expect("write manifest");
    fs::write(&proof_path, "{}").expect("write proof");
    let mut session = SessionFile::new(
        "daw-session-writer-proof-mismatch",
        "riotbox-test",
        "2026-06-03T20:45:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    session.export_receipts.push(receipt);
    write_daw_session_json_package(&session, Some(temp.path()), &attached_destination)
        .expect("write attached DAW session JSON package");
    write_daw_session_json_package(&session, Some(temp.path()), &other_destination)
        .expect("write other DAW session JSON package");
    let mut evidenced_receipt = session.export_receipts[0].clone();
    attach_daw_session_json_package_evidence_to_receipt(
        &mut evidenced_receipt,
        &daw_session_json_package_report(&attached_destination),
    )
    .expect("attach attached DAW JSON package evidence");
    session.export_receipts[0] = evidenced_receipt;

    let blocked =
        write_daw_session_writer_proof_skeleton(&session, Some(temp.path()), &other_destination)
            .expect_err("mismatched JSON package destination should block writer proof");

    assert!(blocked.to_string().contains("package evidence mismatch"));
    assert!(!other_destination.join("daw_session_writer").exists());
}

fn run_riotbox_app_json<const N: usize>(args: [&str; N]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_riotbox-app"))
        .args(args)
        .output()
        .expect("run riotbox-app");
    if !output.status.success() {
        panic!(
            "riotbox-app failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    serde_json::from_slice(&output.stdout).expect("parse riotbox-app stdout json")
}

fn daw_receipt(artifact_path: &str, proof_path: &str) -> ExportReceiptState {
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

fn attach_ready_daw_refs(receipt: &mut ExportReceiptState) {
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
