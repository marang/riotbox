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
        LaunchMode::Load { .. } | LaunchMode::Ingest { .. } => {
            panic!("expected stem package dry-run mode")
        }
    }
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
