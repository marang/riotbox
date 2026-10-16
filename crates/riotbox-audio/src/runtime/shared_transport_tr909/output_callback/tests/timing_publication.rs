//! Generated PCM through the actual callback, not a replacement timing producer.
use super::{config, prepare_output_callback, shared, source};
use crate::{
    runtime::AudioRuntimeTimingSnapshot,
    test_heap::{HeapCounts, measure},
};
use std::time::Instant;

#[test]
fn actual_callback_observes_restored_position_and_play_pause_without_heap_work() {
    let state = shared();
    let mut source = source(0.25);
    source.source_anchor_seconds = Some(0.0);
    source.source_anchor_position_beats = 12.5;
    state.source_monitor.replace_source_and_controls(&source);
    state.transport.update(false, 120.0, 12.5);
    let mut callback = prepare_output_callback::<f32>(&config(), state.clone(), Instant::now())
        .expect("unique production callback");
    let mut output = [0.0; 128];

    assert_eq!(state.telemetry.timing_snapshot(), None);
    let (_, heap) = measure(|| callback(&mut output));
    assert_eq!(heap, HeapCounts::default());
    assert!(output.iter().all(|sample| *sample == 0.0));
    assert_eq!(
        state.telemetry.timing_snapshot(),
        Some(AudioRuntimeTimingSnapshot {
            is_transport_running: false,
            tempo_bpm: 120.0,
            position_beats: 12.5,
        })
    );

    state.transport.update(true, 120.0, 12.5);
    let (_, heap) = measure(|| callback(&mut output));
    assert_eq!(heap, HeapCounts::default());
    assert!(output.iter().any(|sample| *sample > 0.01));
    let running = state
        .telemetry
        .timing_snapshot()
        .expect("observed running callback");
    assert!(running.is_transport_running);
    assert_eq!(running.tempo_bpm, 120.0);
    // Worked geometry: 64 frames at 48 kHz and 120 BPM advance 0.002666... beats.
    assert!((running.position_beats - 12.502_666_666_666_667).abs() < 1e-12);

    state.transport.update(false, 120.0, running.position_beats);
    for _ in 0..5 {
        let (_, heap) = measure(|| callback(&mut output));
        assert_eq!(heap, HeapCounts::default());
    }
    assert!(
        output.iter().all(|sample| *sample == 0.0),
        "existing stop ramp must finish silent"
    );
    let stopped = state
        .telemetry
        .timing_snapshot()
        .expect("observed paused callback");
    assert!(!stopped.is_transport_running);
    assert_eq!(stopped.position_beats, running.position_beats);
    assert_eq!(state.telemetry.snapshot().callback_count, 7);
}
