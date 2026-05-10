use riotbox_core::{
    action::{Action, ActionCommand, ActionParams, ActionResult},
    session::SessionFile,
};

pub(in crate::jam_app) fn apply_ghost_side_effects(
    session: &mut SessionFile,
    action: &Action,
) -> bool {
    let ActionParams::Ghost { mode, proposal_id } = &action.params else {
        return false;
    };

    let (applied, summary) = match action.command {
        ActionCommand::GhostSetMode => {
            let Some(mode) = mode else {
                return update_logged_ghost_result(
                    session,
                    action,
                    false,
                    "ghost mode change ignored: missing mode",
                );
            };
            session.ghost_state.mode = *mode;
            (true, format!("ghost mode set to {mode}"))
        }
        ActionCommand::GhostAcceptSuggestion => {
            let Some(proposal_id) = proposal_id.as_deref() else {
                return update_logged_ghost_result(
                    session,
                    action,
                    false,
                    "ghost accept ignored: missing proposal id",
                );
            };
            if session.ghost_state.accept_suggestion(proposal_id) {
                (true, format!("ghost suggestion accepted: {proposal_id}"))
            } else {
                (
                    false,
                    format!(
                        "ghost accept ignored: {proposal_id} requires assist mode or known proposal"
                    ),
                )
            }
        }
        ActionCommand::GhostRejectSuggestion => {
            let Some(proposal_id) = proposal_id.as_deref() else {
                return update_logged_ghost_result(
                    session,
                    action,
                    false,
                    "ghost reject ignored: missing proposal id",
                );
            };
            if session.ghost_state.reject_suggestion(proposal_id) {
                (true, format!("ghost suggestion rejected: {proposal_id}"))
            } else {
                (
                    false,
                    format!("ghost reject ignored: unknown proposal {proposal_id}"),
                )
            }
        }
        _ => return false,
    };

    update_logged_ghost_result(session, action, applied, summary)
}

fn update_logged_ghost_result(
    session: &mut SessionFile,
    action: &Action,
    accepted: bool,
    summary: impl Into<String>,
) -> bool {
    if let Some(logged_action) = session
        .action_log
        .actions
        .iter_mut()
        .rev()
        .find(|logged_action| logged_action.id == action.id)
    {
        logged_action.result = Some(ActionResult {
            accepted,
            summary: summary.into(),
        });
    }

    accepted
}

#[cfg(test)]
mod tests {
    use riotbox_core::{
        TimestampMs,
        action::{
            ActionStatus, ActionTarget, ActorType, GhostMode, Quantization, TargetScope, UndoPolicy,
        },
        ids::ActionId,
        session::{GhostSuggestionRecord, SessionFile},
    };

    use super::*;

    #[test]
    fn assist_accept_action_marks_matching_suggestion_only() {
        let mut session = ghost_session(GhostMode::Assist);
        let action = ghost_decision_action(
            ActionCommand::GhostAcceptSuggestion,
            Some("ghost-proposal-1"),
            42,
        );
        session.action_log.actions.push(action.clone());

        assert!(apply_ghost_side_effects(&mut session, &action));

        let record = &session.ghost_state.suggestion_history[0];
        assert!(record.accepted);
        assert!(!record.rejected);
        assert_eq!(record.status().label(), "accepted");
        assert_eq!(
            session.action_log.actions[0]
                .result
                .as_ref()
                .expect("ghost result")
                .summary,
            "ghost suggestion accepted: ghost-proposal-1"
        );
    }

    #[test]
    fn watch_accept_action_is_read_only() {
        let mut session = ghost_session(GhostMode::Watch);
        let action = ghost_decision_action(
            ActionCommand::GhostAcceptSuggestion,
            Some("ghost-proposal-1"),
            42,
        );
        session.action_log.actions.push(action.clone());

        assert!(!apply_ghost_side_effects(&mut session, &action));

        let record = &session.ghost_state.suggestion_history[0];
        assert!(!record.accepted);
        assert!(!record.rejected);
        assert_eq!(record.status().label(), "suggested");
        let result = session.action_log.actions[0]
            .result
            .as_ref()
            .expect("ghost result");
        assert!(!result.accepted);
        assert_eq!(
            result.summary,
            "ghost accept ignored: ghost-proposal-1 requires assist mode or known proposal"
        );
    }

    #[test]
    fn reject_action_marks_suggestion_without_accepting_it() {
        let mut session = ghost_session(GhostMode::Watch);
        let action = ghost_decision_action(
            ActionCommand::GhostRejectSuggestion,
            Some("ghost-proposal-1"),
            42,
        );
        session.action_log.actions.push(action.clone());

        assert!(apply_ghost_side_effects(&mut session, &action));

        let record = &session.ghost_state.suggestion_history[0];
        assert!(!record.accepted);
        assert!(record.rejected);
        assert_eq!(record.status().label(), "rejected");
        assert_eq!(
            session.action_log.actions[0]
                .result
                .as_ref()
                .expect("ghost result")
                .summary,
            "ghost suggestion rejected: ghost-proposal-1"
        );
    }

    fn ghost_session(mode: GhostMode) -> SessionFile {
        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-29T16:50:00Z");
        session.ghost_state.mode = mode;
        session
            .ghost_state
            .suggestion_history
            .push(GhostSuggestionRecord {
                proposal_id: "ghost-proposal-1".into(),
                summary: "fill next bar".into(),
                accepted: false,
                rejected: false,
            });
        session
    }

    fn ghost_decision_action(
        command: ActionCommand,
        proposal_id: Option<&str>,
        requested_at: TimestampMs,
    ) -> Action {
        Action {
            id: ActionId(1),
            actor: ActorType::User,
            command,
            params: ActionParams::Ghost {
                mode: None,
                proposal_id: proposal_id.map(str::to_owned),
            },
            target: ActionTarget {
                scope: Some(TargetScope::Ghost),
                ..Default::default()
            },
            requested_at,
            quantization: Quantization::Immediate,
            status: ActionStatus::Committed,
            committed_at: Some(requested_at),
            result: None,
            undo_policy: UndoPolicy::NotUndoable {
                reason: "ghost decision markers only update suggestion state".into(),
            },
            explanation: None,
        }
    }
}
