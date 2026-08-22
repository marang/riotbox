use super::*;
use riotbox_core::action::{ActionCommand, ActionStatus, CommitBoundary};

#[test]
fn space_transport_control_commits_play_then_pause_immediately() {
    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            SessionFile::new("session-1", "0.1.0", "2026-08-22T00:00:00Z"),
            None,
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );

    assert_eq!(
        shell.handle_key_code(crossterm::event::KeyCode::Char(' ')),
        ShellKeyOutcome::ToggleTransport
    );
    let play = commit_transport_toggle(&mut shell, 100);

    assert_eq!(play.len(), 1);
    assert_eq!(play[0].boundary.kind, CommitBoundary::Immediate);
    let play_action = shell
        .app
        .queue
        .history_action(play[0].action_id)
        .expect("committed play action");
    assert_eq!(play_action.command, ActionCommand::TransportPlay);
    assert_eq!(play_action.status, ActionStatus::Committed);
    assert!(shell.app.runtime.transport.is_playing);
    assert!(shell.app.session.runtime_state.transport.is_playing);
    assert_eq!(shell.status_message, "transport started");

    assert_eq!(
        shell.handle_key_code(crossterm::event::KeyCode::Char(' ')),
        ShellKeyOutcome::ToggleTransport
    );
    let pause = commit_transport_toggle(&mut shell, 200);

    assert_eq!(pause.len(), 1);
    assert_eq!(pause[0].boundary.kind, CommitBoundary::Immediate);
    let pause_action = shell
        .app
        .queue
        .history_action(pause[0].action_id)
        .expect("committed pause action");
    assert_eq!(pause_action.command, ActionCommand::TransportPause);
    assert_eq!(pause_action.status, ActionStatus::Committed);
    assert!(!shell.app.runtime.transport.is_playing);
    assert!(!shell.app.session.runtime_state.transport.is_playing);
    assert_eq!(shell.status_message, "transport paused");
}
