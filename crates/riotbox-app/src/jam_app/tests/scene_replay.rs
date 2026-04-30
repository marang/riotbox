#[test]
fn graph_aware_scene_replay_matches_committed_movement_projection() {
    let graph = scene_regression_graph(&["drop".into(), "break".into()]);
    let mut session = sample_session(&graph);
    session.runtime_state.transport.position_beats = 32.0;
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-drop"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-drop"));
    session.runtime_state.scene_state.restore_scene = None;
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-drop"),
        SceneId::from("scene-02-break"),
    ];
    session.runtime_state.lane_state.tr909.reinforcement_mode =
        Some(Tr909ReinforcementModeState::SourceSupport);
    session.runtime_state.lane_state.tr909.pattern_ref = Some("scene-replay-support".into());
    session.runtime_state.lane_state.mc202.role = Some("follower".into());
    session.runtime_state.lane_state.mc202.phrase_variant = None;

    let base_session = session.clone();
    let mut committed_state =
        JamAppState::from_parts(session, Some(graph.clone()), ActionQueue::new());
    let before_scene = render_scene_recipe_mix_buffer(&committed_state);
    assert_eq!(
        committed_state.runtime.tr909_render.source_support_profile,
        Some(Tr909SourceSupportProfile::DropDrive)
    );

    assert_eq!(
        committed_state.queue_scene_select(300),
        QueueControlResult::Enqueued
    );
    let committed = committed_state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 36,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-01-drop")),
        },
        360,
    );
    assert_eq!(committed.len(), 1);
    let committed_scene = render_scene_recipe_mix_buffer(&committed_state);

    let plan = riotbox_core::replay::build_committed_replay_plan(
        &committed_state.session.action_log,
    )
    .expect("committed scene action log builds replay plan");
    let mut replayed_session = base_session;
    replayed_session.action_log = committed_state.session.action_log.clone();
    let report = riotbox_core::replay::apply_graph_aware_replay_plan_to_session(
        &mut replayed_session,
        &plan,
        &graph,
    )
    .expect("graph-aware Scene replay applies launch subset");
    let replayed_state = JamAppState::from_parts(replayed_session, Some(graph), ActionQueue::new());
    let replayed_scene = render_scene_recipe_mix_buffer(&replayed_state);

    assert_eq!(report.applied_action_ids.len(), 1);
    assert_eq!(
        replayed_state.session.runtime_state.scene_state.active_scene,
        committed_state.session.runtime_state.scene_state.active_scene
    );
    assert_eq!(
        replayed_state.session.runtime_state.transport.current_scene,
        committed_state.session.runtime_state.transport.current_scene
    );
    assert_eq!(
        replayed_state.session.runtime_state.scene_state.restore_scene,
        committed_state.session.runtime_state.scene_state.restore_scene
    );
    assert_eq!(
        replayed_state.runtime.tr909_render.current_scene_id.as_deref(),
        Some("scene-02-break")
    );
    assert_eq!(
        replayed_state.runtime.tr909_render.source_support_profile,
        Some(Tr909SourceSupportProfile::BreakLift)
    );
    assert_eq!(
        replayed_state.runtime.tr909_render.source_support_context,
        Some(Tr909SourceSupportContext::SceneTarget)
    );
    assert_eq!(
        replayed_state.session.runtime_state.scene_state.last_movement,
        committed_state.session.runtime_state.scene_state.last_movement
    );
    assert_eq!(
        replayed_state.runtime.tr909_render,
        committed_state.runtime.tr909_render
    );
    assert_eq!(
        replayed_state.runtime.mc202_render,
        committed_state.runtime.mc202_render
    );
    assert_recipe_buffers_match(
        "graph-aware scene replay -> committed scene movement",
        &replayed_scene,
        &committed_scene,
        0.00001,
    );
    assert_recipe_buffers_differ(
        "scene replay projected output leaves previous scene",
        &before_scene,
        &replayed_scene,
        0.002,
    );
}

