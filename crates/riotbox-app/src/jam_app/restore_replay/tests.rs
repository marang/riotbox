//! Characterization of the existing unvalidated App join, not source evidence.
use super::legacy_mc202::mc202_reconstruction_entries;
use crate::jam_app::{JamAppState, helpers::is_mc202_phrase_action};
use riotbox_core::{
    action::{
        Action, ActionCommand, ActionParams, ActionResult, ActionStatus, ActionTarget, ActorType,
        CommitBoundary, Quantization, UndoPolicy,
    },
    ids::ActionId,
    queue::ActionQueue,
    replay::ReplayPlanEntry,
    session::{
        ActionCommitRecord, Mc202RoleState, Mc202SourcePhraseNoteBudgetState,
        Mc202SourcePhrasePlanState, Mc202SourcePhraseSlotState, ReplayPolicy, SessionFile,
        SourceTimingGridConfirmationState,
    },
    transport::CommitBoundaryState,
};

// Retain the original scan independently of the optimized implementation.
fn linear_oracle(session: &SessionFile, cursor: usize) -> Vec<ReplayPlanEntry<'_>> {
    let prefix = &session.action_log.actions[..cursor.min(session.action_log.actions.len())];
    session
        .action_log
        .commit_records
        .iter()
        .filter_map(|record| {
            let action = prefix.iter().find(|action| action.id == record.action_id)?;
            (action.status == ActionStatus::Committed
                && action.result.as_ref().is_none_or(|result| result.accepted)
                && (is_mc202_phrase_action(action.command)
                    || matches!(
                        action.command,
                        ActionCommand::SourceTimingConfirmGrid
                            | ActionCommand::SourceTimingRevertGrid
                    )))
            .then_some(ReplayPlanEntry {
                action,
                commit_record: record,
            })
        })
        .collect()
}

fn assert_all_prefixes(session: &SessionFile) {
    // Core owns policy admission; this existing unvalidated collector must not
    // reinterpret any declared policy or turn adversarial fixtures into proof.
    for policy in [
        ReplayPolicy::DeterministicPreferred,
        ReplayPolicy::SnapshotOnly,
        ReplayPolicy::DeterministicCommittedOrderV2,
    ] {
        let mut declared = session.clone();
        declared.action_log.replay_policy = policy;
        for cursor in 0..=declared.action_log.actions.len() + 1 {
            let actual = mc202_reconstruction_entries(&declared, cursor);
            let expected = linear_oracle(&declared, cursor);
            let fields =
                |input: ReplayPlanEntry<'_>| (input.action.clone(), input.commit_record.clone());
            assert_eq!(
                actual.into_iter().map(fields).collect::<Vec<_>>(),
                expected.into_iter().map(fields).collect::<Vec<_>>(),
                "policy={policy:?}, cursor={cursor}"
            );
        }
    }
}

fn action(id: u64, command: ActionCommand) -> Action {
    Action {
        id: ActionId(id),
        actor: ActorType::User,
        command,
        params: ActionParams::Empty,
        target: ActionTarget::default(),
        requested_at: 10,
        quantization: Quantization::Immediate,
        status: ActionStatus::Committed,
        committed_at: Some(20),
        result: Some(ActionResult {
            accepted: true,
            summary: format!("generated-{id}-{command}"),
        }),
        undo_policy: UndoPolicy::NotUndoable {
            reason: "generated metadata-only join control".into(),
        },
        explanation: None,
    }
}

fn plan(source: &str, touch: f32) -> Mc202SourcePhrasePlanState {
    Mc202SourcePhrasePlanState {
        source_id: source.into(),
        source_section_id: None,
        phrase_slot: Mc202SourcePhraseSlotState {
            phrase_index: 1,
            start_bar: 0,
            end_bar: 3,
        },
        source_expression: None,
        role: Mc202RoleState::Follower,
        rhythm_cells: [Some(0); 16],
        note_budget: Mc202SourcePhraseNoteBudgetState::Sparse,
        touch,
        confidence: 0.8,
        candidate_family: None,
        candidate_count: 0,
        rejected_candidate_count: 0,
        candidate_provenance_refs: vec![format!("generated-metadata-{touch}")],
        candidate_scorecards: vec![],
        phrase_memory_distance: touch,
        fallback_reason: None,
    }
}

fn record(id: u64, beat: u64, persisted: Option<Mc202SourcePhrasePlanState>) -> ActionCommitRecord {
    ActionCommitRecord {
        action_id: ActionId(id),
        boundary: CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: beat,
            bar_index: beat / 4 + 1,
            phrase_index: beat / 16 + 1,
            scene_id: None,
        },
        commit_sequence: 1,
        committed_at: 20,
        mc202_source_phrase_plan: persisted,
    }
}

