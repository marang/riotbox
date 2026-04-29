#[test]
fn queue_accepted_ghost_suggestion_uses_normal_action_queue() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.ghost_state.mode = GhostMode::Assist;
    session.ghost_state.suggestion_history.clear();
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    let suggestion = ghost_fill_suggestion();

    let result = state.queue_accepted_ghost_suggestion(&suggestion, 1_000);

    let GhostSuggestionQueueResult::Enqueued(action_id) = result else {
        panic!("expected enqueued ghost suggestion");
    };
    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, action_id);
    assert_eq!(pending[0].actor, ActorType::Ghost);
    assert_eq!(pending[0].command, ActionCommand::Tr909FillNext);
    assert_eq!(pending[0].quantization, Quantization::NextBar);
    assert_eq!(pending[0].target.scope, Some(TargetScope::LaneTr909));
    assert_eq!(
        pending[0].explanation.as_deref(),
        Some("ghost accepted ghost-fill-1: add a next-bar drum answer")
    );

    let record = state
        .session
        .ghost_state
        .suggestion_history
        .iter()
        .find(|record| record.proposal_id == "ghost-fill-1")
        .expect("suggestion archived");
    assert!(record.accepted);
    assert!(!record.rejected);
    assert_eq!(state.jam_view.pending_actions.len(), 1);
}

#[test]
fn queue_accepted_ghost_suggestion_rejects_watch_and_blocked_proposals() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.ghost_state.mode = GhostMode::Watch;
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let watch_result = state.queue_accepted_ghost_suggestion(&ghost_fill_suggestion(), 1_000);

    assert_eq!(
        watch_result,
        GhostSuggestionQueueResult::Rejected {
            reason: "ghost accept requires assist mode".into()
        }
    );
    assert!(state.queue.pending_actions().is_empty());

    state.session.ghost_state.mode = GhostMode::Assist;
    let mut blocked = ghost_fill_suggestion();
    blocked.safety = GhostSuggestionSafety::Blocked;
    blocked.blockers = vec![GhostSuggestionBlocker {
        kind: GhostSuggestionBlockerKind::Lock,
        object_id: Some("lane.tr909".into()),
        reason: "TR-909 lane is locked".into(),
    }];

    let blocked_result = state.queue_accepted_ghost_suggestion(&blocked, 1_001);

    assert_eq!(
        blocked_result,
        GhostSuggestionQueueResult::Rejected {
            reason: "ghost proposal is blocked".into()
        }
    );
    assert!(state.queue.pending_actions().is_empty());
}

#[test]
fn queue_accepted_ghost_suggestion_does_not_enqueue_decided_proposal_twice() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.ghost_state.mode = GhostMode::Assist;
    session.ghost_state.suggestion_history.clear();
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    let suggestion = ghost_fill_suggestion();

    assert!(matches!(
        state.queue_accepted_ghost_suggestion(&suggestion, 1_000),
        GhostSuggestionQueueResult::Enqueued(_)
    ));

    let duplicate = state.queue_accepted_ghost_suggestion(&suggestion, 1_001);

    assert_eq!(
        duplicate,
        GhostSuggestionQueueResult::Rejected {
            reason: "ghost proposal ghost-fill-1 already decided".into()
        }
    );
    assert_eq!(state.queue.pending_actions().len(), 1);
}

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

fn ghost_fill_suggestion() -> GhostWatchSuggestion {
    GhostWatchSuggestion {
        proposal_id: "ghost-fill-1".into(),
        mode: GhostMode::Watch,
        tool_name: GhostWatchTool::SuggestMacroShift,
        summary: "add a drum answer".into(),
        rationale: "TR-909 has room for a next-bar support move".into(),
        suggested_action: Some(GhostSuggestedAction {
            command: ActionCommand::Tr909FillNext,
            target: ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
            quantization: Quantization::NextBar,
            intent: "add a next-bar drum answer".into(),
        }),
        confidence: GhostSuggestionConfidence::Medium,
        safety: GhostSuggestionSafety::SafeToSuggest,
        blockers: Vec::new(),
        created_at: "2026-04-29T17:00:00Z".into(),
    }
}
