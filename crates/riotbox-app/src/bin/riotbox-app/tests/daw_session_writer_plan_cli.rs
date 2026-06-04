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

#[test]
fn parse_args_builds_daw_session_writer_plan_mode() {
    let launch = parse_args([
        "--daw-session-writer-plan".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-destination".into(),
        "exports/daw-plan".into(),
    ])
    .expect("parse DAW session writer plan mode");

    assert_eq!(launch.observer_path, None);
    match launch.mode {
        LaunchMode::DawSessionWriterPlan {
            session_path,
            destination_path,
        } => {
            assert_eq!(session_path, PathBuf::from("session.json"));
            assert_eq!(destination_path, PathBuf::from("exports/daw-plan"));
        }
        LaunchMode::Load { .. }
        | LaunchMode::Ingest { .. }
        | LaunchMode::StemPackageLocalCiDryRun { .. }
        | LaunchMode::StemPackageLocalCiExecute { .. }
        | LaunchMode::StemPackageLocalCiReport { .. }
        | LaunchMode::LiveRecordingReadinessReport { .. }
        | LaunchMode::DawExportReadinessReport { .. }
        | LaunchMode::DawSessionJsonPackageExecute { .. }
        | LaunchMode::DawSessionJsonPackageEvidenceApply { .. }
        | LaunchMode::DawSessionHostImportProofApply { .. }
        | LaunchMode::DawSessionAudibleOutputProofApply { .. }
        | LaunchMode::DawSessionWriterProofExecute { .. }
        | LaunchMode::DawSessionWriterProofApply { .. } => {
            panic!("expected DAW session writer plan mode")
        }
    }
}

#[test]
fn parse_args_rejects_daw_session_writer_plan_without_required_inputs_or_with_write_args() {
    let missing_session = parse_args([
        "--daw-session-writer-plan".into(),
        "--daw-session-destination".into(),
        "exports/daw-plan".into(),
    ])
    .expect_err("session is required");
    assert!(missing_session.contains("--session"));

    let missing_destination = parse_args([
        "--daw-session-writer-plan".into(),
        "--session".into(),
        "session.json".into(),
    ])
    .expect_err("destination is required");
    assert!(missing_destination.contains("--daw-session-destination"));

    let observer_arg = parse_args([
        "--daw-session-writer-plan".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-destination".into(),
        "exports/daw-plan".into(),
        "--observer".into(),
        "observer.ndjson".into(),
    ])
    .expect_err("plan should not write observer files");
    assert!(observer_arg.contains("reads only an explicit session and destination"));

    let report_mix = parse_args([
        "--daw-session-writer-plan".into(),
        "--daw-export-readiness-report".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-destination".into(),
        "exports/daw-plan".into(),
    ])
    .expect_err("plan should not mix with readiness report");
    assert!(report_mix.contains("cannot be combined"));
}

#[test]
fn parse_args_rejects_daw_session_destination_outside_writer_plan_mode() {
    let dangling_destination = parse_args([
        "--session".into(),
        "session.json".into(),
        "--daw-session-destination".into(),
        "exports/daw-plan".into(),
    ])
    .expect_err("destination needs writer plan mode");
    assert!(dangling_destination.contains("--daw-session-writer-plan"));
    assert!(dangling_destination.contains("--daw-session-json-package-execute"));

    let readiness_destination = parse_args([
        "--daw-export-readiness-report".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-destination".into(),
        "exports/daw-plan".into(),
    ])
    .expect_err("readiness report should not ignore DAW session destination");
    assert!(readiness_destination.contains("DAW export readiness report"));

    let stem_destination = parse_args([
        "--stem-package-local-ci-dry-run".into(),
        "--stem-package-destination".into(),
        "exports/stems".into(),
        "--stem-role".into(),
        "stem_drums".into(),
        "--daw-session-destination".into(),
        "exports/daw-plan".into(),
    ])
    .expect_err("stem package mode should not ignore DAW session destination");
    assert!(stem_destination.contains("DAW destination"));
}

