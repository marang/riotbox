#[test]
fn w30_hook_dawproject_cli_parses_and_blocks_without_v4_receipt() {
    let launch = parse_args([
        "--w30-hook-dawproject-execute".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-destination".into(),
        "exports/riotbox-hook.dawproject".into(),
        "--observer".into(),
        "observer.ndjson".into(),
    ])
    .expect("parse W-30 DAWproject mode");
    assert_eq!(launch.observer_path, Some(PathBuf::from("observer.ndjson")));
    assert!(matches!(
        launch.mode,
        LaunchMode::W30HookDawprojectExecute {
            session_path,
            destination_path,
        } if session_path.as_os_str() == "session.json"
            && destination_path.as_os_str() == "exports/riotbox-hook.dawproject"
    ));

    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("blocked.dawproject");
    save_session_json(
        &session_path,
        &SessionFile::new("blocked", "riotbox-test", "2026-08-28T00:00:00Z"),
    )
    .expect("save blocked Session");
    let blocked = AppLaunch {
        mode: LaunchMode::W30HookDawprojectExecute {
            session_path,
            destination_path: destination_path.clone(),
        },
        observer_path: None,
    };
    let (summary, _) =
        w30_hook_dawproject_execute_summary(&blocked).expect("blocked DAWproject summary");
    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["ready"], false);
    assert_eq!(summary["receipt"], serde_json::Value::Null);
    assert!(!destination_path.exists());
}

#[test]
fn w30_hook_dawproject_cli_commits_session_and_observer_without_audio_rerender() {
    let temp = tempfile::tempdir().expect("tempdir");
    let package_root = temp.path().join("hook-package");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("riotbox-hook.dawproject");
    let observer_path = temp.path().join("observer.ndjson");
    let mut state = crate::jam_app::tests::w30_hook_export_state();
    state
        .commit_stem_package_export_w30_hook_loop(&package_root, 1_300)
        .expect("commit V4 hook package");
    save_session_json(&session_path, &state.session).expect("save V4 Session");
    let launch = AppLaunch {
        mode: LaunchMode::W30HookDawprojectExecute {
            session_path: session_path.clone(),
            destination_path: destination_path.clone(),
        },
        observer_path: Some(observer_path.clone()),
    };
    let mut output = Vec::new();

    write_w30_hook_dawproject_execute_output(&launch, &["riotbox-app".into()], &mut output)
        .expect("execute DAWproject CLI");

    let summary: serde_json::Value = serde_json::from_slice(&output).expect("summary JSON");
    assert_eq!(summary["status"], "ready");
    assert_eq!(summary["boundary"], "w30_hook_dawproject_v1");
    assert_eq!(summary["receipt"]["artifact_count"], 4);
    assert_eq!(summary["action"]["status"], "Committed");
    assert_eq!(summary["commit_records"].as_array().map(Vec::len), Some(1));
    assert!(destination_path.is_file());
    let saved = riotbox_core::persistence::load_session_json(&session_path)
        .expect("load committed Session");
    assert_eq!(saved.export_receipts.len(), 2);
    let observer = fs::read_to_string(observer_path).expect("observer event");
    let event: serde_json::Value = serde_json::from_str(observer.trim()).expect("observer JSON");
    assert_eq!(event["event"], "w30_hook_dawproject_execute");
    assert_eq!(event["summary"]["status"], "ready");
    assert_eq!(
        event["snapshot"]["export"]["lifecycle"]
            .as_array()
            .and_then(|lifecycle| lifecycle.last())
            .map(|entry| &entry["stage"]),
        Some(&serde_json::Value::String("completed".into()))
    );
    assert_eq!(
        event["snapshot"]["export"]["lifecycle"]
            .as_array()
            .and_then(|lifecycle| lifecycle.last())
            .map(|entry| &entry["command"]),
        Some(&serde_json::Value::String("export.daw_session".into()))
    );
    let completed_receipt = event["snapshot"]["export"]["lifecycle"]
        .as_array()
        .and_then(|lifecycle| lifecycle.last())
        .map(|entry| &entry["receipt"])
        .expect("completed DAWproject receipt snapshot");
    assert_eq!(
        completed_receipt["proof_path"],
        destination_path.to_string_lossy().as_ref()
    );
    assert_eq!(
        completed_receipt["arrangement_placement_readiness"]["ready"],
        true
    );
    assert_eq!(completed_receipt["daw_tempo_map_readiness"]["ready"], true);
}
