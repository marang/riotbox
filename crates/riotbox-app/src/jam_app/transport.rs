use super::{
    state::JamAppState,
    transport_helpers::{crossed_commit_boundary, transport_clock_for_state},
};
use riotbox_audio::runtime::AudioRuntimeTimingSnapshot;
use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionTarget, ActorType, CommitBoundary, Quantization,
        TargetScope,
    },
    queue::CommittedActionRef,
    transport::TransportClockState,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportToggleCommit {
    pub command: ActionCommand,
    pub committed: Vec<CommittedActionRef>,
}

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
        self.runtime.transport_driver.pending_audio_is_playing = Some(is_playing);
    }

    /// Commit the musician-facing Play/Pause toggle through the Action Lexicon.
    ///
    /// Transport toggles are immediate performer actions: they use the current
    /// transport clock as their commit boundary and do not wait for a future
    /// beat, bar, or phrase.
    pub fn commit_transport_toggle(&mut self, requested_at: TimestampMs) -> TransportToggleCommit {
        let command = if self.runtime.transport.is_playing {
            ActionCommand::TransportPause
        } else {
            ActionCommand::TransportPlay
        };
        let mut draft = ActionDraft::new(
            ActorType::User,
            command,
            Quantization::Immediate,
            ActionTarget {
                scope: Some(TargetScope::Session),
                ..Default::default()
            },
        );
        draft.explanation = Some(
            match command {
                ActionCommand::TransportPlay => "start transport",
                ActionCommand::TransportPause => "pause transport",
                _ => unreachable!("transport toggle only emits play or pause"),
            }
            .into(),
        );
        self.queue.enqueue(draft, requested_at);

        let boundary = self
            .runtime
            .transport
            .boundary_state(CommitBoundary::Immediate);
        let committed = self.commit_ready_actions(boundary, requested_at);

        TransportToggleCommit { command, committed }
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

        if let Some(boundary) =
            crossed_commit_boundary(&previous, &next_clock, self.source_graph.as_ref())
        {
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
        if let Some(pending_is_playing) = self.runtime.transport_driver.pending_audio_is_playing {
            if timing.is_transport_running != pending_is_playing {
                return Vec::new();
            }
            self.runtime.transport_driver.pending_audio_is_playing = None;
        } else if self.runtime.transport.is_playing && !timing.is_transport_running {
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
            && let Some(boundary) =
                crossed_commit_boundary(&previous, &next_clock, self.source_graph.as_ref())
        {
            return self.commit_ready_actions(boundary, committed_at);
        }

        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use riotbox_core::{
        action::{ActionCommand, ActionParams, ActionStatus, CommitBoundary, TargetScope},
        queue::ActionQueue,
        replay::{apply_replay_plan_to_session, build_committed_replay_plan},
        session::SessionFile,
    };

    use super::JamAppState;

    #[test]
    fn musician_transport_toggle_commits_play_and_pause_through_every_live_surface() {
        let mut session =
            SessionFile::new("transport-toggle", "riotbox-test", "2026-08-22T00:00:00Z");
        session.runtime_state.transport.position_beats = 12.5;
        let mut state = JamAppState::from_parts(session, None, ActionQueue::new());

        let play = state.commit_transport_toggle(100);

        assert_eq!(play.command, ActionCommand::TransportPlay);
        assert_eq!(play.committed.len(), 1);
        assert_eq!(play.committed[0].boundary.kind, CommitBoundary::Immediate);
        assert_eq!(play.committed[0].commit_sequence, 1);
        let play_action = state
            .queue
            .history_action(play.committed[0].action_id)
            .expect("queued play action committed into queue history");
        assert_eq!(play_action.command, ActionCommand::TransportPlay);
        assert_eq!(play_action.params, ActionParams::Empty);
        assert_eq!(play_action.target.scope, Some(TargetScope::Session));
        assert_eq!(play_action.status, ActionStatus::Committed);
        assert!(state.queue.pending_actions().is_empty());
        assert!(state.session.runtime_state.transport.is_playing);
        assert!(state.runtime.transport.is_playing);
        assert_eq!(
            state.runtime.transport_driver.pending_audio_is_playing,
            Some(true)
        );
        assert_eq!(state.session.action_log.actions.len(), 1);
        assert_eq!(state.session.action_log.commit_records.len(), 1);

        let pause = state.commit_transport_toggle(200);

        assert_eq!(pause.command, ActionCommand::TransportPause);
        assert_eq!(pause.committed.len(), 1);
        assert_eq!(pause.committed[0].boundary.kind, CommitBoundary::Immediate);
        assert_eq!(pause.committed[0].commit_sequence, 2);
        assert!(!state.session.runtime_state.transport.is_playing);
        assert!(!state.runtime.transport.is_playing);
        assert_eq!(state.runtime.transport.position_beats, 12.5);
        assert_eq!(
            state.runtime.transport_driver.pending_audio_is_playing,
            Some(false)
        );
        assert_eq!(state.session.action_log.actions.len(), 2);
        assert_eq!(state.session.action_log.commit_records.len(), 2);
    }

    #[test]
    fn committed_transport_toggle_replays_to_the_same_transport_state() {
        let mut baseline =
            SessionFile::new("transport-replay", "riotbox-test", "2026-08-22T00:00:00Z");
        baseline.runtime_state.transport.position_beats = 12.5;
        let mut live = JamAppState::from_parts(baseline.clone(), None, ActionQueue::new());

        live.commit_transport_toggle(100);
        let play_plan =
            build_committed_replay_plan(&live.session.action_log).expect("committed play plan");
        let mut replayed_play = baseline.clone();
        apply_replay_plan_to_session(&mut replayed_play, &play_plan)
            .expect("replay committed play action");
        assert_eq!(
            replayed_play.runtime_state.transport,
            live.session.runtime_state.transport
        );

        live.commit_transport_toggle(200);
        let pause_plan =
            build_committed_replay_plan(&live.session.action_log).expect("committed pause plan");
        let mut replayed_pause = baseline;
        apply_replay_plan_to_session(&mut replayed_pause, &pause_plan)
            .expect("replay committed play and pause actions");
        assert_eq!(
            replayed_pause.runtime_state.transport,
            live.session.runtime_state.transport
        );
    }
}