#[test]
fn daw_session_json_package_execute_writes_package_without_session_mutation() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("daw-out");
    let manifest_path = temp.path().join("exports/arrangement_manifest.json");
    let proof_path = temp.path().join("exports/proof.json");
    fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("create exports");
    fs::write(&manifest_path, "{}").expect("write manifest");
    fs::write(&proof_path, "{}").expect("write proof");
    let mut session = SessionFile::new(
        "daw-json-package-execute-ready",
        "riotbox-test",
        "2026-06-03T18:35:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    session.export_receipts.push(receipt);
    save_session_json(&session_path, &session).expect("save session");
    let before_session = fs::read(&session_path).expect("read session before execute");
    let launch = daw_json_package_execute_launch(session_path.clone(), destination_path.clone());

    let summary = daw_session_json_package_execute_summary(&launch).expect("execute summary");

    assert_eq!(summary["mode"], "daw_session_json_package_execute");
    assert_eq!(summary["status"], "ready");
    assert_eq!(summary["ready"], true);
    assert_eq!(summary["writes_files"], true);
    assert_eq!(summary["mutates_session"], false);
    assert_eq!(summary["observer_events"], false);
    assert!(
        summary["written_package"]["package_dir"]
            .as_str()
            .expect("package dir")
            .ends_with("daw-out/daw_session")
    );
    assert_eq!(summary["package_report"]["status"], "ready");
    assert_eq!(summary["package_report"]["ready"], true);
    assert_eq!(
        summary["daw_session_surface_gate"]["status"],
        "disabled"
    );
    assert_eq!(
        summary["daw_session_surface_gate"]["blockers"],
        json!([
            "json_package_evidence_missing",
            "developer_proof_only",
            "daw_writer_missing",
            "daw_host_import_proof_missing",
            "audible_output_proof_missing"
        ])
    );
    assert_eq!(
        summary["release_blockers"],
        json!([
            "developer_proof_only",
            "daw_writer_missing",
            "daw_host_import_proof_missing",
            "audible_output_proof_missing"
        ])
    );
    assert!(destination_path.join("daw_session/arrangement_manifest.json").exists());
    assert!(destination_path.join("daw_session/tempo_map.json").exists());
    assert!(destination_path.join("daw_session/daw_session_proof.json").exists());
    assert_eq!(
        fs::read(&session_path).expect("read session after execute"),
        before_session
    );
}

#[test]
fn daw_session_json_package_execute_blocks_without_daw_receipt() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("daw-out");
    save_session_json(
        &session_path,
        &SessionFile::new(
            "daw-json-package-execute-blocked",
            "riotbox-test",
            "2026-06-03T18:36:00Z",
        ),
    )
    .expect("save session");
    let launch = daw_json_package_execute_launch(session_path, destination_path.clone());

    let summary = daw_session_json_package_execute_summary(&launch).expect("execute summary");

    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["ready"], false);
    assert_eq!(summary["writes_files"], false);
    assert_eq!(summary["mutates_session"], false);
    assert!(
        summary["readiness_blockers"][0]
            .as_str()
            .expect("blocker")
            .contains("DAW session JSON writer blocked")
    );
    assert_eq!(
        summary["daw_session_surface_gate"]["blockers"],
        json!([
            "no_daw_session_receipt",
            "developer_proof_only",
            "daw_writer_missing",
            "daw_host_import_proof_missing",
            "audible_output_proof_missing"
        ])
    );
    assert!(!destination_path.exists());
}

#[test]
fn daw_session_writer_plan_blocks_without_daw_receipt_but_reports_planned_paths() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("daw-out");
    save_session_json(
        &session_path,
        &SessionFile::new(
            "daw-plan-no-receipt",
            "riotbox-test",
            "2026-06-03T16:10:00Z",
        ),
    )
    .expect("save session");
    let launch = daw_plan_launch(session_path, destination_path.clone());

    let summary = daw_session_writer_plan_summary(&launch).expect("plan summary");

    assert_eq!(summary["mode"], "daw_session_writer_plan");
    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["ready_for_writer"], false);
    assert_eq!(summary["writes_files"], false);
    assert_eq!(summary["readiness_blockers"], json!(["no_daw_session_receipt"]));
    assert_eq!(summary["writer_blockers"], json!(["daw_writer_missing"]));
    assert_eq!(summary["receipt"], serde_json::Value::Null);
    assert_eq!(summary["payload_preview"]["status"], "blocked");
    assert_eq!(summary["payload_preview"]["ready"], false);
    assert_eq!(
        summary["payload_preview"]["blockers"],
        json!(["no_daw_session_receipt"])
    );
    assert_eq!(summary["planned_artifacts"].as_array().expect("planned artifacts").len(), 3);
    assert!(!destination_path.exists());
}

