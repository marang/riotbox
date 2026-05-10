use riotbox_core::{
    action::ActionCommand,
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

pub(in crate::jam_app) fn next_action_id_from_session(session: &SessionFile) -> ActionId {
    ActionId(
        max_action_id(session)
            .map(|id| id.0.saturating_add(1))
            .unwrap_or(1),
    )
}

pub(in crate::jam_app) fn max_action_id(session: &SessionFile) -> Option<ActionId> {
    session
        .action_log
        .actions
        .iter()
        .map(|action| action.id)
        .max()
}
