use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType, CaptureLengthIntent,
        Quantization, TargetScope, UndoPolicy,
    },
};

use super::state::{JamAppState, QueueControlResult};

impl JamAppState {
    pub fn queue_capture_length_intent(
        &mut self,
        intent: CaptureLengthIntent,
        requested_at: TimestampMs,
    ) -> QueueControlResult {
        if self
            .queue
            .pending_actions()
            .iter()
            .any(|action| action.command == ActionCommand::CaptureSetLength)
        {
            return QueueControlResult::AlreadyPending;
        }
        if self.session.runtime_state.capture.length_intent == intent {
            return QueueControlResult::AlreadyInState;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::CaptureSetLength,
            Quantization::Immediate,
            ActionTarget {
                scope: Some(TargetScope::Session),
                object_id: Some("capture-length".into()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::CaptureLength {
            intent: Some(intent),
        };
        draft.undo_policy = UndoPolicy::NotUndoable {
            reason: "capture length selection is replaced by the next explicit selection".into(),
        };
        draft.explanation = Some(format!("set capture length to {intent}"));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_next_capture_length_intent(
        &mut self,
        requested_at: TimestampMs,
    ) -> QueueControlResult {
        self.queue_capture_length_intent(
            self.session.runtime_state.capture.length_intent.next(),
            requested_at,
        )
    }

    pub fn queue_previous_capture_length_intent(
        &mut self,
        requested_at: TimestampMs,
    ) -> QueueControlResult {
        self.queue_capture_length_intent(
            self.session.runtime_state.capture.length_intent.previous(),
            requested_at,
        )
    }

    pub fn queue_capture_bar(&mut self, requested_at: TimestampMs) {
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::CaptureBarGroup,
            Quantization::NextPhrase,
            ActionTarget {
                scope: Some(TargetScope::LaneW30),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Capture { bars: None };
        draft.explanation = Some(format!(
            "capture {} from source into W-30 path",
            self.session.runtime_state.capture.length_intent
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
    }
}
