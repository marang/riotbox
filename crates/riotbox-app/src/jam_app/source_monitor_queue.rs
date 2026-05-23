use super::state::{JamAppState, QueueControlResult};
use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType, Quantization,
        SourceMonitorMode, TargetScope,
    },
};

impl JamAppState {
    pub fn queue_source_monitor_mode(
        &mut self,
        mode: SourceMonitorMode,
        requested_at: TimestampMs,
    ) -> QueueControlResult {
        if self
            .queue
            .pending_actions()
            .iter()
            .any(|action| action.command == ActionCommand::SourceMonitorSetMode)
        {
            return QueueControlResult::AlreadyPending;
        }
        if self.session.runtime_state.source_monitor.mode == mode {
            return QueueControlResult::AlreadyInState;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::SourceMonitorSetMode,
            Quantization::Immediate,
            ActionTarget {
                scope: Some(TargetScope::Session),
                ..Default::default()
            },
        );
        draft.params = ActionParams::SourceMonitor { mode: Some(mode) };
        draft.explanation = Some(format!("set source monitor to {mode}"));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }
}
