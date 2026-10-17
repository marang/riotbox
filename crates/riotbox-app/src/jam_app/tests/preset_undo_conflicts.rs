//! Public metadata-only preset/Undo conflict regressions; no source hydration.
use crate::jam_app::{JamAppError, JamAppState};
use riotbox_core::{
    action::{ActionCommand, ActionStatus, CommitBoundary, SourceMonitorMode},
    ids::{ActionId, SnapshotId},
    persistence::save_session_json,
    queue::ActionQueue,
    replay::{apply_replay_plan_to_session, build_committed_replay_plan},
    session::{ReplayPolicy, SessionFile, Snapshot, SnapshotPayload},
    style::PerformancePresetId,
};
use std::{fs, path::Path};

const POLICIES: [ReplayPolicy; 3] = [
    ReplayPolicy::DeterministicCommittedOrderV2,
    ReplayPolicy::DeterministicPreferred,
    ReplayPolicy::SnapshotOnly,
];
const PRESETS: [PerformancePresetId; 2] = [
    PerformancePresetId::FeralBreakAlphaV1,
    PerformancePresetId::FeralBreakAlphaV2,
];

fn app(policy: ReplayPolicy) -> JamAppState {
    let mut session = SessionFile::new("generated-preset-undo", "test", "2026-10-17");
    session.action_log.replay_policy = policy;
    JamAppState::from_parts(session, None, ActionQueue::new())
}

fn commit(app: &mut JamAppState, at: u64) {
    commit_at(app, at, CommitBoundary::Immediate);
}

fn commit_at(app: &mut JamAppState, at: u64, kind: CommitBoundary) {
    let boundary = app.runtime.transport.boundary_state(kind);
    assert_eq!(app.commit_ready_actions(boundary, at).unwrap().len(), 1);
}

fn activate(app: &mut JamAppState, preset: PerformancePresetId, at: u64) {
    app.queue_performance_preset(preset, at).unwrap();
    commit(app, at);
}

fn assert_blocked_without_mutation(app: &mut JamAppState) {
    let before = app.session.clone();
    let runtime = app.runtime.clone();
    let view = app.jam_view.clone();
    let runtime_view = app.runtime_view.clone();
    let pending: Vec<_> = app.queue.pending_actions().into_iter().cloned().collect();
    let history = app.queue.history().to_vec();
    let next_id = app.queue.clone().allocate_action_id().unwrap();
    assert_eq!(app.undo_last_action(30).unwrap(), None);
    assert_eq!(app.session, before);
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
    assert_eq!(app.queue.clone().allocate_action_id().unwrap(), next_id);
}

#[test]
fn newer_preset_blocks_older_monitor_undo_before_session_or_id_mutation() {
    for policy in POLICIES {
        for preset in PRESETS {
            let mut app = app(policy);
            app.queue_source_monitor_mode(SourceMonitorMode::Blend, 10)
                .unwrap();
            commit(&mut app, 10);
            activate(&mut app, preset, 20);
            assert_eq!(
                app.session.runtime_state.source_monitor.mode,
                preset.definition().source_monitor_mode
            );
            assert_blocked_without_mutation(&mut app);
        }
    }
}

#[test]
fn newer_preset_blocks_older_mc202_touch_snapshot_for_all_live_policies() {
    for policy in POLICIES {
        for preset in PRESETS {
            let mut app = app(policy);
            let initial_touch = app.session.runtime_state.macro_state.mc202_touch;
            app.queue_mc202_role_toggle(10).unwrap();
            commit_at(&mut app, 10, CommitBoundary::Phrase);
            assert_ne!(
                app.session.runtime_state.macro_state.mc202_touch,
                initial_touch
            );
            assert_eq!(
                app.session.runtime_state.undo_state.mc202_snapshots.len(),
                1
            );
            activate(&mut app, preset, 20);
            assert_ne!(
                app.session.runtime_state.macro_state.mc202_touch,
                initial_touch
            );
            assert_blocked_without_mutation(&mut app);
        }
    }
}

