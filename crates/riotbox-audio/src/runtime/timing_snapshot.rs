//! Single-callback-writer publication of observed transport, never requested state.
//!
//! Only atomic primitives vary in the Loom target; it models this exact owner.
use super::timing_sync::{AtomicBool, AtomicU32, AtomicU64, Ordering};

const READ_ATTEMPTS: usize = 3;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct AudioRuntimeTimingSnapshot {
    pub is_transport_running: bool,
    pub tempo_bpm: f32,
    pub position_beats: f64,
}

pub(super) struct CallbackTimingPublication {
    revision: AtomicU64,
    is_transport_running: AtomicBool,
    tempo_bpm_bits: AtomicU32,
    position_beats_bits: AtomicU64,
    applied_position_request_id: AtomicU64,
}

impl CallbackTimingPublication {
    pub(super) fn new() -> Self {
        Self {
            revision: AtomicU64::new(0),
            is_transport_running: AtomicBool::new(false),
            tempo_bpm_bits: AtomicU32::new(0),
            position_beats_bits: AtomicU64::new(0),
            applied_position_request_id: AtomicU64::new(0),
        }
    }

    /// Exactly one writer: the unique production output callback.
    pub(super) fn publish(&self, timing: AudioRuntimeTimingSnapshot, applied_request_id: u64) {
        let revision = self.revision.load(Ordering::SeqCst);
        self.revision
            .store(revision.wrapping_add(1), Ordering::SeqCst);
        self.is_transport_running
            .store(timing.is_transport_running, Ordering::SeqCst);
        self.tempo_bpm_bits
            .store(timing.tempo_bpm.to_bits(), Ordering::SeqCst);
        self.position_beats_bits
            .store(timing.position_beats.to_bits(), Ordering::SeqCst);
        self.applied_position_request_id
            .store(applied_request_id, Ordering::SeqCst);
        // Keep zero reserved for no publication, even at counter rollover.
        self.revision
            .store(revision.wrapping_add(2).max(2), Ordering::SeqCst);
    }

    pub(super) fn snapshot_with_position_request(
        &self,
    ) -> Option<(AudioRuntimeTimingSnapshot, u64)> {
        for _ in 0..READ_ATTEMPTS {
            let before = self.revision.load(Ordering::SeqCst);
            if before == 0 || !before.is_multiple_of(2) {
                continue;
            }
            let timing = AudioRuntimeTimingSnapshot {
                is_transport_running: self.is_transport_running.load(Ordering::SeqCst),
                tempo_bpm: f32::from_bits(self.tempo_bpm_bits.load(Ordering::SeqCst)),
                position_beats: f64::from_bits(self.position_beats_bits.load(Ordering::SeqCst)),
            };
            let applied_request_id = self.applied_position_request_id.load(Ordering::SeqCst);
            // All publication and read operations share the sequentially
            // consistent order: matching even revisions cannot enclose a
            // partial new payload. Never return an unchecked last read.
            if before == self.revision.load(Ordering::SeqCst) {
                return Some((timing, applied_request_id));
            }
        }
        None
    }
}
