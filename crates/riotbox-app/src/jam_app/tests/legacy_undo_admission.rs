//! Public generated metadata Undo admission, without source or audio hydration.
use std::{fs, path::Path};

use crate::jam_app::{JamAppError, JamAppState};
use riotbox_core::{
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionStatus, ActionTarget, ActorType,
        CommitBoundary, Quantization, SourceMonitorMode, UndoPolicy,
    },
    ids::ActionId,
    persistence::save_session_json,
    replay::build_committed_replay_plan,
    session::{ReplayPolicy, SessionFile},
};

#[test]
fn legacy_rollback_undo_refuses_without_changing_session_runtime_queue_ids_or_file() {
    let owner = tempfile::tempdir().unwrap();
    let path = owner.path().join("generated-legacy.json");
    let mut session = SessionFile::new("generated-legacy-undo", "test", "2026-10-16");
    session.action_log.replay_policy = ReplayPolicy::DeterministicPreferred;
    save_session_json(&path, &session).unwrap();
    let mut app = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
    app.queue_source_monitor_mode(SourceMonitorMode::Blend, 900)
        .unwrap();
    let boundary = app
        .runtime
        .transport
        .boundary_state(CommitBoundary::Immediate);
    assert_eq!(app.commit_ready_actions(boundary, 1000).unwrap().len(), 1);
    app.save().unwrap();
    let mut app = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
    assert_eq!(
        app.session
            .runtime_state
            .undo_state
            .source_monitor_snapshots
            .len(),
        1
    );
    app.queue
        .enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::TransportPause,
                Quantization::NextBar,
                ActionTarget::default(),
            ),
            1100,
        )
        .unwrap();
    let before_session = app.session.clone();
    let before_runtime = app.runtime.clone();
    let before_view = app.jam_view.clone();
    let before_runtime_view = app.runtime_view.clone();
    let pending: Vec<_> = app.queue.pending_actions().into_iter().cloned().collect();
    let history = app.queue.history().to_vec();
    let next_id = app.queue.clone().allocate_action_id();
    let saved = fs::read(&path).unwrap();

    let result = app.undo_last_action(500);
    assert!(
        matches!(
            result,
            Err(JamAppError::ReplayAdmission {
                policy: ReplayPolicy::DeterministicPreferred,
                ..
            })
        ),
        "legacy timestamp rollback must be refused before mutation, got {result:?}"
    );
    assert_eq!(app.session, before_session);
    assert_eq!(app.runtime, before_runtime);
    assert_eq!(app.jam_view, before_view);
    assert_eq!(app.runtime_view, before_runtime_view);
    assert_eq!(
        app.queue
            .pending_actions()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>(),
        pending
    );
    assert_eq!(app.queue.history(), history);
    assert_eq!(app.queue.clone().allocate_action_id(), next_id);
    assert_eq!(fs::read(&path).unwrap(), saved);
}

fn loaded_monitor(path: &Path, policy: ReplayPolicy) -> JamAppState {
    let mut session = SessionFile::new("generated-undo-control", "test", "2026-10-16");
    session.action_log.replay_policy = policy;
    save_session_json(path, &session).unwrap();
    let mut app = JamAppState::from_json_files(path, None::<&Path>).unwrap();
    app.queue_source_monitor_mode(SourceMonitorMode::Blend, 900)
        .unwrap();
    commit(&mut app, 1000);
    app.save().unwrap();
    JamAppState::from_json_files(path, None::<&Path>).unwrap()
}

fn commit(app: &mut JamAppState, at: u64) {
    let boundary = app
        .runtime
        .transport
        .boundary_state(CommitBoundary::Immediate);
    assert_eq!(app.commit_ready_actions(boundary, at).unwrap().len(), 1);
}

fn assert_refusal_unchanged(
    app: &mut JamAppState,
    path: &Path,
    requested_at: u64,
    expected: impl FnOnce(&JamAppError) -> bool,
) {
    let session = app.session.clone();
    let runtime = app.runtime.clone();
    let view = app.jam_view.clone();
    let runtime_view = app.runtime_view.clone();
    let pending: Vec<_> = app.queue.pending_actions().into_iter().cloned().collect();
    let history = app.queue.history().to_vec();
    let next_id = app.queue.clone().allocate_action_id();
    let saved = fs::read(path).unwrap();
    let error = app.undo_last_action(requested_at).unwrap_err();
    assert!(expected(&error), "unexpected refusal: {error:?}");
    assert_eq!(app.session, session);
    assert_eq!(app.runtime, runtime);
    assert_eq!(app.jam_view, view);
    assert_eq!(app.runtime_view, runtime_view);
    assert_eq!(
        app.queue
            .pending_actions()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>(),
        pending
    );
    assert_eq!(app.queue.history(), history);
    assert_eq!(app.queue.clone().allocate_action_id(), next_id);
    assert_eq!(fs::read(path).unwrap(), saved);
}

