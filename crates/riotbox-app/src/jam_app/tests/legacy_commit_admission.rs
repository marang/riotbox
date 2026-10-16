//! Generated public commit/transport admission, without source or host access.
use std::{fmt::Debug, fs, path::Path};

use riotbox_core::{
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType, CommitBoundary,
        Quantization, SourceMonitorMode,
    },
    persistence::{copy_session_with_committed_order, save_session_json},
    replay::build_committed_replay_plan,
    session::{ReplayPolicy, SessionFile},
};

use crate::jam_app::{JamAppError, JamAppState};

mod timing;
mod writers;

fn loaded_app(path: &Path, policy: ReplayPolicy) -> JamAppState {
    loaded_app_at(path, policy, 12.0)
}

fn loaded_app_at(path: &Path, policy: ReplayPolicy, position: f64) -> JamAppState {
    let mut session = SessionFile::new("generated-legacy-commit", "test", "2026-10-16");
    session.action_log.replay_policy = policy;
    session.runtime_state.transport.position_beats = position;
    save_session_json(path, &session).unwrap();
    JamAppState::from_json_files(path, None::<&Path>).unwrap()
}

fn enqueue(app: &mut JamAppState, command: ActionCommand, quantization: Quantization) {
    app.queue
        .enqueue(
            ActionDraft::new(
                ActorType::User,
                command,
                quantization,
                ActionTarget::default(),
            ),
            100,
        )
        .unwrap();
}

fn assert_refusal_unchanged<T: Debug>(
    app: &mut JamAppState,
    path: &Path,
    operation: impl FnOnce(&mut JamAppState) -> Result<T, JamAppError>,
) {
    assert_error_unchanged(app, path, operation, |error| {
        matches!(error, JamAppError::ReplayAdmission { .. })
    });
}

