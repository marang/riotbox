//! Reference-free generated Session imports; never launches an audio runtime.

use std::{fmt::Debug, fs, path::Path};

use riotbox_core::{
    action::{
        ActionCommand, ActionDraft, ActionTarget, ActorType, CommitBoundary, Quantization,
        SourceMonitorMode,
    },
    ids::ActionId,
    persistence::save_session_json,
    queue::ActionQueue,
    session::{ActionCommitRecord, SessionFile},
    transport::CommitBoundaryState,
};
use tempfile::tempdir;

use crate::jam_app::{JamAppError, JamAppState};

mod exports;
mod timing;

fn imported_boundary() -> CommitBoundaryState {
    CommitBoundaryState {
        kind: CommitBoundary::Immediate,
        beat_index: 12,
        bar_index: 4,
        phrase_index: 1,
        scene_id: None,
    }
}

fn write_source_free_history(path: &Path, sequence: u32) {
    write_history_at(path, sequence, imported_boundary());
}

fn write_history_at(path: &Path, sequence: u32, boundary: CommitBoundaryState) {
    let mut session = SessionFile::new(
        "generated-sequence-exhaustion",
        "generated-only",
        "2026-10-16T00:00:00Z",
    );
    session.runtime_state.transport.position_beats = 12.5;
    session.runtime_state.transport.is_playing = false;
    assert!(session.source_refs.is_empty());
    assert!(session.source_graph_refs.is_empty());
    assert!(session.captures.is_empty());
    let mut queue = ActionQueue::new();
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
    let committed = queue
        .commit_ready_for_transport(&session.action_log, boundary.clone(), 2)
        .unwrap();
    assert_eq!(committed.len(), 1);
    session
        .action_log
        .actions
        .push(queue.history_action(id).unwrap().clone());
    session.action_log.commit_records.push(ActionCommitRecord {
        action_id: id,
        boundary,
        commit_sequence: sequence,
        committed_at: 2,
        mc202_source_phrase_plan: None,
    });
    save_session_json(path, &session).unwrap();
}

fn load_source_free(path: &Path) -> JamAppState {
    let state = JamAppState::from_json_files(path, None::<&Path>).unwrap();
    assert!(state.source_graph.is_none());
    assert!(state.source_audio_cache.is_none());
    assert!(state.capture_audio_cache.is_empty());
    assert!(state.session.source_refs.is_empty());
    assert!(state.session.source_graph_refs.is_empty());
    assert!(state.session.captures.is_empty());
    state
}

fn enqueue_pause(state: &mut JamAppState) -> ActionId {
    state
        .queue
        .enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::TransportPause,
                Quantization::Immediate,
                ActionTarget::default(),
            ),
            10,
        )
        .unwrap()
}

fn assert_sequence_refusal<T: Debug, E: Debug + Into<JamAppError>>(
    state: &mut JamAppState,
    boundary: &CommitBoundaryState,
    count: usize,
    operation: impl FnOnce(&mut JamAppState) -> Result<T, E>,
) {
    let session = state.session.clone();
    let runtime = state.runtime.clone();
    let queue = format!("{:?}", state.queue);
    let jam_view = state.jam_view.clone();
    let runtime_view = state.runtime_view.clone();
    let error = match operation(state).unwrap_err().into() {
        JamAppError::CommitSequenceExhausted(error) => error,
        other => panic!("expected typed commit-sequence refusal, got {other:?}"),
    };
    assert_eq!(error.boundary, *boundary);
    assert_eq!(error.requested_count, count);
    assert_eq!(state.session, session);
    assert_eq!(state.runtime, runtime);
    assert_eq!(format!("{:?}", state.queue), queue);
    assert_eq!(state.jam_view, jam_view);
    assert_eq!(state.runtime_view, runtime_view);
}

#[test]
fn public_loader_max_sequence_refuses_transport_toggle_without_panicking() {
    let owner = tempdir().unwrap();
    let path = owner.path().join("max-sequence.json");
    write_source_free_history(&path, u32::MAX);
    let mut state = load_source_free(&path);
    assert_eq!(
        state
            .runtime
            .transport
            .boundary_state(CommitBoundary::Immediate),
        imported_boundary(),
    );
    let saved_bytes = fs::read(&path).unwrap();
    assert_sequence_refusal(&mut state, &imported_boundary(), 1, |state| {
        state.commit_transport_toggle(3)
    });
    assert_eq!(fs::read(&path).unwrap(), saved_bytes);
    state.save().unwrap();
    let restored = load_source_free(&path);
    assert!(restored.restore_target_dry_run_summary(1).is_ok());
}

