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
mod tests {
    use super::*;
    use crate::{
        action::{
            ActionCommand, ActionParams, ActionResult, ActionTarget, ActorType, Quantization,
            UndoPolicy,
        },
        ids::SceneId,
        ids::SnapshotId,
        session::ReplayPolicy,
        transport::CommitBoundaryState,
    };

    fn action(id: u64, committed_at: TimestampMs) -> Action {
        Action {
            id: ActionId(id),
            actor: ActorType::User,
            command: ActionCommand::Tr909FillNext,
            params: ActionParams::Empty,
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
        bar_index: u64,
        commit_sequence: u32,
        committed_at: TimestampMs,
    ) -> ActionCommitRecord {
        ActionCommitRecord {
            action_id: ActionId(action_id),
            boundary: CommitBoundaryState {
                kind: CommitBoundary::Bar,
                beat_index,
                bar_index,
                phrase_index: bar_index / 4,
                scene_id: Some(SceneId::from("scene-1")),
            },
            commit_sequence,
            committed_at,
        }
    }

    fn snapshot(action_cursor: usize) -> Snapshot {
        Snapshot {
            snapshot_id: SnapshotId::from("snapshot-1"),
            created_at: "2026-04-29T19:00:00Z".into(),
            label: "test snapshot".into(),
            action_cursor,
        }
    }

    #[test]
    fn replay_plan_orders_actions_by_boundary_and_sequence() {
        let action_log = ActionLog {
            actions: vec![action(1, 200), action(2, 210), action(3, 300)],
            commit_records: vec![
                commit_record(3, 12, 3, 1, 300),
                commit_record(2, 8, 2, 2, 210),
                commit_record(1, 8, 2, 1, 200),
            ],
            replay_policy: ReplayPolicy::DeterministicPreferred,
        };

        let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
        let ordered_ids: Vec<ActionId> = plan.iter().map(|entry| entry.action.id).collect();

        assert_eq!(ordered_ids, vec![ActionId(1), ActionId(2), ActionId(3)]);
        assert_eq!(plan[0].commit_record.commit_sequence, 1);
        assert_eq!(plan[1].commit_record.commit_sequence, 2);
    }

    #[test]
    fn replay_plan_rejects_missing_action() {
        let action_log = ActionLog {
            actions: vec![action(1, 200)],
            commit_records: vec![commit_record(2, 8, 2, 1, 200)],
            replay_policy: ReplayPolicy::DeterministicPreferred,
        };

        let error = build_committed_replay_plan(&action_log).expect_err("plan should fail");

        assert_eq!(
            error,
            ReplayPlanError::MissingAction {
                action_id: ActionId(2)
            }
        );
    }

    #[test]
    fn replay_plan_rejects_commit_timestamp_mismatch() {
        let action_log = ActionLog {
            actions: vec![action(1, 200)],
            commit_records: vec![commit_record(1, 8, 2, 1, 201)],
            replay_policy: ReplayPolicy::DeterministicPreferred,
        };

        let error = build_committed_replay_plan(&action_log).expect_err("plan should fail");

        assert_eq!(
            error,
            ReplayPlanError::CommittedAtMismatch {
                action_id: ActionId(1),
                record_committed_at: 201,
                action_committed_at: 200,
            }
        );
    }

    #[test]
    fn replay_plan_rejects_zero_commit_sequence() {
        let action_log = ActionLog {
            actions: vec![action(1, 200)],
            commit_records: vec![commit_record(1, 8, 2, 0, 200)],
            replay_policy: ReplayPolicy::DeterministicPreferred,
        };

        let error = build_committed_replay_plan(&action_log).expect_err("plan should fail");

        assert_eq!(
            error,
            ReplayPlanError::InvalidCommitSequence {
                action_id: ActionId(1)
            }
        );
    }

    #[test]
    fn replay_plan_rejects_duplicate_action_record() {
        let action_log = ActionLog {
            actions: vec![action(1, 200)],
            commit_records: vec![
                commit_record(1, 8, 2, 1, 200),
                commit_record(1, 12, 3, 1, 200),
            ],
            replay_policy: ReplayPolicy::DeterministicPreferred,
        };

        let error = build_committed_replay_plan(&action_log).expect_err("plan should fail");

        assert_eq!(
            error,
            ReplayPlanError::DuplicateActionRecord {
                action_id: ActionId(1)
            }
        );
    }

    #[test]
    fn replay_plan_rejects_duplicate_sequence_within_boundary() {
        let duplicated_boundary = CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 8,
            bar_index: 2,
            phrase_index: 0,
            scene_id: Some(SceneId::from("scene-1")),
        };
        let action_log = ActionLog {
            actions: vec![action(1, 200), action(2, 210)],
            commit_records: vec![
                ActionCommitRecord {
                    action_id: ActionId(1),
                    boundary: duplicated_boundary.clone(),
                    commit_sequence: 1,
                    committed_at: 200,
                },
                ActionCommitRecord {
                    action_id: ActionId(2),
                    boundary: duplicated_boundary.clone(),
                    commit_sequence: 1,
                    committed_at: 210,
                },
            ],
            replay_policy: ReplayPolicy::DeterministicPreferred,
        };

        let error = build_committed_replay_plan(&action_log).expect_err("plan should fail");

        assert_eq!(
            error,
            ReplayPlanError::DuplicateCommitSequence {
                boundary: duplicated_boundary,
                commit_sequence: 1,
            }
        );
    }

