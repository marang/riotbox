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
