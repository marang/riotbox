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

fn undone_source_monitor_action(id: u64, committed_at: TimestampMs) -> Action {
    let mut action = action(id, committed_at);
    action.command = ActionCommand::SourceMonitorSetMode;
    action.params = ActionParams::SourceMonitor {
        mode: Some(crate::action::SourceMonitorMode::Blend),
    };
    action.status = ActionStatus::Undone;
    action
}

fn typed_undo_marker(id: u64, target_action_id: u64, committed_at: TimestampMs) -> Action {
    Action {
        id: ActionId(id),
        actor: ActorType::User,
        command: ActionCommand::UndoLast,
        params: ActionParams::Undo {
            target_action_id: ActionId(target_action_id),
        },
        target: ActionTarget::default(),
        requested_at: committed_at,
        quantization: Quantization::Immediate,
        status: ActionStatus::Committed,
        committed_at: Some(committed_at),
        result: Some(ActionResult {
            accepted: true,
            summary: "undid target action".into(),
        }),
        undo_policy: UndoPolicy::NotUndoable {
            reason: "undo marker".into(),
        },
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
            phrase_index: bar_index.saturating_sub(1) / 4 + 1,
            scene_id: Some(SceneId::from("scene-1")),
        },
        commit_sequence,
        committed_at,
        mc202_source_phrase_plan: None,
    }
}

fn undo_commit_record(
    action_id: u64,
    beat_index: u64,
    bar_index: u64,
    commit_sequence: u32,
    committed_at: TimestampMs,
) -> ActionCommitRecord {
    let mut record = commit_record(
        action_id,
        beat_index,
        bar_index,
        commit_sequence,
        committed_at,
    );
    record.boundary.kind = CommitBoundary::Immediate;
    record
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
        payload: None,
    }
}

