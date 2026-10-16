//! Deterministic production arming/callback tests. No device or wall-clock
//! callback scheduling: only control allocation is gated, with modeled clocks.

use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use super::{LIVE_MASTER_CALLBACK_GAP_THRESHOLD_MICROS, LiveMasterCaptureRequest};
use crate::{
    mc202::Mc202RenderState,
    runtime::{
        AudioRuntimeLifecycle, AudioRuntimeShell, CallbackTimingSnapshot,
        shared_mc202::SharedMc202RenderState,
        shared_transport_tr909::{
            AudioRuntimeShellTestParts, SharedTr909RenderState, SharedTransportTimingState,
        },
        shared_w30_resample_callback::SharedW30ResampleTapState,
        source_monitor::{SharedSourceMonitorRenderState, SourceMonitorRenderState},
        telemetry::RuntimeTelemetry,
        w30_preview_snapshot::SharedW30PreviewRenderState,
    },
    test_heap::{AllocationPause, with_allocation_paused},
    tr909::Tr909RenderState,
    w30::{W30PreviewRenderState, W30ResampleTapState},
};

fn shell() -> AudioRuntimeShell {
    AudioRuntimeShell::from_test_parts(AudioRuntimeShellTestParts {
        lifecycle: AudioRuntimeLifecycle::Running,
        output: None,
        telemetry: Arc::new(RuntimeTelemetry::new()),
        transport: Arc::new(SharedTransportTimingState::new(true, 120.0, 0.0)),
        tr909_render: Arc::new(SharedTr909RenderState::new(&Tr909RenderState::default())),
        mc202_render: Arc::new(SharedMc202RenderState::new(&Mc202RenderState::default())),
        w30_preview: Arc::new(SharedW30PreviewRenderState::new(
            &W30PreviewRenderState::default(),
        )),
        w30_resample_tap: Arc::new(SharedW30ResampleTapState::new(
            &W30ResampleTapState::default(),
        )),
        source_monitor: Arc::new(SharedSourceMonitorRenderState::new(
            &SourceMonitorRenderState::default(),
        )),
    })
}

fn timing(index: u64) -> CallbackTimingSnapshot {
    // 480 mono frames / 48 kHz = 10 ms = .02 beats at 120 BPM.
    CallbackTimingSnapshot {
        is_transport_running: true,
        tempo_bpm: 120.0,
        render_position_beats: index as f64 * 0.02,
        completed_position_beats: (index + 1) as f64 * 0.02,
    }
}

fn request() -> LiveMasterCaptureRequest {
    LiveMasterCaptureRequest {
        target_frame_count: 4096,
        channel_count: 1,
        expected_tempo_bpm: 120.0,
        start_position_beats: Some(4.0),
    }
}

// Always release before joining the armer, even when a callback/assertion fails.
struct ReleaseOnDrop(Arc<AllocationPause>);
impl Drop for ReleaseOnDrop {
    fn drop(&mut self) {
        self.0.release();
    }
}

