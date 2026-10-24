//! Preset writers obey V2 domain barriers without reinterpreting Legacy stacks.
use super::{action, commit_record, typed_undo_marker};
use crate::{
    TimestampMs,
    action::{
        Action, ActionCommand, ActionParams, ActionStatus, CommitBoundary, Quantization,
        SourceMonitorMode, TargetScope, UndoPolicy,
    },
    ids::ActionId,
    replay::{
        ReplayPlanError, ReplayPolicyUpgradeError, build_committed_replay_plan,
        upgrade_replay_policy_to_committed_order,
    },
    session::{ActionLog, ReplayPolicy},
    style::PerformancePresetId,
};

const PRESETS: [PerformancePresetId; 2] = [
    PerformancePresetId::FeralBreakAlphaV1,
    PerformancePresetId::FeralBreakAlphaV2,
];
const OVERWRITTEN: [ActionCommand; 2] = [
    ActionCommand::SourceMonitorSetMode,
    ActionCommand::Mc202SetRole,
];

fn target(id: u64, at: TimestampMs, command: ActionCommand, status: ActionStatus) -> Action {
    let mut target = action(id, at);
    target.command = command;
    target.status = status;
    match command {
        ActionCommand::SourceMonitorSetMode => {
            target.params = ActionParams::SourceMonitor {
                mode: Some(SourceMonitorMode::Blend),
            };
            target.quantization = Quantization::Immediate;
            target.target.scope = Some(TargetScope::Session);
        }
        ActionCommand::Mc202SetRole => {
            target.params = ActionParams::Mutation {
                intensity: 0.65,
                target_id: Some("follower".into()),
            };
            target.quantization = Quantization::NextPhrase;
            target.target.scope = Some(TargetScope::LaneMc202);
        }
        ActionCommand::Tr909FillNext => {
            target.target.scope = Some(TargetScope::LaneTr909);
        }
        _ => panic!("fixture requires a supported typed target"),
    }
    target
}

fn preset(id: u64, at: TimestampMs, preset_id: PerformancePresetId) -> Action {
    let mut preset = action(id, at);
    preset.command = ActionCommand::PresetActivate;
    preset.params = ActionParams::Preset { preset_id };
    preset.quantization = Quantization::Immediate;
    preset.target.scope = Some(TargetScope::Session);
    preset.target.object_id = Some(preset_id.contract_id().into());
    preset.undo_policy = UndoPolicy::NotUndoable {
        reason: "preset has no typed inverse".into(),
    };
    preset
}

fn history(actions: Vec<Action>, policy: ReplayPolicy) -> ActionLog {
    let commit_records = actions
        .iter()
        .enumerate()
        .map(|(index, action)| {
            // Distinct, increasing musical positions keep Legacy and append
            // ordering identical; upgrade refusal must be about the marker.
            let mut record = commit_record(
                action.id.0,
                (index as u64 + 1) * 16,
                (index as u64 + 1) * 4 + 1,
                1,
                action.committed_at.unwrap(),
            );
            record.boundary.kind = match action.quantization {
                Quantization::Immediate => CommitBoundary::Immediate,
                Quantization::NextPhrase => CommitBoundary::Phrase,
                Quantization::NextBar => CommitBoundary::Bar,
                _ => panic!("fixture requires a supported boundary"),
            };
            record
        })
        .collect();
    ActionLog {
        actions,
        commit_records,
        replay_policy: policy,
    }
}

fn crossing(command: ActionCommand, preset_id: PerformancePresetId) -> ActionLog {
    history(
        vec![
            target(1, 100, command, ActionStatus::Undone),
            preset(2, 200, preset_id),
            typed_undo_marker(3, 1, 300),
        ],
        ReplayPolicy::DeterministicCommittedOrderV2,
    )
}

fn stale_target_error() -> ReplayPlanError {
    ReplayPlanError::InvalidUndoTargetRelation {
        undo_action_id: ActionId(3),
        target_action_id: Some(ActionId(1)),
    }
}

