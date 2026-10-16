use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};

use super::timing_snapshot::CallbackTimingPublication;
use super::{AudioRuntimeTimingSnapshot, CallbackTimingSnapshot};

#[derive(Default)]
pub(super) struct RuntimeTelemetrySnapshot {
    pub(super) callback_count: u64,
    pub(super) max_callback_gap_micros: Option<u64>,
    pub(super) callback_scratch_overflow_count: u64,
    pub(super) stream_error_count: u64,
    pub(super) last_stream_error: Option<String>,
    pub(super) stream_error_buffer_poisoned: bool,
}

pub(super) struct RuntimeTelemetry {
    callback_count: AtomicU64,
    max_callback_gap_micros: AtomicU64,
    last_callback_micros: AtomicU64,
    callback_scratch_overflow_count: AtomicU64,
    stream_error_count: AtomicU64,
    last_stream_error: Mutex<Option<String>>,
    timing: CallbackTimingPublication,
}

impl RuntimeTelemetry {
    pub(super) fn new() -> Self {
        Self {
            callback_count: AtomicU64::new(0),
            max_callback_gap_micros: AtomicU64::new(0),
            last_callback_micros: AtomicU64::new(0),
            callback_scratch_overflow_count: AtomicU64::new(0),
            stream_error_count: AtomicU64::new(0),
            last_stream_error: Mutex::new(None),
            timing: CallbackTimingPublication::new(),
        }
    }

    #[cfg(test)]
    pub(super) fn record_callback_at(&self, now_micros: u64, timing: &CallbackTimingSnapshot) {
        self.record_callback_with_position_request_at(now_micros, timing, 0);
    }

    pub(super) fn record_callback_with_position_request_at(
        &self,
        now_micros: u64,
        timing: &CallbackTimingSnapshot,
        applied_request_id: u64,
    ) {
        let previous = self
            .last_callback_micros
            .swap(now_micros, Ordering::Relaxed);
        if previous != 0 {
            let gap = now_micros.saturating_sub(previous);
            self.max_callback_gap_micros
                .fetch_max(gap, Ordering::Relaxed);
        }
        self.callback_count.fetch_add(1, Ordering::Relaxed);
        self.timing.publish(
            AudioRuntimeTimingSnapshot {
                is_transport_running: timing.is_transport_running,
                tempo_bpm: timing.tempo_bpm,
                position_beats: timing.completed_position_beats,
            },
            applied_request_id,
        );
    }

    pub(super) fn record_callback_scratch_overflow_at(
        &self,
        now_micros: u64,
        timing: &CallbackTimingSnapshot,
        applied_request_id: u64,
    ) {
        self.callback_scratch_overflow_count
            .fetch_add(1, Ordering::Relaxed);
        self.record_callback_with_position_request_at(now_micros, timing, applied_request_id);
    }

    pub(super) fn record_stream_error(&self, message: String) {
        self.stream_error_count.fetch_add(1, Ordering::Relaxed);
        // The device-error callback is not the render callback. Recover this
        // simple diagnostic slot, but leave poison latched for health reporting.
        *self
            .last_stream_error
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(message);
    }

    pub(super) fn snapshot(&self) -> RuntimeTelemetrySnapshot {
        let callback_count = self.callback_count.load(Ordering::Relaxed);
        let max_gap_micros = self.max_callback_gap_micros.load(Ordering::Relaxed);
        let (last_stream_error, stream_error_buffer_poisoned) = match self.last_stream_error.lock()
        {
            Ok(message) => (message.clone(), false),
            Err(poison) => (poison.into_inner().clone(), true),
        };

        RuntimeTelemetrySnapshot {
            callback_count,
            max_callback_gap_micros: (callback_count > 1).then_some(max_gap_micros),
            callback_scratch_overflow_count: self
                .callback_scratch_overflow_count
                .load(Ordering::Relaxed),
            stream_error_count: self.stream_error_count.load(Ordering::Relaxed),
            last_stream_error,
            stream_error_buffer_poisoned,
        }
    }

    #[cfg(test)]
    pub(super) fn timing_snapshot(&self) -> Option<AudioRuntimeTimingSnapshot> {
        // Timing observers need only atomic progress, never the diagnostic
        // message mutex or its String allocation.
        self.timing
            .snapshot_with_position_request()
            .map(|(timing, _)| timing)
    }

    pub(super) fn timing_snapshot_for_position_request(
        &self,
        requested_id: u64,
    ) -> Option<AudioRuntimeTimingSnapshot> {
        let (timing, applied_id) = self.timing.snapshot_with_position_request()?;
        (applied_id == requested_id).then_some(timing)
    }
}

