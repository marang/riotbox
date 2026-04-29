use super::*;
use crate::{
    action::TargetScope,
    replay::{build_committed_replay_plan, build_replay_target_plan},
    session::SessionFile,
};

#[test]
fn snapshot_suffix_replay_converges_with_origin_for_mc202_phrase_actions() {
    let action_log = action_log(vec![
        action(
            1,
            ActionCommand::Mc202GenerateFollower,
            ActionParams::Mutation {
                intensity: 0.76,
                target_id: None,
            },
            100,
        ),
        action(
            2,
            ActionCommand::Mc202GenerateAnswer,
            ActionParams::Mutation {
                intensity: 0.83,
                target_id: None,
            },
            200,
        ),
        action(
            3,
            ActionCommand::Mc202MutatePhrase,
            ActionParams::Mutation {
                intensity: 0.91,
                target_id: Some("mutated_drive".into()),
            },
            300,
        ),
    ]);
    let snapshots = vec![snapshot("snap-after-follower", 1)];
    let origin_plan = build_replay_target_plan(&action_log, &[], 3).expect("origin plan");
    let anchor_plan = build_replay_target_plan(&action_log, &[], 1).expect("anchor plan");
    let snapshot_plan =
        build_replay_target_plan(&action_log, &snapshots, 3).expect("snapshot plan");

    let mut origin_session =
        SessionFile::new("origin-session", "riotbox-test", "2026-04-29T20:30:00Z");
    let mut snapshot_session =
        SessionFile::new("snapshot-session", "riotbox-test", "2026-04-29T20:30:00Z");

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
        Some("snap-after-follower")
    );
    assert_eq!(
        snapshot_session.runtime_state.lane_state.mc202,
        origin_session.runtime_state.lane_state.mc202
    );
    assert_eq!(
        snapshot_session.runtime_state.macro_state.mc202_touch,
        origin_session.runtime_state.macro_state.mc202_touch
    );
    assert_eq!(
        origin_session
            .runtime_state
            .lane_state
            .mc202
            .phrase_ref
            .as_deref(),
        Some("answer-mutated_drive-bar-3")
    );
    assert_eq!(
        origin_session.runtime_state.lane_state.mc202.phrase_variant,
        Some(crate::session::Mc202PhraseVariantState::MutatedDrive)
    );
}

#[test]
fn mc202_set_role_uses_explicit_role_target() {
    let action_log = action_log(vec![targeted_action(
        1,
        ActionCommand::Mc202SetRole,
        ActionParams::Empty,
        ActionTarget {
            scope: Some(TargetScope::LaneMc202),
            object_id: Some("leader".into()),
            ..Default::default()
        },
        100,
    )]);
    let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T20:00:00Z");

    apply_replay_plan_to_session(&mut session, &plan).expect("role replay succeeds");

    assert_eq!(
        session.runtime_state.lane_state.mc202.role.as_deref(),
        Some("leader")
    );
    assert_eq!(
        session.runtime_state.lane_state.mc202.phrase_ref.as_deref(),
        Some("leader-phrase-0")
    );
    assert_eq!(session.runtime_state.macro_state.mc202_touch, 0.85);
}

#[test]
fn mc202_set_role_rejects_missing_role_target_without_mutating_session() {
    let action_log = action_log(vec![action(
        1,
        ActionCommand::Mc202SetRole,
        ActionParams::Empty,
        100,
    )]);
    let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T20:00:00Z");
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan).expect_err("missing role target");

    assert_eq!(
        error,
        ReplayExecutionError::InvalidParams {
            action_id: ActionId(1),
            command: ActionCommand::Mc202SetRole,
            expected: "ActionTarget::object_id or ActionParams::Mutation { target_id: Some(_) }"
        }
    );
    assert_eq!(session, original_session);
}