fn planned_ids(log: &ActionLog) -> Vec<ActionId> {
    build_committed_replay_plan(log)
        .unwrap()
        .iter()
        .map(|entry| entry.action.id)
        .collect()
}

#[test]
fn v2_rejects_monitor_and_mc202_markers_crossing_either_preset() {
    for preset_id in PRESETS {
        for command in OVERWRITTEN {
            let log = crossing(command, preset_id);
            let before = log.clone();
            assert_eq!(
                build_committed_replay_plan(&log).unwrap_err(),
                stale_target_error(),
                "{preset_id:?}/{command:?}",
            );
            assert_eq!(log, before);
        }
    }
}

#[test]
fn the_same_cross_preset_markers_keep_legacy_stack_and_serialization() {
    for preset_id in PRESETS {
        for command in OVERWRITTEN {
            for policy in [
                ReplayPolicy::DeterministicPreferred,
                ReplayPolicy::SnapshotOnly,
            ] {
                let mut log = crossing(command, preset_id);
                log.replay_policy = policy;
                let before = serde_json::to_vec(&log).unwrap();
                assert_eq!(planned_ids(&log), [ActionId(2)]);
                assert_eq!(serde_json::to_vec(&log).unwrap(), before);
            }
        }
    }
}

#[test]
fn explicit_upgrade_refuses_stale_markers_without_mutating_legacy_or_v2() {
    for preset_id in PRESETS {
        for command in OVERWRITTEN {
            for policy in [
                ReplayPolicy::DeterministicPreferred,
                ReplayPolicy::DeterministicCommittedOrderV2,
            ] {
                let mut log = crossing(command, preset_id);
                log.replay_policy = policy;
                let before = log.clone();
                assert_eq!(
                    upgrade_replay_policy_to_committed_order(&mut log),
                    Err(ReplayPolicyUpgradeError::InvalidHistory(
                        stale_target_error()
                    )),
                    "{preset_id:?}/{command:?}/{policy:?}",
                );
                assert_eq!(log, before);
            }
        }
    }
}

#[test]
fn preset_does_not_block_fill_window_or_its_explicit_legacy_upgrade() {
    for preset_id in PRESETS {
        let mut log = crossing(ActionCommand::Tr909FillNext, preset_id);
        assert_eq!(planned_ids(&log), [ActionId(2)]);
        let expected_v2 = log.clone();
        log.replay_policy = ReplayPolicy::DeterministicPreferred;
        assert_eq!(planned_ids(&log), [ActionId(2)]);
        upgrade_replay_policy_to_committed_order(&mut log).unwrap();
        assert_eq!(log, expected_v2);
        assert_eq!(planned_ids(&log), [ActionId(2)]);
    }
}

#[test]
fn v2_accepts_newer_post_preset_target_but_cannot_reopen_the_older_snapshot() {
    for preset_id in PRESETS {
        for command in OVERWRITTEN {
            let log = history(
                vec![
                    target(1, 100, command, ActionStatus::Committed),
                    preset(2, 200, preset_id),
                    target(3, 300, command, ActionStatus::Undone),
                    typed_undo_marker(4, 3, 400),
                ],
                ReplayPolicy::DeterministicCommittedOrderV2,
            );
            assert_eq!(planned_ids(&log), [ActionId(1), ActionId(2)]);
            let mut actions = log.actions;
            actions[0].status = ActionStatus::Undone;
            actions.push(typed_undo_marker(5, 1, 500));
            let stale = history(actions, ReplayPolicy::DeterministicCommittedOrderV2);
            assert_eq!(
                build_committed_replay_plan(&stale).unwrap_err(),
                ReplayPlanError::InvalidUndoTargetRelation {
                    undo_action_id: ActionId(5),
                    target_action_id: Some(ActionId(1)),
                },
            );
        }
    }
}
