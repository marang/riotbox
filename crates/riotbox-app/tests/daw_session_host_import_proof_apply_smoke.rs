use std::{ffi::OsStr, fs, process::Command};

use riotbox_core::{
    action::{ActionCommand, ActionParams, ActionStatus, DawSessionExportBoundary},
    export_readiness::{
        ARRANGEMENT_DAW_PLACEMENT_PACK_ID, EXPORT_READINESS_CONTRACT_SCHEMA,
        ExportReadinessContract, ExportReadinessStatus, ExportScope, PRODUCT_EXPORT_PROOF_SCHEMA,
        ProductExportBoundary, ProductExportRole,
    },
    ids::{ActionId, SourceId},
    persistence::{load_session_json, save_session_json},
    session::{
        DAW_SESSION_HOST_IMPORT_QA_GATE_ID, ExportArrangementPlacementRef, ExportArtifactRole,
        ExportArtifactSetEntry, ExportDawTempoMapRef, ExportReceiptQaGateResult,
        ExportReceiptQaGateStatus, ExportReceiptState, SessionFile,
    },
};
use serde_json::{Value, json};

#[test]
fn daw_session_host_import_proof_apply_smoke_mutates_only_receipt_gate() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let proof_path = temp.path().join("host_import_proof.json");
    let mut session = SessionFile::new(
        "daw-session-host-import-proof-apply-smoke",
        "riotbox-test",
        "2026-06-03T19:50:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    session.export_receipts.push(receipt);
    save_session_json(&session_path, &session).expect("save smoke session");
    fs::write(
        &proof_path,
        json!({
            "schema_id": riotbox_app::jam_app::DAW_SESSION_HOST_IMPORT_PROOF_SCHEMA_ID,
            "schema_version": riotbox_app::jam_app::DAW_SESSION_HOST_IMPORT_PROOF_SCHEMA_VERSION,
            "package_dir": "exports/daw-package/daw_session",
            "imported": true,
            "blockers": []
        })
        .to_string(),
    )
    .expect("write proof");

    let summary = run_riotbox_app_json([
        "--daw-session-host-import-proof-apply",
        "--session",
        session_path.to_str().expect("session path"),
        "--daw-session-host-import-proof",
        proof_path.to_str().expect("proof path"),
    ]);

    assert_eq!(summary["mode"], "daw_session_host_import_proof_apply");
    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["writes_files"], false);
    assert_eq!(summary["mutates_session"], true);
    assert_eq!(
        summary["readiness_blockers"],
        Value::Array(vec!["daw_writer_proof_missing".into()])
    );
    assert_eq!(
        summary["receipt"]["daw_host_import_gate"]["gate_id"],
        DAW_SESSION_HOST_IMPORT_QA_GATE_ID
    );
    assert_eq!(
        summary["receipt"]["daw_host_import_gate"]["status"],
        "failed"
    );
    assert_eq!(
        summary["daw_session_surface_gate"]["blockers"],
        Value::Array(vec![
            "json_package_evidence_missing".into(),
            "developer_proof_only".into(),
            "daw_writer_missing".into(),
            "daw_host_import_proof_missing".into(),
            "audible_output_proof_missing".into(),
        ])
    );

    let mut saved = load_session_json(&session_path).expect("reload applied session");
    let saved_receipt = saved.export_receipts.last_mut().expect("saved receipt");
    saved_receipt
        .artifact_set
        .push(ExportArtifactSetEntry::daw_session_tempo_map(
            "exports/tempo_map.json",
            "tempo-map-sha",
        ));
    saved_receipt
        .artifact_set
        .push(ExportArtifactSetEntry::daw_session_writer_proof(
            "exports/daw_session_writer/writer_proof.json",
            "writer-proof-sha",
        ));
    saved_receipt.qa_gates.push(
        ExportReceiptQaGateResult::daw_session_json_package_integrity(
            true,
            &[],
            vec![
                ExportArtifactRole::ExportManifest,
                ExportArtifactRole::DawSessionTempoMap,
                ExportArtifactRole::ProductExportProof,
            ],
        ),
    );
    saved_receipt
        .qa_gates
        .push(ExportReceiptQaGateResult::daw_session_writer_proof(
            true,
            &[],
            vec![ExportArtifactRole::DawSessionWriterProof],
        ));
    save_session_json(&session_path, &saved).expect("save writer proof prerequisite");

    let ordered_summary = run_riotbox_app_json([
        "--daw-session-host-import-proof-apply",
        "--session",
        session_path.to_str().expect("session path"),
        "--daw-session-host-import-proof",
        proof_path.to_str().expect("proof path"),
    ]);

    assert_eq!(ordered_summary["status"], "ready");
    assert_eq!(ordered_summary["readiness_blockers"], Value::Array(vec![]));
    assert_eq!(
        ordered_summary["daw_session_surface_gate"]["blockers"],
        Value::Array(vec![
            "developer_proof_only".into(),
            "audible_output_proof_missing".into(),
        ])
    );

    let saved = load_session_json(&session_path).expect("reload ordered session");
    let saved_receipt = saved.export_receipts.last().expect("saved receipt");
    let gate = saved_receipt
        .qa_gates
        .iter()
        .find(|gate| gate.gate_id == DAW_SESSION_HOST_IMPORT_QA_GATE_ID)
        .expect("host import gate");
    assert_eq!(gate.status, ExportReceiptQaGateStatus::Passed);
}

