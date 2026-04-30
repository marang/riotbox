use super::ReplayExecutionError;
use crate::{
    action::{ActionCommand, ActionParams},
    ids::{BankId, PadId},
    replay::{
        ReplayPlanEntry, W30ArtifactReplayHydrationError, plan_w30_artifact_replay_hydration,
    },
    session::{CaptureTarget, SessionFile, W30PreviewModeState},
};

pub(super) fn apply_promote_capture_to_w30_pad(
    session: &mut SessionFile,
    entry: &ReplayPlanEntry<'_>,
) -> Result<(), ReplayExecutionError> {
    let action = entry.action;
    let ActionParams::Promotion {
        capture_id: Some(capture_id),
        ..
    } = &action.params
    else {
        return Err(ReplayExecutionError::InvalidParams {
            action_id: action.id,
            command: action.command,
            expected: "ActionParams::Promotion { capture_id: Some(_) }",
        });
    };
    let Some(bank_id) = action.target.bank_id.clone() else {
        return Err(ReplayExecutionError::InvalidParams {
            action_id: action.id,
            command: action.command,
            expected: "ActionTarget { bank_id: Some(_), pad_id: Some(_) }",
        });
    };
    let Some(pad_id) = action.target.pad_id.clone() else {
        return Err(ReplayExecutionError::InvalidParams {
            action_id: action.id,
            command: action.command,
            expected: "ActionTarget { bank_id: Some(_), pad_id: Some(_) }",
        });
    };
    let Some(capture) = session
        .captures
        .iter_mut()
        .find(|capture| capture.capture_id == *capture_id)
    else {
        return Err(ReplayExecutionError::InvalidParams {
            action_id: action.id,
            command: action.command,
            expected: "existing CaptureRef for ActionParams::Promotion.capture_id",
        });
    };

    let target = CaptureTarget::W30Pad {
        bank_id: bank_id.clone(),
        pad_id: pad_id.clone(),
    };
    capture.assigned_target = Some(target.clone());
    capture.notes = Some(updated_capture_promotion_note(
        capture.notes.as_deref(),
        &target,
    ));

    session.runtime_state.lane_state.w30.active_bank = Some(bank_id);
    session.runtime_state.lane_state.w30.focused_pad = Some(pad_id);
    session.runtime_state.lane_state.w30.last_capture = Some(capture_id.clone());
    session.runtime_state.lane_state.w30.preview_mode = Some(W30PreviewModeState::LiveRecall);

    Ok(())
}

pub(super) fn apply_promote_capture_to_scene(
    session: &mut SessionFile,
    entry: &ReplayPlanEntry<'_>,
) -> Result<(), ReplayExecutionError> {
    let action = entry.action;
    let ActionParams::Promotion {
        capture_id: Some(capture_id),
        ..
    } = &action.params
    else {
        return Err(ReplayExecutionError::InvalidParams {
            action_id: action.id,
            command: action.command,
            expected: "ActionParams::Promotion { capture_id: Some(_) }",
        });
    };
    let Some(scene_id) = action.target.scene_id.clone() else {
        return Err(ReplayExecutionError::InvalidParams {
            action_id: action.id,
            command: action.command,
            expected: "ActionTarget { scene_id: Some(_) }",
        });
    };
    let Some(capture) = session
        .captures
        .iter_mut()
        .find(|capture| capture.capture_id == *capture_id)
    else {
        return Err(ReplayExecutionError::InvalidParams {
            action_id: action.id,
            command: action.command,
            expected: "existing CaptureRef for ActionParams::Promotion.capture_id",
        });
    };

    let target = CaptureTarget::Scene(scene_id);
    capture.assigned_target = Some(target.clone());
    capture.notes = Some(updated_capture_promotion_note(
        capture.notes.as_deref(),
        &target,
    ));
    session.runtime_state.lane_state.w30.last_capture = Some(capture_id.clone());

    Ok(())
}

pub(super) fn apply_w30_artifact_hydrated_cue(
    session: &mut SessionFile,
    entry: &ReplayPlanEntry<'_>,
) -> Result<(), ReplayExecutionError> {
    let action = entry.action;
    let hydration = plan_w30_artifact_replay_hydration(session, entry).map_err(|reason| {
        ReplayExecutionError::ArtifactHydration {
            action_id: action.id,
            command: action.command,
            reason,
        }
    })?;
    let source_capture_id = hydration.source_capture_id.clone();
    let capture = session
        .captures
        .iter()
        .find(|capture| capture.capture_id == hydration.produced_capture_id)
        .ok_or(ReplayExecutionError::ArtifactHydration {
            action_id: action.id,
            command: action.command,
            reason: W30ArtifactReplayHydrationError::MissingProducedCapture {
                action_id: action.id,
                command: action.command,
            },
        })?;
    let (bank_id, pad_id) = w30_artifact_target(session, capture, action, &source_capture_id)?;

    session.runtime_state.lane_state.w30.active_bank = Some(bank_id);
    session.runtime_state.lane_state.w30.focused_pad = Some(pad_id);
    session.runtime_state.lane_state.w30.preview_mode = Some(W30PreviewModeState::LiveRecall);
    session.runtime_state.lane_state.w30.last_capture = Some(hydration.produced_capture_id);
    session.runtime_state.macro_state.w30_grit = session
        .runtime_state
        .macro_state
        .w30_grit
        .max(w30_grit_or(action, 0.78));

    Ok(())
}

