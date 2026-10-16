//! Existing Fill through the prepared production callback and its CaptureTap.
//! Green-on-arrival acceptance: no App input, ActionId delivery or device proof.
use std::time::Instant;

use riotbox_core::action::SourceMonitorMode;

use super::{prepare_output_callback, shared};
use crate::{
    runtime::{
        AudioTransportPositionRequest, AudioTransportPositionUpdate, LiveMasterCaptureRequest,
        SourceMonitorRenderState,
    },
    test_heap::{HeapCounts, measure},
    tr909::{Tr909FillRecipeId, Tr909RenderMode, Tr909RenderRouting, Tr909RenderState},
};

const SAMPLE_RATE: u32 = 48_000;
const CALLBACK_FRAMES: usize = 240;
// At 120 BPM, one 4/4 bar is exactly 96,000 mono frames. The eight-bar
// capture contains an unchanged first bar, one Fill bar, then its release.
const BAR_FRAMES: usize = 96_000;
const CAPTURE_FRAMES: usize = BAR_FRAMES * 8;
const FILL_START: usize = BAR_FRAMES;
const FILL_END: usize = BAR_FRAMES * 2;

#[test]
fn captured_fill_changes_only_its_published_bar_and_releases_to_the_no_fill_control() {
    let control = capture_window(false);
    let fill = capture_window(true);
    assert_eq!(control.len(), CAPTURE_FRAMES);
    assert_eq!(fill.len(), CAPTURE_FRAMES);
    assert!(control.iter().all(|sample| sample.to_bits() == 0));
    assert_pcm_bits_eq(&fill[..FILL_START], &control[..FILL_START]);
    assert!(
        fill[FILL_START..FILL_END]
            .iter()
            .zip(&control[FILL_START..FILL_END])
            .any(|(sample, baseline)| (sample - baseline).abs() > 0.0001),
        "the actual captured Fill bar must differ positively from the silent control"
    );
    assert_pcm_bits_eq(&fill[FILL_END..], &control[FILL_END..]);
    assert!(
        fill[FILL_END..].iter().all(|sample| sample.to_bits() == 0),
        "returning to the existing Idle state must leave no Fill tail in later bars"
    );
}

fn capture_window(with_fill: bool) -> Vec<f32> {
    let state = shared();
    // Every other contributor is its explicit default Idle/silent state.
    // Riotbox-only monitoring never supplies source PCM or masks the drum bus.
    state
        .source_monitor
        .update_controls(&SourceMonitorRenderState::control_only(
            SourceMonitorMode::Riotbox,
        ));
    state
        .transport
        .update(
            true,
            120.0,
            AudioTransportPositionUpdate::Reposition(AudioTransportPositionRequest::from_beats(
                0.0,
            )),
        )
        .unwrap();
    let idle = Tr909RenderState {
        routing: Tr909RenderRouting::DrumBusSupport,
        drum_bus_level: 0.6,
        is_transport_running: true,
        tempo_bpm: 120.0,
        ..Tr909RenderState::default()
    };
    let fill = Tr909RenderState {
        mode: Tr909RenderMode::Fill,
        ..idle.clone()
    };
    assert_eq!(
        fill.fill_recipe_id(),
        Some(Tr909FillRecipeId::GenericFillV1)
    );
    state.tr909_render.update(&idle);
    state
        .live_master_capture
        .begin(LiveMasterCaptureRequest {
            target_frame_count: CAPTURE_FRAMES,
            channel_count: 1,
            expected_tempo_bpm: 120.0,
            start_position_beats: Some(0.0),
        })
        .unwrap();
    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        buffer_size: cpal::BufferSize::Fixed(CALLBACK_FRAMES as u32),
    };
    let mut callback =
        Box::new(prepare_output_callback::<f32>(&config, state.clone(), Instant::now()).unwrap());
    // All buffer allocation and snapshot publication is outside measurement.
    let mut output = vec![0.0; CAPTURE_FRAMES];
    for (index, block) in output
        .as_chunks_mut::<CALLBACK_FRAMES>()
        .0
        .iter_mut()
        .enumerate()
    {
        let frame = index * CALLBACK_FRAMES;
        if frame == FILL_START {
            // Both runs publish on exactly the same block. Only Fill mode
            // differs; profile, routing, level, tempo and clock stay identical.
            state
                .tr909_render
                .update(if with_fill { &fill } else { &idle });
        } else if frame == FILL_END {
            state.tr909_render.update(&idle);
        }
        let (_, heap) = measure(|| callback(block));
        assert_eq!(heap, HeapCounts::default());
        assert!(block.iter().all(|sample| sample.is_finite()));
    }
    // End the exact prepared processor before consuming its existing tap.
    // This is not a claim that a device was started or stopped.
    drop(callback);
    let outcome = state.live_master_capture.finish().unwrap();
    assert!(outcome.progress.complete);
    assert_eq!(outcome.progress.target_sample_count, CAPTURE_FRAMES);
    assert_eq!(outcome.progress.written_sample_count, CAPTURE_FRAMES);
    assert_eq!(outcome.progress.transport_mismatch_count, 0);
    assert_eq!(outcome.progress.tempo_mismatch_count, 0);
    assert_eq!(outcome.progress.timing_window_mismatch_count, 0);
    assert_eq!(outcome.progress.stream_error_count, 0);
    assert_eq!(outcome.progress.callback_scratch_overflow_count, 0);
    assert_eq!(outcome.captured_start_position_beats, Some(0.0));
    assert!((outcome.captured_end_position_beats.unwrap() - 32.0).abs() < 1e-9);
    // A scheduled test process can have callback gaps. This proves PCM/tap
    // identity and zero heap, not host scheduling or capture-readiness gates.
    assert_pcm_bits_eq(&outcome.samples, &output);
    outcome.samples
}

fn assert_pcm_bits_eq(actual: &[f32], expected: &[f32]) {
    assert_eq!(actual.len(), expected.len());
    for (frame, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "PCM differs at frame {frame}"
        );
    }
}
