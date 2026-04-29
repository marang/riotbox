use super::*;

pub(in crate::jam_app) fn apply_scene_side_effects(
    session: &mut SessionFile,
    action: &Action,
    boundary: Option<&CommitBoundaryState>,
    source_graph: Option<&SourceGraph>,
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
    session.runtime_state.scene_state.last_movement = scene_movement_state(
        action,
        boundary,
        previous_scene.as_ref(),
        &scene_id,
        source_graph,
    );

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

fn scene_movement_state(
    action: &Action,
    boundary: Option<&CommitBoundaryState>,
    from_scene: Option<&riotbox_core::ids::SceneId>,
    to_scene: &riotbox_core::ids::SceneId,
    source_graph: Option<&SourceGraph>,
) -> Option<SceneMovementState> {
    let boundary = boundary?;
    let kind = match action.command {
        ActionCommand::SceneLaunch => SceneMovementKindState::Launch,
        ActionCommand::SceneRestore => SceneMovementKindState::Restore,
        _ => return None,
    };
    let direction = scene_movement_direction(from_scene, to_scene, source_graph)?;

    Some(SceneMovementState {
        action_id: action.id,
        from_scene: from_scene.cloned(),
        to_scene: to_scene.clone(),
        kind,
        direction,
        tr909_intent: scene_movement_tr909_intent(direction),
        mc202_intent: scene_movement_mc202_intent(direction),
        intensity: scene_movement_intensity(direction),
        committed_bar_index: boundary.bar_index,
        committed_phrase_index: boundary.phrase_index,
    })
}

fn scene_movement_direction(
    from_scene: Option<&riotbox_core::ids::SceneId>,
    to_scene: &riotbox_core::ids::SceneId,
    source_graph: Option<&SourceGraph>,
) -> Option<SceneMovementDirectionState> {
    let graph = source_graph?;
    let from = from_scene
        .and_then(|scene_id| section_for_projected_scene(graph, scene_id))
        .map(|section| energy_rank(section.energy_class))?;
    let to = section_for_projected_scene(graph, to_scene)
        .map(|section| energy_rank(section.energy_class))?;

    Some(match to.cmp(&from) {
        std::cmp::Ordering::Greater => SceneMovementDirectionState::Rise,
        std::cmp::Ordering::Less => SceneMovementDirectionState::Drop,
        std::cmp::Ordering::Equal => SceneMovementDirectionState::Hold,
    })
}

const fn energy_rank(energy: EnergyClass) -> u8 {
    match energy {
        EnergyClass::Unknown | EnergyClass::Low => 0,
        EnergyClass::Medium => 1,
        EnergyClass::High => 2,
        EnergyClass::Peak => 3,
    }
}

const fn scene_movement_tr909_intent(
    direction: SceneMovementDirectionState,
) -> SceneMovementLaneIntentState {
    match direction {
        SceneMovementDirectionState::Rise => SceneMovementLaneIntentState::Drive,
        SceneMovementDirectionState::Drop => SceneMovementLaneIntentState::Release,
        SceneMovementDirectionState::Hold => SceneMovementLaneIntentState::Anchor,
    }
}

const fn scene_movement_mc202_intent(
    direction: SceneMovementDirectionState,
) -> SceneMovementLaneIntentState {
    match direction {
        SceneMovementDirectionState::Rise => SceneMovementLaneIntentState::Lift,
        SceneMovementDirectionState::Drop | SceneMovementDirectionState::Hold => {
            SceneMovementLaneIntentState::Anchor
        }
    }
}

const fn scene_movement_intensity(direction: SceneMovementDirectionState) -> f32 {
    match direction {
        SceneMovementDirectionState::Rise => 0.75,
        SceneMovementDirectionState::Drop => 0.55,
        SceneMovementDirectionState::Hold => 0.35,
    }
}
