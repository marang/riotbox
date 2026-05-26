use riotbox_core::{
    action::{Action, ActionCommand, ActionParams},
    session::SessionFile,
};

pub(in crate::jam_app) fn apply_transport_side_effects(
    session: &mut SessionFile,
    action: &Action,
) -> bool {
    match action.command {
        ActionCommand::TransportPlay => {
            session.runtime_state.transport.is_playing = true;
            true
        }
        ActionCommand::TransportPause => {
            session.runtime_state.transport.is_playing = false;
            true
        }
        ActionCommand::TransportStop => {
            session.runtime_state.transport.is_playing = false;
            session.runtime_state.transport.position_beats = 0.0;
            true
        }
        ActionCommand::TransportSeek => {
            let ActionParams::Transport {
                position_beats: Some(position_beats),
            } = action.params
            else {
                return false;
            };
            session.runtime_state.transport.position_beats = position_beats as f64;
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use riotbox_core::{
        TimestampMs,
        action::{
            Action, ActionCommand, ActionParams, ActionStatus, ActionTarget, ActorType,
            Quantization, UndoPolicy,
        },
        ids::ActionId,
        session::SessionFile,
    };

    use super::apply_transport_side_effects;

    #[test]
    fn transport_side_effect_seeks_session_runtime_position_without_pausing() {
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-05-23T15:02:00Z");
        session.runtime_state.transport.is_playing = true;
        session.runtime_state.transport.position_beats = 4.0;
        let action = transport_action(
            ActionCommand::TransportSeek,
            ActionParams::Transport {
                position_beats: Some(16),
            },
            100,
        );

        assert!(apply_transport_side_effects(&mut session, &action));

        assert!(session.runtime_state.transport.is_playing);
        assert_eq!(session.runtime_state.transport.position_beats, 16.0);
    }

    fn transport_action(
        command: ActionCommand,
        params: ActionParams,
        requested_at: TimestampMs,
    ) -> Action {
        Action {
            id: ActionId(1),
            actor: ActorType::User,
            command,
            params,
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
