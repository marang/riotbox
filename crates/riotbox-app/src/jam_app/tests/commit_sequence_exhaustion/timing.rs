use riotbox_audio::runtime::AudioRuntimeTimingSnapshot;

use super::*;

fn crossed_boundary() -> CommitBoundaryState {
    CommitBoundaryState {
        kind: CommitBoundary::Beat,
        beat_index: 13,
        ..imported_boundary()
    }
}

fn timing() -> AudioRuntimeTimingSnapshot {
    AudioRuntimeTimingSnapshot {
        is_transport_running: true,
        tempo_bpm: 128.0,
        position_beats: 13.0,
    }
}

#[test]
fn advance_refuses_crossed_batch_before_clock_driver_or_session_changes() {
    let owner = tempdir().unwrap();
    let path = owner.path().join("advance.json");
    write_history_at(&path, u32::MAX, crossed_boundary());
    let mut state = load_source_free(&path);
    state.set_transport_playing(true);
    enqueue_pause(&mut state);
    assert_sequence_refusal(&mut state, &crossed_boundary(), 1, |state| {
        state.advance_transport_by(0.5, 20)
    });
}

#[test]
fn observed_snapshot_refuses_whole_batch_before_acknowledgment_or_clock_changes() {
    let owner = tempdir().unwrap();
    let path = owner.path().join("observed.json");
    write_history_at(&path, u32::MAX - 1, crossed_boundary());
    let mut state = load_source_free(&path);
    state.set_transport_playing(true);
    enqueue_pause(&mut state);
    enqueue_pause(&mut state);
    assert_eq!(
        state.runtime.transport_driver.pending_audio_is_playing,
        Some(true)
    );
    assert_sequence_refusal(&mut state, &crossed_boundary(), 2, |state| {
        state.apply_audio_timing_snapshot(Some(timing()), 20)
    });
    assert_eq!(
        state.runtime.transport_driver.pending_audio_is_playing,
        Some(true)
    );
}

#[test]
fn unavailable_or_stale_snapshot_remains_noop_and_empty_full_boundary_can_advance() {
    let owner = tempdir().unwrap();
    let path = owner.path().join("empty-observed.json");
    write_history_at(&path, u32::MAX, crossed_boundary());
    let mut state = load_source_free(&path);
    state.set_transport_playing(true);
    let before = state.runtime.clone();
    assert!(
        state
            .apply_audio_timing_snapshot(None, 20)
            .unwrap()
            .is_empty()
    );
    assert!(
        state
            .apply_audio_timing_snapshot(
                Some(AudioRuntimeTimingSnapshot {
                    is_transport_running: false,
                    ..timing()
                }),
                21
            )
            .unwrap()
            .is_empty()
    );
    assert_eq!(state.runtime, before);
    assert!(
        state
            .apply_audio_timing_snapshot(Some(timing()), 22)
            .unwrap()
            .is_empty()
    );
    assert_eq!(state.runtime.transport.position_beats, 13.0);
    assert_eq!(
        state.runtime.transport_driver.pending_audio_is_playing,
        None
    );
    assert_eq!(
        state.runtime.transport_driver.last_audio_position_beats,
        Some(13)
    );
}
