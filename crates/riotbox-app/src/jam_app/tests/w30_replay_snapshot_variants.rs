#[test]
fn w30_snapshot_payload_restore_hydrates_damage_profile_preview_output() {
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
        committed_state.queue_w30_apply_damage_profile(500),
        Some(QueueControlResult::Enqueued)
    );
    commit_w30_replay_step(&mut committed_state, CommitBoundary::Bar, 36, 10, 2, 600);
    let committed_damage = render_w30_replay_buffer(&committed_state);

    let full_action_log = committed_state.session.action_log.clone();
    let committed_plan = riotbox_core::replay::build_committed_replay_plan(&full_action_log)
        .expect("committed W-30 damage action log builds replay plan");
    assert_eq!(committed_plan.len(), 2);
    let browse_action_id = committed_plan[0].action.id;
    let damage_action_id = committed_plan[1].action.id;
    let browse_action_cursor = action_cursor_for(&full_action_log, browse_action_id, "browse");
    let damage_action_cursor = action_cursor_for(&full_action_log, damage_action_id, "damage");

    let anchor_session = materialize_replay_anchor_session(
        replay_base_session,
        full_action_log.clone(),
        &committed_plan[..1],
        vec![browse_action_id],
        "W-30 browse anchor materializes before damage",
    );

    let mut restore_session = committed_state.session.clone();
    restore_session.runtime_state = Default::default();
    restore_session.snapshots = vec![snapshot_payload_for_anchor(
        "snap-after-w30-browse",
        "after W-30 browse before damage",
        "2026-04-30T13:00:00Z",
        browse_action_cursor,
        &anchor_session.runtime_state,
    )];

    let mut replayed_state =
        JamAppState::from_parts(restore_session, Some(graph), ActionQueue::new());
    replayed_state.source_audio_cache = Some(source_audio_cache);
    let replay_report = replayed_state
        .apply_restore_target_from_snapshot_payload(damage_action_cursor)
        .expect("snapshot payload restore applies W-30 damage suffix");
    let replayed_damage = render_w30_replay_buffer(&replayed_state);

    assert_restore_report_identity(
        &replay_report,
        damage_action_cursor,
        "snap-after-w30-browse",
        browse_action_cursor,
        vec![damage_action_id],
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
        "snapshot payload restore W-30 damage -> committed damage",
        &replayed_damage,
        &committed_damage,
        0.00001,
    );
    assert_w30_replay_buffers_differ(
        "snapshot payload restore W-30 browse -> damage",
        &committed_browse,
        &replayed_damage,
        0.0001,
        0.0001,
    );
}

#[test]
fn w30_snapshot_payload_restore_hydrates_promoted_audition_preview_output() {
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
        committed_state.queue_w30_audition(500),
        Some(QueueControlResult::Enqueued)
    );
    commit_w30_replay_step(&mut committed_state, CommitBoundary::Bar, 36, 10, 2, 600);
    let committed_audition = render_w30_replay_buffer(&committed_state);

    let replayed_state = run_snapshot_payload_restore_probe(
        replay_base_session,
        &committed_state,
        graph,
        SnapshotPayloadRestoreSpec {
            plan_label: "committed W-30 promoted audition action log builds replay plan",
            snapshot_id: "snap-before-w30-audition",
            snapshot_label: "before W-30 promoted audition",
            snapshot_created_at: "2026-04-30T16:55:00Z",
            expected_plan_len: 2,
            anchor_plan_len: 1,
            target_plan_index: 1,
            anchor_label: "W-30 browse anchor materializes before promoted audition",
            restore_expectation: "snapshot payload restore applies W-30 promoted audition suffix",
        },
        |state| {
            state.source_audio_cache = Some(source_audio_cache);
        },
    );
    assert_eq!(
        replayed_state.session.runtime_state.lane_state.w30,
        committed_state.session.runtime_state.lane_state.w30
    );
    assert_eq!(
        replayed_state.runtime.w30_preview.capture_id,
        committed_state.runtime.w30_preview.capture_id
    );
    assert_eq!(
        replayed_state.runtime.w30_preview.mode,
        W30PreviewRenderMode::PromotedAudition
    );
    assert_eq!(
        replayed_state.runtime.w30_preview.source_profile,
        Some(W30PreviewSourceProfile::PromotedAudition)
    );
    assert!(replayed_state.runtime.w30_preview.source_window_preview.is_some());
    assert!(committed_state.runtime.w30_preview.source_window_preview.is_some());
    let replayed_audition = render_w30_preview_buffer(&replayed_state);
    let replayed_metrics = signal_metrics(&replayed_audition);
    assert!(
        replayed_metrics.active_samples > 100
            && replayed_metrics.peak_abs > 0.001
            && replayed_metrics.rms > 0.0001,
        "restored W-30 audition render too close to silence: active {}, peak {}, rms {}, state {:?}",
        replayed_metrics.active_samples,
        replayed_metrics.peak_abs,
        replayed_metrics.rms,
        replayed_state.runtime.w30_preview
    );
    assert_recipe_buffers_match(
        "snapshot payload restore W-30 promoted audition -> committed audition",
        &replayed_audition,
        &committed_audition,
        0.00001,
    );
    assert_w30_replay_buffers_differ(
        "snapshot payload restore W-30 browse -> promoted audition",
        &committed_browse,
        &replayed_audition,
        0.0001,
        0.0001,
    );
}
