#[test]
fn tr909_replay_executor_matches_committed_app_state_and_audio_path() {
    let graph = sample_graph();
    let base_session = sample_session(&graph);
    let mut committed_state =
        JamAppState::from_parts(base_session.clone(), Some(graph.clone()), ActionQueue::new());

    committed_state.queue_tr909_fill(300);
    commit_tr909_replay_step(&mut committed_state, CommitBoundary::Bar, 8, 2, 0, 400);
    let fill = render_tr909_replay_buffer(&committed_state);

    assert!(committed_state.queue_tr909_slam_toggle(500));
    commit_tr909_replay_step(&mut committed_state, CommitBoundary::Beat, 9, 2, 0, 600);
    let slammed_fill = render_tr909_replay_buffer(&committed_state);

    assert_eq!(
        committed_state.queue_tr909_takeover(700),
        QueueControlResult::Enqueued
    );
    commit_tr909_replay_step(&mut committed_state, CommitBoundary::Phrase, 16, 4, 1, 800);
    let takeover = render_tr909_replay_buffer(&committed_state);

    assert_eq!(
        committed_state.queue_tr909_release(900),
        QueueControlResult::Enqueued
    );
    commit_tr909_replay_step(
        &mut committed_state,
        CommitBoundary::Phrase,
        32,
        8,
        2,
        1_000,
    );
    let committed_release = render_tr909_replay_buffer(&committed_state);

    let plan = riotbox_core::replay::build_committed_replay_plan(
        &committed_state.session.action_log,
    )
    .expect("committed TR-909 action log builds replay plan");
    let mut replayed_session = base_session;
    let report = riotbox_core::replay::apply_replay_plan_to_session(&mut replayed_session, &plan)
        .expect("TR-909 replay executor applies support family");
    let replayed_state = JamAppState::from_parts(replayed_session, Some(graph), ActionQueue::new());
    let replayed_release = render_tr909_replay_buffer(&replayed_state);

    assert_eq!(report.applied_action_ids.len(), 4);
    assert_eq!(
        replayed_state.session.runtime_state.lane_state.tr909,
        committed_state.session.runtime_state.lane_state.tr909
    );
    assert_eq!(
        replayed_state.session.runtime_state.macro_state.tr909_slam,
        committed_state.session.runtime_state.macro_state.tr909_slam
    );
    assert_eq!(replayed_state.runtime.tr909_render, committed_state.runtime.tr909_render);
    assert_recipe_buffers_match(
        "replayed TR-909 release -> committed TR-909 release",
        &replayed_release,
        &committed_release,
        0.00001,
    );
    assert_recipe_buffers_differ("fill -> slammed fill", &fill, &slammed_fill, 0.0002);
    assert_recipe_buffers_differ("slammed fill -> takeover", &slammed_fill, &takeover, 0.001);
    assert_recipe_buffers_differ(
        "takeover -> replayed release",
        &takeover,
        &replayed_release,
        0.001,
    );
}

#[test]
fn tr909_target_suffix_replay_helper_matches_committed_app_projection() {
    let graph = sample_graph();
    let base_session = sample_session(&graph);
    let mut committed_state =
        JamAppState::from_parts(base_session.clone(), Some(graph.clone()), ActionQueue::new());

    committed_state.queue_tr909_fill(300);
    commit_tr909_replay_step(&mut committed_state, CommitBoundary::Bar, 8, 2, 0, 400);
    let committed_fill = render_tr909_replay_buffer(&committed_state);

    assert!(committed_state.queue_tr909_slam_toggle(500));
    commit_tr909_replay_step(&mut committed_state, CommitBoundary::Beat, 9, 2, 0, 600);
    let committed_slam = render_tr909_replay_buffer(&committed_state);

    let full_action_log = committed_state.session.action_log.clone();
    let committed_plan = riotbox_core::replay::build_committed_replay_plan(&full_action_log)
        .expect("committed TR-909 action log builds replay plan");
    assert_eq!(committed_plan.len(), 2);
    let fill_action_id = committed_plan[0].action.id;
    let slam_action_id = committed_plan[1].action.id;
    let fill_action_cursor = full_action_log
        .actions
        .iter()
        .position(|action| action.id == fill_action_id)
        .expect("fill action exists in action log")
        + 1;
    let slam_action_cursor = full_action_log
        .actions
        .iter()
        .position(|action| action.id == slam_action_id)
        .expect("slam action exists in action log")
        + 1;
    let mut hydrated_anchor_session = base_session;
    hydrated_anchor_session.action_log = full_action_log.clone();
    hydrated_anchor_session.snapshots = vec![Snapshot {
        snapshot_id: SnapshotId::from("snap-after-fill"),
        created_at: "2026-04-29T22:30:00Z".into(),
        label: "after fill".into(),
        action_cursor: fill_action_cursor,
    }];
    let anchor_report = riotbox_core::replay::apply_replay_plan_to_session(
        &mut hydrated_anchor_session,
        &committed_plan[..1],
    )
    .expect("fill anchor materializes");
    assert_eq!(anchor_report.applied_action_ids, vec![fill_action_id]);

    let suffix_report = riotbox_core::replay::apply_replay_target_suffix_to_session(
        &mut hydrated_anchor_session,
        slam_action_cursor,
        None,
    )
    .expect("target replay suffix applies slam");
    let replayed_state =
        JamAppState::from_parts(hydrated_anchor_session, Some(graph), ActionQueue::new());
    let replayed_slam = render_tr909_replay_buffer(&replayed_state);

    assert_eq!(suffix_report.target_action_cursor, slam_action_cursor);
    assert_eq!(
        suffix_report.anchor_snapshot_id.as_deref(),
        Some("snap-after-fill")
    );
    assert_eq!(suffix_report.anchor_action_cursor, Some(fill_action_cursor));
    assert_eq!(suffix_report.applied_action_ids, vec![slam_action_id]);
    assert_eq!(
        replayed_state.session.runtime_state.lane_state.tr909,
        committed_state.session.runtime_state.lane_state.tr909
    );
    assert_eq!(replayed_state.runtime.tr909_render, committed_state.runtime.tr909_render);
    assert_recipe_buffers_match(
        "target suffix replay TR-909 slam -> committed slam",
        &replayed_slam,
        &committed_slam,
        0.00001,
    );
    assert_recipe_buffers_differ(
        "target suffix replay fill -> slam",
        &committed_fill,
        &replayed_slam,
        0.0002,
    );
}

fn commit_tr909_replay_step(
    state: &mut JamAppState,
    kind: CommitBoundary,
    beat_index: u64,
    bar_index: u64,
    phrase_index: u64,
    committed_at: u64,
) {
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind,
            beat_index,
            bar_index,
            phrase_index,
            scene_id: Some(SceneId::from("scene-1")),
        },
        committed_at,
    );
    assert_eq!(committed.len(), 1);
}

fn render_tr909_replay_buffer(state: &JamAppState) -> Vec<f32> {
    let buffer = render_tr909_offline(&state.runtime.tr909_render, 44_100, 2, 44_100);
    let metrics = signal_metrics(&buffer);
    assert!(
        metrics.active_samples > 100 && metrics.peak_abs > 0.001 && metrics.rms > 0.0001,
        "TR-909 replay render too close to silence: active {}, peak {}, rms {}",
        metrics.active_samples,
        metrics.peak_abs,
        metrics.rms
    );
    buffer
}
