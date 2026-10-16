//! Generated, source-free boundary controls: no audio runtime or source access.

use std::{collections::BTreeSet, fmt::Debug, fs, path::Path};

use riotbox_core::{
    action::{
        ActionCommand, ActionDraft, ActionTarget, ActorType, CommitBoundary, GhostMode,
        Quantization, SourceMonitorMode, UndoPolicy,
    },
    ids::ActionId,
    persistence::save_session_json,
    queue::{ActionIdExhausted, ActionQueue},
    session::{ActionCommitRecord, SessionFile},
    transport::CommitBoundaryState,
};
use tempfile::tempdir;

use crate::jam_app::{
    JamAppError, JamAppState, QueueControlResult, SourceMapNavigationIntent,
    SourceMapNavigationResult, tests::ghost_assist_queue::fixtures::ghost_fill_suggestion,
};

fn immediate_boundary() -> CommitBoundaryState {
    CommitBoundaryState {
        kind: CommitBoundary::Immediate,
        beat_index: 0,
        bar_index: 1,
        phrase_index: 1,
        scene_id: None,
    }
}

fn write_source_free_history(path: &Path, imported_id: u64) {
    let mut session = SessionFile::new(
        "generated-id-exhaustion",
        "generated-only",
        "2026-10-16T00:00:00Z",
    );
    session.runtime_state.transport.position_beats = 12.5;
    session.runtime_state.transport.is_playing = false;
    assert!(session.source_refs.is_empty());
    assert!(session.source_graph_refs.is_empty());
    assert!(session.captures.is_empty());

    // Build coherent persisted history with the real public allocator/commit
    // path, rather than forging a high ID or mismatched commit relation.
    let mut queue = ActionQueue::new();
    queue.reserve_action_ids_after(Some(ActionId(imported_id - 1)));
    let id = queue
        .enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::TransportPause,
                Quantization::Immediate,
                ActionTarget::default(),
            ),
            1,
        )
        .unwrap();
    assert_eq!(id, ActionId(imported_id));
    let committed = queue
        .commit_ready_for_transport(&session.action_log, immediate_boundary(), 2)
        .unwrap();
    assert_eq!(committed.len(), 1);
    session
        .action_log
        .actions
        .push(queue.history_action(id).unwrap().clone());
    session.action_log.commit_records.push(ActionCommitRecord {
        action_id: id,
        boundary: committed[0].boundary.clone(),
        commit_sequence: committed[0].commit_sequence,
        committed_at: 2,
        mc202_source_phrase_plan: None,
    });
    save_session_json(path, &session).unwrap();
}

fn load_source_free(path: &Path) -> JamAppState {
    // RuntimeFull is deliberate: the original public-loader repro had no
    // references to hydrate, and never constructed an audio runtime.
    let mut state = JamAppState::from_json_files(path, None::<&Path>).unwrap();
    assert!(state.source_graph.is_none());
    assert!(state.source_audio_cache.is_none());
    assert!(state.capture_audio_cache.is_empty());
    assert!(state.session.source_refs.is_empty());
    assert!(state.session.source_graph_refs.is_empty());
    assert!(state.session.captures.is_empty());
    assert!(
        state
            .apply_audio_timing_snapshot(None, 3)
            .unwrap()
            .is_empty()
    );
    state
}

fn assert_exhaustion_unchanged<T: Debug, E: Debug + Into<JamAppError>>(
    state: &mut JamAppState,
    operation: impl FnOnce(&mut JamAppState) -> Result<T, E>,
) {
    let session = state.session.clone();
    let runtime = state.runtime.clone();
    let queue = format!("{:?}", state.queue);
    let jam_view = state.jam_view.clone();
    let runtime_view = state.runtime_view.clone();
    assert!(matches!(
        operation(state).unwrap_err().into(),
        JamAppError::ActionIdExhausted(ActionIdExhausted)
    ));
    assert_eq!(state.session, session);
    assert_eq!(state.runtime, runtime);
    assert_eq!(format!("{:?}", state.queue), queue);
    assert_eq!(state.jam_view, jam_view);
    assert_eq!(state.runtime_view, runtime_view);
}

