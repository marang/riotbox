//! Transient cursor answers, built only after the public history-validation gate.
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    action::{ActionCommand, ActionParams, ActionStatus},
    ids::ActionId,
    session::ActionLog,
};

pub(super) enum UndoCursorSafety {
    AllSafe,
    Indexed(Vec<Option<ActionId>>),
}

impl UndoCursorSafety {
    pub(super) fn new(action_log: &ActionLog) -> Self {
        if !action_log
            .actions
            .iter()
            .any(|action| action.status == ActionStatus::Undone)
        {
            return Self::AllSafe;
        }

        let committed_markers: BTreeSet<_> = action_log
            .commit_records
            .iter()
            .map(|record| record.action_id)
            .collect();
        let mut pending_by_id = BTreeMap::new();
        let mut pending_positions = BTreeSet::new();
        let mut answers = Vec::with_capacity(action_log.actions.len() + 1);
        answers.push(None); // Cursor zero precedes every action.

        for (position, action) in action_log.actions.iter().enumerate() {
            // Safety includes unresolved legacy entries, not only the typed
            // validator's accepted/undoable stack. IDs are already unique.
            if action.status == ActionStatus::Undone {
                pending_by_id.insert(action.id, position);
                pending_positions.insert(position);
            }
            if action.command == ActionCommand::UndoLast
                && action.status == ActionStatus::Committed
                && action.result.as_ref().is_some_and(|result| result.accepted)
                && committed_markers.contains(&action.id)
                && let ActionParams::Undo { target_action_id } = &action.params
                && let Some(target_position) = pending_by_id.remove(target_action_id)
            {
                // Only prior targets can have entered pending_by_id. Untyped
                // markers never resolve safety, regardless of validator stack.
                pending_positions.remove(&target_position);
            }
            answers.push(
                pending_positions
                    .first()
                    .map(|index| action_log.actions[*index].id),
            );
        }
        Self::Indexed(answers)
    }

    /// Caller validates the cursor before querying; position, not ID, orders errors.
    pub(super) fn first_unresolved(&self, cursor: usize) -> Option<ActionId> {
        match self {
            Self::AllSafe => None,
            Self::Indexed(answers) => answers[cursor],
        }
    }
}
