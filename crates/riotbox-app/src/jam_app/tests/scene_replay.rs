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
