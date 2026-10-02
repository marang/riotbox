//! Bounded, source-free measurements of the public replay cursor planners.

use super::{
    action, commit_record, snapshot_with_id, typed_undo_marker, undo_commit_record,
    undone_source_monitor_action,
};
use crate::{
    ids::ActionId,
    replay::{ReplayPlanError, build_replay_target_plan, build_snapshot_replay_plan_comparison},
    session::{ActionLog, ReplayPolicy, Snapshot},
};
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
enum HistoryKind {
    Plain,
    TypedUndo,
}

impl HistoryKind {
    fn label(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::TypedUndo => "typed_undo_lifo",
        }
    }

    fn target_count(self, action_count: usize) -> usize {
        match self {
            Self::Plain => 0,
            Self::TypedUndo => action_count / 4,
        }
    }
}

struct ScalingFixture {
    action_log: ActionLog,
    snapshots: Vec<Snapshot>,
}

fn fixture(kind: HistoryKind, action_count: usize, snapshot_count: usize) -> ScalingFixture {
    assert!((4..=4096).contains(&action_count) && action_count.is_multiple_of(4));
    let target_count = kind.target_count(action_count);
    let safe_start = target_count * 2;
    assert!(snapshot_count <= 128 && snapshot_count <= action_count - safe_start);
    let mut actions = Vec::with_capacity(action_count);
    let mut commit_records = Vec::with_capacity(action_count);

    // Typed history: all targets, reverse-LIFO markers, then a committed tail.
    // Late snapshots are safe, forcing complete resolved-history scans instead
    // of an early exit at the first unresolved target.
    for index in 0..action_count {
        let id = index as u64 + 1;
        let committed_at = 100 + id * 20;
        let is_marker = (target_count..safe_start).contains(&index);
        actions.push(if index < target_count {
            undone_source_monitor_action(id, committed_at)
        } else if is_marker {
            typed_undo_marker(id, (safe_start - index) as u64, committed_at)
        } else {
            action(id, committed_at)
        });
        let record = if is_marker {
            undo_commit_record(id, index as u64 * 4, id, 1, committed_at)
        } else {
            commit_record(id, index as u64 * 4, id, 1, committed_at)
        };
        // Every action owns a distinct boundary, so sequence 1 is unique there.
        commit_records.push(record);
    }

    let snapshots = (0..snapshot_count)
        .map(|index| {
            let cursor = safe_start + index * (action_count - safe_start) / snapshot_count;
            snapshot_with_id(&format!("scaling-snapshot-{index}"), cursor)
        })
        .collect();
    ScalingFixture {
        action_log: ActionLog {
            actions,
            commit_records,
            replay_policy: ReplayPolicy::DeterministicPreferred,
        },
        snapshots,
    }
}

