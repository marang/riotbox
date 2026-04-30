#[test]
fn w30_snapshot_payload_restore_hydrates_loop_freeze_artifact_preview_output() {
    let (tempdir, _graph, _source_audio_cache, mut committed_state) =
        w30_source_backed_replay_state();
    let session_path = tempdir.path().join("session.json");
    committed_state.files = Some(JamFileSet {
        session_path: session_path.clone(),
        source_graph_path: None,
    });
    let source_capture_id = CaptureId::from("cap-02");
    let source_capture = committed_state
        .session
        .captures
        .iter_mut()
        .find(|capture| capture.capture_id == source_capture_id)
        .expect("source capture for loop freeze");
    source_capture.assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-a"),
        pad_id: PadId::from("pad-01"),
    });
    committed_state
        .session
        .runtime_state
        .lane_state
        .w30
        .active_bank = Some(BankId::from("bank-a"));
    committed_state
        .session
        .runtime_state
        .lane_state
        .w30
        .focused_pad = Some(PadId::from("pad-01"));
    committed_state
        .session
        .runtime_state
        .lane_state
        .w30
        .last_capture = Some(source_capture_id.clone());
    committed_state.refresh_view();
    let pre_freeze_action_cursor = committed_state.session.action_log.actions.len();
    let pre_freeze_runtime = committed_state.session.runtime_state.clone();

    assert_eq!(
        committed_state.queue_w30_loop_freeze(700),
        Some(QueueControlResult::Enqueued)
    );
    commit_w30_replay_step(
        &mut committed_state,
        CommitBoundary::Phrase,
        48,
        12,
        3,
        800,
    );
    let produced_capture_id = committed_state
        .session
        .captures
        .last()
        .expect("loop-freeze produced capture")
        .capture_id
        .clone();
    assert!(
        committed_state
            .capture_audio_cache
            .contains_key(&produced_capture_id),
        "loop freeze commit should write and cache the produced capture artifact"
    );
    assert_eq!(
        committed_state.session.runtime_state.lane_state.w30.last_capture,
        Some(produced_capture_id.clone())
    );
    let produced_capture = committed_state
        .session
        .captures
        .iter()
        .find(|capture| capture.capture_id == produced_capture_id)
        .expect("loop-freeze produced capture metadata");
    let produced_artifact_path = tempdir.path().join(&produced_capture.storage_path);
    assert!(
        produced_artifact_path.is_file(),
        "loop-freeze commit should persist a reloadable WAV artifact at {}",
        produced_artifact_path.display()
    );
    let committed_pad_playback = committed_state
        .runtime
        .w30_preview
        .pad_playback
        .as_ref()
        .expect("committed loop-freeze artifact playback");
    assert!(
        committed_pad_playback.sample_count > W30_PREVIEW_SAMPLE_WINDOW_LEN,
        "loop-freeze artifact should use pad playback, not only fixed preview samples"
    );
    let committed_buffer = render_w30_preview_offline(
        &committed_state.runtime.w30_preview,
        48_000,
        2,
        committed_pad_playback.sample_count,
    );

    let full_action_log = committed_state.session.action_log.clone();
    let target_action_cursor = full_action_log.actions.len();
    let loop_freeze_action_id = full_action_log
        .actions
        .last()
        .expect("loop-freeze action")
        .id;
    let snapshot_id = SnapshotId::from("snap-before-loop-freeze");
    let mut restore_session = committed_state.session.clone();
    restore_session.runtime_state = Default::default();
    restore_session.snapshots = vec![Snapshot {
        snapshot_id: snapshot_id.clone(),
        created_at: "2026-04-30T10:35:00Z".into(),
        label: "before loop freeze".into(),
        action_cursor: pre_freeze_action_cursor,
        payload: Some(riotbox_core::session::SnapshotPayload {
            payload_version: riotbox_core::session::SnapshotPayloadVersion::V1,
            snapshot_id,
            action_cursor: pre_freeze_action_cursor,
            runtime_state: pre_freeze_runtime,
        }),
    }];

    save_session_json(&session_path, &restore_session).expect("save replay hydration session");
    let mut replayed_state = JamAppState::from_json_files(&session_path, None::<&Path>)
        .expect("reload replay session");
    let report = replayed_state
        .apply_restore_target_from_snapshot_payload(target_action_cursor)
        .expect("snapshot payload restore hydrates loop-freeze artifact suffix");
    assert_eq!(
        replayed_state.session.runtime_state.lane_state.w30.last_capture,
        Some(produced_capture_id.clone())
    );
    assert!(
        replayed_state
            .capture_audio_cache
            .contains_key(&produced_capture_id),
        "replayed state should retain the produced capture audio cache"
    );
    assert_eq!(
        replayed_state.runtime.w30_preview.capture_id.as_deref(),
        Some(produced_capture_id.as_str())
    );
    let replayed_pad_playback = replayed_state
        .runtime
        .w30_preview
        .pad_playback
        .as_ref()
        .expect("replayed loop-freeze artifact playback");
    let replayed_buffer = render_w30_preview_offline(
        &replayed_state.runtime.w30_preview,
        48_000,
        2,
        replayed_pad_playback.sample_count,
    );

    assert_eq!(report.applied_action_ids, vec![loop_freeze_action_id]);
    assert_eq!(
        replayed_state.session.runtime_state.lane_state.w30,
        committed_state.session.runtime_state.lane_state.w30
    );
    assert_eq!(
        replayed_state.runtime.w30_preview.mode,
        committed_state.runtime.w30_preview.mode
    );
    assert_eq!(
        replayed_state.runtime.w30_preview.source_profile,
        committed_state.runtime.w30_preview.source_profile
    );
    assert_eq!(
        replayed_state.runtime.w30_preview.capture_id,
        committed_state.runtime.w30_preview.capture_id
    );
    assert_recipe_buffers_match(
        "snapshot payload restore loop-freeze artifact -> committed loop-freeze",
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
        "loop-freeze artifact playback -> fallback oscillator",
        &replayed_buffer,
        &fallback_buffer,
        0.0005,
        0.001,
    );
}