fn assert_writer_exhaustion_unchanged<T: Debug>(
    state: &mut JamAppState,
    operation: impl FnOnce(&mut JamAppState) -> Result<T, JamAppError>,
) {
    let session = state.session.clone();
    let runtime = state.runtime.clone();
    let queue = format!("{:?}", state.queue);
    assert!(matches!(
        operation(state),
        Err(JamAppError::ActionIdExhausted(ActionIdExhausted))
    ));
    assert_eq!(state.session, session);
    assert_eq!(state.runtime, runtime);
    assert_eq!(format!("{:?}", state.queue), queue);
}

#[test]
fn public_loader_max_and_max_minus_one_keep_unique_save_restore_replay_history() {
    let owner = tempdir().unwrap();
    for imported_id in [u64::MAX, u64::MAX - 1] {
        let path = owner.path().join(format!("session-{imported_id}.json"));
        write_source_free_history(&path, imported_id);
        let mut state = load_source_free(&path);
        assert_eq!(state.runtime.transport.position_beats, 12.5);

        if imported_id == u64::MAX - 1 {
            let play = state.commit_transport_toggle(100).unwrap();
            assert_eq!(play.command, ActionCommand::TransportPlay);
            assert_eq!(play.committed.len(), 1);
            assert_eq!(play.committed[0].action_id, ActionId(u64::MAX));
            assert!(state.runtime.transport.is_playing);
            assert_eq!(state.runtime.transport.position_beats, 12.5);
            state.save().unwrap();
            state = load_source_free(&path);
        }

        let bytes_before = fs::read(&path).unwrap();
        for timestamp in [200, 300] {
            assert_exhaustion_unchanged(&mut state, |state| {
                state.commit_transport_toggle(timestamp)
            });
        }
        assert_eq!(fs::read(&path).unwrap(), bytes_before);
        let cursor = state.session.action_log.actions.len();
        let ids = state
            .session
            .action_log
            .actions
            .iter()
            .map(|action| action.id)
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), cursor);
        assert!(state.restore_target_dry_run_summary(cursor).is_ok());
        state.save().unwrap();
        let mut restored = load_source_free(&path);
        assert_eq!(restored.session.action_log, state.session.action_log);
        restored.apply_restore_target_suffix(cursor).unwrap();
        assert_exhaustion_unchanged(&mut restored, |state| state.commit_transport_toggle(400));
        restored.save().unwrap();
        assert!(
            load_source_free(&path)
                .restore_target_dry_run_summary(cursor)
                .is_ok()
        );
    }
}

#[test]
fn exhausted_direct_controls_refuse_without_queue_session_or_runtime_mutation() {
    let owner = tempdir().unwrap();
    let path = owner.path().join("exhausted-controls.json");
    write_source_free_history(&path, u64::MAX);
    let mut state = load_source_free(&path);
    assert_exhaustion_unchanged(&mut state, |state| state.queue_capture_bar(100));
    assert_exhaustion_unchanged(&mut state, |state| state.queue_scene_mutation(100));
    assert_exhaustion_unchanged(&mut state, |state| state.queue_tr909_fill(100));
    assert_exhaustion_unchanged(&mut state, |state| state.queue_tr909_reinforce(100));
    assert_exhaustion_unchanged(&mut state, |state| state.queue_tr909_slam_toggle(100));
    assert_exhaustion_unchanged(&mut state, |state| state.queue_tr909_takeover(100));
    assert_exhaustion_unchanged(&mut state, |state| state.queue_mc202_generate_follower(100));
    assert_exhaustion_unchanged(&mut state, |state| {
        state.queue_source_monitor_mode(SourceMonitorMode::Blend, 100)
    });

    // Existing eligibility/no-target outcomes take precedence over allocation.
    assert_eq!(state.queue_w30_trigger_pad(100).unwrap(), None);
    assert!(!state.queue_promote_last_capture(100).unwrap());
    assert_eq!(
        state.queue_scene_select(100).unwrap(),
        QueueControlResult::AlreadyInState
    );
    assert_eq!(
        state.queue_source_timing_grid_confirmation(100).unwrap(),
        QueueControlResult::AlreadyInState
    );
    assert_eq!(
        state
            .queue_source_map_navigation(SourceMapNavigationIntent::NextBar, 100)
            .unwrap(),
        SourceMapNavigationResult::Unavailable {
            reason: "source map timing grid unavailable"
        }
    );
    assert_eq!(state.undo_last_action(100).unwrap(), None);
}

