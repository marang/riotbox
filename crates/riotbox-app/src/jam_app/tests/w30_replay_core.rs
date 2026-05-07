#[test]
fn w30_replay_executor_matches_committed_app_state_and_preview_output() {
    let (_tempdir, graph, source_audio_cache, mut committed_state) =
        w30_source_backed_replay_state();
    let replay_base_session = committed_state.session.clone();
    let initial_source_window = committed_state
        .runtime
        .w30_preview
        .source_window_preview
        .as_ref()
        .expect("initial source-window preview")
        .clone();
    let initial_preview = render_w30_replay_buffer(&committed_state);

    assert_eq!(
        committed_state.queue_w30_browse_slice_pool(300),
        Some(QueueControlResult::Enqueued)
    );
    commit_w30_replay_step(&mut committed_state, CommitBoundary::Beat, 33, 9, 2, 400);
    let committed_browse_preview = committed_state
        .runtime
        .w30_preview
        .source_window_preview
        .as_ref()
        .expect("committed browse source-window preview")
        .clone();
    let committed_browse = render_w30_replay_buffer(&committed_state);

    let plan =
        riotbox_core::replay::build_committed_replay_plan(&committed_state.session.action_log)
            .expect("committed W-30 action log builds replay plan");
    let mut replayed_session = replay_base_session;
    replayed_session.action_log = committed_state.session.action_log.clone();
    let report = riotbox_core::replay::apply_replay_plan_to_session(&mut replayed_session, &plan)
        .expect("W-30 replay executor applies cue subset");
    let mut replayed_state =
        JamAppState::from_parts(replayed_session, Some(graph), ActionQueue::new());
    replayed_state.source_audio_cache = Some(source_audio_cache);
    replayed_state.refresh_view();
    let replayed_browse = render_w30_replay_buffer(&replayed_state);

    assert_eq!(report.applied_action_ids.len(), 1);
    assert_eq!(
        replayed_state.session.runtime_state.lane_state.w30,
        committed_state.session.runtime_state.lane_state.w30
    );
    assert_eq!(
        replayed_state.session.runtime_state.macro_state.w30_grit,
        committed_state.session.runtime_state.macro_state.w30_grit
    );
    assert_eq!(
        replayed_state.runtime.w30_preview,
        committed_state.runtime.w30_preview
    );
    assert_recipe_buffers_match(
        "replayed W-30 browse -> committed W-30 browse",
        &replayed_browse,
        &committed_browse,
        0.00001,
    );
    assert_w30_replay_buffers_differ(
        "initial W-30 recall -> browse",
        &initial_preview,
        &committed_browse,
        0.0003,
        0.0008,
    );
    assert!(
        recipe_signal_delta_rms(
            &initial_source_window.samples,
            &committed_browse_preview.samples,
        ) > 0.01,
        "W-30 browse should change the source-window preview samples"
    );
}

