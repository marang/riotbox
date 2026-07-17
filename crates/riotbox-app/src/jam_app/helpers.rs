use riotbox_core::{
    action::{
        Action, ActionCommand, ActionDraft, ActionParams, ActionResult, ActionStatus, ActionTarget,
        ActorType, Quantization, TargetScope, UndoPolicy,
    },
    ids::ActionId,
    session::{CaptureRef, SessionFile},
};

pub(in crate::jam_app) fn is_mc202_phrase_action(command: ActionCommand) -> bool {
    matches!(
        command,
        ActionCommand::Mc202SetRole
            | ActionCommand::Mc202GenerateFollower
            | ActionCommand::Mc202GenerateAnswer
            | ActionCommand::Mc202GeneratePressure
            | ActionCommand::Mc202GenerateInstigator
            | ActionCommand::Mc202MutatePhrase
    )
}

pub(in crate::jam_app) fn append_capture_note(capture: &mut CaptureRef, detail: &str) {
    capture.notes = Some(match capture.notes.as_deref() {
        Some(existing) if !existing.is_empty() => format!("{existing} | {detail}"),
        _ => detail.into(),
    });
}

pub(in crate::jam_app) fn max_action_id(session: &SessionFile) -> Option<ActionId> {
    session
        .action_log
        .actions
        .iter()
        .map(|action| action.id)
        .max()
}

pub(in crate::jam_app) fn action_has_typed_undo_snapshot(
    session: &SessionFile,
    action: &Action,
) -> bool {
    if is_mc202_phrase_action(action.command) {
        return session
            .runtime_state
            .undo_state
            .mc202_snapshots
            .iter()
            .any(|snapshot| snapshot.action_id == action.id);
    }
    if action.command == ActionCommand::SourceMonitorSetMode {
        return session
            .runtime_state
            .undo_state
            .source_monitor_snapshots
            .iter()
            .any(|snapshot| snapshot.action_id == action.id);
    }
    if action.command == ActionCommand::Tr909FillNext {
        return session
            .runtime_state
            .undo_state
            .tr909_fill_snapshots
            .iter()
            .any(|snapshot| snapshot.action_id == action.id);
    }
    false
}

pub(in crate::jam_app) fn normalize_missing_typed_undo_policies(session: &mut SessionFile) -> bool {
    let mc202_snapshot_ids = session
        .runtime_state
        .undo_state
        .mc202_snapshots
        .iter()
        .map(|snapshot| snapshot.action_id)
        .collect::<std::collections::BTreeSet<_>>();
    let monitor_snapshot_ids = session
        .runtime_state
        .undo_state
        .source_monitor_snapshots
        .iter()
        .map(|snapshot| snapshot.action_id)
        .collect::<std::collections::BTreeSet<_>>();
    let fill_snapshot_ids = session
        .runtime_state
        .undo_state
        .tr909_fill_snapshots
        .iter()
        .map(|snapshot| snapshot.action_id)
        .collect::<std::collections::BTreeSet<_>>();
    let commit_record_counts = session.action_log.commit_records.iter().fold(
        std::collections::BTreeMap::<ActionId, usize>::new(),
        |mut counts, record| {
            *counts.entry(record.action_id).or_default() += 1;
            counts
        },
    );

    let mut changed = false;
    for action in &mut session.action_log.actions {
        if action.status != ActionStatus::Committed
            || !matches!(action.undo_policy, UndoPolicy::Undoable)
            || !action.command.has_typed_undo_semantics()
        {
            continue;
        }

        let invalid_contract_reason =
            if !action.result.as_ref().is_some_and(|result| result.accepted) {
                Some("commit has no accepted persisted result")
            } else if commit_record_counts.get(&action.id).copied() != Some(1) {
                Some("commit does not have exactly one persisted commit record")
            } else {
                None
            };

        let missing_snapshot = if is_mc202_phrase_action(action.command) {
            !mc202_snapshot_ids.contains(&action.id)
        } else {
            match action.command {
                ActionCommand::SourceMonitorSetMode => !monitor_snapshot_ids.contains(&action.id),
                ActionCommand::Tr909FillNext => !fill_snapshot_ids.contains(&action.id),
                _ => false,
            }
        };
        if let Some(reason) = invalid_contract_reason {
            action.undo_policy = UndoPolicy::NotUndoable {
                reason: reason.into(),
            };
            changed = true;
        } else if missing_snapshot {
            action.undo_policy = UndoPolicy::NotUndoable {
                reason: "commit has no persisted typed pre-state snapshot".into(),
            };
            changed = true;
        }
    }
    changed
}

pub(in crate::jam_app) fn update_logged_action_result(
    session: &mut SessionFile,
    action_id: ActionId,
    summary: impl Into<String>,
) {
    if let Some(logged_action) = session
        .action_log
        .actions
        .iter_mut()
        .rev()
        .find(|logged_action| logged_action.id == action_id)
    {
        logged_action.result = Some(ActionResult {
            accepted: true,
            summary: summary.into(),
        });
    }
}

pub(in crate::jam_app) fn user_lane_mutation_draft(
    command: ActionCommand,
    quantization: Quantization,
    scope: TargetScope,
    target_id: impl Into<String>,
    intensity: f32,
    explanation: impl Into<String>,
) -> ActionDraft {
    let target_id = target_id.into();
    let mut draft = ActionDraft::new(
        ActorType::User,
        command,
        quantization,
        ActionTarget {
            scope: Some(scope),
            object_id: Some(target_id.clone()),
            ..Default::default()
        },
    );
    draft.params = ActionParams::Mutation {
        intensity,
        target_id: Some(target_id),
    };
    draft.explanation = Some(explanation.into());
    draft
}
