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
