use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering},
};

use arc_swap::ArcSwapOption;

use super::CallbackTimingSnapshot;

pub const LIVE_MASTER_CALLBACK_GAP_THRESHOLD_MICROS: u64 = 100_000;
/// Maximum capture payload admitted before allocating the atomic callback buffer.
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
    /// `None` preserves the V1 immediate-capture contract. V2 arms the
    /// preallocated buffer until this absolute Session transport position.
    pub start_position_beats: Option<f64>,
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
    pub timing_window_mismatch_count: u64,
    pub armed_callback_count: u64,
    pub capture_started: bool,
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
            .saturating_add(self.timing_window_mismatch_count)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LiveMasterCaptureOutcome {
    pub samples: Vec<f32>,
    pub progress: LiveMasterCaptureProgress,
    pub captured_start_position_beats: Option<f64>,
    pub captured_end_position_beats: Option<f64>,
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

    #[cfg(test)]
    fn begin(&self, request: LiveMasterCaptureRequest) -> Result<(), LiveMasterCaptureError> {
        self.begin_after_callback(request, None)
    }

    pub(super) fn begin_after_callback(
        &self,
        request: LiveMasterCaptureRequest,
        last_callback_micros: Option<u64>,
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
            || request
                .start_position_beats
                .is_some_and(|position| !position.is_finite() || position < 0.0)
        {
            return Err(LiveMasterCaptureError::InvalidTarget);
        }
        if self.active.load().is_some() {
            return Err(LiveMasterCaptureError::AlreadyActive);
        }
        let capture = LiveMasterCaptureBuffer::try_new(
            target_sample_count,
            &request,
            last_callback_micros.filter(|_| request.start_position_beats.is_some()),
        )?;
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
            capture.record_captured_callback_timing(timing, now_micros);
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
    channel_count: usize,
    expected_tempo_bpm_bits: u32,
    requested_start_position_beats_bits: u64,
    captured_start_position_beats_bits: AtomicU64,
    captured_end_position_beats_bits: AtomicU64,
    callback_count: AtomicU64,
    armed_callback_count: AtomicU64,
    last_callback_micros: AtomicU64,
    max_callback_gap_micros: AtomicU64,
    callback_gap_over_threshold_count: AtomicU64,
    callback_scratch_overflow_count: AtomicU64,
    stream_error_count: AtomicU64,
    transport_mismatch_count: AtomicU64,
    tempo_mismatch_count: AtomicU64,
    timing_window_mismatch_count: AtomicU64,
    capture_started: AtomicBool,
    complete: AtomicBool,
}

const UNSET_POSITION_BITS: u64 = u64::MAX;

impl LiveMasterCaptureBuffer {
    fn try_new(
        target_sample_count: usize,
        request: &LiveMasterCaptureRequest,
        last_callback_micros: Option<u64>,
    ) -> Result<Self, LiveMasterCaptureError> {
        let mut samples = Vec::new();
        samples
            .try_reserve_exact(target_sample_count)
            .map_err(|_| LiveMasterCaptureError::AllocationFailed)?;
        samples.extend((0..target_sample_count).map(|_| AtomicU32::new(0.0_f32.to_bits())));
        Ok(Self {
            samples: samples.into_boxed_slice(),
            written_sample_count: AtomicUsize::new(0),
            channel_count: usize::from(request.channel_count),
            expected_tempo_bpm_bits: request.expected_tempo_bpm.to_bits(),
            requested_start_position_beats_bits: request
                .start_position_beats
                .map_or(UNSET_POSITION_BITS, f64::to_bits),
            captured_start_position_beats_bits: AtomicU64::new(UNSET_POSITION_BITS),
            captured_end_position_beats_bits: AtomicU64::new(UNSET_POSITION_BITS),
            callback_count: AtomicU64::new(0),
            armed_callback_count: AtomicU64::new(0),
            last_callback_micros: AtomicU64::new(last_callback_micros.unwrap_or(u64::MAX)),
            max_callback_gap_micros: AtomicU64::new(0),
            callback_gap_over_threshold_count: AtomicU64::new(0),
            callback_scratch_overflow_count: AtomicU64::new(0),
            stream_error_count: AtomicU64::new(0),
            transport_mismatch_count: AtomicU64::new(0),
            tempo_mismatch_count: AtomicU64::new(0),
            timing_window_mismatch_count: AtomicU64::new(0),
            capture_started: AtomicBool::new(request.start_position_beats.is_none()),
            complete: AtomicBool::new(false),
        })
    }

    fn record_callback(&self, samples: &[f32], timing: &CallbackTimingSnapshot, now_micros: u64) {
        if self.complete.load(Ordering::Acquire) {
            return;
        }
        let is_bar_window = self.requested_start_position_beats_bits != UNSET_POSITION_BITS;
        if is_bar_window {
            self.record_callback_gap(now_micros);
        }
        if is_bar_window && self.record_transport_and_tempo_faults(timing) {
            if !self.capture_started.load(Ordering::Acquire) {
                self.armed_callback_count.fetch_add(1, Ordering::Relaxed);
            }
            return;
        }
        let Some(source_start) = self.capture_source_start(samples, timing) else {
            return;
        };
        if is_bar_window {
            self.callback_count.fetch_add(1, Ordering::Relaxed);
        } else {
            self.record_captured_callback_timing(timing, now_micros);
        }

        let start = self.written_sample_count.load(Ordering::Relaxed);
        let remaining = self.samples.len().saturating_sub(start);
        let available = samples.len().saturating_sub(source_start);
        let copy_count = remaining.min(available);
        for (slot, sample) in self.samples[start..start + copy_count]
            .iter()
            .zip(&samples[source_start..source_start + copy_count])
        {
            slot.store(sample.to_bits(), Ordering::Relaxed);
        }
        let written = start.saturating_add(copy_count);
        self.written_sample_count.store(written, Ordering::Release);
        self.record_captured_end_position(timing, samples.len(), source_start, copy_count);
        if written == self.samples.len() {
            self.complete.store(true, Ordering::Release);
        }
    }

    fn capture_source_start(
        &self,
        samples: &[f32],
        timing: &CallbackTimingSnapshot,
    ) -> Option<usize> {
        if self.channel_count == 0
            || samples.is_empty()
            || !samples.len().is_multiple_of(self.channel_count)
            || !timing.render_position_beats.is_finite()
            || !timing.completed_position_beats.is_finite()
            || timing.completed_position_beats < timing.render_position_beats
        {
            self.timing_window_mismatch_count
                .fetch_add(1, Ordering::Relaxed);
            return None;
        }
        let frame_count = samples.len() / self.channel_count;
        if frame_count == 0 {
            self.timing_window_mismatch_count
                .fetch_add(1, Ordering::Relaxed);
            return None;
        }
        let callback_beat_span = timing.completed_position_beats - timing.render_position_beats;
        if callback_beat_span <= 0.0 {
            if self.requested_start_position_beats_bits != UNSET_POSITION_BITS {
                self.timing_window_mismatch_count
                    .fetch_add(1, Ordering::Relaxed);
            }
            return None;
        }
        let beats_per_frame = callback_beat_span / frame_count as f64;
        let tolerance = beats_per_frame * 1.0e-6;
        if self.capture_started.load(Ordering::Acquire) {
            if let Some(expected_start) = position_from_bits(
                self.captured_end_position_beats_bits
                    .load(Ordering::Acquire),
            ) && (timing.render_position_beats - expected_start).abs() > tolerance
            {
                self.timing_window_mismatch_count
                    .fetch_add(1, Ordering::Relaxed);
                return None;
            }
            return Some(0);
        }
        self.armed_callback_count.fetch_add(1, Ordering::Relaxed);
        let requested = f64::from_bits(self.requested_start_position_beats_bits);
        if timing.completed_position_beats <= requested + tolerance {
            return None;
        }
        if timing.render_position_beats > requested + beats_per_frame + tolerance {
            self.timing_window_mismatch_count
                .fetch_add(1, Ordering::Relaxed);
            return None;
        }

        let relative_frame =
            ((requested - timing.render_position_beats) / beats_per_frame).max(0.0);
        let frame_offset = (relative_frame - 1.0e-9).ceil().max(0.0) as usize;
        if frame_offset >= frame_count {
            return None;
        }
        let actual_start = timing.render_position_beats + beats_per_frame * frame_offset as f64;
        if actual_start + tolerance < requested
            || actual_start - requested > beats_per_frame + tolerance
        {
            self.timing_window_mismatch_count
                .fetch_add(1, Ordering::Relaxed);
            return None;
        }
        self.captured_start_position_beats_bits
            .store(actual_start.to_bits(), Ordering::Release);
        self.capture_started.store(true, Ordering::Release);
        Some(frame_offset * self.channel_count)
    }

    fn record_captured_end_position(
        &self,
        timing: &CallbackTimingSnapshot,
        supplied_sample_count: usize,
        source_start: usize,
        copy_count: usize,
    ) {
        if self.requested_start_position_beats_bits == UNSET_POSITION_BITS
            || self.channel_count == 0
            || copy_count == 0
        {
            return;
        }
        let supplied_frame_count = supplied_sample_count / self.channel_count;
        let copied_end_frame = source_start.saturating_add(copy_count) / self.channel_count;
        let callback_beat_span = timing.completed_position_beats - timing.render_position_beats;
        if supplied_frame_count == 0 || !callback_beat_span.is_finite() || callback_beat_span <= 0.0
        {
            return;
        }
        let beats_per_frame = callback_beat_span / supplied_frame_count as f64;
        let end_position = timing.render_position_beats + beats_per_frame * copied_end_frame as f64;
        self.captured_end_position_beats_bits
            .store(end_position.to_bits(), Ordering::Release);
    }

    fn record_callback_gap(&self, now_micros: u64) {
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
    }

    fn record_captured_callback_timing(&self, timing: &CallbackTimingSnapshot, now_micros: u64) {
        self.record_callback_gap(now_micros);
        self.callback_count.fetch_add(1, Ordering::Relaxed);
        self.record_transport_and_tempo_faults(timing);
    }

    fn record_transport_and_tempo_faults(&self, timing: &CallbackTimingSnapshot) -> bool {
        let transport_mismatch = !timing.is_transport_running;
        let tempo_mismatch = timing.tempo_bpm.to_bits() != self.expected_tempo_bpm_bits;
        if transport_mismatch {
            self.transport_mismatch_count
                .fetch_add(1, Ordering::Relaxed);
        }
        if tempo_mismatch {
            self.tempo_mismatch_count.fetch_add(1, Ordering::Relaxed);
        }
        transport_mismatch || tempo_mismatch
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
            timing_window_mismatch_count: self.timing_window_mismatch_count.load(Ordering::Relaxed),
            armed_callback_count: self.armed_callback_count.load(Ordering::Relaxed),
            capture_started: self.capture_started.load(Ordering::Acquire),
            complete,
        }
    }

    fn outcome(&self) -> LiveMasterCaptureOutcome {
        let progress = self.progress();
        let samples = self.samples[..progress.written_sample_count]
            .iter()
            .map(|sample| f32::from_bits(sample.load(Ordering::Relaxed)))
            .collect();
        let captured_start_position_beats = position_from_bits(
            self.captured_start_position_beats_bits
                .load(Ordering::Acquire),
        );
        let captured_end_position_beats = position_from_bits(
            self.captured_end_position_beats_bits
                .load(Ordering::Acquire),
        );
        LiveMasterCaptureOutcome {
            samples,
            progress,
            captured_start_position_beats,
            captured_end_position_beats,
        }
    }
}