#[test]
fn w30_target_suffix_replay_helper_matches_committed_app_preview_output() {
    let (_tempdir, graph, source_audio_cache, mut committed_state) =
        w30_source_backed_replay_state();
    let replay_base_session = committed_state.session.clone();

    assert_eq!(
        committed_state.queue_w30_browse_slice_pool(300),
        Some(QueueControlResult::Enqueued)
    );
    commit_w30_replay_step(&mut committed_state, CommitBoundary::Beat, 33, 9, 2, 400);
    let committed_browse = render_w30_replay_buffer(&committed_state);

    assert_eq!(
        committed_state.queue_w30_trigger_pad(500),
        Some(QueueControlResult::Enqueued)
    );
    commit_w30_replay_step(&mut committed_state, CommitBoundary::Beat, 34, 9, 2, 600);
    let committed_trigger = render_w30_replay_buffer(&committed_state);

    let full_action_log = committed_state.session.action_log.clone();
    let committed_plan = riotbox_core::replay::build_committed_replay_plan(&full_action_log)
        .expect("committed W-30 action log builds replay plan");
    assert_eq!(committed_plan.len(), 2);
    let browse_action_id = committed_plan[0].action.id;
    let trigger_action_id = committed_plan[1].action.id;
    let browse_action_cursor = full_action_log
        .actions
        .iter()
        .position(|action| action.id == browse_action_id)
        .expect("browse action exists in action log")
        + 1;
    let trigger_action_cursor = full_action_log
        .actions
        .iter()
        .position(|action| action.id == trigger_action_id)
        .expect("trigger action exists in action log")
        + 1;

    let mut hydrated_anchor_session = replay_base_session;
    hydrated_anchor_session.action_log = full_action_log.clone();
    hydrated_anchor_session.snapshots = vec![Snapshot {
        snapshot_id: SnapshotId::from("snap-after-w30-browse"),
        created_at: "2026-04-29T22:45:00Z".into(),
        label: "after W-30 browse".into(),
        action_cursor: browse_action_cursor,
        payload: None,
    }];
    let anchor_report = riotbox_core::replay::apply_replay_plan_to_session(
        &mut hydrated_anchor_session,
        &committed_plan[..1],
    )
    .expect("W-30 browse anchor materializes");
    assert_eq!(anchor_report.applied_action_ids, vec![browse_action_id]);

    let mut replayed_state =
        JamAppState::from_parts(hydrated_anchor_session, Some(graph), ActionQueue::new());
    replayed_state.source_audio_cache = Some(source_audio_cache);
    let suffix_report = replayed_state
        .apply_restore_target_suffix(trigger_action_cursor)
        .expect("target replay suffix applies W-30 trigger");
    let replayed_trigger = render_w30_replay_buffer(&replayed_state);

    assert_eq!(suffix_report.target_action_cursor, trigger_action_cursor);
    assert_eq!(
        suffix_report.anchor_snapshot_id.as_deref(),
        Some("snap-after-w30-browse")
    );
    assert_eq!(
        suffix_report.anchor_action_cursor,
        Some(browse_action_cursor)
    );
    assert_eq!(suffix_report.applied_action_ids, vec![trigger_action_id]);
    assert_eq!(
        replayed_state.session.runtime_state.lane_state.w30,
        committed_state.session.runtime_state.lane_state.w30
    );
    assert_eq!(
        replayed_state.runtime.w30_preview,
        committed_state.runtime.w30_preview
    );
    assert_recipe_buffers_match(
        "target suffix replay W-30 trigger -> committed trigger",
        &replayed_trigger,
        &committed_trigger,
        0.00001,
    );
    assert_w30_replay_buffers_differ(
        "target suffix replay W-30 browse -> trigger",
        &committed_browse,
        &replayed_trigger,
        0.0001,
        0.0001,
    );
}

#[test]
fn w30_snapshot_payload_restore_runner_matches_committed_app_preview_output() {
    let (_tempdir, graph, source_audio_cache, mut committed_state) =
        w30_source_backed_replay_state();
    let replay_base_session = committed_state.session.clone();

    assert_eq!(
        committed_state.queue_w30_browse_slice_pool(300),
        Some(QueueControlResult::Enqueued)
    );
    commit_w30_replay_step(&mut committed_state, CommitBoundary::Beat, 33, 9, 2, 400);
    let committed_browse = render_w30_replay_buffer(&committed_state);

    assert_eq!(
        committed_state.queue_w30_trigger_pad(500),
        Some(QueueControlResult::Enqueued)
    );
    commit_w30_replay_step(&mut committed_state, CommitBoundary::Beat, 34, 9, 2, 600);
    let committed_trigger = render_w30_replay_buffer(&committed_state);

    let replayed_state = run_snapshot_payload_restore_probe(
        replay_base_session,
        &committed_state,
        graph,
        SnapshotPayloadRestoreSpec {
            plan_label: "committed W-30 action log builds replay plan",
            snapshot_id: "snap-after-w30-browse",
            snapshot_label: "after W-30 browse",
            snapshot_created_at: "2026-04-30T08:25:00Z",
            expected_plan_len: 2,
            anchor_plan_len: 1,
            target_plan_index: 1,
            anchor_label: "W-30 browse anchor materializes",
            restore_expectation: "snapshot payload restore applies W-30 trigger suffix",
        },
        |state| {
            state.source_audio_cache = Some(source_audio_cache);
        },
    );
    let replayed_trigger = render_w30_replay_buffer(&replayed_state);

    assert_eq!(
        replayed_state.session.runtime_state.lane_state.w30,
        committed_state.session.runtime_state.lane_state.w30
    );
    assert_eq!(
        replayed_state.runtime.w30_preview,
        committed_state.runtime.w30_preview
    );
    assert_recipe_buffers_match(
        "snapshot payload restore W-30 trigger -> committed trigger",
        &replayed_trigger,
        &committed_trigger,
        0.00001,
    );
    assert_w30_replay_buffers_differ(
        "snapshot payload restore W-30 browse -> trigger",
        &committed_browse,
        &replayed_trigger,
        0.0001,
        0.0001,
    );
}
