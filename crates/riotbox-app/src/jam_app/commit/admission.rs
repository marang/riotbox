//! Prospective legacy metadata goes to the one Core planner before mutation.
use riotbox_core::{
    TimestampMs, action::ActionDraft, queue::ActionQueue, replay::build_committed_replay_plan,
    session::ActionLog, transport::CommitBoundaryState,
};

use crate::jam_app::JamAppError;

use super::record_committed_action_in_log;

pub(super) fn validate_legacy_history(log: &ActionLog) -> Result<(), JamAppError> {
    if !log.replay_policy.uses_committed_append_order() {
        build_committed_replay_plan(log).map_err(|error| JamAppError::ReplayAdmission {
            policy: log.replay_policy,
            error,
        })?;
    }
    Ok(())
}

pub(super) fn preflight_ready_legacy_commits(
    queue: &ActionQueue,
    history: &ActionLog,
    boundary: &CommitBoundaryState,
    committed_at: TimestampMs,
) -> Result<(), JamAppError> {
    if history.replay_policy.uses_committed_append_order()
        || queue.ready_action_count_for_transport(boundary.kind) == 0
    {
        return Ok(());
    }
    validate_staged_ready(&mut queue.clone(), history, boundary, committed_at)
}

pub(super) fn preflight_legacy_draft(
    queue: &ActionQueue,
    history: &ActionLog,
    draft: &ActionDraft,
    boundary: &CommitBoundaryState,
    requested_at: TimestampMs,
) -> Result<(), JamAppError> {
    if history.replay_policy.uses_committed_append_order() {
        return Ok(());
    }
    let mut prospective_queue = queue.clone();
    prospective_queue.enqueue(draft.clone(), requested_at)?;
    validate_staged_ready(&mut prospective_queue, history, boundary, requested_at)
}

fn validate_staged_ready(
    queue: &mut ActionQueue,
    history: &ActionLog,
    boundary: &CommitBoundaryState,
    committed_at: TimestampMs,
) -> Result<(), JamAppError> {
    let committed = queue.commit_ready_for_transport(history, boundary.clone(), committed_at)?;
    let mut prospective_history = history.clone();
    for committed_ref in committed {
        let action = queue
            .history_action(committed_ref.action_id)
            .ok_or_else(|| {
                JamAppError::InvalidSession("prospective committed queue action is missing".into())
            })?;
        record_committed_action_in_log(
            &mut prospective_history,
            action.clone(),
            &committed_ref,
            committed_at,
        );
    }
    validate_legacy_history(&prospective_history)
}
