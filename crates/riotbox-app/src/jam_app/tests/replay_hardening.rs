#[test]
fn replay_from_zero_restore_rebuilds_commit_boundary_and_queue_cursor() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut live = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let first = live.queue.enqueue(
        ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909FillNext,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        ),
        300,
    );
    let second = live.queue.enqueue(
        ActionDraft::new(
            ActorType::User,
            ActionCommand::MutateScene,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::Scene),
                scene_id: Some(SceneId::from("scene-1")),
                ..Default::default()
            },
        ),
        301,
    );
    let boundary = CommitBoundaryState {
        kind: CommitBoundary::Bar,
        beat_index: 80,
        bar_index: 20,
        phrase_index: 5,
        scene_id: Some(SceneId::from("scene-1")),
    };

    let committed = live.commit_ready_actions(boundary.clone(), 500);

    assert_eq!(committed.len(), 2);
    assert_eq!(committed[0].action_id, first);
    assert_eq!(committed[0].commit_sequence, 1);
    assert_eq!(committed[1].action_id, second);
    assert_eq!(committed[1].commit_sequence, 2);
    assert_eq!(live.runtime.last_commit_boundary, Some(boundary.clone()));

    let tempdir = tempdir().expect("create replay hardening tempdir");
    let session_path = tempdir.path().join("replay-from-zero-session.json");
    save_session_json(&session_path, &live.session).expect("save replay fixture session");

    let mut restored =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect("restore from session");

    assert_eq!(restored.runtime.last_commit_boundary, Some(boundary));
    assert_eq!(restored.queue.pending_actions().len(), 0);
    assert_eq!(restored.session.action_log.commit_records.len(), 2);
    assert_eq!(restored.session.action_log.commit_records[0].action_id, first);
    assert_eq!(restored.session.action_log.commit_records[1].action_id, second);
    assert_eq!(restored.jam_view.recent_actions[0].id, second.to_string());
    assert_eq!(restored.jam_view.recent_actions[1].id, first.to_string());

    let next_id = restored.queue.enqueue(
        ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909FillNext,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        ),
        800,
    );

    assert_eq!(next_id, ActionId(4));
}

#[test]
fn accepted_ghost_action_snapshot_replay_plan_uses_restored_commit_records() {
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
    state.session.snapshots = vec![
        Snapshot {
            snapshot_id: SnapshotId::from("before-ghost"),
            created_at: "2026-04-29T19:12:00Z".into(),
            label: "before ghost fill".into(),
            action_cursor: 0,
        },
        Snapshot {
            snapshot_id: SnapshotId::from("after-ghost"),
            created_at: "2026-04-29T19:12:01Z".into(),
            label: "after ghost fill".into(),
            action_cursor: state.session.action_log.actions.len(),
        },
    ];

    let tempdir = tempdir().expect("create ghost snapshot replay tempdir");
    let session_path = tempdir.path().join("ghost-snapshot-replay-session.json");
    save_session_json(&session_path, &state.session).expect("save ghost replay session");
    let reloaded = load_session_json(&session_path).expect("reload ghost replay session");

    let before_snapshot = reloaded
        .snapshots
        .iter()
        .find(|snapshot| snapshot.snapshot_id.as_str() == "before-ghost")
        .expect("before snapshot restored");
    let after_snapshot = reloaded
        .snapshots
        .iter()
        .find(|snapshot| snapshot.snapshot_id.as_str() == "after-ghost")
        .expect("after snapshot restored");
    let before_comparison = riotbox_core::replay::build_snapshot_replay_plan_comparison(
        &reloaded.action_log,
        before_snapshot,
    )
    .expect("before snapshot replay plan");
    let after_comparison =
        riotbox_core::replay::build_snapshot_replay_plan_comparison(&reloaded.action_log, after_snapshot)
            .expect("after snapshot replay plan");

    assert_eq!(before_comparison.origin.len(), 1);
    let ghost_entry = &before_comparison.origin[0];
    assert_eq!(ghost_entry.action.actor, ActorType::Ghost);
    assert_eq!(ghost_entry.action.command, ActionCommand::Tr909FillNext);
    assert_eq!(before_comparison.snapshot_suffix.len(), 1);
    assert_eq!(
        ghost_entry.commit_record.action_id,
        before_comparison.snapshot_suffix[0].action.id
    );
    assert!(after_comparison.snapshot_suffix.is_empty());

    let target_plan_from_before = riotbox_core::replay::build_replay_target_plan(
        &reloaded.action_log,
        std::slice::from_ref(before_snapshot),
        reloaded.action_log.actions.len(),
    )
    .expect("before snapshot target replay plan");
    let target_plan_from_after = riotbox_core::replay::build_replay_target_plan(
        &reloaded.action_log,
        &reloaded.snapshots,
        reloaded.action_log.actions.len(),
    )
    .expect("after snapshot target replay plan");

    assert_eq!(
        target_plan_from_before
            .anchor
            .map(|snapshot| snapshot.snapshot_id.as_str()),
        Some("before-ghost")
    );
    assert_eq!(target_plan_from_before.suffix.len(), 1);
    assert_eq!(target_plan_from_before.suffix[0].action.actor, ActorType::Ghost);
    assert_eq!(
        target_plan_from_after
            .anchor
            .map(|snapshot| snapshot.snapshot_id.as_str()),
        Some("after-ghost")
    );
    assert!(target_plan_from_after.suffix.is_empty());
}
