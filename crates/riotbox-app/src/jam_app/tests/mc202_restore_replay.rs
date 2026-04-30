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

    let replayed_state = run_snapshot_payload_restore_probe(
        base_session,
        &committed_state,
        graph,
        SnapshotPayloadRestoreSpec {
            plan_label: "committed MC-202 follower/answer action log builds replay plan",
            snapshot_id: "snap-after-mc202-follower",
            snapshot_label: "after MC-202 follower before answer",
            snapshot_created_at: "2026-04-30T13:20:00Z",
            expected_plan_len: 2,
            anchor_plan_len: 1,
            target_plan_index: 1,
            anchor_label: "MC-202 follower anchor materializes",
            restore_expectation: "snapshot payload restore applies MC-202 answer suffix",
        },
        |_| {},
    );
    let replayed_answer = render_mc202_recipe_buffer(&replayed_state.runtime.mc202_render);

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

#[test]
fn mc202_snapshot_payload_restore_hydrates_pressure_projection() {
    let graph = sample_graph();
    let base_session = sample_session(&graph);
    let mut committed_state =
        JamAppState::from_parts(base_session.clone(), Some(graph.clone()), ActionQueue::new());

    assert_eq!(
        committed_state.queue_mc202_generate_follower(300),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut committed_state, 1, 400);

    assert_eq!(
        committed_state.queue_mc202_generate_answer(500),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut committed_state, 2, 600);
    let committed_answer = render_mc202_recipe_buffer(&committed_state.runtime.mc202_render);

    assert_eq!(
        committed_state.queue_mc202_generate_pressure(700),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut committed_state, 3, 800);
    let committed_pressure = render_mc202_recipe_buffer(&committed_state.runtime.mc202_render);

    let replayed_state = run_snapshot_payload_restore_probe(
        base_session,
        &committed_state,
        graph,
        SnapshotPayloadRestoreSpec {
            plan_label: "committed MC-202 pressure action log builds replay plan",
            snapshot_id: "snap-after-mc202-answer",
            snapshot_label: "after MC-202 answer before pressure",
            snapshot_created_at: "2026-04-30T15:50:00Z",
            expected_plan_len: 3,
            anchor_plan_len: 2,
            target_plan_index: 2,
            anchor_label: "MC-202 answer anchor materializes before pressure",
            restore_expectation: "snapshot payload restore applies MC-202 pressure suffix",
        },
        |_| {},
    );
    let replayed_pressure = render_mc202_recipe_buffer(&replayed_state.runtime.mc202_render);

    assert_eq!(
        replayed_state.session.runtime_state.lane_state.mc202,
        committed_state.session.runtime_state.lane_state.mc202
    );
    assert_eq!(
        replayed_state.session.runtime_state.macro_state.mc202_touch,
        committed_state.session.runtime_state.macro_state.mc202_touch
    );
    assert_eq!(
        replayed_state.runtime.mc202_render,
        committed_state.runtime.mc202_render
    );
    assert_recipe_buffers_match(
        "snapshot payload restore MC-202 pressure -> committed pressure",
        &replayed_pressure,
        &committed_pressure,
        0.00001,
    );
    assert_recipe_buffers_differ(
        "snapshot payload restore MC-202 answer -> pressure",
        &committed_answer,
        &replayed_pressure,
        0.004,
    );
}

#[test]
fn mc202_snapshot_payload_restore_hydrates_instigator_projection() {
    let graph = sample_graph();
    let base_session = sample_session(&graph);
    let mut committed_state =
        JamAppState::from_parts(base_session.clone(), Some(graph.clone()), ActionQueue::new());

    assert_eq!(
        committed_state.queue_mc202_generate_follower(300),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut committed_state, 1, 400);

    assert_eq!(
        committed_state.queue_mc202_generate_answer(500),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut committed_state, 2, 600);

    assert_eq!(
        committed_state.queue_mc202_generate_pressure(700),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut committed_state, 3, 800);
    let committed_pressure = render_mc202_recipe_buffer(&committed_state.runtime.mc202_render);

    assert_eq!(
        committed_state.queue_mc202_generate_instigator(900),
        QueueControlResult::Enqueued
    );
    commit_mc202_recipe_step(&mut committed_state, 4, 1_000);
    let committed_instigator = render_mc202_recipe_buffer(&committed_state.runtime.mc202_render);

    let replayed_state = run_snapshot_payload_restore_probe(
        base_session,
        &committed_state,
        graph,
        SnapshotPayloadRestoreSpec {
            plan_label: "committed MC-202 instigator action log builds replay plan",
            snapshot_id: "snap-after-mc202-pressure",
            snapshot_label: "after MC-202 pressure before instigator",
            snapshot_created_at: "2026-04-30T16:00:00Z",
            expected_plan_len: 4,
            anchor_plan_len: 3,
            target_plan_index: 3,
            anchor_label: "MC-202 pressure anchor materializes before instigator",
            restore_expectation: "snapshot payload restore applies MC-202 instigator suffix",
        },
        |_| {},
    );
    let replayed_instigator = render_mc202_recipe_buffer(&replayed_state.runtime.mc202_render);

    assert_eq!(
        replayed_state.session.runtime_state.lane_state.mc202,
        committed_state.session.runtime_state.lane_state.mc202
    );
    assert_eq!(
        replayed_state.session.runtime_state.macro_state.mc202_touch,
        committed_state.session.runtime_state.macro_state.mc202_touch
    );
    assert_eq!(
        replayed_state.runtime.mc202_render,
        committed_state.runtime.mc202_render
    );
    assert_recipe_buffers_match(
        "snapshot payload restore MC-202 instigator -> committed instigator",
        &replayed_instigator,
        &committed_instigator,
        0.00001,
    );
    assert_recipe_buffers_differ(
        "snapshot payload restore MC-202 pressure -> instigator",
        &committed_pressure,
        &replayed_instigator,
        0.004,
    );
}
