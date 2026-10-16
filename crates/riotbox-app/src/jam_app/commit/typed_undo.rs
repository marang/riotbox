//! One typed Session Undo operation, applied directly or after legacy admission.
use riotbox_core::{
    TimestampMs,
    action::{
        Action, ActionCommand, ActionParams, ActionResult, ActionStatus, ActionTarget, ActorType,
        CommitBoundary, Quantization, TargetScope, TypedUndoStateDomain, UndoPolicy,
    },
    ids::ActionId,
    queue::CommittedActionRef,
    session::SessionFile,
    transport::CommitBoundaryState,
};

use super::{admission::validate_legacy_history, record_committed_action_in_log};
use crate::jam_app::{
    JamAppError, JamAppState,
    helpers::{action_has_typed_undo_snapshot, normalize_missing_typed_undo_policies},
    is_mc202_phrase_action,
};

impl JamAppState {
    pub fn undo_last_action(
        &mut self,
        requested_at: TimestampMs,
    ) -> Result<Option<Action>, JamAppError> {
        let Some(undone_index) = latest_trusted_typed_undo_index(&self.session) else {
            if normalize_missing_typed_undo_policies(&mut self.session) {
                self.refresh_view();
            }
            return Ok(None);
        };
        let undo_boundary = CommitBoundaryState {
            kind: CommitBoundary::Immediate,
            beat_index: self.runtime.transport.beat_index,
            bar_index: self.runtime.transport.bar_index,
            phrase_index: self.runtime.transport.phrase_index,
            scene_id: self.runtime.transport.current_scene.clone(),
        };
        // Capacity precedes ID allocation, normalization and snapshot restoration.
        let mut sequences = self
            .session
            .action_log
            .prepare_commit_sequences(&undo_boundary, 1)?;
        let commit_sequence = sequences.next().ok_or_else(|| {
            JamAppError::InvalidSession("admitted Undo marker sequence missing".into())
        })?;
        let action = if self
            .session
            .action_log
            .replay_policy
            .uses_committed_append_order()
        {
            let marker_id = self.queue.allocate_action_id()?;
            apply_undo(
                &mut self.session,
                undone_index,
                &CommittedActionRef {
                    action_id: marker_id,
                    boundary: undo_boundary.clone(),
                    commit_sequence,
                },
                requested_at,
            )
        } else {
            let mut candidate_session = self.session.clone();
            let mut candidate_queue = self.queue.clone();
            let marker_id = candidate_queue.allocate_action_id()?;
            let Some(action) = apply_undo(
                &mut candidate_session,
                undone_index,
                &CommittedActionRef {
                    action_id: marker_id,
                    boundary: undo_boundary.clone(),
                    commit_sequence,
                },
                requested_at,
            ) else {
                return Ok(None);
            };
            validate_legacy_history(&candidate_session.action_log)?;
            self.session = candidate_session;
            self.queue = candidate_queue;
            Some(action)
        };
        if action.is_some() {
            self.runtime.last_commit_boundary = Some(undo_boundary);
            self.refresh_view();
        }
        Ok(action)
    }
}