#[test]
fn forward_legacy_and_rollback_v2_undo_keep_exact_marker_and_roundtrip() {
    let owner = tempfile::tempdir().unwrap();
    for (policy, at) in [
        (ReplayPolicy::DeterministicPreferred, 1100),
        (ReplayPolicy::SnapshotOnly, 1100),
        (ReplayPolicy::DeterministicCommittedOrderV2, 500),
    ] {
        let path = owner.path().join(format!("{policy:?}.json"));
        let mut app = loaded_monitor(&path, policy);
        let target_id = app.session.action_log.actions[0].id;
        let next_id = app.queue.clone().allocate_action_id().unwrap();
        let marker = app.undo_last_action(at).unwrap().unwrap();
        assert_eq!(marker.id, next_id);
        assert_eq!(marker.requested_at, at);
        assert_eq!(marker.committed_at, Some(at));
        assert_eq!(
            marker.params,
            ActionParams::Undo {
                target_action_id: target_id
            }
        );
        assert_eq!(
            app.session.action_log.actions[0].status,
            ActionStatus::Undone
        );
        assert_eq!(
            app.session
                .action_log
                .commit_records
                .last()
                .unwrap()
                .committed_at,
            at
        );
        assert!(
            app.session
                .runtime_state
                .undo_state
                .source_monitor_snapshots
                .is_empty()
        );
        assert_eq!(
            app.session.runtime_state.source_monitor.mode,
            SourceMonitorMode::Source
        );
        assert_eq!(app.session.action_log.replay_policy, policy);
        build_committed_replay_plan(&app.session.action_log).unwrap();
        app.save().unwrap();
        let mut loaded = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
        assert_eq!(loaded.session, app.session);
        let next_id = loaded.queue.clone().allocate_action_id();
        assert_eq!(loaded.undo_last_action(1).unwrap(), None);
        assert_eq!(loaded.queue.clone().allocate_action_id(), next_id);
        assert_eq!(loaded.session, app.session);
    }
}

#[test]
fn snapshot_only_rollback_is_admission_refusal_not_implicit_upgrade() {
    let owner = tempfile::tempdir().unwrap();
    let path = owner.path().join("snapshot-only.json");
    let mut app = loaded_monitor(&path, ReplayPolicy::SnapshotOnly);
    assert_refusal_unchanged(&mut app, &path, 500, |error| {
        matches!(
            error,
            JamAppError::ReplayAdmission {
                policy: ReplayPolicy::SnapshotOnly,
                ..
            }
        )
    });
}

#[test]
fn legacy_capacity_and_id_failures_precede_undo_admission_without_mutation() {
    let owner = tempfile::tempdir().unwrap();
    for policy in [
        ReplayPolicy::DeterministicPreferred,
        ReplayPolicy::SnapshotOnly,
    ] {
        let path = owner.path().join(format!("capacity-{policy:?}.json"));
        let mut app = loaded_monitor(&path, policy);
        app.session.action_log.commit_records[0].commit_sequence = u32::MAX;
        app.save().unwrap();
        let mut app = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
        app.queue.reserve_action_ids_after(Some(ActionId(u64::MAX)));
        assert_refusal_unchanged(&mut app, &path, 500, |error| {
            matches!(error, JamAppError::CommitSequenceExhausted(_))
        });

        let path = owner.path().join(format!("id-{policy:?}.json"));
        let mut app = loaded_monitor(&path, policy);
        app.queue.reserve_action_ids_after(Some(ActionId(u64::MAX)));
        assert_refusal_unchanged(&mut app, &path, 500, |error| {
            matches!(error, JamAppError::ActionIdExhausted(_))
        });
    }
}

