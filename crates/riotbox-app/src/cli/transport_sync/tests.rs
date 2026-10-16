use std::cell::{Cell, RefCell};

use super::{TransportControl, publish_transport, sync_transport};
use crate::jam_app::JamAppState;
use riotbox_audio::runtime::{
    AudioRuntimeTimingSnapshot, AudioTransportControlError, AudioTransportPositionRequest,
    AudioTransportPositionUpdate,
};
use riotbox_core::{
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType, CommitBoundary,
        Quantization,
    },
    queue::ActionQueue,
    session::SessionFile,
};

#[derive(Default)]
struct Control {
    observed: Cell<Option<AudioRuntimeTimingSnapshot>>,
    awaiting_callback: Cell<bool>,
    failure: Cell<Option<AudioTransportControlError>>,
    updates: RefCell<Vec<AudioTransportPositionUpdate>>,
}

impl Control {
    fn complete(&self, position: f64) {
        self.awaiting_callback.set(false);
        self.observed.set(Some(AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 120.0,
            position_beats: position,
        }));
    }
}

impl TransportControl for Control {
    fn timing_snapshot(&self) -> Option<AudioRuntimeTimingSnapshot> {
        if self.awaiting_callback.get() {
            None
        } else {
            self.observed.get()
        }
    }

    fn update_transport_state(
        &self,
        _playing: bool,
        _bpm: f32,
        position: AudioTransportPositionUpdate,
    ) -> Result<(), AudioTransportControlError> {
        if let Some(error) = self.failure.get() {
            return Err(error);
        }
        self.updates.borrow_mut().push(position);
        if matches!(position, AudioTransportPositionUpdate::Reposition(_)) {
            self.awaiting_callback.set(true);
        }
        Ok(())
    }
}

fn app() -> JamAppState {
    let mut session = SessionFile::new("generated-transport", "test", "2026-10-16T00:00:00Z");
    session.runtime_state.transport.is_playing = true;
    session.runtime_state.transport.position_beats = 12.0;
    JamAppState::from_parts(session, None, ActionQueue::new())
}

fn commit(state: &mut JamAppState, command: ActionCommand, target: Option<u64>) {
    let mut draft = ActionDraft::new(
        ActorType::User,
        command,
        Quantization::Immediate,
        ActionTarget::default(),
    );
    if let Some(position) = target {
        draft.params = ActionParams::Transport {
            position_beats: Some(position),
        };
    }
    state.queue.enqueue(draft, 100).unwrap();
    state
        .commit_ready_actions(
            state
                .runtime
                .transport
                .boundary_state(CommitBoundary::Immediate),
            110,
        )
        .unwrap();
}

#[test]
fn committed_seek_is_delivered_before_old_observation_and_waits_for_callback() {
    let mut state = app();
    let runtime = Control::default();
    runtime.complete(12.5);
    commit(&mut state, ActionCommand::TransportSeek, Some(8));
    let session = state.session.clone();
    assert!(
        sync_transport(&mut state, &runtime, 120)
            .unwrap()
            .is_empty()
    );
    assert_eq!(state.session, session);
    assert_eq!(
        runtime.updates.borrow()[0],
        AudioTransportPositionUpdate::Reposition(AudioTransportPositionRequest::from_beats(8.0))
    );
    assert!(
        state
            .runtime
            .transport_driver
            .pending_audio_position
            .is_none()
    );

    // Neither a second poll nor Preserve resends/restarts the admitted request.
    sync_transport(&mut state, &runtime, 130).unwrap();
    assert_eq!(state.session, session);
    assert_eq!(
        runtime
            .updates
            .borrow()
            .iter()
            .filter(|update| matches!(update, AudioTransportPositionUpdate::Reposition(_)))
            .count(),
        1
    );
    runtime.complete(8.25);
    sync_transport(&mut state, &runtime, 140).unwrap();
    assert_eq!(state.runtime.transport.position_beats, 8.25);
    assert_eq!(state.session.runtime_state.transport.position_beats, 8.25);
}

