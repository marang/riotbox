//! Requested transport publication, distinct from observed callback timing.
use super::control_snapshot::{
    begin_coherent_snapshot_update, coherent_snapshot_or, finish_coherent_snapshot_update,
    try_coherent_snapshot,
};
use super::control_sync::{AtomicBool, AtomicU32, AtomicU64, Ordering};

/// A runtime-only target, not a second persisted transport position.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AudioTransportPositionRequest(u64);

impl AudioTransportPositionRequest {
    #[must_use]
    pub const fn from_beats(position_beats: f64) -> Self {
        Self(position_beats.to_bits())
    }

    #[must_use]
    pub const fn position_beats(self) -> f64 {
        f64::from_bits(self.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AudioTransportPositionUpdate {
    Preserve,
    Reposition(AudioTransportPositionRequest),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AudioTransportControlError {
    InvalidPosition,
    PositionRequestExhausted,
}

impl std::fmt::Display for AudioTransportControlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidPosition => "audio transport target must be finite and nonnegative",
            Self::PositionRequestExhausted => {
                "audio transport position requests exhausted; restart the runtime"
            }
        })
    }
}

impl std::error::Error for AudioTransportControlError {}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct RealtimeTransportTimingState {
    pub(super) is_transport_running: bool,
    pub(super) tempo_bpm: f32,
    pub(super) position_beats: f64,
    pub(super) position_request_id: u64,
}

pub(super) struct SharedTransportTimingState {
    revision: AtomicU64,
    is_transport_running: AtomicBool,
    tempo_bpm_bits: AtomicU32,
    position_beats_bits: AtomicU64,
    position_request_id: AtomicU64,
}

impl SharedTransportTimingState {
    pub(super) fn new(is_transport_running: bool, tempo_bpm: f32, position_beats: f64) -> Self {
        Self {
            revision: AtomicU64::new(0),
            is_transport_running: AtomicBool::new(is_transport_running),
            tempo_bpm_bits: AtomicU32::new(tempo_bpm.to_bits()),
            position_beats_bits: AtomicU64::new(position_beats.to_bits()),
            position_request_id: AtomicU64::new(0),
        }
    }

    /// The shell serializes control writers. Validate and allocate before any mutation.
    pub(super) fn update(
        &self,
        is_transport_running: bool,
        tempo_bpm: f32,
        position: AudioTransportPositionUpdate,
    ) -> Result<(), AudioTransportControlError> {
        let reposition = match position {
            AudioTransportPositionUpdate::Preserve => None,
            AudioTransportPositionUpdate::Reposition(request) => {
                let target = request.position_beats();
                if !target.is_finite() || target < 0.0 {
                    return Err(AudioTransportControlError::InvalidPosition);
                }
                let id = self
                    .position_request_id
                    .load(Ordering::Acquire)
                    .checked_add(1)
                    .ok_or(AudioTransportControlError::PositionRequestExhausted)?;
                Some((target, id))
            }
        };
        begin_coherent_snapshot_update(&self.revision);
        self.is_transport_running
            .store(is_transport_running, Ordering::Release);
        self.tempo_bpm_bits
            .store(tempo_bpm.to_bits(), Ordering::Release);
        if let Some((target, id)) = reposition {
            self.position_beats_bits
                .store(target.to_bits(), Ordering::Release);
            self.position_request_id.store(id, Ordering::Release);
        }
        finish_coherent_snapshot_update(&self.revision);
        Ok(())
    }

    pub(super) fn try_snapshot(&self) -> Option<RealtimeTransportTimingState> {
        try_coherent_snapshot(&self.revision, || self.read_snapshot_fields())
    }

    /// Exclusive offline ownership excludes concurrent control writers.
    #[cfg(test)]
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
            position_request_id: self.position_request_id.load(Ordering::Acquire),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Ordering;
    use super::{
        AudioTransportControlError, AudioTransportPositionRequest, AudioTransportPositionUpdate,
        SharedTransportTimingState,
    };

    #[test]
    fn invalid_targets_and_exhausted_requests_leave_the_whole_publication_unchanged() {
        // This exact source is also included by control_snapshot_loom: its
        // test context supplies the same model seam with Loom atomics.
        super::super::control_sync::model(|| {
            let state = SharedTransportTimingState::new(true, 120.0, 12.5);
            for target in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.01] {
                let before = state.try_snapshot().unwrap();
                let revision = state.revision.load(Ordering::Acquire);
                assert_eq!(
                    state.update(
                        false,
                        99.0,
                        AudioTransportPositionUpdate::Reposition(
                            AudioTransportPositionRequest::from_beats(target)
                        )
                    ),
                    Err(AudioTransportControlError::InvalidPosition)
                );
                assert_eq!(state.try_snapshot(), Some(before));
                assert_eq!(state.revision.load(Ordering::Acquire), revision);
            }
            state.position_request_id.store(u64::MAX, Ordering::Release);
            let before = state.try_snapshot().unwrap();
            let revision = state.revision.load(Ordering::Acquire);
            assert_eq!(
                state.update(
                    false,
                    99.0,
                    AudioTransportPositionUpdate::Reposition(
                        AudioTransportPositionRequest::from_beats(0.0)
                    )
                ),
                Err(AudioTransportControlError::PositionRequestExhausted)
            );
            assert_eq!(state.try_snapshot(), Some(before));
            assert_eq!(state.revision.load(Ordering::Acquire), revision);
            state
                .update(false, 99.0, AudioTransportPositionUpdate::Preserve)
                .unwrap();
            let preserved = state.try_snapshot().unwrap();
            assert!(!preserved.is_transport_running);
            assert_eq!(preserved.tempo_bpm, 99.0);
            assert_eq!(preserved.position_beats, 12.5);
            assert_eq!(preserved.position_request_id, u64::MAX);
        });
    }
}
