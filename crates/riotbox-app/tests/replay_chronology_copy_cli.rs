//! Actual headless entrypoint proof using only freshly generated metadata.
use std::{fs, process::Command};

use riotbox_core::{
    persistence::{load_session_json, save_session_json},
    session::{ReplayPolicy, SessionFile, SourceRef},
    source_graph::DecodeProfile,
};

#[test]
fn actual_cli_publishes_explicit_copy_before_any_state_or_reference_hydration() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("original.json");
    let destination = dir.path().join("upgraded.json");
    let mut session = SessionFile::new("generated-binary-copy", "test", "2026-10-16");
    session.action_log.replay_policy = ReplayPolicy::DeterministicPreferred;
    session.source_refs.push(SourceRef {
        source_id: "unavailable-generated-source".into(),
        path_hint: "never-created/source.wav".into(),
        content_hash: "generated-metadata-only".into(),
        duration_seconds: 1.0,
        decode_profile: DecodeProfile::Native,
    });
    save_session_json(&input, &session).unwrap();
    let original = fs::read(&input).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_riotbox-app"))
        .current_dir(dir.path())
        .args([
            "--replay-chronology-upgrade-copy",
            "--session",
            "original.json",
            "--replay-chronology-destination",
            "upgraded.json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let summary: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(summary["disposition"], "Upgraded");
    assert_eq!(summary["input_modified"], false);
    assert_eq!(summary["references_verified"], false);
    assert_eq!(fs::read(&input).unwrap(), original);
    let loaded = load_session_json(&destination).unwrap();
    assert_eq!(
        loaded.action_log.replay_policy,
        ReplayPolicy::DeterministicCommittedOrderV2
    );
    assert_eq!(loaded.source_refs, session.source_refs);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);

    // The real binary must refuse an occupied target without changing either.
    let copied = fs::read(&destination).unwrap();
    let second = Command::new(env!("CARGO_BIN_EXE_riotbox-app"))
        .current_dir(dir.path())
        .args([
            "--replay-chronology-upgrade-copy",
            "--session",
            "original.json",
            "--replay-chronology-destination",
            "upgraded.json",
        ])
        .output()
        .unwrap();
    assert!(!second.status.success());
    assert!(second.stdout.is_empty());
    assert_eq!(fs::read(&input).unwrap(), original);
    assert_eq!(fs::read(&destination).unwrap(), copied);
}