fn valid_typed_undo_action_log() -> ActionLog {
    ActionLog {
        actions: vec![
            undone_source_monitor_action(1, 200),
            typed_undo_marker(2, 1, 220),
        ],
        commit_records: vec![
            commit_record(1, 8, 2, 1, 200),
            undo_commit_record(2, 8, 2, 1, 220),
        ],
        replay_policy: ReplayPolicy::DeterministicPreferred,
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
fn replay_plan_skips_undone_historical_commit() {
    let undone = undone_source_monitor_action(1, 200);
    let action_log = ActionLog {
        actions: vec![undone, action(2, 210)],
        commit_records: vec![
            commit_record(1, 8, 2, 1, 200),
            commit_record(2, 8, 2, 2, 210),
        ],
        replay_policy: ReplayPolicy::DeterministicPreferred,
    };

    let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
    let action_ids = plan.iter().map(|entry| entry.action.id).collect::<Vec<_>>();

    assert_eq!(action_ids, vec![ActionId(2)]);
}

#[test]
fn replay_plan_keeps_rejected_committed_record_as_non_executable_history() {
    let mut rejected_at_side_effect = action(1, 200);
    rejected_at_side_effect.result = Some(ActionResult {
        accepted: false,
        summary: "source-owned phrase unavailable at this boundary".into(),
    });
    let action_log = ActionLog {
        actions: vec![rejected_at_side_effect, action(2, 210)],
        commit_records: vec![
            commit_record(1, 8, 3, 1, 200),
            commit_record(2, 8, 3, 2, 210),
        ],
        replay_policy: ReplayPolicy::DeterministicPreferred,
    };

    let plan = build_committed_replay_plan(&action_log).expect("valid replay history");
    let action_ids = plan.iter().map(|entry| entry.action.id).collect::<Vec<_>>();

    assert_eq!(action_log.commit_records.len(), 2);
    assert_eq!(action_ids, vec![ActionId(2)]);
}

#[test]
fn replay_plan_rejects_undone_action_without_typed_undo_semantics() {
    let mut undone = action(1, 200);
    undone.command = ActionCommand::Tr909SetSlam;
    undone.status = ActionStatus::Undone;
    let action_log = ActionLog {
        actions: vec![undone],
        commit_records: vec![commit_record(1, 8, 2, 1, 200)],
        replay_policy: ReplayPolicy::DeterministicPreferred,
    };

    let error = build_committed_replay_plan(&action_log)
        .expect_err("unsupported live undo must remain an honest replay error");

    assert_eq!(
        error,
        ReplayPlanError::NonCommittedAction {
            action_id: ActionId(1),
            status: ActionStatus::Undone,
        }
    );
}

#[test]
fn replay_plan_rejects_non_committed_statuses_other_than_undone() {
    for status in [
        ActionStatus::Requested,
        ActionStatus::Queued,
        ActionStatus::PendingCommit,
        ActionStatus::Rejected,
        ActionStatus::Failed,
    ] {
        let mut non_committed = action(1, 200);
        non_committed.status = status;
        let action_log = ActionLog {
            actions: vec![non_committed],
            commit_records: vec![commit_record(1, 8, 2, 1, 200)],
            replay_policy: ReplayPolicy::DeterministicPreferred,
        };

        let error = build_committed_replay_plan(&action_log).expect_err("plan should fail");

        assert_eq!(
            error,
            ReplayPlanError::NonCommittedAction {
                action_id: ActionId(1),
                status,
            }
        );
    }
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
fn replay_plan_rejects_duplicate_action_ids_even_without_duplicate_commit_records() {
    let action_log = ActionLog {
        actions: vec![action(1, 200), action(1, 210)],
        commit_records: vec![commit_record(1, 8, 2, 1, 200)],
        replay_policy: ReplayPolicy::DeterministicPreferred,
    };

    let error = build_committed_replay_plan(&action_log).expect_err("plan should fail");

    assert_eq!(
        error,
        ReplayPlanError::DuplicateActionId {
            action_id: ActionId(1)
        }
    );
}

#[test]
fn replay_plan_validates_and_omits_typed_undo_marker() {
    let action_log = ActionLog {
        actions: vec![
            undone_source_monitor_action(1, 200),
            typed_undo_marker(2, 1, 220),
            action(3, 300),
        ],
        commit_records: vec![
            commit_record(1, 8, 2, 1, 200),
            undo_commit_record(2, 8, 2, 1, 220),
            commit_record(3, 12, 3, 1, 300),
        ],
        replay_policy: ReplayPolicy::DeterministicPreferred,
    };

    let plan = build_committed_replay_plan(&action_log).expect("typed undo history is valid");

    assert_eq!(
        plan.iter().map(|entry| entry.action.id).collect::<Vec<_>>(),
        vec![ActionId(3)]
    );
}

#[test]
fn replay_plan_rejects_commit_record_for_untyped_undo_marker() {
    let mut marker = typed_undo_marker(2, 1, 220);
    marker.params = ActionParams::Empty;
    let action_log = ActionLog {
        actions: vec![undone_source_monitor_action(1, 200), marker],
        commit_records: vec![
            commit_record(1, 8, 2, 1, 200),
            undo_commit_record(2, 8, 2, 1, 220),
        ],
        replay_policy: ReplayPolicy::DeterministicPreferred,
    };

    let error = build_committed_replay_plan(&action_log).expect_err("marker should fail");

    assert_eq!(
        error,
        ReplayPlanError::InvalidUndoTargetRelation {
            undo_action_id: ActionId(2),
            target_action_id: None,
        }
    );
}

#[test]
fn replay_plan_rejects_typed_undo_marker_contract_mutations() {
    type LogMutation = fn(&mut ActionLog);
    let mutations: [(&str, LogMutation); 7] = [
        ("marker undoable", |log| {
            log.actions[1].undo_policy = UndoPolicy::Undoable
        }),
        ("marker quantized", |log| {
            log.actions[1].quantization = Quantization::NextBeat
        }),
        ("marker non-immediate boundary", |log| {
            log.commit_records[1].boundary.kind = CommitBoundary::Bar
        }),
        ("marker missing result", |log| log.actions[1].result = None),
        ("marker rejected result", |log| {
            log.actions[1].result = Some(ActionResult {
                accepted: false,
                summary: "rejected".into(),
            });
        }),
        ("target not undoable", |log| {
            log.actions[0].undo_policy = UndoPolicy::NotUndoable {
                reason: "legacy state has no snapshot".into(),
            };
        }),
        ("target rejected result", |log| {
            log.actions[0].result = Some(ActionResult {
                accepted: false,
                summary: "rejected".into(),
            });
        }),
    ];

    for (label, mutate) in mutations {
        let mut action_log = valid_typed_undo_action_log();
        mutate(&mut action_log);

        let error = build_committed_replay_plan(&action_log).expect_err(label);
        assert_eq!(
            error,
            ReplayPlanError::InvalidUndoTargetRelation {
                undo_action_id: ActionId(2),
                target_action_id: Some(ActionId(1)),
            },
            "{label}"
        );
    }
}

#[test]
fn replay_plan_rejects_typed_undo_marker_without_its_commit_record() {
    let mut action_log = valid_typed_undo_action_log();
    action_log.commit_records.pop();

    let error = build_committed_replay_plan(&action_log).expect_err("marker should fail");

    assert_eq!(
        error,
        ReplayPlanError::InvalidUndoTargetRelation {
            undo_action_id: ActionId(2),
            target_action_id: Some(ActionId(1)),
        }
    );
}

#[test]
fn replay_plan_rejects_two_typed_markers_for_the_same_target() {
    let mut action_log = valid_typed_undo_action_log();
    action_log.actions.push(typed_undo_marker(3, 1, 230));
    action_log
        .commit_records
        .push(undo_commit_record(3, 8, 2, 2, 230));

    let error = build_committed_replay_plan(&action_log).expect_err("second marker should fail");

    assert_eq!(
        error,
        ReplayPlanError::InvalidUndoTargetRelation {
            undo_action_id: ActionId(3),
            target_action_id: Some(ActionId(1)),
        }
    );
}

#[test]
fn replay_plan_rejects_marker_that_skips_newer_active_typed_action() {
    let mut action_log = valid_typed_undo_action_log();
    let newer_monitor = {
        let mut action = action(2, 210);
        action.command = ActionCommand::SourceMonitorSetMode;
        action.params = ActionParams::SourceMonitor {
            mode: Some(crate::action::SourceMonitorMode::Riotbox),
        };
        action.quantization = Quantization::Immediate;
        action
    };
    action_log.actions = vec![
        undone_source_monitor_action(1, 200),
        newer_monitor,
        typed_undo_marker(3, 1, 220),
    ];
    action_log.commit_records = vec![
        commit_record(1, 8, 2, 1, 200),
        undo_commit_record(2, 8, 2, 1, 210),
        undo_commit_record(3, 8, 2, 2, 220),
    ];

    let error = build_committed_replay_plan(&action_log).expect_err("skipped target should fail");

    assert_eq!(
        error,
        ReplayPlanError::InvalidUndoTargetRelation {
            undo_action_id: ActionId(3),
            target_action_id: Some(ActionId(1)),
        }
    );
}

#[test]
fn legacy_untyped_marker_does_not_make_a_post_marker_snapshot_safe() {
    let mut legacy_marker = typed_undo_marker(2, 1, 220);
    legacy_marker.params = ActionParams::Empty;
    let action_log = ActionLog {
        actions: vec![undone_source_monitor_action(1, 200), legacy_marker],
        commit_records: vec![commit_record(1, 8, 2, 1, 200)],
        replay_policy: ReplayPolicy::DeterministicPreferred,
    };

    let error = build_snapshot_replay_plan_comparison(
        &action_log,
        &snapshot_with_id("legacy-post-marker", 2),
    )
    .expect_err("legacy marker cannot establish a safe snapshot boundary");

    assert_eq!(
        error,
        ReplayPlanError::SnapshotContainsUndoneAction {
            snapshot_id: SnapshotId::from("legacy-post-marker"),
            action_id: ActionId(1),
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
                mc202_source_phrase_plan: None,
            },
            ActionCommitRecord {
                action_id: ActionId(2),
                boundary: duplicated_boundary.clone(),
                commit_sequence: 1,
                committed_at: 210,
                mc202_source_phrase_plan: None,
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
fn snapshot_comparison_rejects_payload_that_contains_now_undone_state() {
    let undone = undone_source_monitor_action(1, 200);
    let action_log = ActionLog {
        actions: vec![undone],
        commit_records: vec![commit_record(1, 8, 2, 1, 200)],
        replay_policy: ReplayPolicy::DeterministicPreferred,
    };
    let snapshot = snapshot_with_id("snap-before-undo", 1);

    let error = build_snapshot_replay_plan_comparison(&action_log, &snapshot)
        .expect_err("stale snapshot payload must not be treated as replay truth");

    assert_eq!(
        error,
        ReplayPlanError::SnapshotContainsUndoneAction {
            snapshot_id: SnapshotId::from("snap-before-undo"),
            action_id: ActionId(1),
        }
    );
}

#[test]
fn snapshot_comparison_accepts_payload_created_after_typed_undo() {
    let action_log = ActionLog {
        actions: vec![
            undone_source_monitor_action(1, 200),
            typed_undo_marker(2, 1, 220),
            action(3, 300),
        ],
        commit_records: vec![
            commit_record(1, 8, 2, 1, 200),
            undo_commit_record(2, 8, 2, 1, 220),
            commit_record(3, 12, 3, 1, 300),
        ],
        replay_policy: ReplayPolicy::DeterministicPreferred,
    };

    let comparison =
        build_snapshot_replay_plan_comparison(&action_log, &snapshot_with_id("snap-after-undo", 2))
            .expect("post-undo snapshot is safe");

    assert_eq!(
        comparison
            .snapshot_suffix
            .iter()
            .map(|entry| entry.action.id)
            .collect::<Vec<_>>(),
        vec![ActionId(3)]
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
fn replay_target_plan_skips_snapshot_anchor_that_contains_undone_state() {
    let undone = undone_source_monitor_action(1, 200);
    let action_log = ActionLog {
        actions: vec![undone, action(2, 210)],
        commit_records: vec![
            commit_record(1, 8, 2, 1, 200),
            commit_record(2, 8, 2, 2, 210),
        ],
        replay_policy: ReplayPolicy::DeterministicPreferred,
    };
    let snapshots = vec![
        snapshot_with_id("snap-safe-empty", 0),
        snapshot_with_id("snap-before-undo", 1),
    ];

    let target_plan =
        build_replay_target_plan(&action_log, &snapshots, 2).expect("latest target is replayable");

    assert_eq!(
        target_plan
            .anchor
            .map(|snapshot| snapshot.snapshot_id.as_str()),
        Some("snap-safe-empty")
    );
    assert_eq!(
        target_plan
            .suffix
            .iter()
            .map(|entry| entry.action.id)
            .collect::<Vec<_>>(),
        vec![ActionId(2)]
    );
}

#[test]
fn replay_target_plan_rejects_historical_cursor_affected_by_later_undo() {
    let undone = undone_source_monitor_action(1, 200);
    let action_log = ActionLog {
        actions: vec![undone, action(2, 210)],
        commit_records: vec![
            commit_record(1, 8, 2, 1, 200),
            commit_record(2, 8, 2, 2, 210),
        ],
        replay_policy: ReplayPolicy::DeterministicPreferred,
    };

    let error = build_replay_target_plan(&action_log, &[], 1)
        .expect_err("historical pre-undo state cannot be reconstructed safely");

    assert_eq!(
        error,
        ReplayPlanError::HistoricalReplayTargetContainsUndoneAction {
            target_action_cursor: 1,
            action_id: ActionId(1),
        }
    );
}

#[test]
fn replay_target_plan_accepts_post_undo_cursor_and_snapshot_anchor() {
    let action_log = ActionLog {
        actions: vec![
            undone_source_monitor_action(1, 200),
            typed_undo_marker(2, 1, 220),
            action(3, 300),
        ],
        commit_records: vec![
            commit_record(1, 8, 2, 1, 200),
            undo_commit_record(2, 8, 2, 1, 220),
            commit_record(3, 12, 3, 1, 300),
        ],
        replay_policy: ReplayPolicy::DeterministicPreferred,
    };
    let snapshots = vec![snapshot_with_id("snap-after-undo", 2)];

    let post_undo = build_replay_target_plan(&action_log, &snapshots, 2)
        .expect("post-undo historical cursor is replayable");
    assert_eq!(
        post_undo
            .anchor
            .map(|snapshot| snapshot.snapshot_id.as_str()),
        Some("snap-after-undo")
    );
    assert!(post_undo.suffix.is_empty());

    let with_tail = build_replay_target_plan(&action_log, &snapshots, 3)
        .expect("tail after post-undo snapshot is replayable");
    assert_eq!(
        with_tail
            .suffix
            .iter()
            .map(|entry| entry.action.id)
            .collect::<Vec<_>>(),
        vec![ActionId(3)]
    );
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
