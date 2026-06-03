use std::{fs, path::Path, process::Command};

use riotbox_app::jam_app::daw_session_writer_plan;
use riotbox_core::{
    export_readiness::{
        ARRANGEMENT_DAW_PLACEMENT_PACK_ID, EXPORT_READINESS_CONTRACT_SCHEMA,
        ExportReadinessContract, ExportReadinessStatus, ExportScope, PRODUCT_EXPORT_PROOF_SCHEMA,
        ProductExportBoundary, ProductExportRole,
    },
    ids::{ActionId, SourceId},
    persistence::save_session_json,
    session::{
        ExportArrangementPlacementRef, ExportArtifactSetEntry, ExportDawTempoMapRef,
        ExportReceiptState, SessionFile,
    },
};
use serde_json::Value;

#[test]
fn daw_session_writer_plan_smoke_covers_ready_for_writer_and_missing_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("daw-out");
    let manifest_path = temp.path().join("exports/arrangement_manifest.json");
    let proof_path = temp.path().join("exports/proof.json");
    fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("create exports");
    fs::write(&manifest_path, "{}").expect("write manifest");
    fs::write(&proof_path, "{}").expect("write proof");
    let mut session = SessionFile::new(
        "daw-session-writer-plan-smoke",
        "riotbox-test",
        "2026-06-03T16:35:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    session.export_receipts.push(receipt);
    save_session_json(&session_path, &session).expect("save smoke session");

    let ready_plan = run_plan(&session_path, &destination_path);
    assert_eq!(ready_plan["status"], "ready_for_writer");
    assert_eq!(ready_plan["ready_for_writer"], true);
    assert_eq!(ready_plan["writes_files"], false);
    assert_eq!(
        ready_plan["writer_blockers"],
        Value::Array(vec!["daw_writer_missing".into()])
    );
    assert_eq!(ready_plan["readiness_blockers"], Value::Array(Vec::new()));
    assert_eq!(ready_plan["payload_preview"]["status"], "ready");
    assert_eq!(ready_plan["payload_preview"]["ready"], true);
    assert_eq!(
        ready_plan["payload_preview"]["proof"]["manifest_sha256"],
        ready_plan["payload_preview"]["manifest"]["normalized_json_sha256"]
    );
    assert_eq!(
        ready_plan["planned_artifacts"]
            .as_array()
            .expect("planned artifacts")
            .len(),
        3
    );
    assert!(!destination_path.exists());

    let missing_destination_plan =
        daw_session_writer_plan(&session, Some(temp.path()), Path::new(""));
    let missing_destination_plan =
        serde_json::to_value(missing_destination_plan).expect("serialize missing destination plan");
    assert_eq!(
        missing_destination_plan["readiness_blockers"],
        Value::Array(vec!["missing_destination_root".into()])
    );
    assert_eq!(
        missing_destination_plan["payload_preview"]["blockers"],
        Value::Array(vec!["missing_destination_root".into()])
    );
    assert_eq!(
        missing_destination_plan["payload_preview"]["manifest"],
        Value::Null
    );

    fs::remove_file(&manifest_path).expect("remove manifest");
    let missing_plan = run_plan(&session_path, &destination_path);
    assert_eq!(missing_plan["status"], "blocked");
    assert_eq!(missing_plan["ready_for_writer"], false);
    assert_eq!(
        missing_plan["readiness_blockers"],
        Value::Array(vec!["missing_local_files".into()])
    );
    assert_eq!(missing_plan["payload_preview"]["status"], "blocked");
    assert_eq!(
        missing_plan["payload_preview"]["blockers"],
        Value::Array(vec!["missing_local_files".into()])
    );
}

fn run_plan(session_path: &std::path::Path, destination_path: &std::path::Path) -> Value {
    run_riotbox_app_json([
        "--daw-session-writer-plan",
        "--session",
        session_path.to_str().expect("session path"),
        "--daw-session-destination",
        destination_path.to_str().expect("destination path"),
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
