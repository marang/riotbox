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
    let mut restore_session = committed_state.session.clone();
    restore_session.runtime_state = Default::default();
    restore_session.snapshots = vec![snapshot_payload_for_anchor(
        "snap-before-loop-freeze",
        "before loop freeze",
        "2026-04-30T10:35:00Z",
        pre_freeze_action_cursor,
        &pre_freeze_runtime,
    )];

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

    assert_restore_report_identity(
        &report,
        target_action_cursor,
        "snap-before-loop-freeze",
        pre_freeze_action_cursor,
        vec![loop_freeze_action_id],
    );
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
