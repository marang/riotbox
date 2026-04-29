#[test]
fn stale_stopped_audio_timing_snapshot_does_not_cancel_transport_start() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.update_transport_clock(TransportClockState {
        is_playing: false,
        position_beats: 32.0,
        beat_index: 32,
        bar_index: 8,
        phrase_index: 1,
        current_scene: Some(SceneId::from("scene-1")),
    });
    state.set_transport_playing(true);

    let committed = state.apply_audio_timing_snapshot(
        AudioRuntimeTimingSnapshot {
            is_transport_running: false,
            tempo_bpm: 124.0,
            position_beats: 32.0,
        },
        2_500,
    );

    assert!(committed.is_empty());
    assert!(state.runtime.transport.is_playing);
    assert_eq!(state.runtime.transport.position_beats, 32.0);
    assert!(state.jam_view.transport.is_playing);
    assert_eq!(state.jam_view.transport.position_beats, 32.0);
}

#[test]
fn committed_capture_actions_materialize_capture_refs() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.queue_capture_bar(300);

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 32,
            bar_index: 8,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(state.session.captures.len(), 2);
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .w30
            .last_capture
            .as_ref()
            .map(ToString::to_string),
        Some("cap-02".into())
    );
    assert_eq!(state.jam_view.capture.capture_count, 2);
    assert_eq!(
        state.jam_view.capture.last_capture_id.as_deref(),
        Some("cap-02")
    );
    assert_eq!(state.jam_view.capture.last_capture_target.as_deref(), None);
    assert_eq!(state.jam_view.capture.last_capture_target_kind, None);
    assert_eq!(state.jam_view.capture.last_capture_origin_count, 2);
    assert_eq!(state.jam_view.capture.unassigned_capture_count, 2);
    assert_eq!(state.jam_view.capture.promoted_capture_count, 0);
    let source_window = state.session.captures[1]
        .source_window
        .as_ref()
        .expect("capture source window");
    assert_eq!(source_window.source_id, SourceId::from("src-1"));
    assert!((source_window.start_seconds - 15.238).abs() < 0.01);
    assert!((source_window.end_seconds - 22.857).abs() < 0.01);
    assert_eq!(source_window.start_frame, 731_428);
    assert_eq!(source_window.end_frame, 1_097_142);

    let tempdir = tempdir().expect("create capture window tempdir");
    let session_path = tempdir.path().join("capture-window.json");
    save_session_json(&session_path, &state.session).expect("save capture-window session");
    let reloaded = load_session_json(&session_path).expect("reload capture-window session");
    assert_eq!(
        reloaded.captures[1].source_window,
        Some(source_window.clone())
    );
}

#[test]
fn committed_promotion_actions_assign_target_to_existing_capture() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.queue_promote_last_capture(300);

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 33,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(state.session.captures.len(), 1);
    assert_eq!(
        state.session.captures[0].assigned_target,
        Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        })
    );
    assert_eq!(
        state.jam_view.capture.last_capture_target.as_deref(),
        Some("pad bank-a/pad-01")
    );
    assert_eq!(
        state.jam_view.capture.last_capture_target_kind,
        Some(CaptureTargetKindView::W30Pad)
    );
    assert_eq!(
        state.jam_view.capture.last_promotion_result.as_deref(),
        Some("promoted to pad bank-a/pad-01")
    );
}

#[test]
fn second_promotion_updates_existing_capture_note_and_target() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.queue_promote_last_capture(300);
    state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 33,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-02"));
    assert!(state.queue_promote_last_capture(401));
    state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 37,
            bar_index: 10,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        500,
    );

    assert_eq!(
        state.session.captures[0].assigned_target,
        Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-02"),
        })
    );
    assert_eq!(
        state.session.captures[0].notes.as_deref(),
        Some("keeper | promoted to pad bank-a/pad-02")
    );
}

#[test]
fn toggling_pin_latest_capture_updates_session_and_view() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(state.toggle_pin_latest_capture(), Some(true));
    assert!(state.session.captures[0].is_pinned);
    assert_eq!(state.jam_view.capture.pinned_capture_count, 1);
    assert_eq!(
        state.jam_view.capture.pinned_capture_ids,
        vec!["cap-01".to_string()]
    );

    assert_eq!(state.toggle_pin_latest_capture(), Some(false));
    assert!(!state.session.captures[0].is_pinned);
    assert_eq!(state.jam_view.capture.pinned_capture_count, 0);
    assert!(state.jam_view.capture.pinned_capture_ids.is_empty());
}

