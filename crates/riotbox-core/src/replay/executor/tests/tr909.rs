use super::*;
use crate::{
    replay::build_replay_target_plan,
    session::{SessionFile, Tr909ReinforcementModeState, Tr909TakeoverProfileState},
};

#[test]
fn snapshot_suffix_replay_converges_with_origin_for_tr909_support_moves() {
    let action_log = action_log(vec![
        action(
            1,
            ActionCommand::Tr909SetSlam,
            ActionParams::Mutation {
                intensity: 0.87,
                target_id: Some("enabled".into()),
            },
            100,
        ),
        action(2, ActionCommand::Tr909FillNext, ActionParams::Empty, 200),
        action(
            3,
            ActionCommand::Tr909Takeover,
            ActionParams::Mutation {
                intensity: 1.0,
                target_id: Some("takeover".into()),
            },
            300,
        ),
        action(
            4,
            ActionCommand::Tr909Release,
            ActionParams::Mutation {
                intensity: 0.0,
                target_id: Some("release".into()),
            },
            400,
        ),
    ]);
    let snapshots = vec![snapshot("snap-after-fill", 2)];
    let origin_plan = build_replay_target_plan(&action_log, &[], 4).expect("origin plan");
    let anchor_plan = build_replay_target_plan(&action_log, &[], 2).expect("anchor plan");
    let snapshot_plan =
        build_replay_target_plan(&action_log, &snapshots, 4).expect("snapshot plan");

    let mut origin_session =
        SessionFile::new("origin-session", "riotbox-test", "2026-04-29T21:05:00Z");
    let mut snapshot_session =
        SessionFile::new("snapshot-session", "riotbox-test", "2026-04-29T21:05:00Z");

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
        Some("snap-after-fill")
    );
    assert_eq!(
        snapshot_session.runtime_state.lane_state.tr909,
        origin_session.runtime_state.lane_state.tr909
    );
    assert_eq!(
        snapshot_session.runtime_state.macro_state.tr909_slam,
        origin_session.runtime_state.macro_state.tr909_slam
    );
    assert_eq!(origin_session.runtime_state.macro_state.tr909_slam, 0.87);
    assert!(
        !origin_session
            .runtime_state
            .lane_state
            .tr909
            .takeover_enabled
    );
    assert_eq!(
        origin_session
            .runtime_state
            .lane_state
            .tr909
            .takeover_profile,
        None
    );
    assert_eq!(
        origin_session
            .runtime_state
            .lane_state
            .tr909
            .pattern_ref
            .as_deref(),
        Some("release-phrase-1")
    );
    assert_eq!(
        origin_session.runtime_state.lane_state.tr909.last_fill_bar,
        Some(2)
    );
    assert_eq!(
        origin_session
            .runtime_state
            .lane_state
            .tr909
            .reinforcement_mode,
        Some(Tr909ReinforcementModeState::SourceSupport)
    );
}

#[test]
fn tr909_scene_lock_and_reinforce_use_boundary_context() {
    let action_log = action_log(vec![
        action(
            1,
            ActionCommand::Tr909ReinforceBreak,
            ActionParams::Empty,
            100,
        ),
        action(
            2,
            ActionCommand::Tr909SceneLock,
            ActionParams::Mutation {
                intensity: 1.0,
                target_id: Some("scene_lock".into()),
            },
            200,
        ),
    ]);
    let plan = build_replay_target_plan(&action_log, &[], 2).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T21:05:00Z");

    apply_replay_plan_to_session(&mut session, &plan.suffix).expect("TR-909 replay succeeds");

    assert!(session.runtime_state.lane_state.tr909.takeover_enabled);
    assert_eq!(
        session.runtime_state.lane_state.tr909.takeover_profile,
        Some(Tr909TakeoverProfileState::SceneLockTakeover)
    );
    assert_eq!(
        session
            .runtime_state
            .lane_state
            .tr909
            .pattern_ref
            .as_deref(),
        Some("lock-phrase-0")
    );
    assert_eq!(
        session.runtime_state.lane_state.tr909.reinforcement_mode,
        Some(Tr909ReinforcementModeState::Takeover)
    );
}

#[test]
fn tr909_slam_rejects_missing_mutation_params_without_mutating_session() {
    let action_log = action_log(vec![action(
        1,
        ActionCommand::Tr909SetSlam,
        ActionParams::Empty,
        100,
    )]);
    let plan = build_replay_target_plan(&action_log, &[], 1).expect("origin plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T21:05:00Z");
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan.suffix)
        .expect_err("TR-909 slam requires explicit mutation params");

    assert_eq!(
        error,
        ReplayExecutionError::InvalidParams {
            action_id: ActionId(1),
            command: ActionCommand::Tr909SetSlam,
            expected: "ActionParams::Mutation { intensity, .. }"
        }
    );
    assert_eq!(session, original_session);
}
