//! Independent scan oracle retained from the pre-index cursor rules.
use super::{
    action, commit_record, snapshot_with_id, typed_undo_marker, undo_commit_record,
    undone_source_monitor_action,
};
use crate::{
    action::{ActionCommand, ActionParams, ActionStatus},
    ids::ActionId,
    replay::plan::{
        ReplayPlanError, ReplayTargetPlan, SnapshotReplayPlanComparison, action_ids_before_cursor,
        build_committed_replay_plan, build_replay_target_plan,
        build_snapshot_replay_plan_comparison, select_replay_snapshot_anchor,
    },
    session::{ActionLog, ReplayPolicy, Snapshot},
};

fn scan_unresolved(log: &ActionLog, cursor: usize) -> Option<ActionId> {
    log.actions
        .iter()
        .take(cursor)
        .enumerate()
        .find_map(|(index, target)| {
            let resolved = log
                .actions
                .iter()
                .take(cursor)
                .skip(index + 1)
                .any(|marker| {
                    marker.command == ActionCommand::UndoLast
                        && marker.status == ActionStatus::Committed
                        && marker.result.as_ref().is_some_and(|result| result.accepted)
                        && matches!(&marker.params, ActionParams::Undo { target_action_id }
                    if *target_action_id == target.id)
                        && log
                            .commit_records
                            .iter()
                            .any(|record| record.action_id == marker.id)
                });
            (target.status == ActionStatus::Undone && !resolved).then_some(target.id)
        })
}

fn scan_target<'a>(
    log: &'a ActionLog,
    snapshots: &'a [Snapshot],
    target: usize,
) -> Result<ReplayTargetPlan<'a>, ReplayPlanError> {
    let origin = build_committed_replay_plan(log)?;
    select_replay_snapshot_anchor(snapshots, target, log.actions.len())?;
    let mut anchor: Option<&Snapshot> = None;
    for snapshot in snapshots {
        if snapshot.action_cursor <= target
            && scan_unresolved(log, snapshot.action_cursor).is_none()
            && anchor.is_none_or(|prior| snapshot.action_cursor >= prior.action_cursor)
        {
            anchor = Some(snapshot);
        }
    }
    if target < log.actions.len()
        && let Some(action_id) = scan_unresolved(log, target)
    {
        return Err(
            ReplayPlanError::HistoricalReplayTargetContainsUndoneAction {
                target_action_cursor: target,
                action_id,
            },
        );
    }
    let skipped =
        action_ids_before_cursor(&log.actions, anchor.map_or(0, |snap| snap.action_cursor));
    let wanted = action_ids_before_cursor(&log.actions, target);
    let suffix = origin
        .iter()
        .filter(|entry| wanted.contains(&entry.action.id))
        .filter(|entry| !skipped.contains(&entry.action.id))
        .cloned()
        .collect();
    Ok(ReplayTargetPlan {
        origin,
        suffix,
        anchor,
        target_action_cursor: target,
    })
}

fn scan_comparison<'a>(
    log: &'a ActionLog,
    snapshot: &Snapshot,
) -> Result<SnapshotReplayPlanComparison<'a>, ReplayPlanError> {
    if snapshot.action_cursor > log.actions.len() {
        return Err(ReplayPlanError::SnapshotCursorOutOfBounds {
            action_cursor: snapshot.action_cursor,
            action_count: log.actions.len(),
        });
    }
    let origin = build_committed_replay_plan(log)?;
    if let Some(action_id) = scan_unresolved(log, snapshot.action_cursor) {
        return Err(ReplayPlanError::SnapshotContainsUndoneAction {
            snapshot_id: snapshot.snapshot_id.clone(),
            action_id,
        });
    }
    let applied = action_ids_before_cursor(&log.actions, snapshot.action_cursor);
    let snapshot_suffix = origin
        .iter()
        .filter(|entry| !applied.contains(&entry.action.id))
        .cloned()
        .collect();
    Ok(SnapshotReplayPlanComparison {
        origin,
        snapshot_suffix,
        snapshot_action_cursor: snapshot.action_cursor,
    })
}

fn history(targets: usize, tail: usize, legacy: bool) -> ActionLog {
    let mut log = ActionLog {
        actions: Vec::new(),
        commit_records: Vec::new(),
        replay_policy: ReplayPolicy::DeterministicPreferred,
    };
    if legacy {
        log.actions.push(undone_source_monitor_action(900, 200));
        log.commit_records.push(commit_record(900, 8, 2, 900, 200));
        let mut marker = typed_undo_marker(901, 900, 220);
        marker.params = ActionParams::Empty;
        log.actions.push(marker); // Legacy marker has no commit record and is untrusted.
    }
    for index in 0..targets {
        let id = index as u64 + 1;
        let at = 300 + id;
        log.actions.push(undone_source_monitor_action(id, at));
        log.commit_records
            .push(commit_record(id, 12, 3, id as u32, at));
    }
    for (sequence, index) in (0..targets).rev().enumerate() {
        let id = 100 + sequence as u64;
        let at = 500 + id;
        log.actions
            .push(typed_undo_marker(id, index as u64 + 1, at));
        log.commit_records
            .push(undo_commit_record(id, 12, 3, sequence as u32 + 1, at));
    }
    for index in 0..tail {
        let id = 200 + index as u64;
        let at = 1000 + id;
        log.actions.push(action(id, at));
        log.commit_records
            .push(commit_record(id, 16, 4, index as u32 + 1, at));
    }
    log
}

