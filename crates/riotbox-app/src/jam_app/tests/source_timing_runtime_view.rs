#[test]
fn source_timing_grid_confirmation_queues_commits_and_persists_session_truth() {
    let mut graph = sample_graph();
    graph.timing.primary_hypothesis_id = Some("primary-grid".into());
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert!(state.session.runtime_state.source_timing.confirmed_grid.is_none());
    assert_eq!(
        state.queue_source_timing_grid_confirmation(100),
        QueueControlResult::Enqueued
    );
    assert!(matches!(
        state.queue.pending_actions()[0].undo_policy,
        UndoPolicy::NotUndoable { .. }
    ));
    assert_eq!(
        state.queue_source_timing_grid_confirmation(101),
        QueueControlResult::AlreadyPending
    );

    let committed = state.commit_ready_actions(immediate_boundary(), 120);

    assert_eq!(committed.len(), 1);
    let confirmed = state
        .session
        .runtime_state
        .source_timing
        .confirmed_grid
        .as_ref()
        .expect("source timing grid confirmation");
    assert_eq!(confirmed.source_id, SourceId::from("src-1"));
    assert_eq!(confirmed.hypothesis_id.as_deref(), Some("primary-grid"));
    assert_eq!(confirmed.confirmed_by_action, committed[0].action_id);
    assert_eq!(confirmed.confirmed_at, 120);
    assert_eq!(
        state.queue_source_timing_grid_confirmation(121),
        QueueControlResult::AlreadyInState
    );
}

#[test]
fn source_timing_grid_revert_queues_commits_and_clears_session_truth() {
    let mut graph = sample_graph();
    graph.timing.primary_hypothesis_id = Some("primary-grid".into());
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_source_timing_grid_revert(99),
        QueueControlResult::AlreadyInState
    );
    assert_eq!(
        state.queue_source_timing_grid_confirmation(100),
        QueueControlResult::Enqueued
    );
    let committed_confirm = state.commit_ready_actions(immediate_boundary(), 120);
    assert_eq!(committed_confirm.len(), 1);
    assert!(state.session.runtime_state.source_timing.confirmed_grid.is_some());

    assert_eq!(
        state.queue_source_timing_grid_revert(121),
        QueueControlResult::Enqueued
    );
    assert_eq!(
        state.queue_source_timing_grid_revert(122),
        QueueControlResult::AlreadyPending
    );
    let committed_revert = state.commit_ready_actions(immediate_boundary(), 140);

    assert_eq!(committed_revert.len(), 1);
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .expect("committed source timing revert")
            .command,
        ActionCommand::SourceTimingRevertGrid
    );
    assert!(state.session.runtime_state.source_timing.confirmed_grid.is_none());
    assert_eq!(
        state.queue_source_timing_grid_revert(141),
        QueueControlResult::AlreadyInState
    );
}

fn immediate_boundary() -> CommitBoundaryState {
    CommitBoundaryState {
        kind: CommitBoundary::Immediate,
        beat_index: 0,
        bar_index: 0,
        phrase_index: 0,
        scene_id: None,
    }
}
