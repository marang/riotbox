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
            .any(|action| source_timing_trust_change_pending(action.command))
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

    pub fn queue_source_timing_grid_revert(
        &mut self,
        requested_at: TimestampMs,
    ) -> QueueControlResult {
        let Some(confirmed) = self
            .session
            .runtime_state
            .source_timing
            .confirmed_grid
            .as_ref()
        else {
            return QueueControlResult::AlreadyInState;
        };
        if self
            .queue
            .pending_actions()
            .iter()
            .any(|action| source_timing_trust_change_pending(action.command))
        {
            return QueueControlResult::AlreadyPending;
        }

        let source_id = confirmed.source_id.clone();
        let hypothesis_id = confirmed.hypothesis_id.clone();
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::SourceTimingRevertGrid,
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
            reason: "source timing grid revert is an explicit trust-state action".into(),
        };
        draft.explanation = Some(match hypothesis_id {
            Some(hypothesis_id) => {
                format!("revert source timing grid confirmation {hypothesis_id}")
            }
            None => "revert source timing grid confirmation".into(),
        });
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }
}

fn source_timing_trust_change_pending(command: ActionCommand) -> bool {
    matches!(
        command,
        ActionCommand::SourceTimingConfirmGrid | ActionCommand::SourceTimingRevertGrid
    )
}