#[test]
fn failed_delivery_retains_committed_intent_and_never_reads_old_timing() {
    let mut state = app();
    let runtime = Control::default();
    runtime.complete(12.5);
    commit(&mut state, ActionCommand::TransportSeek, Some(8));
    let session = state.session.clone();
    let app_runtime = state.runtime.clone();
    runtime
        .failure
        .set(Some(AudioTransportControlError::PositionRequestExhausted));
    assert!(sync_transport(&mut state, &runtime, 120).is_err());
    assert_eq!(state.session, session);
    assert_eq!(state.runtime, app_runtime);
    assert!(runtime.updates.borrow().is_empty());
}

#[test]
fn stop_and_a_second_same_position_seek_remain_explicit_requests() {
    let mut state = app();
    let runtime = Control::default();
    for (command, position) in [
        (ActionCommand::TransportStop, None),
        (ActionCommand::TransportSeek, Some(0)),
        (ActionCommand::TransportSeek, Some(0)),
    ] {
        commit(&mut state, command, position);
        publish_transport(&mut state, &runtime).unwrap();
    }
    assert_eq!(
        *runtime.updates.borrow(),
        vec![
            AudioTransportPositionUpdate::Reposition(AudioTransportPositionRequest::from_beats(
                0.0
            ));
            3
        ]
    );
    assert_eq!(state.session.action_log.commit_records.len(), 3);
}

#[test]
fn ordinary_progress_publishes_only_preserve_and_no_new_session_action() {
    let mut state = app();
    let runtime = Control::default();
    runtime.complete(12.5);
    sync_transport(&mut state, &runtime, 120).unwrap();
    assert_eq!(state.runtime.transport.position_beats, 12.5);
    assert_eq!(
        *runtime.updates.borrow(),
        vec![AudioTransportPositionUpdate::Preserve; 2]
    );
    assert!(state.session.action_log.actions.is_empty());
}

#[test]
fn a_boundary_seek_is_delivered_before_stale_timing_can_commit_later_queue_work() {
    let mut state = app();
    let runtime = Control::default();
    let mut seek = ActionDraft::new(
        ActorType::User,
        ActionCommand::TransportSeek,
        Quantization::NextBeat,
        ActionTarget::default(),
    );
    seek.params = ActionParams::Transport {
        position_beats: Some(4),
    };
    state.queue.enqueue(seek, 100).unwrap();
    runtime.complete(13.0);
    assert_eq!(sync_transport(&mut state, &runtime, 120).unwrap().len(), 1);
    assert_eq!(state.runtime.transport.position_beats, 4.0);
    assert!(runtime.awaiting_callback.get());
    state
        .queue
        .enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::TransportPause,
                Quantization::NextBeat,
                ActionTarget::default(),
            ),
            130,
        )
        .unwrap();
    let session = state.session.clone();
    assert!(
        sync_transport(&mut state, &runtime, 140)
            .unwrap()
            .is_empty()
    );
    assert_eq!(state.session, session);
    assert_eq!(state.queue.pending_actions().len(), 1);
    runtime.complete(4.25);
    assert!(
        sync_transport(&mut state, &runtime, 150)
            .unwrap()
            .is_empty()
    );
    assert_eq!(state.queue.pending_actions().len(), 1);
    runtime.complete(5.0);
    assert_eq!(sync_transport(&mut state, &runtime, 160).unwrap().len(), 1);
    assert!(!state.runtime.transport.is_playing);
    assert!(state.queue.pending_actions().is_empty());
}

#[test]
fn same_runtime_restore_delivers_the_fractional_session_cursor_before_old_timing() {
    for playing in [false, true] {
        let mut state = app();
        state.session.runtime_state.transport.is_playing = playing;
        state.session.runtime_state.transport.position_beats = 12.5;
        state.apply_restore_target_suffix(0).unwrap();
        assert_eq!(
            state.runtime.transport_driver.pending_audio_position,
            Some(AudioTransportPositionRequest::from_beats(12.5))
        );
        let runtime = Control::default();
        runtime.complete(18.0);
        let session = state.session.clone();
        sync_transport(&mut state, &runtime, 120).unwrap();
        assert_eq!(state.session, session);
        assert_eq!(
            runtime.updates.borrow()[0],
            AudioTransportPositionUpdate::Reposition(AudioTransportPositionRequest::from_beats(
                12.5
            ))
        );
    }
}