#[cfg(test)]
mod telemetry_poison_tests {
    use super::RuntimeTelemetry;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    fn poison(telemetry: &RuntimeTelemetry, prior_message: Option<&str>) {
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                let mut last = telemetry.last_stream_error.lock().expect("initial lock");
                *last = prior_message.map(str::to_owned);
                panic!("injected telemetry writer panic");
            }))
            .is_err()
        );
    }

    #[test]
    fn poisoned_error_buffer_accepts_later_stream_errors() {
        let telemetry = RuntimeTelemetry::new();
        poison(&telemetry, Some("previous error"));
        telemetry.record_stream_error("device disconnected".into());
        let snapshot = telemetry.snapshot();
        assert_eq!(snapshot.stream_error_count, 1);
        assert!(snapshot.stream_error_buffer_poisoned);
        assert_eq!(
            snapshot.last_stream_error.as_deref(),
            Some("device disconnected")
        );
    }

    #[test]
    fn poisoned_error_buffer_remains_readable() {
        let telemetry = RuntimeTelemetry::new();
        poison(&telemetry, Some("previous error"));
        let snapshot = telemetry.snapshot();
        assert_eq!(
            snapshot.stream_error_count, 0,
            "poison is not a fabricated device error"
        );
        assert!(snapshot.stream_error_buffer_poisoned);
        assert_eq!(
            snapshot.last_stream_error.as_deref(),
            Some("previous error")
        );
    }

    #[test]
    fn timing_progress_does_not_lock_the_error_buffer() {
        use std::{
            sync::{Arc, mpsc},
            thread,
            time::Duration,
        };

        let telemetry = Arc::new(RuntimeTelemetry::new());
        let guard = telemetry.last_stream_error.lock().expect("initial lock");
        let reader_telemetry = Arc::clone(&telemetry);
        let (send, receive) = mpsc::channel();
        let reader = thread::spawn(move || {
            reader_telemetry.record_callback_at(
                100,
                &super::CallbackTimingSnapshot {
                    is_transport_running: true,
                    tempo_bpm: 132.0,
                    render_position_beats: 3.5,
                    completed_position_beats: 4.0,
                },
            );
            send.send(reader_telemetry.timing_snapshot())
                .expect("send timing");
        });
        let result = receive.recv_timeout(Duration::from_secs(2));
        // Release before asserting: a lock regression must fail, not hang CI.
        drop(guard);
        reader.join().expect("timing thread");
        let timing = result
            .expect("timing should not wait on the diagnostic message lock")
            .expect("completed callback timing");
        assert!(timing.is_transport_running);
        assert_eq!(timing.tempo_bpm, 132.0);
        assert_eq!(timing.position_beats, 4.0);
    }

    #[test]
    fn public_health_keeps_poison_visible_without_fabricating_stream_errors() {
        use crate::runtime::shared_mc202::SharedMc202RenderState;
        use crate::runtime::shared_transport_tr909::{
            AudioRuntimeShellTestParts, SharedTr909RenderState, SharedTransportTimingState,
        };
        use crate::runtime::shared_w30_resample_callback::SharedW30ResampleTapState;
        use crate::runtime::source_monitor::SharedSourceMonitorRenderState;
        use crate::runtime::w30_preview_snapshot::SharedW30PreviewRenderState;
        use crate::runtime::{AudioRuntimeLifecycle, AudioRuntimeShell};
        use std::sync::Arc;

        for (lifecycle, prior_message) in [
            (AudioRuntimeLifecycle::Running, Some("previous error")),
            (AudioRuntimeLifecycle::Stopped, Some("previous error")),
            (AudioRuntimeLifecycle::Running, None),
        ] {
            let telemetry = Arc::new(RuntimeTelemetry::new());
            assert!(!telemetry.snapshot().stream_error_buffer_poisoned);
            poison(&telemetry, prior_message);
            let shell = AudioRuntimeShell::from_test_parts(AudioRuntimeShellTestParts {
                lifecycle,
                output: None,
                telemetry: Arc::clone(&telemetry),
                transport: Arc::new(SharedTransportTimingState::new(false, 132.0, 0.0)),
                tr909_render: Arc::new(SharedTr909RenderState::new(&Default::default())),
                mc202_render: Arc::new(SharedMc202RenderState::new(&Default::default())),
                w30_preview: Arc::new(SharedW30PreviewRenderState::new(&Default::default())),
                w30_resample_tap: Arc::new(SharedW30ResampleTapState::new(&Default::default())),
                source_monitor: Arc::new(SharedSourceMonitorRenderState::new(&Default::default())),
            });
            let health = shell.health_snapshot();
            assert_eq!(health.stream_error_count, 0);
            assert_eq!(
                health.lifecycle,
                if lifecycle == AudioRuntimeLifecycle::Running {
                    AudioRuntimeLifecycle::Faulted
                } else {
                    lifecycle
                }
            );
            assert_eq!(
                health.last_stream_error.as_deref(),
                Some(if prior_message.is_some() {
                    "stream-error telemetry degraded (poisoned buffer); last stream error: previous error"
                } else {
                    "stream-error telemetry degraded (poisoned buffer); no stream error message available"
                })
            );
            telemetry.record_stream_error("new device error".into());
            let health = shell.health_snapshot();
            assert_eq!(health.stream_error_count, 1);
            assert_eq!(
                health.last_stream_error.as_deref(),
                Some(
                    "stream-error telemetry degraded (poisoned buffer); last stream error: new device error"
                )
            );
        }
    }
}
