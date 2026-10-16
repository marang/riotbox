//! Sequence failures run through the same controller, without a device.
use std::{collections::BTreeSet, path::Path};

use riotbox_core::{
    action::{ActionCommand, ActionDraft, ActionTarget, ActorType, CommitBoundary, Quantization},
    persistence::save_session_json,
    queue::ActionQueue,
    session::ActionCommitRecord,
    transport::CommitBoundaryState,
};

use super::{Case, Event, LiveRecordingDuration, execute_with_control, fixture};
use crate::jam_app::{JamAppError, JamAppState};

fn boundary_at(state: &JamAppState, position: f64, kind: CommitBoundary) -> CommitBoundaryState {
    let mut session = state.session.clone();
    session.runtime_state.transport.position_beats = position;
    JamAppState::from_parts(session, None, ActionQueue::new())
        .runtime
        .transport
        .boundary_state(kind)
}

fn import_sequence(state: &mut JamAppState, boundary: CommitBoundaryState, sequence: u32) {
    let mut queue = ActionQueue::new();
    let command = if state.runtime.transport.is_playing {
        ActionCommand::TransportPlay
    } else {
        ActionCommand::TransportPause
    };
    let id = queue
        .enqueue(
            ActionDraft::new(
                ActorType::User,
                command,
                Quantization::Immediate,
                ActionTarget::default(),
            ),
            1,
        )
        .unwrap();
    assert_eq!(
        queue
            .commit_ready_for_transport(&state.session.action_log, boundary.clone(), 2)
            .unwrap()
            .len(),
        1
    );
    state
        .session
        .action_log
        .actions
        .push(queue.history_action(id).unwrap().clone());
    state
        .session
        .action_log
        .commit_records
        .push(ActionCommitRecord {
            action_id: id,
            boundary,
            commit_sequence: sequence,
            committed_at: 2,
            mc202_source_phrase_plan: None,
        });
    let path = state.files.as_ref().unwrap().session_path.clone();
    save_session_json(&path, &state.session).unwrap();
    *state = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
    assert!(state.source_graph.is_none());
    assert!(state.source_audio_cache.is_none());
    assert!(state.session.captures.is_empty());
    assert!(state.restore_target_dry_run_summary(1).is_ok());
}