/// Mutates only supplied Session state, with the exact already-allocated marker.
fn apply_undo(
    session: &mut SessionFile,
    undone_index: usize,
    committed_ref: &CommittedActionRef,
    requested_at: TimestampMs,
) -> Option<Action> {
    normalize_missing_typed_undo_policies(session);
    let undone_action_id = session.action_log.actions[undone_index].id;
    let undone_command = session.action_log.actions[undone_index].command;
    let is_mc202_undo = is_mc202_phrase_action(undone_command);
    let is_source_monitor_undo = undone_command == ActionCommand::SourceMonitorSetMode;
    let is_tr909_fill_undo = undone_command == ActionCommand::Tr909FillNext;
    let mc202_restored = is_mc202_undo && restore_mc202_undo_snapshot(session, undone_action_id);
    let source_monitor_restored =
        is_source_monitor_undo && restore_source_monitor_undo_snapshot(session, undone_action_id);
    let tr909_fill_restored =
        is_tr909_fill_undo && restore_tr909_fill_undo_snapshot(session, undone_action_id);
    if (is_mc202_undo && !mc202_restored)
        || (is_source_monitor_undo && !source_monitor_restored)
        || (is_tr909_fill_undo && !tr909_fill_restored)
    {
        return None;
    }
    let undo_summary = match (mc202_restored, source_monitor_restored, tr909_fill_restored) {
        (true, false, false) => {
            format!("undone by user at {requested_at}; restored MC-202 lane state")
        }
        (false, true, false) => {
            format!("undone by user at {requested_at}; restored source monitor mode")
        }
        (false, false, true) => {
            format!("undone by user at {requested_at}; restored TR-909 Fill state")
        }
        _ => format!("undone by user at {requested_at}"),
    };
    let undone = &mut session.action_log.actions[undone_index];
    undone.status = ActionStatus::Undone;
    undone.result = Some(ActionResult {
        accepted: true,
        summary: undo_summary,
    });
    let undo_action = Action {
        id: committed_ref.action_id,
        actor: ActorType::User,
        command: ActionCommand::UndoLast,
        params: ActionParams::Undo {
            target_action_id: undone_action_id,
        },
        target: ActionTarget {
            scope: Some(TargetScope::Session),
            ..Default::default()
        },
        requested_at,
        quantization: Quantization::Immediate,
        status: ActionStatus::Committed,
        committed_at: Some(requested_at),
        result: Some(ActionResult {
            accepted: true,
            summary: "undid most recent undoable action".into(),
        }),
        undo_policy: UndoPolicy::NotUndoable {
            reason: "undo marker actions are not themselves undoable".into(),
        },
        explanation: Some("undo most recent committed action".into()),
    };
    record_committed_action_in_log(
        &mut session.action_log,
        undo_action.clone(),
        committed_ref,
        requested_at,
    );
    Some(undo_action)
}

fn restore_mc202_undo_snapshot(session: &mut SessionFile, action_id: ActionId) -> bool {
    let Some(index) = session
        .runtime_state
        .undo_state
        .mc202_snapshots
        .iter()
        .rposition(|snapshot| snapshot.action_id == action_id)
    else {
        return false;
    };
    let snapshot = session
        .runtime_state
        .undo_state
        .mc202_snapshots
        .remove(index);
    snapshot.apply_to_session(session);
    true
}

fn restore_source_monitor_undo_snapshot(session: &mut SessionFile, action_id: ActionId) -> bool {
    let Some(index) = session
        .runtime_state
        .undo_state
        .source_monitor_snapshots
        .iter()
        .rposition(|snapshot| snapshot.action_id == action_id)
    else {
        return false;
    };
    let snapshot = session
        .runtime_state
        .undo_state
        .source_monitor_snapshots
        .remove(index);
    snapshot.apply_to_session(session);
    true
}

fn restore_tr909_fill_undo_snapshot(session: &mut SessionFile, action_id: ActionId) -> bool {
    let Some(index) = session
        .runtime_state
        .undo_state
        .tr909_fill_snapshots
        .iter()
        .rposition(|snapshot| snapshot.action_id == action_id)
    else {
        return false;
    };
    let snapshot = session
        .runtime_state
        .undo_state
        .tr909_fill_snapshots
        .remove(index);
    snapshot.apply_to_session(session);
    true
}

fn latest_trusted_typed_undo_index(session: &SessionFile) -> Option<usize> {
    let domains = [
        TypedUndoStateDomain::Mc202Lane,
        TypedUndoStateDomain::SourceMonitorMode,
        TypedUndoStateDomain::Tr909FillWindow,
    ];
    let mut blocked_domains = std::collections::BTreeSet::new();
    for (index, action) in session.action_log.actions.iter().enumerate().rev() {
        let committed_effect_may_exist = action.status == ActionStatus::Committed
            && !action
                .result
                .as_ref()
                .is_some_and(|result| !result.accepted);
        if let Some(domain) = action.command.typed_undo_state_domain()
            && !blocked_domains.contains(&domain)
            && matches!(action.undo_policy, UndoPolicy::Undoable)
            && action.result.as_ref().is_some_and(|result| result.accepted)
            && session
                .action_log
                .commit_records
                .iter()
                .filter(|record| record.action_id == action.id)
                .count()
                == 1
            && action_has_typed_undo_snapshot(session, action)
        {
            return Some(index);
        }
        if committed_effect_may_exist {
            blocked_domains.extend(
                domains
                    .into_iter()
                    .filter(|domain| action.command.mutates_typed_undo_state_domain(*domain)),
            );
        }
    }
    None
}
