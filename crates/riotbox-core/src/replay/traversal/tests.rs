//! Public replay interfaces with generated metadata, no Graph/audio access.
use crate::{
    action::{
        Action, ActionCommand, ActionParams, ActionResult, ActionStatus, ActionTarget, ActorType,
        CommitBoundary, Quantization, UndoPolicy,
    },
    ids::ActionId,
    replay::{
        ReplayExecutionError, ReplayPlanEntry, ReplayTargetExecutionError,
        apply_replay_entry_to_session, apply_replay_target_suffix_to_session,
        apply_replay_target_suffix_with_mc202_legacy_resolver,
    },
    session::{
        ActionCommitRecord, Mc202RoleState, Mc202SourcePhraseCandidateFamilyState,
        Mc202SourcePhraseNoteBudgetState, Mc202SourcePhrasePlanState, Mc202SourcePhraseSlotState,
        SessionFile,
    },
    transport::CommitBoundaryState,
};

fn action(id: u64, command: ActionCommand, params: ActionParams) -> Action {
    Action {
        id: ActionId(id),
        actor: ActorType::User,
        command,
        params,
        target: ActionTarget::default(),
        requested_at: id * 10,
        quantization: Quantization::Immediate,
        status: ActionStatus::Committed,
        committed_at: Some(id * 10),
        result: Some(ActionResult {
            accepted: true,
            summary: "generated control".into(),
        }),
        undo_policy: if command.has_typed_undo_semantics() {
            UndoPolicy::Undoable
        } else {
            UndoPolicy::NotUndoable {
                reason: "generated structural action".into(),
            }
        },
        explanation: None,
    }
}

fn session(actions: Vec<Action>) -> SessionFile {
    let mut session = SessionFile::new("generated-traversal", "test", "2026-10-16");
    session.action_log.commit_records = actions
        .iter()
        .map(|action| ActionCommitRecord {
            action_id: action.id,
            boundary: CommitBoundaryState {
                kind: CommitBoundary::Immediate,
                beat_index: 0,
                bar_index: 1,
                phrase_index: 1,
                scene_id: None,
            },
            commit_sequence: action.id.0 as u32,
            committed_at: action.committed_at.unwrap(),
            mc202_source_phrase_plan: None,
        })
        .collect();
    session.action_log.actions = actions;
    session
}

fn mutation(touch: f32) -> ActionParams {
    ActionParams::Mutation {
        intensity: touch,
        target_id: None,
    }
}

fn plan(role: Mc202RoleState, touch: f32) -> Mc202SourcePhrasePlanState {
    Mc202SourcePhrasePlanState {
        source_id: "generated-metadata-only".into(),
        source_section_id: None,
        phrase_slot: Mc202SourcePhraseSlotState {
            phrase_index: 1,
            start_bar: 0,
            end_bar: 3,
        },
        source_expression: None,
        role,
        rhythm_cells: [Some(0); 16],
        note_budget: Mc202SourcePhraseNoteBudgetState::Sparse,
        touch,
        confidence: 0.8,
        candidate_family: Some(match role {
            Mc202RoleState::Leader | Mc202RoleState::Follower => {
                Mc202SourcePhraseCandidateFamilyState::CallBackStab
            }
            Mc202RoleState::Answer => Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer,
            Mc202RoleState::Pressure => Mc202SourcePhraseCandidateFamilyState::SubPressureShove,
            Mc202RoleState::Instigator => {
                Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator
            }
        }),
        candidate_count: 0,
        rejected_candidate_count: 0,
        candidate_provenance_refs: vec!["generated control, not source evidence".into()],
        candidate_scorecards: vec![],
        phrase_memory_distance: 0.5,
        fallback_reason: None,
    }
}