#[test]
fn initial_play_sequence_refusal_stops_without_mutation_or_publication() {
    let (_temp, launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
    let boundary = boundary_at(&state, 0.0, CommitBoundary::Immediate);
    import_sequence(&mut state, boundary, u32::MAX);
    let before = state.session.clone();
    let queue = format!("{:?}", state.queue);
    let error = execute_with_control(&launch, &mut state, &mut control).unwrap_err();
    assert!(matches!(
        error.downcast_ref::<JamAppError>(),
        Some(JamAppError::CommitSequenceExhausted(_))
    ));
    assert_eq!(state.session, before);
    assert_eq!(format!("{:?}", state.queue), queue);
    assert_eq!(*control.events.borrow(), [Event::Stop]);
    assert!(control.stopped.get());
    control.assert_unpublished();
}

#[test]
fn full_start_boundary_allows_queue_only_recording_at_another_final_boundary() {
    let (_temp, launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, true);
    let boundary = boundary_at(&state, 0.0, CommitBoundary::Immediate);
    import_sequence(&mut state, boundary, u32::MAX);
    let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
    assert_eq!(summary["ready"], true);
    assert_eq!(summary["runtime_stopped"], true);
    assert_eq!(
        *control.events.borrow(),
        [Event::Begin, Event::Stop, Event::Finish]
    );
    assert_eq!(state.session.action_log.commit_records.len(), 2);
    assert_eq!(
        state.session.action_log.commit_records[0].commit_sequence,
        u32::MAX
    );
    assert_eq!(
        state.session.action_log.commit_records[1].commit_sequence,
        1
    );
    assert_ne!(
        state.session.action_log.commit_records[0].boundary,
        state.session.action_log.commit_records[1].boundary
    );
    assert_eq!(state.session.export_receipts.len(), 1);
}

#[test]
fn final_writer_sequence_refusal_rejects_consumed_capture_without_publishing() {
    let (_temp, launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, true);
    let boundary = boundary_at(&state, 12.0, CommitBoundary::Immediate);
    import_sequence(&mut state, boundary, u32::MAX);
    let path = state.files.as_ref().unwrap().session_path.clone();
    let bytes = std::fs::read(&path).unwrap();
    let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
    assert_eq!(summary["ready"], false);
    assert_eq!(summary["runtime_stopped"], true);
    assert_eq!(
        *control.events.borrow(),
        [Event::Begin, Event::Stop, Event::Finish, Event::Stop]
    );
    assert!(
        state.queue.pending_actions().is_empty(),
        "a finalized failed take cannot remain pending"
    );
    let failed_take = state
        .queue
        .history()
        .iter()
        .find(|action| action.command == ActionCommand::ExportLiveRecording)
        .unwrap();
    assert_eq!(
        failed_take.status,
        riotbox_core::action::ActionStatus::Rejected
    );
    assert_eq!(state.session.action_log.actions.len(), 1);
    assert!(state.session.export_receipts.is_empty());
    assert_eq!(std::fs::read(path).unwrap(), bytes);
    control.assert_unpublished();
}

#[test]
fn acquired_capture_sync_refusal_stops_aborts_and_keeps_uncommitted_fill() {
    let (_temp, launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
    let boundary = boundary_at(&state, 4.0, CommitBoundary::Bar);
    import_sequence(&mut state, boundary, u32::MAX);
    state.queue_tr909_fill(900).unwrap();
    let fill_id = state.queue.pending_actions()[0].id;
    let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
    assert_eq!(summary["ready"], false);
    assert_eq!(summary["runtime_stopped"], true);
    assert!(
        summary["failure_reason"]
            .as_str()
            .unwrap()
            .contains("timing commit failed")
    );
    assert!(control.stopped.get());
    let events = control.events.borrow();
    assert_eq!(events[0], Event::Begin);
    let stop = events
        .iter()
        .position(|event| *event == Event::Stop)
        .unwrap();
    let abort = events
        .iter()
        .position(|event| *event == Event::Abort)
        .unwrap();
    assert!(stop < abort);
    assert!(!events.contains(&Event::Finish));
    assert!(
        state
            .queue
            .pending_actions()
            .iter()
            .any(|action| action.id == fill_id)
    );
    assert!(
        !state
            .session
            .action_log
            .actions
            .iter()
            .any(|action| action.id == fill_id)
    );
    assert!(state.session.export_receipts.is_empty());
    control.assert_unpublished();
}

#[test]
fn final_pause_sequence_refusal_still_stops_and_aborts_without_publication() {
    let (_temp, launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
    // Controlled 4/4 fixture: next bar begins at4; the two-bar take ends at12.
    let boundary = boundary_at(&state, 12.0, CommitBoundary::Immediate);
    import_sequence(&mut state, boundary, u32::MAX);
    let path = state.files.as_ref().unwrap().session_path.clone();
    let bytes = std::fs::read(&path).unwrap();
    let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
    assert_eq!(summary["ready"], false);
    assert_eq!(summary["runtime_stopped"], true);
    assert!(
        summary["failure_reason"]
            .as_str()
            .unwrap()
            .contains("transport cleanup failed")
    );
    assert!(control.stopped.get());
    assert!(
        state.runtime.transport.is_playing,
        "never invent a committed Pause"
    );
    let events = control.events.borrow();
    assert!(events.contains(&Event::Stop));
    assert!(events.contains(&Event::Abort));
    assert!(!events.contains(&Event::Finish));
    assert!(state.session.export_receipts.is_empty());
    assert_eq!(std::fs::read(path).unwrap(), bytes);
    control.assert_unpublished();
}

#[test]
fn final_boundary_two_remaining_slots_save_pause_and_recording_uniquely() {
    let (_temp, launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
    let boundary = boundary_at(&state, 12.0, CommitBoundary::Immediate);
    import_sequence(&mut state, boundary.clone(), u32::MAX - 2);
    let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
    assert_eq!(summary["ready"], true);
    assert_eq!(
        *control.events.borrow(),
        [Event::Begin, Event::Stop, Event::Finish]
    );
    let path = &state.files.as_ref().unwrap().session_path;
    let restored = JamAppState::from_json_files(path, None::<&Path>).unwrap();
    let sequences = restored
        .session
        .action_log
        .commit_records
        .iter()
        .filter(|record| record.boundary == boundary)
        .map(|record| record.commit_sequence)
        .collect::<BTreeSet<_>>();
    assert_eq!(sequences, [u32::MAX - 2, u32::MAX - 1, u32::MAX].into());
    assert_eq!(restored.session.export_receipts.len(), 1);
    assert!(
        restored
            .restore_target_dry_run_summary(restored.session.action_log.actions.len())
            .is_ok()
    );
}
