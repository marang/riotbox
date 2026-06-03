#[test]
fn parse_args_builds_stem_package_local_ci_report_mode() {
    let launch = parse_args([
        "--stem-package-local-ci-report".into(),
        "--session".into(),
        "session.json".into(),
    ])
    .expect("parse stem package report mode");

    assert_eq!(launch.observer_path, None);
    match launch.mode {
        LaunchMode::StemPackageLocalCiReport { session_path } => {
            assert_eq!(session_path, PathBuf::from("session.json"));
        }
        LaunchMode::Load { .. }
        | LaunchMode::Ingest { .. }
        | LaunchMode::StemPackageLocalCiDryRun { .. }
        | LaunchMode::StemPackageLocalCiExecute { .. }
        | LaunchMode::DawExportReadinessReport { .. }
        | LaunchMode::DawSessionJsonPackageExecute { .. }
        | LaunchMode::DawSessionJsonPackageEvidenceApply { .. }
        | LaunchMode::DawSessionWriterPlan { .. } => {
            panic!("expected stem package report mode")
        }
    }
}

#[test]
fn parse_args_rejects_stem_package_report_without_session_or_with_write_args() {
    let missing_session =
        parse_args(["--stem-package-local-ci-report".into()]).expect_err("session is required");
    assert!(missing_session.contains("--session"));

    let write_args = parse_args([
        "--stem-package-local-ci-report".into(),
        "--session".into(),
        "session.json".into(),
        "--stem-package-destination".into(),
        "exports/stem-proof".into(),
        "--stem-role".into(),
        "stem_drums".into(),
    ])
    .expect_err("report should reject write-shaped args");
    assert!(write_args.contains("reads only an explicit session"));

    let observer_arg = parse_args([
        "--stem-package-local-ci-report".into(),
        "--session".into(),
        "session.json".into(),
        "--observer".into(),
        "observer.ndjson".into(),
    ])
    .expect_err("report should not write observer files");
    assert!(observer_arg.contains("reads only an explicit session"));
}

#[test]
fn stem_package_local_ci_report_summarizes_ready_package_without_rewriting() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("stem-proof");
    save_session_json(
        &session_path,
        &SessionFile::new("report-session", "riotbox-test", "2026-06-03T12:10:00Z"),
    )
    .expect("save session");
    let execute_launch = AppLaunch {
        mode: LaunchMode::StemPackageLocalCiExecute {
            session_path: session_path.clone(),
            source_graph_path: None,
            destination_path,
            claimed_stem_roles: vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        },
        observer_path: None,
    };
    stem_package_local_ci_execute_summary(&execute_launch).expect("execute package proof");
    let drums_path = temp
        .path()
        .join("stem-proof/stem_package/stems/stem_drums.wav");
    let drums_modified = fs::metadata(&drums_path)
        .expect("drums metadata")
        .modified()
        .expect("drums modified timestamp");
    let report_launch = AppLaunch {
        mode: LaunchMode::StemPackageLocalCiReport {
            session_path: session_path.clone(),
        },
        observer_path: None,
    };

    let summary = stem_package_local_ci_report_summary(&report_launch).expect("report summary");

    assert_eq!(summary["mode"], "stem_package_local_ci_report");
    assert_eq!(summary["status"], "ready");
    assert_eq!(summary["ready"], true);
    assert_eq!(summary["writes_files"], false);
    assert_eq!(summary["developer_proof_status"], "local_ci_package_ready");
    assert_eq!(
        summary["musician_export_readiness"],
        "not_final_daw_export_workflow"
    );
    assert_eq!(summary["stem_roles"], json!(["stem_drums", "stem_bass"]));
    assert_eq!(
        summary["receipt"]["pack_id"],
        riotbox_core::export_readiness::STEM_PACKAGE_LOCAL_CI_PACK_ID
    );
    assert_eq!(summary["receipt"]["export_role"], "package_manifest");
    assert_eq!(
        summary["receipt"]["export_boundary"],
        "stem_package.local_ci_package_v1"
    );
    assert_eq!(summary["readiness_blockers"], json!([]));
    assert_eq!(summary["missing_local_files"], json!([]));
    assert_eq!(summary["qa_gates"].as_array().expect("qa gate array").len(), 5);
    assert_eq!(summary["manifest"]["role"], "export_manifest");
    assert_eq!(summary["proof"]["role"], "product_export_proof");
    assert_eq!(
        fs::metadata(&drums_path)
            .expect("drums metadata after report")
            .modified()
            .expect("drums modified timestamp after report"),
        drums_modified
    );
}

