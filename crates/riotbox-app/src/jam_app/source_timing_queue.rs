use super::state::{JamAppState, QueueControlResult};
use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType, Quantization,
        TargetScope, UndoPolicy,
    },
};

impl JamAppState {
    pub fn queue_source_timing_grid_confirmation(
        &mut self,
        requested_at: TimestampMs,
    ) -> QueueControlResult {
        let Some(graph) = self.source_graph.as_ref() else {
            return QueueControlResult::AlreadyInState;
        };
        let source_id = graph.source.source_id.clone();
        let hypothesis_id = graph.timing.primary_hypothesis_id.clone();

        if self
            .queue
            .pending_actions()
            .iter()
            .any(|action| action.command == ActionCommand::SourceTimingConfirmGrid)
        {
            return QueueControlResult::AlreadyPending;
        }
        if self
            .session
            .runtime_state
            .source_timing
            .confirmed_grid
            .as_ref()
            .is_some_and(|confirmed| {
                confirmed.source_id == source_id
                    && confirmed.hypothesis_id.as_deref() == hypothesis_id.as_deref()
            })
        {
            return QueueControlResult::AlreadyInState;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::SourceTimingConfirmGrid,
            Quantization::Immediate,
            ActionTarget {
                scope: Some(TargetScope::Session),
                object_id: hypothesis_id.clone(),
                ..Default::default()
            },
        );
        draft.params = ActionParams::SourceTimingGrid {
            source_id: Some(source_id),
            hypothesis_id: hypothesis_id.clone(),
        };
        draft.undo_policy = UndoPolicy::NotUndoable {
            reason: "source timing grid confirmation requires a dedicated revert action".into(),
        };
        draft.explanation = Some(match hypothesis_id {
            Some(hypothesis_id) => format!("confirm source timing grid {hypothesis_id}"),
            None => "confirm source timing grid".into(),
        });
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }
}
