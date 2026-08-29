use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering},
};

use arc_swap::ArcSwapOption;

use super::CallbackTimingSnapshot;

pub const LIVE_MASTER_CALLBACK_GAP_THRESHOLD_MICROS: u64 = 100_000;
/// Maximum V1 capture payload admitted before allocating the atomic callback buffer.
///
/// Each slot is one four-byte `f32` bit pattern, so this bounds the callback-owned
/// allocation to 64 MiB. Finalization may create one equally sized ordinary `f32`
/// copy on the control thread.
pub const LIVE_MASTER_MAX_INTERLEAVED_SAMPLE_COUNT: usize = 16_777_216;

#[derive(Clone, Debug, PartialEq)]
pub struct LiveMasterCaptureRequest {
    pub target_frame_count: usize,
    pub channel_count: u16,
    pub expected_tempo_bpm: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveMasterCaptureProgress {
    pub target_sample_count: usize,
    pub written_sample_count: usize,
    pub callback_count: u64,
    pub max_callback_gap_micros: Option<u64>,
    pub callback_gap_over_threshold_count: u64,
    pub callback_scratch_overflow_count: u64,
    pub stream_error_count: u64,
    pub transport_mismatch_count: u64,
    pub tempo_mismatch_count: u64,
    pub complete: bool,
}

impl LiveMasterCaptureProgress {
    #[must_use]
    pub fn fault_count(&self) -> u64 {
        self.callback_gap_over_threshold_count
            .saturating_add(self.callback_scratch_overflow_count)
            .saturating_add(self.stream_error_count)
            .saturating_add(self.transport_mismatch_count)
            .saturating_add(self.tempo_mismatch_count)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LiveMasterCaptureOutcome {
    pub samples: Vec<f32>,
    pub progress: LiveMasterCaptureProgress,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiveMasterCaptureError {
    InvalidTarget,
    AllocationFailed,
    AlreadyActive,
    NotActive,
    NotComplete(LiveMasterCaptureProgress),
    CallbackStillActive(LiveMasterCaptureProgress),
}

pub(super) struct SharedLiveMasterCapture {
    active: ArcSwapOption<LiveMasterCaptureBuffer>,
    control: Mutex<LiveMasterCaptureControl>,
}

#[derive(Default)]
struct LiveMasterCaptureControl {
    /// Buffers removed from `active` while a callback still holds an `Arc`.
    /// Reaping stays on the control thread so the realtime callback can never
    /// become the owner that deallocates a large capture buffer.
    retired: Vec<Arc<LiveMasterCaptureBuffer>>,
}

impl SharedLiveMasterCapture {
    pub(super) fn new() -> Self {
        Self {
            active: ArcSwapOption::empty(),
            control: Mutex::new(LiveMasterCaptureControl::default()),
        }
    }

    pub(super) fn begin(
        &self,
        request: LiveMasterCaptureRequest,
    ) -> Result<(), LiveMasterCaptureError> {
        let mut control = self
            .control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::reap_retired(&mut control);
        let target_sample_count = request
            .target_frame_count
            .checked_mul(usize::from(request.channel_count))
            .filter(|count| *count > 0 && *count <= LIVE_MASTER_MAX_INTERLEAVED_SAMPLE_COUNT)
            .ok_or(LiveMasterCaptureError::InvalidTarget)?;
        if request.channel_count == 0
            || !request.expected_tempo_bpm.is_finite()
            || request.expected_tempo_bpm <= 0.0
        {
            return Err(LiveMasterCaptureError::InvalidTarget);
        }
        if self.active.load().is_some() {
            return Err(LiveMasterCaptureError::AlreadyActive);
        }
        let capture =
            LiveMasterCaptureBuffer::try_new(target_sample_count, request.expected_tempo_bpm)?;
        self.active.store(Some(Arc::new(capture)));
        Ok(())
    }

    pub(super) fn progress(&self) -> Option<LiveMasterCaptureProgress> {
        self.active.load_full().map(|capture| capture.progress())
    }

    pub(super) fn finish(&self) -> Result<LiveMasterCaptureOutcome, LiveMasterCaptureError> {
        let mut control = self
            .control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::reap_retired(&mut control);
        let capture = self
            .active
            .load_full()
            .ok_or(LiveMasterCaptureError::NotActive)?;
        let progress = capture.progress();
        if !progress.complete {
            return Err(LiveMasterCaptureError::NotComplete(progress));
        }

        let Some(removed) = self.active.swap(None) else {
            return Err(LiveMasterCaptureError::NotActive);
        };
        debug_assert!(Arc::ptr_eq(&capture, &removed));
        if Arc::strong_count(&removed) > 2 {
            let progress = removed.progress();
            drop(capture);
            control.retired.push(removed);
            return Err(LiveMasterCaptureError::CallbackStillActive(progress));
        }
        let outcome = removed.outcome();
        drop(capture);
        Ok(outcome)
    }

    pub(super) fn abort(&self) -> Option<LiveMasterCaptureProgress> {
        let mut control = self
            .control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::reap_retired(&mut control);
        let capture = self.active.swap(None)?;
        let progress = capture.progress();
        if Arc::strong_count(&capture) > 1 {
            control.retired.push(capture);
        }
        Some(progress)
    }

    pub(super) fn record_callback(
        &self,
        samples: &[f32],
        timing: &CallbackTimingSnapshot,
        now_micros: u64,
    ) {
        if let Some(capture) = self.active.load_full() {
            capture.record_callback(samples, timing, now_micros);
        }
    }

    pub(super) fn record_scratch_overflow(&self, timing: &CallbackTimingSnapshot, now_micros: u64) {
        if let Some(capture) = self.active.load_full() {
            capture.record_callback_timing(timing, now_micros);
            capture
                .callback_scratch_overflow_count
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    pub(super) fn record_stream_error(&self) {
        if let Some(capture) = self.active.load_full() {
            capture.stream_error_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn reap_retired(control: &mut LiveMasterCaptureControl) {
        control
            .retired
            .retain(|capture| Arc::strong_count(capture) > 1);
    }
}

struct LiveMasterCaptureBuffer {
    samples: Box<[AtomicU32]>,
    written_sample_count: AtomicUsize,
    expected_tempo_bpm_bits: u32,
    callback_count: AtomicU64,
    last_callback_micros: AtomicU64,
    max_callback_gap_micros: AtomicU64,
    callback_gap_over_threshold_count: AtomicU64,
    callback_scratch_overflow_count: AtomicU64,
    stream_error_count: AtomicU64,
    transport_mismatch_count: AtomicU64,
    tempo_mismatch_count: AtomicU64,
    complete: AtomicBool,
}

impl LiveMasterCaptureBuffer {
    fn try_new(
        target_sample_count: usize,
        expected_tempo_bpm: f32,
    ) -> Result<Self, LiveMasterCaptureError> {
        let mut samples = Vec::new();
        samples
            .try_reserve_exact(target_sample_count)
            .map_err(|_| LiveMasterCaptureError::AllocationFailed)?;
        samples.extend((0..target_sample_count).map(|_| AtomicU32::new(0.0_f32.to_bits())));
        Ok(Self {
            samples: samples.into_boxed_slice(),
            written_sample_count: AtomicUsize::new(0),
            expected_tempo_bpm_bits: expected_tempo_bpm.to_bits(),
            callback_count: AtomicU64::new(0),
            last_callback_micros: AtomicU64::new(u64::MAX),
            max_callback_gap_micros: AtomicU64::new(0),
            callback_gap_over_threshold_count: AtomicU64::new(0),
            callback_scratch_overflow_count: AtomicU64::new(0),
            stream_error_count: AtomicU64::new(0),
            transport_mismatch_count: AtomicU64::new(0),
            tempo_mismatch_count: AtomicU64::new(0),
            complete: AtomicBool::new(false),
        })
    }

    fn record_callback(&self, samples: &[f32], timing: &CallbackTimingSnapshot, now_micros: u64) {
        if self.complete.load(Ordering::Acquire) {
            return;
        }
        self.record_callback_timing(timing, now_micros);

        let start = self.written_sample_count.load(Ordering::Relaxed);
        let remaining = self.samples.len().saturating_sub(start);
        let copy_count = remaining.min(samples.len());
        for (slot, sample) in self.samples[start..start + copy_count]
            .iter()
            .zip(&samples[..copy_count])
        {
            slot.store(sample.to_bits(), Ordering::Relaxed);
        }
        let written = start.saturating_add(copy_count);
        self.written_sample_count.store(written, Ordering::Release);
        if written == self.samples.len() {
            self.complete.store(true, Ordering::Release);
        }
    }

    fn record_callback_timing(&self, timing: &CallbackTimingSnapshot, now_micros: u64) {
        let previous = self
            .last_callback_micros
            .swap(now_micros, Ordering::Relaxed);
        if previous != u64::MAX {
            let gap = now_micros.saturating_sub(previous);
            self.max_callback_gap_micros
                .fetch_max(gap, Ordering::Relaxed);
            if gap > LIVE_MASTER_CALLBACK_GAP_THRESHOLD_MICROS {
                self.callback_gap_over_threshold_count
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
        self.callback_count.fetch_add(1, Ordering::Relaxed);
        if !timing.is_transport_running {
            self.transport_mismatch_count
                .fetch_add(1, Ordering::Relaxed);
        }
        if timing.tempo_bpm.to_bits() != self.expected_tempo_bpm_bits {
            self.tempo_mismatch_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn progress(&self) -> LiveMasterCaptureProgress {
        // Acquire completion before reading the payload and fault counters so a
        // completed snapshot observes everything published by the final callback.
        let complete = self.complete.load(Ordering::Acquire);
        let max_gap = self.max_callback_gap_micros.load(Ordering::Relaxed);
        LiveMasterCaptureProgress {
            target_sample_count: self.samples.len(),
            written_sample_count: self.written_sample_count.load(Ordering::Acquire),
            callback_count: self.callback_count.load(Ordering::Relaxed),
            max_callback_gap_micros: (max_gap > 0).then_some(max_gap),
            callback_gap_over_threshold_count: self
                .callback_gap_over_threshold_count
                .load(Ordering::Relaxed),
            callback_scratch_overflow_count: self
                .callback_scratch_overflow_count
                .load(Ordering::Relaxed),
            stream_error_count: self.stream_error_count.load(Ordering::Relaxed),
            transport_mismatch_count: self.transport_mismatch_count.load(Ordering::Relaxed),
            tempo_mismatch_count: self.tempo_mismatch_count.load(Ordering::Relaxed),
            complete,
        }
    }

    fn outcome(&self) -> LiveMasterCaptureOutcome {
        let progress = self.progress();
        let samples = self.samples[..progress.written_sample_count]
            .iter()
            .map(|sample| f32::from_bits(sample.load(Ordering::Relaxed)))
            .collect();
        LiveMasterCaptureOutcome { samples, progress }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timing(running: bool, tempo_bpm: f32) -> CallbackTimingSnapshot {
        CallbackTimingSnapshot {
            is_transport_running: running,
            tempo_bpm,
            render_position_beats: 0.0,
            completed_position_beats: 0.25,
        }
    }

    #[test]
    fn bounded_capture_keeps_exact_post_limiter_samples_across_callbacks() {
        let shared = SharedLiveMasterCapture::new();
        shared
            .begin(LiveMasterCaptureRequest {
                target_frame_count: 3,
                channel_count: 2,
                expected_tempo_bpm: 128.0,
            })
            .expect("begin capture");

        shared.record_callback(&[0.1, -0.1, 0.2, -0.2], &timing(true, 128.0), 1_000);
        shared.record_callback(&[0.3, -0.3, 0.9, 0.8], &timing(true, 128.0), 2_000);

        let outcome = shared.finish().expect("complete capture");
        assert_eq!(outcome.samples, vec![0.1, -0.1, 0.2, -0.2, 0.3, -0.3]);
        assert_eq!(outcome.progress.written_sample_count, 6);
        assert_eq!(outcome.progress.callback_count, 2);
        assert_eq!(outcome.progress.fault_count(), 0);
    }

    #[test]
    fn capture_records_realtime_faults_without_blocking_sample_progress() {
        let shared = SharedLiveMasterCapture::new();
        shared
            .begin(LiveMasterCaptureRequest {
                target_frame_count: 2,
                channel_count: 1,
                expected_tempo_bpm: 128.0,
            })
            .expect("begin capture");

        shared.record_callback(&[0.1], &timing(false, 127.0), 1_000);
        shared.record_stream_error();
        shared.record_callback(
            &[0.2],
            &timing(true, 128.0),
            1_000 + LIVE_MASTER_CALLBACK_GAP_THRESHOLD_MICROS + 1,
        );

        let outcome = shared.finish().expect("complete capture");
        assert_eq!(outcome.progress.transport_mismatch_count, 1);
        assert_eq!(outcome.progress.tempo_mismatch_count, 1);
        assert_eq!(outcome.progress.stream_error_count, 1);
        assert_eq!(outcome.progress.callback_gap_over_threshold_count, 1);
        assert_eq!(outcome.progress.fault_count(), 4);
    }

    #[test]
    fn incomplete_capture_cannot_be_finalized_and_can_be_aborted() {
        let shared = SharedLiveMasterCapture::new();
        shared
            .begin(LiveMasterCaptureRequest {
                target_frame_count: 2,
                channel_count: 2,
                expected_tempo_bpm: 128.0,
            })
            .expect("begin capture");
        shared.record_callback(&[0.1, -0.1], &timing(true, 128.0), 1_000);

        let error = shared.finish().expect_err("incomplete capture rejected");
        assert!(matches!(error, LiveMasterCaptureError::NotComplete(_)));
        let aborted = shared.abort().expect("active capture aborted");
        assert_eq!(aborted.written_sample_count, 2);
        assert!(!aborted.complete);
        assert!(shared.progress().is_none());
    }

    #[test]
    fn capture_rejects_payloads_above_the_bounded_allocation_before_allocating() {
        let shared = SharedLiveMasterCapture::new();

        let error = shared
            .begin(LiveMasterCaptureRequest {
                target_frame_count: LIVE_MASTER_MAX_INTERLEAVED_SAMPLE_COUNT + 1,
                channel_count: 1,
                expected_tempo_bpm: 128.0,
            })
            .expect_err("oversized capture rejected");

        assert_eq!(error, LiveMasterCaptureError::InvalidTarget);
        assert!(shared.progress().is_none());
    }

    #[test]
    fn finish_and_abort_retire_callback_owned_buffers_without_waiting() {
        let shared = SharedLiveMasterCapture::new();
        shared
            .begin(LiveMasterCaptureRequest {
                target_frame_count: 1,
                channel_count: 1,
                expected_tempo_bpm: 128.0,
            })
            .expect("begin completed capture");
        let held_completed = shared.active.load_full().expect("held callback Arc");
        shared.record_callback(&[0.25], &timing(true, 128.0), 1_000);

        let error = shared
            .finish()
            .expect_err("finish rejects a callback-owned buffer without waiting");
        assert!(matches!(
            error,
            LiveMasterCaptureError::CallbackStillActive(_)
        ));
        assert_eq!(shared.control.lock().unwrap().retired.len(), 1);

        drop(held_completed);
        shared
            .begin(LiveMasterCaptureRequest {
                target_frame_count: 2,
                channel_count: 1,
                expected_tempo_bpm: 128.0,
            })
            .expect("next begin reaps retired completed capture");
        assert!(shared.control.lock().unwrap().retired.is_empty());
        let held_incomplete = shared.active.load_full().expect("held callback Arc");

        let progress = shared.abort().expect("abort does not wait for held Arc");
        assert!(!progress.complete);
        assert_eq!(shared.control.lock().unwrap().retired.len(), 1);
        drop(held_incomplete);
    }
}
