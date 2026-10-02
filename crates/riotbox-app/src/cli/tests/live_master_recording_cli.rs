use crate::cli::args::parse_args;
use crate::cli::live_master_recording::apply_live_master_observer_status;
use crate::cli::live_master_recording::validate_live_master_observer_path;
use crate::cli::model::LaunchMode;
use crate::cli::observer::UserSessionObserver;
use crate::cli::observer::launch_summary;
use riotbox_core::action::LiveRecordingDuration;
use serde_json::Value;
use serde_json::json;
use std::fs;
use std::io;
use std::path::PathBuf;

#[test]
fn live_master_recording_cli_parses_one_explicit_real_audio_path() {
    let launch = parse_args([
        "--live-master-recording-execute".into(),
        "--session".into(),
        "session.json".into(),
        "--graph".into(),
        "graph.json".into(),
        "--live-recording-destination".into(),
        "exports/live-master.wav".into(),
        "--observer".into(),
        "artifacts/live-master.ndjson".into(),
    ])
    .expect("parse live master recording execute");

    assert_eq!(
        launch.observer_path,
        Some(PathBuf::from("artifacts/live-master.ndjson"))
    );
    let summary = launch_summary(&launch);
    assert_eq!(summary["duration_bars"], 2);
    assert_eq!(summary["duration_beats"], 8);
    assert_eq!(summary["boundary"], "runtime_master_bar_window_v4");
    match launch.mode {
        LaunchMode::LiveMasterRecordingExecute {
            session_path,
            source_graph_path,
            destination_path,
            duration,
        } => {
            assert_eq!(session_path, PathBuf::from("session.json"));
            assert_eq!(source_graph_path, Some(PathBuf::from("graph.json")));
            assert_eq!(destination_path, PathBuf::from("exports/live-master.wav"));
            assert_eq!(duration, LiveRecordingDuration::TwoBars);
        }
        other => panic!("expected live master recording execute, got {other:?}"),
    }
}

#[test]
fn live_master_recording_cli_accepts_only_typed_bounded_windows_and_reports_them() {
    for (bars, duration, boundary) in [
        (
            "2",
            LiveRecordingDuration::TwoBars,
            "runtime_master_bar_window_v4",
        ),
        (
            "8",
            LiveRecordingDuration::EightBars,
            "runtime_master_bar_window_v3",
        ),
        (
            "16",
            LiveRecordingDuration::SixteenBars,
            "runtime_master_bar_window_v3",
        ),
    ] {
        let mut args = recording_args();
        args.extend(["--live-recording-bars".into(), bars.into()]);
        let launch = parse_args(args).expect("bounded duration");
        assert!(
            matches!(launch.mode, LaunchMode::LiveMasterRecordingExecute {
            duration: actual, ..
        } if actual == duration)
        );
        let summary = launch_summary(&launch);
        assert_eq!(summary["duration_bars"], json!(duration.bars()));
        assert_eq!(summary["duration_beats"], json!(duration.duration_beats()));
        assert_eq!(summary["boundary"], boundary);
    }
}

#[test]
fn live_master_recording_cli_rejects_missing_invalid_and_repeated_windows() {
    let mut missing = recording_args();
    missing.push("--live-recording-bars".into());
    assert!(parse_args(missing).unwrap_err().contains("missing value"));
    for invalid in [
        "",
        "0",
        "1",
        "4",
        "17",
        "-2",
        "2.0",
        "02",
        "all",
        "--observer",
    ] {
        let mut args = recording_args();
        args.extend(["--live-recording-bars".into(), invalid.into()]);
        assert!(
            parse_args(args)
                .unwrap_err()
                .contains("requires 2, 8, or 16")
        );
    }
    for repeated in ["8", "16"] {
        let mut args = recording_args();
        args.extend([
            "--live-recording-bars".into(),
            "8".into(),
            "--live-recording-bars".into(),
            repeated.into(),
        ]);
        assert!(parse_args(args).unwrap_err().contains("at most once"));
    }
}

#[test]
fn live_master_recording_cli_rejects_orphan_windows_before_other_mode_dispatch() {
    for mode in [
        None,
        Some("--live-recording-readiness-report"),
        Some("--daw-export-readiness-report"),
        Some("--stem-package-local-ci-report"),
        Some("--live-master-dawproject-execute"),
    ] {
        let mut args = vec![
            "--session".into(),
            "session.json".into(),
            "--live-recording-bars".into(),
            "8".into(),
        ];
        args.extend(mode.map(str::to_owned));
        assert_eq!(
            parse_args(args).unwrap_err(),
            "--live-recording-bars requires --live-master-recording-execute"
        );
    }
}

