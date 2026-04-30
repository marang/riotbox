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
    let follower_action_cursor = action_cursor_for(&full_action_log, follower_action_id, "follower");
    let answer_action_cursor = action_cursor_for(&full_action_log, answer_action_id, "answer");

    let anchor_session = materialize_replay_anchor_session(
        base_session,
        full_action_log.clone(),
        &committed_plan[..1],
        vec![follower_action_id],
        "MC-202 follower anchor materializes",
    );

    let mut restore_session = committed_state.session.clone();
    restore_session.runtime_state = Default::default();
    restore_session.snapshots = vec![snapshot_payload_for_anchor(
        "snap-after-mc202-follower",
        "after MC-202 follower before answer",
        "2026-04-30T13:20:00Z",
        follower_action_cursor,
        &anchor_session.runtime_state,
    )];

    let mut replayed_state =
        JamAppState::from_parts(restore_session, Some(graph), ActionQueue::new());
    let replay_report = replayed_state
        .apply_restore_target_from_snapshot_payload(answer_action_cursor)
        .expect("snapshot payload restore applies MC-202 answer suffix");
    let replayed_answer = render_mc202_recipe_buffer(&replayed_state.runtime.mc202_render);

    assert_restore_report_identity(
        &replay_report,
        answer_action_cursor,
        "snap-after-mc202-follower",
        follower_action_cursor,
        vec![answer_action_id],
    );
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
