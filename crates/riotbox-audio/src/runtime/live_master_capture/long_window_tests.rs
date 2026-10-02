//! Generated-PCM proof of the generic tap's bounded 8/16-bar window behavior.
//! This does not exercise a host stream or provide source/listening evidence.

use super::{
    LIVE_MASTER_CALLBACK_GAP_THRESHOLD_MICROS, LIVE_MASTER_MAX_INTERLEAVED_SAMPLE_COUNT,
    LiveMasterCaptureError, LiveMasterCaptureOutcome, LiveMasterCaptureRequest,
    SharedLiveMasterCapture,
};
use crate::runtime::CallbackTimingSnapshot;
use crate::runtime::shared_transport_tr909::RealtimeTransportTimingState;
use crate::runtime::shared_w30_resample_callback::{
    TransportTimingCallbackState, advance_transport_timing,
};
use riotbox_core::action::LiveRecordingDuration;

struct SyntheticWindow {
    request: LiveMasterCaptureRequest,
    sample_rate: u32,
    origin_beats: f64,
}

impl SyntheticWindow {
    fn new(bar_count: u32, sample_rate: u32, tempo_bpm: f32, channel_count: u16) -> Self {
        let target_frame_count = (f64::from(bar_count * 4) * 60.0 * f64::from(sample_rate)
            / f64::from(tempo_bpm))
        .round() as usize;
        Self {
            request: LiveMasterCaptureRequest {
                target_frame_count,
                channel_count,
                expected_tempo_bpm: tempo_bpm,
                start_position_beats: Some(4.0),
            },
            sample_rate,
            origin_beats: 4.0,
        }
    }

    fn begin(&self) -> SharedLiveMasterCapture {
        let shared = SharedLiveMasterCapture::new();
        shared
            .begin(self.request.clone())
            .expect("begin long window");
        shared
    }

    fn beats_per_frame(&self) -> f64 {
        f64::from(self.request.expected_tempo_bpm) / (60.0 * f64::from(self.sample_rate))
    }

    fn position(&self, frame: usize) -> f64 {
        self.origin_beats + frame as f64 * self.beats_per_frame()
    }

    fn timing(&self, first_frame: usize, frame_count: usize) -> CallbackTimingSnapshot {
        CallbackTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: self.request.expected_tempo_bpm,
            render_position_beats: self.position(first_frame),
            completed_position_beats: self.position(first_frame + frame_count),
        }
    }

    fn callback_micros(&self, end_frame: usize) -> u64 {
        1_000 + end_frame as u64 * 1_000_000 / u64::from(self.sample_rate)
    }

    fn samples(&self, first_frame: usize, frame_count: usize) -> Vec<f32> {
        let channels = usize::from(self.request.channel_count);
        (first_frame * channels..(first_frame + frame_count) * channels)
            .map(sample_at)
            .collect()
    }

    fn record_frames(
        &self,
        shared: &SharedLiveMasterCapture,
        first_frame: usize,
        frame_count: usize,
    ) {
        shared.record_callback(
            &self.samples(first_frame, frame_count),
            &self.timing(first_frame, frame_count),
            self.callback_micros(first_frame + frame_count),
        );
    }

    fn record_prefix(&self, shared: &SharedLiveMasterCapture, end_frame: usize) {
        for first_frame in (0..end_frame).step_by(1024) {
            self.record_frames(shared, first_frame, 1024.min(end_frame - first_frame));
        }
    }
}

fn sample_at(interleaved_index: usize) -> f32 {
    // A unique, exactly representable binary fraction for every fixture sample.
    // Neither channel can be silently duplicated or shifted by a whole frame.
    interleaved_index as f32 / 16_777_216.0
}