fn position_from_bits(bits: u64) -> Option<f64> {
    (bits != UNSET_POSITION_BITS).then(|| f64::from_bits(bits))
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

    fn timing_range(start: f64, end: f64) -> CallbackTimingSnapshot {
        CallbackTimingSnapshot {
            is_transport_running: true,
            tempo_bpm: 120.0,
            render_position_beats: start,
            completed_position_beats: end,
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
                start_position_beats: None,
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
                start_position_beats: None,
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
    fn v1_immediate_capture_preserves_its_legacy_fault_observation_boundary() {
        let shared = SharedLiveMasterCapture::new();
        shared
            .begin(LiveMasterCaptureRequest {
                target_frame_count: 1,
                channel_count: 1,
                expected_tempo_bpm: 128.0,
                start_position_beats: None,
            })
            .expect("begin V1 capture");

        shared.record_callback(
            &[0.1],
            &CallbackTimingSnapshot {
                is_transport_running: false,
                tempo_bpm: 128.0,
                render_position_beats: 1.0,
                completed_position_beats: 1.0,
            },
            1_000,
        );
        let skipped = shared.progress().expect("V1 skipped callback progress");
        assert_eq!(skipped.written_sample_count, 0);
        assert_eq!(skipped.callback_count, 0);
        assert_eq!(skipped.fault_count(), 0);

        shared.record_callback(&[0.2], &timing(true, 128.0), 2_000);
        let outcome = shared.finish().expect("complete V1 capture");
        assert_eq!(outcome.samples, vec![0.2]);
        assert_eq!(outcome.progress.callback_count, 1);
        assert_eq!(outcome.progress.fault_count(), 0);
    }

    #[test]
    fn bar_window_waits_and_copies_only_post_boundary_frames() {
        let shared = SharedLiveMasterCapture::new();
        shared
            .begin(LiveMasterCaptureRequest {
                target_frame_count: 4,
                channel_count: 1,
                expected_tempo_bpm: 120.0,
                start_position_beats: Some(4.0),
            })
            .expect("begin aligned capture");

        shared.record_callback(&[0.0, 1.0, 2.0, 3.0], &timing_range(3.5, 3.9), 1_000);
        let waiting = shared.progress().expect("armed progress");
        assert_eq!(waiting.written_sample_count, 0);
        assert!(!waiting.capture_started);

        shared.record_callback(&[10.0, 11.0, 12.0, 13.0], &timing_range(3.9, 4.3), 2_000);
        shared.record_callback(&[20.0, 21.0], &timing_range(4.3, 4.5), 3_000);

        let outcome = shared.finish().expect("complete aligned capture");
        assert_eq!(outcome.samples, vec![11.0, 12.0, 13.0, 20.0]);
        assert_eq!(outcome.captured_start_position_beats, Some(4.0));
        assert_eq!(outcome.captured_end_position_beats, Some(4.4));
        assert_eq!(outcome.progress.armed_callback_count, 2);
        assert_eq!(outcome.progress.callback_count, 2);
        assert_eq!(outcome.progress.fault_count(), 0);
    }

    #[test]
    fn bar_window_counts_callback_gaps_while_armed_before_the_boundary() {
        let shared = SharedLiveMasterCapture::new();
        shared
            .begin(LiveMasterCaptureRequest {
                target_frame_count: 2,
                channel_count: 1,
                expected_tempo_bpm: 120.0,
                start_position_beats: Some(4.0),
            })
            .expect("begin armed capture");

        shared.record_callback(&[1.0, 2.0], &timing_range(3.0, 3.2), 1_000);
        shared.record_callback(
            &[3.0, 4.0],
            &timing_range(3.2, 3.4),
            1_000 + LIVE_MASTER_CALLBACK_GAP_THRESHOLD_MICROS + 1,
        );

        let progress = shared.progress().expect("armed gap progress");
        assert_eq!(progress.written_sample_count, 0);
        assert_eq!(progress.callback_count, 0);
        assert_eq!(progress.armed_callback_count, 2);
        assert_eq!(progress.callback_gap_over_threshold_count, 1);
        assert_eq!(progress.fault_count(), 1);
    }

    #[test]
    fn bar_window_counts_the_gap_from_the_last_runtime_callback_into_arming() {
        let shared = SharedLiveMasterCapture::new();
        shared
            .begin_after_callback(
                LiveMasterCaptureRequest {
                    target_frame_count: 2,
                    channel_count: 1,
                    expected_tempo_bpm: 120.0,
                    start_position_beats: Some(4.0),
                },
                Some(1_000),
            )
            .expect("begin armed capture after runtime callback");

        shared.record_callback(
            &[1.0, 2.0],
            &timing_range(3.0, 3.2),
            1_000 + LIVE_MASTER_CALLBACK_GAP_THRESHOLD_MICROS + 1,
        );

        let progress = shared.progress().expect("initial armed gap progress");
        assert_eq!(progress.callback_gap_over_threshold_count, 1);
        assert_eq!(progress.fault_count(), 1);
    }

    #[test]
    fn bar_window_output_is_identical_across_callback_partitions() {
        fn capture(parts: &[(&[f32], f64, f64)]) -> LiveMasterCaptureOutcome {
            let shared = SharedLiveMasterCapture::new();
            shared
                .begin(LiveMasterCaptureRequest {
                    target_frame_count: 4,
                    channel_count: 1,
                    expected_tempo_bpm: 120.0,
                    start_position_beats: Some(4.0),
                })
                .expect("begin partition capture");
            for (index, (samples, start, end)) in parts.iter().enumerate() {
                shared.record_callback(
                    samples,
                    &timing_range(*start, *end),
                    1_000 + index as u64 * 1_000,
                );
            }
            shared.finish().expect("complete partition capture")
        }

        let whole = capture(&[(&[1.0, 2.0, 3.0, 4.0], 4.0, 4.4)]);
        let partitioned = capture(&[(&[1.0, 2.0], 4.0, 4.2), (&[3.0, 4.0], 4.2, 4.4)]);

        assert_eq!(whole.samples, partitioned.samples);
        assert_eq!(whole.captured_start_position_beats, Some(4.0));
        assert_eq!(partitioned.captured_start_position_beats, Some(4.0));
        assert!((whole.captured_end_position_beats.unwrap() - 4.4).abs() < 1.0e-12);
        assert!((partitioned.captured_end_position_beats.unwrap() - 4.4).abs() < 1.0e-12);
    }

    #[test]
    fn bar_window_fails_closed_when_transport_skips_the_requested_start() {
        let shared = SharedLiveMasterCapture::new();
        shared
            .begin(LiveMasterCaptureRequest {
                target_frame_count: 2,
                channel_count: 1,
                expected_tempo_bpm: 120.0,
                start_position_beats: Some(4.0),
            })
            .expect("begin skipped-window capture");

        shared.record_callback(&[1.0, 2.0], &timing_range(4.2, 4.4), 1_000);

        let progress = shared.progress().expect("faulted armed progress");
        assert_eq!(progress.written_sample_count, 0);
        assert_eq!(progress.timing_window_mismatch_count, 1);
        assert_eq!(progress.fault_count(), 1);
        assert!(!progress.capture_started);
        assert!(matches!(
            shared.finish(),
            Err(LiveMasterCaptureError::NotComplete(_))
        ));
    }

    #[test]
    fn bar_window_fails_closed_when_callback_timing_jumps_after_capture_starts() {
        let shared = SharedLiveMasterCapture::new();
        shared
            .begin(LiveMasterCaptureRequest {
                target_frame_count: 4,
                channel_count: 1,
                expected_tempo_bpm: 120.0,
                start_position_beats: Some(4.0),
            })
            .expect("begin discontinuous capture");

        shared.record_callback(&[1.0, 2.0], &timing_range(4.0, 4.2), 1_000);
        shared.record_callback(&[3.0, 4.0], &timing_range(4.4, 4.6), 2_000);

        let progress = shared.progress().expect("faulted capture progress");
        assert_eq!(progress.written_sample_count, 2);
        assert_eq!(progress.timing_window_mismatch_count, 1);
        assert_eq!(progress.fault_count(), 1);
        assert!(progress.capture_started);
        assert!(!progress.complete);
        assert!(matches!(
            shared.finish(),
            Err(LiveMasterCaptureError::NotComplete(_))
        ));
    }

    #[test]
    fn bar_window_fails_closed_immediately_when_transport_stops_after_capture_starts() {
        let shared = SharedLiveMasterCapture::new();
        shared
            .begin(LiveMasterCaptureRequest {
                target_frame_count: 4,
                channel_count: 1,
                expected_tempo_bpm: 120.0,
                start_position_beats: Some(4.0),
            })
            .expect("begin interrupted capture");

        shared.record_callback(&[1.0, 2.0], &timing_range(4.0, 4.2), 1_000);
        shared.record_callback(
            &[3.0, 4.0],
            &CallbackTimingSnapshot {
                is_transport_running: false,
                tempo_bpm: 120.0,
                render_position_beats: 4.2,
                completed_position_beats: 4.2,
            },
            2_000,
        );

        let progress = shared.progress().expect("interrupted progress");
        assert_eq!(progress.written_sample_count, 2);
        assert_eq!(progress.transport_mismatch_count, 1);
        assert_eq!(progress.fault_count(), 1);
        assert!(!progress.complete);
    }

    #[test]
    fn incomplete_capture_cannot_be_finalized_and_can_be_aborted() {
        let shared = SharedLiveMasterCapture::new();
        shared
            .begin(LiveMasterCaptureRequest {
                target_frame_count: 2,
                channel_count: 2,
                expected_tempo_bpm: 128.0,
                start_position_beats: None,
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
                start_position_beats: None,
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
                start_position_beats: None,
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
                start_position_beats: None,
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
