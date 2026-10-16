use super::{action, commit_record, undone_source_monitor_action};
use crate::{
    action::{ActionCommand, ActionParams, ActionStatus, CommitBoundary},
    ids::ActionId,
    replay::{
        ReplayPlanError, ReplayPolicyUpgradeError, build_committed_replay_plan,
        upgrade_replay_policy_to_committed_order,
    },
    session::{ActionLog, ReplayPolicy, SessionFile, SessionVersion},
};

fn legacy() -> ActionLog {
    ActionLog {
        actions: vec![action(1, 200), action(2, 210)],
        commit_records: vec![
            commit_record(1, 8, 3, 1, 200),
            commit_record(2, 8, 3, 2, 210),
        ],
        replay_policy: ReplayPolicy::DeterministicPreferred,
    }
}

#[test]
fn fresh_policy_is_required_versioned_and_rejected_by_old_enum_readers() {
    #[derive(Debug, serde::Deserialize)]
    enum OldReplayPolicy {
        DeterministicPreferred,
        SnapshotOnly,
    }
    let session = SessionFile::new("fresh", "test", "2026-10-16");
    assert_eq!(session.session_version, SessionVersion::V1);
    assert_eq!(
        session.action_log.replay_policy,
        ReplayPolicy::DeterministicCommittedOrderV2
    );
    let value = serde_json::to_value(session.action_log.replay_policy).unwrap();
    assert!(serde_json::from_value::<OldReplayPolicy>(value).is_err());
    assert!(serde_json::from_str::<OldReplayPolicy>("\"DeterministicPreferred\"").is_ok());
    assert!(serde_json::from_str::<ReplayPolicy>("\"UnknownChronology\"").is_err());
    assert!(serde_json::from_str::<ActionLog>(r#"{"actions":[],"commit_records":[]}"#).is_err());
}

#[test]
fn legacy_roundtrip_keeps_fields_policy_and_permuted_records_without_mutation() {
    let mut log = legacy();
    log.commit_records.reverse();
    let bytes = serde_json::to_vec(&log).unwrap();
    let decoded: ActionLog = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(decoded, log);
    assert_eq!(serde_json::to_vec(&decoded).unwrap(), bytes);
    let plan = build_committed_replay_plan(&decoded).unwrap();
    assert_eq!(
        plan.iter().map(|entry| entry.action.id).collect::<Vec<_>>(),
        vec![ActionId(1), ActionId(2)]
    );
    assert_eq!(serde_json::to_vec(&decoded).unwrap(), bytes);
}

#[test]
fn explicit_unambiguous_upgrade_changes_only_policy_and_is_idempotent() {
    let mut log = legacy();
    let mut expected = log.clone();
    expected.replay_policy = ReplayPolicy::DeterministicCommittedOrderV2;
    upgrade_replay_policy_to_committed_order(&mut log).unwrap();
    assert_eq!(log, expected);
    upgrade_replay_policy_to_committed_order(&mut log).unwrap();
    assert_eq!(log, expected);
}

#[test]
fn ambiguous_legacy_order_and_permutations_refuse_upgrade_without_mutation() {
    let mut log = legacy();
    log.commit_records[0] = commit_record(1, 12, 4, 1, 200);
    let before = log.clone();
    assert!(matches!(
        upgrade_replay_policy_to_committed_order(&mut log),
        Err(ReplayPolicyUpgradeError::AmbiguousLegacyOrder { .. })
    ));
    assert_eq!(log, before);
    log = legacy();
    log.commit_records.reverse();
    let before = log.clone();
    assert!(matches!(
        upgrade_replay_policy_to_committed_order(&mut log),
        Err(ReplayPolicyUpgradeError::InvalidHistory(
            ReplayPlanError::CommitRecordOrderMismatch { .. }
        ))
    ));
    assert_eq!(log, before);
}

#[test]
fn incomplete_legacy_and_snapshot_only_policy_are_not_silently_upgraded() {
    let mut log = legacy();
    log.commit_records.clear();
    assert!(
        build_committed_replay_plan(&log).unwrap().is_empty(),
        "retain declared legacy omission semantics"
    );
    let before = log.clone();
    assert!(matches!(
        upgrade_replay_policy_to_committed_order(&mut log),
        Err(ReplayPolicyUpgradeError::InvalidHistory(
            ReplayPlanError::MissingCommittedActionRecord { .. }
        ))
    ));
    assert_eq!(log, before);
    log.replay_policy = ReplayPolicy::SnapshotOnly;
    let before = log.clone();
    assert_eq!(
        upgrade_replay_policy_to_committed_order(&mut log),
        Err(ReplayPolicyUpgradeError::UnsupportedPolicy(
            ReplayPolicy::SnapshotOnly
        ))
    );
    assert_eq!(log, before);
}

#[test]
fn declared_append_history_rejects_missing_records_and_orphan_undone_state() {
    let mut log = legacy();
    log.replay_policy = ReplayPolicy::DeterministicCommittedOrderV2;
    log.commit_records.pop();
    assert_eq!(
        build_committed_replay_plan(&log).unwrap_err(),
        ReplayPlanError::MissingCommittedActionRecord {
            action_id: ActionId(2)
        }
    );
    log.actions = vec![undone_source_monitor_action(1, 200)];
    assert_eq!(
        build_committed_replay_plan(&log).unwrap_err(),
        ReplayPlanError::UnresolvedUndoneAction {
            action_id: ActionId(1)
        }
    );
}

#[test]
fn revisited_sequences_increase_without_requiring_contiguity() {
    let mut log = legacy();
    log.replay_policy = ReplayPolicy::DeterministicCommittedOrderV2;
    log.commit_records[1].commit_sequence = 9;
    assert!(build_committed_replay_plan(&log).is_ok());
    log.commit_records[0].commit_sequence = 10;
    assert_eq!(
        build_committed_replay_plan(&log).unwrap_err(),
        ReplayPlanError::NonIncreasingCommitSequence {
            action_id: ActionId(2),
            previous_sequence: 10,
            commit_sequence: 9
        }
    );
}

#[test]
fn failed_result_records_still_participate_in_cursor_order_validation() {
    let mut log = legacy();
    log.replay_policy = ReplayPolicy::DeterministicCommittedOrderV2;
    log.actions[0].result.as_mut().unwrap().accepted = false;
    let plan = build_committed_replay_plan(&log).unwrap();
    assert_eq!(
        plan.iter().map(|entry| entry.action.id).collect::<Vec<_>>(),
        vec![ActionId(2)]
    );
    log.commit_records.reverse();
    assert!(matches!(
        build_committed_replay_plan(&log),
        Err(ReplayPlanError::CommitRecordOrderMismatch { .. })
    ));
}

#[test]
fn uncommitted_entries_do_not_need_records_or_disturb_committed_cursors() {
    let mut log = legacy();
    log.replay_policy = ReplayPolicy::DeterministicCommittedOrderV2;
    let mut rejected = action(99, 205);
    rejected.status = ActionStatus::Rejected;
    rejected.committed_at = None;
    log.actions.insert(1, rejected);
    assert_eq!(build_committed_replay_plan(&log).unwrap().len(), 2);
}

#[test]
fn legacy_backward_navigation_is_visible_and_only_safe_explicit_upgrade_admits_it() {
    let mut log = legacy();
    for record in &mut log.commit_records {
        record.boundary.kind = CommitBoundary::Immediate;
    }
    log.actions[1].command = ActionCommand::TransportSeek;
    log.actions[1].params = ActionParams::Transport {
        position_beats: Some(0),
    };
    assert_eq!(
        build_committed_replay_plan(&log).unwrap_err(),
        ReplayPlanError::LegacyNavigationRequiresChronologyUpgrade {
            action_id: ActionId(2)
        }
    );
    upgrade_replay_policy_to_committed_order(&mut log).unwrap();
    assert_eq!(build_committed_replay_plan(&log).unwrap().len(), 2);
}
