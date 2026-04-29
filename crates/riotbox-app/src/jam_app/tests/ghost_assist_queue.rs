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

#[test]
fn accepted_ghost_action_replay_fixture_survives_session_roundtrip() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.ghost_state.mode = GhostMode::Assist;
    session.ghost_state.suggestion_history.clear();
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    state.update_transport_clock(TransportClockState {
        is_playing: true,
        position_beats: 32.0,
        beat_index: 32,
        bar_index: 8,
        phrase_index: 2,
        current_scene: Some(SceneId::from("scene-a")),
    });
    state.set_current_ghost_suggestion(ghost_fill_suggestion());

    assert!(matches!(
        state.accept_current_ghost_suggestion(1_000),
        GhostSuggestionQueueResult::Enqueued(_)
    ));
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 32,
            bar_index: 8,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-a")),
        },
        1_500,
    );
    assert_eq!(committed.len(), 1);

    let tempdir = tempdir().expect("create ghost replay fixture tempdir");
    let session_path = tempdir.path().join("ghost-accepted-action-session.json");
    save_session_json(&session_path, &state.session).expect("save ghost replay fixture session");
    let reloaded = load_session_json(&session_path).expect("reload ghost replay fixture session");

    let action = reloaded
        .action_log
        .actions
        .iter()
        .find(|action| {
            action.actor == ActorType::Ghost && action.command == ActionCommand::Tr909FillNext
        })
        .expect("committed ghost action survived reload");
    assert_eq!(action.actor, ActorType::Ghost);
    assert_eq!(action.command, ActionCommand::Tr909FillNext);
    assert_eq!(action.status, ActionStatus::Committed);
    assert_eq!(action.committed_at, Some(1_500));
    assert_eq!(
        action.explanation.as_deref(),
        Some("ghost accepted ghost-fill-1: add a next-bar drum answer")
    );
    assert_eq!(reloaded.action_log.commit_records.len(), 1);
    let commit_record = &reloaded.action_log.commit_records[0];
    assert_eq!(commit_record.action_id, action.id);
    assert_eq!(commit_record.boundary.kind, CommitBoundary::Bar);
    assert_eq!(commit_record.boundary.beat_index, 32);
    assert_eq!(commit_record.boundary.bar_index, 8);
    assert_eq!(commit_record.boundary.phrase_index, 2);
    assert_eq!(
        commit_record
            .boundary
            .scene_id
            .as_ref()
            .map(ToString::to_string),
        Some("scene-a".into())
    );
    assert_eq!(commit_record.commit_sequence, 1);
    assert_eq!(commit_record.committed_at, 1_500);

    let record = reloaded
        .ghost_state
        .suggestion_history
        .iter()
        .find(|record| record.proposal_id == "ghost-fill-1")
        .expect("ghost suggestion decision survived reload");
    assert_eq!(
        record.status(),
        riotbox_core::session::GhostSuggestionStatus::Accepted
    );
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

#[test]
fn jam_state_feed_creates_source_backed_capture_suggestion_in_assist() {
    let graph = ghost_capture_candidate_graph();
    let mut session = sample_session(&graph);
    session.ghost_state.mode = GhostMode::Assist;
    session.ghost_state.suggestion_history.clear();
    session.captures.clear();
    session.runtime_state.lane_state.w30.last_capture = None;

    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert!(state.refresh_current_ghost_suggestion_from_jam_state());

    let suggestion = state
        .runtime
        .current_ghost_suggestion
        .as_ref()
        .expect("current ghost suggestion");
    assert_eq!(suggestion.proposal_id, "ghost-jam-capture-src-1");
    assert_eq!(suggestion.mode, GhostMode::Assist);
    assert_eq!(suggestion.tool_name, GhostWatchTool::SuggestCapture);
    assert_eq!(suggestion.confidence, GhostSuggestionConfidence::High);
    assert_eq!(suggestion.safety, GhostSuggestionSafety::NeedsAssistAcceptance);
    assert_eq!(
        suggestion
            .suggested_action
            .as_ref()
            .expect("suggested action")
            .command,
        ActionCommand::CaptureNow
    );
    assert_eq!(
        suggestion
            .suggested_action
            .as_ref()
            .expect("suggested action")
            .target
            .scope,
        Some(TargetScope::LaneW30)
    );
    assert_eq!(state.session.ghost_state.suggestion_history.len(), 1);
    assert_eq!(state.jam_view.ghost.suggestion_count, 1);
    assert_eq!(
        state.jam_view.ghost.latest_summary.as_deref(),
        Some("capture the current source-backed hit")
    );
}

