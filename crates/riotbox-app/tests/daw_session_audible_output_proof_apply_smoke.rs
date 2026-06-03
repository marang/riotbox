use std::{ffi::OsStr, fs, process::Command};

use riotbox_core::{
    export_readiness::{
        ARRANGEMENT_DAW_PLACEMENT_PACK_ID, EXPORT_READINESS_CONTRACT_SCHEMA,
        ExportReadinessContract, ExportReadinessStatus, ExportScope, PRODUCT_EXPORT_PROOF_SCHEMA,
        ProductExportBoundary, ProductExportRole,
    },
    ids::{ActionId, SourceId},
    persistence::{load_session_json, save_session_json},
    session::{
        DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID, ExportArrangementPlacementRef,
        ExportArtifactSetEntry, ExportDawTempoMapRef, ExportReceiptQaGateStatus,
        ExportReceiptState, SessionFile,
    },
};
use serde_json::{Value, json};

#[test]
fn daw_session_audible_output_proof_apply_smoke_mutates_only_receipt_gate() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let proof_path = temp.path().join("audible_output_proof.json");
    let mut session = SessionFile::new(
        "daw-session-audible-output-proof-apply-smoke",
        "riotbox-test",
        "2026-06-03T20:50:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    session.export_receipts.push(receipt);
    save_session_json(&session_path, &session).expect("save smoke session");
    fs::write(
        &proof_path,
        json!({
            "schema_id": riotbox_app::jam_app::DAW_SESSION_AUDIBLE_OUTPUT_PROOF_SCHEMA_ID,
            "schema_version": riotbox_app::jam_app::DAW_SESSION_AUDIBLE_OUTPUT_PROOF_SCHEMA_VERSION,
            "package_dir": "exports/daw-package/daw_session",
            "audible": true,
            "blockers": []
        })
        .to_string(),
    )
    .expect("write proof");

    let summary = run_riotbox_app_json([
        "--daw-session-audible-output-proof-apply",
        "--session",
        session_path.to_str().expect("session path"),
        "--daw-session-audible-output-proof",
        proof_path.to_str().expect("proof path"),
    ]);

    assert_eq!(summary["mode"], "daw_session_audible_output_proof_apply");
    assert_eq!(summary["status"], "ready");
    assert_eq!(summary["writes_files"], false);
    assert_eq!(summary["mutates_session"], true);
    assert_eq!(
        summary["receipt"]["daw_audible_output_gate"]["gate_id"],
        DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID
    );
    assert_eq!(
        summary["daw_session_surface_gate"]["blockers"],
        Value::Array(vec![
            "json_package_evidence_missing".into(),
            "developer_proof_only".into(),
            "daw_writer_missing".into(),
            "daw_host_import_proof_missing".into(),
        ])
    );

    let saved = load_session_json(&session_path).expect("reload applied session");
    let saved_receipt = saved.export_receipts.last().expect("saved receipt");
    let gate = saved_receipt
        .qa_gates
        .iter()
        .find(|gate| gate.gate_id == DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID)
        .expect("audible output gate");
    assert_eq!(gate.status, ExportReceiptQaGateStatus::Passed);
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
