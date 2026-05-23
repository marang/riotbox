#[test]
fn capture_length_control_commits_immediate_length_change() {
    let session = SessionFile::new("session-1", "0.1.0", "2026-05-23T00:00:00Z");
    let mut shell = JamShellState::new(
        JamAppState::from_parts(session, None, ActionQueue::new()),
        ShellLaunchMode::Load,
    );

    commit_capture_length_change(&mut shell, 123, false);

    assert_eq!(
        shell.app.session.runtime_state.capture.length_intent,
        CaptureLengthIntent::OneBar
    );
    assert_eq!(shell.status_message, "capture length 1 bar");
    assert!(shell.app.queue.pending_actions().is_empty());
    assert_eq!(shell.app.session.action_log.actions.len(), 1);
    assert_eq!(
        shell.app.session.action_log.actions[0].command,
        ActionCommand::CaptureSetLength
    );
}
