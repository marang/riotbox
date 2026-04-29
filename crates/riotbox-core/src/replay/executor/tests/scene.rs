use super::*;
use crate::{
    action::{ActionTarget, TargetScope},
    ids::SceneId,
    replay::build_replay_target_plan,
    session::SessionFile,
};

#[test]
fn snapshot_suffix_replay_converges_with_origin_for_scene_launch_restore() {
    let action_log = action_log(vec![
        scene_action(1, ActionCommand::SceneLaunch, "scene-b", 100),
        scene_action(2, ActionCommand::SceneRestore, "scene-a", 200),
    ]);
    let snapshots = vec![snapshot("snap-after-launch", 1)];
    let origin_plan = build_replay_target_plan(&action_log, &[], 2).expect("origin plan");
    let anchor_plan = build_replay_target_plan(&action_log, &[], 1).expect("anchor plan");
    let snapshot_plan =
        build_replay_target_plan(&action_log, &snapshots, 2).expect("snapshot plan");

    let mut origin_session = scene_session("origin-session", "scene-a");
    let mut snapshot_session = scene_session("snapshot-session", "scene-a");

    apply_replay_plan_to_session(&mut origin_session, &origin_plan.suffix)
        .expect("origin replay succeeds");
    apply_replay_plan_to_session(&mut snapshot_session, &anchor_plan.suffix)
        .expect("anchor replay succeeds");
    apply_replay_plan_to_session(&mut snapshot_session, &snapshot_plan.suffix)
        .expect("snapshot suffix replay succeeds");

    assert_eq!(
        snapshot_plan
            .anchor
            .map(|snapshot| snapshot.snapshot_id.as_str()),
        Some("snap-after-launch")
    );
    assert_eq!(
        snapshot_session.runtime_state.transport.current_scene,
        origin_session.runtime_state.transport.current_scene
    );
    assert_eq!(
        snapshot_session.runtime_state.scene_state.active_scene,
        origin_session.runtime_state.scene_state.active_scene
    );
    assert_eq!(
        snapshot_session.runtime_state.scene_state.restore_scene,
        origin_session.runtime_state.scene_state.restore_scene
    );
    assert_eq!(
        origin_session.runtime_state.scene_state.active_scene,
        Some(SceneId::from("scene-a"))
    );
    assert_eq!(
        origin_session.runtime_state.scene_state.restore_scene,
        Some(SceneId::from("scene-b"))
    );
    assert_eq!(origin_session.runtime_state.scene_state.last_movement, None);
}

#[test]
fn scene_replay_rejects_missing_target_without_mutating_session() {
    let action_log = action_log(vec![targeted_action(
        1,
        ActionCommand::SceneLaunch,
        ActionParams::Empty,
        ActionTarget {
            scope: Some(TargetScope::Scene),
            ..Default::default()
        },
        100,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = scene_session("session-1", "scene-a");
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan.suffix)
        .expect_err("scene replay requires explicit scene id");

    assert_eq!(
        error,
        ReplayExecutionError::InvalidParams {
            action_id: ActionId(1),
            command: ActionCommand::SceneLaunch,
            expected: "ActionTarget { scene_id: Some(_) } or ActionParams::Scene { scene_id: Some(_) }",
        }
    );
    assert_eq!(session, original_session);
}

fn scene_session(session_id: &str, active_scene: &str) -> SessionFile {
    let mut session = SessionFile::new(session_id, "riotbox-test", "2026-04-29T21:45:00Z");
    let active_scene = SceneId::from(active_scene);
    session.runtime_state.scene_state.scenes =
        vec![SceneId::from("scene-a"), SceneId::from("scene-b")];
    session.runtime_state.scene_state.active_scene = Some(active_scene.clone());
    session.runtime_state.transport.current_scene = Some(active_scene);
    session
}

fn scene_action(
    id: u64,
    command: ActionCommand,
    scene_id: &str,
    committed_at: TimestampMs,
) -> Action {
    targeted_action(
        id,
        command,
        ActionParams::Scene {
            scene_id: Some(SceneId::from(scene_id)),
        },
        ActionTarget {
            scope: Some(TargetScope::Scene),
            scene_id: Some(SceneId::from(scene_id)),
            ..Default::default()
        },
        committed_at,
    )
}