#[test]
fn daw_session_writer_plan_surfaces_missing_source_files_after_contracts_are_ready() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("daw-out");
    let mut session = SessionFile::new(
        "daw-plan-missing-source",
        "riotbox-test",
        "2026-06-03T16:11:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    session.export_receipts.push(receipt);
    save_session_json(&session_path, &session).expect("save session");
    let launch = daw_plan_launch(session_path, destination_path);

    let summary = daw_session_writer_plan_summary(&launch).expect("plan summary");

    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["ready_for_writer"], false);
    assert_eq!(summary["readiness_blockers"], json!(["missing_local_files"]));
    assert_eq!(summary["payload_preview"]["status"], "blocked");
    assert_eq!(
        summary["payload_preview"]["blockers"],
        json!(["missing_local_files"])
    );
    assert_eq!(summary["payload_preview"]["manifest"], serde_json::Value::Null);
    assert_eq!(
        summary["payload_preview"]["tempo_map"],
        serde_json::Value::Null
    );
    assert_eq!(
        summary["operator_readiness"]["artifact_preflight"]["blockers"],
        json!(["missing_local_files"])
    );
    assert_eq!(summary["placement_refs"][0]["scene_id"], "scene-a");
    assert_eq!(summary["tempo_map_ref"]["source_id"], "src-1");
}

#[test]
fn daw_session_writer_plan_ready_for_writer_is_read_only_and_keeps_writer_blocker() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("daw-out");
    let manifest_path = temp.path().join("exports/arrangement_manifest.json");
    let proof_path = temp.path().join("exports/proof.json");
    fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("create exports");
    fs::write(&manifest_path, "{}").expect("write manifest");
    fs::write(&proof_path, "{}").expect("write proof");
    let mut session = SessionFile::new(
        "daw-plan-ready",
        "riotbox-test",
        "2026-06-03T16:12:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    session.export_receipts.push(receipt);
    save_session_json(&session_path, &session).expect("save session");
    let launch = daw_plan_launch(session_path, destination_path.clone());

    let summary = daw_session_writer_plan_summary(&launch).expect("plan summary");

    assert_eq!(summary["status"], "ready_for_writer");
    assert_eq!(summary["ready_for_writer"], true);
    assert_eq!(summary["writes_files"], false);
    assert_eq!(summary["readiness_blockers"], json!([]));
    assert_eq!(summary["writer_blockers"], json!(["daw_writer_missing"]));
    assert_eq!(summary["planned_artifacts"][0]["role"], "arrangement_manifest");
    assert_eq!(summary["planned_artifacts"][1]["role"], "tempo_map");
    assert_eq!(summary["planned_artifacts"][2]["role"], "daw_session_proof");
    assert_eq!(summary["payload_preview"]["status"], "ready");
    assert_eq!(summary["payload_preview"]["ready"], true);
    assert_eq!(summary["payload_preview"]["blockers"], json!([]));
    assert_eq!(
        summary["payload_preview"]["manifest"]["schema_id"],
        "riotbox.daw_session_manifest"
    );
    assert_eq!(
        summary["payload_preview"]["proof"]["schema_id"],
        "riotbox.daw_session_proof"
    );
    assert_eq!(
        summary["payload_preview"]["tempo_map"]["schema_id"],
        "riotbox.daw_session_tempo_map"
    );
    assert!(
        summary["payload_preview"]["tempo_map"]["normalized_json_sha256"]
            .as_str()
            .expect("tempo map hash")
            .len()
            >= 32
    );
    assert_eq!(
        summary["payload_preview"]["proof"]["manifest_sha256"],
        summary["payload_preview"]["manifest"]["normalized_json_sha256"]
    );
    assert!(
        summary["payload_preview"]["manifest"]["planned_path"]
            .as_str()
            .expect("manifest planned path")
            .ends_with("daw-out/daw_session/arrangement_manifest.json")
    );
    assert!(
        summary["payload_preview"]["tempo_map"]["planned_path"]
            .as_str()
            .expect("tempo map planned path")
            .ends_with("daw-out/daw_session/tempo_map.json")
    );
    assert!(
        summary["planned_artifacts"][0]["path"]
            .as_str()
            .expect("planned path")
            .ends_with("daw-out/daw_session/arrangement_manifest.json")
    );
    assert_eq!(summary["source_artifacts"][0]["role"], "arrangement_manifest");
    assert_eq!(summary["source_artifacts"][1]["role"], "product_export_proof");
    assert_eq!(summary["operator_readiness"]["status"], "ready_for_writer");
    assert!(!destination_path.exists());
}

fn daw_plan_launch(session_path: PathBuf, destination_path: PathBuf) -> AppLaunch {
    AppLaunch {
        mode: LaunchMode::DawSessionWriterPlan {
            session_path,
            destination_path,
        },
        observer_path: None,
    }
}

fn daw_json_package_execute_launch(session_path: PathBuf, destination_path: PathBuf) -> AppLaunch {
    AppLaunch {
        mode: LaunchMode::DawSessionJsonPackageExecute {
            session_path,
            destination_path,
        },
        observer_path: None,
    }
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