#[test]
fn daw_session_host_import_proof_export_execute_smoke_commits_action_and_observer_lifecycle() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let proof_path = temp.path().join("host_import_proof.json");
    let observer_path = temp.path().join("observer.ndjson");
    let mut session = SessionFile::new(
        "daw-session-host-import-proof-export-smoke",
        "riotbox-test",
        "2026-06-04T17:20:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    attach_json_package_and_writer_prereqs(&mut receipt);
    session.export_receipts.push(receipt);
    save_session_json(&session_path, &session).expect("save smoke session");
    fs::write(
        &proof_path,
        json!({
            "schema_id": riotbox_app::jam_app::DAW_SESSION_HOST_IMPORT_PROOF_SCHEMA_ID,
            "schema_version": riotbox_app::jam_app::DAW_SESSION_HOST_IMPORT_PROOF_SCHEMA_VERSION,
            "package_dir": "exports/daw-package/daw_session",
            "imported": true,
            "blockers": []
        })
        .to_string(),
    )
    .expect("write proof");

    let summary = run_riotbox_app_json([
        "--daw-session-host-import-proof-export-execute",
        "--session",
        session_path.to_str().expect("session path"),
        "--daw-session-host-import-proof",
        proof_path.to_str().expect("proof path"),
        "--observer",
        observer_path.to_str().expect("observer path"),
    ]);

    assert_eq!(
        summary["mode"],
        "daw_session_host_import_proof_export_execute"
    );
    assert_eq!(summary["status"], "ready");
    assert_eq!(summary["writes_files"], false);
    assert_eq!(summary["mutates_session"], true);
    assert_eq!(summary["observer_events"], true);
    assert_eq!(summary["boundary"], "host_import_proof_v1");
    assert_eq!(
        summary["daw_session_surface_gate"]["blockers"],
        json!(["developer_proof_only", "audible_output_proof_missing"])
    );
    assert_eq!(
        summary["commit_records"].as_array().expect("records").len(),
        1
    );

    let saved_session = load_session_json(&session_path).expect("load saved session");
    let saved_receipt = saved_session.export_receipts.last().expect("saved receipt");
    let host_gate = saved_receipt
        .qa_gates
        .iter()
        .find(|gate| gate.gate_id == DAW_SESSION_HOST_IMPORT_QA_GATE_ID)
        .expect("host import gate");
    assert_eq!(host_gate.status, ExportReceiptQaGateStatus::Passed);
    assert!(
        saved_receipt.qa_gates.iter().all(
            |gate| gate.gate_id != riotbox_core::session::DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID
        )
    );

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
            destination_path,
            receipt_id,
            ..
        } => {
            assert_eq!(*boundary, DawSessionExportBoundary::HostImportProofV1);
            assert_eq!(
                destination_path.as_deref(),
                Some(proof_path.to_string_lossy().as_ref())
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
    assert_eq!(
        observer["event"],
        "daw_session_host_import_proof_export_execute"
    );
    let lifecycle = observer["snapshot"]["export"]["lifecycle"]
        .as_array()
        .expect("observer lifecycle");
    assert_eq!(lifecycle.len(), 3);
    assert_eq!(lifecycle[0]["stage"], "requested");
    assert_eq!(lifecycle[1]["stage"], "started");
    assert_eq!(lifecycle[2]["stage"], "completed");
    assert_eq!(lifecycle[2]["command"], "export.daw_session");
    assert_eq!(
        lifecycle[2]["receipt"]["proof_gates"]["host_import_proof"]["status"],
        "passed"
    );
    assert_eq!(
        lifecycle[2]["receipt"]["proof_stack"]["missing_layers"],
        json!(["audible_output_proof"])
    );
}

fn run_riotbox_app_json<I, S>(args: I) -> Value
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let binary = env!("CARGO_BIN_EXE_riotbox-app");
    let output = Command::new(binary)
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

fn attach_json_package_and_writer_prereqs(receipt: &mut ExportReceiptState) {
    receipt
        .artifact_set
        .push(ExportArtifactSetEntry::daw_session_tempo_map(
            "exports/tempo_map.json",
            "tempo-map-sha",
        ));
    receipt
        .artifact_set
        .push(ExportArtifactSetEntry::daw_session_writer_proof(
            "exports/daw_session_writer/writer_proof.json",
            "writer-proof-sha",
        ));
    receipt.qa_gates.push(
        ExportReceiptQaGateResult::daw_session_json_package_integrity(
            true,
            &[],
            vec![
                ExportArtifactRole::ExportManifest,
                ExportArtifactRole::DawSessionTempoMap,
                ExportArtifactRole::ProductExportProof,
            ],
        ),
    );
    receipt
        .qa_gates
        .push(ExportReceiptQaGateResult::daw_session_writer_proof(
            true,
            &[],
            vec![ExportArtifactRole::DawSessionWriterProof],
        ));
}