#[test]
fn full_boundary_accepts_empty_batch_but_refuses_pending_work_without_relabeling() {
    let owner = tempdir().unwrap();
    let path = owner.path().join("max-batch.json");
    write_source_free_history(&path, u32::MAX);
    let mut state = load_source_free(&path);
    assert!(
        state
            .commit_ready_actions(imported_boundary(), 3)
            .unwrap()
            .is_empty()
    );
    enqueue_pause(&mut state);
    for timestamp in [4, 5] {
        assert_sequence_refusal(&mut state, &imported_boundary(), 1, |state| {
            state.commit_ready_actions(imported_boundary(), timestamp)
        });
    }
}

#[test]
fn max_minus_one_refuses_whole_two_action_batch_and_toggle_with_prior_ready_work() {
    let owner = tempdir().unwrap();
    let path = owner.path().join("two-actions.json");
    write_source_free_history(&path, u32::MAX - 1);
    let mut state = load_source_free(&path);
    enqueue_pause(&mut state);
    assert_sequence_refusal(&mut state, &imported_boundary(), 2, |state| {
        state.commit_transport_toggle(20)
    });
    enqueue_pause(&mut state);
    assert_sequence_refusal(&mut state, &imported_boundary(), 2, |state| {
        state.commit_ready_actions(imported_boundary(), 30)
    });
}

#[test]
fn final_sequence_commits_once_saves_reloads_replays_and_other_boundary_stays_usable() {
    let owner = tempdir().unwrap();
    let path = owner.path().join("last-sequence.json");
    write_source_free_history(&path, u32::MAX - 1);
    let mut state = load_source_free(&path);
    let toggle = state.commit_transport_toggle(10).unwrap();
    assert_eq!(toggle.committed.len(), 1);
    assert_eq!(toggle.committed[0].commit_sequence, u32::MAX);
    state.save().unwrap();
    let mut state = load_source_free(&path);
    assert_sequence_refusal(&mut state, &imported_boundary(), 1, |state| {
        state.commit_transport_toggle(20)
    });
    assert!(state.restore_target_dry_run_summary(2).is_ok());
    state.apply_restore_target_suffix(2).unwrap();
    enqueue_pause(&mut state);
    let mut other = imported_boundary();
    other.kind = CommitBoundary::Beat;
    let committed = state.commit_ready_actions(other, 30).unwrap();
    assert_eq!(committed.len(), 1);
    assert_eq!(committed[0].commit_sequence, 1);
    state.save().unwrap();
    assert!(
        load_source_free(&path)
            .restore_target_dry_run_summary(3)
            .is_ok()
    );
}

fn monitor_at_sequence(path: &Path, prior_sequence: u32) -> JamAppState {
    write_source_free_history(path, prior_sequence);
    let mut state = load_source_free(path);
    state
        .queue_source_monitor_mode(SourceMonitorMode::Blend, 10)
        .unwrap();
    state.commit_ready_actions(imported_boundary(), 11).unwrap();
    assert_eq!(
        state
            .session
            .runtime_state
            .undo_state
            .source_monitor_snapshots
            .len(),
        1
    );
    state
}

#[test]
fn exhausted_undo_preserves_target_snapshot_and_sequence_failure_wins_before_id_allocation() {
    let owner = tempdir().unwrap();
    let path = owner.path().join("full-undo.json");
    let mut state = monitor_at_sequence(&path, u32::MAX - 1);
    assert_sequence_refusal(&mut state, &imported_boundary(), 1, |state| {
        state.undo_last_action(20)
    });
    state
        .queue
        .reserve_action_ids_after(Some(ActionId(u64::MAX)));
    assert_sequence_refusal(&mut state, &imported_boundary(), 1, |state| {
        state.undo_last_action(30)
    });
    assert_sequence_refusal(&mut state, &imported_boundary(), 1, |state| {
        state.commit_transport_toggle(40)
    });
    state.save().unwrap();
    let mut restored = load_source_free(&path);
    assert_sequence_refusal(&mut restored, &imported_boundary(), 1, |state| {
        state.undo_last_action(50)
    });
}

#[test]
fn final_undo_marker_uses_last_sequence_and_no_target_precedes_capacity_refusal() {
    let owner = tempdir().unwrap();
    let path = owner.path().join("last-undo.json");
    let mut state = monitor_at_sequence(&path, u32::MAX - 2);
    let undo = state.undo_last_action(20).unwrap().unwrap();
    assert_eq!(
        state
            .session
            .action_log
            .commit_records
            .last()
            .unwrap()
            .action_id,
        undo.id
    );
    assert_eq!(
        state
            .session
            .action_log
            .commit_records
            .last()
            .unwrap()
            .commit_sequence,
        u32::MAX
    );
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
    assert!(state.undo_last_action(30).unwrap().is_none());
    state.save().unwrap();
    assert!(
        load_source_free(&path)
            .restore_target_dry_run_summary(3)
            .is_ok()
    );
}
