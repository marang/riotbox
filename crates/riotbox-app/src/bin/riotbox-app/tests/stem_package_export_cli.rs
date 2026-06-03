#[test]
fn parse_args_builds_stem_package_local_ci_dry_run_mode() {
    let launch = parse_args([
        "--stem-package-local-ci-dry-run".into(),
        "--stem-package-destination".into(),
        "artifacts/audio_qa/local/stem-proof".into(),
        "--stem-role".into(),
        "stem_drums".into(),
        "--stem-role".into(),
        "bass".into(),
    ])
    .expect("parse stem package dry-run mode");

    assert_eq!(launch.observer_path, None);
    match launch.mode {
        LaunchMode::StemPackageLocalCiDryRun {
            destination_path,
            claimed_stem_roles,
        } => {
            assert_eq!(
                destination_path,
                PathBuf::from("artifacts/audio_qa/local/stem-proof")
            );
            assert_eq!(
                claimed_stem_roles,
                vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass]
            );
        }
        LaunchMode::Load { .. }
        | LaunchMode::Ingest { .. }
        | LaunchMode::StemPackageLocalCiExecute { .. } => {
            panic!("expected stem package dry-run mode")
        }
    }
}

#[test]
fn parse_args_builds_stem_package_local_ci_execute_mode() {
    let launch = parse_args([
        "--stem-package-local-ci-execute".into(),
        "--session".into(),
        "session.json".into(),
        "--graph".into(),
        "graph.json".into(),
        "--stem-package-destination".into(),
        "artifacts/audio_qa/local/stem-proof".into(),
        "--stem-roles".into(),
        "stem_drums,bass".into(),
        "--observer".into(),
        "observer.ndjson".into(),
    ])
    .expect("parse stem package execute mode");

    assert_eq!(launch.observer_path, Some(PathBuf::from("observer.ndjson")));
    match launch.mode {
        LaunchMode::StemPackageLocalCiExecute {
            session_path,
            source_graph_path,
            destination_path,
            claimed_stem_roles,
        } => {
            assert_eq!(session_path, PathBuf::from("session.json"));
            assert_eq!(source_graph_path, Some(PathBuf::from("graph.json")));
            assert_eq!(
                destination_path,
                PathBuf::from("artifacts/audio_qa/local/stem-proof")
            );
            assert_eq!(
                claimed_stem_roles,
                vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass]
            );
        }
        LaunchMode::Load { .. }
        | LaunchMode::Ingest { .. }
        | LaunchMode::StemPackageLocalCiDryRun { .. } => {
            panic!("expected stem package execute mode")
        }
    }
}

#[test]
fn parse_args_rejects_stem_package_execute_without_session_destination_or_roles() {
    let missing_session = parse_args([
        "--stem-package-local-ci-execute".into(),
        "--stem-package-destination".into(),
        "exports/stem-proof".into(),
        "--stem-role".into(),
        "stem_drums".into(),
    ])
    .expect_err("session is required");
    assert!(missing_session.contains("--session"));

    let missing_destination = parse_args([
        "--stem-package-local-ci-execute".into(),
        "--session".into(),
        "session.json".into(),
        "--stem-role".into(),
        "stem_drums".into(),
    ])
    .expect_err("destination is required");
    assert!(missing_destination.contains("--stem-package-destination"));

    let missing_roles = parse_args([
        "--stem-package-local-ci-execute".into(),
        "--session".into(),
        "session.json".into(),
        "--stem-package-destination".into(),
        "exports/stem-proof".into(),
    ])
    .expect_err("roles are required");
    assert!(missing_roles.contains("requires at least one --stem-role"));
}

