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

    let plan = riotbox_core::replay::build_committed_replay_plan(
        &committed_state.session.action_log,
    )
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

    let mut anchor_session = replay_base_session;
    anchor_session.action_log = full_action_log.clone();
    let anchor_report =
        riotbox_core::replay::apply_replay_plan_to_session(&mut anchor_session, &committed_plan[..1])
            .expect("W-30 browse anchor materializes");
    assert_eq!(anchor_report.applied_action_ids, vec![browse_action_id]);

    let snapshot_id = SnapshotId::from("snap-after-w30-browse");
    let mut restore_session = committed_state.session.clone();
    restore_session.runtime_state = Default::default();
    restore_session.snapshots = vec![Snapshot {
        snapshot_id: snapshot_id.clone(),
        created_at: "2026-04-30T08:25:00Z".into(),
        label: "after W-30 browse".into(),
        action_cursor: browse_action_cursor,
        payload: Some(riotbox_core::session::SnapshotPayload {
            payload_version: riotbox_core::session::SnapshotPayloadVersion::V1,
            snapshot_id,
            action_cursor: browse_action_cursor,
            runtime_state: anchor_session.runtime_state.clone(),
        }),
    }];

    let mut replayed_state =
        JamAppState::from_parts(restore_session, Some(graph), ActionQueue::new());
    replayed_state.source_audio_cache = Some(source_audio_cache);
    let replay_report = replayed_state
        .apply_restore_target_from_snapshot_payload(trigger_action_cursor)
        .expect("snapshot payload restore applies W-30 trigger suffix");
    let replayed_trigger = render_w30_replay_buffer(&replayed_state);

    assert_eq!(replay_report.target_action_cursor, trigger_action_cursor);
    assert_eq!(
        replay_report.anchor_snapshot_id.as_deref(),
        Some("snap-after-w30-browse")
    );
    assert_eq!(
        replay_report.anchor_action_cursor,
        Some(browse_action_cursor)
    );
    assert_eq!(replay_report.applied_action_ids, vec![trigger_action_id]);
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
    committed_state.session.runtime_state.lane_state.w30.active_bank =
        Some(BankId::from("bank-a"));
    committed_state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
    committed_state.session.runtime_state.lane_state.w30.last_capture =
        Some(source_capture_id.clone());
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
    let mut replayed_state =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect("reload replay session");
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
    let fallback_buffer =
        render_w30_preview_offline(&fallback_preview, 48_000, 2, replayed_pad_playback.sample_count);
    assert_w30_replay_buffers_differ(
        "loop-freeze artifact playback -> fallback oscillator",
        &replayed_buffer,
        &fallback_buffer,
        0.0005,
        0.001,
    );
}

fn w30_source_backed_replay_state() -> (
    tempfile::TempDir,
    SourceGraph,
    SourceAudioCache,
    JamAppState,
) {
    let tempdir = tempdir().expect("create source audio tempdir");
    let source_path = tempdir.path().join("source.wav");
    write_pcm16_wave(&source_path, 48_000, 2, 2.0);
    let source_audio_cache =
        SourceAudioCache::load_pcm_wav(&source_path).expect("load source audio cache");

    let mut graph = sample_graph();
    graph.source.path = source_path.to_string_lossy().into_owned();
    graph.source.duration_seconds = 2.0;
    let mut state = w30_slice_pool_state_with_source_windows(graph.clone(), source_audio_cache.clone());
    let cap_02 = state
        .session
        .captures
        .iter_mut()
        .find(|capture| capture.capture_id == CaptureId::from("cap-02"))
        .expect("cap-02 fixture");
    cap_02.source_window = Some(CaptureSourceWindow {
        source_id: graph.source.source_id.clone(),
        start_seconds: 0.0625,
        end_seconds: 0.5625,
        start_frame: 3_000,
        end_frame: 27_000,
    });
    state.refresh_view();

    (tempdir, graph, source_audio_cache, state)
}

fn commit_w30_replay_step(
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

fn render_w30_replay_buffer(state: &JamAppState) -> Vec<f32> {
    let buffer = render_w30_preview_offline(
        &state.runtime.w30_preview,
        48_000,
        2,
        W30_PREVIEW_SAMPLE_WINDOW_LEN,
    );
    let metrics = signal_metrics(&buffer);
    assert!(
        metrics.active_samples > 100 && metrics.peak_abs > 0.001 && metrics.rms > 0.0001,
        "W-30 replay render too close to silence: active {}, peak {}, rms {}",
        metrics.active_samples,
        metrics.peak_abs,
        metrics.rms
    );
    buffer
}

fn assert_w30_replay_buffers_differ(
    label: &str,
    left: &[f32],
    right: &[f32],
    min_rms_delta: f32,
    min_peak_delta: f32,
) {
    let delta = signal_delta_metrics(left, right);
    assert!(
        delta.rms >= min_rms_delta,
        "{label} signal delta RMS {} below {min_rms_delta}; peak {}, active {}, zero crossings {}",
        delta.rms,
        delta.peak_abs,
        delta.active_samples,
        delta.zero_crossings
    );
    assert!(
        delta.peak_abs >= min_peak_delta,
        "{label} signal delta peak {} below {min_peak_delta}",
        delta.peak_abs
    );
}