#[test]
fn missing_resolver_observes_pre_entry_memory_and_core_captures_ordinary_undo_snapshots() {
    let mut set_role = action(2, ActionCommand::Mc202SetRole, mutation(0.2));
    set_role.target.object_id = Some("follower".into());
    let mut session = session(vec![
        action(1, ActionCommand::Mc202GenerateFollower, mutation(0.8)),
        set_role,
        action(3, ActionCommand::Mc202GenerateAnswer, mutation(0.1)),
    ]);
    let initial = plan(Mc202RoleState::Leader, 0.4);
    let follower = plan(Mc202RoleState::Follower, 0.8);
    let quieter = plan(Mc202RoleState::Follower, 0.2);
    let answer = plan(Mc202RoleState::Answer, 0.2);
    session.runtime_state.lane_state.mc202.source_phrase_plan = Some(initial.clone());
    session.runtime_state.macro_state.mc202_touch = 0.4;
    let mut observed = Vec::new();
    let report = apply_replay_target_suffix_with_mc202_legacy_resolver(
        &mut session,
        3,
        None,
        |pre, entry| {
            observed.push((
                entry.action.id,
                pre.runtime_state
                    .lane_state
                    .mc202
                    .source_phrase_plan
                    .clone(),
                pre.runtime_state.macro_state.mc202_touch,
                pre.runtime_state.undo_state.mc202_snapshots.len(),
            ));
            Some(match entry.action.id.0 {
                1 => follower.clone(),
                2 => quieter.clone(),
                3 => answer.clone(),
                _ => unreachable!(),
            })
        },
    )
    .unwrap();
    assert_eq!(
        observed,
        vec![
            (ActionId(1), Some(initial.clone()), 0.4, 0),
            (ActionId(2), Some(follower.clone()), 0.8, 1),
            (ActionId(3), Some(quieter.clone()), 0.2, 2),
        ]
    );
    assert_eq!(
        report.applied_action_ids,
        vec![ActionId(1), ActionId(2), ActionId(3)]
    );
    assert_eq!(
        session.runtime_state.lane_state.mc202.source_phrase_plan,
        Some(answer)
    );
    assert_eq!(session.runtime_state.macro_state.mc202_touch, 0.2);
    let snapshots = &session.runtime_state.undo_state.mc202_snapshots;
    assert_eq!(
        snapshots
            .iter()
            .map(|snapshot| (
                snapshot.action_id,
                snapshot.source_phrase_plan.clone(),
                snapshot.touch
            ))
            .collect::<Vec<_>>(),
        vec![
            (ActionId(1), Some(initial), 0.4),
            (ActionId(2), Some(follower), 0.8),
            (ActionId(3), Some(quieter.clone()), 0.2)
        ]
    );
    let snapshot = snapshots[2].clone();
    snapshot.apply_to_session(&mut session);
    assert_eq!(
        session.runtime_state.lane_state.mc202.source_phrase_plan,
        Some(quieter)
    );
    assert_eq!(session.runtime_state.macro_state.mc202_touch, 0.2);
}

#[test]
fn persisted_source_and_stay_out_plans_bypass_legacy_resolver() {
    let mut session = session(vec![
        action(1, ActionCommand::Mc202GenerateFollower, mutation(0.7)),
        action(2, ActionCommand::Mc202GenerateAnswer, mutation(0.2)),
    ]);
    let persisted = plan(Mc202RoleState::Follower, 0.7);
    let mut stay_out = plan(Mc202RoleState::Answer, 0.2);
    stay_out.candidate_family = Some(Mc202SourcePhraseCandidateFamilyState::StayOut);
    stay_out.rhythm_cells = [None; 16];
    stay_out.fallback_reason = Some("generated explicit degraded decision".into());
    assert!(persisted.is_source_derived());
    assert!(!stay_out.is_source_derived());
    session.action_log.commit_records[0].mc202_source_phrase_plan = Some(persisted.clone());
    session.action_log.commit_records[1].mc202_source_phrase_plan = Some(stay_out.clone());
    let log = session.action_log.clone();
    apply_replay_target_suffix_with_mc202_legacy_resolver(&mut session, 2, None, |_, _| {
        panic!("persisted Some must never invoke a selector")
    })
    .unwrap();
    assert_eq!(
        session.runtime_state.lane_state.mc202.source_phrase_plan,
        Some(stay_out)
    );
    assert_eq!(
        session.runtime_state.undo_state.mc202_snapshots[1].source_phrase_plan,
        Some(persisted)
    );
    assert_eq!(session.action_log, log);
}

