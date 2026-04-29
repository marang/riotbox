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
    snapshot_with_id("snapshot-1", action_cursor)
}

fn snapshot_with_id(snapshot_id: &str, action_cursor: usize) -> Snapshot {
    Snapshot {
        snapshot_id: SnapshotId::from(snapshot_id),
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

#[test]
fn snapshot_anchor_selects_exact_target_cursor() {
    let snapshots = vec![snapshot_with_id("snap-1", 1), snapshot_with_id("snap-3", 3)];

    let selected = select_replay_snapshot_anchor(&snapshots, 3, 4).expect("valid snapshot anchors");

    assert_eq!(
        selected.map(|snapshot| snapshot.snapshot_id.as_str()),
        Some("snap-3")
    );
}

#[test]
fn snapshot_anchor_selects_nearest_prior_cursor() {
    let snapshots = vec![
        snapshot_with_id("snap-1", 1),
        snapshot_with_id("snap-2", 2),
        snapshot_with_id("snap-5", 5),
    ];

    let selected = select_replay_snapshot_anchor(&snapshots, 4, 5).expect("valid snapshot anchors");

    assert_eq!(
        selected.map(|snapshot| snapshot.snapshot_id.as_str()),
        Some("snap-2")
    );
}

#[test]
fn snapshot_anchor_prefers_latest_snapshot_for_same_cursor() {
    let snapshots = vec![
        snapshot_with_id("snap-2-a", 2),
        snapshot_with_id("snap-2-b", 2),
    ];

    let selected = select_replay_snapshot_anchor(&snapshots, 2, 2).expect("valid snapshot anchors");

    assert_eq!(
        selected.map(|snapshot| snapshot.snapshot_id.as_str()),
        Some("snap-2-b")
    );
}

#[test]
fn snapshot_anchor_returns_none_without_prior_snapshot() {
    let snapshots = vec![snapshot_with_id("snap-2", 2)];

    let selected = select_replay_snapshot_anchor(&snapshots, 1, 2).expect("valid snapshot anchors");

    assert!(selected.is_none());
}

#[test]
fn snapshot_anchor_rejects_snapshot_cursor_beyond_action_log() {
    let snapshots = vec![snapshot_with_id("bad-snap", 3)];

    let error = select_replay_snapshot_anchor(&snapshots, 1, 2).expect_err("anchor should fail");

    assert_eq!(
        error,
        ReplayPlanError::SnapshotCursorOutOfBounds {
            action_cursor: 3,
            action_count: 2,
        }
    );
}

#[test]
fn snapshot_anchor_rejects_target_cursor_beyond_action_log() {
    let snapshots = vec![snapshot_with_id("snap-1", 1)];

    let error = select_replay_snapshot_anchor(&snapshots, 3, 2).expect_err("anchor should fail");

    assert_eq!(
        error,
        ReplayPlanError::ReplayTargetCursorOutOfBounds {
            target_action_cursor: 3,
            action_count: 2,
        }
    );
}

#[test]
fn replay_target_plan_without_anchor_replays_from_origin_to_target() {
    let action_log = ActionLog {
        actions: vec![action(1, 200), action(2, 210), action(3, 300)],
        commit_records: vec![
            commit_record(3, 12, 3, 1, 300),
            commit_record(2, 8, 2, 2, 210),
            commit_record(1, 8, 2, 1, 200),
        ],
        replay_policy: ReplayPolicy::DeterministicPreferred,
    };

    let target_plan = build_replay_target_plan(&action_log, &[], 2).expect("valid target plan");
    let suffix_ids: Vec<ActionId> = target_plan
        .suffix
        .iter()
        .map(|entry| entry.action.id)
        .collect();

    assert!(target_plan.anchor.is_none());
    assert_eq!(target_plan.origin.len(), 3);
    assert_eq!(suffix_ids, vec![ActionId(1), ActionId(2)]);
}

#[test]
fn replay_target_plan_with_prior_anchor_replays_suffix_to_target() {
    let action_log = ActionLog {
        actions: vec![action(1, 200), action(2, 210), action(3, 300)],
        commit_records: vec![
            commit_record(3, 12, 3, 1, 300),
            commit_record(2, 8, 2, 2, 210),
            commit_record(1, 8, 2, 1, 200),
        ],
        replay_policy: ReplayPolicy::DeterministicPreferred,
    };
    let snapshots = vec![snapshot_with_id("snap-1", 1)];

    let target_plan =
        build_replay_target_plan(&action_log, &snapshots, 3).expect("valid target plan");
    let suffix_ids: Vec<ActionId> = target_plan
        .suffix
        .iter()
        .map(|entry| entry.action.id)
        .collect();

    assert_eq!(
        target_plan
            .anchor
            .map(|snapshot| snapshot.snapshot_id.as_str()),
        Some("snap-1")
    );
    assert_eq!(suffix_ids, vec![ActionId(2), ActionId(3)]);
}

#[test]
fn replay_target_plan_with_exact_anchor_has_empty_suffix() {
    let action_log = ActionLog {
        actions: vec![action(1, 200), action(2, 210)],
        commit_records: vec![
            commit_record(2, 8, 2, 2, 210),
            commit_record(1, 8, 2, 1, 200),
        ],
        replay_policy: ReplayPolicy::DeterministicPreferred,
    };
    let snapshots = vec![snapshot_with_id("snap-2", 2)];

    let target_plan =
        build_replay_target_plan(&action_log, &snapshots, 2).expect("valid target plan");

    assert_eq!(
        target_plan
            .anchor
            .map(|snapshot| snapshot.snapshot_id.as_str()),
        Some("snap-2")
    );
    assert!(target_plan.suffix.is_empty());
}

#[test]
fn replay_target_plan_rejects_target_cursor_beyond_action_log() {
    let action_log = ActionLog {
        actions: vec![action(1, 200)],
        commit_records: vec![commit_record(1, 8, 2, 1, 200)],
        replay_policy: ReplayPolicy::DeterministicPreferred,
    };

    let error = build_replay_target_plan(&action_log, &[], 2).expect_err("target plan should fail");

    assert_eq!(
        error,
        ReplayPlanError::ReplayTargetCursorOutOfBounds {
            target_action_cursor: 2,
            action_count: 1,
        }
    );
}

#[test]
fn replay_target_plan_rejects_snapshot_cursor_beyond_action_log() {
    let action_log = ActionLog {
        actions: vec![action(1, 200)],
        commit_records: vec![commit_record(1, 8, 2, 1, 200)],
        replay_policy: ReplayPolicy::DeterministicPreferred,
    };
    let snapshots = vec![snapshot_with_id("bad-snap", 2)];

    let error =
        build_replay_target_plan(&action_log, &snapshots, 1).expect_err("target plan should fail");

    assert_eq!(
        error,
        ReplayPlanError::SnapshotCursorOutOfBounds {
            action_cursor: 2,
            action_count: 1,
        }
    );
}