#[test]
fn parse_args_rejects_stem_package_execute_mixed_with_ingest_or_dry_run() {
    let ingest_error = parse_args([
        "--stem-package-local-ci-execute".into(),
        "--session".into(),
        "session.json".into(),
        "--stem-package-destination".into(),
        "exports/stem-proof".into(),
        "--stem-role".into(),
        "stem_drums".into(),
        "--source".into(),
        "source.wav".into(),
    ])
    .expect_err("execute should not mix with ingest launch");
    assert!(ingest_error.contains("cannot be combined"));

    let mode_error = parse_args([
        "--stem-package-local-ci-execute".into(),
        "--stem-package-local-ci-dry-run".into(),
        "--session".into(),
        "session.json".into(),
        "--stem-package-destination".into(),
        "exports/stem-proof".into(),
        "--stem-role".into(),
        "stem_drums".into(),
    ])
    .expect_err("execute and dry-run should not mix");
    assert!(mode_error.contains("cannot be combined"));
}

#[test]
fn parse_args_rejects_stem_package_dry_run_without_explicit_destination_or_roles() {
    let missing_destination = parse_args([
        "--stem-package-local-ci-dry-run".into(),
        "--stem-role".into(),
        "stem_drums".into(),
    ])
    .expect_err("destination is required");
    assert!(missing_destination.contains("--stem-package-destination"));

    let missing_roles = parse_args([
        "--stem-package-local-ci-dry-run".into(),
        "--stem-package-destination".into(),
        "exports/stem-proof".into(),
    ])
    .expect_err("roles are required");
    assert!(missing_roles.contains("requires at least one --stem-role"));
}

#[test]
fn parse_args_rejects_stem_package_dry_run_mixed_with_session_launch() {
    let error = parse_args([
        "--stem-package-local-ci-dry-run".into(),
        "--stem-package-destination".into(),
        "exports/stem-proof".into(),
        "--stem-role".into(),
        "stem_drums".into(),
        "--session".into(),
        "session.json".into(),
    ])
    .expect_err("dry-run should not mix with session launch");

    assert!(error.contains("cannot be combined"));
}

#[test]
fn parse_args_rejects_stem_package_dry_run_mixed_with_default_seed_flag() {
    let error = parse_args([
        "--stem-package-local-ci-dry-run".into(),
        "--stem-package-destination".into(),
        "exports/stem-proof".into(),
        "--stem-role".into(),
        "stem_drums".into(),
        "--seed".into(),
        "19".into(),
    ])
    .expect_err("dry-run should not mix with explicit seed launch flag");

    assert!(error.contains("cannot be combined"));
}

#[test]
fn stem_package_local_ci_execute_writes_files_receipt_session_and_observer_event() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("stem-proof");
    let observer_path = temp.path().join("observer.ndjson");
    save_session_json(
        &session_path,
        &SessionFile::new("execute-session", "riotbox-test", "2026-06-03T11:40:00Z"),
    )
    .expect("save session");
    let launch = AppLaunch {
        mode: LaunchMode::StemPackageLocalCiExecute {
            session_path: session_path.clone(),
            source_graph_path: None,
            destination_path: destination_path.clone(),
            claimed_stem_roles: vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        },
        observer_path: Some(observer_path.clone()),
    };

    let (summary, shell) =
        stem_package_local_ci_execute_summary(&launch).expect("execute summary");
    let mut observer = UserSessionObserver::open(&observer_path).expect("open observer");
    observer
        .record(json!({
            "event": "stem_package_local_ci_execute",
            "summary": summary.clone(),
            "snapshot": observer_snapshot(&shell),
        }))
        .expect("write observer event");

    assert_eq!(summary["mode"], "stem_package_local_ci_execute");
    assert_eq!(summary["status"], "ready");
    assert_eq!(summary["ready"], true);
    assert_eq!(summary["writes_files"], true);
    assert_eq!(summary["claimed_stem_roles"], json!(["stem_drums", "stem_bass"]));
    assert_eq!(summary["unsupported_claimed_roles"], json!([]));
    assert_eq!(summary["readiness_blockers"], json!([]));
    assert_eq!(summary["receipt"]["artifact_count"], 4);
    assert!(destination_path.join("stem_package/stems/stem_drums.wav").is_file());
    assert!(destination_path.join("stem_package/stems/stem_bass.wav").is_file());
    assert!(destination_path.join("stem_package/stem_package_manifest.json").is_file());
    assert!(destination_path.join("stem_package/stem_package_proof.json").is_file());

    let persisted =
        riotbox_core::persistence::load_session_json(&session_path).expect("reload session");
    assert_eq!(persisted.export_receipts.len(), 1);
    assert_eq!(persisted.action_log.actions.len(), 1);
    assert_eq!(persisted.action_log.commit_records.len(), 1);
    assert_eq!(
        persisted.export_receipts[0].created_by_action,
        persisted.action_log.actions[0].id
    );
    let observer_body = fs::read_to_string(&observer_path).expect("read observer");
    assert!(observer_body.contains("stem_package_local_ci_execute"));
    assert!(observer_body.contains("\"status\":\"ready\""));
}