fn capture_partitioned(window: &SyntheticWindow, partitions: &[usize]) -> LiveMasterCaptureOutcome {
    let shared = window.begin();
    window.record_frames(&shared, 0, 8);
    let waiting = shared.progress().expect("armed progress");
    assert_eq!(waiting.written_sample_count, 0);
    assert!(!waiting.capture_started);

    // The requested bar begins 17.25 frames after the fixture origin, so the
    // first complete frame at/after that boundary is frame 18.
    let end_frame = 18 + window.request.target_frame_count;
    let mut first_frame = 8;
    for requested_count in partitions.iter().copied().cycle() {
        let remaining = end_frame - first_frame;
        let frame_count = if requested_count >= remaining {
            remaining + 3
        } else {
            requested_count
        };
        window.record_frames(&shared, first_frame, frame_count);
        first_frame += frame_count;
        if first_frame > end_frame {
            break;
        }
    }

    let complete = shared.progress().expect("complete progress");
    assert!(complete.complete);
    // Already-complete capture must not absorb a subsequent callback or faults
    // from its unrelated transport state.
    shared.record_callback(
        &[0.99, -0.99],
        &CallbackTimingSnapshot {
            is_transport_running: false,
            tempo_bpm: 1.0,
            render_position_beats: 0.0,
            completed_position_beats: 0.0,
        },
        window.callback_micros(first_frame) + LIVE_MASTER_CALLBACK_GAP_THRESHOLD_MICROS + 1,
    );
    assert_eq!(shared.progress(), Some(complete));
    shared.finish().expect("complete long window")
}

#[test]
fn eight_and_sixteen_bar_windows_keep_exact_complete_frames_across_callback_partitions() {
    for bar_count in [8, 16] {
        for sample_rate in [44_100, 48_000, 96_000] {
            for tempo_bpm in [120.0, 137.25] {
                let mut window = SyntheticWindow::new(bar_count, sample_rate, tempo_bpm, 2);
                window.origin_beats -= 17.25 * window.beats_per_frame();
                let regular = capture_partitioned(&window, &[257]);
                let varied = capture_partitioned(&window, &[1, 127, 511, 2048, 13]);

                assert_eq!(regular.samples, varied.samples);
                let channels = usize::from(window.request.channel_count);
                let expected_count = window.request.target_frame_count * channels;
                let expected_start = window.position(18);
                let expected_end = window.position(18 + window.request.target_frame_count);
                let tolerance = window.beats_per_frame() * 1.0e-6;
                for outcome in [&regular, &varied] {
                    assert_eq!(outcome.samples.len(), expected_count);
                    assert_eq!(outcome.progress.target_sample_count, expected_count);
                    assert_eq!(outcome.progress.written_sample_count, expected_count);
                    assert_eq!(outcome.progress.fault_count(), 0);
                    assert!(outcome.progress.capture_started);
                    assert!(outcome.progress.complete);
                    assert!(outcome.progress.armed_callback_count >= 2);
                    let start = outcome.captured_start_position_beats.expect("actual start");
                    let end = outcome.captured_end_position_beats.expect("actual end");
                    assert!((start - expected_start).abs() < tolerance);
                    assert!((end - expected_end).abs() < tolerance);
                    assert!(start >= 4.0);
                    assert!(start - 4.0 <= window.beats_per_frame());
                    assert!(
                        (end - start - f64::from(bar_count * 4)).abs() <= window.beats_per_frame()
                    );
                }
                assert!(
                    regular
                        .samples
                        .iter()
                        .enumerate()
                        .all(|(index, sample)| *sample == sample_at(18 * channels + index))
                );
            }
        }
    }
}

