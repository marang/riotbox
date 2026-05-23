use riotbox_core::{
    action::{Action, ActionCommand, ActionParams},
    session::{SessionFile, SourceTimingGridConfirmationState},
};

pub(in crate::jam_app) fn apply_source_timing_side_effects(
    session: &mut SessionFile,
    action: &Action,
) -> bool {
    if action.command != ActionCommand::SourceTimingConfirmGrid {
        return false;
    }
    let ActionParams::SourceTimingGrid {
        source_id: Some(source_id),
        hypothesis_id,
    } = &action.params
    else {
        return false;
    };

    session.runtime_state.source_timing.confirmed_grid = Some(SourceTimingGridConfirmationState {
        source_id: source_id.clone(),
        hypothesis_id: hypothesis_id.clone(),
        confirmed_by_action: action.id,
        confirmed_at: action.committed_at.unwrap_or(action.requested_at),
    });
    true
}

#[cfg(test)]
mod tests {
    use riotbox_core::{
        TimestampMs,
        action::{
            Action, ActionCommand, ActionParams, ActionStatus, ActionTarget, ActorType,
            Quantization, UndoPolicy,
        },
        ids::{ActionId, SourceId},
        session::SessionFile,
    };

    use super::apply_source_timing_side_effects;

    #[test]
    fn source_timing_side_effect_confirms_grid_in_session_runtime_state() {
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-05-23T12:20:00Z");
        let action = confirm_grid_action("src-1", Some("primary-grid"), 100);

        assert!(apply_source_timing_side_effects(&mut session, &action));

        let confirmed = session
            .runtime_state
            .source_timing
            .confirmed_grid
            .expect("grid confirmation");
        assert_eq!(confirmed.source_id, SourceId::from("src-1"));
        assert_eq!(confirmed.hypothesis_id.as_deref(), Some("primary-grid"));
        assert_eq!(confirmed.confirmed_by_action, ActionId(1));
        assert_eq!(confirmed.confirmed_at, 100);
    }

    #[test]
    fn source_timing_side_effect_rejects_missing_source_id() {
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-05-23T12:21:00Z");
        let mut action = confirm_grid_action("src-1", Some("primary-grid"), 100);
        action.params = ActionParams::SourceTimingGrid {
            source_id: None,
            hypothesis_id: Some("primary-grid".into()),
        };

        assert!(!apply_source_timing_side_effects(&mut session, &action));
        assert!(session.runtime_state.source_timing.confirmed_grid.is_none());
    }

    fn confirm_grid_action(
        source_id: &str,
        hypothesis_id: Option<&str>,
        requested_at: TimestampMs,
    ) -> Action {
        Action {
            id: ActionId(1),
            actor: ActorType::User,
            command: ActionCommand::SourceTimingConfirmGrid,
            params: ActionParams::SourceTimingGrid {
                source_id: Some(SourceId::from(source_id)),
                hypothesis_id: hypothesis_id.map(Into::into),
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
