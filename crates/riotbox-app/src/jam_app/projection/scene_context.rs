use riotbox_core::ids::SceneId;
use riotbox_core::session::{SceneMovementKindState, SceneMovementState, SessionFile};

pub(super) fn active_scene_movement(session: &SessionFile) -> Option<&SceneMovementState> {
    let movement = session
        .runtime_state
        .scene_state
        .active_projection_movement
        .as_ref()
        .or_else(|| {
            // Backward compatibility for sessions written before projection state
            // was persisted. A restore transition must never become the fallback
            // audio profile because it is an event, not the restored scene state.
            session
                .runtime_state
                .scene_state
                .last_movement
                .as_ref()
                .filter(|movement| movement.kind == SceneMovementKindState::Launch)
        })?;
    let active_scene = session
        .runtime_state
        .scene_state
        .active_scene
        .as_ref()
        .or(session.runtime_state.transport.current_scene.as_ref())?;
    (movement.to_scene == *active_scene).then_some(movement)
}

pub(super) fn scene_context(session: &SessionFile) -> Option<&SceneId> {
    session
        .runtime_state
        .transport
        .current_scene
        .as_ref()
        .or(session.runtime_state.scene_state.active_scene.as_ref())
}
