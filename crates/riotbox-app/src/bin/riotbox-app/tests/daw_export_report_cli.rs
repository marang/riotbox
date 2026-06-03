use riotbox_core::{
    export_readiness::{
        ARRANGEMENT_DAW_PLACEMENT_PACK_ID, EXPORT_READINESS_CONTRACT_SCHEMA,
        ExportReadinessContract, ExportReadinessStatus, ExportScope, PRODUCT_EXPORT_PROOF_SCHEMA,
        ProductExportBoundary, ProductExportRole, UnsupportedExportScope,
    },
    session::{
        ExportArrangementPlacementRef, ExportArtifactSetEntry, ExportDawTempoMapRef,
        ExportReceiptState,
    },
};

#[test]
fn parse_args_builds_daw_export_readiness_report_mode() {
    let launch = parse_args([
        "--daw-export-readiness-report".into(),
        "--session".into(),
        "session.json".into(),
    ])
    .expect("parse DAW export report mode");

    assert_eq!(launch.observer_path, None);
    match launch.mode {
        LaunchMode::DawExportReadinessReport { session_path } => {
            assert_eq!(session_path, PathBuf::from("session.json"));
        }
        LaunchMode::Load { .. }
        | LaunchMode::Ingest { .. }
        | LaunchMode::StemPackageLocalCiDryRun { .. }
        | LaunchMode::StemPackageLocalCiExecute { .. }
        | LaunchMode::StemPackageLocalCiReport { .. } => {
            panic!("expected DAW export readiness report mode")
        }
    }
}

#[test]
fn parse_args_rejects_daw_export_report_without_session_or_with_write_args() {
    let missing_session =
        parse_args(["--daw-export-readiness-report".into()]).expect_err("session is required");
    assert!(missing_session.contains("--session"));

    let observer_arg = parse_args([
        "--daw-export-readiness-report".into(),
        "--session".into(),
        "session.json".into(),
        "--observer".into(),
        "observer.ndjson".into(),
    ])
    .expect_err("report should not write observer files");
    assert!(observer_arg.contains("reads only an explicit session"));

    let write_args = parse_args([
        "--daw-export-readiness-report".into(),
        "--session".into(),
        "session.json".into(),
        "--stem-package-destination".into(),
        "exports/stem-proof".into(),
    ])
    .expect_err("report should reject write-shaped args");
    assert!(write_args.contains("reads only an explicit session"));
}

#[test]
fn daw_export_report_blocks_without_daw_session_receipt() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    save_session_json(
        &session_path,
        &SessionFile::new(
            "daw-report-no-receipt",
            "riotbox-test",
            "2026-06-03T15:10:00Z",
        ),
    )
    .expect("save session");
    let report_launch = daw_report_launch(session_path);

    let summary = daw_export_readiness_report_summary(&report_launch).expect("report summary");

    assert_eq!(summary["mode"], "daw_export_readiness_report");
    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["ready_for_next_gate"], false);
    assert_eq!(summary["writes_files"], false);
    assert_eq!(summary["developer_proof_status"], "no_daw_session_receipt");
    assert_eq!(
        summary["musician_export_readiness"],
        "not_final_daw_export_workflow"
    );
    assert_eq!(
        summary["release_blockers"],
        json!(["developer_proof_only", "daw_writer_missing"])
    );
    assert_eq!(
        summary["readiness_blockers"],
        json!(["no_daw_session_receipt"])
    );
    assert_eq!(summary["receipt"], serde_json::Value::Null);
}

