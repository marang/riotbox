#[test]
fn stage_style_snapshot_payload_restore_converges_supported_multi_lane_suffix() {
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
    session.runtime_state.lane_state.tr909.pattern_ref =
        Some("stage-style-snapshot-support".into());
    session.runtime_state.lane_state.mc202.role = Some("follower".into());
    session.runtime_state.lane_state.mc202.phrase_variant = None;

    let base_session = session.clone();
    let mut committed_state =
        JamAppState::from_parts(session, Some(graph.clone()), ActionQueue::new());
    let before_stage_run = render_scene_recipe_mix_buffer(&committed_state);

    assert_eq!(
        committed_state.queue_scene_select(300),
        QueueControlResult::Enqueued
    );
    commit_stage_style_step(
        &mut committed_state,
        CommitBoundary::Bar,
        36,
        9,
        2,
        "scene-01-drop",
        360,
    );

    assert_eq!(
        committed_state.queue_mc202_generate_answer(500),
        QueueControlResult::Enqueued
    );
    commit_stage_style_step(
        &mut committed_state,
        CommitBoundary::Phrase,
        48,
        12,
        3,
        "scene-02-break",
        560,
    );
    let anchor_mix = render_scene_recipe_mix_buffer(&committed_state);

    committed_state.queue_tr909_fill(700);
    commit_stage_style_step(
        &mut committed_state,
        CommitBoundary::Bar,
        52,
        13,
        3,
        "scene-02-break",
        760,
    );

    assert!(committed_state.queue_tr909_slam_toggle(900));
    commit_stage_style_step(
        &mut committed_state,
        CommitBoundary::Beat,
        53,
        13,
        3,
        "scene-02-break",
        960,
    );

    assert_eq!(
        committed_state.queue_scene_restore(1_100),
        QueueControlResult::Enqueued
    );
    commit_stage_style_step(
        &mut committed_state,
        CommitBoundary::Bar,
        56,
        14,
        3,
        "scene-02-break",
        1_160,
    );
    let committed_final_mix = render_scene_recipe_mix_buffer(&committed_state);

    let replayed_state = run_graph_aware_snapshot_payload_restore_probe(
        base_session,
        &committed_state,
        graph,
        SnapshotPayloadRestoreSpec {
            plan_label: "stage-style supported action log builds replay plan",
            snapshot_id: "snap-stage-style-after-scene-answer",
            snapshot_label: "after scene launch and MC-202 answer",
            snapshot_created_at: "2026-04-30T18:20:00Z",
            expected_plan_len: 5,
            anchor_plan_len: 2,
            target_plan_index: 4,
            anchor_label: "Stage-style Scene/MC-202 anchor materializes",
            restore_expectation: "snapshot payload restore applies TR-909 and Scene suffix",
        },
        |_| {},
    );
    let replayed_final_mix = render_scene_recipe_mix_buffer(&replayed_state);

    let convergence = riotbox_core::replay::build_latest_snapshot_replay_convergence_summary(
        &replayed_state.session.action_log,
        &replayed_state.session.snapshots,
    )
    .expect("stage-style latest snapshot convergence summary builds");
    assert_eq!(convergence.target_action_cursor, 6);
    assert_eq!(convergence.origin_action_count, 6);
    assert_eq!(convergence.origin_replay_entry_count, 5);
    assert_eq!(convergence.snapshot_count, 1);
    assert_eq!(
        convergence.anchor_snapshot_id.as_deref(),
        Some("snap-stage-style-after-scene-answer")
    );
    assert_eq!(convergence.anchor_action_cursor, Some(3));
    assert_eq!(
        convergence.anchor_payload_readiness,
        riotbox_core::replay::SnapshotPayloadReadiness::Ready
    );
    assert_eq!(convergence.suffix_action_count, 3);
    assert!(convergence.needs_replay);
    assert!(!convergence.needs_full_replay);
    assert_eq!(convergence.origin_unsupported_action_count, 0);
    assert_eq!(convergence.suffix_unsupported_action_count, 0);
    assert_eq!(
        convergence.suffix_commands,
        vec![
            ActionCommand::Tr909FillNext,
            ActionCommand::Tr909SetSlam,
            ActionCommand::SceneRestore,
        ]
    );

    assert_eq!(
        replayed_state.session.runtime_state.scene_state,
        committed_state.session.runtime_state.scene_state
    );
    assert_eq!(
        replayed_state.session.runtime_state.lane_state.mc202,
        committed_state.session.runtime_state.lane_state.mc202
    );
    assert_eq!(
        replayed_state.session.runtime_state.lane_state.tr909,
        committed_state.session.runtime_state.lane_state.tr909
    );
    assert_eq!(
        replayed_state.session.runtime_state.macro_state,
        committed_state.session.runtime_state.macro_state
    );
    assert_eq!(
        replayed_state.session.runtime_state.transport.current_scene,
        committed_state.session.runtime_state.transport.current_scene
    );
    assert_eq!(
        replayed_state.runtime.mc202_render,
        committed_state.runtime.mc202_render
    );
    assert_eq!(
        replayed_state.runtime.tr909_render,
        committed_state.runtime.tr909_render
    );
    assert_recipe_buffers_match(
        "stage-style snapshot restore final mix -> committed final mix",
        &replayed_final_mix,
        &committed_final_mix,
        0.00001,
    );
    assert_recipe_buffers_differ(
        "stage-style run leaves initial mixed output",
        &before_stage_run,
        &replayed_final_mix,
        0.003,
    );
    assert_recipe_buffers_differ(
        "stage-style suffix changes the snapshot anchor mix",
        &anchor_mix,
        &replayed_final_mix,
        0.003,
    );
}

fn commit_stage_style_step(
    state: &mut JamAppState,
    kind: CommitBoundary,
    beat_index: u64,
    bar_index: u64,
    phrase_index: u64,
    scene_id: &str,
    committed_at: u64,
) {
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind,
            beat_index,
            bar_index,
            phrase_index,
            scene_id: Some(SceneId::from(scene_id)),
        },
        committed_at,
    );
    assert_eq!(committed.len(), 1);
}
