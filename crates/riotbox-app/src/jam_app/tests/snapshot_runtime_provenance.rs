use crate::jam_app::state::JamAppState;
use riotbox_core::{
    action::{
        Action, ActionCommand, ActionDraft, ActionParams, ActionResult, ActionStatus, ActionTarget,
        ActorType, CommitBoundary, Quantization, UndoPolicy,
    },
    ids::{ActionId, SnapshotId, SourceId},
    queue::ActionQueue,
    replay::{
        SnapshotPayloadHydrationError, SnapshotRuntimeActionError, SnapshotRuntimeField,
        SnapshotRuntimeValidationError,
    },
    session::{
        ActionCommitRecord, SessionFile, Snapshot, SnapshotPayload, SourceRef,
        SourceTimingGridConfirmationState,
    },
    source_graph::DecodeProfile,
    transport::CommitBoundaryState,
};

fn snapshot_session(future_confirmation: bool) -> SessionFile {
    let mut session = SessionFile::new("generated-provenance", "test", "2026-10-24");
    let source_id = SourceId::from("generated-source");
    session.source_refs.push(SourceRef {
        source_id: source_id.clone(),
        path_hint: "generated-metadata-only-never-opened.wav".into(),
        content_hash: "generated-test-identity".into(),
        duration_seconds: 10.0,
        decode_profile: DecodeProfile::Native,
    });
    session.action_log.actions.push(Action {
        id: ActionId(1),
        actor: ActorType::User,
        command: ActionCommand::SourceTimingConfirmGrid,
        params: ActionParams::SourceTimingGrid {
            source_id: Some(source_id.clone()),
            hypothesis_id: None,
            confirmed_bpm: Some(128.0),
        },
        target: ActionTarget::default(),
        requested_at: 10,
        quantization: Quantization::Immediate,
        status: ActionStatus::Committed,
        committed_at: Some(20),
        result: Some(ActionResult {
            accepted: true,
            summary: "generated confirmation".into(),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: None,
    });
    session.action_log.commit_records.push(ActionCommitRecord {
        action_id: ActionId(1),
        boundary: CommitBoundaryState {
            kind: CommitBoundary::Immediate,
            beat_index: 0,
            bar_index: 0,
            phrase_index: 0,
            scene_id: None,
        },
        commit_sequence: 1,
        committed_at: 20,
        mc202_source_phrase_plan: None,
    });
    let mut runtime = session.runtime_state.clone();
    if future_confirmation {
        runtime.source_timing.confirmed_grid = Some(SourceTimingGridConfirmationState {
            source_id,
            hypothesis_id: None,
            confirmed_by_action: ActionId(1),
            confirmed_at: 20,
        });
        runtime.source_timing.confirmed_bpm = Some(128.0);
    }
    let snapshot_id = SnapshotId::from("anchor-zero");
    session.snapshots.push(Snapshot {
        snapshot_id: snapshot_id.clone(),
        created_at: "2026-10-24".into(),
        label: "generated zero cursor".into(),
        action_cursor: 0,
        payload: Some(SnapshotPayload::from_runtime_state(
            &snapshot_id,
            0,
            &runtime,
        )),
    });
    session
}

#[test]
fn app_snapshot_payload_rejects_after_cursor_provenance_without_mutation() {
    let mut app = JamAppState::from_parts(snapshot_session(true), None, ActionQueue::new());
    app.queue
        .enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::Tr909FillNext,
                Quantization::NextBar,
                ActionTarget::default(),
            ),
            30,
        )
        .unwrap();
    let before = app.clone();

    let result = app.apply_restore_target_from_snapshot_payload(0);

    assert_eq!(
        result.unwrap_err(),
        SnapshotPayloadHydrationError::RuntimeProvenance {
            snapshot_id: "anchor-zero".into(),
            reason: SnapshotRuntimeValidationError::Action {
                field: SnapshotRuntimeField::TimingConfirmation,
                action_id: ActionId(1),
                reason: SnapshotRuntimeActionError::AfterAnchor {
                    action_index: 0,
                    action_cursor: 0
                },
            },
        }
    );
    assert_eq!(app.session, before.session);
    assert_eq!(app.runtime, before.runtime);
    assert_eq!(app.jam_view, before.jam_view);
    assert_eq!(app.runtime_view, before.runtime_view);
    assert_eq!(app.queue.history(), before.queue.history());
    assert_eq!(app.queue.pending_actions(), before.queue.pending_actions());
    assert_eq!(
        app.queue.clone().allocate_action_id(),
        before.queue.clone().allocate_action_id()
    );
}

