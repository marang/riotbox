//! Restore validation over transient indexes; Session remains the only history truth.
use super::JamAppError;
use riotbox_core::{
    action::{ActionCommand, ActionParams, ActionStatus},
    session::SessionFile,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub(super) fn validate_mvp_session_restore_contracts(
    session: &SessionFile,
) -> Result<(), JamAppError> {
    if session.source_refs.len() > 1 {
        return Err(JamAppError::InvalidSession(
            "Riotbox MVP currently supports exactly one source reference per session".into(),
        ));
    }

    if session.source_graph_refs.len() > 1 {
        return Err(JamAppError::InvalidSession(
            "Riotbox MVP currently supports exactly one source graph reference per session".into(),
        ));
    }

    if let (Some(source_ref), Some(graph_ref)) = (
        session.source_refs.first(),
        session.source_graph_refs.first(),
    ) && source_ref.source_id != graph_ref.source_id
    {
        return Err(JamAppError::InvalidSession(format!(
            "source ref {} does not match source graph ref {}",
            source_ref.source_id, graph_ref.source_id
        )));
    }

    let mut action_by_id = BTreeMap::new();
    for action in &session.action_log.actions {
        if action_by_id.insert(action.id, action).is_some() {
            return Err(JamAppError::InvalidSession(format!(
                "action log contains duplicate action id {}",
                action.id
            )));
        }
    }

    let action_count = session.action_log.actions.len();
    for snapshot in &session.snapshots {
        if snapshot.action_cursor > action_count {
            return Err(JamAppError::InvalidSession(format!(
                "snapshot {} action cursor {} exceeds action log length {}",
                snapshot.snapshot_id, snapshot.action_cursor, action_count
            )));
        }

        let Some(payload) = snapshot.payload.as_ref() else {
            continue;
        };

        if payload.snapshot_id != snapshot.snapshot_id {
            return Err(JamAppError::InvalidSession(format!(
                "snapshot {} payload snapshot id {} does not match owning snapshot",
                snapshot.snapshot_id, payload.snapshot_id
            )));
        }

        if payload.action_cursor != snapshot.action_cursor {
            return Err(JamAppError::InvalidSession(format!(
                "snapshot {} payload action cursor {} does not match snapshot action cursor {}",
                snapshot.snapshot_id, payload.action_cursor, snapshot.action_cursor
            )));
        }
    }

    for commit_record in &session.action_log.commit_records {
        let Some(action) = action_by_id.get(&commit_record.action_id) else {
            return Err(JamAppError::InvalidSession(format!(
                "commit record references missing action {}",
                commit_record.action_id
            )));
        };

        let has_valid_commit_history = action.status == ActionStatus::Committed
            || (action.status == ActionStatus::Undone && action.command.has_typed_undo_semantics());
        if !has_valid_commit_history {
            return Err(JamAppError::InvalidSession(format!(
                "commit record references action {} with invalid committed-history status {:?}",
                commit_record.action_id, action.status
            )));
        }

        let Some(action_committed_at) = action.committed_at else {
            return Err(JamAppError::InvalidSession(format!(
                "commit record references action {} without committed_at timestamp",
                commit_record.action_id
            )));
        };

        if commit_record.committed_at != action_committed_at {
            return Err(JamAppError::InvalidSession(format!(
                "commit record for action {} has committed_at {} but action has committed_at {}",
                commit_record.action_id, commit_record.committed_at, action_committed_at
            )));
        }

        if commit_record.commit_sequence == 0 {
            return Err(JamAppError::InvalidSession(format!(
                "commit record for action {} has invalid sequence 0",
                commit_record.action_id
            )));
        }
    }

    let mut first_record_by_action = BTreeMap::new();
    let mut first_record_by_boundary_sequence = HashMap::new();
    for (index, commit_record) in session.action_log.commit_records.iter().enumerate() {
        let key = (&commit_record.boundary, commit_record.commit_sequence);
        let action_duplicate = first_record_by_action.get(&commit_record.action_id);
        let boundary_duplicate = first_record_by_boundary_sequence.get(&key);
        // Preserve the old nested scan's first conflicting previous record,
        // including action-ID precedence when both conflicts share that record.
        if action_duplicate
            .is_some_and(|previous| boundary_duplicate.is_none_or(|other| previous <= other))
        {
            return Err(JamAppError::InvalidSession(format!(
                "commit record is duplicated for action {}",
                commit_record.action_id
            )));
        }
        if boundary_duplicate.is_some() {
            return Err(JamAppError::InvalidSession(format!(
                "commit record sequence {} is duplicated within boundary {:?} beat {} bar {} phrase {}",
                commit_record.commit_sequence,
                commit_record.boundary.kind,
                commit_record.boundary.beat_index,
                commit_record.boundary.bar_index,
                commit_record.boundary.phrase_index
            )));
        }
        first_record_by_action.insert(commit_record.action_id, index);
        first_record_by_boundary_sequence.insert(key, index);
    }

    riotbox_core::replay::build_committed_replay_plan(&session.action_log).map_err(|error| {
        JamAppError::InvalidSession(format!("action replay contract is invalid: {error:?}"))
    })?;

    let trusted_undo_targets: BTreeSet<_> = session
        .action_log
        .actions
        .iter()
        .filter(|marker| {
            marker.command == ActionCommand::UndoLast
                && marker.status == ActionStatus::Committed
                && marker.result.as_ref().is_some_and(|result| result.accepted)
                && first_record_by_action.contains_key(&marker.id)
        })
        .filter_map(|marker| match marker.params {
            ActionParams::Undo { target_action_id } => Some(target_action_id),
            _ => None,
        })
        .collect();

    for action in &session.action_log.actions {
        if action.status != ActionStatus::Undone
            || !matches!(
                action.command,
                ActionCommand::SourceMonitorSetMode | ActionCommand::Tr909FillNext
            )
        {
            continue;
        }
        if !trusted_undo_targets.contains(&action.id) {
            return Err(JamAppError::InvalidSession(format!(
                "legacy undone {} action {} has no trusted typed undo marker",
                action.command, action.id
            )));
        }
    }

    Ok(())
}
