use super::*;
use crate::{
    TimestampMs,
    action::{
        Action, ActionParams, ActionResult, ActionStatus, ActionTarget, ActorType, CommitBoundary,
        Quantization, UndoPolicy,
    },
    ids::SnapshotId,
    session::{ActionCommitRecord, ActionLog, ReplayPolicy, Snapshot},
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

fn targeted_action(
    id: u64,
    command: ActionCommand,
    params: ActionParams,
    target: ActionTarget,
    committed_at: TimestampMs,
) -> Action {
    Action {
        target,
        ..action(id, command, params, committed_at)
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

fn snapshot(snapshot_id: &str, action_cursor: usize) -> Snapshot {
    Snapshot {
        snapshot_id: SnapshotId::from(snapshot_id),
        created_at: "2026-04-29T20:30:00Z".into(),
        label: "test snapshot".into(),
        action_cursor,
    }
}

mod mc202;
mod structural;
mod tr909;
