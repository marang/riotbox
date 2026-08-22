use super::{ReplayExecutionError, ReplayExecutionReport, ReplayPlanEntry};
use crate::{
    action::{Action, ActionCommand, ActionParams},
    ids::SceneId,
    session::{
        SceneMovementDirectionState, SceneMovementKindState, SceneMovementLaneIntentState,
        SceneMovementState, SceneMovementW30IntentState, SceneState, SessionFile,
    },
    source_graph::{EnergyClass, SourceGraph, section_for_projected_scene},
    transport::CommitBoundaryState,
};

pub fn apply_graph_aware_replay_plan_to_session(
    session: &mut SessionFile,
    entries: &[ReplayPlanEntry<'_>],
    source_graph: &SourceGraph,
) -> Result<ReplayExecutionReport, ReplayExecutionError> {
    let mut working = session.clone();
    let mut applied_action_ids = Vec::with_capacity(entries.len());

    for entry in entries {
        let scene_movement = derive_scene_movement_for_replay_entry(&working, entry, source_graph);
        super::apply_replay_entry_to_session(&mut working, entry)?;
        if matches!(
            entry.action.command,
            ActionCommand::SceneLaunch | ActionCommand::SceneRestore
        ) {
            if entry.action.command == ActionCommand::SceneLaunch {
                // The graph-free executor has already preserved the previous projection
                // in the restore slot. Hydrate only the new launch projection here.
                working.runtime_state.scene_state.active_projection_movement =
                    scene_movement.clone();
            }
            working.runtime_state.scene_state.last_movement = scene_movement;
        }
        applied_action_ids.push(entry.action.id);
    }

    *session = working;
    Ok(ReplayExecutionReport { applied_action_ids })
}

/// Moves the persistent lane-audio projection alongside the Scene restore pointer.
///
/// `last_movement` remains the latest transition/anchor event. The projection is
/// separate because restoring a scene must restore its former lane state rather
/// than treating the inverse transition as a new persistent musical profile.
pub fn apply_scene_audio_projection_transition(
    scene_state: &mut SceneState,
    command: ActionCommand,
    launch_movement: Option<SceneMovementState>,
    target_scene: &SceneId,
) {
    if !matches!(
        command,
        ActionCommand::SceneLaunch | ActionCommand::SceneRestore
    ) {
        return;
    }
    let previous_projection = scene_state.active_projection_movement.take();
    match command {
        ActionCommand::SceneLaunch => {
            scene_state.active_projection_movement =
                launch_movement.filter(|movement| movement.to_scene == *target_scene);
            scene_state.restore_projection_movement = previous_projection;
        }
        ActionCommand::SceneRestore => {
            scene_state.active_projection_movement = scene_state
                .restore_projection_movement
                .take()
                .filter(|movement| movement.to_scene == *target_scene);
            scene_state.restore_projection_movement = previous_projection;
        }
        _ => {}
    }
}

pub fn derive_scene_movement_for_replay_entry(
    session: &SessionFile,
    entry: &ReplayPlanEntry<'_>,
    source_graph: &SourceGraph,
) -> Option<SceneMovementState> {
    let scene_id = scene_target_from_action(entry.action)?;
    let previous_scene = session
        .runtime_state
        .scene_state
        .active_scene
        .as_ref()
        .or(session.runtime_state.transport.current_scene.as_ref());

    derive_scene_movement_state(
        entry.action,
        &entry.commit_record.boundary,
        previous_scene,
        scene_id,
        source_graph,
    )
}

pub fn derive_scene_movement_state(
    action: &Action,
    boundary: &CommitBoundaryState,
    from_scene: Option<&SceneId>,
    to_scene: &SceneId,
    source_graph: &SourceGraph,
) -> Option<SceneMovementState> {
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
        w30_intent: SceneMovementW30IntentState::Pin,
        intensity: scene_movement_intensity(direction),
        committed_bar_index: boundary.bar_index,
        committed_phrase_index: boundary.phrase_index,
    })
}

fn scene_target_from_action(action: &Action) -> Option<&SceneId> {
    action.target.scene_id.as_ref().or(match &action.params {
        ActionParams::Scene {
            scene_id: Some(scene_id),
        } => Some(scene_id),
        _ => None,
    })
}

fn scene_movement_direction(
    from_scene: Option<&SceneId>,
    to_scene: &SceneId,
    source_graph: &SourceGraph,
) -> Option<SceneMovementDirectionState> {
    let from = from_scene
        .and_then(|scene_id| section_for_projected_scene(source_graph, scene_id))
        .map(|section| energy_rank(section.energy_class))?;
    let to = section_for_projected_scene(source_graph, to_scene)
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
