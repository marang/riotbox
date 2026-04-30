#[test]
fn snapshot_payload_restore_hydrates_capture_loop_artifact_preview_output() {
    let tempdir = tempdir().expect("create capture loop replay tempdir");
    let session_path = tempdir.path().join("session.json");
    let captures_dir = tempdir.path().join("captures");
    fs::create_dir_all(&captures_dir).expect("create captures dir");
    write_w30_capture_to_pad_artifact_wave(captures_dir.join("cap-01.wav"), 48_000, 2, 2.0);

    let action_id = ActionId(78);
    let capture_id = CaptureId::from("cap-01");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-30T12:44:00Z");
    session.runtime_state.mixer_state.music_level = 1.0;
    session.captures.push(CaptureRef {
        capture_id: capture_id.clone(),
        capture_type: CaptureType::Loop,
        source_origin_refs: vec!["source-1".into()],
        source_window: Some(CaptureSourceWindow {
            source_id: "source-1".into(),
            start_seconds: 0.0,
            end_seconds: 2.0,
            start_frame: 0,
            end_frame: 96_000,
        }),
        lineage_capture_refs: Vec::new(),
        resample_generation_depth: 0,
        created_from_action: Some(action_id),
        storage_path: "captures/cap-01.wav".into(),
        assigned_target: None,
        is_pinned: false,
        notes: Some("source-window loop capture".into()),
    });
    session.action_log.actions.push(Action {
        id: action_id,
        actor: ActorType::User,
        command: ActionCommand::CaptureLoop,
        params: ActionParams::Capture { bars: Some(2) },
        target: ActionTarget {
            scope: Some(TargetScope::LaneW30),
            ..Default::default()
        },
        requested_at: 480,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(500),
        result: Some(ActionResult {
            accepted: true,
            summary: "capture loop committed".into(),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: Some("capture source-window loop into W-30 path".into()),
    });
    session.action_log.commit_records.push(ActionCommitRecord {
        action_id,
        boundary: CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 16,
            bar_index: 4,
            phrase_index: 1,
            scene_id: None,
        },
        commit_sequence: 1,
        committed_at: 500,
    });
    session.snapshots = vec![Snapshot {
        snapshot_id: SnapshotId::from("before-capture-loop"),
        created_at: "2026-04-30T12:44:01Z".into(),
        label: "before capture loop".into(),
        action_cursor: 0,
        payload: Some(riotbox_core::session::SnapshotPayload::from_runtime_state(
            &SnapshotId::from("before-capture-loop"),
            0,
            &session.runtime_state,
        )),
    }];
    save_session_json(&session_path, &session).expect("save capture loop replay session");

    let mut committed_state = JamAppState::from_json_files(&session_path, None::<&Path>)
        .expect("load committed comparison state");
    committed_state.session.runtime_state.lane_state.w30.last_capture = Some(capture_id.clone());
    committed_state.session.runtime_state.lane_state.w30.preview_mode =
        Some(W30PreviewModeState::LiveRecall);
    committed_state.refresh_view();
    let committed_pad_playback = committed_state
        .runtime
        .w30_preview
        .pad_playback
        .as_ref()
        .expect("committed capture.loop artifact playback");
    let committed_buffer = render_w30_preview_offline(
        &committed_state.runtime.w30_preview,
        48_000,
        2,
        committed_pad_playback.sample_count,
    );

    let mut replayed_state = JamAppState::from_json_files(&session_path, None::<&Path>)
        .expect("reload capture loop replay session");
    let report = replayed_state
        .apply_restore_target_from_snapshot_payload(1)
        .expect("snapshot payload restore hydrates capture.loop artifact suffix");
    let replayed_pad_playback = replayed_state
        .runtime
        .w30_preview
        .pad_playback
        .as_ref()
        .expect("replayed capture.loop artifact playback");
    let replayed_buffer = render_w30_preview_offline(
        &replayed_state.runtime.w30_preview,
        48_000,
        2,
        replayed_pad_playback.sample_count,
    );

    assert_eq!(report.applied_action_ids, vec![action_id]);
    assert_eq!(
        replayed_state.session.runtime_state.lane_state.w30.last_capture,
        Some(capture_id)
    );
    assert_recipe_buffers_match(
        "snapshot payload restore capture.loop artifact -> committed capture",
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
        "capture.loop artifact playback -> fallback oscillator",
        &replayed_buffer,
        &fallback_buffer,
        0.0005,
        0.001,
    );
}
