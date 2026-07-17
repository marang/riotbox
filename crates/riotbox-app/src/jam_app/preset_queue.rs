use super::state::{JamAppState, QueueControlResult};
use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType, Quantization,
        TargetScope,
    },
    style::PerformancePresetId,
};

impl JamAppState {
    pub fn queue_performance_preset(
        &mut self,
        preset_id: PerformancePresetId,
        requested_at: TimestampMs,
    ) -> QueueControlResult {
        if self
            .queue
            .has_pending_command(ActionCommand::PresetActivate)
        {
            return QueueControlResult::AlreadyPending;
        }
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::PresetActivate,
            Quantization::Immediate,
            ActionTarget {
                scope: Some(TargetScope::Session),
                object_id: Some(preset_id.contract_id().into()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Preset { preset_id };
        draft.explanation = Some(format!("activate {}", preset_id.label()));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }
}

#[cfg(test)]
mod tests {
    use riotbox_core::{
        action::{ActionCommand, ActionParams, CommitBoundary},
        queue::ActionQueue,
        session::SessionFile,
        style::{PerformancePresetId, StyleProfileId},
        transport::CommitBoundaryState,
    };

    use super::{JamAppState, QueueControlResult};

    #[test]
    fn preset_activation_queues_once_while_pending() {
        let session = SessionFile::new("session-1", "riotbox-test", "2026-07-17T14:30:00Z");
        let mut state = JamAppState::from_parts(session, None, ActionQueue::new());

        assert_eq!(
            state.queue_performance_preset(PerformancePresetId::FeralBreakAlphaV1, 100),
            QueueControlResult::Enqueued
        );
        assert_eq!(
            state.queue_performance_preset(PerformancePresetId::FeralBreakAlphaV1, 101),
            QueueControlResult::AlreadyPending
        );

        let pending = state.queue.pending_actions();
        let queued = pending.first().expect("queued preset action");
        assert_eq!(queued.command, ActionCommand::PresetActivate);
        assert_eq!(
            queued.params,
            ActionParams::Preset {
                preset_id: PerformancePresetId::FeralBreakAlphaV1
            }
        );
    }

    #[test]
    fn preset_defaults_can_be_recalled_through_session_product_spine() {
        let session = SessionFile::new("session-1", "riotbox-test", "2026-07-17T14:32:00Z");
        let mut state = JamAppState::from_parts(session, None, ActionQueue::new());
        assert_eq!(
            state.queue_performance_preset(PerformancePresetId::FeralBreakAlphaV1, 100),
            QueueControlResult::Enqueued
        );

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Immediate,
                beat_index: 0,
                bar_index: 0,
                phrase_index: 0,
                scene_id: None,
            },
            100,
        );

        assert_eq!(committed.len(), 1);
        assert_eq!(
            state.session.runtime_state.style.active_profile,
            Some(StyleProfileId::FeralRebuild)
        );
        assert_eq!(
            state.session.runtime_state.style.active_preset,
            Some(PerformancePresetId::FeralBreakAlphaV1)
        );
        assert_eq!(state.session.action_log.actions.len(), 1);
        assert_eq!(state.session.action_log.commit_records.len(), 1);
        state.session.runtime_state.macro_state.chaos = 0.0;
        assert_eq!(
            state.queue_performance_preset(PerformancePresetId::FeralBreakAlphaV1, 101),
            QueueControlResult::Enqueued,
            "the musician must be able to recall versioned defaults after later control edits"
        );
        let recalled = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Immediate,
                beat_index: 0,
                bar_index: 0,
                phrase_index: 0,
                scene_id: None,
            },
            101,
        );
        assert_eq!(recalled.len(), 1);
        assert_eq!(
            state.session.runtime_state.macro_state,
            PerformancePresetId::FeralBreakAlphaV1
                .definition()
                .macro_state
        );
        assert_eq!(state.session.action_log.actions.len(), 2);
    }
}