#[test]
fn last_available_pending_control_preserves_already_pending_precedence() {
    let owner = tempdir().unwrap();
    let path = owner.path().join("last-pending.json");
    write_source_free_history(&path, u64::MAX - 1);
    let mut state = load_source_free(&path);
    assert_eq!(
        state
            .queue_source_monitor_mode(SourceMonitorMode::Blend, 100)
            .unwrap(),
        QueueControlResult::Enqueued
    );
    assert_eq!(state.queue.pending_actions()[0].id, ActionId(u64::MAX));
    assert_eq!(
        state
            .queue_source_monitor_mode(SourceMonitorMode::Riotbox, 200)
            .unwrap(),
        QueueControlResult::AlreadyPending
    );
    assert_exhaustion_unchanged(&mut state, |state| state.commit_transport_toggle(300));
}

#[test]
fn exhausted_undo_preserves_accepted_action_and_its_restore_snapshot() {
    let owner = tempdir().unwrap();
    let path = owner.path().join("last-monitor.json");
    write_source_free_history(&path, u64::MAX - 1);
    let mut state = load_source_free(&path);
    state
        .queue_source_monitor_mode(SourceMonitorMode::Blend, 100)
        .unwrap();
    assert_eq!(
        state
            .commit_ready_actions(immediate_boundary(), 101)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .undo_state
            .source_monitor_snapshots
            .len(),
        1
    );
    for timestamp in [200, 300] {
        assert_exhaustion_unchanged(&mut state, |state| state.undo_last_action(timestamp));
    }
    state.save().unwrap();
    let mut restored = load_source_free(&path);
    assert_exhaustion_unchanged(&mut restored, |state| state.undo_last_action(400));
    assert!(restored.restore_target_dry_run_summary(2).is_ok());
}

#[test]
fn exhausted_undo_does_not_even_normalize_an_older_missing_snapshot() {
    let owner = tempdir().unwrap();
    let path = owner.path().join("undo-normalization-order.json");
    write_source_free_history(&path, u64::MAX - 1);
    let mut state = load_source_free(&path);
    state
        .queue_source_monitor_mode(SourceMonitorMode::Blend, 100)
        .unwrap();
    state
        .commit_ready_actions(immediate_boundary(), 101)
        .unwrap();

    // Deliberately malformed in-memory older history makes normalization
    // observable. The latest Source Monitor target remains fully restorable;
    // failure to allocate its marker must precede even this unrelated rewrite.
    let older = &mut state.session.action_log.actions[0];
    older.command = ActionCommand::Tr909FillNext;
    older.undo_policy = UndoPolicy::Undoable;
    assert_exhaustion_unchanged(&mut state, |state| state.undo_last_action(200));
}

