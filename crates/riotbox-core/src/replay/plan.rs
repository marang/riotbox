use std::cmp::Ordering;

use crate::{
    TimestampMs,
    action::{Action, ActionStatus, CommitBoundary},
    ids::ActionId,
    session::{ActionCommitRecord, ActionLog, Snapshot},
    transport::CommitBoundaryState,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayPlanError {
    MissingAction {
        action_id: ActionId,
    },
    NonCommittedAction {
        action_id: ActionId,
        status: ActionStatus,
    },
    MissingCommittedAt {
        action_id: ActionId,
    },
    InvalidCommitSequence {
        action_id: ActionId,
    },
    DuplicateActionRecord {
        action_id: ActionId,
    },
    DuplicateCommitSequence {
        boundary: CommitBoundaryState,
        commit_sequence: u32,
    },
    SnapshotCursorOutOfBounds {
        action_cursor: usize,
        action_count: usize,
    },
    ReplayTargetCursorOutOfBounds {
        target_action_cursor: usize,
        action_count: usize,
    },
    CommittedAtMismatch {
        action_id: ActionId,
        record_committed_at: TimestampMs,
        action_committed_at: TimestampMs,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReplayPlanEntry<'a> {
    pub action: &'a Action,
    pub commit_record: &'a ActionCommitRecord,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotReplayPlanComparison<'a> {
    pub origin: Vec<ReplayPlanEntry<'a>>,
    pub snapshot_suffix: Vec<ReplayPlanEntry<'a>>,
    pub snapshot_action_cursor: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReplayTargetPlan<'a> {
    pub origin: Vec<ReplayPlanEntry<'a>>,
    pub suffix: Vec<ReplayPlanEntry<'a>>,
    pub anchor: Option<&'a Snapshot>,
    pub target_action_cursor: usize,
}

pub fn build_committed_replay_plan(
    action_log: &ActionLog,
) -> Result<Vec<ReplayPlanEntry<'_>>, ReplayPlanError> {
    let mut entries = Vec::with_capacity(action_log.commit_records.len());
    let mut seen_action_ids = Vec::with_capacity(action_log.commit_records.len());
    let mut seen_boundary_sequences = Vec::with_capacity(action_log.commit_records.len());

    for commit_record in &action_log.commit_records {
        if commit_record.commit_sequence == 0 {
            return Err(ReplayPlanError::InvalidCommitSequence {
                action_id: commit_record.action_id,
            });
        }

        if seen_action_ids.contains(&commit_record.action_id) {
            return Err(ReplayPlanError::DuplicateActionRecord {
                action_id: commit_record.action_id,
            });
        }

        if seen_boundary_sequences.iter().any(|(boundary, sequence)| {
            boundary == &commit_record.boundary && sequence == &commit_record.commit_sequence
        }) {
            return Err(ReplayPlanError::DuplicateCommitSequence {
                boundary: commit_record.boundary.clone(),
                commit_sequence: commit_record.commit_sequence,
            });
        }

        seen_action_ids.push(commit_record.action_id);
        seen_boundary_sequences.push((
            commit_record.boundary.clone(),
            commit_record.commit_sequence,
        ));

        let Some(action) = action_log
            .actions
            .iter()
            .find(|action| action.id == commit_record.action_id)
        else {
            return Err(ReplayPlanError::MissingAction {
                action_id: commit_record.action_id,
            });
        };

        if action.status != ActionStatus::Committed {
            return Err(ReplayPlanError::NonCommittedAction {
                action_id: action.id,
                status: action.status,
            });
        }

        let Some(action_committed_at) = action.committed_at else {
            return Err(ReplayPlanError::MissingCommittedAt {
                action_id: action.id,
            });
        };

        if commit_record.committed_at != action_committed_at {
            return Err(ReplayPlanError::CommittedAtMismatch {
                action_id: action.id,
                record_committed_at: commit_record.committed_at,
                action_committed_at,
            });
        }

        entries.push(ReplayPlanEntry {
            action,
            commit_record,
        });
    }

    entries.sort_by(compare_replay_entries);
    Ok(entries)
}

pub fn build_snapshot_replay_plan_comparison<'a>(
    action_log: &'a ActionLog,
    snapshot: &Snapshot,
) -> Result<SnapshotReplayPlanComparison<'a>, ReplayPlanError> {
    if snapshot.action_cursor > action_log.actions.len() {
        return Err(ReplayPlanError::SnapshotCursorOutOfBounds {
            action_cursor: snapshot.action_cursor,
            action_count: action_log.actions.len(),
        });
    }

    let origin = build_committed_replay_plan(action_log)?;
    let applied_action_ids: Vec<ActionId> = action_log
        .actions
        .iter()
        .take(snapshot.action_cursor)
        .map(|action| action.id)
        .collect();
    let snapshot_suffix = origin
        .iter()
        .filter(|entry| !applied_action_ids.contains(&entry.action.id))
        .cloned()
        .collect();

    Ok(SnapshotReplayPlanComparison {
        origin,
        snapshot_suffix,
        snapshot_action_cursor: snapshot.action_cursor,
    })
}

