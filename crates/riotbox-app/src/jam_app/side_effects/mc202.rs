use riotbox_core::{
    action::{Action, ActionCommand, ActionParams, ActionResult},
    session::{Mc202PhraseIntentState, Mc202RoleState, SessionFile},
    transport::CommitBoundaryState,
};

pub(in crate::jam_app) fn apply_mc202_side_effects(
    session: &mut SessionFile,
    action: &Action,
    boundary: Option<&CommitBoundaryState>,
) {
    match action.command {
        ActionCommand::Mc202SetRole => {
            let Some(role_label) =
                action
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
            let Some(role) = Mc202RoleState::from_label(&role_label) else {
                set_logged_mc202_result(
                    session,
                    action,
                    false,
                    format!("rejected unknown MC-202 role {role_label}"),
                );
                return;
            };
            let role_label = role.label();

            session.runtime_state.lane_state.mc202.role = Some(role_label.into());
            session.runtime_state.lane_state.mc202.phrase_ref =
                Some(boundary_phrase_ref(boundary, role_label));
            session.runtime_state.lane_state.mc202.phrase_variant = None;

            let touch = match &action.params {
                ActionParams::Mutation { intensity, .. } => intensity.clamp(0.0, 1.0),
                _ => mc202_set_role_default_touch(role),
            };
            session.runtime_state.macro_state.mc202_touch = touch;

            set_logged_mc202_result(
                session,
                action,
                true,
                format!("set MC-202 role to {role_label} at {touch:.2}"),
            );
        }
        ActionCommand::Mc202GenerateFollower => {
            apply_generated_role(session, action, boundary, Mc202RoleState::Follower);
        }
        ActionCommand::Mc202GenerateAnswer => {
            apply_generated_role(session, action, boundary, Mc202RoleState::Answer);
        }
        ActionCommand::Mc202GeneratePressure => {
            apply_generated_role(session, action, boundary, Mc202RoleState::Pressure);
        }
        ActionCommand::Mc202GenerateInstigator => {
            apply_generated_role(session, action, boundary, Mc202RoleState::Instigator);
        }
        ActionCommand::Mc202MutatePhrase => {
            let current_role_label = session
                .runtime_state
                .lane_state
                .mc202
                .role
                .clone()
                .unwrap_or_else(|| "follower".into());
            let Some(current_role) = Mc202RoleState::from_label(&current_role_label) else {
                set_logged_mc202_result(
                    session,
                    action,
                    false,
                    format!("rejected unknown MC-202 role {current_role_label}"),
                );
                return;
            };
            let Some(intent) = mc202_phrase_intent_from_action(action) else {
                set_logged_mc202_result(
                    session,
                    action,
                    false,
                    "rejected unknown MC-202 phrase intent",
                );
                return;
            };
            let current_role_label = current_role.label();
            let variant = intent.label();
            let bar_index = boundary.map_or(0, |boundary| boundary.bar_index).max(1);
            let phrase_ref = format!("{current_role_label}-{variant}-bar-{bar_index}");
            let touch = match &action.params {
                ActionParams::Mutation { intensity, .. } => intensity.clamp(0.0, 1.0),
                _ => 0.88,
            };

            session.runtime_state.lane_state.mc202.role = Some(current_role_label.into());
            session.runtime_state.lane_state.mc202.phrase_ref = Some(phrase_ref.clone());
            session.runtime_state.lane_state.mc202.phrase_variant = intent.phrase_variant();
            session.runtime_state.macro_state.mc202_touch =
                session.runtime_state.macro_state.mc202_touch.max(touch);

            set_logged_mc202_result(
                session,
                action,
                true,
                format!("mutated MC-202 phrase {phrase_ref} as {variant}"),
            );
        }
        _ => {}
    }
}

fn apply_generated_role(
    session: &mut SessionFile,
    action: &Action,
    boundary: Option<&CommitBoundaryState>,
    role: Mc202RoleState,
) {
    let role_label = role.label();
    let phrase_ref = boundary_phrase_ref(boundary, role_label);
    let touch = match &action.params {
        ActionParams::Mutation { intensity, .. } => intensity.clamp(0.0, 1.0),
        _ => role.default_touch(),
    };

    session.runtime_state.lane_state.mc202.role = Some(role_label.into());
    session.runtime_state.lane_state.mc202.phrase_ref = Some(phrase_ref.clone());
    session.runtime_state.lane_state.mc202.phrase_variant = None;
    session.runtime_state.macro_state.mc202_touch =
        session.runtime_state.macro_state.mc202_touch.max(touch);

    set_logged_mc202_result(
        session,
        action,
        true,
        format!(
            "generated MC-202 {role_label} phrase {phrase_ref} at {:.2}",
            session.runtime_state.macro_state.mc202_touch
        ),
    );
}

fn mc202_set_role_default_touch(role: Mc202RoleState) -> f32 {
    if role == Mc202RoleState::Leader {
        0.85
    } else {
        0.65
    }
}

fn mc202_phrase_intent_from_action(action: &Action) -> Option<Mc202PhraseIntentState> {
    match &action.params {
        ActionParams::Mutation {
            target_id: Some(target_id),
            ..
        } => match Mc202PhraseIntentState::from_label(target_id) {
            Some(Mc202PhraseIntentState::MutatedDrive) => {
                Some(Mc202PhraseIntentState::MutatedDrive)
            }
            _ => None,
        },
        _ => Some(Mc202PhraseIntentState::MutatedDrive),
    }
}

fn set_logged_mc202_result(
    session: &mut SessionFile,
    action: &Action,
    accepted: bool,
    summary: impl Into<String>,
) {
    if let Some(logged_action) = session
        .action_log
        .actions
        .iter_mut()
        .rev()
        .find(|logged_action| logged_action.id == action.id)
    {
        logged_action.result = Some(ActionResult {
            accepted,
            summary: summary.into(),
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
