//! Requested transport publication, distinct from observed callback timing.
use super::control_snapshot::{
    begin_coherent_snapshot_update, coherent_snapshot_or, finish_coherent_snapshot_update,
    try_coherent_snapshot,
};
use super::control_sync::{AtomicBool, AtomicU32, AtomicU64, Ordering};

#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct RealtimeTransportTimingState {
    pub(super) is_transport_running: bool,
    pub(super) tempo_bpm: f32,
    pub(super) position_beats: f64,
}

pub(super) struct SharedTransportTimingState {
    revision: AtomicU64,
    is_transport_running: AtomicBool,
    tempo_bpm_bits: AtomicU32,
    position_beats_bits: AtomicU64,
}

impl SharedTransportTimingState {
    pub(super) fn new(is_transport_running: bool, tempo_bpm: f32, position_beats: f64) -> Self {
        Self {
            revision: AtomicU64::new(0),
            is_transport_running: AtomicBool::new(is_transport_running),
            tempo_bpm_bits: AtomicU32::new(tempo_bpm.to_bits()),
            position_beats_bits: AtomicU64::new(position_beats.to_bits()),
        }
    }

    pub(super) fn update(&self, is_transport_running: bool, tempo_bpm: f32, position_beats: f64) {
        begin_coherent_snapshot_update(&self.revision);
        self.is_transport_running
            .store(is_transport_running, Ordering::Release);
        self.tempo_bpm_bits
            .store(tempo_bpm.to_bits(), Ordering::Release);
        self.position_beats_bits
            .store(position_beats.to_bits(), Ordering::Release);
        finish_coherent_snapshot_update(&self.revision);
    }

    pub(super) fn try_snapshot(&self) -> Option<RealtimeTransportTimingState> {
        try_coherent_snapshot(&self.revision, || self.read_snapshot_fields())
    }

    /// Exclusive offline ownership excludes concurrent control writers.
    pub(super) fn snapshot_exclusive(&mut self) -> RealtimeTransportTimingState {
        self.read_snapshot_fields()
    }

    pub(super) fn snapshot_or_previous(
        &self,
        previous: &RealtimeTransportTimingState,
    ) -> RealtimeTransportTimingState {
        coherent_snapshot_or(&self.revision, previous, || self.read_snapshot_fields())
    }

    fn read_snapshot_fields(&self) -> RealtimeTransportTimingState {
        RealtimeTransportTimingState {
            is_transport_running: self.is_transport_running.load(Ordering::Acquire),
            tempo_bpm: f32::from_bits(self.tempo_bpm_bits.load(Ordering::Acquire)),
            position_beats: f64::from_bits(self.position_beats_bits.load(Ordering::Acquire)),
        }
    }
}
