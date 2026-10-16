//! One control-side handoff for the interactive shell and live recorder.
//! Session owns intent; the audio owner filters observations until the callback
//! acknowledges the latest delivered reposition. No clock lives here.

use crate::jam_app::{JamAppError, JamAppState};
use riotbox_audio::runtime::{
    AudioRuntimeShell, AudioRuntimeTimingSnapshot, AudioTransportControlError,
    AudioTransportPositionUpdate,
};
use riotbox_core::{TimestampMs, queue::CommittedActionRef};

pub(super) trait TransportControl {
    fn timing_snapshot(&self) -> Option<AudioRuntimeTimingSnapshot>;
    fn update_transport_state(
        &self,
        playing: bool,
        bpm: f32,
        position: AudioTransportPositionUpdate,
    ) -> Result<(), AudioTransportControlError>;
}

impl TransportControl for AudioRuntimeShell {
    fn timing_snapshot(&self) -> Option<AudioRuntimeTimingSnapshot> {
        self.timing_snapshot()
    }

    fn update_transport_state(
        &self,
        playing: bool,
        bpm: f32,
        position: AudioTransportPositionUpdate,
    ) -> Result<(), AudioTransportControlError> {
        self.update_transport_state(playing, bpm, position)
    }
}

pub(super) fn publish_transport(
    state: &mut JamAppState,
    runtime: &impl TransportControl,
) -> Result<(), AudioTransportControlError> {
    let pending = state.runtime.transport_driver.pending_audio_position;
    runtime.update_transport_state(
        state.runtime.transport.is_playing,
        state.runtime.tr909_render.tempo_bpm,
        pending.map_or(
            AudioTransportPositionUpdate::Preserve,
            AudioTransportPositionUpdate::Reposition,
        ),
    )?;
    // Delivery is not callback acknowledgment. The runtime's timing getter
    // remains unavailable until that exact request has been applied.
    state.runtime.transport_driver.pending_audio_position = None;
    Ok(())
}

pub(super) fn sync_transport(
    state: &mut JamAppState,
    runtime: &impl TransportControl,
    committed_at: TimestampMs,
) -> Result<Vec<CommittedActionRef>, JamAppError> {
    // Navigation may have committed since the preceding control-loop iteration.
    // Dispatch it BEFORE reading an observation, not after it overwrote intent.
    publish_transport(state, runtime)?;
    let committed = state.apply_audio_timing_snapshot(runtime.timing_snapshot(), committed_at)?;
    // Crossing a queue boundary can itself commit transport intent.
    publish_transport(state, runtime)?;
    Ok(committed)
}

#[cfg(test)]
mod tests;
