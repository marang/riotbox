use riotbox_core::{
    action::{Action, ActionCommand, ActionParams, ActionResult},
    replay::derive_scene_movement_state,
    session::SessionFile,
    source_graph::SourceGraph,
    transport::CommitBoundaryState,
};

pub(in crate::jam_app) fn apply_scene_side_effects(
    session: &mut SessionFile,
    action: &Action,
    boundary: Option<&CommitBoundaryState>,
    source_graph: Option<&SourceGraph>,
) {
    if matches!(action.command, ActionCommand::MutateScene) {
        apply_scene_mutation_side_effects(session, action, boundary);
        return;
    }

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
    session.runtime_state.scene_state.last_movement = boundary.and_then(|boundary| {
        source_graph.and_then(|source_graph| {
            derive_scene_movement_state(
                action,
                boundary,
                previous_scene.as_ref(),
                &scene_id,
                source_graph,
            )
        })
    });

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

fn apply_scene_mutation_side_effects(
    session: &mut SessionFile,
    action: &Action,
    boundary: Option<&CommitBoundaryState>,
) {
    let ActionParams::Mutation {
        intensity,
        target_id,
    } = &action.params
    else {
        return;
    };

    let previous_aggression = session.runtime_state.macro_state.scene_aggression;
    let next_aggression = mutated_scene_aggression(previous_aggression, *intensity);
    session.runtime_state.macro_state.scene_aggression = next_aggression;

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
        let scene_label = target_id
            .as_deref()
            .or_else(|| {
                action
                    .target
                    .scene_id
                    .as_ref()
                    .map(|scene_id| scene_id.as_str())
            })
            .unwrap_or("current scene");
        logged_action.result = Some(ActionResult {
            accepted: true,
            summary: format!(
                "mutated scene {scene_label} at {position}; aggression {:.2} -> {:.2}",
                previous_aggression, next_aggression
            ),
        });
    }
}

fn mutated_scene_aggression(previous_aggression: f32, intensity: f32) -> f32 {
    let bump = intensity.clamp(0.0, 1.0).max(0.10) * 0.25;
    (previous_aggression + bump).clamp(0.0, 1.0)
}
