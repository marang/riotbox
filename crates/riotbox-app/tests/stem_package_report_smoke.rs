use std::{fs, process::Command};

use riotbox_core::{persistence::save_session_json, session::SessionFile};
use serde_json::Value;

#[test]
fn stem_package_local_ci_report_smoke_covers_ready_and_missing_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let destination_path = temp.path().join("stem-proof");
    save_session_json(
        &session_path,
        &SessionFile::new(
            "stem-package-report-smoke",
            "riotbox-test",
            "2026-06-03T12:35:00Z",
        ),
    )
    .expect("save smoke session");

    let execute = run_riotbox_app_json([
        "--stem-package-local-ci-execute",
        "--session",
        session_path.to_str().expect("session path"),
        "--stem-package-destination",
        destination_path.to_str().expect("destination path"),
        "--stem-role",
        "stem_drums",
        "--stem-role",
        "stem_bass",
    ]);
    assert_eq!(execute["status"], "ready");
    assert_eq!(execute["receipt"]["pack_id"], "stem-package-local-ci");
    assert_eq!(execute["receipt"]["export_role"], "package_manifest");
    assert_eq!(
        execute["receipt"]["export_boundary"],
        "stem_package.local_ci_package_v1"
    );

    let ready_report = run_report(&session_path);
    assert_eq!(ready_report["status"], "ready");
    assert_eq!(ready_report["ready"], true);
    assert_eq!(ready_report["writes_files"], false);
    assert_eq!(
        ready_report["developer_proof_status"],
        "local_ci_package_ready"
    );
    assert_eq!(
        ready_report["musician_export_readiness"],
        "not_final_daw_export_workflow"
    );
    assert_eq!(ready_report["receipt"]["pack_id"], "stem-package-local-ci");
    assert_eq!(ready_report["receipt"]["export_role"], "package_manifest");
    assert_eq!(
        ready_report["receipt"]["export_boundary"],
        "stem_package.local_ci_package_v1"
    );
    assert_eq!(
        ready_report["stem_roles"],
        Value::Array(vec!["stem_drums".into(), "stem_bass".into()])
    );
    assert_eq!(ready_report["readiness_blockers"], Value::Array(Vec::new()));
    assert_eq!(
        ready_report["missing_local_files"],
        Value::Array(Vec::new())
    );

    fs::remove_file(destination_path.join("stem_package/stems/stem_bass.wav"))
        .expect("remove bass stem");
    let missing_report = run_report(&session_path);
    assert_eq!(missing_report["status"], "blocked");
    assert_eq!(missing_report["ready"], false);
    assert_eq!(
        missing_report["readiness_blockers"],
        Value::Array(vec!["missing_local_files".into()])
    );
    let missing = missing_report["missing_local_files"]
        .as_array()
        .expect("missing file array");
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0]["role"], "stem_bass");
    assert_eq!(missing[0]["availability_reason"], "missing_file");
}

fn run_report(session_path: &std::path::Path) -> Value {
    run_riotbox_app_json([
        "--stem-package-local-ci-report",
        "--session",
        session_path.to_str().expect("session path"),
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