#[test]
fn independent_domain_undo_is_v2_only_when_legacy_global_stack_disagrees() {
    let owner = tempfile::tempdir().unwrap();
    for policy in [
        ReplayPolicy::DeterministicPreferred,
        ReplayPolicy::SnapshotOnly,
        ReplayPolicy::DeterministicCommittedOrderV2,
    ] {
        let path = owner.path().join(format!("domains-{policy:?}.json"));
        let mut app = loaded_monitor(&path, policy);
        let monitor_id = app.session.action_log.actions[0].id;
        // Metadata-only Fill operations exercise another typed Undo domain.
        // The non-undoable second Fill blocks restoration of the first Fill,
        // not the independent earlier Source Monitor action.
        for (at, undoable) in [(1100, true), (1200, false)] {
            let mut draft = ActionDraft::new(
                ActorType::User,
                ActionCommand::Tr909FillNext,
                Quantization::Immediate,
                ActionTarget::default(),
            );
            if !undoable {
                draft.undo_policy = UndoPolicy::NotUndoable {
                    reason: "generated domain barrier".into(),
                };
            }
            app.queue.enqueue(draft, at).unwrap();
            commit(&mut app, at);
        }
        app.save().unwrap();
        let mut app = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
        if policy.uses_committed_append_order() {
            let fill_state = app.session.runtime_state.lane_state.tr909.clone();
            let marker = app.undo_last_action(1300).unwrap().unwrap();
            assert_eq!(
                marker.params,
                ActionParams::Undo {
                    target_action_id: monitor_id
                }
            );
            assert_eq!(
                app.session.runtime_state.source_monitor.mode,
                SourceMonitorMode::Source
            );
            assert_eq!(app.session.runtime_state.lane_state.tr909, fill_state);
            app.save().unwrap();
            assert_eq!(
                JamAppState::from_json_files(&path, None::<&Path>)
                    .unwrap()
                    .session,
                app.session
            );
        } else {
            assert_refusal_unchanged(&mut app, &path, 1300, |error| {
                matches!(error,
                JamAppError::ReplayAdmission { policy: actual, .. } if *actual == policy)
            });
        }
    }
}

#[test]
fn refused_legacy_undo_does_not_leak_staged_missing_snapshot_normalization() {
    let owner = tempfile::tempdir().unwrap();
    let path = owner.path().join("staged-normalization.json");
    let mut app = loaded_monitor(&path, ReplayPolicy::DeterministicPreferred);
    app.queue_source_monitor_mode(SourceMonitorMode::Riotbox, 1100)
        .unwrap();
    commit(&mut app, 1200);
    // A public Session metadata edit removes an older snapshot. The newer
    // trusted target remains selectable; normalization belongs to its staged
    // transaction and must not leak when Core refuses the rollback marker.
    app.session
        .runtime_state
        .undo_state
        .source_monitor_snapshots
        .remove(0);
    app.save().unwrap();
    assert_refusal_unchanged(&mut app, &path, 500, |error| {
        matches!(error, JamAppError::ReplayAdmission { .. })
    });
    assert!(matches!(
        app.session.action_log.actions[0].undo_policy,
        UndoPolicy::Undoable
    ));
}

#[test]
fn legacy_no_target_still_normalizes_before_returning_none_despite_full_capacities() {
    let owner = tempfile::tempdir().unwrap();
    let path = owner.path().join("no-target.json");
    let mut app = loaded_monitor(&path, ReplayPolicy::DeterministicPreferred);
    app.session
        .runtime_state
        .undo_state
        .source_monitor_snapshots
        .clear();
    app.session.action_log.commit_records[0].commit_sequence = u32::MAX;
    app.queue.reserve_action_ids_after(Some(ActionId(u64::MAX)));
    let records = app.session.action_log.commit_records.clone();
    let next_id = app.queue.clone().allocate_action_id();
    assert_eq!(app.undo_last_action(500).unwrap(), None);
    assert!(matches!(
        app.session.action_log.actions[0].undo_policy,
        UndoPolicy::NotUndoable { .. }
    ));
    assert_eq!(app.session.action_log.actions.len(), 1);
    assert_eq!(app.session.action_log.commit_records, records);
    assert_eq!(app.queue.clone().allocate_action_id(), next_id);
    app.save().unwrap();
    assert_eq!(
        JamAppState::from_json_files(&path, None::<&Path>)
            .unwrap()
            .session,
        app.session
    );
}
