#[test]
fn committed_w30_bank_swap_updates_lane_focus_and_log_result() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-a"),
        pad_id: PadId::from("pad-01"),
    });
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
            bank_id: BankId::from("bank-b"),
            pad_id: PadId::from("pad-01"),
        }),
        is_pinned: false,
        notes: Some("bank b".into()),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_swap_bank(612),
        Some(QueueControlResult::Enqueued)
    );

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 41,
            bar_index: 11,
            phrase_index: 3,
            scene_id: Some(SceneId::from("scene-1")),
        },
        712,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .as_ref()
            .map(ToString::to_string),
        Some("bank-b".into())
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .w30
            .focused_pad
            .as_ref()
            .map(ToString::to_string),
        Some("pad-01".into())
    );
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
    assert_eq!(state.jam_view.lanes.w30_pending_bank_swap_target, None);
    assert_eq!(state.runtime_view.w30_preview_mode, "live_recall");
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("swapped W-30 bank to bank-b/pad-01 with cap-02")
    );
}

#[test]
fn committed_w30_slice_pool_browse_updates_last_capture_and_log_result() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-a"),
        pad_id: PadId::from("pad-01"),
    });
    state.session.captures.push(CaptureRef {
        capture_id: CaptureId::from("cap-02"),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["asset-b".into()],
        source_window: None,
        lineage_capture_refs: vec![CaptureId::from("cap-01")],
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: "captures/cap-02.wav".into(),
        assigned_target: Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        }),
        is_pinned: false,
        notes: Some("alt slice".into()),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
    state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_browse_slice_pool(713),
        Some(QueueControlResult::Enqueued)
    );

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Beat,
            beat_index: 42,
            bar_index: 11,
            phrase_index: 3,
            scene_id: Some(SceneId::from("scene-1")),
        },
        813,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .as_ref()
            .map(ToString::to_string),
        Some("bank-a".into())
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .w30
            .focused_pad
            .as_ref()
            .map(ToString::to_string),
        Some("pad-01".into())
    );
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
    assert_eq!(state.jam_view.lanes.w30_pending_slice_pool_target, None);
    assert_eq!(state.runtime_view.w30_preview_mode, "live_recall");
    assert_eq!(state.runtime_view.w30_preview_profile, "slice_pool_browse");
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("browsed W-30 slice pool to cap-02 on bank-a/pad-01 at beat 42 / phrase 3")
    );
}

#[test]
fn committed_w30_damage_profile_updates_grit_and_log_result() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-a"),
        pad_id: PadId::from("pad-01"),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
    state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
    state.session.runtime_state.lane_state.w30.preview_mode = Some(W30PreviewModeState::LiveRecall);
    state.session.runtime_state.macro_state.w30_grit = 0.4;
    state.refresh_view();

    assert_eq!(
        state.queue_w30_apply_damage_profile(620),
        Some(QueueControlResult::Enqueued)
    );

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 45,
            bar_index: 12,
            phrase_index: 3,
            scene_id: Some(SceneId::from("scene-1")),
        },
        720,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state.session.runtime_state.macro_state.w30_grit,
        JamAppState::W30_DAMAGE_PROFILE_GRIT
    );
    assert_eq!(state.jam_view.lanes.w30_pending_damage_profile_target, None);
    assert_eq!(state.runtime_view.w30_preview_mode, "live_recall");
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("applied shred damage profile to cap-01 on W-30 pad bank-a/pad-01")
    );
}

#[test]
fn committed_w30_step_focus_updates_lane_focus_and_preview() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-a"),
        pad_id: PadId::from("pad-01"),
    });
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
            bank_id: BankId::from("bank-b"),
            pad_id: PadId::from("pad-04"),
        }),
        is_pinned: true,
        notes: Some("secondary".into()),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
    state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_step_focus(612),
        Some(QueueControlResult::Enqueued)
    );

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Beat,
            beat_index: 37,
            bar_index: 10,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        702,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .as_ref()
            .map(ToString::to_string),
        Some("bank-b".into())
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .w30
            .focused_pad
            .as_ref()
            .map(ToString::to_string),
        Some("pad-04".into())
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .w30
            .last_capture
            .as_ref()
            .map(ToString::to_string),
        Some("cap-01".into())
    );
    assert_eq!(
        state.jam_view.lanes.w30_active_bank.as_deref(),
        Some("bank-b")
    );
    assert_eq!(
        state.jam_view.lanes.w30_focused_pad.as_deref(),
        Some("pad-04")
    );
    assert_eq!(state.jam_view.lanes.w30_pending_focus_step_target, None);
    assert_eq!(
        state.runtime.w30_preview.mode,
        W30PreviewRenderMode::LiveRecall
    );
    assert_eq!(
        state.runtime.w30_preview.routing,
        W30PreviewRenderRouting::MusicBusPreview
    );
    assert_eq!(
        state.runtime.w30_preview.source_profile,
        Some(W30PreviewSourceProfile::PromotedRecall)
    );
    assert_eq!(
        state.runtime.w30_preview.active_bank_id.as_deref(),
        Some("bank-b")
    );
    assert_eq!(
        state.runtime.w30_preview.focused_pad_id.as_deref(),
        Some("pad-04")
    );
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("focused W-30 pad bank-b/pad-04 at beat 37 / phrase 2")
    );
}

