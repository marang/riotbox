#[test]
fn queue_w30_apply_damage_profile_targets_focused_lane_capture_on_next_bar() {
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
    state.refresh_view();

    assert_eq!(
        state.queue_w30_apply_damage_profile(644),
        Some(QueueControlResult::Enqueued)
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::W30ApplyDamageProfile);
    assert_eq!(pending[0].quantization, Quantization::NextBar);
    assert_eq!(
        pending[0].target.bank_id.as_ref().map(ToString::to_string),
        Some("bank-a".into())
    );
    assert_eq!(
        pending[0].target.pad_id.as_ref().map(ToString::to_string),
        Some("pad-01".into())
    );
    assert!(matches!(
        &pending[0].params,
        ActionParams::Mutation {
            intensity,
            target_id: Some(target_id),
        } if (*intensity - JamAppState::W30_DAMAGE_PROFILE_GRIT).abs() < f32::EPSILON
            && target_id == "cap-01"
    ));
    assert_eq!(
        pending[0].explanation.as_deref(),
        Some("apply shred damage profile to cap-01 on W-30 pad bank-a/pad-01")
    );
    assert_eq!(
        state
            .jam_view
            .lanes
            .w30_pending_damage_profile_target
            .as_deref(),
        Some("bank-a/pad-01")
    );
}

#[test]
fn queue_w30_live_recall_falls_back_to_latest_pinned_capture_without_explicit_focus() {
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
        is_pinned: true,
        notes: Some("keeper".into()),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-z"));
    state.session.runtime_state.lane_state.w30.focused_pad = None;
    state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_live_recall(605),
        Some(QueueControlResult::Enqueued)
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
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
}

#[test]
fn queueing_w30_pad_cues_blocks_conflicting_pending_actions() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-b"),
        pad_id: PadId::from("pad-03"),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_live_recall(630),
        Some(QueueControlResult::Enqueued)
    );
    assert_eq!(
        state.queue_w30_promoted_audition(631),
        Some(QueueControlResult::AlreadyPending)
    );
    assert_eq!(
        state.queue_w30_step_focus(631),
        Some(QueueControlResult::AlreadyPending)
    );
    assert_eq!(
        state.queue_w30_swap_bank(631),
        Some(QueueControlResult::AlreadyPending)
    );
    assert_eq!(
        state.queue_w30_apply_damage_profile(631),
        Some(QueueControlResult::AlreadyPending)
    );
    assert_eq!(
        state.queue_w30_trigger_pad(632),
        Some(QueueControlResult::AlreadyPending)
    );

    let other_graph = sample_graph();
    let mut other_state = JamAppState::from_parts(
        sample_session(&other_graph),
        Some(other_graph),
        ActionQueue::new(),
    );
    other_state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-c"),
        pad_id: PadId::from("pad-05"),
    });
    other_state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-c"));
    other_state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-05"));
    other_state.refresh_view();

    assert_eq!(
        other_state.queue_w30_promoted_audition(632),
        Some(QueueControlResult::Enqueued)
    );
    assert_eq!(
        other_state.queue_w30_live_recall(633),
        Some(QueueControlResult::AlreadyPending)
    );
    assert_eq!(
        other_state.queue_w30_step_focus(633),
        Some(QueueControlResult::AlreadyPending)
    );
    assert_eq!(
        other_state.queue_w30_swap_bank(633),
        Some(QueueControlResult::AlreadyPending)
    );
    assert_eq!(
        other_state.queue_w30_apply_damage_profile(633),
        Some(QueueControlResult::AlreadyPending)
    );
    assert_eq!(
        other_state.queue_w30_trigger_pad(634),
        Some(QueueControlResult::AlreadyPending)
    );
}

#[test]
fn queueing_w30_internal_resample_blocks_duplicate_pending_actions() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-b"),
        pad_id: PadId::from("pad-03"),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
    state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_internal_resample(635),
        Some(QueueControlResult::Enqueued)
    );
    assert_eq!(
        state.queue_w30_internal_resample(636),
        Some(QueueControlResult::AlreadyPending)
    );
}

#[test]
fn queueing_w30_internal_resample_blocks_pending_loop_freeze() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-b"),
        pad_id: PadId::from("pad-03"),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
    state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_loop_freeze(635),
        Some(QueueControlResult::Enqueued)
    );
    assert_eq!(
        state.queue_w30_internal_resample(636),
        Some(QueueControlResult::AlreadyPending)
    );
}

#[test]
fn queueing_w30_loop_freeze_blocks_pending_internal_resample() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-b"),
        pad_id: PadId::from("pad-03"),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
    state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_internal_resample(635),
        Some(QueueControlResult::Enqueued)
    );
    assert_eq!(
        state.queue_w30_loop_freeze(636),
        Some(QueueControlResult::AlreadyPending)
    );
}

#[test]
fn committed_w30_live_recall_updates_lane_focus_and_log_result() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-b"),
        pad_id: PadId::from("pad-03"),
    });
    state.session.captures[0].is_pinned = true;
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_live_recall(610),
        Some(QueueControlResult::Enqueued)
    );

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 33,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        700,
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
        Some("pad-03".into())
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
        Some("pad-03")
    );
    assert_eq!(state.jam_view.lanes.w30_pending_recall_target, None);
    assert_eq!(state.jam_view.lanes.w30_pending_audition_target, None);
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
        Some(W30PreviewSourceProfile::PinnedRecall)
    );
    assert_eq!(
        state.runtime.w30_preview.capture_id.as_deref(),
        Some("cap-01")
    );
    assert_eq!(state.runtime_view.w30_preview_mode, "live_recall");
    assert_eq!(state.runtime_view.w30_preview_profile, "pinned_recall");
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("recalled cap-01 on W-30 pad bank-b/pad-03")
    );
}

