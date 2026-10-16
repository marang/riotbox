//! Existing UI feedback distinguishes queue admission from commit capacity.
use crate::{
    cli::controls::{
        commit_capture_length_change, commit_transport_toggle, confirm_source_timing_grid,
        queue_and_commit_performance_preset, queue_and_commit_source_monitor_mode,
    },
    jam_app::JamAppState,
    ui::{JamShellState, ShellLaunchMode},
};
use riotbox_core::{
    action::{
        ActionCommand, ActionDraft, ActionTarget, ActorType, CommitBoundary, Quantization,
        SourceMonitorMode,
    },
    queue::ActionQueue,
    session::{ActionCommitRecord, SessionFile},
    style::PerformancePresetId,
};

fn shell() -> JamShellState {
    let mut state = JamAppState::from_parts(
        SessionFile::new(
            "generated-sequence-feedback",
            "test",
            "2026-10-16T00:00:00Z",
        ),
        None,
        ActionQueue::new(),
    );
    let boundary = state
        .runtime
        .transport
        .boundary_state(CommitBoundary::Immediate);
    let id = state
        .queue
        .enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::TransportPause,
                Quantization::Immediate,
                ActionTarget::default(),
            ),
            1,
        )
        .unwrap();
    state
        .queue
        .commit_ready_for_transport(&state.session.action_log, boundary.clone(), 2)
        .unwrap();
    state
        .session
        .action_log
        .actions
        .push(state.queue.history_action(id).unwrap().clone());
    state
        .session
        .action_log
        .commit_records
        .push(ActionCommitRecord {
            action_id: id,
            boundary,
            commit_sequence: u32::MAX,
            committed_at: 2,
            mc202_source_phrase_plan: None,
        });
    JamShellState::new(state, ShellLaunchMode::Load)
}

#[test]
fn direct_toggle_refuses_without_queue_or_session_mutation() {
    let mut shell = shell();
    let session = shell.app.session.clone();
    let runtime = shell.app.runtime.clone();
    let queue = format!("{:?}", shell.app.queue);
    assert!(commit_transport_toggle(&mut shell, 100).is_empty());
    assert!(
        shell
            .status_message
            .starts_with("commit sequence exhausted")
    );
    assert_eq!(shell.app.session, session);
    assert_eq!(shell.app.runtime, runtime);
    assert_eq!(format!("{:?}", shell.app.queue), queue);
}

#[test]
fn queue_then_commit_keeps_real_pending_action_and_reports_blocked_commit() {
    let operations: [fn(&mut JamShellState); 3] = [
        |shell| {
            assert!(
                queue_and_commit_source_monitor_mode(shell, SourceMonitorMode::Blend, 100)
                    .is_empty()
            )
        },
        |shell| {
            assert!(
                queue_and_commit_performance_preset(
                    shell,
                    PerformancePresetId::FeralBreakAlphaV2,
                    100
                )
                .is_empty()
            )
        },
        |shell| commit_capture_length_change(shell, 100, true),
    ];
    for operation in operations {
        let mut shell = shell();
        let session = shell.app.session.clone();
        let runtime = shell.app.runtime.clone();
        let history = shell.app.queue.history().to_vec();
        operation(&mut shell);
        assert!(
            shell
                .status_message
                .starts_with("queued; commit blocked: commit sequence exhausted")
        );
        assert_eq!(shell.app.queue.pending_actions().len(), 1);
        assert_eq!(shell.app.queue.history(), history);
        assert_eq!(shell.app.session, session);
        assert_eq!(shell.app.runtime, runtime);
        let queue = format!("{:?}", shell.app.queue);
        operation(&mut shell);
        assert!(shell.status_message.contains("already queued"));
        assert_eq!(format!("{:?}", shell.app.queue), queue);
    }
}

#[test]
fn unavailable_grid_precedence_does_not_become_a_capacity_error() {
    let mut shell = shell();
    let session = shell.app.session.clone();
    let queue = format!("{:?}", shell.app.queue);
    confirm_source_timing_grid(&mut shell, 100);
    assert_eq!(
        shell.status_message,
        "no source timing grid available to confirm"
    );
    assert_eq!(shell.app.session, session);
    assert_eq!(format!("{:?}", shell.app.queue), queue);
}