pub fn select_replay_snapshot_anchor(
    snapshots: &[Snapshot],
    target_action_cursor: usize,
    action_count: usize,
) -> Result<Option<&Snapshot>, ReplayPlanError> {
    if target_action_cursor > action_count {
        return Err(ReplayPlanError::ReplayTargetCursorOutOfBounds {
            target_action_cursor,
            action_count,
        });
    }

    let mut selected: Option<(usize, &Snapshot)> = None;
    for (index, snapshot) in snapshots.iter().enumerate() {
        if snapshot.action_cursor > action_count {
            return Err(ReplayPlanError::SnapshotCursorOutOfBounds {
                action_cursor: snapshot.action_cursor,
                action_count,
            });
        }

        if snapshot.action_cursor > target_action_cursor {
            continue;
        }

        let should_select = match selected {
            Some((selected_index, selected_snapshot)) => {
                snapshot.action_cursor > selected_snapshot.action_cursor
                    || (snapshot.action_cursor == selected_snapshot.action_cursor
                        && index > selected_index)
            }
            None => true,
        };
        if should_select {
            selected = Some((index, snapshot));
        }
    }

    Ok(selected.map(|(_, snapshot)| snapshot))
}

pub fn build_replay_target_plan<'a>(
    action_log: &'a ActionLog,
    snapshots: &'a [Snapshot],
    target_action_cursor: usize,
) -> Result<ReplayTargetPlan<'a>, ReplayPlanError> {
    let anchor =
        select_replay_snapshot_anchor(snapshots, target_action_cursor, action_log.actions.len())?;
    let origin = build_committed_replay_plan(action_log)?;
    let anchor_cursor = anchor.map_or(0, |snapshot| snapshot.action_cursor);
    let skipped_action_ids: Vec<ActionId> = action_log
        .actions
        .iter()
        .take(anchor_cursor)
        .map(|action| action.id)
        .collect();
    let target_action_ids: Vec<ActionId> = action_log
        .actions
        .iter()
        .take(target_action_cursor)
        .map(|action| action.id)
        .collect();
    let suffix = origin
        .iter()
        .filter(|entry| target_action_ids.contains(&entry.action.id))
        .filter(|entry| !skipped_action_ids.contains(&entry.action.id))
        .cloned()
        .collect();

    Ok(ReplayTargetPlan {
        origin,
        suffix,
        anchor,
        target_action_cursor,
    })
}

fn compare_replay_entries(left: &ReplayPlanEntry<'_>, right: &ReplayPlanEntry<'_>) -> Ordering {
    compare_commit_records(left.commit_record, right.commit_record)
        .then_with(|| left.action.id.cmp(&right.action.id))
}

fn compare_commit_records(left: &ActionCommitRecord, right: &ActionCommitRecord) -> Ordering {
    left.boundary
        .beat_index
        .cmp(&right.boundary.beat_index)
        .then_with(|| left.boundary.bar_index.cmp(&right.boundary.bar_index))
        .then_with(|| left.boundary.phrase_index.cmp(&right.boundary.phrase_index))
        .then_with(|| boundary_rank(left.boundary.kind).cmp(&boundary_rank(right.boundary.kind)))
        .then_with(|| left.boundary.scene_id.cmp(&right.boundary.scene_id))
        .then_with(|| left.commit_sequence.cmp(&right.commit_sequence))
        .then_with(|| left.action_id.cmp(&right.action_id))
}

const fn boundary_rank(kind: CommitBoundary) -> u8 {
    match kind {
        CommitBoundary::Immediate => 0,
        CommitBoundary::Beat => 1,
        CommitBoundary::HalfBar => 2,
        CommitBoundary::Bar => 3,
        CommitBoundary::Phrase => 4,
        CommitBoundary::Scene => 5,
    }
}

#[cfg(test)]
mod tests;