#[test]
fn stem_package_local_ci_execute_runner_writes_observer_event() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("stem-proof");
    let observer_path = temp.path().join("observer.ndjson");
    save_session_json(
        &session_path,
        &SessionFile::new("execute-session", "riotbox-test", "2026-06-03T11:44:00Z"),
    )
    .expect("save session");
    let launch = AppLaunch {
        mode: LaunchMode::StemPackageLocalCiExecute {
            session_path: session_path.clone(),
            source_graph_path: None,
            destination_path: destination_path.clone(),
            claimed_stem_roles: vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        },
        observer_path: Some(observer_path.clone()),
    };

    let mut output = Vec::new();
    write_stem_package_local_ci_execute_output(
        &launch,
        &[
            "riotbox-app".into(),
            "--stem-package-local-ci-execute".into(),
            "--session".into(),
            session_path.to_string_lossy().into_owned(),
        ],
        &mut output,
    )
    .expect("execute runner");

    let stdout_json: serde_json::Value =
        serde_json::from_slice(&output).expect("parse stdout json");
    assert_eq!(stdout_json["status"], "ready");
    let observer_body = fs::read_to_string(&observer_path).expect("read observer");
    assert!(observer_body.contains("\"event\":\"stem_package_local_ci_execute\""));
    assert!(observer_body.contains("\"capture_context\":\"non_interactive_cli\""));
    assert!(observer_body.contains("\"status\":\"ready\""));
    let persisted =
        riotbox_core::persistence::load_session_json(&session_path).expect("reload session");
    assert_eq!(persisted.export_receipts.len(), 1);
}

#[test]
fn stem_package_local_ci_execute_reports_unsupported_roles_without_ready_receipt() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("stem-proof");
    save_session_json(
        &session_path,
        &SessionFile::new("execute-session", "riotbox-test", "2026-06-03T11:41:00Z"),
    )
    .expect("save session");
    let launch = AppLaunch {
        mode: LaunchMode::StemPackageLocalCiExecute {
            session_path: session_path.clone(),
            source_graph_path: None,
            destination_path: destination_path.clone(),
            claimed_stem_roles: vec![ExportArtifactRole::StemMusic],
        },
        observer_path: None,
    };

    let (summary, _shell) =
        stem_package_local_ci_execute_summary(&launch).expect("blocked summary");

    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["ready"], false);
    assert_eq!(summary["writes_files"], false);
    assert_eq!(summary["unsupported_claimed_roles"], json!(["stem_music"]));
    assert!(
        summary["readiness_blockers"]
            .as_array()
            .expect("readiness blocker array")[0]
            .as_str()
            .expect("blocker string")
            .contains("unsupported local CI stem package role")
    );
    assert!(!destination_path.join("stem_package").exists());
    let persisted =
        riotbox_core::persistence::load_session_json(&session_path).expect("reload session");
    assert!(persisted.export_receipts.is_empty());
    assert!(persisted.action_log.actions.is_empty());
}