#[test]
fn slow_capture_preparation_with_continuing_callbacks_does_not_fabricate_a_gap() {
    let shell = shell();
    let telemetry = Arc::clone(&shell.telemetry);
    let mut reader = shell.live_master_capture.take_callback_reader().unwrap();
    reader.record_callback(&[0.1; 480], &timing(0), 1_000);
    telemetry.record_callback_at(1_000, &timing(0));
    let pause = Arc::new(AllocationPause::default());
    let armer_pause = Arc::clone(&pause);
    thread::scope(|scope| {
        let armer = scope.spawn(move || {
            let result = with_allocation_paused(4096 * size_of::<f32>(), &armer_pause, || {
                shell.begin_live_master_capture(request())
            });
            (shell, result)
        });
        // Defined after spawn so unwinding releases before scope joins.
        let release = ReleaseOnDrop(Arc::clone(&pause));
        let deadline = Instant::now() + Duration::from_secs(3);
        while !pause.entered() {
            assert!(
                Instant::now() < deadline,
                "payload allocation barrier not reached"
            );
            thread::yield_now();
        }
        for index in 1..=20 {
            let now = 1_000 + index * 10_000;
            reader.record_callback(&[0.1; 480], &timing(index), now);
            telemetry.record_callback_at(now, &timing(index));
        }
        drop(release);
        let (shell, result) = armer.join().unwrap();
        result.unwrap();
        assert!(!pause.timed_out(), "allocation barrier timed out");
        reader.record_callback(&[0.1; 480], &timing(21), 211_000);
        let progress = shell.live_master_capture_progress().unwrap();
        assert_eq!(progress.timing_window_mismatch_count, 0);
        assert_eq!(progress.written_sample_count, 0);
        assert_eq!(progress.armed_callback_count, 1);
        assert_eq!(progress.max_callback_gap_micros, Some(10_000));
        assert_eq!(progress.fault_count(), 0);
        shell.abort_live_master_capture().unwrap();
    });
}

#[test]
fn genuine_initial_armed_and_captured_callback_gaps_remain_faults() {
    let shell = shell();
    let mut reader = shell.live_master_capture.take_callback_reader().unwrap();
    reader.record_callback(&[0.1; 480], &timing(0), 1_000);
    shell.telemetry.record_callback_at(1_000, &timing(0));
    shell.begin_live_master_capture(request()).unwrap();
    let gap = LIVE_MASTER_CALLBACK_GAP_THRESHOLD_MICROS + 1;
    reader.record_callback(&[0.1; 480], &timing(1), 1_000 + gap);
    reader.record_callback(&[0.1; 480], &timing(2), 1_000 + gap * 2);
    reader.record_callback(&[0.1; 480], &timing(200), 1_000 + gap * 3);
    reader.record_callback(&[0.1; 480], &timing(201), 1_000 + gap * 4);
    let progress = shell.live_master_capture_progress().unwrap();
    assert_eq!(progress.callback_gap_over_threshold_count, 4);
    assert_eq!(progress.max_callback_gap_micros, Some(gap));
    assert_eq!(progress.timing_window_mismatch_count, 0);
    assert_eq!(progress.written_sample_count, 960);
    assert_eq!(progress.callback_count, 2);
    shell.abort_live_master_capture().unwrap();
}

#[test]
fn scratch_callbacks_refresh_idle_history_and_keep_active_fault_accounting() {
    let shell = shell();
    let mut reader = shell.live_master_capture.take_callback_reader().unwrap();
    reader.record_callback(&[0.1; 480], &timing(0), 1_000);
    reader.record_scratch_overflow(&timing(20), 201_000);
    shell.begin_live_master_capture(request()).unwrap();
    reader.record_scratch_overflow(&timing(21), 211_000);
    reader.record_callback(&[0.1; 480], &timing(22), 221_000);
    let progress = shell.live_master_capture_progress().unwrap();
    assert_eq!(progress.max_callback_gap_micros, Some(10_000));
    assert_eq!(progress.callback_gap_over_threshold_count, 0);
    assert_eq!(progress.callback_scratch_overflow_count, 1);
    assert_eq!(progress.fault_count(), 1);
    shell.abort_live_master_capture().unwrap();
}

