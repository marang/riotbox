//! Pure compatibility adapter; the source selector and its inputs stay unchanged.
use riotbox_core::{
    action::{ActionCommand, ActionParams},
    replay::ReplayPlanEntry,
    session::{Mc202RoleState, Mc202SourcePhrasePlanState, SessionFile},
    source_graph::SourceGraph,
};

use super::{
    MC202_MUTATION_DEFAULT_TOUCH, mc202_phrase_intent_from_action, mc202_set_role_default_touch,
    source_phrase::derive_mc202_source_phrase_plan,
};

pub(in crate::jam_app) fn derive_legacy_mc202_source_phrase_plan(
    session: &SessionFile,
    entry: &ReplayPlanEntry<'_>,
    source_graph: Option<&SourceGraph>,
) -> Option<Mc202SourcePhrasePlanState> {
    let action = entry.action;
    let (role, default_touch, explicit_mutation) = match action.command {
        ActionCommand::Mc202SetRole => {
            let label = action
                .target
                .object_id
                .as_deref()
                .or(match &action.params {
                    ActionParams::Mutation { target_id, .. } => target_id.as_deref(),
                    _ => None,
                })?;
            let role = Mc202RoleState::from_label(label)?;
            (role, mc202_set_role_default_touch(role), false)
        }
        ActionCommand::Mc202GenerateFollower => (
            Mc202RoleState::Follower,
            Mc202RoleState::Follower.default_touch(),
            false,
        ),
        ActionCommand::Mc202GenerateAnswer => (
            Mc202RoleState::Answer,
            Mc202RoleState::Answer.default_touch(),
            false,
        ),
        ActionCommand::Mc202GeneratePressure => (
            Mc202RoleState::Pressure,
            Mc202RoleState::Pressure.default_touch(),
            false,
        ),
        ActionCommand::Mc202GenerateInstigator => (
            Mc202RoleState::Instigator,
            Mc202RoleState::Instigator.default_touch(),
            false,
        ),
        ActionCommand::Mc202MutatePhrase => {
            mc202_phrase_intent_from_action(action)?;
            (
                session
                    .runtime_state
                    .lane_state
                    .mc202
                    .role
                    .unwrap_or(Mc202RoleState::Follower),
                MC202_MUTATION_DEFAULT_TOUCH,
                true,
            )
        }
        _ => return None,
    };
    let touch = match &action.params {
        ActionParams::Mutation { intensity, .. } => intensity.clamp(0.0, 1.0),
        _ => default_touch,
    };
    let effective_touch = if action.command == ActionCommand::Mc202SetRole {
        touch
    } else {
        session.runtime_state.macro_state.mc202_touch.max(touch)
    };
    derive_mc202_source_phrase_plan(
        session,
        source_graph,
        Some(&entry.commit_record.boundary),
        role,
        effective_touch,
        explicit_mutation,
    )
    .ok()
    .flatten()
}
