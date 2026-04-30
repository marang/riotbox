#[test]
fn w30_snapshot_payload_restore_replays_promote_capture_to_pad_for_resample_artifact() {
    let tempdir = tempdir().expect("create promoted resample replay tempdir");
    let source_path = tempdir.path().join("source.wav");
    let session_path = tempdir.path().join("session.json");
    let graph_path = tempdir.path().join("source_graph.json");
    write_pcm16_wave(&source_path, 48_000, 2, 8.0);

    let mut graph = sample_graph();
    graph.source.path = source_path.to_string_lossy().into_owned();
    graph.source.duration_seconds = 8.0;
    graph.source.sample_rate = 48_000;
    graph.source.channel_count = 2;
    let mut session = sample_session(&graph);
    session.captures.clear();
    session.runtime_state.lane_state.w30.last_capture = None;
    save_source_graph_json(&graph_path, &graph).expect("save replay source graph");
    save_session_json(&session_path, &session).expect("save replay session");

    let mut committed_state =
        JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("load app state");
    committed_state.queue_capture_bar(300);
    commit_w30_replay_step(&mut committed_state, CommitBoundary::Phrase, 0, 1, 0, 400);
    assert!(committed_state.queue_promote_last_capture(410));
    commit_w30_replay_step(&mut committed_state, CommitBoundary::Bar, 4, 2, 0, 500);
    assert_eq!(
        committed_state.queue_w30_internal_resample(650),
        Some(QueueControlResult::Enqueued)
    );
    commit_w30_replay_step(&mut committed_state, CommitBoundary::Phrase, 32, 8, 2, 740);

    let produced_capture_id = CaptureId::from("cap-02");
    let produced_capture = committed_state
        .session
        .captures
        .iter()
        .find(|capture| capture.capture_id == produced_capture_id)
        .expect("resample produced capture metadata");
    assert_eq!(produced_capture.capture_type, CaptureType::Resample);
    assert_eq!(produced_capture.assigned_target, None);
    assert!(
        tempdir.path().join(&produced_capture.storage_path).is_file(),
        "resample artifact should exist before promotion replay"
    );

    let pre_promotion_action_cursor = committed_state.session.action_log.actions.len();
    let pre_promotion_runtime = committed_state.session.runtime_state.clone();
    assert!(committed_state.queue_promote_last_capture(780));
    commit_w30_replay_step(&mut committed_state, CommitBoundary::Bar, 36, 9, 2, 820);

    let promoted_capture = committed_state
        .session
        .captures
        .iter()
        .find(|capture| capture.capture_id == produced_capture_id)
        .expect("promoted resample capture");
    assert_eq!(
        promoted_capture.assigned_target,
        Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        })
    );
    let committed_pad_playback = committed_state
        .runtime
        .w30_preview
        .pad_playback
        .as_ref()
        .expect("committed promoted resample artifact playback");
    let committed_buffer = render_w30_preview_offline(
        &committed_state.runtime.w30_preview,
        48_000,
        2,
        committed_pad_playback.sample_count,
    );

    let replayed_state = run_snapshot_payload_restore_probe_from_anchor_runtime(
        &committed_state,
        graph,
        SnapshotPayloadRestoreSpec {
            plan_label: "committed resample promotion action log builds replay plan",
            snapshot_id: "snap-before-resample-pad-promotion",
            snapshot_label: "before resample pad promotion",
            snapshot_created_at: "2026-04-30T12:25:00Z",
            expected_plan_len: 4,
            anchor_plan_len: 3,
            target_plan_index: 3,
            anchor_label: "W-30 resample anchor materializes before pad promotion",
            restore_expectation: "snapshot payload restore replays resample pad promotion",
        },
        pre_promotion_action_cursor,
        &pre_promotion_runtime,
        |state| {
            state.files = Some(JamFileSet {
                session_path: session_path.clone(),
                source_graph_path: Some(graph_path.clone()),
            });
            state.refresh_capture_audio_cache();
        },
    );
    let replayed_pad_playback = replayed_state
        .runtime
        .w30_preview
        .pad_playback
        .as_ref()
        .expect("replayed promoted resample artifact playback");
    let replayed_buffer = render_w30_preview_offline(
        &replayed_state.runtime.w30_preview,
        48_000,
        2,
        replayed_pad_playback.sample_count,
    );

    assert_eq!(
        replayed_state.session.runtime_state.lane_state.w30,
        committed_state.session.runtime_state.lane_state.w30
    );
    assert_eq!(
        replayed_state.runtime.w30_preview.capture_id,
        committed_state.runtime.w30_preview.capture_id
    );
    assert_recipe_buffers_match(
        "snapshot payload restore promote.capture_to_pad resample artifact -> committed promotion",
        &replayed_buffer,
        &committed_buffer,
        0.0015,
    );

    let mut fallback_preview = replayed_state.runtime.w30_preview.clone();
    fallback_preview.source_window_preview = None;
    fallback_preview.pad_playback = None;
    let fallback_buffer = render_w30_preview_offline(
        &fallback_preview,
        48_000,
        2,
        replayed_pad_playback.sample_count,
    );
    assert_w30_replay_buffers_differ(
        "promoted resample artifact playback -> fallback oscillator",
        &replayed_buffer,
        &fallback_buffer,
        0.0005,
        0.001,
    );
}