#[test]
fn daw_export_report_distinguishes_unsupported_placement_and_tempo_blockers() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let mut session = SessionFile::new(
        "daw-report-blocked",
        "riotbox-test",
        "2026-06-03T15:11:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    receipt
        .unsupported_scopes
        .push(UnsupportedExportScope::DawExport);
    session.export_receipts.push(receipt);
    save_session_json(&session_path, &session).expect("save session");
    let report_launch = daw_report_launch(session_path);

    let summary = daw_export_readiness_report_summary(&report_launch).expect("report summary");

    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["ready_for_next_gate"], false);
    assert_eq!(
        summary["readiness_blockers"],
        json!([
            "unsupported_command_boundary",
            "arrangement_placement_blocked",
            "daw_tempo_map_blocked"
        ])
    );
    assert_eq!(
        summary["arrangement_placement_readiness"]["blockers"],
        json!(["unsupported_daw_export_flag_present", "missing_placement_refs"])
    );
    assert_eq!(
        summary["daw_tempo_map_readiness"]["blockers"],
        json!(["unsupported_daw_export_flag_present", "missing_tempo_map_ref"])
    );
    assert_eq!(
        summary["artifact_preflight"]["blockers"],
        json!(["arrangement_placement_blocked"])
    );
}

#[test]
fn daw_export_report_lists_missing_files_after_contracts_are_ready() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let mut session = SessionFile::new(
        "daw-report-missing-file",
        "riotbox-test",
        "2026-06-03T15:12:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    session.export_receipts.push(receipt);
    save_session_json(&session_path, &session).expect("save session");
    let report_launch = daw_report_launch(session_path);

    let summary = daw_export_readiness_report_summary(&report_launch).expect("report summary");

    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["ready_for_next_gate"], false);
    assert_eq!(summary["readiness_blockers"], json!(["missing_local_files"]));
    assert_eq!(
        summary["artifact_preflight"]["blockers"],
        json!(["missing_local_files"])
    );
    let missing = summary["artifact_preflight"]["missing_local_files"]
        .as_array()
        .expect("missing file array");
    assert_eq!(missing.len(), 1);
    assert!(
        missing[0]
            .as_str()
            .expect("missing file path")
            .ends_with("exports/arrangement_manifest.json")
    );
}

#[test]
fn daw_export_report_ready_for_writer_remains_read_only_and_not_musician_runnable() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let manifest_path = temp.path().join("exports/arrangement_manifest.json");
    let proof_path = temp.path().join("exports/proof.json");
    fs::create_dir_all(manifest_path.parent().expect("manifest parent")).expect("create exports");
    fs::write(&manifest_path, "{}").expect("write manifest");
    fs::write(&proof_path, "{}").expect("write proof");
    let manifest_modified = fs::metadata(&manifest_path)
        .expect("manifest metadata")
        .modified()
        .expect("manifest modified timestamp");
    let mut session = SessionFile::new(
        "daw-report-ready-for-writer",
        "riotbox-test",
        "2026-06-03T15:13:00Z",
    );
    let mut receipt = daw_receipt("exports/arrangement_manifest.json", "exports/proof.json");
    attach_ready_daw_refs(&mut receipt);
    session.export_receipts.push(receipt);
    save_session_json(&session_path, &session).expect("save session");
    let report_launch = daw_report_launch(session_path);

    let summary = daw_export_readiness_report_summary(&report_launch).expect("report summary");

    assert_eq!(summary["status"], "ready_for_writer");
    assert_eq!(summary["ready_for_next_gate"], true);
    assert_eq!(summary["writes_files"], false);
    assert_eq!(summary["developer_proof_status"], "ready_for_writer");
    assert_eq!(
        summary["musician_export_readiness"],
        "not_final_daw_export_workflow"
    );
    assert_eq!(
        summary["release_blockers"],
        json!(["developer_proof_only", "daw_writer_missing"])
    );
    assert_eq!(summary["readiness_blockers"], json!([]));
    assert_eq!(summary["arrangement_placement_readiness"]["status"], "ready");
    assert_eq!(summary["daw_tempo_map_readiness"]["status"], "ready");
    assert_eq!(summary["artifact_preflight"]["status"], "ready");
    assert_eq!(
        fs::metadata(&manifest_path)
            .expect("manifest metadata after report")
            .modified()
            .expect("manifest modified timestamp after report"),
        manifest_modified
    );
}

fn daw_report_launch(session_path: PathBuf) -> AppLaunch {
    AppLaunch {
        mode: LaunchMode::DawExportReadinessReport { session_path },
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