#[test]
fn default_graph_free_target_does_not_backfill_missing_plans() {
    let mut session = session(vec![action(
        1,
        ActionCommand::Mc202GenerateFollower,
        mutation(0.6),
    )]);
    session.runtime_state.lane_state.mc202.source_phrase_plan =
        Some(plan(Mc202RoleState::Leader, 0.9));
    let report = apply_replay_target_suffix_to_session(&mut session, 1, None).unwrap();
    assert_eq!(report.applied_action_ids, vec![ActionId(1)]);
    assert_eq!(session.runtime_state.macro_state.mc202_touch, 0.6);
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
fn direct_and_default_set_role_lower_clamped_or_default_touch_while_generate_and_mutate_keep_max() {
    for (label, params, role_touch, final_touch) in [
        ("follower", mutation(0.2), 0.2, 0.2),
        ("follower", ActionParams::Empty, 0.65, 0.65),
        ("leader", ActionParams::Empty, 0.85, 0.85),
        ("follower", mutation(-1.0), 0.0, 0.1),
        ("follower", mutation(1.2), 1.0, 1.0),
    ] {
        let mut set_role = action(1, ActionCommand::Mc202SetRole, params);
        set_role.target.object_id = Some(label.into());
        let mut session = session(vec![
            set_role,
            action(2, ActionCommand::Mc202GenerateAnswer, mutation(0.1)),
            action(3, ActionCommand::Mc202MutatePhrase, mutation(0.05)),
        ]);
        session.runtime_state.macro_state.mc202_touch = 0.9;
        let log = session.action_log.clone();
        let mut direct = session.clone();
        apply_replay_entry_to_session(
            &mut direct,
            &ReplayPlanEntry {
                action: &log.actions[0],
                commit_record: &log.commit_records[0],
            },
        )
        .unwrap();
        assert_eq!(direct.runtime_state.macro_state.mc202_touch, role_touch);
        apply_replay_target_suffix_to_session(&mut session, 3, None).unwrap();
        assert_eq!(session.runtime_state.macro_state.mc202_touch, final_touch);
        assert_eq!(
            session.runtime_state.undo_state.mc202_snapshots[0].touch,
            0.9
        );
        assert_eq!(
            session.runtime_state.undo_state.mc202_snapshots[1].touch,
            role_touch
        );
        assert_eq!(
            session.runtime_state.undo_state.mc202_snapshots[2].touch,
            final_touch
        );
    }
}

#[test]
fn late_unsupported_or_invalid_entry_rolls_back_entire_session_after_legacy_callback() {
    for command in [ActionCommand::MutateLane, ActionCommand::TransportSeek] {
        let mut session = session(vec![
            action(1, ActionCommand::Mc202GenerateFollower, mutation(0.6)),
            action(2, command, ActionParams::Empty),
        ]);
        session.runtime_state.lane_state.mc202.source_phrase_plan =
            Some(plan(Mc202RoleState::Leader, 0.3));
        session.runtime_state.macro_state.mc202_touch = 0.3;
        let before = session.clone();
        let mut calls = Vec::new();
        let error = apply_replay_target_suffix_with_mc202_legacy_resolver(
            &mut session,
            2,
            None,
            |_, entry| {
                calls.push(entry.action.id);
                Some(plan(Mc202RoleState::Follower, 0.6))
            },
        )
        .unwrap_err();
        match command {
            ActionCommand::MutateLane => assert!(matches!(
                error,
                ReplayTargetExecutionError::Execution(ReplayExecutionError::UnsupportedAction {
                    action_id: ActionId(2),
                    command: ActionCommand::MutateLane
                })
            )),
            ActionCommand::TransportSeek => assert!(matches!(
                error,
                ReplayTargetExecutionError::Execution(ReplayExecutionError::InvalidParams {
                    action_id: ActionId(2),
                    command: ActionCommand::TransportSeek,
                    ..
                })
            )),
            _ => unreachable!(),
        }
        assert_eq!(calls, vec![ActionId(1)]);
        assert_eq!(session, before);
    }
}

#[test]
fn core_planner_excludes_undone_rejected_and_uncommitted_actions_before_resolution() {
    let mut session = session(vec![
        action(1, ActionCommand::Mc202GenerateFollower, mutation(0.6)),
        action(
            2,
            ActionCommand::UndoLast,
            ActionParams::Undo {
                target_action_id: ActionId(1),
            },
        ),
        action(3, ActionCommand::Mc202GenerateAnswer, mutation(0.7)),
        action(4, ActionCommand::Mc202SetRole, ActionParams::Empty),
        action(5, ActionCommand::Mc202GeneratePressure, mutation(0.8)),
    ]);
    session.action_log.actions[0].status = ActionStatus::Undone;
    session.action_log.actions[2]
        .result
        .as_mut()
        .unwrap()
        .accepted = false;
    session.action_log.actions[3].status = ActionStatus::Queued;
    session.action_log.actions[3].committed_at = None;
    session.action_log.commit_records.remove(3);
    let log = session.action_log.clone();
    let mut calls = Vec::new();
    let report =
        apply_replay_target_suffix_with_mc202_legacy_resolver(&mut session, 5, None, |_, entry| {
            calls.push(entry.action.id);
            Some(plan(Mc202RoleState::Pressure, 0.8))
        })
        .unwrap();
    assert_eq!(calls, vec![ActionId(5)]);
    assert_eq!(report.applied_action_ids, vec![ActionId(5)]);
    assert_eq!(session.runtime_state.undo_state.mc202_snapshots.len(), 1);
    assert_eq!(
        session.runtime_state.undo_state.mc202_snapshots[0].action_id,
        ActionId(5)
    );
    assert_eq!(session.action_log, log);
}
