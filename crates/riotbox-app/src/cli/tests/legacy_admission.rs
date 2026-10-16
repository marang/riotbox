//! Existing shell-control and observer seams, never an interactive terminal.
use std::fs;

use crate::{
    cli::{
        controls::{
            commit_transport_toggle, queue_and_commit_source_monitor_mode,
            record_key_outcome_then_immediate_commit,
        },
        observer::UserSessionObserver,
    },
    jam_app::JamAppState,
    ui::{JamShellState, ShellKeyOutcome, ShellLaunchMode},
};
use riotbox_core::{
    action::{
        ActionCommand, ActionDraft, ActionTarget, ActorType, Quantization, SourceMonitorMode,
    },
    queue::ActionQueue,
    session::{ReplayPolicy, SessionFile},
};

#[test]
fn refusal_is_visible_and_observer_records_only_intent_not_a_successful_commit() {
    let owner = tempfile::tempdir().unwrap();
    for policy in [
        ReplayPolicy::DeterministicPreferred,
        ReplayPolicy::SnapshotOnly,
    ] {
        for toggle in [false, true] {
            let mut session = SessionFile::new("generated-shell-admission", "test", "2026-10-16");
            session.action_log.replay_policy = policy;
            session.runtime_state.transport.position_beats = 12.0;
            let mut shell = JamShellState::new(
                JamAppState::from_parts(session, None, ActionQueue::new()),
                ShellLaunchMode::Load,
            );
            shell
                .app
                .queue
                .enqueue(
                    ActionDraft::new(
                        ActorType::User,
                        ActionCommand::TransportStop,
                        Quantization::Immediate,
                        ActionTarget::default(),
                    ),
                    10,
                )
                .unwrap();
            let before = shell.app.session.clone();
            let next_id = shell.app.queue.clone().allocate_action_id().unwrap();
            let committed = if toggle {
                commit_transport_toggle(&mut shell, 20)
            } else {
                queue_and_commit_source_monitor_mode(&mut shell, SourceMonitorMode::Blend, 20)
            };
            assert!(committed.is_empty());
            assert_eq!(shell.app.session, before);
            assert!(
                shell
                    .status_message
                    .contains("operation refused for declared replay policy")
            );
            assert!(
                shell
                    .status_message
                    .contains("committed history was not changed")
            );
            if policy == ReplayPolicy::SnapshotOnly {
                assert!(
                    shell
                        .status_message
                        .contains("SnapshotOnly is not supported")
                );
            } else {
                assert!(shell.status_message.contains("unambiguous legacy history"));
            }
            if toggle {
                assert_eq!(shell.app.queue.pending_actions().len(), 1);
                assert_eq!(
                    shell.app.queue.clone().allocate_action_id().unwrap(),
                    next_id
                );
            } else {
                assert!(shell.status_message.starts_with("queued; commit blocked:"));
                assert_eq!(shell.app.queue.pending_actions().len(), 2);
                assert_eq!(shell.app.queue.pending_actions()[1].id, next_id);
            }
            let path = owner.path().join(format!("{policy:?}-{toggle}.ndjson"));
            let mut observer = UserSessionObserver::open(&path).unwrap();
            record_key_outcome_then_immediate_commit(
                &mut observer,
                20,
                if toggle { "space" } else { "M" },
                if toggle {
                    ShellKeyOutcome::ToggleTransport
                } else {
                    ShellKeyOutcome::QueueSourceMonitorMode(SourceMonitorMode::Blend)
                },
                &shell,
                &committed,
            )
            .unwrap();
            drop(observer);
            let events: Vec<serde_json::Value> = fs::read_to_string(path)
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0]["event"], "key_outcome");
            assert!(shell.app.queue.history().is_empty());
        }
    }
}
