//! Generated metadata and in-memory recorder controls; no audio runtime.
use std::{fs, path::Path};

use riotbox_core::{
    action::{ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType, Quantization},
    ids::ActionId,
    persistence::save_session_json,
    session::ReplayPolicy,
};

use super::{Case, Event, LiveRecordingDuration, execute_with_control, fixture};
use crate::jam_app::{JamAppError, JamAppState};

fn declare_policy(state: &mut JamAppState, policy: ReplayPolicy, position: f64) {
    state.session.action_log.replay_policy = policy;
    state.session.runtime_state.transport.position_beats = position;
    let path = state.files.as_ref().unwrap().session_path.clone();
    save_session_json(&path, &state.session).unwrap();
    *state = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
    assert!(state.source_graph.is_none());
    assert!(state.source_audio_cache.is_none());
}

fn enqueue_unsafe(state: &mut JamAppState, seek: bool, quantization: Quantization) -> ActionId {
    let mut draft = ActionDraft::new(
        ActorType::User,
        if seek {
            ActionCommand::TransportSeek
        } else {
            ActionCommand::TransportStop
        },
        quantization,
        ActionTarget::default(),
    );
    if seek {
        draft.params = ActionParams::Transport {
            position_beats: Some(1),
        };
    }
    state.queue.enqueue(draft, 900).unwrap()
}

fn assert_still_pending(state: &JamAppState, id: ActionId) {
    assert!(
        state
            .queue
            .pending_actions()
            .iter()
            .any(|action| action.id == id)
    );
    assert!(!state.queue.history().iter().any(|action| action.id == id));
    assert!(
        !state
            .session
            .action_log
            .actions
            .iter()
            .any(|action| action.id == id)
    );
    assert!(
        !state
            .session
            .action_log
            .commit_records
            .iter()
            .any(|record| record.action_id == id)
    );
    assert!(state.session.export_receipts.is_empty());
}

#[test]
fn initial_toggle_legacy_stop_admission_refusal_stops_before_capture_begin() {
    for policy in [
        ReplayPolicy::DeterministicPreferred,
        ReplayPolicy::SnapshotOnly,
    ] {
        let (_temp, launch, mut state, mut control) =
            fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
        declare_policy(&mut state, policy, 12.0);
        let id = enqueue_unsafe(&mut state, false, Quantization::Immediate);
        let path = state.files.as_ref().unwrap().session_path.clone();
        let saved = fs::read(&path).unwrap();
        let next_id = state.queue.clone().allocate_action_id();
        let error = execute_with_control(&launch, &mut state, &mut control).unwrap_err();
        assert!(matches!(error.downcast_ref::<JamAppError>(),
            Some(JamAppError::ReplayAdmission { policy: actual, .. }) if *actual == policy));
        assert_eq!(*control.events.borrow(), [Event::Stop]);
        assert!(control.stopped.get());
        assert!(control.request.borrow().is_none());
        assert_still_pending(&state, id);
        assert_eq!(state.queue.clone().allocate_action_id(), next_id);
        assert!(state.session.action_log.actions.is_empty());
        assert_eq!(fs::read(&path).unwrap(), saved);
        control.assert_unpublished();
    }
}

#[test]
fn acquired_capture_legacy_boundary_admission_refusal_stops_aborts_without_publication() {
    for policy in [
        ReplayPolicy::DeterministicPreferred,
        ReplayPolicy::SnapshotOnly,
    ] {
        for seek in [false, true] {
            let (_temp, launch, mut state, mut control) =
                fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
            declare_policy(&mut state, policy, 0.0);
            let id = enqueue_unsafe(&mut state, seek, Quantization::NextBar);
            let path = state.files.as_ref().unwrap().session_path.clone();
            let saved = fs::read(&path).unwrap();
            let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
            assert_eq!(summary["ready"], false);
            assert_eq!(summary["status"], "blocked");
            assert_eq!(summary["runtime_stopped"], true);
            assert_eq!(summary["writes_files"], false);
            let reason = summary["failure_reason"].as_str().unwrap();
            assert!(reason.contains("timing commit failed"), "{reason}");
            assert!(
                reason.contains(&format!(
                    "operation refused for declared replay policy {policy:?}"
                )),
                "{reason}"
            );
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
            assert!(control.stopped.get());
            assert_still_pending(&state, id);
            // A cleanup Play/Pause may commit; the refused navigation and take
            // must not acquire records, receipts, or saved bytes.
            assert!(
                !state
                    .session
                    .action_log
                    .actions
                    .iter()
                    .any(|action| action.command == ActionCommand::ExportLiveRecording)
            );
            assert_eq!(state.session.action_log.replay_policy, policy);
            assert_eq!(fs::read(&path).unwrap(), saved);
            control.assert_unpublished();
        }
    }
}
