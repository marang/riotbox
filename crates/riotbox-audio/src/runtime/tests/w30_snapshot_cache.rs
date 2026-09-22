use crate::{
    runtime::{
        RealtimeW30PreviewRenderState, SharedW30PreviewRenderState, W30PreviewCallbackState,
        W30PreviewSnapshotCache, render_w30_preview_buffer,
    },
    w30::{
        W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN, W30PadPlaybackSampleWindow, W30PreviewRenderMode,
        W30PreviewRenderRouting, W30PreviewRenderState,
    },
};
use std::{hint::black_box, time::Instant};

fn fixture() -> W30PreviewRenderState {
    W30PreviewRenderState {
        mode: W30PreviewRenderMode::LiveRecall,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        trigger_revision: 1,
        trigger_velocity: 0.8,
        music_bus_level: 0.7,
        pad_playback: Some(W30PadPlaybackSampleWindow {
            source_start_frame: 0,
            source_end_frame: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN as u64,
            source_sample_rate: 48_000,
            playback_frame_count: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN as u64,
            sample_count: W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN,
            loop_enabled: true,
            playback_rate: 1.0,
            reverse: false,
            gate_step_fraction: 0.0,
            loop_crossfade_sample_count: 0,
            chop_slice_count: 0,
            chop_slice_starts: [0; 8],
            hook_articulation: None,
            samples: [0.25; W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN],
        }),
        ..W30PreviewRenderState::default()
    }
}

/// Informational release benchmark: no source/device access or CI time limit.
/// Changed-state writer cost is excluded; unchanged reads are batch-timed.
#[test]
#[ignore = "manual release-mode snapshot measurement"]
fn benchmark_w30_snapshot_cost() {
    let mut state = fixture();
    let shared = SharedW30PreviewRenderState::new(&state);
    let mut previous = shared.snapshot();
    let iterations = 2_000;
    for cached in [false, true] {
        let mut cache = W30PreviewSnapshotCache::new(&shared);
        for changed in [false, true] {
            let mut results = Vec::new();
            for _ in 0..7 {
                let mut elapsed = std::time::Duration::ZERO;
                let batch_start = Instant::now();
                for _ in 0..iterations {
                    if changed {
                        state.trigger_revision += 1;
                        shared.update(&state);
                    }
                    let start = changed.then(Instant::now);
                    if cached {
                        black_box(cache.refresh(&shared));
                    } else {
                        let snapshot = shared.snapshot_or_previous(&previous);
                        previous = snapshot;
                        black_box(&snapshot);
                    }
                    if let Some(start) = start {
                        elapsed += start.elapsed();
                    }
                }
                if !changed {
                    elapsed = batch_start.elapsed();
                }
                results.push(elapsed.as_nanos() / iterations);
            }
            results.sort_unstable();
            println!(
                "w30_snapshot cached={cached} changed={changed} median_ns={}",
                results[3]
            );
        }
    }
}

fn render(snapshot: &RealtimeW30PreviewRenderState) -> Vec<f32> {
    let mut data = vec![0.0; 256];
    let mut callback = W30PreviewCallbackState::with_sample_rate_and_channels(48_000, 1);
    render_w30_preview_buffer(&mut data, 48_000, 1, snapshot, &mut callback);
    data
}

#[test]
fn changed_publication_without_retrigger_changes_render_and_matches_uncached() {
    let mut state = fixture();
    let shared = SharedW30PreviewRenderState::new(&state);
    let mut cache = W30PreviewSnapshotCache::new(&shared);
    let original = render(cache.refresh(&shared));
    assert_eq!(original, render(&shared.snapshot()));
    assert!(original.iter().any(|sample| sample.abs() > 0.001));

    state.pad_playback.as_mut().unwrap().samples.fill(-0.4);
    shared.update(&state); // Deliberately keep trigger_revision unchanged.
    let changed = render(cache.refresh(&shared));
    assert_eq!(changed, render(&shared.snapshot()));
    assert_ne!(original, changed);

    // Per-callback timing may be overridden even while sample data is reused.
    cache.refresh(&shared).position_beats = 8.0;
    assert_eq!(cache.refresh(&shared).position_beats, 8.0);
    state.mode = W30PreviewRenderMode::Idle;
    shared.update(&state);
    assert_eq!(render(cache.refresh(&shared)), vec![0.0; 256]);
}
