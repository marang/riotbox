#[test]
fn p014_scene_chain_launch_restore_replay_proves_transition_state_and_mix() {
    let graph = scene_regression_graph(&["break".into(), "drop".into(), "intro".into()]);
    let mut session = sample_session(&graph);
    session.runtime_state.transport.position_beats = 32.0;
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-break"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-break"));
    session.runtime_state.scene_state.restore_scene = None;
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-break"),
        SceneId::from("scene-02-drop"),
        SceneId::from("scene-03-intro"),
    ];
    session.runtime_state.lane_state.tr909.reinforcement_mode =
        Some(Tr909ReinforcementModeState::SourceSupport);
    session.runtime_state.lane_state.tr909.pattern_ref = Some("p014-scene-chain-support".into());
    session.runtime_state.lane_state.mc202.role = Some(Mc202RoleState::Follower);
    session.runtime_state.lane_state.mc202.phrase_variant = None;

    let base_session = session.clone();
    let mut committed_state =
        JamAppState::from_parts(session, Some(graph.clone()), ActionQueue::new());
    let contract = &committed_state.jam_view.scene.arrangement_contract;
    assert_eq!(contract.scene_count, 3);
    assert!(contract.has_active_scene);
    assert!(contract.has_next_scene);
    assert!(!contract.has_pending_scene_transition);
    assert!(contract.requires_replay_state_proof);
    assert!(contract.requires_output_path_proof_for_audible_changes);

    let baseline_break = render_scene_recipe_mix_buffer(&committed_state);

    assert_eq!(
        committed_state.queue_scene_select(300),
        QueueControlResult::Enqueued
    );
    assert!(
        committed_state
            .jam_view
            .scene
            .arrangement_contract
            .has_pending_scene_transition
    );
    commit_scene_chain_step(&mut committed_state, 36, 9, 2, "scene-01-break", 360);
    assert_scene_chain_movement(
        &committed_state,
        SceneChainMovementExpectation {
            kind: "launch",
            from_scene: Some("scene-01-break"),
            to_scene: "scene-02-drop",
            direction: SceneMovementDirectionState::Rise,
            tr909_intent: SceneMovementLaneIntentState::Drive,
            mc202_intent: SceneMovementLaneIntentState::Lift,
            committed_bar_index: 9,
            committed_phrase_index: 2,
        },
    );
    assert_eq!(
        committed_state.session.runtime_state.scene_state.restore_scene,
        Some(SceneId::from("scene-01-break"))
    );
    assert_eq!(
        committed_state.runtime.tr909_render.current_scene_id.as_deref(),
        Some("scene-02-drop")
    );
    assert_eq!(
        committed_state.runtime.tr909_render.source_support_context,
        Some(Tr909SourceSupportContext::SceneTarget)
    );
    assert_eq!(
        committed_state.runtime.tr909_render.phrase_variation,
        Some(Tr909PhraseVariation::PhraseDrive)
    );
    assert_eq!(
        committed_state.runtime.mc202_render.contour_hint,
        Mc202ContourHint::Lift
    );
    let first_drop = render_scene_recipe_mix_buffer(&committed_state);

    assert_eq!(
        committed_state.queue_scene_select(420),
        QueueControlResult::Enqueued
    );
    commit_scene_chain_step(&mut committed_state, 40, 10, 2, "scene-02-drop", 480);
    assert_scene_chain_movement(
        &committed_state,
        SceneChainMovementExpectation {
            kind: "launch",
            from_scene: Some("scene-02-drop"),
            to_scene: "scene-03-intro",
            direction: SceneMovementDirectionState::Drop,
            tr909_intent: SceneMovementLaneIntentState::Release,
            mc202_intent: SceneMovementLaneIntentState::Anchor,
            committed_bar_index: 10,
            committed_phrase_index: 2,
        },
    );
    assert_eq!(
        committed_state.session.runtime_state.scene_state.restore_scene,
        Some(SceneId::from("scene-02-drop"))
    );
    assert_eq!(
        committed_state.runtime.tr909_render.current_scene_id.as_deref(),
        Some("scene-03-intro")
    );
    assert_eq!(
        committed_state.runtime.tr909_render.source_support_context,
        Some(Tr909SourceSupportContext::SceneTarget)
    );
    assert_eq!(
        committed_state.runtime.tr909_render.phrase_variation,
        Some(Tr909PhraseVariation::PhraseRelease)
    );
    let second_intro = render_scene_recipe_mix_buffer(&committed_state);

    assert_eq!(
        committed_state.queue_scene_restore(540),
        QueueControlResult::Enqueued
    );
    commit_scene_chain_step(&mut committed_state, 44, 11, 2, "scene-03-intro", 600);
    assert_scene_chain_movement(
        &committed_state,
        SceneChainMovementExpectation {
            kind: "restore",
            from_scene: Some("scene-03-intro"),
            to_scene: "scene-02-drop",
            direction: SceneMovementDirectionState::Rise,
            tr909_intent: SceneMovementLaneIntentState::Drive,
            mc202_intent: SceneMovementLaneIntentState::Lift,
            committed_bar_index: 11,
            committed_phrase_index: 2,
        },
    );
    assert_eq!(
        committed_state.session.runtime_state.scene_state.restore_scene,
        Some(SceneId::from("scene-03-intro"))
    );
    assert_eq!(
        committed_state.runtime.tr909_render.current_scene_id.as_deref(),
        Some("scene-02-drop")
    );
    assert_eq!(
        committed_state.runtime.tr909_render.phrase_variation,
        Some(Tr909PhraseVariation::PhraseDrive)
    );
    assert_eq!(
        committed_state.runtime.mc202_render.contour_hint,
        Mc202ContourHint::Lift
    );
    let restored_drop = render_scene_recipe_mix_buffer(&committed_state);

    let scene_commands = committed_state
        .session
        .action_log
        .actions
        .iter()
        .filter(|action| {
            matches!(
                action.command,
                ActionCommand::SceneLaunch | ActionCommand::SceneRestore
            )
        })
        .map(|action| action.command)
        .collect::<Vec<_>>();
    assert_eq!(
        scene_commands,
        vec![
            ActionCommand::SceneLaunch,
            ActionCommand::SceneLaunch,
            ActionCommand::SceneRestore,
        ]
    );
    assert_eq!(committed_state.session.action_log.commit_records.len(), 3);

    let plan = riotbox_core::replay::build_committed_replay_plan(
        &committed_state.session.action_log,
    )
    .expect("P014 scene-chain action log builds replay plan");
    assert_eq!(plan.len(), 3);
    let mut replayed_session = base_session;
    replayed_session.action_log = committed_state.session.action_log.clone();
    let report = riotbox_core::replay::apply_graph_aware_replay_plan_to_session(
        &mut replayed_session,
        &plan,
        &graph,
    )
    .expect("P014 graph-aware Scene chain replay applies launch/restore plan");
    let replayed_state = JamAppState::from_parts(replayed_session, Some(graph), ActionQueue::new());
    let replayed_drop = render_scene_recipe_mix_buffer(&replayed_state);

    assert_eq!(report.applied_action_ids.len(), 3);
    assert_eq!(
        replayed_state.session.runtime_state.scene_state,
        committed_state.session.runtime_state.scene_state
    );
    assert_eq!(
        replayed_state.session.runtime_state.transport.current_scene,
        committed_state.session.runtime_state.transport.current_scene
    );
    assert_eq!(
        replayed_state.jam_view.scene.last_movement,
        committed_state.jam_view.scene.last_movement
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
        "P014 scene-chain replay -> committed restored drop",
        &replayed_drop,
        &restored_drop,
        0.00001,
    );
    assert_recipe_buffers_differ(
        "P014 first launch leaves baseline break",
        &baseline_break,
        &first_drop,
        0.004,
    );
    assert_recipe_buffers_differ(
        "P014 second launch leaves first drop",
        &first_drop,
        &second_intro,
        0.003,
    );
    assert_recipe_buffers_differ(
        "P014 restore leaves intro chain target",
        &second_intro,
        &restored_drop,
        0.0035,
    );
    assert_recipe_buffers_match(
        "P014 restored drop returns to first launch movement intent",
        &first_drop,
        &restored_drop,
        0.00001,
    );
}