#[test]
fn w30_snapshot_payload_restore_hydrates_promote_resample_artifact_preview_output() {
    let tempdir = tempdir().expect("create resample replay tempdir");
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
    session.runtime_state.macro_state.w30_grit = 0.73;
    save_source_graph_json(&graph_path, &graph).expect("save replay source graph");
    save_session_json(&session_path, &session).expect("save replay session");

    let mut committed_state =
        JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("load app state");
    committed_state.queue_capture_bar(300);
    commit_w30_replay_step(&mut committed_state, CommitBoundary::Phrase, 0, 1, 0, 400);
    assert!(committed_state.queue_promote_last_capture(410));
    commit_w30_replay_step(&mut committed_state, CommitBoundary::Bar, 4, 2, 0, 500);

    let pre_resample_action_cursor = committed_state.session.action_log.actions.len();
    let pre_resample_runtime = committed_state.session.runtime_state.clone();
    assert_eq!(
        committed_state.session.runtime_state.lane_state.w30.last_capture,
        Some(CaptureId::from("cap-01"))
    );

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
    assert_eq!(produced_capture.storage_path, "captures/cap-02.wav");
    assert!(
        committed_state
            .capture_audio_cache
            .contains_key(&produced_capture_id),
        "resample commit should write and cache the produced artifact"
    );
    assert_eq!(
        committed_state.session.runtime_state.lane_state.w30.last_capture,
        Some(produced_capture_id.clone())
    );
    let produced_artifact_path = tempdir.path().join(&produced_capture.storage_path);
    assert!(
        produced_artifact_path.is_file(),
        "resample commit should persist a reloadable WAV artifact at {}",
        produced_artifact_path.display()
    );
    let committed_pad_playback = committed_state
        .runtime
        .w30_preview
        .pad_playback
        .as_ref()
        .expect("committed resample artifact playback");
    let committed_buffer = render_w30_preview_offline(
        &committed_state.runtime.w30_preview,
        48_000,
        2,
        committed_pad_playback.sample_count,
    );

    let full_action_log = committed_state.session.action_log.clone();
    let target_action_cursor = full_action_log.actions.len();
    let resample_action_id = full_action_log
        .actions
        .last()
        .expect("resample action")
        .id;
    let snapshot_id = SnapshotId::from("snap-before-promote-resample");
    let mut restore_session = committed_state.session.clone();
    restore_session.runtime_state = Default::default();
    restore_session.snapshots = vec![Snapshot {
        snapshot_id: snapshot_id.clone(),
        created_at: "2026-04-30T11:20:00Z".into(),
        label: "before promote resample".into(),
        action_cursor: pre_resample_action_cursor,
        payload: Some(riotbox_core::session::SnapshotPayload {
            payload_version: riotbox_core::session::SnapshotPayloadVersion::V1,
            snapshot_id,
            action_cursor: pre_resample_action_cursor,
            runtime_state: pre_resample_runtime,
        }),
    }];

    save_session_json(&session_path, &restore_session).expect("save resample replay session");
    let mut replayed_state = JamAppState::from_json_files(&session_path, Some(&graph_path))
        .expect("reload replay session");
    let report = replayed_state
        .apply_restore_target_from_snapshot_payload(target_action_cursor)
        .expect("snapshot payload restore hydrates promote.resample artifact suffix");
    let replayed_pad_playback = replayed_state
        .runtime
        .w30_preview
        .pad_playback
        .as_ref()
        .expect("replayed resample artifact playback");
    let replayed_buffer = render_w30_preview_offline(
        &replayed_state.runtime.w30_preview,
        48_000,
        2,
        replayed_pad_playback.sample_count,
    );

    assert_eq!(report.applied_action_ids, vec![resample_action_id]);
    assert_eq!(
        replayed_state.session.runtime_state.lane_state.w30,
        committed_state.session.runtime_state.lane_state.w30
    );
    assert_eq!(
        replayed_state.runtime.w30_preview.capture_id,
        committed_state.runtime.w30_preview.capture_id
    );
    assert_eq!(
        replayed_state.runtime.w30_preview.source_profile,
        committed_state.runtime.w30_preview.source_profile
    );
    assert_recipe_buffers_match(
        "snapshot payload restore promote.resample artifact -> committed resample",
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
        "promote.resample artifact playback -> fallback oscillator",
        &replayed_buffer,
        &fallback_buffer,
        0.0005,
        0.001,
    );
}

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

    let full_action_log = committed_state.session.action_log.clone();
    let target_action_cursor = full_action_log.actions.len();
    let promotion_action_id = full_action_log
        .actions
        .last()
        .expect("promotion action")
        .id;
    let snapshot_id = SnapshotId::from("snap-before-resample-pad-promotion");
    let mut restore_session = committed_state.session.clone();
    restore_session.runtime_state = Default::default();
    restore_session.snapshots = vec![Snapshot {
        snapshot_id: snapshot_id.clone(),
        created_at: "2026-04-30T12:25:00Z".into(),
        label: "before resample pad promotion".into(),
        action_cursor: pre_promotion_action_cursor,
        payload: Some(riotbox_core::session::SnapshotPayload {
            payload_version: riotbox_core::session::SnapshotPayloadVersion::V1,
            snapshot_id,
            action_cursor: pre_promotion_action_cursor,
            runtime_state: pre_promotion_runtime,
        }),
    }];

    save_session_json(&session_path, &restore_session).expect("save promotion replay session");
    let mut replayed_state = JamAppState::from_json_files(&session_path, Some(&graph_path))
        .expect("reload replay session");
    let report = replayed_state
        .apply_restore_target_from_snapshot_payload(target_action_cursor)
        .expect("snapshot payload restore replays resample pad promotion");
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

    assert_eq!(report.applied_action_ids, vec![promotion_action_id]);
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
