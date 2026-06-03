use std::{fs, path::Path, process::Command};

use riotbox_app::jam_app::{
    attach_daw_session_json_package_evidence_to_receipt, daw_session_json_package_report,
    daw_session_writer_plan, write_daw_session_json_package,
};
use riotbox_core::{
    export_readiness::{
        ARRANGEMENT_DAW_PLACEMENT_PACK_ID, EXPORT_READINESS_CONTRACT_SCHEMA,
        ExportReadinessContract, ExportReadinessStatus, ExportScope, PRODUCT_EXPORT_PROOF_SCHEMA,
        ProductExportBoundary, ProductExportRole,
    },
    ids::{ActionId, SourceId},
    persistence::save_session_json,
    session::{
        DAW_SESSION_JSON_PACKAGE_QA_GATE_ID, ExportArrangementPlacementRef, ExportArtifactRole,
        ExportArtifactSetEntry, ExportDawTempoMapRef, ExportReceiptQaGateStatus,
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
        ready_plan["payload_preview"]["tempo_map"]["schema_id"],
        "riotbox.daw_session_tempo_map"
    );
    assert!(
        ready_plan["payload_preview"]["tempo_map"]["normalized_json_sha256"]
            .as_str()
            .expect("tempo map hash")
            .len()
            >= 32
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
    assert_eq!(
        missing_destination_plan["payload_preview"]["tempo_map"],
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
    assert_eq!(missing_plan["payload_preview"]["tempo_map"], Value::Null);
}

#[test]
fn daw_session_json_writer_smoke_writes_only_explicit_json_package() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("daw-out");
    let manifest_path = temp.path().join("exports/arrangement_manifest.json");
    let proof_path = temp.path().join("exports/proof.json");
    fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("create exports");
    fs::write(&manifest_path, "{}").expect("write manifest");
    fs::write(&proof_path, "{}").expect("write proof");
    let mut session = SessionFile::new(
        "daw-session-json-writer-smoke",
        "riotbox-test",
        "2026-06-03T17:05:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    session.export_receipts.push(receipt);
    let before_session = serde_json::to_value(&session).expect("serialize session before writer");
    save_session_json(&session_path, &session).expect("save smoke session");

    let written = write_daw_session_json_package(&session, Some(temp.path()), &destination_path)
        .expect("write DAW session JSON proof package");

    assert!(written.package_dir.ends_with("daw-out/daw_session"));
    assert!(written.manifest_path.exists());
    assert!(written.tempo_map_path.exists());
    assert!(written.proof_path.exists());
    assert_eq!(written.manifest_sha256, written.proof_manifest_sha256);
    assert!(!destination_path.join(".daw_session_staging_42").exists());
    let manifest: Value =
        serde_json::from_slice(&fs::read(&written.manifest_path).expect("read manifest"))
            .expect("manifest json");
    let tempo_map: Value =
        serde_json::from_slice(&fs::read(&written.tempo_map_path).expect("read tempo map"))
            .expect("tempo map json");
    let proof: Value = serde_json::from_slice(&fs::read(&written.proof_path).expect("read proof"))
        .expect("proof json");
    assert_eq!(manifest["schema_id"], "riotbox.daw_session_manifest");
    assert_eq!(tempo_map["schema_id"], "riotbox.daw_session_tempo_map");
    assert_eq!(proof["schema_id"], "riotbox.daw_session_proof");
    assert_eq!(proof["manifest_sha256"], written.manifest_sha256);
    assert_eq!(
        serde_json::to_value(&session).expect("serialize session after writer"),
        before_session
    );

    let existing = write_daw_session_json_package(&session, Some(temp.path()), &destination_path)
        .expect_err("existing final package should reject");
    assert!(existing.to_string().contains("already exists"));

    let blocked_destination = temp.path().join("blocked-daw-out");
    fs::remove_file(&manifest_path).expect("remove source manifest");
    let blocked = write_daw_session_json_package(&session, Some(temp.path()), &blocked_destination)
        .expect_err("missing source file should block writer");
    assert!(
        blocked
            .to_string()
            .contains("DAW session JSON writer blocked")
    );
    assert!(!blocked_destination.exists());
}

#[test]
fn daw_session_json_package_execute_smoke_writes_only_explicit_json_package() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("daw-out");
    let manifest_path = temp.path().join("exports/arrangement_manifest.json");
    let proof_path = temp.path().join("exports/proof.json");
    fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("create exports");
    fs::write(&manifest_path, "{}").expect("write manifest");
    fs::write(&proof_path, "{}").expect("write proof");
    let mut session = SessionFile::new(
        "daw-session-json-package-execute-smoke",
        "riotbox-test",
        "2026-06-03T18:40:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    session.export_receipts.push(receipt);
    save_session_json(&session_path, &session).expect("save smoke session");
    let before_session = fs::read(&session_path).expect("read session before execute");

    let summary = run_riotbox_app_json([
        "--daw-session-json-package-execute",
        "--session",
        session_path.to_str().expect("session path"),
        "--daw-session-destination",
        destination_path.to_str().expect("destination path"),
    ]);

    assert_eq!(summary["mode"], "daw_session_json_package_execute");
    assert_eq!(summary["status"], "ready");
    assert_eq!(summary["writes_files"], true);
    assert_eq!(summary["mutates_session"], false);
    assert_eq!(summary["observer_events"], false);
    assert_eq!(summary["package_report"]["status"], "ready");
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
    assert!(
        destination_path
            .join("daw_session/arrangement_manifest.json")
            .exists()
    );
    assert!(destination_path.join("daw_session/tempo_map.json").exists());
    assert!(
        destination_path
            .join("daw_session/daw_session_proof.json")
            .exists()
    );
    assert_eq!(
        fs::read(&session_path).expect("read session after execute"),
        before_session
    );

    let existing = run_riotbox_app_json([
        "--daw-session-json-package-execute",
        "--session",
        session_path.to_str().expect("session path"),
        "--daw-session-destination",
        destination_path.to_str().expect("destination path"),
    ]);
    assert_eq!(existing["status"], "blocked");
    assert_eq!(existing["writes_files"], false);
    assert!(
        existing["readiness_blockers"][0]
            .as_str()
            .expect("blocker")
            .contains("already exists")
    );
}

#[test]
fn daw_session_json_package_report_smoke_validates_written_package() {
    let temp = tempfile::tempdir().expect("tempdir");
    let destination_path = temp.path().join("daw-out");
    let manifest_path = temp.path().join("exports/arrangement_manifest.json");
    let proof_path = temp.path().join("exports/proof.json");
    fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("create exports");
    fs::write(&manifest_path, "{}").expect("write manifest");
    fs::write(&proof_path, "{}").expect("write proof");
    let mut session = SessionFile::new(
        "daw-session-json-package-report-smoke",
        "riotbox-test",
        "2026-06-03T17:20:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    session.export_receipts.push(receipt);
    write_daw_session_json_package(&session, Some(temp.path()), &destination_path)
        .expect("write DAW session JSON proof package");

    let ready_report = serde_json::to_value(daw_session_json_package_report(&destination_path))
        .expect("serialize ready report");
    assert_eq!(ready_report["status"], "ready");
    assert_eq!(ready_report["ready"], true);
    assert_eq!(ready_report["writes_files"], false);
    assert_eq!(ready_report["blockers"], Value::Array(Vec::new()));
    assert_eq!(
        ready_report["artifacts"][0]["schema_id"],
        "riotbox.daw_session_manifest"
    );
    assert_eq!(
        ready_report["artifacts"][1]["schema_id"],
        "riotbox.daw_session_tempo_map"
    );
    assert_eq!(
        ready_report["artifacts"][2]["schema_id"],
        "riotbox.daw_session_proof"
    );

    let mut evidenced_receipt = session.export_receipts[0].clone();
    attach_daw_session_json_package_evidence_to_receipt(
        &mut evidenced_receipt,
        &daw_session_json_package_report(&destination_path),
    )
    .expect("attach DAW package evidence");
    assert_eq!(
        evidenced_receipt
            .artifact_set
            .iter()
            .map(|artifact| artifact.role)
            .collect::<Vec<_>>(),
        vec![
            ExportArtifactRole::ExportManifest,
            ExportArtifactRole::DawSessionTempoMap,
            ExportArtifactRole::ProductExportProof,
        ]
    );
    assert!(
        evidenced_receipt
            .artifact_set
            .iter()
            .all(|artifact| artifact.location_identity().contains("daw_session"))
    );
    let package_gate = evidenced_receipt
        .qa_gates
        .iter()
        .find(|gate| gate.gate_id == DAW_SESSION_JSON_PACKAGE_QA_GATE_ID)
        .expect("DAW package gate");
    assert_eq!(package_gate.status, ExportReceiptQaGateStatus::Passed);
    assert_eq!(
        package_gate.artifact_roles,
        vec![
            ExportArtifactRole::ExportManifest,
            ExportArtifactRole::DawSessionTempoMap,
            ExportArtifactRole::ProductExportProof,
        ]
    );

    let proof_file = destination_path.join("daw_session/daw_session_proof.json");
    let mut proof_json: Value =
        serde_json::from_slice(&fs::read(&proof_file).expect("read proof")).expect("proof json");
    proof_json["manifest_sha256"] = "wrong-manifest-sha".into();
    fs::write(
        &proof_file,
        serde_json::to_vec_pretty(&proof_json).expect("proof bytes"),
    )
    .expect("write mismatched proof");
    let mismatch_report = serde_json::to_value(daw_session_json_package_report(&destination_path))
        .expect("serialize mismatch report");
    assert_eq!(mismatch_report["status"], "blocked");
    assert_eq!(
        mismatch_report["blockers"],
        Value::Array(vec!["proof_manifest_hash_mismatch".into()])
    );
    let mut blocked_receipt = session.export_receipts[0].clone();
    attach_daw_session_json_package_evidence_to_receipt(
        &mut blocked_receipt,
        &daw_session_json_package_report(&destination_path),
    )
    .expect("attach blocked DAW package evidence");
    let blocked_gate = blocked_receipt
        .qa_gates
        .iter()
        .find(|gate| gate.gate_id == DAW_SESSION_JSON_PACKAGE_QA_GATE_ID)
        .expect("blocked DAW package gate");
    assert_eq!(blocked_gate.status, ExportReceiptQaGateStatus::Failed);
    assert!(
        blocked_gate
            .summary
            .as_deref()
            .expect("summary")
            .contains("proof_manifest_hash_mismatch")
    );

    let tempo_map_file = destination_path.join("daw_session/tempo_map.json");
    fs::remove_file(&tempo_map_file).expect("remove tempo map");
    let missing_report = serde_json::to_value(daw_session_json_package_report(&destination_path))
        .expect("serialize missing report");
    assert_eq!(missing_report["status"], "blocked");
    assert!(
        missing_report["blockers"]
            .as_array()
            .expect("blockers")
            .contains(&Value::String("missing_tempo_map_file".into()))
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