#[test]
fn live_master_recording_cli_rejects_conflicting_modes_with_explicit_window() {
    for conflict in [
        "--live-recording-readiness-report",
        "--daw-export-readiness-report",
        "--stem-package-local-ci-report",
        "--live-master-dawproject-execute",
    ] {
        let mut args = recording_args();
        args.extend(["--live-recording-bars".into(), "16".into(), conflict.into()]);
        assert!(parse_args(args).unwrap_err().contains("cannot be combined"));
    }
}

fn recording_args() -> Vec<String> {
    [
        "--live-master-recording-execute",
        "--session",
        "session.json",
        "--live-recording-destination",
        "exports/live-master.wav",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

#[test]
fn live_master_recording_cli_requires_session_and_wav_destination_flag_pair() {
    let missing_destination = parse_args([
        "--live-master-recording-execute".into(),
        "--session".into(),
        "session.json".into(),
    ])
    .expect_err("destination required");
    assert!(missing_destination.contains("--live-recording-destination"));

    let orphan_destination = parse_args([
        "--session".into(),
        "session.json".into(),
        "--live-recording-destination".into(),
        "exports/live-master.wav".into(),
    ])
    .expect_err("execute flag required");
    assert!(orphan_destination.contains("--live-master-recording-execute"));
}

#[test]
fn live_master_observer_rejects_session_graph_destination_and_proof_aliases() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session = temp.path().join("session.json");
    let graph = temp.path().join("graph.json");
    let destination = temp.path().join("live-master.wav");
    let proof = crate::jam_app::live_master_recording_proof_path(&destination).expect("proof path");
    fs::write(&session, b"session").expect("session fixture");
    fs::write(&graph, b"graph").expect("graph fixture");

    for (label, observer) in [
        ("Session", session.as_path()),
        ("Source Graph", graph.as_path()),
        ("recording destination", destination.as_path()),
        ("recording proof", proof.as_path()),
    ] {
        let error =
            validate_live_master_observer_path(observer, &session, Some(&graph), &destination)
                .expect_err("protected artifact alias rejected");
        assert!(error.to_string().contains(label), "{error}");
    }
}

#[test]
fn live_master_observer_is_fresh_and_cannot_hide_a_successful_capture_on_write_failure() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session = temp.path().join("session.json");
    let destination = temp.path().join("live-master.wav");
    let observer_path = temp.path().join("observer.ndjson");
    fs::write(&session, b"session").expect("session fixture");
    validate_live_master_observer_path(&observer_path, &session, None, &destination)
        .expect("fresh observer path");
    let _observer = UserSessionObserver::open_new(&observer_path).expect("create observer");
    let second_open = match UserSessionObserver::open_new(&observer_path) {
        Ok(_) => panic!("observer cannot be truncated or reused"),
        Err(error) => error,
    };
    assert_eq!(second_open.kind(), io::ErrorKind::AlreadyExists);

    let mut summary = json!({"ready": true, "status": "ready"});
    let observer_error = Err(io::Error::new(io::ErrorKind::BrokenPipe, "observer lost"));
    apply_live_master_observer_status(&mut summary, true, &observer_error);

    assert_eq!(summary["ready"], Value::Bool(true));
    assert_eq!(summary["status"], Value::String("ready".into()));
    assert_eq!(
        summary["observer_status"],
        Value::String("write_failed".into())
    );
    assert_eq!(summary["observer_events"], Value::Bool(false));
}

#[test]
fn live_master_observer_preflight_does_not_create_the_recording_destination_parent() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session = temp.path().join("session.json");
    let destination_parent = temp.path().join("missing-export-parent");
    let destination = destination_parent.join("live-master.wav");
    let observer_path = temp.path().join("observer").join("events.ndjson");
    fs::write(&session, b"session").expect("session fixture");

    let error = validate_live_master_observer_path(&observer_path, &session, None, &destination)
        .expect_err("missing destination parent remains invalid");

    assert_eq!(
        error.downcast_ref::<io::Error>().map(io::Error::kind),
        Some(io::ErrorKind::NotFound)
    );
    assert!(!destination_parent.exists());
    assert!(!observer_path.parent().unwrap().exists());

    let shared_missing_parent = temp.path().join("shared-missing-parent");
    let shared_destination = shared_missing_parent.join("live-master.wav");
    let shared_observer = shared_missing_parent.join("events.ndjson");
    validate_live_master_observer_path(&shared_observer, &session, None, &shared_destination)
        .expect_err("observer must not create a shared missing destination parent");
    assert!(!shared_missing_parent.exists());
}