#[test]
fn scene_restore_snapshot_payload_restore_matches_committed_movement_projection() {
    let graph = scene_regression_graph(&["drop".into(), "break".into()]);
    let mut session = sample_session(&graph);
    session.runtime_state.transport.position_beats = 32.0;
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-drop"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-drop"));
    session.runtime_state.scene_state.restore_scene = None;
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-drop"),
        SceneId::from("scene-02-break"),
    ];
    session.runtime_state.lane_state.tr909.reinforcement_mode =
        Some(Tr909ReinforcementModeState::SourceSupport);
    session.runtime_state.lane_state.tr909.pattern_ref = Some("scene-restore-support".into());
    session.runtime_state.lane_state.mc202.role = Some("follower".into());
    session.runtime_state.lane_state.mc202.phrase_variant = None;

    let base_session = session.clone();
    let mut committed_state =
        JamAppState::from_parts(session, Some(graph.clone()), ActionQueue::new());
    let before_launch = render_scene_recipe_mix_buffer(&committed_state);

    assert_eq!(
        committed_state.queue_scene_select(300),
        QueueControlResult::Enqueued
    );
    let launched = committed_state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 36,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-01-drop")),
        },
        360,
    );
    assert_eq!(launched.len(), 1);
    let after_launch = render_scene_recipe_mix_buffer(&committed_state);

    assert_eq!(
        committed_state.queue_scene_restore(420),
        QueueControlResult::Enqueued
    );
    let restored = committed_state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 40,
            bar_index: 10,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-02-break")),
        },
        480,
    );
    assert_eq!(restored.len(), 1);
    let committed_restore = render_scene_recipe_mix_buffer(&committed_state);

    let replayed_state = run_graph_aware_snapshot_payload_restore_probe(
        base_session,
        &committed_state,
        graph,
        SnapshotPayloadRestoreSpec {
            plan_label: "committed Scene restore action log builds replay plan",
            snapshot_id: "snap-after-scene-launch",
            snapshot_label: "after Scene launch before restore",
            snapshot_created_at: "2026-04-30T16:40:00Z",
            expected_plan_len: 2,
            anchor_plan_len: 1,
            target_plan_index: 1,
            anchor_label: "Scene launch anchor materializes before restore",
            restore_expectation: "snapshot payload restore applies Scene restore suffix",
        },
        |_| {},
    );
    let replayed_restore = render_scene_recipe_mix_buffer(&replayed_state);

    assert_eq!(
        replayed_state.session.runtime_state.scene_state.active_scene,
        committed_state.session.runtime_state.scene_state.active_scene
    );
    assert_eq!(
        replayed_state.session.runtime_state.scene_state.restore_scene,
        committed_state.session.runtime_state.scene_state.restore_scene
    );
    assert_eq!(
        replayed_state.session.runtime_state.scene_state.last_movement,
        committed_state.session.runtime_state.scene_state.last_movement
    );
    assert_eq!(
        replayed_state.session.runtime_state.transport.current_scene,
        committed_state.session.runtime_state.transport.current_scene
    );
    assert_eq!(
        replayed_state.jam_view.scene.active_scene,
        committed_state.jam_view.scene.active_scene
    );
    assert_eq!(
        replayed_state.jam_view.scene.restore_scene,
        committed_state.jam_view.scene.restore_scene
    );
    assert_eq!(
        replayed_state.runtime.tr909_render,
        committed_state.runtime.tr909_render
    );
    assert_eq!(
        replayed_state.runtime.mc202_render,
        committed_state.runtime.mc202_render
    );
    assert_recipe_buffers_match(
        "snapshot payload restore Scene restore -> committed restore",
        &replayed_restore,
        &committed_restore,
        0.00001,
    );
    assert_recipe_buffers_differ(
        "snapshot payload restore Scene launch -> restore",
        &after_launch,
        &replayed_restore,
        0.004,
    );
    assert_recipe_buffers_differ(
        "snapshot payload restore Scene restore keeps movement energy",
        &before_launch,
        &replayed_restore,
        0.002,
    );
}
