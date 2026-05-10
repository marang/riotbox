#[test]
fn legacy_w30_preview_mode_is_backfilled_from_committed_preview_history() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.w30.preview_mode = None;
    session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
    session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
    session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
    session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-b"),
        pad_id: PadId::from("pad-03"),
    });
    session.action_log.actions.push(Action {
        id: ActionId(2),
        actor: ActorType::User,
        command: ActionCommand::W30AuditionPromoted,
        params: ActionParams::Mutation {
            target_id: Some("cap-01".into()),
            intensity: 0.68,
        },
        target: ActionTarget {
            scope: Some(TargetScope::LaneW30),
            bank_id: Some(BankId::from("bank-b")),
            pad_id: Some(PadId::from("pad-03")),
            ..Default::default()
        },
        requested_at: 600,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(700),
        undo_policy: UndoPolicy::Undoable,
        result: Some(ActionResult {
            accepted: true,
            summary: "auditioned cap-01 on W-30 pad bank-b/pad-03".into(),
        }),
        explanation: Some("audition promoted cap-01 on W-30 pad bank-b/pad-03".into()),
    });

    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.session.runtime_state.lane_state.w30.preview_mode,
        Some(W30PreviewModeState::PromotedAudition)
    );
    assert_eq!(
        state.runtime.w30_preview.mode,
        W30PreviewRenderMode::PromotedAudition
    );
}

#[test]
fn explicit_w30_preview_mode_overrides_stale_action_history() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.w30.preview_mode = Some(W30PreviewModeState::LiveRecall);
    session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
    session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
    session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
    session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-b"),
        pad_id: PadId::from("pad-03"),
    });
    session.action_log.actions.push(Action {
        id: ActionId(2),
        actor: ActorType::User,
        command: ActionCommand::W30AuditionPromoted,
        params: ActionParams::Mutation {
            target_id: Some("cap-01".into()),
            intensity: 0.68,
        },
        target: ActionTarget {
            scope: Some(TargetScope::LaneW30),
            bank_id: Some(BankId::from("bank-b")),
            pad_id: Some(PadId::from("pad-03")),
            ..Default::default()
        },
        requested_at: 600,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(700),
        undo_policy: UndoPolicy::Undoable,
        result: Some(ActionResult {
            accepted: true,
            summary: "auditioned cap-01 on W-30 pad bank-b/pad-03".into(),
        }),
        explanation: Some("audition promoted cap-01 on W-30 pad bank-b/pad-03".into()),
    });

    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.session.runtime_state.lane_state.w30.preview_mode,
        Some(W30PreviewModeState::LiveRecall)
    );
    assert_eq!(
        state.runtime.w30_preview.mode,
        W30PreviewRenderMode::LiveRecall
    );
}

#[test]
fn committed_tr909_actions_update_lane_state() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.queue_tr909_fill(300);
    state.queue_tr909_reinforce(301);

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 36,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    assert_eq!(committed.len(), 2);
    assert_eq!(
        state.session.runtime_state.lane_state.tr909.last_fill_bar,
        Some(9)
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .pattern_ref
            .as_deref(),
        Some("reinforce-scene-1")
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .reinforcement_mode,
        Some(Tr909ReinforcementModeState::BreakReinforce)
    );
    assert!(
        !state
            .session
            .runtime_state
            .lane_state
            .tr909
            .fill_armed_next_bar
    );
    assert_eq!(
        state.jam_view.lanes.tr909_reinforcement_mode,
        Some(Tr909ReinforcementModeState::BreakReinforce)
    );
    assert_eq!(
        state.runtime.tr909_render.mode,
        Tr909RenderMode::BreakReinforce
    );
    assert_eq!(
        state.runtime.tr909_render.routing,
        Tr909RenderRouting::DrumBusSupport
    );
    assert_eq!(
        state.runtime.tr909_render.pattern_ref.as_deref(),
        Some("reinforce-scene-1")
    );
}

#[test]
fn committed_mc202_role_change_updates_lane_state_and_macro_touch() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_mc202_role_toggle(300),
        QueueControlResult::Enqueued
    );

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 36,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state.session.runtime_state.lane_state.mc202.role.as_deref(),
        Some("leader")
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .mc202
            .phrase_ref
            .as_deref(),
        Some("leader-scene-1")
    );
    assert_eq!(state.session.runtime_state.macro_state.mc202_touch, 0.85);
    assert_eq!(state.jam_view.lanes.mc202_role.as_deref(), Some("leader"));
    assert_eq!(state.jam_view.lanes.mc202_pending_role, None);
    assert_eq!(
        state.jam_view.lanes.mc202_phrase_ref.as_deref(),
        Some("leader-scene-1")
    );
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("set MC-202 role to leader at 0.85")
    );
}

