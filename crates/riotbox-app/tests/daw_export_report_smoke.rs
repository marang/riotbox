use std::{fs, process::Command};

use riotbox_core::{
    export_readiness::{
        ARRANGEMENT_DAW_PLACEMENT_PACK_ID, EXPORT_READINESS_CONTRACT_SCHEMA,
        ExportReadinessContract, ExportReadinessStatus, ExportScope, PRODUCT_EXPORT_PROOF_SCHEMA,
        ProductExportBoundary, ProductExportRole,
    },
    ids::{ActionId, SourceId},
    persistence::save_session_json,
    session::{
        ExportArrangementPlacementRef, ExportArtifactRole, ExportArtifactSetEntry,
        ExportDawTempoMapRef, ExportReceiptQaGateResult, ExportReceiptState, SessionFile,
    },
};
use serde_json::{Value, json};

#[test]
fn daw_export_readiness_report_smoke_covers_ready_for_writer_and_missing_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let manifest_path = temp.path().join("exports/arrangement_manifest.json");
    let proof_path = temp.path().join("exports/proof.json");
    fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("create exports");
    fs::write(&manifest_path, "{}").expect("write manifest");
    fs::write(&proof_path, "{}").expect("write proof");
    let mut session = SessionFile::new(
        "daw-export-report-smoke",
        "riotbox-test",
        "2026-06-03T15:35:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    session.export_receipts.push(receipt);
    save_session_json(&session_path, &session).expect("save smoke session");

    let ready_report = run_report(&session_path);
    assert_eq!(ready_report["status"], "ready_for_writer");
    assert_eq!(ready_report["ready_for_next_gate"], true);
    assert_eq!(ready_report["writes_files"], false);
    assert_eq!(ready_report["developer_proof_status"], "ready_for_writer");
    assert_eq!(
        ready_report["proof_gates"]["writer_proof"]["status"],
        "missing"
    );
    assert_eq!(
        ready_report["proof_gates"]["writer_proof"]["artifact_available"],
        false
    );
    assert_eq!(
        ready_report["release_blockers"],
        Value::Array(vec![
            "developer_proof_only".into(),
            "daw_writer_missing".into(),
            "daw_host_import_proof_missing".into(),
            "audible_output_proof_missing".into(),
        ])
    );
    assert_eq!(ready_report["readiness_blockers"], Value::Array(Vec::new()));

    fs::remove_file(&manifest_path).expect("remove manifest");
    let missing_report = run_report(&session_path);
    assert_eq!(missing_report["status"], "blocked");
    assert_eq!(missing_report["ready_for_next_gate"], false);
    assert_eq!(
        missing_report["readiness_blockers"],
        Value::Array(vec!["missing_local_files".into()])
    );
    assert_eq!(
        missing_report["artifact_preflight"]["blockers"],
        Value::Array(vec!["missing_local_files".into()])
    );
}

#[test]
fn daw_export_readiness_report_smoke_covers_complete_developer_proof_stack() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let manifest_path = temp.path().join("exports/arrangement_manifest.json");
    let tempo_map_path = temp.path().join("exports/tempo_map.json");
    let proof_path = temp.path().join("exports/proof.json");
    let writer_proof_path = temp.path().join("exports/daw-session/writer_proof.json");
    fs::create_dir_all(writer_proof_path.parent().expect("writer proof parent"))
        .expect("create nested exports");
    fs::write(&manifest_path, "{}").expect("write manifest");
    fs::write(&tempo_map_path, "{}").expect("write tempo map");
    fs::write(&proof_path, "{}").expect("write proof");
    fs::write(&writer_proof_path, "{}").expect("write writer proof");

    let mut session = SessionFile::new(
        "daw-export-report-complete-proof-stack-smoke",
        "riotbox-test",
        "2026-06-04T11:20:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    attach_complete_developer_proof_stack(&mut receipt);
    session.export_receipts.push(receipt);
    save_session_json(&session_path, &session).expect("save smoke session");

    let report = run_report(&session_path);
    assert_eq!(report["status"], "ready_for_writer");
    assert_eq!(report["ready_for_next_gate"], true);
    assert_eq!(report["writes_files"], false);
    assert_eq!(
        report["proof_stack"]["status"],
        "complete_developer_proof_only"
    );
    assert_eq!(report["proof_stack"]["all_required_proofs_passed"], true);
    assert_eq!(report["proof_stack"]["missing_layers"], json!([]));
    assert_eq!(
        report["proof_gates"]["json_package_integrity"]["status"],
        "passed"
    );
    assert_eq!(report["proof_gates"]["writer_proof"]["status"], "passed");
    assert_eq!(
        report["proof_gates"]["writer_proof"]["artifact_available"],
        true
    );
    assert_eq!(
        report["proof_gates"]["host_import_proof"]["status"],
        "passed"
    );
    assert_eq!(
        report["proof_gates"]["audible_output_proof"]["status"],
        "passed"
    );
    assert_eq!(report["release_blockers"], json!(["developer_proof_only"]));
    assert_eq!(report["daw_session_surface_gate"]["status"], "disabled");
    assert_eq!(report["daw_session_surface_gate"]["runnable"], false);
    assert_eq!(
        report["daw_session_surface_gate"]["blockers"],
        json!(["developer_proof_only"])
    );
    assert_eq!(
        report["musician_export_readiness"],
        "not_final_daw_export_workflow"
    );
}

fn run_report(session_path: &std::path::Path) -> Value {
    run_riotbox_app_json([
        "--daw-export-readiness-report",
        "--session",
        session_path.to_str().expect("session path"),
    ])
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

fn attach_complete_developer_proof_stack(receipt: &mut ExportReceiptState) {
    receipt.artifact_set = vec![
        ExportArtifactSetEntry::export_manifest(
            "exports/arrangement_manifest.json",
            "manifest-sha",
        ),
        ExportArtifactSetEntry::daw_session_tempo_map("exports/tempo_map.json", "tempo-map-sha"),
        ExportArtifactSetEntry::product_export_proof("exports/proof.json", "proof-sha"),
        ExportArtifactSetEntry::daw_session_writer_proof(
            "exports/daw-session/writer_proof.json",
            "writer-proof-sha",
        ),
    ];
    receipt.qa_gates = vec![
        ExportReceiptQaGateResult::daw_session_json_package_integrity(
            true,
            &[],
            vec![
                ExportArtifactRole::ExportManifest,
                ExportArtifactRole::DawSessionTempoMap,
                ExportArtifactRole::ProductExportProof,
            ],
        ),
        ExportReceiptQaGateResult::daw_session_writer_proof(
            true,
            &[],
            vec![ExportArtifactRole::DawSessionWriterProof],
        ),
        ExportReceiptQaGateResult::daw_session_host_import_proof(true, &[]),
        ExportReceiptQaGateResult::daw_session_audible_output_proof(true, &[]),
    ];
}
