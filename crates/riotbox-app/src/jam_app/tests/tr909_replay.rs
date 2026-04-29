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