#[test]
fn committed_mc202_role_change_rejects_unknown_role_without_mutating_lane_state() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let original_mc202 = session.runtime_state.lane_state.mc202.clone();
    let original_touch = session.runtime_state.macro_state.mc202_touch;
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let mut draft = ActionDraft::new(
        ActorType::User,
        ActionCommand::Mc202SetRole,
        Quantization::NextPhrase,
        ActionTarget {
            scope: Some(TargetScope::LaneMc202),
            object_id: Some("scene_lock".into()),
            ..Default::default()
        },
    );
    draft.params = ActionParams::Mutation {
        intensity: 0.99,
        target_id: Some("scene_lock".into()),
    };
    state.queue.enqueue(draft, 300);

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 36,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(state.session.runtime_state.lane_state.mc202, original_mc202);
    assert_eq!(
        state.session.runtime_state.macro_state.mc202_touch,
        original_touch
    );
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| (result.accepted, result.summary.as_str())),
        Some((false, "rejected unknown MC-202 role scene_lock"))
    );
}

#[test]
fn committed_mc202_follower_generation_updates_phrase_ref_and_touch() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_mc202_generate_follower(300),
        QueueControlResult::Enqueued
    );

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 36,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state.session.runtime_state.lane_state.mc202.role.as_deref(),
        Some("follower")
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .mc202
            .phrase_ref
            .as_deref(),
        Some("follower-scene-1")
    );
    assert_eq!(state.session.runtime_state.macro_state.mc202_touch, 0.78);
    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Follower);
    assert_eq!(
        state.runtime.mc202_render.phrase_shape,
        Mc202PhraseShape::FollowerDrive
    );
    assert_eq!(
        state.runtime.mc202_render.routing,
        Mc202RenderRouting::MusicBusBass
    );
    assert_eq!(state.jam_view.lanes.mc202_role.as_deref(), Some("follower"));
    assert!(!state.jam_view.lanes.mc202_pending_follower_generation);
    assert!(!state.jam_view.lanes.mc202_pending_answer_generation);
    assert_eq!(
        state.jam_view.lanes.mc202_phrase_ref.as_deref(),
        Some("follower-scene-1")
    );
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("generated MC-202 follower phrase follower-scene-1 at 0.78")
    );
}

#[test]
fn committed_mc202_answer_generation_updates_phrase_ref_and_touch() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_mc202_generate_answer(300),
        QueueControlResult::Enqueued
    );

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 36,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state.session.runtime_state.lane_state.mc202.role.as_deref(),
        Some("answer")
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .mc202
            .phrase_ref
            .as_deref(),
        Some("answer-scene-1")
    );
    assert_eq!(state.session.runtime_state.macro_state.mc202_touch, 0.82);
    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Answer);
    assert_eq!(
        state.runtime.mc202_render.phrase_shape,
        Mc202PhraseShape::AnswerHook
    );
    assert_eq!(
        state.runtime.mc202_render.routing,
        Mc202RenderRouting::MusicBusBass
    );
    assert_eq!(state.jam_view.lanes.mc202_role.as_deref(), Some("answer"));
    assert!(!state.jam_view.lanes.mc202_pending_follower_generation);
    assert!(!state.jam_view.lanes.mc202_pending_answer_generation);
    assert_eq!(
        state.jam_view.lanes.mc202_phrase_ref.as_deref(),
        Some("answer-scene-1")
    );
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("generated MC-202 answer phrase answer-scene-1 at 0.82")
    );
}

#[test]
fn committed_mc202_pressure_generation_updates_phrase_ref_touch_and_render_shape() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_mc202_generate_pressure(300),
        QueueControlResult::Enqueued
    );

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 36,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state.session.runtime_state.lane_state.mc202.role.as_deref(),
        Some("pressure")
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .mc202
            .phrase_ref
            .as_deref(),
        Some("pressure-scene-1")
    );
    assert_eq!(state.session.runtime_state.macro_state.mc202_touch, 0.84);
    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Pressure);
    assert_eq!(
        state.runtime.mc202_render.phrase_shape,
        Mc202PhraseShape::PressureCell
    );
    assert_eq!(
        state.runtime.mc202_render.note_budget,
        riotbox_audio::mc202::Mc202NoteBudget::Sparse
    );
    assert_eq!(
        state.runtime.mc202_render.routing,
        Mc202RenderRouting::MusicBusBass
    );
    assert_eq!(state.jam_view.lanes.mc202_role.as_deref(), Some("pressure"));
    assert!(!state.jam_view.lanes.mc202_pending_pressure_generation);
    assert_eq!(
        state.jam_view.lanes.mc202_phrase_ref.as_deref(),
        Some("pressure-scene-1")
    );
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("generated MC-202 pressure phrase pressure-scene-1 at 0.84")
    );
}
