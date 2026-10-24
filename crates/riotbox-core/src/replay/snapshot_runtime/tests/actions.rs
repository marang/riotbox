//! Action ownership, append-prefix chronology, and transactional selected-anchor admission.
use super::fixtures::{
    ACTION_FIELDS, action_error, action_session, assert_refused, owner, payload, session,
    set_action_reference, snapshot,
};
use crate::{
    action::{
        ActionCommand, ActionParams, ActionStatus, CaptureLengthIntent, CommitBoundary,
        Quantization, SourceMonitorMode, UndoPolicy,
    },
    ids::ActionId,
    replay::{
        ReplayPlanError, SnapshotPayloadHydrationError, SnapshotRuntimeActionError as ActionError,
        SnapshotRuntimeField as Field, hydrate_replay_target_from_snapshot_payload as hydrate,
        hydrate_replay_target_from_snapshot_payload_with_mc202_legacy_resolver,
    },
    session::{ReplayPolicy, RuntimeState},
};
use std::cell::Cell;

#[test]
fn all_eight_action_paths_admit_prefix_owners_and_refuse_missing_future_and_wrong_owners() {
    for field in ACTION_FIELDS {
        let valid = action_session(field, 1, ActionId(900));
        let restored = hydrate(&valid, 1, None).unwrap();
        assert_eq!(
            restored.session.runtime_state,
            valid.snapshots[0].payload.as_ref().unwrap().runtime_state,
            "{field:?}"
        );
        assert!(restored.replay_report.applied_action_ids.is_empty());

        assert_refused(
            &action_session(field, 1, ActionId(444)),
            1,
            None,
            action_error(field, 444, ActionError::Missing),
        );
        assert_refused(
            &action_session(field, 0, ActionId(900)),
            1,
            None,
            action_error(
                field,
                900,
                ActionError::AfterAnchor {
                    action_index: 0,
                    action_cursor: 0,
                },
            ),
        );
        let mut wrong = valid;
        wrong.action_log.actions[0].command = ActionCommand::TransportPlay;
        wrong.action_log.actions[0].params = ActionParams::Empty;
        assert_refused(
            &wrong,
            1,
            None,
            action_error(field, 900, ActionError::WrongOwner),
        );
    }
}

#[test]
fn all_eight_action_paths_refuse_rejected_and_uncommitted_owners() {
    for field in ACTION_FIELDS {
        let mut rejected = action_session(field, 1, ActionId(900));
        rejected.action_log.actions[0]
            .result
            .as_mut()
            .unwrap()
            .accepted = false;
        assert_refused(
            &rejected,
            1,
            None,
            action_error(field, 900, ActionError::NotCommitted),
        );
        let mut pending = action_session(field, 1, ActionId(900));
        pending.action_log.actions[0].status = ActionStatus::Requested;
        pending.action_log.actions[0].committed_at = None;
        pending.action_log.commit_records.clear();
        assert_refused(
            &pending,
            1,
            None,
            action_error(field, 900, ActionError::NotCommitted),
        );
    }
}

#[test]
fn legacy_partial_record_coverage_does_not_weaken_v2_or_commit_status_admission() {
    let mut legacy = action_session(Field::CaptureLength, 1, ActionId(900));
    legacy.action_log.replay_policy = ReplayPolicy::DeterministicPreferred;
    legacy.action_log.commit_records.clear();
    hydrate(&legacy, 1, None).expect("legacy prefix owners may lack a commit record");

    let mut v2 = legacy.clone();
    v2.action_log.replay_policy = ReplayPolicy::DeterministicCommittedOrderV2;
    let before = v2.clone();
    assert_eq!(
        hydrate(&v2, 1, None).unwrap_err(),
        SnapshotPayloadHydrationError::Plan(ReplayPlanError::MissingCommittedActionRecord {
            action_id: ActionId(900)
        })
    );
    assert_eq!(v2, before);

    legacy.action_log.actions[0].committed_at = None;
    assert_refused(
        &legacy,
        1,
        None,
        action_error(Field::CaptureLength, 900, ActionError::NotCommitted),
    );

    let mut unsupported = action_session(Field::CaptureLength, 1, ActionId(900));
    unsupported.action_log.replay_policy = ReplayPolicy::DeterministicPreferred;
    unsupported.action_log.commit_records.clear();
    unsupported.action_log.actions[0].command = ActionCommand::CaptureLoop;
    unsupported.action_log.actions[0].status = ActionStatus::Undone;
    let before = unsupported.clone();
    // Existing cursor safety refuses this unresolved unsupported Undo before
    // hydration; legacy partial records cannot make the unsafe anchor usable.
    assert_eq!(
        hydrate(&unsupported, 1, None).unwrap_err(),
        SnapshotPayloadHydrationError::MissingSnapshotAnchor {
            target_action_cursor: 1
        }
    );
    assert_eq!(unsupported, before);
}

