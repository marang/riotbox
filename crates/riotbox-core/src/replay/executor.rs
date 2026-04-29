use crate::{
    action::{ActionCommand, ActionParams},
    ids::ActionId,
    replay::ReplayPlanEntry,
    session::{Mc202PhraseVariantState, SessionFile},
};

const REPLAY_SUPPORTED_ACTION_COMMANDS: &[ActionCommand] = &[
    ActionCommand::TransportPlay,
    ActionCommand::TransportPause,
    ActionCommand::TransportStop,
    ActionCommand::TransportSeek,
    ActionCommand::LockObject,
    ActionCommand::UnlockObject,
    ActionCommand::GhostSetMode,
    ActionCommand::Mc202SetRole,
    ActionCommand::Mc202GenerateFollower,
    ActionCommand::Mc202GenerateAnswer,
    ActionCommand::Mc202GeneratePressure,
    ActionCommand::Mc202GenerateInstigator,
    ActionCommand::Mc202MutatePhrase,
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
        ActionCommand::Mc202SetRole => {
            let role = action
                .target
                .object_id
                .clone()
                .or_else(|| match &action.params {
                    ActionParams::Mutation { target_id, .. } => target_id.clone(),
                    _ => None,
                })
                .ok_or(ReplayExecutionError::InvalidParams {
                    action_id: action.id,
                    command: action.command,
                    expected: "ActionTarget::object_id or ActionParams::Mutation { target_id: Some(_) }",
                })?;

            apply_mc202_role(
                session,
                entry,
                &role,
                mc202_touch_or(action, if role == "leader" { 0.85 } else { 0.65 }),
            );
        }
        ActionCommand::Mc202GenerateFollower => {
            apply_mc202_role(session, entry, "follower", mc202_touch_or(action, 0.78));
        }
        ActionCommand::Mc202GenerateAnswer => {
            apply_mc202_role(session, entry, "answer", mc202_touch_or(action, 0.82));
        }
        ActionCommand::Mc202GeneratePressure => {
            apply_mc202_role(session, entry, "pressure", mc202_touch_or(action, 0.84));
        }
        ActionCommand::Mc202GenerateInstigator => {
            apply_mc202_role(session, entry, "instigator", mc202_touch_or(action, 0.90));
        }
        ActionCommand::Mc202MutatePhrase => {
            let current_role = session
                .runtime_state
                .lane_state
                .mc202
                .role
                .clone()
                .unwrap_or_else(|| "follower".into());
            let variant = match &action.params {
                ActionParams::Mutation {
                    target_id: Some(target_id),
                    ..
                } if target_id == "mutated_drive" => target_id.clone(),
                _ => "mutated_drive".into(),
            };
            let bar_index = entry.commit_record.boundary.bar_index.max(1);
            let phrase_ref = format!("{current_role}-{variant}-bar-{bar_index}");
            let touch = mc202_touch_or(action, 0.88);

            session.runtime_state.lane_state.mc202.role = Some(current_role);
            session.runtime_state.lane_state.mc202.phrase_ref = Some(phrase_ref);
            session.runtime_state.lane_state.mc202.phrase_variant =
                Some(Mc202PhraseVariantState::MutatedDrive);
            session.runtime_state.macro_state.mc202_touch =
                session.runtime_state.macro_state.mc202_touch.max(touch);
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

fn apply_mc202_role(
    session: &mut SessionFile,
    entry: &ReplayPlanEntry<'_>,
    role: &str,
    touch: f32,
) {
    session.runtime_state.lane_state.mc202.role = Some(role.into());
    session.runtime_state.lane_state.mc202.phrase_ref =
        Some(mc202_boundary_phrase_ref(entry, role));
    session.runtime_state.lane_state.mc202.phrase_variant = None;
    session.runtime_state.macro_state.mc202_touch =
        session.runtime_state.macro_state.mc202_touch.max(touch);
}

fn mc202_touch_or(action: &crate::action::Action, fallback: f32) -> f32 {
    match &action.params {
        ActionParams::Mutation { intensity, .. } => intensity.clamp(0.0, 1.0),
        _ => fallback,
    }
}

fn mc202_boundary_phrase_ref(entry: &ReplayPlanEntry<'_>, role: &str) -> String {
    entry.commit_record.boundary.scene_id.as_ref().map_or_else(
        || {
            format!(
                "{role}-phrase-{}",
                entry.commit_record.boundary.phrase_index
            )
        },
        |scene_id| format!("{role}-{scene_id}"),
    )
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
