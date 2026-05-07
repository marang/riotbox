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