#[test]
fn sixteen_bar_capture_from_the_actual_runtime_clock_meets_the_publication_end_position_gate() {
    for (partitions, old_gate_rejects) in [
        (&[64][..], true),
        (&[128][..], true),
        (&[1024][..], false),
        (&[1, 127, 511, 2048, 13][..], false),
    ] {
        let window = SyntheticWindow::new(16, 96_000, 137.25, 2);
        let shared = window.begin();
        let control = RealtimeTransportTimingState {
            is_transport_running: true,
            tempo_bpm: window.request.expected_tempo_bpm,
            position_beats: 0.0,
        };
        let mut clock = TransportTimingCallbackState::default();
        let mut first_frame = 0;
        for callback_frames in partitions.iter().copied().cycle() {
            let timing =
                advance_transport_timing(&control, &mut clock, window.sample_rate, callback_frames);
            shared.record_callback(
                &window.samples(first_frame, callback_frames),
                &timing,
                window.callback_micros(first_frame + callback_frames),
            );
            first_frame += callback_frames;
            let progress = shared.progress().expect("healthy clock capture");
            assert_eq!(progress.fault_count(), 0);
            assert!(first_frame < window.request.target_frame_count * 2);
            if progress.complete {
                break;
            }
        }
        let outcome = shared.finish().expect("complete runtime-clock capture");
        let channels = usize::from(window.request.channel_count);
        assert_eq!(
            outcome.samples.len(),
            window.request.target_frame_count * channels
        );
        assert_eq!(outcome.progress.written_sample_count, outcome.samples.len());
        assert_eq!(outcome.progress.fault_count(), 0);
        let start = outcome.captured_start_position_beats.expect("actual start");
        let end = outcome.captured_end_position_beats.expect("actual end");
        let beats_per_frame =
            f64::from(window.request.expected_tempo_bpm) / 60.0 / f64::from(window.sample_rate);
        let nominal_duration = beats_per_frame * window.request.target_frame_count as f64;
        assert!((nominal_duration - 64.0).abs() / beats_per_frame <= 0.500_001);
        assert!(start >= 4.0 && start - 4.0 <= beats_per_frame * (1.0 + 1.0e-6));
        let expected_end = start + nominal_duration;
        let end_error = (end - expected_end).abs();
        if old_gate_rejects {
            // Preserve the actual-clock RED evidence: the old flat epsilon
            // rejected these complete, fault-free 64/128-frame callback runs.
            assert!(end_error > beats_per_frame * 1.0e-6);
        }
        let position_bound = LiveRecordingDuration::SixteenBars
            .position_roundoff_bound(
                beats_per_frame,
                start,
                end,
                expected_end,
                outcome.progress.callback_count,
                window.request.target_frame_count as u64,
            )
            .expect("bounded V3 representation error");
        assert!(
            end_error <= position_bound,
            "actual runtime clock misses V3 publication bound: partitions={partitions:?}, error_frames={:?}, bound_frames={:?}",
            end_error / beats_per_frame,
            position_bound / beats_per_frame,
        );
    }
}

#[derive(Clone, Copy, Debug)]
enum LateFault {
    TransportStop,
    TempoChange,
    TimingDiscontinuity,
    ScratchOverflow,
    StreamError,
    CallbackGap,
}

