use super::*;

pub(in crate::jam_app) fn apply_w30_side_effects(
    session: &mut SessionFile,
    action: &Action,
    boundary: Option<&CommitBoundaryState>,
) {
    if !matches!(
        action.command,
        ActionCommand::W30LiveRecall
            | ActionCommand::W30SwapBank
            | ActionCommand::W30BrowseSlicePool
            | ActionCommand::W30ApplyDamageProfile
            | ActionCommand::W30LoopFreeze
            | ActionCommand::W30StepFocus
            | ActionCommand::W30AuditionRawCapture
            | ActionCommand::W30AuditionPromoted
            | ActionCommand::W30TriggerPad
    ) {
        return;
    }

    let Some(bank_id) = action.target.bank_id.clone() else {
        return;
    };
    let Some(pad_id) = action.target.pad_id.clone() else {
        return;
    };
    let source_capture_id = match &action.params {
        ActionParams::Promotion {
            capture_id: Some(capture_id),
            ..
        } => Some(capture_id.clone()),
        _ => None,
    };
    let capture_id = match &action.params {
        ActionParams::Mutation {
            target_id: Some(target_id),
            ..
        } => Some(CaptureId::from(target_id.clone())),
        ActionParams::Promotion { .. } if action.command == ActionCommand::W30LoopFreeze => {
            session.runtime_state.lane_state.w30.last_capture.clone()
        }
        _ => None,
    };

    session.runtime_state.lane_state.w30.active_bank = Some(bank_id.clone());
    session.runtime_state.lane_state.w30.focused_pad = Some(pad_id.clone());
    session.runtime_state.lane_state.w30.preview_mode = Some(match action.command {
        ActionCommand::W30AuditionRawCapture => W30PreviewModeState::RawCaptureAudition,
        ActionCommand::W30AuditionPromoted => W30PreviewModeState::PromotedAudition,
        ActionCommand::W30ApplyDamageProfile => session
            .runtime_state
            .lane_state
            .w30
            .preview_mode
            .unwrap_or(W30PreviewModeState::LiveRecall),
        ActionCommand::W30LiveRecall
        | ActionCommand::W30SwapBank
        | ActionCommand::W30BrowseSlicePool
        | ActionCommand::W30LoopFreeze
        | ActionCommand::W30StepFocus
        | ActionCommand::W30TriggerPad => W30PreviewModeState::LiveRecall,
        _ => unreachable!("checked above"),
    });
    if let Some(capture_id) = capture_id.clone() {
        session.runtime_state.lane_state.w30.last_capture = Some(capture_id);
    }
    if matches!(
        action.command,
        ActionCommand::W30AuditionRawCapture
            | ActionCommand::W30AuditionPromoted
            | ActionCommand::W30TriggerPad
            | ActionCommand::W30ApplyDamageProfile
    ) {
        let grit = match &action.params {
            ActionParams::Mutation { intensity, .. } => match action.command {
                ActionCommand::W30AuditionRawCapture => intensity.clamp(0.0, 1.0),
                ActionCommand::W30AuditionPromoted => intensity.clamp(0.0, 1.0),
                ActionCommand::W30TriggerPad => (intensity * 0.82).clamp(0.0, 1.0),
                ActionCommand::W30ApplyDamageProfile => intensity.clamp(0.0, 1.0),
                _ => unreachable!("checked above"),
            },
            _ => 0.68,
        };
        session.runtime_state.macro_state.w30_grit =
            session.runtime_state.macro_state.w30_grit.max(grit);
    }

    if let Some(logged_action) = session
        .action_log
        .actions
        .iter_mut()
        .rev()
        .find(|logged_action| logged_action.id == action.id)
    {
        let summary = match action.command {
            ActionCommand::W30StepFocus => {
                let position = boundary.map_or_else(
                    || "beat pending".to_string(),
                    |boundary| {
                        format!(
                            "beat {} / phrase {}",
                            boundary.beat_index, boundary.phrase_index
                        )
                    },
                );
                format!("focused W-30 pad {bank_id}/{pad_id} at {position}")
            }
            ActionCommand::W30LiveRecall => capture_id.as_ref().map_or_else(
                || format!("recalled W-30 pad {bank_id}/{pad_id}"),
                |capture_id| format!("recalled {capture_id} on W-30 pad {bank_id}/{pad_id}"),
            ),
            ActionCommand::W30SwapBank => capture_id.as_ref().map_or_else(
                || format!("swapped W-30 bank to {bank_id}/{pad_id}"),
                |capture_id| format!("swapped W-30 bank to {bank_id}/{pad_id} with {capture_id}"),
            ),
            ActionCommand::W30BrowseSlicePool => {
                let position = boundary.map_or_else(
                    || "beat pending".to_string(),
                    |boundary| {
                        format!(
                            "beat {} / phrase {}",
                            boundary.beat_index, boundary.phrase_index
                        )
                    },
                );
                capture_id.as_ref().map_or_else(
                    || format!("browsed W-30 slice pool on {bank_id}/{pad_id} at {position}"),
                    |capture_id| {
                        format!(
                            "browsed W-30 slice pool to {capture_id} on {bank_id}/{pad_id} at {position}"
                        )
                    },
                )
            }
            ActionCommand::W30ApplyDamageProfile => capture_id.as_ref().map_or_else(
                || {
                    format!(
                        "applied {} damage profile on W-30 pad {bank_id}/{pad_id}",
                        JamAppState::W30_DAMAGE_PROFILE_LABEL
                    )
                },
                |capture_id| {
                    format!(
                        "applied {} damage profile to {capture_id} on W-30 pad {bank_id}/{pad_id}",
                        JamAppState::W30_DAMAGE_PROFILE_LABEL
                    )
                },
            ),
            ActionCommand::W30AuditionRawCapture => capture_id.as_ref().map_or_else(
                || format!("auditioned raw capture on W-30 preview {bank_id}/{pad_id}"),
                |capture_id| {
                    format!("auditioned raw {capture_id} on W-30 preview {bank_id}/{pad_id}")
                },
            ),
            ActionCommand::W30AuditionPromoted => capture_id.as_ref().map_or_else(
                || format!("auditioned W-30 pad {bank_id}/{pad_id}"),
                |capture_id| format!("auditioned {capture_id} on W-30 pad {bank_id}/{pad_id}"),
            ),
            ActionCommand::W30LoopFreeze => match (source_capture_id.as_ref(), capture_id.as_ref())
            {
                (Some(source_capture_id), Some(capture_id)) => format!(
                    "froze {source_capture_id} into {capture_id} for W-30 reuse on {bank_id}/{pad_id}"
                ),
                (None, Some(capture_id)) => {
                    format!("froze {capture_id} for W-30 reuse on {bank_id}/{pad_id}")
                }
                _ => format!("froze W-30 reuse on {bank_id}/{pad_id}"),
            },
            ActionCommand::W30TriggerPad => {
                let position = boundary.map_or_else(
                    || "beat pending".to_string(),
                    |boundary| {
                        format!(
                            "beat {} / phrase {}",
                            boundary.beat_index, boundary.phrase_index
                        )
                    },
                );
                capture_id.as_ref().map_or_else(
                    || format!("triggered W-30 pad {bank_id}/{pad_id} at {position}"),
                    |capture_id| {
                        format!(
                            "triggered {capture_id} on W-30 pad {bank_id}/{pad_id} at {position}"
                        )
                    },
                )
            }
            _ => unreachable!("checked above"),
        };
        logged_action.result = Some(ActionResult {
            accepted: true,
            summary,
        });
    }
}
