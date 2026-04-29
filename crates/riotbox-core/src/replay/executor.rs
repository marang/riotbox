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
mod tests {
    use super::*;
    use crate::{
        TimestampMs,
        action::{
            Action, ActionParams, ActionResult, ActionStatus, ActionTarget, ActorType,
            CommitBoundary, GhostMode, Quantization, UndoPolicy,
        },
        replay::build_committed_replay_plan,
        session::{ActionCommitRecord, ActionLog, ReplayPolicy},
        transport::CommitBoundaryState,
    };

    fn action(
        id: u64,
        command: ActionCommand,
        params: ActionParams,
        committed_at: TimestampMs,
    ) -> Action {
        Action {
            id: ActionId(id),
            actor: ActorType::User,
            command,
            params,
            target: ActionTarget::default(),
            requested_at: committed_at - 10,
            quantization: Quantization::NextBar,
            status: ActionStatus::Committed,
            committed_at: Some(committed_at),
            result: Some(ActionResult {
                accepted: true,
                summary: "committed".into(),
            }),
            undo_policy: UndoPolicy::Undoable,
            explanation: None,
        }
    }

    fn commit_record(
        action_id: u64,
        beat_index: u64,
        commit_sequence: u32,
        committed_at: TimestampMs,
    ) -> ActionCommitRecord {
        ActionCommitRecord {
            action_id: ActionId(action_id),
            boundary: CommitBoundaryState {
                kind: CommitBoundary::Bar,
                beat_index,
                bar_index: beat_index / 4,
                phrase_index: beat_index / 16,
                scene_id: None,
            },
            commit_sequence,
            committed_at,
        }
    }

    fn action_log(actions: Vec<Action>) -> ActionLog {
        let commit_records = actions
            .iter()
            .enumerate()
            .map(|(index, action)| {
                commit_record(
                    action.id.0,
                    ((index + 1) * 4) as u64,
                    1,
                    action.committed_at.expect("test action committed"),
                )
            })
            .collect();

        ActionLog {
            actions,
            commit_records,
            replay_policy: ReplayPolicy::DeterministicPreferred,
        }
    }

    #[test]
    fn plan_executor_applies_supported_structural_actions_in_commit_order() {
        let action_log = action_log(vec![
            action(1, ActionCommand::TransportPlay, ActionParams::Empty, 100),
            action(
                2,
                ActionCommand::TransportSeek,
                ActionParams::Transport {
                    position_beats: Some(16),
                },
                200,
            ),
            action(
                3,
                ActionCommand::LockObject,
                ActionParams::Lock {
                    object_id: "pad-a1".into(),
                },
                300,
            ),
            action(
                4,
                ActionCommand::GhostSetMode,
                ActionParams::Ghost {
                    mode: Some(GhostMode::Assist),
                    proposal_id: None,
                },
                400,
            ),
            action(5, ActionCommand::TransportPause, ActionParams::Empty, 500),
        ]);
        let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T20:00:00Z");

        let report =
            apply_replay_plan_to_session(&mut session, &plan).expect("supported replay plan");

        assert_eq!(
            report.applied_action_ids,
            vec![
                ActionId(1),
                ActionId(2),
                ActionId(3),
                ActionId(4),
                ActionId(5)
            ]
        );
        assert!(!session.runtime_state.transport.is_playing);
        assert_eq!(session.runtime_state.transport.position_beats, 16.0);
        assert_eq!(
            session.runtime_state.lock_state.locked_object_ids,
            vec!["pad-a1".to_string()]
        );
        assert_eq!(session.ghost_state.mode, GhostMode::Assist);
    }

    #[test]
    fn plan_executor_handles_stop_unlock_and_duplicate_lock_deterministically() {
        let action_log = action_log(vec![
            action(
                1,
                ActionCommand::TransportSeek,
                ActionParams::Transport {
                    position_beats: Some(32),
                },
                100,
            ),
            action(2, ActionCommand::TransportPlay, ActionParams::Empty, 200),
            action(
                3,
                ActionCommand::LockObject,
                ActionParams::Lock {
                    object_id: "scene-drop".into(),
                },
                300,
            ),
            action(
                4,
                ActionCommand::LockObject,
                ActionParams::Lock {
                    object_id: "scene-drop".into(),
                },
                400,
            ),
            action(
                5,
                ActionCommand::UnlockObject,
                ActionParams::Lock {
                    object_id: "scene-drop".into(),
                },
                500,
            ),
            action(6, ActionCommand::TransportStop, ActionParams::Empty, 600),
        ]);
        let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T20:00:00Z");

        apply_replay_plan_to_session(&mut session, &plan).expect("supported replay plan");

        assert!(!session.runtime_state.transport.is_playing);
        assert_eq!(session.runtime_state.transport.position_beats, 0.0);
        assert!(
            session
                .runtime_state
                .lock_state
                .locked_object_ids
                .is_empty()
        );
    }

    #[test]
    fn plan_executor_rejects_unsupported_actions_without_mutating_session() {
        let action_log = action_log(vec![
            action(1, ActionCommand::TransportPlay, ActionParams::Empty, 100),
            action(2, ActionCommand::Tr909FillNext, ActionParams::Empty, 200),
        ]);
        let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T20:00:00Z");
        let original_session = session.clone();

        let error = apply_replay_plan_to_session(&mut session, &plan).expect_err("unsupported");

        assert_eq!(
            error,
            ReplayExecutionError::UnsupportedAction {
                action_id: ActionId(2),
                command: ActionCommand::Tr909FillNext
            }
        );
        assert_eq!(session, original_session);
    }

    #[test]
    fn plan_executor_rejects_invalid_params_without_mutating_session() {
        let action_log = action_log(vec![action(
            1,
            ActionCommand::GhostSetMode,
            ActionParams::Ghost {
                mode: None,
                proposal_id: None,
            },
            100,
        )]);
        let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
        let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T20:00:00Z");
        let original_session = session.clone();

        let error = apply_replay_plan_to_session(&mut session, &plan).expect_err("invalid params");

        assert_eq!(
            error,
            ReplayExecutionError::InvalidParams {
                action_id: ActionId(1),
                command: ActionCommand::GhostSetMode,
                expected: "ActionParams::Ghost { mode: Some(_) }"
            }
        );
        assert_eq!(session, original_session);
    }

    #[test]
    fn supported_action_list_documents_the_initial_executor_subset() {
        assert_eq!(
            replay_supported_action_commands(),
            &[
                ActionCommand::TransportPlay,
                ActionCommand::TransportPause,
                ActionCommand::TransportStop,
                ActionCommand::TransportSeek,
                ActionCommand::LockObject,
                ActionCommand::UnlockObject,
                ActionCommand::GhostSetMode,
            ]
        );
    }
}