fn compare_all_cursors(log: &ActionLog) {
    let count = log.actions.len();
    let mut snapshots: Vec<_> = (0..=count)
        .rev()
        .map(|cursor| snapshot_with_id(&format!("first-{cursor}"), cursor))
        .collect();
    snapshots.extend((0..=count).map(|cursor| snapshot_with_id(&format!("last-{cursor}"), cursor)));
    for target in 0..=count + 1 {
        for candidates in [&snapshots[..], &[][..]] {
            assert_eq!(
                build_replay_target_plan(log, candidates, target),
                scan_target(log, candidates, target),
                "target {target}"
            );
        }
    }
    for snapshot in &snapshots {
        assert_eq!(
            build_snapshot_replay_plan_comparison(log, snapshot),
            scan_comparison(log, snapshot),
            "snapshot {}",
            snapshot.snapshot_id.as_str()
        );
    }
    snapshots.push(snapshot_with_id("out-of-range", count + 1));
    assert_eq!(
        build_replay_target_plan(log, &snapshots, count),
        scan_target(log, &snapshots, count)
    );
    let bad = snapshots.last().unwrap();
    assert_eq!(
        build_snapshot_replay_plan_comparison(log, bad),
        scan_comparison(log, bad)
    );
}

#[test]
fn cursor_plans_match_scan_oracle_for_typed_legacy_and_plain_histories() {
    for targets in 0..=5 {
        for tail in 0..=3 {
            for legacy in [false, true] {
                let log = history(targets, tail, legacy);
                build_committed_replay_plan(&log).expect("fixture is a valid history");
                compare_all_cursors(&log);
                if !legacy && targets > 1 {
                    let mut nonmonotonic = log;
                    let remap = |id: ActionId| {
                        if id.0 <= targets as u64 {
                            ActionId(1000 - id.0 * 100)
                        } else {
                            id
                        }
                    };
                    for action in &mut nonmonotonic.actions {
                        action.id = remap(action.id);
                        if let ActionParams::Undo { target_action_id } = &mut action.params {
                            *target_action_id = remap(*target_action_id);
                        }
                    }
                    for record in &mut nonmonotonic.commit_records {
                        record.action_id = remap(record.action_id);
                    }
                    build_committed_replay_plan(&nonmonotonic).expect("IDs need not be monotonic");
                    compare_all_cursors(&nonmonotonic);
                }
            }
        }
    }
}

#[test]
fn cursor_plans_match_scan_oracle_for_rejected_and_noncommitted_actions() {
    let mut log = history(3, 2, false);
    log.actions
        .last_mut()
        .unwrap()
        .result
        .as_mut()
        .unwrap()
        .accepted = false;
    let mut pending = action(800, 2000);
    pending.status = ActionStatus::Queued;
    pending.committed_at = None;
    pending.result = None;
    log.actions.insert(1, pending);
    log.commit_records.reverse();
    build_committed_replay_plan(&log).expect("fixture is a valid mixed history");
    compare_all_cursors(&log);
}

#[test]
fn cursor_errors_keep_scan_precedence_for_malformed_histories_and_cursors() {
    let valid = history(3, 2, false);
    let mut malformed = Vec::new();
    let mut log = valid.clone();
    log.actions.push(log.actions[0].clone());
    malformed.push(log);
    let mut log = valid.clone();
    log.commit_records.push(log.commit_records[0].clone());
    malformed.push(log);
    let mut log = valid.clone();
    log.commit_records[0].action_id = ActionId(999);
    malformed.push(log);
    let mut log = valid.clone();
    log.actions[3].params = ActionParams::Undo {
        target_action_id: ActionId(1),
    };
    malformed.push(log);
    let mut log = valid.clone();
    log.actions[3].result.as_mut().unwrap().accepted = false;
    malformed.push(log);
    let mut log = valid.clone();
    log.commit_records.remove(3);
    malformed.push(log);
    let mut log = valid.clone();
    log.actions[3].params = ActionParams::Empty;
    malformed.push(log);
    let mut log = valid.clone();
    log.actions[3].result = None;
    malformed.push(log);
    let mut log = valid.clone();
    log.actions.swap(2, 3); // Typed marker precedes its own target.
    malformed.push(log);
    let mut log = valid;
    log.actions[0].committed_at = None;
    malformed.push(log);
    for log in malformed {
        assert!(build_committed_replay_plan(&log).is_err());
        compare_all_cursors(&log);
    }
}
