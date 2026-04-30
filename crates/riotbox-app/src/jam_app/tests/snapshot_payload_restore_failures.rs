fn unsupported_promote_resample_action(id: u64) -> Action {
    Action {
        id: ActionId(id),
        actor: ActorType::User,
        command: ActionCommand::PromoteResample,
        params: ActionParams::Empty,
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
            summary: "promote-resample committed".into(),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: Some("unsupported artifact-producing action".into()),
    }
}

fn loop_freeze_commit_record(action_id: u64) -> ActionCommitRecord {
    ActionCommitRecord {
        action_id: ActionId(action_id),
        boundary: CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 40,
            bar_index: 10,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        commit_sequence: 1,
        committed_at: 500,
    }
}

#[test]
fn app_snapshot_payload_restore_rejects_missing_payload_without_mutating_state() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    let original_session = state.session.clone();
    let original_runtime_view = state.runtime_view.clone();

    let error = state
        .apply_restore_target_from_snapshot_payload(state.session.action_log.actions.len())
        .expect_err("missing snapshot payload should reject at app boundary");

    assert_eq!(
        error,
        riotbox_core::replay::SnapshotPayloadHydrationError::MissingSnapshotPayload {
            snapshot_id: "snap-1".into(),
        }
    );
    assert_eq!(
        state.session, original_session,
        "missing payload restore must not mutate app session"
    );
    assert_eq!(state.runtime_view, original_runtime_view);
    assert_eq!(
        state.runtime_view.replay_restore_payload,
        "payload missing | snapshot restore blocked"
    );
}

#[test]
fn app_snapshot_payload_restore_rejects_unsupported_suffix_without_mutating_state() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    let snapshot = session.snapshots[0].clone();
    session.snapshots[0].payload = Some(riotbox_core::session::SnapshotPayload::from_runtime_state(
        &snapshot.snapshot_id,
        snapshot.action_cursor,
        &session.runtime_state,
    ));
    session
        .action_log
        .actions
        .push(unsupported_promote_resample_action(77));
    session
        .action_log
        .commit_records
        .push(loop_freeze_commit_record(77));

    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    let original_session = state.session.clone();
    let original_runtime_view = state.runtime_view.clone();
    let target_cursor = state.session.action_log.actions.len();

    let error = state
        .apply_restore_target_from_snapshot_payload(target_cursor)
        .expect_err("unsupported suffix should reject after payload hydration");

    assert!(matches!(
        error,
        riotbox_core::replay::SnapshotPayloadHydrationError::Execution(
            riotbox_core::replay::ReplayTargetExecutionError::Execution(
                riotbox_core::replay::ReplayExecutionError::UnsupportedAction {
                    action_id: ActionId(77),
                    command: ActionCommand::PromoteResample,
                }
            )
        )
    ));
    assert_eq!(
        state.session, original_session,
        "unsupported payload restore suffix must not mutate app session"
    );
    assert_eq!(state.runtime_view, original_runtime_view);
    assert_eq!(
        state.runtime_view.replay_restore_payload,
        "payload ready | snapshot restore ok"
    );
    assert_eq!(
        state.runtime_view.replay_restore_status,
        "blocked: 1 unsupported suffix action(s)"
    );
    assert_eq!(
        state.runtime_view.replay_restore_unsupported,
        "unsupported suffix 1: promote.resample"
    );
}

#[test]
fn app_snapshot_payload_restore_hydrates_plannable_w30_artifact_suffix() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    let snapshot = session.snapshots[0].clone();
    session.snapshots[0].payload = Some(riotbox_core::session::SnapshotPayload::from_runtime_state(
        &snapshot.snapshot_id,
        snapshot.action_cursor,
        &session.runtime_state,
    ));
    session
        .action_log
        .actions
        .push(unsupported_loop_freeze_promotion_action(78));
    session
        .action_log
        .commit_records
        .push(loop_freeze_commit_record(78));
    session.captures.push(loop_freeze_capture_for_action(78));

    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    let target_cursor = state.session.action_log.actions.len();
    let target_plan = riotbox_core::replay::build_replay_target_plan(
        &state.session.action_log,
        &state.session.snapshots,
        target_cursor,
    )
    .expect("build target plan");
    let suffix_entry = target_plan
        .suffix
        .last()
        .expect("artifact-producing suffix entry");
    let hydration_plan =
        riotbox_core::replay::plan_w30_artifact_replay_hydration(&state.session, suffix_entry)
            .expect("explicit W-30 artifact identity is plannable");
    assert_eq!(hydration_plan.produced_capture_id, CaptureId::from("cap-02"));
    assert_eq!(hydration_plan.source_capture_id, CaptureId::from("cap-01"));

    let report = state
        .apply_restore_target_from_snapshot_payload(target_cursor)
        .expect("planned loop-freeze artifact suffix hydrates W-30 preview state");

    assert_eq!(
        report.applied_action_ids,
        vec![ActionId(78)],
        "restore must execute only the loop-freeze suffix"
    );
    assert_eq!(
        state.session.runtime_state.lane_state.w30.last_capture,
        Some(CaptureId::from("cap-02"))
    );
    assert_eq!(state.runtime_view.w30_preview_mode, "live_recall");
    assert_eq!(state.runtime_view.w30_preview_profile, "pinned_recall");
}

fn unsupported_loop_freeze_promotion_action(id: u64) -> Action {
    Action {
        id: ActionId(id),
        actor: ActorType::User,
        command: ActionCommand::W30LoopFreeze,
        params: ActionParams::Promotion {
            capture_id: Some(CaptureId::from("cap-01")),
            destination: Some("w30:loop_freeze".into()),
        },
        target: ActionTarget {
            scope: Some(TargetScope::LaneW30),
            bank_id: Some(BankId::from("bank-a")),
            pad_id: Some(PadId::from("pad-01")),
            ..Default::default()
        },
        requested_at: 480,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(500),
        result: Some(ActionResult {
            accepted: true,
            summary: "loop-freeze committed".into(),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: Some("artifact-producing W-30 action".into()),
    }
}

fn loop_freeze_capture_for_action(action_id: u64) -> CaptureRef {
    CaptureRef {
        capture_id: CaptureId::from("cap-02"),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["source-1".into()],
        source_window: None,
        lineage_capture_refs: vec![CaptureId::from("cap-01")],
        resample_generation_depth: 0,
        created_from_action: Some(ActionId(action_id)),
        storage_path: "captures/cap-02.wav".into(),
        assigned_target: Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        }),
        is_pinned: true,
        notes: None,
    }
}
