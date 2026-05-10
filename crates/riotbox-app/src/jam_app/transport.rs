use super::{
    state::JamAppState,
    transport_helpers::{crossed_commit_boundary, transport_clock_for_state},
};
use riotbox_audio::runtime::AudioRuntimeTimingSnapshot;
use riotbox_core::{TimestampMs, queue::CommittedActionRef, transport::TransportClockState};

impl JamAppState {
    pub fn update_transport_clock(&mut self, clock: TransportClockState) {
        self.runtime.transport = clock.clone();
        self.session.runtime_state.transport.is_playing = clock.is_playing;
        self.session.runtime_state.transport.position_beats = clock.position_beats;
        self.session.runtime_state.transport.current_scene = clock.current_scene.clone();
        self.session.runtime_state.scene_state.active_scene = clock.current_scene;
        self.refresh_view();
    }

    pub fn set_transport_playing(&mut self, is_playing: bool) {
        let next_clock = transport_clock_for_state(
            self.runtime.transport.position_beats,
            is_playing,
            self.runtime.transport.current_scene.clone(),
            self.source_graph.as_ref(),
        );
        self.update_transport_clock(next_clock);
        self.runtime.transport_driver.last_audio_position_beats =
            is_playing.then_some(self.runtime.transport.beat_index);
    }

    pub fn advance_transport_by(
        &mut self,
        delta_beats: f64,
        committed_at: TimestampMs,
    ) -> Vec<CommittedActionRef> {
        if !self.runtime.transport.is_playing || delta_beats <= 0.0 {
            return Vec::new();
        }

        let previous = self.runtime.transport.clone();
        let next_position = (previous.position_beats + delta_beats).max(0.0);
        let next_clock = transport_clock_for_state(
            next_position,
            true,
            previous.current_scene.clone(),
            self.source_graph.as_ref(),
        );
        self.update_transport_clock(next_clock.clone());

        if let Some(boundary) = crossed_commit_boundary(&previous, &next_clock) {
            self.commit_ready_actions(boundary, committed_at)
        } else {
            Vec::new()
        }
    }

    pub fn apply_audio_timing_snapshot(
        &mut self,
        timing: AudioRuntimeTimingSnapshot,
        committed_at: TimestampMs,
    ) -> Vec<CommittedActionRef> {
        if self.runtime.transport.is_playing && !timing.is_transport_running {
            return Vec::new();
        }

        let previous = self.runtime.transport.clone();
        let next_clock = transport_clock_for_state(
            timing.position_beats,
            timing.is_transport_running,
            previous.current_scene.clone(),
            self.source_graph.as_ref(),
        );
        self.update_transport_clock(next_clock.clone());
        self.runtime.transport_driver.last_audio_position_beats =
            timing.is_transport_running.then_some(next_clock.beat_index);

        if timing.is_transport_running
            && let Some(boundary) = crossed_commit_boundary(&previous, &next_clock)
        {
            return self.commit_ready_actions(boundary, committed_at);
        }

        Vec::new()
    }
}
