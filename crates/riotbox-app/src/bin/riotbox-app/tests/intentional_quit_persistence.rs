use std::{fs, path::Path};

use riotbox_core::{
    persistence::{load_session_json, save_session_json},
    queue::ActionQueue,
    session::SessionFile,
    style::{PerformancePresetId, StyleProfileId},
};

use crate::{
    jam_app::{JamAppState, JamFileSet},
    ui::{JamShellState, ShellLaunchMode},
};

use super::{UserSessionObserver, persist_and_record_quit};

#[test]
fn intentional_quit_persists_live_session_before_recording_exit() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let observer_path = temp.path().join("observer/events.ndjson");
    let session = SessionFile::new("quit-save", "0.1.0", "2026-07-17T00:00:00Z");
    save_session_json(&session_path, &session).expect("save initial session");

    let state =
        JamAppState::from_json_files(&session_path, Option::<&Path>::None).expect("load session");
    let mut shell = JamShellState::new(state, ShellLaunchMode::Load);
    shell.app.session.runtime_state.style.active_profile = Some(StyleProfileId::FeralRebuild);
    shell.app.session.runtime_state.style.active_preset =
        Some(PerformancePresetId::FeralBreakAlphaV2);
    let mut observer = UserSessionObserver::open(&observer_path).expect("open observer");

    persist_and_record_quit(&shell, Some(&mut observer), 123, "q")
        .expect("persist and record intentional quit");
    drop(observer);

    let restored = load_session_json(&session_path).expect("reload persisted session");
    assert_eq!(
        restored.runtime_state.style.active_profile,
        Some(StyleProfileId::FeralRebuild)
    );
    assert_eq!(
        restored.runtime_state.style.active_preset,
        Some(PerformancePresetId::FeralBreakAlphaV2)
    );

    let content = fs::read_to_string(observer_path).expect("read observer");
    assert!(content.contains("\"event\":\"key_outcome\""));
    assert!(content.contains("\"key\":\"q\""));
    assert!(content.contains("\"outcome\":\"quit\""));
}

#[test]
fn failed_intentional_quit_save_does_not_record_a_clean_exit() {
    let temp = tempfile::tempdir().expect("tempdir");
    let invalid_session_path = temp.path().join("session-target");
    fs::create_dir(&invalid_session_path).expect("create invalid session target directory");
    let observer_path = temp.path().join("observer/events.ndjson");
    let mut state = JamAppState::from_parts(
        SessionFile::new("quit-save-failure", "0.1.0", "2026-07-18T00:00:00Z"),
        None,
        ActionQueue::new(),
    );
    state.files = Some(JamFileSet {
        session_path: invalid_session_path,
        source_graph_path: None,
    });
    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    let mut observer = UserSessionObserver::open(&observer_path).expect("open observer");

    let result = persist_and_record_quit(&shell, Some(&mut observer), 123, "q");
    assert!(result.is_err());
    drop(observer);

    let content = fs::read_to_string(observer_path).expect("read observer");
    assert!(!content.contains("\"outcome\":\"quit\""));
}