#[test]
fn app_snapshot_payload_valid_prefix_and_suffix_use_existing_restore() {
    let mut session = snapshot_session(true);
    session.snapshots[0].action_cursor = 1;
    session.snapshots[0].payload.as_mut().unwrap().action_cursor = 1;
    let mut app = JamAppState::from_parts(session, None, ActionQueue::new());
    assert!(
        app.apply_restore_target_from_snapshot_payload(1)
            .unwrap()
            .applied_action_ids
            .is_empty()
    );
    assert_eq!(
        app.session
            .runtime_state
            .source_timing
            .confirmed_grid
            .as_ref()
            .unwrap()
            .confirmed_by_action,
        ActionId(1)
    );

    // The same accepted action can also be replayed by the ordinary suffix from
    // an honest zero-cursor payload; validation must not reject later history.
    let mut app = JamAppState::from_parts(snapshot_session(false), None, ActionQueue::new());
    assert_eq!(
        app.apply_restore_target_from_snapshot_payload(1)
            .unwrap()
            .applied_action_ids,
        vec![ActionId(1)]
    );
    assert_eq!(
        app.session.runtime_state.source_timing.confirmed_bpm,
        Some(128.0)
    );
}

#[test]
fn app_snapshot_payload_uses_anchor_not_target_and_never_falls_back() {
    let mut session = snapshot_session(false);
    let mut invalid = session.snapshots[0].clone();
    invalid.snapshot_id = SnapshotId::from("selected-corrupt");
    invalid.payload = snapshot_session(true).snapshots[0].payload.clone();
    invalid.payload.as_mut().unwrap().snapshot_id = invalid.snapshot_id.clone();
    session.snapshots.push(invalid);
    let mut app = JamAppState::from_parts(session, None, ActionQueue::new());
    let before = app.clone();
    assert!(matches!(app.apply_restore_target_from_snapshot_payload(1),
        Err(SnapshotPayloadHydrationError::RuntimeProvenance { snapshot_id, reason: SnapshotRuntimeValidationError::Action {
            reason: SnapshotRuntimeActionError::AfterAnchor { action_cursor: 0, .. }, .. } }) if snapshot_id == "selected-corrupt"));
    assert_eq!(app.session, before.session);
    assert_eq!(app.runtime, before.runtime);
    assert_eq!(app.runtime_view, before.runtime_view);
}

#[test]
fn app_snapshot_payload_rejects_missing_and_wrong_owner_provenance() {
    for missing in [true, false] {
        let mut session = snapshot_session(true);
        session.snapshots[0].action_cursor = 1;
        session.snapshots[0].payload.as_mut().unwrap().action_cursor = 1;
        let expected = if missing {
            session.snapshots[0]
                .payload
                .as_mut()
                .unwrap()
                .runtime_state
                .source_timing
                .confirmed_grid
                .as_mut()
                .unwrap()
                .confirmed_by_action = ActionId(999);
            SnapshotRuntimeActionError::Missing
        } else {
            session.action_log.actions[0].command = ActionCommand::TransportPlay;
            SnapshotRuntimeActionError::WrongOwner
        };
        let mut app = JamAppState::from_parts(session, None, ActionQueue::new());
        let before = app.clone();
        assert!(matches!(app.apply_restore_target_from_snapshot_payload(1),
            Err(SnapshotPayloadHydrationError::RuntimeProvenance { reason: SnapshotRuntimeValidationError::Action { reason, .. }, .. }) if reason == expected));
        assert_eq!(app.session, before.session);
        assert_eq!(app.runtime, before.runtime);
        assert_eq!(app.runtime_view, before.runtime_view);
    }
}

#[test]
fn app_snapshot_payload_accepts_initial_state_without_provenance() {
    let mut app = JamAppState::from_parts(snapshot_session(false), None, ActionQueue::new());
    let report = app.apply_restore_target_from_snapshot_payload(0).unwrap();
    assert!(report.applied_action_ids.is_empty());
    assert!(
        app.session
            .runtime_state
            .source_timing
            .confirmed_grid
            .is_none()
    );
}
