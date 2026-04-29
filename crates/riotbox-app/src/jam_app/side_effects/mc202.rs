use super::*;

pub(in crate::jam_app) fn apply_mc202_side_effects(
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