#[test]
fn long_windows_keep_late_faults_and_refuse_incomplete_finalization() {
    for bar_count in [8, 16] {
        for fault in [
            LateFault::TransportStop,
            LateFault::TempoChange,
            LateFault::TimingDiscontinuity,
            LateFault::ScratchOverflow,
            LateFault::StreamError,
            LateFault::CallbackGap,
        ] {
            let window = SyntheticWindow::new(bar_count, 48_000, 137.25, 2);
            let shared = window.begin();
            let remaining = 1024;
            let first_frame = window.request.target_frame_count - remaining;
            window.record_prefix(&shared, first_frame);
            let before = shared.progress().expect("late-window progress");
            assert_eq!(before.fault_count(), 0);
            assert!(!before.complete);
            assert!(before.capture_started);
            let mut timing = window.timing(first_frame, remaining);
            let mut now_micros = window.callback_micros(first_frame + remaining);
            match fault {
                LateFault::TransportStop => {
                    timing.is_transport_running = false;
                    timing.completed_position_beats = timing.render_position_beats;
                }
                LateFault::TempoChange => timing.tempo_bpm += 1.0,
                LateFault::TimingDiscontinuity => {
                    timing = window.timing(first_frame + 1, remaining);
                }
                LateFault::ScratchOverflow => {
                    shared.record_scratch_overflow(&timing, now_micros);
                }
                LateFault::StreamError => shared.record_stream_error(),
                LateFault::CallbackGap => {
                    now_micros += LIVE_MASTER_CALLBACK_GAP_THRESHOLD_MICROS + 1;
                }
            }
            if !matches!(fault, LateFault::ScratchOverflow) {
                shared.record_callback(
                    &window.samples(first_frame, remaining),
                    &timing,
                    now_micros,
                );
            }

            let progress = shared.progress().expect("late fault progress");
            let observed_count = match fault {
                LateFault::TransportStop => progress.transport_mismatch_count,
                LateFault::TempoChange => progress.tempo_mismatch_count,
                LateFault::TimingDiscontinuity => progress.timing_window_mismatch_count,
                LateFault::ScratchOverflow => progress.callback_scratch_overflow_count,
                LateFault::StreamError => progress.stream_error_count,
                LateFault::CallbackGap => progress.callback_gap_over_threshold_count,
            };
            assert_eq!(observed_count, 1, "{bar_count} bars: {fault:?}");
            assert_eq!(progress.fault_count(), 1, "{bar_count} bars: {fault:?}");
            if matches!(fault, LateFault::StreamError | LateFault::CallbackGap) {
                // These faults are evidence for the control-thread acceptance
                // gate, not permission for the tap to discard their counters.
                let outcome = shared.finish().expect("complete but faulted payload");
                assert_eq!(outcome.progress, progress);
                assert!(outcome.progress.complete);
                assert_eq!(outcome.samples.len(), progress.target_sample_count);
            } else {
                assert_eq!(progress.written_sample_count, before.written_sample_count);
                assert!(!progress.complete);
                assert_eq!(
                    shared.finish(),
                    Err(LiveMasterCaptureError::NotComplete(progress.clone()))
                );
                assert_eq!(shared.abort(), Some(progress));
                assert!(shared.progress().is_none());
            }
        }
    }
}

#[test]
fn long_windows_cannot_finalize_one_frame_early_and_abort_drops_the_active_payload() {
    for bar_count in [8, 16] {
        let window = SyntheticWindow::new(bar_count, 44_100, 97.5, 1);
        let shared = window.begin();
        window.record_prefix(&shared, window.request.target_frame_count - 1);
        let progress = shared.progress().expect("incomplete long window");
        assert_eq!(
            progress.written_sample_count,
            progress.target_sample_count - 1
        );
        assert_eq!(progress.fault_count(), 0);
        assert!(!progress.complete);
        assert_eq!(
            shared.finish(),
            Err(LiveMasterCaptureError::NotComplete(progress.clone()))
        );
        assert_eq!(shared.abort(), Some(progress));
        window.record_frames(&shared, window.request.target_frame_count - 1, 1);
        assert!(shared.progress().is_none());
        assert_eq!(shared.finish(), Err(LiveMasterCaptureError::NotActive));

        shared
            .begin(window.request.clone())
            .expect("rearm after abort");
        let rearmed = shared.progress().expect("new armed window");
        assert_eq!(rearmed.written_sample_count, 0);
        assert_eq!(rearmed.callback_count, 0);
        assert_eq!(rearmed.fault_count(), 0);
        assert!(!rearmed.capture_started);
        assert!(!rearmed.complete);
        shared.abort().expect("clean up rearmed window");
    }
}

#[test]
fn long_window_requests_still_reject_the_existing_allocation_cap_and_overflow() {
    let shared = SharedLiveMasterCapture::new();
    let mut requests = vec![
        SyntheticWindow::new(8, 192_000, 20.0, 2).request,
        SyntheticWindow::new(16, 192_000, 20.0, 2).request,
    ];
    for target_frame_count in [LIVE_MASTER_MAX_INTERLEAVED_SAMPLE_COUNT / 2 + 1, usize::MAX] {
        requests.push(LiveMasterCaptureRequest {
            target_frame_count,
            channel_count: 2,
            expected_tempo_bpm: 137.25,
            start_position_beats: Some(4.0),
        });
    }
    for request in requests {
        assert_eq!(
            shared.begin(request),
            Err(LiveMasterCaptureError::InvalidTarget)
        );
        assert!(shared.progress().is_none());
    }
}