fn assert_error_unchanged<T: Debug>(
    app: &mut JamAppState,
    path: &Path,
    operation: impl FnOnce(&mut JamAppState) -> Result<T, JamAppError>,
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
    let error = operation(app).expect_err("unsupported legacy commit must refuse");
    assert!(expected(&error), "{error:?}");
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

fn commit_immediate(app: &mut JamAppState, at: u64) -> usize {
    let boundary = app
        .runtime
        .transport
        .boundary_state(CommitBoundary::Immediate);
    app.commit_ready_actions(boundary, at).unwrap().len()
}

#[test]
fn generic_legacy_stop_refuses_before_drain_or_transport_and_preserves_reloadable_save() {
    let owner = tempfile::tempdir().unwrap();
    let path = owner.path().join("legacy-stop.json");
    let mut app = loaded_app(&path, ReplayPolicy::DeterministicPreferred);
    enqueue(
        &mut app,
        ActionCommand::TransportStop,
        Quantization::Immediate,
    );
    assert_refusal_unchanged(&mut app, &path, |app| {
        let boundary = app
            .runtime
            .transport
            .boundary_state(CommitBoundary::Immediate);
        app.commit_ready_actions(boundary, 110)
    });
    app.save().unwrap();
    let loaded = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
    assert_eq!(loaded.session, app.session);
    assert_eq!(
        app.queue.pending_actions().len(),
        1,
        "successful enqueue remains pending"
    );
}

#[test]
fn unsafe_member_refuses_entire_mixed_ready_batch_without_relabeling_other_pending_intent() {
    let owner = tempfile::tempdir().unwrap();
    for policy in [
        ReplayPolicy::DeterministicPreferred,
        ReplayPolicy::SnapshotOnly,
    ] {
        for command in [ActionCommand::TransportStop, ActionCommand::TransportSeek] {
            let path = owner.path().join(format!("{policy:?}-{command:?}.json"));
            let mut app = loaded_app(&path, policy);
            app.queue_source_monitor_mode(SourceMonitorMode::Blend, 90)
                .unwrap();
            let mut draft = ActionDraft::new(
                ActorType::User,
                command,
                Quantization::Immediate,
                ActionTarget::default(),
            );
            if command == ActionCommand::TransportSeek {
                draft.params = ActionParams::Transport {
                    position_beats: Some(0),
                };
            }
            app.queue.enqueue(draft, 100).unwrap();
            enqueue(
                &mut app,
                ActionCommand::TransportPause,
                Quantization::NextBar,
            );
            assert_refusal_unchanged(&mut app, &path, |app| {
                let boundary = app
                    .runtime
                    .transport
                    .boundary_state(CommitBoundary::Immediate);
                app.commit_ready_actions(boundary, 110)
            });
            assert_eq!(app.queue.pending_actions().len(), 3);
            assert!(
                app.session
                    .runtime_state
                    .undo_state
                    .source_monitor_snapshots
                    .is_empty()
            );
        }
    }
}

#[test]
fn toggle_preflights_preexisting_unsafe_ready_work_without_consuming_its_own_id() {
    let owner = tempfile::tempdir().unwrap();
    for policy in [
        ReplayPolicy::DeterministicPreferred,
        ReplayPolicy::SnapshotOnly,
    ] {
        let path = owner.path().join(format!("{policy:?}-toggle.json"));
        let mut app = loaded_app(&path, policy);
        enqueue(
            &mut app,
            ActionCommand::TransportStop,
            Quantization::Immediate,
        );
        assert_refusal_unchanged(&mut app, &path, |app| app.commit_transport_toggle(110));
        assert_eq!(app.queue.pending_actions().len(), 1);
    }
}

#[test]
fn supported_legacy_navigation_and_v2_stop_monitor_roundtrip_without_policy_changes() {
    let owner = tempfile::tempdir().unwrap();
    for policy in [
        ReplayPolicy::DeterministicPreferred,
        ReplayPolicy::SnapshotOnly,
        ReplayPolicy::DeterministicCommittedOrderV2,
    ] {
        let path = owner.path().join(format!("{policy:?}-zero-stop.json"));
        let mut app = loaded_app_at(&path, policy, 0.0);
        enqueue(
            &mut app,
            ActionCommand::TransportStop,
            Quantization::Immediate,
        );
        assert_eq!(commit_immediate(&mut app, 110), 1);
        app.save().unwrap();
        let loaded = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
        assert_eq!(loaded.session, app.session);
        assert_eq!(loaded.session.action_log.replay_policy, policy);

        let path = owner.path().join(format!("{policy:?}-forward.json"));
        let mut app = loaded_app(&path, policy);
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::TransportSeek,
            Quantization::Immediate,
            ActionTarget::default(),
        );
        draft.params = ActionParams::Transport {
            position_beats: Some(16),
        };
        app.queue.enqueue(draft, 100).unwrap();
        assert_eq!(commit_immediate(&mut app, 110), 1);
        assert_eq!(app.runtime.transport.position_beats, 16.0);
        app.save().unwrap();
        assert_eq!(
            JamAppState::from_json_files(&path, None::<&Path>)
                .unwrap()
                .session,
            app.session
        );
    }
    let path = owner.path().join("v2-stop-monitor.json");
    let mut app = loaded_app(&path, ReplayPolicy::DeterministicCommittedOrderV2);
    enqueue(
        &mut app,
        ActionCommand::TransportStop,
        Quantization::Immediate,
    );
    assert_eq!(commit_immediate(&mut app, 110), 1);
    app.queue_source_monitor_mode(SourceMonitorMode::Blend, 120)
        .unwrap();
    assert_eq!(commit_immediate(&mut app, 90), 1);
    app.save().unwrap();
    let mut loaded = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
    assert_eq!(loaded.session, app.session);
    loaded.apply_restore_target_suffix(2).unwrap();
    assert_eq!(
        loaded.session.runtime_state.source_monitor.mode,
        SourceMonitorMode::Blend
    );
    assert_eq!(loaded.runtime.transport.position_beats, 0.0);
}

