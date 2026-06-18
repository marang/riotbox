use super::*;
use crate::{
    action::TargetScope,
    ids::SourceId,
    replay::{build_committed_replay_plan, build_replay_target_plan},
    session::{
        Mc202RoleState, Mc202SourcePhraseCandidateFamilyState, Mc202SourcePhraseNoteBudgetState,
        Mc202SourcePhrasePlanState, Mc202SourcePhraseSlotState, SessionFile,
    },
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
fn snapshot_suffix_replay_persists_mc202_source_phrase_plan_without_graph() {
    let mut action_log = action_log(vec![
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
    ]);
    let follower_plan = source_phrase_plan(
        "src-1",
        Mc202RoleState::Follower,
        Mc202SourcePhraseCandidateFamilyState::SubPressureShove,
        0,
    );
    let answer_plan = source_phrase_plan(
        "src-1",
        Mc202RoleState::Answer,
        Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer,
        3,
    );
    action_log.commit_records[0].mc202_source_phrase_plan = Some(follower_plan);
    action_log.commit_records[1].mc202_source_phrase_plan = Some(answer_plan.clone());

    let snapshots = vec![snapshot("snap-after-follower", 1)];
    let origin_plan = build_replay_target_plan(&action_log, &[], 2).expect("origin plan");
    let anchor_plan = build_replay_target_plan(&action_log, &[], 1).expect("anchor plan");
    let snapshot_plan =
        build_replay_target_plan(&action_log, &snapshots, 2).expect("snapshot plan");
    let mut origin_session =
        SessionFile::new("origin-session", "riotbox-test", "2026-06-18T20:30:00Z");
    let mut snapshot_session =
        SessionFile::new("snapshot-session", "riotbox-test", "2026-06-18T20:30:00Z");

    apply_replay_plan_to_session(&mut origin_session, &origin_plan.suffix)
        .expect("origin replay succeeds");
    apply_replay_plan_to_session(&mut snapshot_session, &anchor_plan.suffix)
        .expect("anchor replay succeeds");
    apply_replay_plan_to_session(&mut snapshot_session, &snapshot_plan.suffix)
        .expect("snapshot suffix replay succeeds");

    assert_eq!(
        snapshot_session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan,
        Some(answer_plan.clone())
    );
    assert_eq!(
        origin_session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan,
        Some(answer_plan)
    );
    assert_eq!(
        snapshot_session.runtime_state.lane_state.mc202,
        origin_session.runtime_state.lane_state.mc202
    );
}

#[test]
fn mc202_phrase_replay_without_persisted_plan_clears_stale_source_plan() {
    let action_log = action_log(vec![action(
        1,
        ActionCommand::Mc202GenerateAnswer,
        ActionParams::Mutation {
            intensity: 0.83,
            target_id: None,
        },
        100,
    )]);
    let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-06-18T21:00:00Z");
    session.runtime_state.lane_state.mc202.source_phrase_plan = Some(source_phrase_plan(
        "src-1",
        Mc202RoleState::Follower,
        Mc202SourcePhraseCandidateFamilyState::SubPressureShove,
        0,
    ));

    apply_replay_plan_to_session(&mut session, &plan).expect("MC-202 replay succeeds");

    assert_eq!(
        session.runtime_state.lane_state.mc202.role,
        Some(Mc202RoleState::Answer)
    );
    assert!(
        session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .is_none()
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
        session.runtime_state.lane_state.mc202.role,
        Some(Mc202RoleState::Leader)
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

#[test]
fn mc202_set_role_rejects_unknown_role_target_without_mutating_session() {
    let action_log = action_log(vec![targeted_action(
        1,
        ActionCommand::Mc202SetRole,
        ActionParams::Empty,
        ActionTarget {
            scope: Some(TargetScope::LaneMc202),
            object_id: Some("scene_lock".into()),
            ..Default::default()
        },
        100,
    )]);
    let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T20:00:00Z");
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan).expect_err("unknown role target");

    assert_eq!(
        error,
        ReplayExecutionError::InvalidParams {
            action_id: ActionId(1),
            command: ActionCommand::Mc202SetRole,
            expected: "known MC-202 role label"
        }
    );
    assert_eq!(session, original_session);
}

#[test]
fn mc202_mutate_phrase_rejects_unknown_intent_target_without_mutating_session() {
    let action_log = action_log(vec![action(
        1,
        ActionCommand::Mc202MutatePhrase,
        ActionParams::Mutation {
            intensity: 0.91,
            target_id: Some("scene_lock".into()),
        },
        100,
    )]);
    let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T20:00:00Z");
    session.runtime_state.lane_state.mc202.role = Some(Mc202RoleState::Follower);
    let original_session = session.clone();

    let error =
        apply_replay_plan_to_session(&mut session, &plan).expect_err("unknown phrase intent");

    assert_eq!(
        error,
        ReplayExecutionError::InvalidParams {
            action_id: ActionId(1),
            command: ActionCommand::Mc202MutatePhrase,
            expected: "known MC-202 phrase mutation intent label"
        }
    );
    assert_eq!(session, original_session);
}

#[test]
fn mc202_mutate_phrase_rejects_base_intent_without_mutating_session() {
    let action_log = action_log(vec![action(
        1,
        ActionCommand::Mc202MutatePhrase,
        ActionParams::Mutation {
            intensity: 0.91,
            target_id: Some("base".into()),
        },
        100,
    )]);
    let plan = build_committed_replay_plan(&action_log).expect("valid replay plan");
    let mut session = SessionFile::new("session-1", "riotbox-test", "2026-04-29T20:00:00Z");
    session.runtime_state.lane_state.mc202.role = Some(Mc202RoleState::Follower);
    let original_session = session.clone();

    let error = apply_replay_plan_to_session(&mut session, &plan).expect_err("base intent");

    assert_eq!(
        error,
        ReplayExecutionError::InvalidParams {
            action_id: ActionId(1),
            command: ActionCommand::Mc202MutatePhrase,
            expected: "known MC-202 phrase mutation intent label"
        }
    );
    assert_eq!(session, original_session);
}

fn source_phrase_plan(
    source_id: &str,
    role: Mc202RoleState,
    family: Mc202SourcePhraseCandidateFamilyState,
    rotation: usize,
) -> Mc202SourcePhrasePlanState {
    let mut rhythm_cells = [None; 16];
    for step in [1, 5, 9, 13] {
        rhythm_cells[(step + rotation) % 16] = Some(0);
    }
    Mc202SourcePhrasePlanState {
        source_id: SourceId::from(source_id),
        phrase_slot: Mc202SourcePhraseSlotState {
            phrase_index: 1,
            start_bar: 0,
            end_bar: 7,
        },
        source_expression: None,
        role,
        rhythm_cells,
        note_budget: Mc202SourcePhraseNoteBudgetState::Sparse,
        touch: role.default_touch(),
        confidence: 0.86,
        candidate_family: Some(family),
        candidate_count: 3,
        rejected_candidate_count: 1,
        candidate_provenance_refs: vec![format!("candidate_family:{}", family.label())],
        candidate_scorecards: Vec::new(),
        phrase_memory_distance: 1.0,
        fallback_reason: None,
    }
}
