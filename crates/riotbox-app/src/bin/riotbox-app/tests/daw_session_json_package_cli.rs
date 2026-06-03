#[test]
fn parse_args_builds_daw_session_json_package_execute_mode() {
    let launch = parse_args([
        "--daw-session-json-package-execute".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-destination".into(),
        "exports/daw-package".into(),
    ])
    .expect("parse DAW session JSON package execute mode");

    assert_eq!(launch.observer_path, None);
    match launch.mode {
        LaunchMode::DawSessionJsonPackageExecute {
            session_path,
            destination_path,
        } => {
            assert_eq!(session_path, PathBuf::from("session.json"));
            assert_eq!(destination_path, PathBuf::from("exports/daw-package"));
        }
        LaunchMode::Load { .. }
        | LaunchMode::Ingest { .. }
        | LaunchMode::StemPackageLocalCiDryRun { .. }
        | LaunchMode::StemPackageLocalCiExecute { .. }
        | LaunchMode::StemPackageLocalCiReport { .. }
        | LaunchMode::DawExportReadinessReport { .. }
        | LaunchMode::DawSessionWriterPlan { .. } => {
            panic!("expected DAW session JSON package execute mode")
        }
    }
}

#[test]
fn parse_args_rejects_daw_session_json_package_execute_without_required_inputs_or_with_observer() {
    let missing_session = parse_args([
        "--daw-session-json-package-execute".into(),
        "--daw-session-destination".into(),
        "exports/daw-package".into(),
    ])
    .expect_err("session is required");
    assert!(missing_session.contains("--session"));

    let missing_destination = parse_args([
        "--daw-session-json-package-execute".into(),
        "--session".into(),
        "session.json".into(),
    ])
    .expect_err("destination is required");
    assert!(missing_destination.contains("--daw-session-destination"));

    let observer_arg = parse_args([
        "--daw-session-json-package-execute".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-destination".into(),
        "exports/daw-package".into(),
        "--observer".into(),
        "observer.ndjson".into(),
    ])
    .expect_err("execute should not write observer files");
    assert!(observer_arg.contains("reads only an explicit session and destination"));

    let plan_mix = parse_args([
        "--daw-session-json-package-execute".into(),
        "--daw-session-writer-plan".into(),
        "--session".into(),
        "session.json".into(),
        "--daw-session-destination".into(),
        "exports/daw-package".into(),
    ])
    .expect_err("execute should not mix with writer plan");
    assert!(plan_mix.contains("cannot be combined"));
}
