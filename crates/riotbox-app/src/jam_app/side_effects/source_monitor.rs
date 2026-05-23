use riotbox_core::{
    action::{Action, ActionCommand, ActionParams},
    session::SessionFile,
};

pub(in crate::jam_app) fn apply_source_monitor_side_effects(
    session: &mut SessionFile,
    action: &Action,
) -> bool {
    let ActionParams::SourceMonitor { mode: Some(mode) } = action.params else {
        return false;
    };
    if action.command != ActionCommand::SourceMonitorSetMode {
        return false;
    }

    session.runtime_state.source_monitor.mode = mode;
    true
}

#[cfg(test)]
mod tests {
    use riotbox_core::{
        TimestampMs,
        action::{
            Action, ActionCommand, ActionParams, ActionStatus, ActionTarget, ActorType,
            Quantization, SourceMonitorMode, UndoPolicy,
        },
        ids::ActionId,
        session::SessionFile,
    };

    use super::apply_source_monitor_side_effects;

    #[test]
    fn source_monitor_side_effect_updates_session_runtime_state() {
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-05-23T08:20:00Z");
        let action = monitor_action(SourceMonitorMode::Blend, 100);

        assert!(apply_source_monitor_side_effects(&mut session, &action));

        assert_eq!(
            session.runtime_state.source_monitor.mode,
            SourceMonitorMode::Blend
        );
    }

    #[test]
    fn source_monitor_side_effect_rejects_missing_mode() {
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-05-23T08:21:00Z");
        let mut action = monitor_action(SourceMonitorMode::Blend, 100);
        action.params = ActionParams::SourceMonitor { mode: None };

        assert!(!apply_source_monitor_side_effects(&mut session, &action));
        assert_eq!(
            session.runtime_state.source_monitor.mode,
            SourceMonitorMode::Source
        );
    }

    fn monitor_action(mode: SourceMonitorMode, requested_at: TimestampMs) -> Action {
        Action {
            id: ActionId(1),
            actor: ActorType::User,
            command: ActionCommand::SourceMonitorSetMode,
            params: ActionParams::SourceMonitor { mode: Some(mode) },
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
