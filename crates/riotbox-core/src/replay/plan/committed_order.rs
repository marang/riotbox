//! Validates declared append chronology; never sorts or repairs persisted history.
use std::collections::BTreeMap;

use super::{
    ReplayPlanEntry, ReplayPlanError, action_index_by_id, boundary_sequence_key,
    build_replay_plan_with_policy, compare_commit_records,
    undo_validation::validate_typed_undo_relations,
};
use crate::{
    action::{ActionCommand, ActionParams, ActionStatus},
    ids::ActionId,
    session::{ActionLog, ReplayPolicy},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayPolicyUpgradeError {
    InvalidHistory(ReplayPlanError),
    UnsupportedPolicy(ReplayPolicy),
    AmbiguousLegacyOrder {
        stored_action_id: ActionId,
        legacy_action_id: ActionId,
    },
}

impl std::fmt::Display for ReplayPolicyUpgradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHistory(error) => {
                write!(f, "replay chronology upgrade refused: {error:?}")
            }
            Self::UnsupportedPolicy(policy) => {
                write!(f, "replay chronology upgrade does not support {policy:?}")
            }
            Self::AmbiguousLegacyOrder {
                stored_action_id,
                legacy_action_id,
            } => write!(
                f,
                "ambiguous legacy chronology: stored action {stored_action_id}, legacy canonical action {legacy_action_id}; no history was changed"
            ),
        }
    }
}

impl std::error::Error for ReplayPolicyUpgradeError {}

/// Explicit metadata-only upgrade. Never called by load/save or recording.
///
/// It admits only histories whose full old canonical order, stored record order
/// and committed action subsequence already agree. Ambiguous adoption needs a
/// separate decision; this operation cannot reorder, repair or infer provenance.
pub fn upgrade_replay_policy_to_committed_order(
    log: &mut ActionLog,
) -> Result<(), ReplayPolicyUpgradeError> {
    let invalid = ReplayPolicyUpgradeError::InvalidHistory;
    match log.replay_policy {
        ReplayPolicy::SnapshotOnly => {
            return Err(ReplayPolicyUpgradeError::UnsupportedPolicy(
                log.replay_policy,
            ));
        }
        ReplayPolicy::DeterministicCommittedOrderV2 => {
            build_replay_plan_with_policy(log, log.replay_policy).map_err(invalid)?;
            return Ok(());
        }
        ReplayPolicy::DeterministicPreferred => {}
    }
    let actions = action_index_by_id(&log.actions).map_err(invalid)?;
    validate_typed_undo_relations(log, &actions, ReplayPolicy::DeterministicPreferred)
        .map_err(invalid)?;
    build_replay_plan_with_policy(log, ReplayPolicy::DeterministicCommittedOrderV2)
        .map_err(invalid)?;
    let mut canonical = log.commit_records.iter().collect::<Vec<_>>();
    canonical.sort_by(|left, right| compare_commit_records(left, right));
    if let Some((stored, legacy)) = log
        .commit_records
        .iter()
        .zip(canonical)
        .find(|(stored, legacy)| stored.action_id != legacy.action_id)
    {
        return Err(ReplayPolicyUpgradeError::AmbiguousLegacyOrder {
            stored_action_id: stored.action_id,
            legacy_action_id: legacy.action_id,
        });
    }
    log.replay_policy = ReplayPolicy::DeterministicCommittedOrderV2;
    Ok(())
}

pub(super) fn validate_legacy_navigation(
    entries: &[ReplayPlanEntry<'_>],
) -> Result<(), ReplayPlanError> {
    for entry in entries {
        let backwards = match entry.action.command {
            ActionCommand::TransportSeek => matches!(entry.action.params,
                ActionParams::Transport { position_beats: Some(target) }
                    if target < entry.commit_record.boundary.beat_index),
            ActionCommand::TransportStop => entry.commit_record.boundary.beat_index > 0,
            _ => false,
        };
        if backwards {
            return Err(ReplayPlanError::LegacyNavigationRequiresChronologyUpgrade {
                action_id: entry.action.id,
            });
        }
    }
    Ok(())
}

pub(super) fn validate_append_history(log: &ActionLog) -> Result<(), ReplayPlanError> {
    // Common record validation has already checked identities, statuses,
    // timestamps and uniqueness. Include structural and failed-result records:
    // executable filtering must not hide a corrupt cursor/history relationship.
    let mut records = log.commit_records.iter();
    for action in log.actions.iter().filter(|action| {
        matches!(
            action.status,
            ActionStatus::Committed | ActionStatus::Undone
        )
    }) {
        let record = records
            .next()
            .ok_or(ReplayPlanError::MissingCommittedActionRecord {
                action_id: action.id,
            })?;
        if record.action_id != action.id {
            return Err(ReplayPlanError::CommitRecordOrderMismatch {
                expected_action_id: action.id,
                record_action_id: record.action_id,
            });
        }
    }
    let mut last_sequences = BTreeMap::new();
    for record in &log.commit_records {
        let (beat, bar, phrase, kind, scene, _) = boundary_sequence_key(record);
        if let Some(previous_sequence) =
            last_sequences.insert((beat, bar, phrase, kind, scene), record.commit_sequence)
            && record.commit_sequence <= previous_sequence
        {
            return Err(ReplayPlanError::NonIncreasingCommitSequence {
                action_id: record.action_id,
                previous_sequence,
                commit_sequence: record.commit_sequence,
            });
        }
    }
    Ok(())
}
