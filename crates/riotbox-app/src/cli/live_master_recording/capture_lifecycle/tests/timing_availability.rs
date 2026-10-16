//! Admission of unavailable and first-observed callback timing by the real controller.

use std::time::Duration;

use riotbox_core::action::LiveRecordingDuration;

use super::super::{
    RuntimeControl, execute_with_control, sync_live_master_runtime, wait_for_live_transport,
};
use super::{Case, Event, fixture};

#[test]
fn unavailable_timing_preserves_restored_transport_and_pending_acknowledgment() {
    let (_temp, _launch, mut state, control) = fixture(
        Case::TimingUnavailableDelayed,
        LiveRecordingDuration::TwoBars,
        false,
    );
    state.runtime.transport.position_beats = 12.5;
    state.set_transport_playing(true);
    state
        .queue_tr909_fill(900)
        .expect("generated action ID available");
    control.position_offset.set(12.5);
    let session = state.session.clone();
    let runtime = state.runtime.clone();
    let pending = state
        .queue
        .pending_actions()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    assert!(control.timing_snapshot().is_none());
    sync_live_master_runtime(&mut state, &control);

    assert_eq!(state.session, session);
    assert_eq!(state.runtime, runtime);
    assert_eq!(
        state.queue.pending_actions(),
        pending.iter().collect::<Vec<_>>()
    );
    assert_eq!(
        state.runtime.transport_driver.pending_audio_is_playing,
        Some(true)
    );
    assert_eq!(control.elapsed.get(), Duration::ZERO);
    assert!(control.events.borrow().is_empty());

    wait_for_live_transport(&mut state, &control).unwrap();

    assert_eq!(control.elapsed.get(), Duration::from_millis(100));
    assert_eq!(state.runtime.transport.position_beats, 12.7);
    assert_eq!(state.session.runtime_state.transport.position_beats, 12.7);
    assert_eq!(
        state.runtime.transport_driver.pending_audio_is_playing,
        None
    );
    assert_eq!(
        state.queue.pending_actions(),
        pending.iter().collect::<Vec<_>>()
    );
    assert!(control.events.borrow().is_empty());
}

#[test]
fn unavailable_timing_waits_for_observation_before_arming_and_publishing() {
    let (_temp, launch, mut state, mut control) = fixture(
        Case::TimingUnavailableDelayed,
        LiveRecordingDuration::TwoBars,
        false,
    );
    let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();

    assert_eq!(summary["ready"], true);
    assert_eq!(summary["runtime_stopped"], true);
    assert!(control.elapsed.get() >= Duration::from_millis(100));
    assert_eq!(
        *control.events.borrow(),
        [Event::Begin, Event::Stop, Event::Finish]
    );
    assert_eq!(state.session.export_receipts.len(), 1);
    assert!(state.session.export_receipts[0].live_recording_runtime_master_ready());
}