#[test]
fn safe_post_marker_anchor_accepts_supported_typed_undone_provenance() {
    let mut undone = owner(Field::SourceMonitorUndo, 900);
    undone.status = ActionStatus::Undone;
    let mut marker = owner(Field::CaptureLength, 3);
    marker.command = ActionCommand::UndoLast;
    marker.params = ActionParams::Undo {
        target_action_id: undone.id,
    };
    marker.quantization = Quantization::Immediate;
    marker.undo_policy = UndoPolicy::NotUndoable {
        reason: "generated typed marker".into(),
    };
    let mut runtime = RuntimeState::default();
    runtime.source_monitor.mode = SourceMonitorMode::Source;
    // This retained pre-action metadata refers to a real, now-undone prefix
    // owner. Admission does not impose whole-runtime replay equality or make
    // an already-undone action a new live Undo candidate.
    set_action_reference(&mut runtime, Field::SourceMonitorUndo, undone.id);
    let mut candidate = session(vec![undone, marker], 2, runtime.clone());
    candidate.action_log.commit_records[1].boundary.kind = CommitBoundary::Immediate;
    let restored = hydrate(&candidate, 2, None).expect("trusted post-marker anchor is safe");
    assert!(restored.replay_report.applied_action_ids.is_empty());
    assert_eq!(restored.session.runtime_state, runtime);
}

#[test]
fn prefix_uses_anchor_vector_position_not_target_numeric_id_or_timestamp() {
    for ids in [[900, 3], [3, 900]] {
        let mut runtime = RuntimeState::default();
        set_action_reference(&mut runtime, Field::CaptureLength, ActionId(ids[0]));
        let mut candidate = session(
            ids.into_iter()
                .map(|id| owner(Field::CaptureLength, id))
                .collect(),
            1,
            runtime,
        );
        let restored = hydrate(&candidate, 2, None).unwrap();
        assert_eq!(restored.replay_report.anchor_action_cursor, Some(1));
        assert_eq!(
            restored.replay_report.applied_action_ids,
            vec![ActionId(ids[1])]
        );
        payload(&mut candidate).capture.length_set_by_action = Some(ActionId(ids[1]));
        assert_refused(
            &candidate,
            2,
            None,
            action_error(
                Field::CaptureLength,
                ids[1],
                ActionError::AfterAnchor {
                    action_index: 1,
                    action_cursor: 1,
                },
            ),
        );
    }
}

#[test]
fn owner_identity_checks_do_not_accept_a_different_timing_length_or_scene_target() {
    let mut timing = action_session(Field::TimingConfirmation, 1, ActionId(900));
    timing.action_log.actions[0].params = ActionParams::SourceTimingGrid {
        source_id: Some("different-source".into()),
        hypothesis_id: None,
        confirmed_bpm: Some(126.0),
    };
    assert_refused(
        &timing,
        1,
        None,
        action_error(Field::TimingConfirmation, 900, ActionError::WrongOwner),
    );
    let mut length = action_session(Field::CaptureLength, 1, ActionId(900));
    length.action_log.actions[0].params = ActionParams::CaptureLength {
        intent: Some(CaptureLengthIntent::Phrase),
    };
    assert_refused(
        &length,
        1,
        None,
        action_error(Field::CaptureLength, 900, ActionError::WrongOwner),
    );
    for field in [
        Field::LastSceneMovement,
        Field::ActiveSceneProjection,
        Field::RestoreSceneProjection,
    ] {
        let mut scene = action_session(field, 1, ActionId(900));
        scene.action_log.actions[0].params = ActionParams::Scene {
            scene_id: Some("wrong-destination".into()),
        };
        assert_refused(
            &scene,
            1,
            None,
            action_error(field, 900, ActionError::WrongOwner),
        );
    }
}

#[test]
fn selected_only_validation_refuses_without_other_anchor_fallback_or_suffix_resolution() {
    let mut candidate = action_session(Field::CaptureLength, 1, ActionId(900));
    let mut irrelevant = RuntimeState::default();
    set_action_reference(&mut irrelevant, Field::CaptureLength, ActionId(444));
    candidate
        .snapshots
        .insert(0, snapshot("older-invalid", 0, &irrelevant));
    hydrate(&candidate, 1, None).unwrap();

    // A usable older anchor must not become an automatic repair for the selected one.
    candidate.snapshots[0] = snapshot("older-valid", 0, &RuntimeState::default());
    payload(&mut candidate).capture.length_set_by_action = Some(ActionId(444));
    let suffix = owner(Field::Mc202Undo, 3);
    let runtime = candidate.snapshots[1]
        .payload
        .as_ref()
        .unwrap()
        .runtime_state
        .clone();
    let mut with_suffix = session(vec![owner(Field::CaptureLength, 900), suffix], 1, runtime);
    with_suffix
        .snapshots
        .insert(0, candidate.snapshots[0].clone());
    let calls = Cell::new(0);
    payload(&mut with_suffix).capture.length_set_by_action = Some(ActionId(900));
    hydrate_replay_target_from_snapshot_payload_with_mc202_legacy_resolver(
        &with_suffix,
        2,
        None,
        |_, _| {
            calls.set(calls.get() + 1);
            None
        },
    )
    .expect("valid paired control actually invokes legacy suffix resolution");
    assert_eq!(calls.get(), 1);
    calls.set(0);
    payload(&mut with_suffix).capture.length_set_by_action = Some(ActionId(444));
    let before = with_suffix.clone();
    let error = hydrate_replay_target_from_snapshot_payload_with_mc202_legacy_resolver(
        &with_suffix,
        2,
        None,
        |_, _| {
            calls.set(calls.get() + 1);
            None
        },
    )
    .unwrap_err();
    assert_eq!(
        error,
        SnapshotPayloadHydrationError::RuntimeProvenance {
            snapshot_id: "selected".into(),
            reason: action_error(Field::CaptureLength, 444, ActionError::Missing),
        }
    );
    assert_eq!(
        calls.get(),
        0,
        "invalid anchors must never replan legacy suffix entries"
    );
    assert_eq!(with_suffix, before);
}