#[test]
fn queue_w30_live_recall_targets_committed_lane_focus_before_latest_pinned_capture() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures.push(CaptureRef {
        capture_id: CaptureId::from("cap-02"),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["asset-b".into()],
        source_window: None,
        lineage_capture_refs: Vec::new(),
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: "captures/cap-02.wav".into(),
        assigned_target: Some(CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-04".into(),
        }),
        is_pinned: false,
        notes: Some("secondary".into()),
    });
    state.session.captures.push(CaptureRef {
        capture_id: CaptureId::from("cap-03"),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["asset-c".into()],
        source_window: None,
        lineage_capture_refs: Vec::new(),
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: "captures/cap-03.wav".into(),
        assigned_target: Some(CaptureTarget::W30Pad {
            bank_id: "bank-c".into(),
            pad_id: "pad-07".into(),
        }),
        is_pinned: true,
        notes: Some("keeper".into()),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-04"));
    state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-03"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_live_recall(600),
        Some(QueueControlResult::Enqueued)
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::W30LiveRecall);
    assert_eq!(
        pending[0].target.bank_id.as_ref().map(ToString::to_string),
        Some("bank-b".into())
    );
    assert_eq!(
        pending[0].target.pad_id.as_ref().map(ToString::to_string),
        Some("pad-04".into())
    );
    assert_eq!(
        pending[0].explanation.as_deref(),
        Some("recall cap-02 on W-30 pad bank-b/pad-04")
    );
    assert_eq!(
        state.jam_view.lanes.w30_pending_recall_target.as_deref(),
        Some("bank-b/pad-04")
    );
    assert_eq!(state.jam_view.lanes.w30_pending_audition, None);
    assert_eq!(state.jam_view.lanes.w30_pending_audition_target, None);
}

#[test]
fn queue_w30_promoted_audition_targets_committed_lane_focus() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures.push(CaptureRef {
        capture_id: CaptureId::from("cap-02"),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["asset-b".into()],
        source_window: None,
        lineage_capture_refs: Vec::new(),
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: "captures/cap-02.wav".into(),
        assigned_target: Some(CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-04".into(),
        }),
        is_pinned: false,
        notes: Some("secondary".into()),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-04"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_promoted_audition(620),
        Some(QueueControlResult::Enqueued)
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::W30AuditionPromoted);
    assert_eq!(
        pending[0].target.bank_id.as_ref().map(ToString::to_string),
        Some("bank-b".into())
    );
    assert_eq!(
        pending[0].target.pad_id.as_ref().map(ToString::to_string),
        Some("pad-04".into())
    );
    assert_eq!(
        pending[0].explanation.as_deref(),
        Some("audition promoted cap-02 on W-30 pad bank-b/pad-04")
    );
    assert_eq!(
        state.jam_view.lanes.w30_pending_audition_target.as_deref(),
        Some("bank-b/pad-04")
    );
    let pending_audition = state
        .jam_view
        .lanes
        .w30_pending_audition
        .as_ref()
        .expect("pending promoted audition projects into Jam view");
    assert_eq!(pending_audition.kind, W30PendingAuditionKind::Promoted);
    assert_eq!(pending_audition.target, "bank-b/pad-04");
    assert_eq!(pending_audition.quantization, "next_bar");
    assert_eq!(state.jam_view.lanes.w30_pending_recall_target, None);
}

#[test]
fn queue_w30_audition_targets_raw_capture_before_promotion() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_w30_audition(620),
        Some(QueueControlResult::Enqueued)
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::W30AuditionRawCapture);
    assert_eq!(
        pending[0].target.bank_id.as_ref().map(ToString::to_string),
        Some("bank-a".into())
    );
    assert_eq!(
        pending[0].target.pad_id.as_ref().map(ToString::to_string),
        Some("pad-01".into())
    );
    assert_eq!(
        pending[0].explanation.as_deref(),
        Some("audition raw capture cap-01 on W-30 preview bank-a/pad-01")
    );
    assert_eq!(
        state.jam_view.lanes.w30_pending_audition_target.as_deref(),
        Some("bank-a/pad-01")
    );
    let pending_audition = state
        .jam_view
        .lanes
        .w30_pending_audition
        .as_ref()
        .expect("pending raw audition projects into Jam view");
    assert_eq!(pending_audition.kind, W30PendingAuditionKind::RawCapture);
    assert_eq!(pending_audition.target, "bank-a/pad-01");
    assert_eq!(pending_audition.quantization, "next_bar");
}

#[test]
fn committed_w30_raw_capture_audition_updates_preview_state() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_w30_audition(620),
        Some(QueueControlResult::Enqueued)
    );
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 40,
            bar_index: 10,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        700,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .map(|action| action.command),
        Some(ActionCommand::W30AuditionRawCapture)
    );
    assert_eq!(
        state.session.runtime_state.lane_state.w30.preview_mode,
        Some(W30PreviewModeState::RawCaptureAudition)
    );
    assert_eq!(
        state.runtime.w30_preview.mode,
        W30PreviewRenderMode::RawCaptureAudition
    );
    assert_eq!(
        state.runtime.w30_preview.source_profile,
        Some(W30PreviewSourceProfile::RawCaptureAudition)
    );
    assert_eq!(
        state.runtime.w30_preview.capture_id.as_deref(),
        Some("cap-01")
    );
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("auditioned raw cap-01 on W-30 preview bank-a/pad-01")
    );
}

