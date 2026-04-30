#[test]
fn mc202_snapshot_payload_restore_hydrates_answer_projection() {
    let graph = sample_graph();
    let base_session = sample_session(&graph);
    let mut committed_state =
        JamAppState::from_parts(base_session.clone(), Some(graph.clone()), ActionQueue::new());

    assert_eq!(
        committed_state.queue_mc202_generate_follower(300),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut committed_state, 1, 400);
    let committed_follower = render_mc202_recipe_buffer(&committed_state.runtime.mc202_render);

    assert_eq!(
        committed_state.queue_mc202_generate_answer(500),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut committed_state, 2, 600);
    let committed_answer = render_mc202_recipe_buffer(&committed_state.runtime.mc202_render);

    let full_action_log = committed_state.session.action_log.clone();
    let committed_plan = riotbox_core::replay::build_committed_replay_plan(&full_action_log)
        .expect("committed MC-202 follower/answer action log builds replay plan");
    assert_eq!(committed_plan.len(), 2);
    let follower_action_id = committed_plan[0].action.id;
    let answer_action_id = committed_plan[1].action.id;
    let follower_action_cursor = full_action_log
        .actions
        .iter()
        .position(|action| action.id == follower_action_id)
        .expect("follower action exists in action log")
        + 1;
    let answer_action_cursor = full_action_log
        .actions
        .iter()
        .position(|action| action.id == answer_action_id)
        .expect("answer action exists in action log")
        + 1;

    let mut anchor_session = base_session;
    anchor_session.action_log = full_action_log.clone();
    let anchor_report =
        riotbox_core::replay::apply_replay_plan_to_session(&mut anchor_session, &committed_plan[..1])
            .expect("MC-202 follower anchor materializes");
    assert_eq!(anchor_report.applied_action_ids, vec![follower_action_id]);

    let snapshot_id = SnapshotId::from("snap-after-mc202-follower");
    let mut restore_session = committed_state.session.clone();
    restore_session.runtime_state = Default::default();
    restore_session.snapshots = vec![Snapshot {
        snapshot_id: snapshot_id.clone(),
        created_at: "2026-04-30T13:20:00Z".into(),
        label: "after MC-202 follower before answer".into(),
        action_cursor: follower_action_cursor,
        payload: Some(riotbox_core::session::SnapshotPayload {
            payload_version: riotbox_core::session::SnapshotPayloadVersion::V1,
            snapshot_id,
            action_cursor: follower_action_cursor,
            runtime_state: anchor_session.runtime_state.clone(),
        }),
    }];

    let mut replayed_state =
        JamAppState::from_parts(restore_session, Some(graph), ActionQueue::new());
    let replay_report = replayed_state
        .apply_restore_target_from_snapshot_payload(answer_action_cursor)
        .expect("snapshot payload restore applies MC-202 answer suffix");
    let replayed_answer = render_mc202_recipe_buffer(&replayed_state.runtime.mc202_render);

    assert_eq!(replay_report.target_action_cursor, answer_action_cursor);
    assert_eq!(
        replay_report.anchor_snapshot_id.as_deref(),
        Some("snap-after-mc202-follower")
    );
    assert_eq!(
        replay_report.anchor_action_cursor,
        Some(follower_action_cursor)
    );
    assert_eq!(replay_report.applied_action_ids, vec![answer_action_id]);
    assert_eq!(
        replayed_state.session.runtime_state.lane_state.mc202,
        committed_state.session.runtime_state.lane_state.mc202
    );
    assert_eq!(
        replayed_state.session.runtime_state.macro_state.mc202_touch,
        committed_state.session.runtime_state.macro_state.mc202_touch
    );
    assert_eq!(replayed_state.runtime.mc202_render, committed_state.runtime.mc202_render);
    assert_recipe_buffers_match(
        "snapshot payload restore MC-202 answer -> committed answer",
        &replayed_answer,
        &committed_answer,
        0.00001,
    );
    assert_recipe_buffers_differ(
        "snapshot payload restore MC-202 follower -> answer",
        &committed_follower,
        &replayed_answer,
        0.005,
    );
}