#[test]
fn stem_package_local_ci_execute_does_not_regenerate_over_existing_package() {
    let temp = tempfile::tempdir().expect("tempdir");
    let first_session_path = temp.path().join("first-session.json");
    let second_session_path = temp.path().join("second-session.json");
    let destination_path = temp.path().join("stem-proof");
    save_session_json(
        &first_session_path,
        &SessionFile::new("first-session", "riotbox-test", "2026-06-03T11:42:00Z"),
    )
    .expect("save first session");
    save_session_json(
        &second_session_path,
        &SessionFile::new("second-session", "riotbox-test", "2026-06-03T11:43:00Z"),
    )
    .expect("save second session");
    let first_launch = AppLaunch {
        mode: LaunchMode::StemPackageLocalCiExecute {
            session_path: first_session_path,
            source_graph_path: None,
            destination_path: destination_path.clone(),
            claimed_stem_roles: vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        },
        observer_path: None,
    };
    stem_package_local_ci_execute_summary(&first_launch).expect("first execute succeeds");
    let second_launch = AppLaunch {
        mode: LaunchMode::StemPackageLocalCiExecute {
            session_path: second_session_path.clone(),
            source_graph_path: None,
            destination_path: destination_path.clone(),
            claimed_stem_roles: vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        },
        observer_path: None,
    };

    let (summary, _shell) =
        stem_package_local_ci_execute_summary(&second_launch).expect("blocked summary");

    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["ready"], false);
    assert!(
        summary["readiness_blockers"]
            .as_array()
            .expect("readiness blocker array")[0]
            .as_str()
            .expect("blocker string")
            .contains("already exists")
    );
    let second_session =
        riotbox_core::persistence::load_session_json(&second_session_path).expect("reload session");
    assert!(second_session.export_receipts.is_empty());
}

#[test]
fn stem_package_local_ci_dry_run_reports_ready_plan_without_writing_files() {
    let temp = tempfile::tempdir().expect("tempdir");
    let destination_path = temp.path().join("stem-proof");
    let launch = AppLaunch {
        mode: LaunchMode::StemPackageLocalCiDryRun {
            destination_path: destination_path.clone(),
            claimed_stem_roles: vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        },
        observer_path: None,
    };

    let summary = stem_package_local_ci_dry_run_summary(&launch).expect("dry-run summary");

    assert_eq!(summary["mode"], "stem_package_local_ci_dry_run");
    assert_eq!(summary["status"], "ready");
    assert_eq!(summary["ready"], true);
    assert_eq!(summary["writes_files"], false);
    assert_eq!(summary["boundary"], "stem_package.local_ci_package_v1");
    assert_eq!(summary["claimed_stem_roles"], json!(["stem_drums", "stem_bass"]));
    assert_eq!(summary["supported_roles"], json!(["stem_drums", "stem_bass"]));
    assert_eq!(summary["unsupported_claimed_roles"], json!([]));
    assert_eq!(summary["readiness_blockers"], json!([]));
    assert_eq!(
        summary["planned_artifacts"]
            .as_array()
            .expect("planned artifact array")
            .len(),
        4
    );
    assert!(
        summary["planned_artifacts"]
            .as_array()
            .expect("planned artifact array")
            .iter()
            .any(|artifact| artifact["role"] == "stem_drums" && artifact["media_type"] == "audio_wav")
    );
    assert!(!destination_path.exists());
}

#[test]
fn stem_package_local_ci_dry_run_reports_unsupported_roles_without_writing_files() {
    let temp = tempfile::tempdir().expect("tempdir");
    let destination_path = temp.path().join("stem-proof");
    let launch = AppLaunch {
        mode: LaunchMode::StemPackageLocalCiDryRun {
            destination_path: destination_path.clone(),
            claimed_stem_roles: vec![ExportArtifactRole::StemMusic],
        },
        observer_path: None,
    };

    let summary = stem_package_local_ci_dry_run_summary(&launch).expect("dry-run summary");

    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["ready"], false);
    assert_eq!(summary["writes_files"], false);
    assert_eq!(summary["unsupported_claimed_roles"], json!(["stem_music"]));
    assert!(
        summary["readiness_blockers"]
            .as_array()
            .expect("readiness blocker array")[0]
            .as_str()
            .expect("blocker string")
            .contains("UnsupportedStemRole")
    );
    assert_eq!(summary["planned_artifacts"], json!([]));
    assert!(!destination_path.exists());
}
