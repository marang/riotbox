//! Explicit input opt-in must fail before Session loading or device startup.
use std::process::{Command, Stdio};

#[test]
fn redirected_terminal_input_refuses_before_missing_session_or_audio_start() {
    let owner = tempfile::tempdir().unwrap();
    let session = owner.path().join("not-created-session.json");
    let destination = owner.path().join("not-created-take.wav");
    let output = Command::new(env!("CARGO_BIN_EXE_riotbox-app"))
        .arg("--live-master-recording-execute")
        .arg("--session")
        .arg(&session)
        .arg("--live-recording-destination")
        .arg(&destination)
        .arg("--live-recording-terminal-input")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let error = String::from_utf8(output.stderr).unwrap();
    let expected_refusal = if cfg!(target_os = "linux") {
        "requires stdin to be a TTY"
    } else {
        "V2 is supported on Linux only"
    };
    assert!(error.contains(expected_refusal), "{error}");
    assert!(output.stdout.is_empty());
    assert!(!session.exists());
    assert!(!destination.exists());
    assert!(!destination.with_extension("wav.riotbox.json").exists());
}
