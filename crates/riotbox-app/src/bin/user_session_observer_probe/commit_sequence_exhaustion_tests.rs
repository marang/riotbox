use std::{fs, io};

use crossterm::event::KeyCode;
use riotbox_app::{jam_app::JamAppError, ui::JamShellState};
use riotbox_core::{
    action::CommitBoundary, ids::SceneId, replay::build_committed_replay_plan,
    session::CommitSequenceExhausted, transport::CommitBoundaryState,
};
use serde_json::json;

use super::{NdjsonWriter, apply_probe_key, commit_boundary, observer_snapshot, probe_shell};

fn shell_with_exhausted_boundary(boundary: CommitBoundaryState) -> JamShellState {
    let mut shell = probe_shell("exhausted-sequence-observer");
    shell
        .app
        .queue_next_capture_length_intent(1)
        .expect("fixture action ID available");
    let committed = shell
        .app
        .commit_ready_actions(boundary, 2)
        .expect("fixture commit sequence available");
    assert_eq!(committed.len(), 1);
    shell.app.session.action_log.commit_records[0].commit_sequence = u32::MAX;
    assert_eq!(
        build_committed_replay_plan(&shell.app.session.action_log)
            .expect("MAX-sequence history remains valid")
            .len(),
        1
    );
    shell
}

#[test]
fn exhausted_toggle_sequence_is_visible_without_success_event_or_product_mutation() {
    let initial = probe_shell("initial-clock");
    let boundary = initial
        .app
        .runtime
        .transport
        .boundary_state(CommitBoundary::Immediate);
    let mut shell = shell_with_exhausted_boundary(boundary.clone());
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("events.ndjson");
    let mut writer = NdjsonWriter::open(&path).expect("open observer");
    let expected = CommitSequenceExhausted {
        boundary,
        requested_count: 1,
        max_existing: u32::MAX,
    };
    let session_before = shell.app.session.clone();
    let runtime_before = shell.app.runtime.clone();
    let queue_before = format!("{:?}", shell.app.queue);
    let mut expected_observer = observer_snapshot(&shell);
    expected_observer["status_message"] = json!(expected.to_string());

    let error = apply_probe_key(&mut shell, &mut writer, 100, KeyCode::Char(' '))
        .expect_err("exhausted toggle must fail visibly");

    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert!(matches!(
        error.get_ref().and_then(|error| error.downcast_ref::<JamAppError>()),
        Some(JamAppError::CommitSequenceExhausted(cause)) if cause == &expected
    ));
    assert_eq!(shell.status_message, expected.to_string());
    assert_eq!(shell.app.session, session_before);
    assert_eq!(shell.app.runtime, runtime_before);
    assert_eq!(format!("{:?}", shell.app.queue), queue_before);
    assert_eq!(observer_snapshot(&shell), expected_observer);
    assert!(fs::read_to_string(path).expect("read observer").is_empty());
}

#[test]
fn exhausted_probe_boundary_preserves_clock_pending_action_and_event_stream() {
    let boundary = CommitBoundaryState {
        kind: CommitBoundary::Bar,
        beat_index: 16,
        bar_index: 5,
        phrase_index: 2,
        scene_id: Some(SceneId::from("scene-1")),
    };
    let mut shell = shell_with_exhausted_boundary(boundary.clone());
    shell
        .app
        .queue_tr909_fill(10)
        .expect("fixture action ID available");
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("events.ndjson");
    let mut writer = NdjsonWriter::open(&path).expect("open observer");
    let expected = CommitSequenceExhausted {
        boundary,
        requested_count: 1,
        max_existing: u32::MAX,
    };
    let session_before = shell.app.session.clone();
    let runtime_before = shell.app.runtime.clone();
    let queue_before = format!("{:?}", shell.app.queue);
    let mut expected_observer = observer_snapshot(&shell);
    expected_observer["status_message"] = json!(expected.to_string());

    let error = commit_boundary(&mut shell, &mut writer, 100, CommitBoundary::Bar, 1, 1)
        .expect_err("exhausted boundary must fail before clock movement");

    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(
        error
            .get_ref()
            .and_then(|error| error.downcast_ref::<CommitSequenceExhausted>()),
        Some(&expected)
    );
    assert_eq!(shell.status_message, expected.to_string());
    assert_eq!(shell.app.session, session_before);
    assert_eq!(shell.app.runtime, runtime_before);
    assert_eq!(format!("{:?}", shell.app.queue), queue_before);
    assert_eq!(observer_snapshot(&shell), expected_observer);
    assert!(fs::read_to_string(path).expect("read observer").is_empty());
}

#[test]
fn queued_monitor_action_remains_pending_when_its_later_commit_is_refused() {
    let initial = probe_shell("initial-clock");
    let boundary = initial
        .app
        .runtime
        .transport
        .boundary_state(CommitBoundary::Immediate);
    let mut shell = shell_with_exhausted_boundary(boundary.clone());
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("events.ndjson");
    let mut writer = NdjsonWriter::open(&path).expect("open observer");
    let expected = CommitSequenceExhausted {
        boundary,
        requested_count: 1,
        max_existing: u32::MAX,
    };
    let session_before = shell.app.session.clone();
    let runtime_before = shell.app.runtime.clone();
    let history_before = shell.app.queue.history().to_vec();
    let next_id = shell
        .app
        .queue
        .clone()
        .allocate_action_id()
        .expect("fixture action ID available");

    let error = apply_probe_key(&mut shell, &mut writer, 100, KeyCode::Char('M'))
        .expect_err("a later refused commit must not announce success");

    assert!(matches!(
        error.get_ref().and_then(|error| error.downcast_ref::<JamAppError>()),
        Some(JamAppError::CommitSequenceExhausted(cause)) if cause == &expected
    ));
    assert_eq!(shell.status_message, expected.to_string());
    assert_eq!(shell.app.session, session_before);
    assert_eq!(shell.app.runtime, runtime_before);
    assert_eq!(shell.app.queue.history(), history_before);
    let pending = shell.app.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, next_id);
    assert_eq!(
        pending[0].command,
        riotbox_core::action::ActionCommand::SourceMonitorSetMode
    );
    assert_eq!(
        pending[0].status,
        riotbox_core::action::ActionStatus::Queued
    );
    assert!(fs::read_to_string(path).expect("read observer").is_empty());
}