fn commit_scene_chain_step(
    state: &mut JamAppState,
    beat_index: u64,
    bar_index: u64,
    phrase_index: u64,
    scene_id: &str,
    committed_at: u64,
) {
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index,
            bar_index,
            phrase_index,
            scene_id: Some(SceneId::from(scene_id)),
        },
        committed_at,
    );
    assert_eq!(committed.len(), 1);
}

struct SceneChainMovementExpectation<'a> {
    kind: &'a str,
    from_scene: Option<&'a str>,
    to_scene: &'a str,
    direction: SceneMovementDirectionState,
    tr909_intent: SceneMovementLaneIntentState,
    mc202_intent: SceneMovementLaneIntentState,
    committed_bar_index: u64,
    committed_phrase_index: u64,
}

fn assert_scene_chain_movement(
    state: &JamAppState,
    expected: SceneChainMovementExpectation<'_>,
) {
    let movement = state
        .session
        .runtime_state
        .scene_state
        .last_movement
        .as_ref()
        .expect("scene-chain transition records landed movement");
    assert_eq!(movement.kind.label(), expected.kind);
    assert_eq!(
        movement.from_scene.as_ref().map(ToString::to_string).as_deref(),
        expected.from_scene
    );
    assert_eq!(movement.to_scene, SceneId::from(expected.to_scene));
    assert_eq!(movement.direction, expected.direction);
    assert_eq!(movement.tr909_intent, expected.tr909_intent);
    assert_eq!(movement.mc202_intent, expected.mc202_intent);
    assert_eq!(movement.committed_bar_index, expected.committed_bar_index);
    assert_eq!(
        movement.committed_phrase_index,
        expected.committed_phrase_index
    );

    let view_movement = state
        .jam_view
        .scene
        .last_movement
        .as_ref()
        .expect("scene-chain transition projects movement into Jam view");
    assert_eq!(view_movement.kind, expected.kind);
    assert_eq!(view_movement.from_scene.as_deref(), expected.from_scene);
    assert_eq!(view_movement.to_scene, expected.to_scene);
    assert_eq!(view_movement.direction, movement.direction.label());
    assert_eq!(view_movement.tr909_intent, movement.tr909_intent.label());
    assert_eq!(view_movement.mc202_intent, movement.mc202_intent.label());
    assert!(state.jam_view.scene.arrangement_contract.has_landed_movement);
}