    #[test]
    fn snapshot_comparison_keeps_origin_and_selects_suffix_after_cursor() {
        let action_log = ActionLog {
            actions: vec![action(1, 200), action(2, 210), action(3, 300)],
            commit_records: vec![
                commit_record(3, 12, 3, 1, 300),
                commit_record(2, 8, 2, 2, 210),
                commit_record(1, 8, 2, 1, 200),
            ],
            replay_policy: ReplayPolicy::DeterministicPreferred,
        };

        let comparison =
            build_snapshot_replay_plan_comparison(&action_log, &snapshot(2)).expect("valid plan");
        let origin_ids: Vec<ActionId> = comparison
            .origin
            .iter()
            .map(|entry| entry.action.id)
            .collect();
        let suffix_ids: Vec<ActionId> = comparison
            .snapshot_suffix
            .iter()
            .map(|entry| entry.action.id)
            .collect();

        assert_eq!(origin_ids, vec![ActionId(1), ActionId(2), ActionId(3)]);
        assert_eq!(suffix_ids, vec![ActionId(3)]);
        assert_eq!(comparison.snapshot_action_cursor, 2);
    }

    #[test]
    fn snapshot_comparison_with_zero_cursor_replays_full_origin() {
        let action_log = ActionLog {
            actions: vec![action(1, 200), action(2, 210)],
            commit_records: vec![
                commit_record(2, 8, 2, 2, 210),
                commit_record(1, 8, 2, 1, 200),
            ],
            replay_policy: ReplayPolicy::DeterministicPreferred,
        };

        let comparison =
            build_snapshot_replay_plan_comparison(&action_log, &snapshot(0)).expect("valid plan");
        let suffix_ids: Vec<ActionId> = comparison
            .snapshot_suffix
            .iter()
            .map(|entry| entry.action.id)
            .collect();

        assert_eq!(suffix_ids, vec![ActionId(1), ActionId(2)]);
    }

    #[test]
    fn snapshot_comparison_at_log_end_has_empty_suffix() {
        let action_log = ActionLog {
            actions: vec![action(1, 200), action(2, 210)],
            commit_records: vec![
                commit_record(2, 8, 2, 2, 210),
                commit_record(1, 8, 2, 1, 200),
            ],
            replay_policy: ReplayPolicy::DeterministicPreferred,
        };

        let comparison =
            build_snapshot_replay_plan_comparison(&action_log, &snapshot(2)).expect("valid plan");

        assert!(comparison.snapshot_suffix.is_empty());
    }

    #[test]
    fn snapshot_comparison_rejects_cursor_beyond_action_log() {
        let action_log = ActionLog {
            actions: vec![action(1, 200)],
            commit_records: vec![commit_record(1, 8, 2, 1, 200)],
            replay_policy: ReplayPolicy::DeterministicPreferred,
        };

        let error = build_snapshot_replay_plan_comparison(&action_log, &snapshot(2))
            .expect_err("plan should fail");

        assert_eq!(
            error,
            ReplayPlanError::SnapshotCursorOutOfBounds {
                action_cursor: 2,
                action_count: 1,
            }
        );
    }
}