#[test]
fn stem_package_local_ci_report_keeps_product_mix_receipts_separate() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let mut session = SessionFile::new(
        "product-mix-only-session",
        "riotbox-test",
        "2026-06-03T12:09:00Z",
    );
    let contract = riotbox_core::export_readiness::ExportReadinessContract {
        schema: riotbox_core::export_readiness::EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: riotbox_core::export_readiness::ExportReadinessStatus::Reproducible,
        proof_schema: riotbox_core::export_readiness::PRODUCT_EXPORT_PROOF_SCHEMA.into(),
        export_scope: riotbox_core::export_readiness::ExportScope::ProductMix,
        boundary: riotbox_core::export_readiness::ProductExportBoundary::FeralGridGeneratedSupport,
        pack_id: riotbox_core::export_readiness::PRODUCT_EXPORT_PACK_ID.into(),
        export_role: riotbox_core::export_readiness::ProductExportRole::FullGridMix,
        export_artifact: "product-mix.wav".into(),
        source_sha256: "source-sha".into(),
        export_sha256: "export-sha".into(),
        normalized_manifest_sha256: "manifest-sha".into(),
        unsupported_scopes: riotbox_core::export_readiness::default_unsupported_export_scopes(),
    };
    session
        .export_receipts
        .push(riotbox_core::session::ExportReceiptState::from_readiness_contract(
            ActionId(9),
            44_000,
            &contract,
            "product-mix.wav",
            "product-proof.json",
            None,
        ));
    save_session_json(&session_path, &session).expect("save session");
    let report_launch = AppLaunch {
        mode: LaunchMode::StemPackageLocalCiReport { session_path },
        observer_path: None,
    };

    let summary = stem_package_local_ci_report_summary(&report_launch).expect("report summary");

    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["developer_proof_status"], "no_stem_package_receipt");
    assert_eq!(summary["readiness_blockers"], json!(["no_stem_package_receipt"]));
    assert_eq!(summary["product_mix_receipt_count"], 1);
    assert_eq!(summary["receipt"], serde_json::Value::Null);
}

#[test]
fn stem_package_local_ci_report_lists_missing_stem_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("stem-proof");
    save_session_json(
        &session_path,
        &SessionFile::new("report-session", "riotbox-test", "2026-06-03T12:11:00Z"),
    )
    .expect("save session");
    let execute_launch = AppLaunch {
        mode: LaunchMode::StemPackageLocalCiExecute {
            session_path: session_path.clone(),
            source_graph_path: None,
            destination_path,
            claimed_stem_roles: vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        },
        observer_path: None,
    };
    stem_package_local_ci_execute_summary(&execute_launch).expect("execute package proof");
    fs::remove_file(
        temp.path()
            .join("stem-proof/stem_package/stems/stem_bass.wav"),
    )
    .expect("remove bass stem");
    let report_launch = AppLaunch {
        mode: LaunchMode::StemPackageLocalCiReport { session_path },
        observer_path: None,
    };

    let summary = stem_package_local_ci_report_summary(&report_launch).expect("report summary");

    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["ready"], false);
    assert_eq!(summary["readiness_blockers"], json!(["missing_local_files"]));
    let missing = summary["missing_local_files"]
        .as_array()
        .expect("missing files");
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0]["role"], "stem_bass");
    assert_eq!(missing[0]["availability_reason"], "missing_file");
}

#[test]
fn stem_package_local_ci_report_lists_missing_manifest_and_proof_files() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("stem-proof");
    save_session_json(
        &session_path,
        &SessionFile::new("report-session", "riotbox-test", "2026-06-03T12:12:00Z"),
    )
    .expect("save session");
    let execute_launch = AppLaunch {
        mode: LaunchMode::StemPackageLocalCiExecute {
            session_path: session_path.clone(),
            source_graph_path: None,
            destination_path,
            claimed_stem_roles: vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        },
        observer_path: None,
    };
    stem_package_local_ci_execute_summary(&execute_launch).expect("execute package proof");
    fs::remove_file(
        temp.path()
            .join("stem-proof/stem_package/stem_package_manifest.json"),
    )
    .expect("remove manifest");
    fs::remove_file(
        temp.path()
            .join("stem-proof/stem_package/stem_package_proof.json"),
    )
    .expect("remove proof");
    let report_launch = AppLaunch {
        mode: LaunchMode::StemPackageLocalCiReport { session_path },
        observer_path: None,
    };

    let summary = stem_package_local_ci_report_summary(&report_launch).expect("report summary");

    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["ready"], false);
    assert_eq!(summary["readiness_blockers"], json!(["missing_local_files"]));
    let missing_roles = summary["missing_local_files"]
        .as_array()
        .expect("missing files")
        .iter()
        .map(|entry| entry["role"].as_str().expect("role"))
        .collect::<Vec<_>>();
    assert!(missing_roles.contains(&"export_manifest"));
    assert!(missing_roles.contains(&"product_export_proof"));
}

#[test]
fn stem_package_local_ci_report_surfaces_unsupported_scope_flag_blocker() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("stem-proof");
    save_session_json(
        &session_path,
        &SessionFile::new("report-session", "riotbox-test", "2026-06-03T12:13:00Z"),
    )
    .expect("save session");
    let execute_launch = AppLaunch {
        mode: LaunchMode::StemPackageLocalCiExecute {
            session_path: session_path.clone(),
            source_graph_path: None,
            destination_path,
            claimed_stem_roles: vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        },
        observer_path: None,
    };
    stem_package_local_ci_execute_summary(&execute_launch).expect("execute package proof");
    let mut persisted =
        riotbox_core::persistence::load_session_json(&session_path).expect("reload session");
    persisted.export_receipts[0]
        .unsupported_scopes
        .push(riotbox_core::export_readiness::UnsupportedExportScope::StemPackage);
    save_session_json(&session_path, &persisted).expect("save unsupported scope session");
    let report_launch = AppLaunch {
        mode: LaunchMode::StemPackageLocalCiReport { session_path },
        observer_path: None,
    };

    let summary = stem_package_local_ci_report_summary(&report_launch).expect("report summary");

    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["ready"], false);
    assert_eq!(
        summary["readiness_blockers"],
        json!(["unsupported_scope_flag_present"])
    );
    assert_eq!(summary["missing_local_files"], json!([]));
}
