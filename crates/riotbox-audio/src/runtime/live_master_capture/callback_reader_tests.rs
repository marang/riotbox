//! Generated capture publication/lifecycle checks using the unique callback
//! consumer. Timing qualification continues to use the same buffer operations.

use std::sync::Arc;

use super::{LiveMasterCaptureError, LiveMasterCaptureRequest, SharedLiveMasterCapture};
use crate::runtime::CallbackTimingSnapshot;

fn request(frame_count: usize) -> LiveMasterCaptureRequest {
    LiveMasterCaptureRequest {
        target_frame_count: frame_count,
        channel_count: 1,
        expected_tempo_bpm: 128.0,
        start_position_beats: None,
    }
}

fn timing() -> CallbackTimingSnapshot {
    CallbackTimingSnapshot {
        is_transport_running: true,
        tempo_bpm: 128.0,
        render_position_beats: 0.0,
        completed_position_beats: 0.25,
    }
}

#[test]
fn callback_reader_is_unique_even_after_it_is_dropped() {
    let shared = SharedLiveMasterCapture::new();
    let reader = shared.take_callback_reader().expect("unique reader");
    assert!(shared.take_callback_reader().is_none());
    drop(reader);
    assert!(shared.take_callback_reader().is_none());
}

#[test]
fn callback_reader_handles_empty_state_and_observes_the_latest_complete_publication() {
    let shared = SharedLiveMasterCapture::new();
    let mut reader = shared.take_callback_reader().unwrap();
    reader.record_callback(&[0.9], &timing(), 100);
    reader.record_scratch_overflow(&timing(), 200);
    assert!(shared.progress().is_none());

    shared.begin(request(1)).unwrap();
    let abandoned = shared.abort().unwrap();
    assert_eq!(abandoned.written_sample_count, 0);
    shared.begin(request(2)).unwrap();
    reader.record_callback(&[0.25, -0.5], &timing(), 1_000);

    let outcome = shared.finish().unwrap();
    assert_eq!(outcome.samples, [0.25, -0.5]);
    assert_eq!(outcome.progress.callback_count, 1);
    assert_eq!(outcome.progress.fault_count(), 0);
    reader.record_callback(&[0.9], &timing(), 2_000);
    assert!(shared.progress().is_none());
}

#[test]
fn callback_reader_preserves_exact_samples_and_scratch_fault_accounting() {
    let shared = SharedLiveMasterCapture::new();
    let mut reader = shared.take_callback_reader().unwrap();
    shared
        .begin(LiveMasterCaptureRequest {
            channel_count: 2,
            ..request(2)
        })
        .unwrap();
    reader.record_callback(&[0.25, -0.5], &timing(), 1_000);
    reader.record_scratch_overflow(&timing(), 2_000);
    shared.record_stream_error();
    reader.record_callback(&[0.75, -0.25, 0.9, 0.9], &timing(), 3_000);

    let outcome = shared.finish().unwrap();
    assert_eq!(outcome.samples, [0.25, -0.5, 0.75, -0.25]);
    assert_eq!(outcome.progress.callback_count, 3);
    assert_eq!(outcome.progress.callback_scratch_overflow_count, 1);
    assert_eq!(outcome.progress.stream_error_count, 1);
    assert_eq!(outcome.progress.fault_count(), 2);
}

#[test]
fn incomplete_finish_preserves_admission_for_the_remaining_samples() {
    let shared = SharedLiveMasterCapture::new();
    let mut reader = shared.take_callback_reader().unwrap();
    shared.begin(request(2)).unwrap();
    reader.record_callback(&[0.25], &timing(), 1_000);
    assert!(matches!(
        shared.finish(),
        Err(LiveMasterCaptureError::NotComplete(_))
    ));

    reader.record_callback(&[-0.5], &timing(), 2_000);
    assert_eq!(shared.finish().unwrap().samples, [0.25, -0.5]);
}

