use riotbox_core::{
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionResult, ActionTarget, ActorType,
        Quantization, TargetScope,
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
