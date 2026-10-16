use super::{
    action, commit_record, snapshot_with_id, typed_undo_marker, undo_commit_record,
    undone_source_monitor_action,
};
use crate::{
    action::{ActionCommand, UndoPolicy},
    ids::ActionId,
    replay::{
        build_committed_replay_plan, build_replay_target_plan,
        build_snapshot_replay_plan_comparison,
    },
    session::ActionLog,
};

fn fresh_backwards_history() -> ActionLog {
    let mut log = ActionLog::default();
    // IDs and wall-clock metadata deliberately disagree with commit chronology.
    log.actions = vec![action(100, 300), action(2, 100), action(70, 200)];
    log.commit_records = vec![
        commit_record(100, 12, 4, 1, 300),
        commit_record(2, 8, 3, 1, 100),
        commit_record(70, 0, 1, 1, 200),
    ];
    log
}

#[test]
fn fresh_history_uses_committed_order_not_transport_id_or_wall_clock() {
    let log = fresh_backwards_history();
    let plan = build_committed_replay_plan(&log).unwrap();
    assert_eq!(
        plan.iter().map(|entry| entry.action.id).collect::<Vec<_>>(),
        vec![ActionId(100), ActionId(2), ActionId(70)]
    );
}

#[test]
fn snapshot_and_target_suffix_keep_backward_committed_order() {
    let log = fresh_backwards_history();
    let snapshots = [snapshot_with_id("first-commit", 1)];
    let target = build_replay_target_plan(&log, &snapshots, 3).unwrap();
    assert_eq!(
        target
            .suffix
            .iter()
            .map(|entry| entry.action.id)
            .collect::<Vec<_>>(),
        vec![ActionId(2), ActionId(70)]
    );
    let comparison = build_snapshot_replay_plan_comparison(&log, &snapshots[0]).unwrap();
    assert_eq!(
        comparison
            .snapshot_suffix
            .iter()
            .map(|entry| entry.action.id)
            .collect::<Vec<_>>(),
        vec![ActionId(2), ActionId(70)]
    );
}

#[test]
fn revisiting_a_complete_boundary_keeps_global_order_and_local_sequence() {
    let mut log = fresh_backwards_history();
    log.commit_records[2] = commit_record(70, 12, 4, 2, 200);
    let plan = build_committed_replay_plan(&log).unwrap();
    assert_eq!(
        plan.iter().map(|entry| entry.action.id).collect::<Vec<_>>(),
        vec![ActionId(100), ActionId(2), ActionId(70)]
    );
    assert_eq!(
        plan.iter()
            .map(|entry| entry.commit_record.commit_sequence)
            .collect::<Vec<_>>(),
        vec![1, 1, 2]
    );
}

#[test]
fn fresh_undo_uses_append_chronology_even_when_wall_clock_moves_backwards() {
    let mut log = ActionLog::default();
    log.actions = vec![
        undone_source_monitor_action(1, 1000),
        typed_undo_marker(2, 1, 500),
    ];
    log.commit_records = vec![
        commit_record(1, 8, 3, 1, 1000),
        undo_commit_record(2, 8, 3, 1, 500),
    ];
    assert!(build_committed_replay_plan(&log).unwrap().is_empty());
}

#[test]
fn fresh_undo_barrier_blocks_only_the_mutated_domain_not_an_older_monitor_target() {
    let mut log = ActionLog::default();
    let monitor = undone_source_monitor_action(1, 1000);
    let mut mc202 = action(2, 1100);
    mc202.command = ActionCommand::Mc202SetRole;
    let mut revert = action(3, 1200);
    revert.command = ActionCommand::SourceTimingRevertGrid;
    revert.undo_policy = UndoPolicy::NotUndoable {
        reason: "generated domain barrier".into(),
    };
    log.actions = vec![monitor, mc202, revert, typed_undo_marker(4, 1, 1300)];
    log.commit_records = vec![
        commit_record(1, 8, 3, 1, 1000),
        commit_record(2, 12, 4, 1, 1100),
        undo_commit_record(3, 12, 4, 1, 1200),
        undo_commit_record(4, 12, 4, 2, 1300),
    ];
    let plan = build_committed_replay_plan(&log).unwrap();
    assert_eq!(
        plan.iter().map(|entry| entry.action.id).collect::<Vec<_>>(),
        vec![ActionId(2), ActionId(3)]
    );
}
