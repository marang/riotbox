use riotbox_core::{
    action::{Action, ActionCommand, ActionParams, ActionResult},
    ids::CaptureId,
    session::{
        Mc202PhraseVariantState, SessionFile, Tr909ReinforcementModeState,
        Tr909TakeoverProfileState, W30PreviewModeState,
    },
    transport::CommitBoundaryState,
};

use super::JamAppState;

pub(super) fn apply_w30_side_effects(
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

pub(super) fn apply_tr909_side_effects(
    session: &mut SessionFile,
    action: &Action,
    boundary: Option<&CommitBoundaryState>,
) {
    match action.command {
        ActionCommand::Tr909SetSlam => {
            let intensity = match &action.params {
                ActionParams::Mutation { intensity, .. } => intensity.clamp(0.0, 1.0),
                _ => session.runtime_state.macro_state.tr909_slam,
            };
            session.runtime_state.macro_state.tr909_slam = intensity;
            session.runtime_state.lane_state.tr909.slam_enabled = intensity > 0.0;

            if let Some(logged_action) = session
                .action_log
                .actions
                .iter_mut()
                .rev()
                .find(|logged_action| logged_action.id == action.id)
            {
                let summary = if intensity > 0.0 {
                    format!("enabled TR-909 slam at {:.2}", intensity)
                } else {
                    "disabled TR-909 slam".into()
                };
                logged_action.result = Some(ActionResult {
                    accepted: true,
                    summary,
                });
            }
        }
        ActionCommand::Tr909FillNext => {
            session.runtime_state.lane_state.tr909.fill_armed_next_bar = false;
            session.runtime_state.lane_state.tr909.last_fill_bar =
                boundary.map(|boundary| boundary.bar_index);
            session.runtime_state.lane_state.tr909.pattern_ref =
                boundary.map(|boundary| format!("fill-bar-{}", boundary.bar_index));
            session.runtime_state.lane_state.tr909.reinforcement_mode =
                Some(Tr909ReinforcementModeState::Fills);
        }
        ActionCommand::Tr909ReinforceBreak => {
            session.runtime_state.lane_state.tr909.reinforcement_mode =
                Some(Tr909ReinforcementModeState::BreakReinforce);
            session.runtime_state.lane_state.tr909.pattern_ref = boundary.map(|boundary| {
                boundary.scene_id.as_ref().map_or_else(
                    || format!("reinforce-phrase-{}", boundary.phrase_index),
                    |scene_id| format!("reinforce-{scene_id}"),
                )
            });
        }
        ActionCommand::Tr909Takeover => {
            session.runtime_state.lane_state.tr909.takeover_enabled = true;
            session.runtime_state.lane_state.tr909.takeover_profile =
                Some(Tr909TakeoverProfileState::ControlledPhraseTakeover);
            session.runtime_state.lane_state.tr909.pattern_ref = boundary.map(|boundary| {
                boundary.scene_id.as_ref().map_or_else(
                    || format!("takeover-phrase-{}", boundary.phrase_index),
                    |scene_id| format!("takeover-{scene_id}"),
                )
            });
            session.runtime_state.lane_state.tr909.reinforcement_mode =
                Some(Tr909ReinforcementModeState::Takeover);
        }
        ActionCommand::Tr909SceneLock => {
            session.runtime_state.lane_state.tr909.takeover_enabled = true;
            session.runtime_state.lane_state.tr909.takeover_profile =
                Some(Tr909TakeoverProfileState::SceneLockTakeover);
            session.runtime_state.lane_state.tr909.pattern_ref = boundary.map(|boundary| {
                boundary.scene_id.as_ref().map_or_else(
                    || format!("lock-phrase-{}", boundary.phrase_index),
                    |scene_id| format!("lock-{scene_id}"),
                )
            });
            session.runtime_state.lane_state.tr909.reinforcement_mode =
                Some(Tr909ReinforcementModeState::Takeover);
        }
        ActionCommand::Tr909Release => {
            session.runtime_state.lane_state.tr909.takeover_enabled = false;
            session.runtime_state.lane_state.tr909.takeover_profile = None;
            session.runtime_state.lane_state.tr909.pattern_ref = boundary.map(|boundary| {
                boundary.scene_id.as_ref().map_or_else(
                    || format!("release-phrase-{}", boundary.phrase_index),
                    |scene_id| format!("release-{scene_id}"),
                )
            });
            if session.runtime_state.lane_state.tr909.reinforcement_mode
                == Some(Tr909ReinforcementModeState::Takeover)
            {
                session.runtime_state.lane_state.tr909.reinforcement_mode =
                    Some(Tr909ReinforcementModeState::SourceSupport);
            }
        }
        _ => {}
    }
}

pub(super) fn apply_mc202_side_effects(
    session: &mut SessionFile,
    action: &Action,
    boundary: Option<&CommitBoundaryState>,
) {
    match action.command {
        ActionCommand::Mc202SetRole => {
            let Some(role) = action
                .target
                .object_id
                .clone()
                .or_else(|| match &action.params {
                    ActionParams::Mutation { target_id, .. } => target_id.clone(),
                    _ => None,
                })
            else {
                return;
            };

            session.runtime_state.lane_state.mc202.role = Some(role.clone());
            session.runtime_state.lane_state.mc202.phrase_ref =
                Some(boundary_phrase_ref(boundary, &role));
            session.runtime_state.lane_state.mc202.phrase_variant = None;

            let touch = match &action.params {
                ActionParams::Mutation { intensity, .. } => intensity.clamp(0.0, 1.0),
                _ if role == "leader" => 0.85,
                _ => 0.65,
            };
            session.runtime_state.macro_state.mc202_touch = touch;

            if let Some(logged_action) = session
                .action_log
                .actions
                .iter_mut()
                .rev()
                .find(|logged_action| logged_action.id == action.id)
            {
                logged_action.result = Some(ActionResult {
                    accepted: true,
                    summary: format!("set MC-202 role to {role} at {touch:.2}"),
                });
            }
        }
        ActionCommand::Mc202GenerateFollower => {
            let role = "follower";
            let phrase_ref = boundary_phrase_ref(boundary, role);
            let touch = match &action.params {
                ActionParams::Mutation { intensity, .. } => intensity.clamp(0.0, 1.0),
                _ => 0.78,
            };

            session.runtime_state.lane_state.mc202.role = Some(role.into());
            session.runtime_state.lane_state.mc202.phrase_ref = Some(phrase_ref.clone());
            session.runtime_state.lane_state.mc202.phrase_variant = None;
            session.runtime_state.macro_state.mc202_touch =
                session.runtime_state.macro_state.mc202_touch.max(touch);

            if let Some(logged_action) = session
                .action_log
                .actions
                .iter_mut()
                .rev()
                .find(|logged_action| logged_action.id == action.id)
            {
                logged_action.result = Some(ActionResult {
                    accepted: true,
                    summary: format!(
                        "generated MC-202 follower phrase {phrase_ref} at {:.2}",
                        session.runtime_state.macro_state.mc202_touch
                    ),
                });
            }
        }
        ActionCommand::Mc202GenerateAnswer => {
            let role = "answer";
            let phrase_ref = boundary_phrase_ref(boundary, role);
            let touch = match &action.params {
                ActionParams::Mutation { intensity, .. } => intensity.clamp(0.0, 1.0),
                _ => 0.82,
            };

            session.runtime_state.lane_state.mc202.role = Some(role.into());
            session.runtime_state.lane_state.mc202.phrase_ref = Some(phrase_ref.clone());
            session.runtime_state.lane_state.mc202.phrase_variant = None;
            session.runtime_state.macro_state.mc202_touch =
                session.runtime_state.macro_state.mc202_touch.max(touch);

            if let Some(logged_action) = session
                .action_log
                .actions
                .iter_mut()
                .rev()
                .find(|logged_action| logged_action.id == action.id)
            {
                logged_action.result = Some(ActionResult {
                    accepted: true,
                    summary: format!(
                        "generated MC-202 answer phrase {phrase_ref} at {:.2}",
                        session.runtime_state.macro_state.mc202_touch
                    ),
                });
            }
        }
        ActionCommand::Mc202GeneratePressure => {
            let role = "pressure";
            let phrase_ref = boundary_phrase_ref(boundary, role);
            let touch = match &action.params {
                ActionParams::Mutation { intensity, .. } => intensity.clamp(0.0, 1.0),
                _ => 0.84,
            };

            session.runtime_state.lane_state.mc202.role = Some(role.into());
            session.runtime_state.lane_state.mc202.phrase_ref = Some(phrase_ref.clone());
            session.runtime_state.lane_state.mc202.phrase_variant = None;
            session.runtime_state.macro_state.mc202_touch =
                session.runtime_state.macro_state.mc202_touch.max(touch);

            if let Some(logged_action) = session
                .action_log
                .actions
                .iter_mut()
                .rev()
                .find(|logged_action| logged_action.id == action.id)
            {
                logged_action.result = Some(ActionResult {
                    accepted: true,
                    summary: format!(
                        "generated MC-202 pressure phrase {phrase_ref} at {:.2}",
                        session.runtime_state.macro_state.mc202_touch
                    ),
                });
            }
        }
        ActionCommand::Mc202GenerateInstigator => {
            let role = "instigator";
            let phrase_ref = boundary_phrase_ref(boundary, role);
            let touch = match &action.params {
                ActionParams::Mutation { intensity, .. } => intensity.clamp(0.0, 1.0),
                _ => 0.90,
            };

            session.runtime_state.lane_state.mc202.role = Some(role.into());
            session.runtime_state.lane_state.mc202.phrase_ref = Some(phrase_ref.clone());
            session.runtime_state.lane_state.mc202.phrase_variant = None;
            session.runtime_state.macro_state.mc202_touch =
                session.runtime_state.macro_state.mc202_touch.max(touch);

            if let Some(logged_action) = session
                .action_log
                .actions
                .iter_mut()
                .rev()
                .find(|logged_action| logged_action.id == action.id)
            {
                logged_action.result = Some(ActionResult {
                    accepted: true,
                    summary: format!(
                        "generated MC-202 instigator phrase {phrase_ref} at {:.2}",
                        session.runtime_state.macro_state.mc202_touch
                    ),
                });
            }
        }
        ActionCommand::Mc202MutatePhrase => {
            let current_role = session
                .runtime_state
                .lane_state
                .mc202
                .role
                .clone()
                .unwrap_or_else(|| "follower".into());
            let variant = match &action.params {
                ActionParams::Mutation {
                    target_id: Some(target_id),
                    ..
                } if target_id == "mutated_drive" => target_id.clone(),
                _ => "mutated_drive".into(),
            };
            let bar_index = boundary.map_or(0, |boundary| boundary.bar_index).max(1);
            let phrase_ref = format!("{}-{variant}-bar-{}", current_role, bar_index);
            let touch = match &action.params {
                ActionParams::Mutation { intensity, .. } => intensity.clamp(0.0, 1.0),
                _ => 0.88,
            };

            session.runtime_state.lane_state.mc202.role = Some(current_role.clone());
            session.runtime_state.lane_state.mc202.phrase_ref = Some(phrase_ref.clone());
            session.runtime_state.lane_state.mc202.phrase_variant =
                Some(Mc202PhraseVariantState::MutatedDrive);
            session.runtime_state.macro_state.mc202_touch =
                session.runtime_state.macro_state.mc202_touch.max(touch);

            if let Some(logged_action) = session
                .action_log
                .actions
                .iter_mut()
                .rev()
                .find(|logged_action| logged_action.id == action.id)
            {
                logged_action.result = Some(ActionResult {
                    accepted: true,
                    summary: format!("mutated MC-202 phrase {phrase_ref} as {variant}"),
                });
            }
        }
        _ => {}
    }
}

pub(super) fn apply_scene_side_effects(
    session: &mut SessionFile,
    action: &Action,
    boundary: Option<&CommitBoundaryState>,
) {
    if !matches!(
        action.command,
        ActionCommand::SceneLaunch | ActionCommand::SceneRestore
    ) {
        return;
    }

    let Some(scene_id) = action
        .target
        .scene_id
        .clone()
        .or_else(|| match &action.params {
            ActionParams::Scene {
                scene_id: Some(scene_id),
            } => Some(scene_id.clone()),
            _ => None,
        })
    else {
        return;
    };

    let previous_scene = session
        .runtime_state
        .scene_state
        .active_scene
        .clone()
        .or_else(|| session.runtime_state.transport.current_scene.clone());

    session.runtime_state.scene_state.active_scene = Some(scene_id.clone());
    session.runtime_state.transport.current_scene = Some(scene_id.clone());
    session.runtime_state.scene_state.restore_scene = previous_scene
        .as_ref()
        .filter(|previous_scene| **previous_scene != scene_id)
        .cloned();

    if let Some(logged_action) = session
        .action_log
        .actions
        .iter_mut()
        .rev()
        .find(|logged_action| logged_action.id == action.id)
    {
        let position = boundary.map_or_else(
            || "pending scene boundary".to_string(),
            |boundary| {
                format!(
                    "bar {} / phrase {}",
                    boundary.bar_index, boundary.phrase_index
                )
            },
        );
        let verb = match action.command {
            ActionCommand::SceneLaunch => "launched",
            ActionCommand::SceneRestore => "restored",
            _ => unreachable!("scene side effects only handle launch and restore"),
        };
        let target_kind = if action
            .explanation
            .as_deref()
            .is_some_and(|explanation| explanation.contains("contrast scene"))
        {
            "contrast scene"
        } else {
            "scene"
        };
        logged_action.result = Some(ActionResult {
            accepted: true,
            summary: format!("{verb} {target_kind} {scene_id} at {position}"),
        });
    }
}

fn boundary_phrase_ref(boundary: Option<&CommitBoundaryState>, role: &str) -> String {
    boundary.map_or_else(
        || format!("{role}-phrase"),
        |boundary| {
            boundary.scene_id.as_ref().map_or_else(
                || format!("{role}-phrase-{}", boundary.phrase_index),
                |scene_id| format!("{role}-{scene_id}"),
            )
        },
    )
}