#[test]
fn post_preset_monitor_undo_preserves_live_replay_and_roundtrip() {
    let owner = tempfile::tempdir().unwrap();
    for policy in POLICIES {
        for preset in PRESETS {
            let mut app = app(policy);
            let base = app.session.clone();
            activate(&mut app, preset, 10);
            app.queue_source_monitor_mode(SourceMonitorMode::Source, 20)
                .unwrap();
            commit(&mut app, 20);
            assert!(app.undo_last_action(30).unwrap().is_some());
            assert_eq!(
                app.session.runtime_state.source_monitor.mode,
                preset.definition().source_monitor_mode
            );
            let plan = build_committed_replay_plan(&app.session.action_log).unwrap();
            let mut replayed = base;
            apply_replay_plan_to_session(&mut replayed, &plan).unwrap();
            assert_eq!(
                app.session.runtime_state.source_monitor,
                replayed.runtime_state.source_monitor
            );
            assert_eq!(
                app.session.runtime_state.macro_state,
                replayed.runtime_state.macro_state
            );
            let path = owner.path().join(format!("{policy:?}-{preset:?}.json"));
            save_session_json(&path, &app.session).unwrap();
            let loaded = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
            loaded.save().unwrap();
            assert_eq!(loaded.session, app.session);
            assert_eq!(loaded.session.action_log.replay_policy, policy);
        }
    }
}

#[test]
fn older_fill_survives_preset_but_keeps_legacy_global_stack_admission() {
    for policy in POLICIES {
        for preset in PRESETS {
            let mut app = app(policy);
            app.session.runtime_state.lane_state.tr909.last_fill_bar = Some(7);
            app.queue_tr909_fill(10).unwrap();
            commit_at(&mut app, 10, CommitBoundary::Bar);
            app.queue_source_monitor_mode(SourceMonitorMode::Blend, 15)
                .unwrap();
            commit(&mut app, 15);
            activate(&mut app, preset, 20);
            let before = app.session.clone();
            let next_id = app.queue.clone().allocate_action_id();
            let result = app.undo_last_action(30);
            if policy == ReplayPolicy::DeterministicCommittedOrderV2 {
                let marker = result.unwrap().unwrap();
                assert_eq!(
                    app.session.action_log.actions[0].status,
                    ActionStatus::Undone
                );
                assert_eq!(
                    app.session.action_log.actions[1].status,
                    ActionStatus::Committed
                );
                assert_eq!(
                    app.session.runtime_state.lane_state.tr909.last_fill_bar,
                    Some(7)
                );
                assert_eq!(
                    app.session.runtime_state.source_monitor.mode,
                    preset.definition().source_monitor_mode
                );
                assert_eq!(marker.command, ActionCommand::UndoLast);
                build_committed_replay_plan(&app.session.action_log).unwrap();
            } else {
                assert!(matches!(result, Err(JamAppError::ReplayAdmission { .. })));
                assert_eq!(app.session, before);
                assert_eq!(app.queue.clone().allocate_action_id(), next_id);
            }
        }
    }
}

#[test]
fn newer_mc202_undo_restores_preset_touch_and_live_replay_parity() {
    for policy in POLICIES {
        for preset in PRESETS {
            let mut app = app(policy);
            let base = app.session.clone();
            activate(&mut app, preset, 10);
            let preset_touch = app.session.runtime_state.macro_state.mc202_touch;
            app.queue_mc202_role_toggle(20).unwrap();
            commit_at(&mut app, 20, CommitBoundary::Phrase);
            assert!(app.undo_last_action(30).unwrap().is_some());
            assert_eq!(
                app.session.runtime_state.macro_state.mc202_touch,
                preset_touch
            );
            let plan = build_committed_replay_plan(&app.session.action_log).unwrap();
            let mut replayed = base;
            apply_replay_plan_to_session(&mut replayed, &plan).unwrap();
            assert_eq!(
                app.session.runtime_state.macro_state,
                replayed.runtime_state.macro_state
            );
            assert_eq!(
                app.session.runtime_state.lane_state.mc202,
                replayed.runtime_state.lane_state.mc202
            );
        }
    }
}

