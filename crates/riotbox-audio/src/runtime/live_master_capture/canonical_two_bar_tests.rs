//! Source-free production-clock/tap evidence for the canonical V4 two-bar window.
use super::{LiveMasterCaptureRequest, SharedLiveMasterCapture};
use crate::runtime::shared_transport_tr909::RealtimeTransportTimingState;
use crate::runtime::shared_w30_resample_callback::{
    TransportTimingCallbackState, advance_transport_timing,
};
use riotbox_core::{action::LiveRecordingDuration, session::ExportLiveRecordingTimingWindow};

#[test]
fn canonical_two_bar_ties_from_the_actual_clock_keep_full_timing_readiness() {
    for (sample_rate, bpm) in [
        (48_000, 121.5_f32),
        (48_000, 166.5),
        (44_100, 80.6632),
        (44_100, 80.6838),
        (96_000, 137.25),
    ] {
        for partitions in [&[128][..], &[1024][..], &[1, 127, 511, 2048, 13][..]] {
            let duration = LiveRecordingDuration::TwoBars;
            let micros = (f64::from(bpm) * 1_000_000.0).round() as u64;
            let frame_count = duration.target_frame_count(sample_rate, micros).unwrap();
            let shared = SharedLiveMasterCapture::new();
            shared
                .begin(LiveMasterCaptureRequest {
                    target_frame_count: frame_count as usize,
                    channel_count: 2,
                    expected_tempo_bpm: bpm,
                    start_position_beats: Some(4.0),
                })
                .unwrap();
            let control = RealtimeTransportTimingState {
                is_transport_running: true,
                tempo_bpm: bpm,
                position_beats: 0.0,
            };
            let mut clock = TransportTimingCallbackState::default();
            let mut rendered = 0_u64;
            for count in partitions.iter().copied().cycle() {
                let timing = advance_transport_timing(&control, &mut clock, sample_rate, count);
                rendered += count as u64;
                shared.record_callback(
                    &vec![0.25; count * 2],
                    &timing,
                    1_000 + rendered * 1_000_000 / u64::from(sample_rate),
                );
                let progress = shared.progress().unwrap();
                assert_eq!(
                    progress.fault_count(),
                    0,
                    "{sample_rate}/{bpm}/{partitions:?}"
                );
                assert!(rendered < frame_count * 2);
                if progress.complete {
                    break;
                }
            }
            let outcome = shared.finish().unwrap();
            assert_eq!(outcome.samples.len() as u64, frame_count * 2);
            let start = outcome.captured_start_position_beats.unwrap();
            let end = outcome.captured_end_position_beats.unwrap();
            let span = f64::from(bpm) / 60.0 / f64::from(sample_rate);
            let nominal_duration = span * frame_count as f64;
            let expected_end = start + nominal_duration;
            let bound = duration
                .position_roundoff_bound(
                    span,
                    start,
                    end,
                    expected_end,
                    outcome.progress.callback_count,
                    frame_count,
                )
                .unwrap();
            assert!((end - expected_end).abs() <= bound);
            let start_error = (start - 4.0) / span;
            let duration_error = (nominal_duration - 8.0).abs() / span;
            assert!((0.0..=1.000_001).contains(&start_error));
            assert!(duration_error <= 0.500_001);
            let evidence = ExportLiveRecordingTimingWindow {
                confirmed_bpm_micros: micros,
                bar_grid_anchor_position_microbeats: 0,
                beat_span_per_frame_nanobeats: (span * 1_000_000_000.0).round() as u64,
                requested_start_position_microbeats: 4_000_000,
                captured_start_position_microbeats: (start * 1_000_000.0).round() as u64,
                captured_end_position_microbeats: (end * 1_000_000.0).round() as u64,
                start_alignment_error_frame_micros: (start_error * 1_000_000.0).round() as u64,
                duration_error_frame_micros: (duration_error * 1_000_000.0).round() as u64,
                beats_per_bar: 4,
                duration_beats: 8,
            };
            assert!(
                evidence.canonical_runtime_bar_window_ready(sample_rate, frame_count, duration),
                "{sample_rate}/{bpm}/{partitions:?}: {evidence:?}"
            );
        }
    }
}