#[test]
fn finish_accepts_an_owned_but_quiescent_slot_and_closes_stale_timing_work() {
    let shared = SharedLiveMasterCapture::new();
    let mut reader = shared.take_callback_reader().unwrap();
    shared.begin(request(1)).unwrap();
    reader.record_callback(&[0.25], &timing(), 1_000);

    let stale = reader.output.read().as_ref().unwrap();
    let extra_owner = Arc::clone(stale);
    let progress = stale.progress();
    let outcome = shared.finish().expect("ownership is not callback activity");
    assert_eq!(outcome.samples, [0.25]);
    assert_eq!(outcome.progress, progress);
    stale.record_callback(&[0.9], &timing(), 2_000);
    stale.record_scratch_overflow(&timing(), 3_000);
    assert_eq!(stale.progress(), progress);
    assert!(stale.admission.try_enter().is_none());
    assert!(stale.admission.is_quiescent());
    drop(extra_owner);
}

#[test]
fn finish_rejects_and_retires_a_held_lease_without_waiting() {
    let shared = SharedLiveMasterCapture::new();
    let mut reader = shared.take_callback_reader().unwrap();
    shared.begin(request(1)).unwrap();
    reader.record_callback(&[0.25], &timing(), 1_000);

    let stale = reader.output.read().as_ref().unwrap();
    let lease = stale.admission.try_enter().expect("admitted callback work");
    let error = shared.finish().expect_err("held lease is still active");
    assert!(matches!(
        error,
        LiveMasterCaptureError::CallbackStillActive(_)
    ));
    assert!(shared.progress().is_none());
    assert!(stale.admission.try_enter().is_none());
    assert!(!stale.admission.is_quiescent());
    assert_eq!(shared.control.lock().unwrap().retired.len(), 1);
    drop(lease);
    assert!(stale.admission.is_quiescent());

    shared.begin(request(1)).unwrap();
    reader.record_callback(&[-0.5], &timing(), 2_000);
    assert_eq!(shared.finish().unwrap().samples, [-0.5]);
}

#[test]
fn abort_closes_a_stale_payload_with_a_held_lease_without_waiting() {
    let shared = SharedLiveMasterCapture::new();
    let mut reader = shared.take_callback_reader().unwrap();
    shared.begin(request(2)).unwrap();
    reader.record_callback(&[0.25], &timing(), 1_000);

    let stale = reader.output.read().as_ref().unwrap();
    let lease = stale.admission.try_enter().expect("admitted callback work");
    let aborted = shared.abort().expect("abort does not wait for a lease");
    assert!(!aborted.complete);
    assert_eq!(aborted.written_sample_count, 1);
    assert!(shared.progress().is_none());
    assert!(!stale.admission.is_quiescent());
    assert!(stale.admission.try_enter().is_none());

    stale.record_callback(&[0.9], &timing(), 2_000);
    stale.record_scratch_overflow(&timing(), 3_000);
    shared.record_stream_error();
    assert_eq!(stale.progress(), aborted);
    assert_eq!(shared.control.lock().unwrap().retired.len(), 1);
    drop(lease);
    assert!(stale.admission.is_quiescent());

    shared.begin(request(1)).unwrap();
    reader.record_callback(&[-0.5], &timing(), 4_000);
    let outcome = shared.finish().unwrap();
    assert_eq!(outcome.samples, [-0.5]);
    assert_eq!(outcome.progress.fault_count(), 0);
}

#[test]
fn advancing_the_reader_drops_no_slot_owners_and_control_eventually_reaps() {
    let shared = SharedLiveMasterCapture::new();
    let mut reader = shared.take_callback_reader().unwrap();
    shared.begin(request(1)).unwrap();
    reader.record_callback(&[0.25], &timing(), 1_000);
    let weak = Arc::downgrade(reader.output.read().as_ref().unwrap());
    shared.finish().unwrap();
    shared.begin(request(1)).unwrap();

    let owner_count = weak.strong_count();
    assert!(owner_count > 0);
    reader.record_callback(&[-0.5], &timing(), 2_000);
    assert_eq!(weak.strong_count(), owner_count);
    shared.abort().unwrap();

    // Rotate all three slots through producer ownership. Reaping is a control
    // operation after publication has released the old slot's owning Arc.
    for callback in 0..4 {
        shared.begin(request(1)).unwrap();
        reader.record_callback(&[0.5], &timing(), 3_000 + callback * 1_000);
        shared.abort().unwrap();
    }
    assert!(weak.upgrade().is_none());
}