#[test]
fn final_available_id_can_commit_one_unique_undo_marker_and_reload() {
    let owner = tempdir().unwrap();
    let path = owner.path().join("final-undo-marker.json");
    write_source_free_history(&path, u64::MAX - 2);
    let mut state = load_source_free(&path);
    state
        .queue_source_monitor_mode(SourceMonitorMode::Blend, 100)
        .unwrap();
    state
        .commit_ready_actions(immediate_boundary(), 101)
        .unwrap();
    let undo = state.undo_last_action(200).unwrap().unwrap();
    assert_eq!(undo.id, ActionId(u64::MAX));
    assert_eq!(undo.command, ActionCommand::UndoLast);
    assert_eq!(
        state.session.runtime_state.source_monitor.mode,
        SourceMonitorMode::Source
    );
    assert!(
        state
            .session
            .runtime_state
            .undo_state
            .source_monitor_snapshots
            .is_empty()
    );
    assert_exhaustion_unchanged(&mut state, |state| state.commit_transport_toggle(300));
    state.save().unwrap();
    let mut restored = load_source_free(&path);
    assert!(restored.restore_target_dry_run_summary(3).is_ok());
    assert_eq!(restored.session.action_log, state.session.action_log);
    assert_exhaustion_unchanged(&mut restored, |state| state.commit_transport_toggle(400));
}

#[test]
fn exhausted_ghost_never_archives_accepts_or_clears_a_proposal() {
    let owner = tempdir().unwrap();
    let path = owner.path().join("exhausted-ghost.json");
    write_source_free_history(&path, u64::MAX);
    let mut state = load_source_free(&path);
    state.session.ghost_state.mode = GhostMode::Assist;
    let suggestion = ghost_fill_suggestion();
    assert_exhaustion_unchanged(&mut state, |state| {
        state.queue_accepted_ghost_suggestion(&suggestion, 100)
    });
    state.set_current_ghost_suggestion(suggestion);
    assert_exhaustion_unchanged(&mut state, |state| {
        state.accept_current_ghost_suggestion(200)
    });
    assert!(!state.session.ghost_state.suggestion_history[0].accepted);
}

#[test]
fn exhausted_export_autoqueue_fails_before_writer_or_receipt_side_effects() {
    let owner = tempdir().unwrap();
    let path = owner.path().join("exhausted-exports.json");
    write_source_free_history(&path, u64::MAX);
    let mut state = load_source_free(&path);
    let missing_proof = owner.path().join("never-opened-proof.json");
    let destination = owner.path().join("never-created-export");
    let daw = owner.path().join("never-created.dawproject");
    let session_bytes = fs::read(&path).unwrap();

    assert_writer_exhaustion_unchanged(&mut state, |state| {
        state.commit_product_mix_export_from_proof(&missing_proof, &destination, 100)
    });
    assert_writer_exhaustion_unchanged(&mut state, |state| {
        state.reject_product_mix_export_request(100, "explicit rejection")
    });
    assert_writer_exhaustion_unchanged(&mut state, |state| {
        state.commit_stem_package_export_local_ci_package(&destination, 100, Vec::new())
    });
    assert_writer_exhaustion_unchanged(&mut state, |state| {
        state.commit_stem_package_export_from_product_handoff(&missing_proof, &destination, 100)
    });
    assert_writer_exhaustion_unchanged(&mut state, |state| {
        state.commit_stem_package_export_w30_hook_loop(&destination, 100)
    });
    assert_writer_exhaustion_unchanged(&mut state, |state| {
        state.commit_w30_hook_dawproject_export(None, &daw, 100)
    });
    assert_writer_exhaustion_unchanged(&mut state, |state| {
        state.commit_live_master_dawproject_export(&daw, 100)
    });
    assert_writer_exhaustion_unchanged(&mut state, |state| {
        state.commit_and_save_live_master_dawproject_export(&daw, 100)
    });
    assert_writer_exhaustion_unchanged(&mut state, |state| {
        state.commit_daw_session_writer_export(None, &destination, 100)
    });
    assert_writer_exhaustion_unchanged(&mut state, |state| {
        state.commit_daw_session_host_import_proof_export(&missing_proof, 100)
    });
    assert_writer_exhaustion_unchanged(&mut state, |state| {
        state.commit_daw_session_audible_output_proof_export(&missing_proof, 100)
    });
    assert!(!destination.exists());
    assert!(!daw.exists());
    assert!(!missing_proof.exists());
    assert_eq!(fs::read(&path).unwrap(), session_bytes);
    assert!(state.session.export_receipts.is_empty());
}
