use crate::{
    action::{Action, ActionCommand, ActionParams},
    session::{SessionFile, SourceTimingGridConfirmationState},
};

use super::ReplayExecutionError;

pub(super) fn apply_source_timing_replay_action(
    session: &mut SessionFile,
    action: &Action,
) -> Result<(), ReplayExecutionError> {
    let ActionParams::SourceTimingGrid {
        source_id: Some(ref source_id),
        ref hypothesis_id,
    } = action.params
    else {
        return Err(ReplayExecutionError::InvalidParams {
            action_id: action.id,
            command: action.command,
            expected: "ActionParams::SourceTimingGrid { source_id: Some(_) }",
        });
    };

    match action.command {
        ActionCommand::SourceTimingConfirmGrid => {
            session.runtime_state.source_timing.confirmed_grid =
                Some(SourceTimingGridConfirmationState {
                    source_id: source_id.clone(),
                    hypothesis_id: hypothesis_id.clone(),
                    confirmed_by_action: action.id,
                    confirmed_at: action.committed_at.unwrap_or(action.requested_at),
                });
        }
        ActionCommand::SourceTimingRevertGrid
            if session
                .runtime_state
                .source_timing
                .confirmed_grid
                .as_ref()
                .is_some_and(|confirmed| {
                    confirmed.source_id == *source_id
                        && confirmed.hypothesis_id.as_deref() == hypothesis_id.as_deref()
                }) =>
        {
            session.runtime_state.source_timing.confirmed_grid = None;
            if session
                .runtime_state
                .lane_state
                .mc202
                .source_phrase_plan
                .as_ref()
                .is_some_and(|plan| plan.source_id == *source_id)
            {
                session.runtime_state.lane_state.mc202.source_phrase_plan = None;
            }
        }
        ActionCommand::SourceTimingRevertGrid => {}
        _ => {}
    }
    Ok(())
}
