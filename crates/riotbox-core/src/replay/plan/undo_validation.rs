//! Structural Undo validation for declared replay chronology versions.
use std::collections::{BTreeMap, BTreeSet};

use super::ReplayPlanError;
use crate::{
    action::{
        Action, ActionCommand, ActionParams, ActionStatus, CommitBoundary, Quantization,
        TypedUndoStateDomain, UndoPolicy,
    },
    ids::ActionId,
    session::{ActionLog, ReplayPolicy},
};

enum UndoCandidates {
    Legacy(Vec<ActionId>),
    Append(BTreeMap<TypedUndoStateDomain, Vec<(usize, ActionId)>>),
}

impl UndoCandidates {
    fn last(&self) -> Option<ActionId> {
        match self {
            Self::Legacy(stack) => stack.last().copied(),
            Self::Append(domains) => domains
                .values()
                .filter_map(|stack| stack.last())
                .max_by_key(|(index, _)| index)
                .map(|(_, id)| *id),
        }
    }

    fn push(&mut self, index: usize, action: &Action) {
        match self {
            Self::Legacy(stack) => stack.push(action.id),
            Self::Append(domains) => {
                if let Some(domain) = action.command.typed_undo_state_domain() {
                    domains.entry(domain).or_default().push((index, action.id));
                }
            }
        }
    }

    fn barrier(&mut self, command: ActionCommand) {
        if let Self::Append(domains) = self {
            for (domain, stack) in domains {
                if command.mutates_typed_undo_state_domain(*domain) {
                    stack.clear();
                }
            }
        }
    }

    fn pop(&mut self) {
        match self {
            Self::Legacy(stack) => {
                stack.pop();
            }
            Self::Append(domains) => {
                let latest = domains
                    .iter()
                    .filter_map(|(domain, stack)| stack.last().map(|(index, _)| (*index, *domain)))
                    .max_by_key(|(index, _)| *index);
                if let Some((_, domain)) = latest
                    && let Some(stack) = domains.get_mut(&domain)
                {
                    stack.pop();
                }
            }
        }
    }
}

pub(super) fn validate_typed_undo_relations(
    action_log: &ActionLog,
    action_by_id: &BTreeMap<ActionId, &Action>,
    policy: ReplayPolicy,
) -> Result<(), ReplayPlanError> {
    let append = policy.uses_committed_append_order();
    let mut candidates = if append {
        UndoCandidates::Append(BTreeMap::new())
    } else {
        UndoCandidates::Legacy(Vec::new())
    };
    let mut resolved_targets = BTreeSet::new();
    let mut first_record_by_action = BTreeMap::new();
    for record in &action_log.commit_records {
        // Preserve the legacy first-record/error precedence for duplicates.
        first_record_by_action
            .entry(record.action_id)
            .or_insert(record);
    }
    for (index, action) in action_log.actions.iter().enumerate() {
        if action.command != ActionCommand::UndoLast {
            let eligible = matches!(
                action.status,
                ActionStatus::Committed | ActionStatus::Undone
            ) && action.command.has_typed_undo_semantics()
                && matches!(action.undo_policy, UndoPolicy::Undoable)
                && action.result.as_ref().is_some_and(|result| result.accepted);
            if eligible {
                candidates.push(index, action);
            } else if append
                && action.status == ActionStatus::Committed
                && !action
                    .result
                    .as_ref()
                    .is_some_and(|result| !result.accepted)
            {
                // Same Core-owned domain predicates as live Undo. A non-undoable
                // MC-202 mutation cannot block a still-valid Source Monitor target.
                candidates.barrier(action.command);
            }
            continue;
        }
        let target_action_id = match &action.params {
            ActionParams::Undo { target_action_id } => Some(*target_action_id),
            _ => None,
        };
        let Some(target_action_id) = target_action_id else {
            if first_record_by_action.contains_key(&action.id) {
                return Err(ReplayPlanError::InvalidUndoTargetRelation {
                    undo_action_id: action.id,
                    target_action_id: None,
                });
            }
            // Legacy untyped markers remain untrusted cursor evidence.
            if action.status == ActionStatus::Committed
                && let Some(target_id) = candidates.last()
                && action_by_id
                    .get(&target_id)
                    .is_some_and(|target| target.status == ActionStatus::Undone)
            {
                candidates.pop();
            }
            continue;
        };
        let marker_record = first_record_by_action.get(&action.id);
        let target = action_by_id.get(&target_action_id);
        let target_record = first_record_by_action.get(&target_action_id);
        let latest_active_target = candidates.last();
        let valid = marker_record.zip(target).zip(target_record).is_some_and(
            |((marker_record, target), _target_record)| {
                action.status == ActionStatus::Committed
                    && action.quantization == Quantization::Immediate
                    && matches!(action.undo_policy, UndoPolicy::NotUndoable { .. })
                    && action.result.as_ref().is_some_and(|result| result.accepted)
                    && marker_record.boundary.kind == CommitBoundary::Immediate
                    && target.status == ActionStatus::Undone
                    && target.command.has_typed_undo_semantics()
                    && matches!(target.undo_policy, UndoPolicy::Undoable)
                    && target.result.as_ref().is_some_and(|result| result.accepted)
                    && target
                        .committed_at
                        .zip(action.committed_at)
                        .is_some_and(|(target_at, marker_at)| append || target_at <= marker_at)
                    && latest_active_target == Some(target_action_id)
                    && resolved_targets.insert(target_action_id)
            },
        );
        if !valid {
            return Err(ReplayPlanError::InvalidUndoTargetRelation {
                undo_action_id: action.id,
                target_action_id: Some(target_action_id),
            });
        }
        candidates.pop();
    }
    if append
        && let Some(action) = action_log.actions.iter().find(|action| {
            action.status == ActionStatus::Undone && !resolved_targets.contains(&action.id)
        })
    {
        return Err(ReplayPlanError::UnresolvedUndoneAction {
            action_id: action.id,
        });
    }
    Ok(())
}