#[test]
fn scaling_fixtures_keep_late_cursors_safe_and_pre_undo_cursors_unsafe() {
    const ACTIONS: usize = 16;
    for kind in [HistoryKind::Plain, HistoryKind::TypedUndo] {
        let fixture = fixture(kind, ACTIONS, 4);
        let safe_start = kind.target_count(ACTIONS) * 2;
        assert_eq!(fixture.action_log.actions.len(), ACTIONS);
        assert_eq!(fixture.action_log.commit_records.len(), ACTIONS);
        assert_eq!(fixture.snapshots.len(), 4);
        assert!(fixture.snapshots.windows(2).all(|pair| {
            pair[0].action_cursor < pair[1].action_cursor
                && pair[0].snapshot_id != pair[1].snapshot_id
        }));

        let plan = build_replay_target_plan(&fixture.action_log, &fixture.snapshots, ACTIONS)
            .expect("complete generated history is valid");
        assert_eq!(plan.anchor, fixture.snapshots.last());
        assert_eq!(
            plan.origin
                .iter()
                .map(|entry| entry.action.id)
                .collect::<Vec<_>>(),
            ((safe_start + 1)..=ACTIONS)
                .map(|id| ActionId(id as u64))
                .collect::<Vec<_>>()
        );
        let without_snapshots = build_replay_target_plan(&fixture.action_log, &[], ACTIONS)
            .expect("the full-tail path without snapshots is valid");
        assert!(without_snapshots.anchor.is_none());
        assert_eq!(without_snapshots.origin, plan.origin);
        assert_eq!(without_snapshots.suffix, plan.origin);

        for snapshot in &fixture.snapshots {
            let comparison = build_snapshot_replay_plan_comparison(&fixture.action_log, snapshot)
                .expect("each generated late snapshot is safe");
            assert_eq!(comparison.origin, plan.origin);
            assert_eq!(
                comparison.snapshot_suffix.len(),
                ACTIONS - snapshot.action_cursor
            );
            let historical = build_replay_target_plan(
                &fixture.action_log,
                &fixture.snapshots,
                snapshot.action_cursor,
            )
            .expect("each safe historical target has an exact anchor");
            assert_eq!(historical.anchor, Some(snapshot));
            assert!(historical.suffix.is_empty());
        }

        for cursor in 1..safe_start {
            let unsafe_snapshot = snapshot_with_id("unsafe-before-resolution", cursor);
            assert_eq!(
                build_snapshot_replay_plan_comparison(&fixture.action_log, &unsafe_snapshot)
                    .unwrap_err(),
                ReplayPlanError::SnapshotContainsUndoneAction {
                    snapshot_id: unsafe_snapshot.snapshot_id,
                    action_id: ActionId(1),
                }
            );
            assert_eq!(
                build_replay_target_plan(&fixture.action_log, &fixture.snapshots, cursor)
                    .unwrap_err(),
                ReplayPlanError::HistoricalReplayTargetContainsUndoneAction {
                    target_action_cursor: cursor,
                    action_id: ActionId(1),
                }
            );
        }
    }
}

fn median_build_time<T>(mut build: impl FnMut() -> Result<T, ReplayPlanError>) -> Duration {
    let mut samples = [Duration::ZERO; 5];
    for sample in &mut samples {
        let started = Instant::now();
        let result = black_box(build());
        *sample = started.elapsed();
        // Validation and destruction of each result are outside its timed window.
        let result = result.expect("generated benchmark history must remain valid");
        black_box(&result);
        drop(result);
    }
    samples.sort_unstable();
    samples[2]
}

#[test]
#[ignore = "manual release-mode public replay cursor scaling measurement"]
fn public_replay_cursor_scaling_release_benchmark() {
    if cfg!(debug_assertions) {
        panic!("run this ignored benchmark with --release");
    }
    println!("replay_cursor_scaling samples=5 statistic=median timing=public_build_only");
    for kind in [HistoryKind::Plain, HistoryKind::TypedUndo] {
        for action_count in [256, 1024, 4096] {
            for snapshot_count in [0, 16, 64, 128] {
                // Fixture creation and destruction are outside all sample windows.
                let fixture = fixture(kind, action_count, snapshot_count);
                let median = median_build_time(|| {
                    build_replay_target_plan(
                        black_box(&fixture.action_log),
                        black_box(fixture.snapshots.as_slice()),
                        black_box(action_count),
                    )
                });
                println!(
                    "path=build_replay_target_plan history={} actions={action_count} commits={} undo_targets={} snapshots={snapshot_count} target_cursor={action_count} median_ns={}",
                    kind.label(),
                    fixture.action_log.commit_records.len(),
                    kind.target_count(action_count),
                    median.as_nanos(),
                );
            }

            // This public entrypoint takes one snapshot, not a snapshot catalog.
            // Use a safe mid-tail cursor to retain a nonempty comparison suffix.
            let mut fixture = fixture(kind, action_count, 1);
            let safe_start = kind.target_count(action_count) * 2;
            fixture.snapshots[0].action_cursor = safe_start + (action_count - safe_start) / 2;
            let median = median_build_time(|| {
                build_snapshot_replay_plan_comparison(
                    black_box(&fixture.action_log),
                    black_box(&fixture.snapshots[0]),
                )
            });
            println!(
                "path=build_snapshot_replay_plan_comparison history={} actions={action_count} commits={} undo_targets={} snapshots=1 snapshot_cursor={} median_ns={}",
                kind.label(),
                fixture.action_log.commit_records.len(),
                kind.target_count(action_count),
                fixture.snapshots[0].action_cursor,
                median.as_nanos(),
            );
        }
    }
}