#[test]
fn blocked_history_roundtrips_and_restores_without_reopening_old_undo() {
    let owner = tempfile::tempdir().unwrap();
    for policy in POLICIES {
        let mut app = app(policy);
        let initial = app.session.runtime_state.clone();
        app.queue_source_monitor_mode(SourceMonitorMode::Blend, 10)
            .unwrap();
        commit(&mut app, 10);
        activate(&mut app, PerformancePresetId::FeralBreakAlphaV2, 20);
        let snapshot_id = SnapshotId::from("before-preset-history");
        app.session.snapshots.push(Snapshot {
            snapshot_id: snapshot_id.clone(),
            created_at: "2026-10-17".into(),
            label: "generated valid base".into(),
            action_cursor: 0,
            payload: Some(SnapshotPayload::from_runtime_state(
                &snapshot_id,
                0,
                &initial,
            )),
        });
        let path = owner.path().join(format!("blocked-{policy:?}.json"));
        save_session_json(&path, &app.session).unwrap();
        let saved = fs::read(&path).unwrap();
        let mut loaded = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
        assert_blocked_without_mutation(&mut loaded);
        assert_eq!(fs::read(&path).unwrap(), saved);
        loaded.save().unwrap();
        loaded
            .apply_restore_target_from_snapshot_payload(2)
            .unwrap();
        assert_eq!(
            loaded.session.runtime_state.source_monitor.mode,
            SourceMonitorMode::Riotbox
        );
        assert_blocked_without_mutation(&mut loaded);
        assert_eq!(loaded.session.action_log.replay_policy, policy);
    }
}

/// Metadata fixture of the old producer's already-demonstrated invalid V2
/// relation. Build valid actions first, then explicitly forge the crossed marker;
/// no current live operation is claimed to produce it after the repair.
fn historical_cross_preset_marker() -> SessionFile {
    let mut product = app(ReplayPolicy::DeterministicCommittedOrderV2);
    product
        .queue_source_monitor_mode(SourceMonitorMode::Blend, 10)
        .unwrap();
    commit(&mut product, 10);
    activate(&mut product, PerformancePresetId::FeralBreakAlphaV2, 20);
    let mut marker_owner = app(ReplayPolicy::DeterministicCommittedOrderV2);
    marker_owner
        .queue_source_monitor_mode(SourceMonitorMode::Blend, 10)
        .unwrap();
    commit(&mut marker_owner, 10);
    let mut marker = marker_owner.undo_last_action(30).unwrap().unwrap();
    marker.id = ActionId(3);
    let mut record = marker_owner
        .session
        .action_log
        .commit_records
        .last()
        .unwrap()
        .clone();
    record.action_id = marker.id;
    record.commit_sequence = 3;
    product.session.action_log.actions[0].status = ActionStatus::Undone;
    product.session.action_log.actions.push(marker);
    product.session.action_log.commit_records.push(record);
    product.session.runtime_state.source_monitor.mode = SourceMonitorMode::Source;
    product
        .session
        .runtime_state
        .undo_state
        .source_monitor_snapshots
        .clear();
    product.session
}

#[test]
fn persisted_invalid_v2_marker_is_refused_without_touching_the_file_or_restore_target() {
    let owner = tempfile::tempdir().unwrap();
    let path = owner.path().join("generated-invalid-v2.json");
    let session = historical_cross_preset_marker();
    save_session_json(&path, &session).unwrap();
    let saved = fs::read(&path).unwrap();
    let result = JamAppState::from_json_files(&path, None::<&Path>);
    assert!(
        matches!(result, Err(JamAppError::InvalidSession(message)) if message.contains("InvalidUndoTargetRelation"))
    );
    assert_eq!(fs::read(&path).unwrap(), saved);
    let mut app = JamAppState::from_parts(session, None, ActionQueue::new());
    let before = app.session.clone();
    let runtime = app.runtime.clone();
    assert!(app.apply_restore_target_suffix(3).is_err());
    assert_eq!(app.session, before);
    assert_eq!(app.runtime, runtime);
}