#[test]
fn completed_and_closed_slots_do_not_stale_the_next_captures_history() {
    for finish in [false, true] {
        let shell = shell();
        let mut reader = shell.live_master_capture.take_callback_reader().unwrap();
        shell
            .begin_live_master_capture(LiveMasterCaptureRequest {
                target_frame_count: 1,
                start_position_beats: Some(0.0),
                ..request()
            })
            .unwrap();
        reader.record_callback(&[0.1; 480], &timing(0), 1_000);
        let old = Arc::clone(reader.output.read().as_ref().unwrap());
        let completed = old.progress();
        // Completed but still-published capture: history must advance without
        // changing the completed buffer or inheriting this unrelated gap.
        reader.record_callback(&[0.1; 480], &timing(20), 201_000);
        assert_eq!(old.progress(), completed);
        if finish {
            shell.finish_live_master_capture().unwrap();
        } else {
            shell.abort_live_master_capture().unwrap();
        }
        reader.record_scratch_overflow(&timing(40), 401_000);
        assert_eq!(old.progress(), completed);
        shell.begin_live_master_capture(request()).unwrap();
        reader.record_callback(&[0.1; 480], &timing(41), 411_000);
        let next = shell.live_master_capture_progress().unwrap();
        assert_eq!(next.max_callback_gap_micros, Some(10_000));
        assert_eq!(next.fault_count(), 0);
        assert_eq!(old.progress(), completed);
        // Even a direct stale borrow cannot seed timing after admission closes.
        old.record_callback(&[0.1; 480], &timing(42), 421_000, Some(0));
        old.record_scratch_overflow(&timing(42), 421_000, Some(0));
        assert_eq!(old.progress(), completed);
        shell.abort_live_master_capture().unwrap();
    }
}

#[test]
fn first_callback_without_prior_history_does_not_invent_a_gap() {
    let shell = shell();
    let mut reader = shell.live_master_capture.take_callback_reader().unwrap();
    shell.begin_live_master_capture(request()).unwrap();
    reader.record_callback(&[0.1; 480], &timing(0), 210_000);
    let progress = shell.live_master_capture_progress().unwrap();
    assert_eq!(progress.max_callback_gap_micros, None);
    assert_eq!(progress.fault_count(), 0);
    shell.abort_live_master_capture().unwrap();
}

#[test]
fn zero_timestamp_and_exact_gap_threshold_remain_valid() {
    let shell = shell();
    let mut reader = shell.live_master_capture.take_callback_reader().unwrap();
    reader.record_callback(&[0.1; 480], &timing(0), 0);
    shell.begin_live_master_capture(request()).unwrap();
    reader.record_callback(
        &[0.1; 480],
        &timing(1),
        LIVE_MASTER_CALLBACK_GAP_THRESHOLD_MICROS,
    );
    let progress = shell.live_master_capture_progress().unwrap();
    assert_eq!(
        progress.max_callback_gap_micros,
        Some(LIVE_MASTER_CALLBACK_GAP_THRESHOLD_MICROS)
    );
    assert_eq!(progress.fault_count(), 0);
    reader.record_callback(
        &[0.1; 480],
        &timing(2),
        LIVE_MASTER_CALLBACK_GAP_THRESHOLD_MICROS * 2 + 1,
    );
    assert_eq!(
        shell
            .live_master_capture_progress()
            .unwrap()
            .callback_gap_over_threshold_count,
        1
    );
    shell.abort_live_master_capture().unwrap();
}

#[test]
fn v1_first_copied_callback_does_not_inherit_idle_or_skipped_prefix_gaps() {
    let shell = shell();
    let mut reader = shell.live_master_capture.take_callback_reader().unwrap();
    reader.record_callback(&[0.1; 480], &timing(0), 1_000);
    shell
        .begin_live_master_capture(LiveMasterCaptureRequest {
            target_frame_count: 480,
            start_position_beats: None,
            ..request()
        })
        .unwrap();
    let stopped = CallbackTimingSnapshot {
        is_transport_running: false,
        completed_position_beats: 0.0,
        ..timing(0)
    };
    reader.record_callback(&[0.1; 480], &stopped, 201_000);
    reader.record_callback(&[0.2; 480], &timing(0), 401_000);
    let outcome = shell.finish_live_master_capture().unwrap();
    assert_eq!(outcome.samples, [0.2; 480]);
    assert_eq!(outcome.progress.max_callback_gap_micros, None);
    assert_eq!(outcome.progress.callback_count, 1);
    assert_eq!(outcome.progress.fault_count(), 0);
}