#[test]
fn refused_legacy_operation_can_be_retried_only_in_explicitly_upgraded_sibling() {
    let owner = tempfile::tempdir().unwrap();
    let original = owner.path().join("legacy.json");
    let copy = owner.path().join("v2.json");
    let mut app = loaded_app(&original, ReplayPolicy::DeterministicPreferred);
    enqueue(
        &mut app,
        ActionCommand::TransportStop,
        Quantization::Immediate,
    );
    assert_refusal_unchanged(&mut app, &original, |app| {
        let boundary = app
            .runtime
            .transport
            .boundary_state(CommitBoundary::Immediate);
        app.commit_ready_actions(boundary, 110)
    });
    let original_bytes = fs::read(&original).unwrap();
    let disposition = copy_session_with_committed_order(&original, &copy).unwrap();
    assert_eq!(
        disposition.previous_policy,
        ReplayPolicy::DeterministicPreferred
    );
    let mut copied = JamAppState::from_json_files(&copy, None::<&Path>).unwrap();
    assert_eq!(
        copied.session.action_log.replay_policy,
        ReplayPolicy::DeterministicCommittedOrderV2
    );
    enqueue(
        &mut copied,
        ActionCommand::TransportStop,
        Quantization::Immediate,
    );
    assert_eq!(commit_immediate(&mut copied, 110), 1);
    copied.save().unwrap();
    let restored = JamAppState::from_json_files(&copy, None::<&Path>).unwrap();
    assert_eq!(restored.session, copied.session);
    build_committed_replay_plan(&restored.session.action_log).unwrap();
    assert_eq!(fs::read(&original).unwrap(), original_bytes);
    assert_eq!(
        app.session.action_log.replay_policy,
        ReplayPolicy::DeterministicPreferred
    );
}

#[test]
fn historical_stop_only_copy_remains_supported_without_loosening_live_legacy_admission() {
    let owner = tempfile::tempdir().unwrap();
    let original = owner.path().join("historical-stop.json");
    let copy = owner.path().join("historical-stop-v2.json");
    let mut app = loaded_app(&original, ReplayPolicy::DeterministicCommittedOrderV2);
    enqueue(
        &mut app,
        ActionCommand::TransportStop,
        Quantization::Immediate,
    );
    assert_eq!(commit_immediate(&mut app, 110), 1);
    // A generated representation of a historical file, not a newly admitted
    // legacy operation and never an actual user Session migration.
    app.session.action_log.replay_policy = ReplayPolicy::DeterministicPreferred;
    save_session_json(&original, &app.session).unwrap();
    let before = fs::read(&original).unwrap();
    assert!(JamAppState::from_json_files(&original, None::<&Path>).is_err());
    copy_session_with_committed_order(&original, &copy).unwrap();
    let loaded = JamAppState::from_json_files(&copy, None::<&Path>).unwrap();
    assert_eq!(loaded.session.runtime_state.transport.position_beats, 0.0);
    assert_eq!(
        loaded.session.action_log.actions,
        app.session.action_log.actions
    );
    assert_eq!(
        loaded.session.action_log.commit_records,
        app.session.action_log.commit_records
    );
    assert_eq!(
        loaded.session.action_log.replay_policy,
        ReplayPolicy::DeterministicCommittedOrderV2
    );
    assert_eq!(fs::read(&original).unwrap(), before);
}

#[test]
fn batch_and_toggle_preserve_sequence_capacity_precedence_over_legacy_refusal() {
    let owner = tempfile::tempdir().unwrap();
    for toggle in [false, true] {
        let path = owner.path().join(format!("legacy-capacity-{toggle}.json"));
        let mut app = loaded_app(&path, ReplayPolicy::DeterministicPreferred);
        enqueue(
            &mut app,
            ActionCommand::TransportPause,
            Quantization::Immediate,
        );
        assert_eq!(commit_immediate(&mut app, 110), 1);
        app.session.action_log.commit_records[0].commit_sequence = u32::MAX;
        app.save().unwrap();
        let mut app = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
        enqueue(
            &mut app,
            ActionCommand::TransportStop,
            Quantization::Immediate,
        );
        assert_error_unchanged(
            &mut app,
            &path,
            |app| {
                if toggle {
                    app.commit_transport_toggle(120)
                        .map(|toggle| toggle.committed)
                } else {
                    let boundary = app
                        .runtime
                        .transport
                        .boundary_state(CommitBoundary::Immediate);
                    app.commit_ready_actions(boundary, 120)
                }
            },
            |error| matches!(error, JamAppError::CommitSequenceExhausted(_)),
        );
    }
}
