//! Device-free timing availability through the production App transport consumer.
use crate::jam_app::JamAppState;
use riotbox_audio::runtime::AudioRuntimeTimingSnapshot;
use riotbox_core::{
    action::{ActionCommand, ActionDraft, ActionTarget, ActorType, Quantization, TargetScope},
    queue::ActionQueue,
    session::SessionFile,
};

fn restored_paused_state(id: &str) -> JamAppState {
    let mut session = SessionFile::new(id, "riotbox-test", "2026-10-16T00:00:00Z");
    session.runtime_state.transport.position_beats = 12.5;
    JamAppState::from_parts(session, None, ActionQueue::new())
}

#[test]
fn unobserved_audio_timing_preserves_a_paused_restored_cursor() {
    let mut state = restored_paused_state("unobserved-restored-cursor");
    assert!(!state.runtime.transport.is_playing);
    assert_eq!(state.runtime.transport.position_beats, 12.5);
    assert_eq!(
        state.runtime.transport_driver.pending_audio_is_playing,
        None
    );

    // The runtime withholds timing until a complete callback is observed.
    assert!(state.apply_audio_timing_snapshot(None, 100).is_empty());
    assert_eq!(state.runtime.transport.position_beats, 12.5);
    assert_eq!(state.session.runtime_state.transport.position_beats, 12.5);

    assert!(
        state
            .apply_audio_timing_snapshot(
                Some(AudioRuntimeTimingSnapshot {
                    is_transport_running: false,
                    tempo_bpm: 128.0,
                    position_beats: 12.5,
                }),
                200,
            )
            .is_empty()
    );
    assert_eq!(state.runtime.transport.position_beats, 12.5);
    assert_eq!(state.session.runtime_state.transport.position_beats, 12.5);
}

#[test]
fn unavailable_timing_cannot_acknowledge_play_or_pause_or_commit_queued_actions() {
    for requested_playing in [false, true] {
        let mut state = restored_paused_state("unavailable-pending-transport");
        state.set_transport_playing(requested_playing);
        state.queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::CaptureNow,
                Quantization::NextBar,
                ActionTarget {
                    scope: Some(TargetScope::LaneW30),
                    ..Default::default()
                },
            ),
            50,
        );
        let before_session = state.session.clone();
        let before_transport = state.runtime.transport.clone();
        let before_driver = state.runtime.transport_driver.clone();
        let before_queue: Vec<_> = state.queue.pending_actions().into_iter().cloned().collect();

        assert!(state.apply_audio_timing_snapshot(None, 100).is_empty());

        assert_eq!(state.session, before_session);
        assert_eq!(state.runtime.transport, before_transport);
        assert_eq!(state.runtime.transport_driver, before_driver);
        assert_eq!(
            state
                .queue
                .pending_actions()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>(),
            before_queue
        );
        assert!(state.runtime.last_commit_boundary.is_none());
    }
}

#[test]
fn observed_beat_zero_is_not_treated_as_missing_timing() {
    let mut state = restored_paused_state("observed-beat-zero");
    state.set_transport_playing(true);
    state.apply_audio_timing_snapshot(
        Some(AudioRuntimeTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 128.0,
            position_beats: 0.0,
        }),
        100,
    );
    assert!(state.runtime.transport.is_playing);
    assert_eq!(state.runtime.transport.position_beats, 0.0);
    assert_eq!(state.session.runtime_state.transport.position_beats, 0.0);
    assert_eq!(
        state.runtime.transport_driver.pending_audio_is_playing,
        None
    );
}