pub(super) fn apply_w30_cue(
    session: &mut SessionFile,
    entry: &ReplayPlanEntry<'_>,
) -> Result<(), ReplayExecutionError> {
    let action = entry.action;
    let bank_id = action
        .target
        .bank_id
        .clone()
        .ok_or(ReplayExecutionError::InvalidParams {
            action_id: action.id,
            command: action.command,
            expected: "ActionTarget { bank_id: Some(_), pad_id: Some(_) }",
        })?;
    let pad_id = action
        .target
        .pad_id
        .clone()
        .ok_or(ReplayExecutionError::InvalidParams {
            action_id: action.id,
            command: action.command,
            expected: "ActionTarget { bank_id: Some(_), pad_id: Some(_) }",
        })?;

    let preview_mode = match action.command {
        ActionCommand::W30AuditionRawCapture => W30PreviewModeState::RawCaptureAudition,
        ActionCommand::W30AuditionPromoted => W30PreviewModeState::PromotedAudition,
        ActionCommand::W30ApplyDamageProfile => session
            .runtime_state
            .lane_state
            .w30
            .preview_mode
            .unwrap_or(W30PreviewModeState::LiveRecall),
        ActionCommand::W30LiveRecall
        | ActionCommand::W30TriggerPad
        | ActionCommand::W30SwapBank
        | ActionCommand::W30BrowseSlicePool
        | ActionCommand::W30StepFocus => W30PreviewModeState::LiveRecall,
        _ => unreachable!("checked by caller"),
    };

    let capture_id = match &action.params {
        ActionParams::Empty if action.command == ActionCommand::W30StepFocus => None,
        _ if action.command == ActionCommand::W30StepFocus => {
            return Err(ReplayExecutionError::InvalidParams {
                action_id: action.id,
                command: action.command,
                expected: "ActionParams::Empty",
            });
        }
        ActionParams::Mutation {
            target_id: Some(target_id),
            ..
        } => Some(crate::ids::CaptureId::from(target_id.clone())),
        _ => {
            return Err(ReplayExecutionError::InvalidParams {
                action_id: action.id,
                command: action.command,
                expected: "ActionParams::Mutation { target_id: Some(_), .. }",
            });
        }
    };

    session.runtime_state.lane_state.w30.active_bank = Some(bank_id);
    session.runtime_state.lane_state.w30.focused_pad = Some(pad_id);
    session.runtime_state.lane_state.w30.preview_mode = Some(preview_mode);
    if let Some(capture_id) = capture_id {
        session.runtime_state.lane_state.w30.last_capture = Some(capture_id);
    }

    if matches!(
        action.command,
        ActionCommand::W30AuditionRawCapture
            | ActionCommand::W30AuditionPromoted
            | ActionCommand::W30TriggerPad
            | ActionCommand::W30ApplyDamageProfile
    ) {
        session.runtime_state.macro_state.w30_grit = session
            .runtime_state
            .macro_state
            .w30_grit
            .max(w30_grit_or(action, 0.68));
    }

    Ok(())
}

fn w30_artifact_target(
    session: &SessionFile,
    capture: &crate::session::CaptureRef,
    action: &crate::action::Action,
    source_capture_id: &crate::ids::CaptureId,
) -> Result<(BankId, PadId), ReplayExecutionError> {
    if let Some(CaptureTarget::W30Pad { bank_id, pad_id }) = capture.assigned_target.as_ref() {
        return Ok((bank_id.clone(), pad_id.clone()));
    }

    if let (Some(bank_id), Some(pad_id)) =
        (action.target.bank_id.clone(), action.target.pad_id.clone())
    {
        return Ok((bank_id, pad_id));
    }

    if let Some((bank_id, pad_id)) = session
        .captures
        .iter()
        .find(|capture| capture.capture_id == *source_capture_id)
        .and_then(|capture| match capture.assigned_target.as_ref() {
            Some(CaptureTarget::W30Pad { bank_id, pad_id }) => {
                Some((bank_id.clone(), pad_id.clone()))
            }
            _ => None,
        })
    {
        return Ok((bank_id, pad_id));
    }

    if let (Some(bank_id), Some(pad_id)) = (
        session.runtime_state.lane_state.w30.active_bank.clone(),
        session.runtime_state.lane_state.w30.focused_pad.clone(),
    ) {
        return Ok((bank_id, pad_id));
    }

    Err(ReplayExecutionError::InvalidParams {
        action_id: action.id,
        command: action.command,
        expected: "produced/source CaptureTarget::W30Pad, ActionTarget { bank_id: Some(_), pad_id: Some(_) }, or existing W-30 focus",
    })
}

fn updated_capture_promotion_note(existing_notes: Option<&str>, target: &CaptureTarget) -> String {
    let promotion = match target {
        CaptureTarget::W30Pad { bank_id, pad_id } => {
            format!("promoted to pad {bank_id}/{pad_id}")
        }
        CaptureTarget::Scene(scene_id) => format!("promoted to scene {scene_id}"),
    };
    match existing_notes {
        Some(existing_notes) => {
            let base = existing_notes
                .split(" | promoted to ")
                .next()
                .unwrap_or(existing_notes);
            format!("{base} | {promotion}")
        }
        None => promotion,
    }
}

fn w30_grit_or(action: &crate::action::Action, fallback: f32) -> f32 {
    match &action.params {
        ActionParams::Mutation { intensity, .. } => match action.command {
            ActionCommand::W30TriggerPad => (intensity * 0.82).clamp(0.0, 1.0),
            _ => intensity.clamp(0.0, 1.0),
        },
        _ => fallback,
    }
}
