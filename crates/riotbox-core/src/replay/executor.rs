use crate::{
    action::{ActionCommand, ActionParams},
    ids::ActionId,
    replay::ReplayPlanEntry,
    session::SessionFile,
};

const REPLAY_SUPPORTED_ACTION_COMMANDS: &[ActionCommand] = &[
    ActionCommand::TransportPlay,
    ActionCommand::TransportPause,
    ActionCommand::TransportStop,
    ActionCommand::TransportSeek,
    ActionCommand::LockObject,
    ActionCommand::UnlockObject,
    ActionCommand::GhostSetMode,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayExecutionError {
    UnsupportedAction {
        action_id: ActionId,
        command: ActionCommand,
    },
    InvalidParams {
        action_id: ActionId,
        command: ActionCommand,
        expected: &'static str,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayExecutionReport {
    pub applied_action_ids: Vec<ActionId>,
}

#[must_use]
pub const fn replay_supported_action_commands() -> &'static [ActionCommand] {
    REPLAY_SUPPORTED_ACTION_COMMANDS
}

pub fn apply_replay_entry_to_session(
    session: &mut SessionFile,
    entry: &ReplayPlanEntry<'_>,
) -> Result<(), ReplayExecutionError> {
    let action = entry.action;
    match action.command {
        ActionCommand::TransportPlay => {
            session.runtime_state.transport.is_playing = true;
        }
        ActionCommand::TransportPause => {
            session.runtime_state.transport.is_playing = false;
        }
        ActionCommand::TransportStop => {
            session.runtime_state.transport.is_playing = false;
            session.runtime_state.transport.position_beats = 0.0;
        }
        ActionCommand::TransportSeek => {
            let ActionParams::Transport {
                position_beats: Some(position_beats),
            } = action.params
            else {
                return Err(ReplayExecutionError::InvalidParams {
                    action_id: action.id,
                    command: action.command,
                    expected: "ActionParams::Transport { position_beats: Some(_) }",
                });
            };
            session.runtime_state.transport.position_beats = position_beats as f64;
        }
        ActionCommand::LockObject => {
            let ActionParams::Lock { ref object_id } = action.params else {
                return Err(ReplayExecutionError::InvalidParams {
                    action_id: action.id,
                    command: action.command,
                    expected: "ActionParams::Lock { object_id }",
                });
            };
            if !session
                .runtime_state
                .lock_state
                .locked_object_ids
                .contains(object_id)
            {
                session
                    .runtime_state
                    .lock_state
                    .locked_object_ids
                    .push(object_id.clone());
            }
        }
        ActionCommand::UnlockObject => {
            let ActionParams::Lock { ref object_id } = action.params else {
                return Err(ReplayExecutionError::InvalidParams {
                    action_id: action.id,
                    command: action.command,
                    expected: "ActionParams::Lock { object_id }",
                });
            };
            session
                .runtime_state
                .lock_state
                .locked_object_ids
                .retain(|locked_object_id| locked_object_id != object_id);
        }
        ActionCommand::GhostSetMode => {
            let ActionParams::Ghost {
                mode: Some(mode), ..
            } = action.params
            else {
                return Err(ReplayExecutionError::InvalidParams {
                    action_id: action.id,
                    command: action.command,
                    expected: "ActionParams::Ghost { mode: Some(_) }",
                });
            };
            session.ghost_state.mode = mode;
        }
        command => {
            return Err(ReplayExecutionError::UnsupportedAction {
                action_id: action.id,
                command,
            });
        }
    }

    Ok(())
}

pub fn apply_replay_plan_to_session(
    session: &mut SessionFile,
    plan: &[ReplayPlanEntry<'_>],
) -> Result<ReplayExecutionReport, ReplayExecutionError> {
    let mut candidate = session.clone();
    let mut applied_action_ids = Vec::with_capacity(plan.len());

    for entry in plan {
        apply_replay_entry_to_session(&mut candidate, entry)?;
        applied_action_ids.push(entry.action.id);
    }

    *session = candidate;
    Ok(ReplayExecutionReport { applied_action_ids })
}

#[cfg(test)]
mod tests;
