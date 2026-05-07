#[test]
fn current_ghost_suggestion_slot_archives_and_clears_without_queueing() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.ghost_state.suggestion_history.clear();
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.set_current_ghost_suggestion(ghost_fill_suggestion());

    assert_eq!(
        state
            .runtime
            .current_ghost_suggestion
            .as_ref()
            .map(|suggestion| suggestion.proposal_id.as_str()),
        Some("ghost-fill-1")
    );
    assert_eq!(state.session.ghost_state.suggestion_history.len(), 1);
    assert_eq!(
        state.session.ghost_state.suggestion_history[0].status(),
        riotbox_core::session::GhostSuggestionStatus::Suggested
    );
    assert!(state.queue.pending_actions().is_empty());

    state.clear_current_ghost_suggestion();

    assert!(state.runtime.current_ghost_suggestion.is_none());
    assert!(state.queue.pending_actions().is_empty());
}

#[test]
fn accept_current_ghost_suggestion_queues_in_assist_and_clears_slot() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.ghost_state.mode = GhostMode::Assist;
    session.ghost_state.suggestion_history.clear();
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    state.set_current_ghost_suggestion(ghost_fill_suggestion());

    let result = state.accept_current_ghost_suggestion(1_000);

    assert!(matches!(result, GhostSuggestionQueueResult::Enqueued(_)));
    assert!(state.runtime.current_ghost_suggestion.is_none());
    assert_eq!(state.queue.pending_actions().len(), 1);
    assert!(state.session.ghost_state.suggestion_history[0].accepted);
}

#[test]
fn accept_current_ghost_suggestion_respects_watch_read_only() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.ghost_state.mode = GhostMode::Watch;
    session.ghost_state.suggestion_history.clear();
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    state.set_current_ghost_suggestion(ghost_fill_suggestion());

    let result = state.accept_current_ghost_suggestion(1_000);

    assert_eq!(
        result,
        GhostSuggestionQueueResult::Rejected {
            reason: "ghost accept requires assist mode".into()
        }
    );
    assert!(state.runtime.current_ghost_suggestion.is_some());
    assert!(state.queue.pending_actions().is_empty());
    assert!(!state.session.ghost_state.suggestion_history[0].accepted);
}

#[test]
fn reject_current_ghost_suggestion_marks_decision_and_clears_slot() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.ghost_state.suggestion_history.clear();
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    state.set_current_ghost_suggestion(ghost_fill_suggestion());

    assert!(state.reject_current_ghost_suggestion());

    assert!(state.runtime.current_ghost_suggestion.is_none());
    assert_eq!(
        state.session.ghost_state.suggestion_history[0].status(),
        riotbox_core::session::GhostSuggestionStatus::Rejected
    );
    assert!(state.queue.pending_actions().is_empty());
}

#[test]
fn accept_current_ghost_suggestion_without_slot_is_explicit_noop() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.ghost_state.mode = GhostMode::Assist;
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.accept_current_ghost_suggestion(1_000),
        GhostSuggestionQueueResult::Rejected {
            reason: "no current ghost suggestion".into()
        }
    );
    assert!(!state.reject_current_ghost_suggestion());
    assert!(state.queue.pending_actions().is_empty());
}
