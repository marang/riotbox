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
fn queue_accepted_ghost_suggestion_respects_pending_budget() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.ghost_state.mode = GhostMode::Assist;
    session.ghost_state.budgets.max_pending_actions = 1;
    session.ghost_state.suggestion_history.clear();
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    let first = ghost_fill_suggestion();
    let mut second = ghost_fill_suggestion();
    second.proposal_id = "ghost-fill-2".into();

    assert!(matches!(
        state.queue_accepted_ghost_suggestion(&first, 1_000),
        GhostSuggestionQueueResult::Enqueued(_)
    ));

    let over_budget = state.queue_accepted_ghost_suggestion(&second, 1_001);

    assert_eq!(
        over_budget,
        GhostSuggestionQueueResult::Rejected {
            reason: "ghost pending action budget exceeded".into()
        }
    );
    assert_eq!(state.queue.pending_actions().len(), 1);
    assert!(
        state
            .session
            .ghost_state
            .suggestion_history
            .iter()
            .all(|record| record.proposal_id != "ghost-fill-2")
    );
}

#[test]
fn queue_accepted_ghost_suggestion_respects_phrase_budget() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.ghost_state.mode = GhostMode::Assist;
    session.ghost_state.budgets.max_actions_per_phrase = 1;
    session.ghost_state.budgets.max_pending_actions = 2;
    session.ghost_state.suggestion_history.clear();
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    state.update_transport_clock(TransportClockState {
        is_playing: true,
        position_beats: 48.0,
        beat_index: 48,
        bar_index: 12,
        phrase_index: 3,
        current_scene: None,
    });
    let first = ghost_fill_suggestion();
    let mut second = ghost_fill_suggestion();
    second.proposal_id = "ghost-fill-2".into();

    assert!(matches!(
        state.queue_accepted_ghost_suggestion(&first, 1_000),
        GhostSuggestionQueueResult::Enqueued(_)
    ));
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 48,
            bar_index: 12,
            phrase_index: 3,
            scene_id: None,
        },
        1_500,
    );
    assert_eq!(committed.len(), 1);

    let over_budget = state.queue_accepted_ghost_suggestion(&second, 1_501);

    assert_eq!(
        over_budget,
        GhostSuggestionQueueResult::Rejected {
            reason: "ghost phrase action budget exceeded".into()
        }
    );
    assert_eq!(state.queue.pending_actions().len(), 0);
    assert_eq!(state.session.action_log.commit_records.len(), 1);
    assert_eq!(
        state.session.action_log.commit_records[0].boundary.phrase_index,
        3
    );
    assert!(
        state
            .session
            .ghost_state
            .suggestion_history
            .iter()
            .all(|record| record.proposal_id != "ghost-fill-2")
    );
}

#[test]
fn queue_accepted_ghost_suggestion_respects_destructive_scene_budget() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.ghost_state.mode = GhostMode::Assist;
    session.ghost_state.budgets.max_actions_per_phrase = 2;
    session.ghost_state.budgets.max_pending_actions = 2;
    session
        .ghost_state
        .budgets
        .max_destructive_actions_per_scene = 1;
    session.ghost_state.suggestion_history.clear();
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    state.update_transport_clock(TransportClockState {
        is_playing: true,
        position_beats: 64.0,
        beat_index: 64,
        bar_index: 16,
        phrase_index: 4,
        current_scene: Some(SceneId::from("scene-a")),
    });
    let first = ghost_destructive_takeover_suggestion("ghost-takeover-1");
    let second = ghost_destructive_takeover_suggestion("ghost-takeover-2");

    assert!(matches!(
        state.queue_accepted_ghost_suggestion(&first, 1_000),
        GhostSuggestionQueueResult::Enqueued(_)
    ));
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 64,
            bar_index: 16,
            phrase_index: 4,
            scene_id: Some(SceneId::from("scene-a")),
        },
        1_500,
    );
    assert_eq!(committed.len(), 1);

    let over_budget = state.queue_accepted_ghost_suggestion(&second, 1_501);

    assert_eq!(
        over_budget,
        GhostSuggestionQueueResult::Rejected {
            reason: "ghost destructive scene budget exceeded".into()
        }
    );
    assert_eq!(state.queue.pending_actions().len(), 0);
    assert!(
        state
            .session
            .ghost_state
            .suggestion_history
            .iter()
            .all(|record| record.proposal_id != "ghost-takeover-2")
    );
}

#[test]
fn queue_accepted_ghost_suggestion_does_not_spend_destructive_budget_for_fill() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.ghost_state.mode = GhostMode::Assist;
    session
        .ghost_state
        .budgets
        .max_destructive_actions_per_scene = 0;
    session.ghost_state.suggestion_history.clear();
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert!(matches!(
        state.queue_accepted_ghost_suggestion(&ghost_fill_suggestion(), 1_000),
        GhostSuggestionQueueResult::Enqueued(_)
    ));
    assert_eq!(state.queue.pending_actions().len(), 1);
}