#[test]
fn jam_state_feed_does_not_create_watch_or_decided_capture_candidate() {
    let graph = ghost_capture_candidate_graph();
    let mut watch_session = sample_session(&graph);
    watch_session.ghost_state.mode = GhostMode::Watch;
    watch_session.ghost_state.suggestion_history.clear();
    watch_session.captures.clear();
    watch_session.runtime_state.lane_state.w30.last_capture = None;

    let mut watch_state =
        JamAppState::from_parts(watch_session, Some(graph.clone()), ActionQueue::new());

    assert!(!watch_state.refresh_current_ghost_suggestion_from_jam_state());
    assert!(watch_state.runtime.current_ghost_suggestion.is_none());
    assert!(watch_state.session.ghost_state.suggestion_history.is_empty());

    let mut decided_session = sample_session(&graph);
    decided_session.ghost_state.mode = GhostMode::Assist;
    decided_session.captures.clear();
    decided_session.runtime_state.lane_state.w30.last_capture = None;
    decided_session.ghost_state.suggestion_history = vec![GhostSuggestionRecord {
        proposal_id: "ghost-jam-capture-src-1".into(),
        summary: "capture the current source-backed hit".into(),
        accepted: false,
        rejected: true,
    }];

    let mut decided_state = JamAppState::from_parts(decided_session, Some(graph), ActionQueue::new());

    assert!(!decided_state.refresh_current_ghost_suggestion_from_jam_state());

    assert!(decided_state.runtime.current_ghost_suggestion.is_none());
    assert_eq!(decided_state.session.ghost_state.suggestion_history.len(), 1);
    assert!(decided_state.session.ghost_state.suggestion_history[0].rejected);
}

#[test]
fn jam_state_feed_does_not_create_capture_suggestion_when_capture_already_exists() {
    let graph = ghost_capture_candidate_graph();
    let mut session = sample_session(&graph);
    session.ghost_state.mode = GhostMode::Assist;
    session.ghost_state.suggestion_history.clear();

    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert!(!state.refresh_current_ghost_suggestion_from_jam_state());

    assert!(state.runtime.current_ghost_suggestion.is_none());
    assert!(state.session.ghost_state.suggestion_history.is_empty());
}

#[test]
fn rejecting_auto_fed_ghost_suggestion_prevents_refresh_repopulation() {
    let graph = ghost_capture_candidate_graph();
    let mut session = sample_session(&graph);
    session.ghost_state.mode = GhostMode::Assist;
    session.ghost_state.suggestion_history.clear();
    session.captures.clear();
    session.runtime_state.lane_state.w30.last_capture = None;
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert!(state.refresh_current_ghost_suggestion_from_jam_state());
    assert!(state.runtime.current_ghost_suggestion.is_some());
    assert!(state.reject_current_ghost_suggestion());
    assert!(state.runtime.current_ghost_suggestion.is_none());

    assert!(!state.refresh_current_ghost_suggestion_from_jam_state());

    assert!(state.runtime.current_ghost_suggestion.is_none());
    assert_eq!(
        state.session.ghost_state.suggestion_history[0].status(),
        riotbox_core::session::GhostSuggestionStatus::Rejected
    );
}

#[test]
fn auto_fed_ghost_suggestion_respects_w30_lane_lock() {
    let graph = ghost_capture_candidate_graph();
    let mut session = sample_session(&graph);
    session.ghost_state.mode = GhostMode::Assist;
    session.ghost_state.suggestion_history.clear();
    session.captures.clear();
    session.runtime_state.lane_state.w30.last_capture = None;
    session.runtime_state.lock_state.locked_object_ids = vec!["lane.w30".into()];
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert!(state.refresh_current_ghost_suggestion_from_jam_state());
    let suggestion = state
        .runtime
        .current_ghost_suggestion
        .as_ref()
        .expect("current ghost suggestion");
    assert_eq!(suggestion.safety, GhostSuggestionSafety::Blocked);
    assert_eq!(
        suggestion.blockers[0].reason,
        "W-30 lane is locked by the performer"
    );

    assert_eq!(
        state.accept_current_ghost_suggestion(1_000),
        GhostSuggestionQueueResult::Rejected {
            reason: "ghost proposal is blocked".into()
        }
    );
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

fn ghost_destructive_takeover_suggestion(proposal_id: &str) -> GhostWatchSuggestion {
    let mut suggestion = ghost_fill_suggestion();
    suggestion.proposal_id = proposal_id.into();
    suggestion.tool_name = GhostWatchTool::SuggestSceneMutation;
    suggestion.summary = "take over the current scene".into();
    suggestion.rationale = "TR-909 can safely carry the next phrase".into();
    suggestion.suggested_action = Some(GhostSuggestedAction {
        command: ActionCommand::Tr909Takeover,
        target: ActionTarget {
            scope: Some(TargetScope::LaneTr909),
            ..Default::default()
        },
        quantization: Quantization::NextPhrase,
        intent: "take over the next phrase with TR-909".into(),
    });
    suggestion
}

fn ghost_capture_candidate_graph() -> SourceGraph {
    let mut graph = sample_graph();
    graph.candidates.push(Candidate {
        candidate_id: "capture-candidate-a".into(),
        candidate_type: CandidateType::CaptureCandidate,
        asset_ref: "asset-a".into(),
        score: 0.86,
        confidence: 0.88,
        tags: vec!["capture".into(), "feral".into()],
        constraints: vec!["bar_aligned".into()],
        provenance_refs: vec!["provider:decoded.wav_baseline".into()],
    });
    graph
}