fn session() -> SessionFile {
    let mut session = SessionFile::new("generated-restore-join", "test", "2026-10-16");
    // Low-level from_parts intentionally does not validate these adversarial logs.
    session.action_log.replay_policy = ReplayPolicy::DeterministicPreferred;
    session.action_log.actions = vec![
        action(100, ActionCommand::Mc202SetRole),
        action(2, ActionCommand::TransportPlay),
        action(70, ActionCommand::Mc202GenerateFollower),
        action(4, ActionCommand::Mc202MutatePhrase),
    ];
    session.action_log.commit_records = vec![
        record(100, 12, Some(plan("generated", 0.1))),
        record(2, 12, None),
        record(70, 0, None),
        record(4, 8, Some(plan("generated", 0.9))),
    ];
    session
}

#[test]
fn original_scan_matches_all_prefixes_with_backward_and_permuted_record_order() {
    let mut session = session();
    assert_all_prefixes(&session);
    session.action_log.commit_records.reverse();
    assert_all_prefixes(&session);
    session.action_log.commit_records.rotate_left(1);
    assert_all_prefixes(&session);
    session.action_log.commit_records.clear();
    assert_all_prefixes(&session);
    session.action_log.actions.clear();
    assert_all_prefixes(&session);
}

#[test]
fn every_status_result_and_first_duplicate_identity_matches_the_original_scan() {
    for status in [
        ActionStatus::Committed,
        ActionStatus::Undone,
        ActionStatus::Rejected,
        ActionStatus::Queued,
    ] {
        for accepted in [Some(true), Some(false), None] {
            for command in [ActionCommand::TransportPlay, ActionCommand::Mc202SetRole] {
                let mut session = session();
                let mut first = action(9, command);
                first.status = status;
                first.result = accepted.map(|accepted| ActionResult {
                    accepted,
                    summary: "first identity".into(),
                });
                session.action_log.actions =
                    vec![first, action(9, ActionCommand::Mc202GenerateFollower)];
                session.action_log.commit_records =
                    vec![record(9, 12, Some(plan("generated", 0.5)))];
                assert_all_prefixes(&session);
                let eligible = status == ActionStatus::Committed
                    && accepted != Some(false)
                    && command == ActionCommand::Mc202SetRole;
                assert_eq!(
                    mc202_reconstruction_entries(&session, 2).len(),
                    usize::from(eligible)
                );
            }
        }
    }
}

#[test]
fn missing_duplicate_and_future_records_keep_each_boundary_and_plan_in_stored_order() {
    let mut session = session();
    session
        .action_log
        .commit_records
        .insert(0, record(999, 44, None));
    session
        .action_log
        .commit_records
        .push(record(100, 40, Some(plan("generated", 0.4))));
    assert_all_prefixes(&session);
    let selected = mc202_reconstruction_entries(&session, usize::MAX);
    assert_eq!(
        selected
            .iter()
            .map(|input| input.action.id)
            .collect::<Vec<_>>(),
        vec![ActionId(100), ActionId(70), ActionId(4), ActionId(100)]
    );
    assert_eq!(
        selected.last().unwrap().commit_record.boundary.beat_index,
        40
    );
    assert_eq!(
        selected
            .last()
            .unwrap()
            .commit_record
            .mc202_source_phrase_plan,
        Some(plan("generated", 0.4))
    );
}

#[test]
fn reconstruction_preserves_last_plan_trust_missing_graph_and_empty_input_rules() {
    for last_plan in [
        Some(plan("generated", 0.9)),
        Some(plan("untrusted", 0.9)),
        None,
    ] {
        let mut session = session();
        session.runtime_state.source_timing.confirmed_grid =
            Some(SourceTimingGridConfirmationState {
                source_id: "generated".into(),
                hypothesis_id: None,
                confirmed_by_action: ActionId(77),
                confirmed_at: 1,
            });
        session
            .action_log
            .commit_records
            .last_mut()
            .unwrap()
            .mc202_source_phrase_plan = last_plan.clone();
        let app = JamAppState::from_parts(session, None, ActionQueue::new());
        let expected = last_plan.filter(|plan| plan.source_id.as_str() == "generated");
        assert_eq!(
            app.session
                .runtime_state
                .lane_state
                .mc202
                .source_phrase_plan,
            expected
        );
        assert!(app.source_audio_cache.is_none() && app.capture_audio_cache.is_empty());
    }
    let mut session = session();
    session.action_log.actions.clear();
    let retained = plan("untrusted-but-no-replay-input", 0.7);
    session.runtime_state.lane_state.mc202.source_phrase_plan = Some(retained.clone());
    let app = JamAppState::from_parts(session, None, ActionQueue::new());
    assert_eq!(
        app.session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan,
        Some(retained)
    );
}
