use riotbox_core::{
    action::{Action, ActionCommand, ActionParams},
    session::SessionFile,
};

pub(in crate::jam_app) fn apply_capture_side_effects(
    session: &mut SessionFile,
    action: &Action,
) -> bool {
    let ActionCommand::CaptureSetLength = action.command else {
        return false;
    };
    let ActionParams::CaptureLength {
        intent: Some(intent),
    } = action.params
    else {
        return false;
    };

    session.runtime_state.capture.length_intent = intent;
    session.runtime_state.capture.length_set_by_action = Some(action.id);
    session.runtime_state.capture.length_set_at = action.committed_at.or(Some(action.requested_at));
    true
}

#[cfg(test)]
mod tests {
    use riotbox_core::{
        TimestampMs,
        action::{
            Action, ActionCommand, ActionParams, ActionStatus, ActionTarget, ActorType,
            CaptureLengthIntent, Quantization, UndoPolicy,
        },
        ids::ActionId,
        session::SessionFile,
    };

    use super::apply_capture_side_effects;

    #[test]
    fn capture_length_side_effect_updates_session_runtime_intent() {
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-05-23T15:30:00Z");
        let action = capture_length_action(CaptureLengthIntent::Phrase, 120);

        assert!(apply_capture_side_effects(&mut session, &action));

        assert_eq!(
            session.runtime_state.capture.length_intent,
            CaptureLengthIntent::Phrase
        );
        assert_eq!(
            session.runtime_state.capture.length_set_by_action,
            Some(ActionId(1))
        );
        assert_eq!(session.runtime_state.capture.length_set_at, Some(120));
    }

    fn capture_length_action(intent: CaptureLengthIntent, requested_at: TimestampMs) -> Action {
        Action {
            id: ActionId(1),
            actor: ActorType::User,
            command: ActionCommand::CaptureSetLength,
            params: ActionParams::CaptureLength {
                intent: Some(intent),
            },
            target: ActionTarget::default(),
            requested_at,
            quantization: Quantization::Immediate,
            status: ActionStatus::Committed,
            committed_at: Some(requested_at),
            result: None,
            undo_policy: UndoPolicy::Undoable,
            explanation: None,
        }
    }
}
