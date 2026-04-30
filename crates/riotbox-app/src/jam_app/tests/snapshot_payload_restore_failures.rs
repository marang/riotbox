fn unsupported_loop_freeze_action(id: u64) -> Action {
    Action {
        id: ActionId(id),
        actor: ActorType::User,
        command: ActionCommand::W30LoopFreeze,
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
            summary: "loop-freeze committed".into(),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: Some("artifact-producing W-30 action".into()),
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
        .push(unsupported_loop_freeze_action(77));
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
                    command: ActionCommand::W30LoopFreeze,
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
        "unsupported suffix 1: w30.loop_freeze"
    );
}
