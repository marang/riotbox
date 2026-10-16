//! Whole-batch admission against accepted persisted commit history.
use super::ActionQueue;
use crate::{
    action::{
        ActionCommand, ActionDraft, ActionStatus, ActionTarget, ActorType, CommitBoundary,
        Quantization,
    },
    ids::ActionId,
    replay::build_committed_replay_plan,
    session::{ActionCommitRecord, ActionLog, CommitSequenceExhausted, SessionFile},
    transport::CommitBoundaryState,
};

fn boundary() -> CommitBoundaryState {
    CommitBoundaryState {
        kind: CommitBoundary::Immediate,
        beat_index: 12,
        bar_index: 4,
        phrase_index: 1,
        scene_id: None,
    }
}

fn draft(command: ActionCommand, quantization: Quantization) -> ActionDraft {
    ActionDraft::new(
        ActorType::User,
        command,
        quantization,
        ActionTarget::default(),
    )
}

fn session_at_sequence(sequence: u32) -> SessionFile {
    let mut seed = ActionQueue::new();
    let id = seed
        .enqueue(
            draft(ActionCommand::TransportPause, Quantization::Immediate),
            1,
        )
        .unwrap();
    seed.commit_ready_for_transport(&ActionLog::default(), boundary(), 2)
        .unwrap();
    let mut session = SessionFile::new("generated-sequence-capacity", "test", "2026-10-16");
    session.action_log.actions.push(seed.history()[0].clone());
    session.action_log.commit_records.push(ActionCommitRecord {
        action_id: id,
        boundary: boundary(),
        commit_sequence: sequence,
        committed_at: 2,
        mc202_source_phrase_plan: None,
    });
    assert!(build_committed_replay_plan(&session.action_log).is_ok());
    session
}

fn queue_after(session: &SessionFile) -> ActionQueue {
    let mut queue = ActionQueue::new();
    queue.reserve_action_ids_after(
        session
            .action_log
            .actions
            .iter()
            .map(|action| action.id)
            .max(),
    );
    queue
}

#[test]
fn full_persisted_boundary_refuses_queue_drain_without_consuming_actions() {
    let session = session_at_sequence(u32::MAX);
    let mut queue = queue_after(&session);
    queue
        .enqueue(
            draft(ActionCommand::TransportPlay, Quantization::Immediate),
            3,
        )
        .unwrap();
    let pending_before: Vec<_> = queue.pending_actions().into_iter().cloned().collect();
    assert_eq!(
        queue.commit_ready_for_transport(&session.action_log, boundary(), 4),
        Err(CommitSequenceExhausted {
            boundary: boundary(),
            requested_count: 1,
            max_existing: u32::MAX,
        })
    );
    assert_eq!(
        queue.pending_actions(),
        pending_before.iter().collect::<Vec<_>>(),
        "accepted MAX history must refuse the whole drain before queue mutation"
    );
    assert!(queue.history().is_empty());
    assert_eq!(
        session.action_log.commit_records[0].commit_sequence,
        u32::MAX
    );
}

#[test]
fn two_ready_actions_cannot_partially_use_the_last_slot_or_relabel_other_actions() {
    let session = session_at_sequence(u32::MAX - 1);
    let mut queue = queue_after(&session);
    // Establish a real pre-existing PendingCommit status beside newly Queued actions.
    queue
        .enqueue(draft(ActionCommand::CaptureNow, Quantization::NextBar), 3)
        .unwrap();
    queue
        .commit_ready_for_transport(&session.action_log, boundary(), 4)
        .unwrap();
    queue
        .enqueue(
            draft(ActionCommand::TransportPlay, Quantization::Immediate),
            5,
        )
        .unwrap();
    queue
        .enqueue(
            draft(ActionCommand::TransportPause, Quantization::Immediate),
            6,
        )
        .unwrap();
    queue
        .enqueue(
            draft(ActionCommand::ExportProductMix, Quantization::Immediate),
            7,
        )
        .unwrap();
    let before: Vec<_> = queue.pending_actions().into_iter().cloned().collect();
    let history_before = queue.history().to_vec();
    let log_before = session.action_log.clone();
    assert_eq!(
        queue.ready_action_count_for_transport(CommitBoundary::Immediate),
        2
    );
    let error = queue
        .commit_ready_for_transport(&session.action_log, boundary(), 8)
        .unwrap_err();
    assert_eq!(error.requested_count, 2);
    assert_eq!(error.max_existing, u32::MAX - 1);
    assert_eq!(queue.pending_actions(), before.iter().collect::<Vec<_>>());
    assert_eq!(queue.history(), history_before);
    assert_eq!(session.action_log, log_before);
    assert_eq!(
        queue.pending_actions()[0].status,
        ActionStatus::PendingCommit
    );
    assert_eq!(queue.pending_actions()[3].status, ActionStatus::Queued);
}

#[test]
fn final_slot_assigns_max_and_next_drain_rechecks_incrementally_recorded_history() {
    let mut session = session_at_sequence(u32::MAX - 1);
    let mut queue = queue_after(&session);
    let id = queue
        .enqueue(
            draft(ActionCommand::TransportPlay, Quantization::Immediate),
            3,
        )
        .unwrap();
    // A trailing non-ready action must survive the admitted ready-stream traversal.
    let later = queue
        .enqueue(draft(ActionCommand::CaptureNow, Quantization::NextBar), 4)
        .unwrap();
    let committed = queue
        .commit_ready_for_transport(&session.action_log, boundary(), 5)
        .unwrap();
    assert_eq!(committed.len(), 1);
    assert_eq!(committed[0].action_id, id);
    assert_eq!(committed[0].commit_sequence, u32::MAX);
    assert_eq!(queue.pending_actions()[0].id, later);
    assert_eq!(
        queue.pending_actions()[0].status,
        ActionStatus::PendingCommit
    );
    session
        .action_log
        .actions
        .push(queue.history_action(id).unwrap().clone());
    session.action_log.commit_records.push(ActionCommitRecord {
        action_id: id,
        boundary: boundary(),
        commit_sequence: committed[0].commit_sequence,
        committed_at: 5,
        mc202_source_phrase_plan: None,
    });
    assert!(build_committed_replay_plan(&session.action_log).is_ok());
    queue
        .enqueue(
            draft(ActionCommand::TransportPause, Quantization::Immediate),
            6,
        )
        .unwrap();
    let before: Vec<_> = queue.pending_actions().into_iter().cloned().collect();
    assert!(
        queue
            .commit_ready_for_transport(&session.action_log, boundary(), 7)
            .is_err()
    );
    assert_eq!(queue.pending_actions(), before.iter().collect::<Vec<_>>());
    assert_eq!(queue.history().len(), 1);
}

#[test]
fn zero_ready_at_max_keeps_normal_successful_relabeling_without_committing_exports() {
    let session = session_at_sequence(u32::MAX);
    let mut queue = queue_after(&session);
    assert!(
        queue
            .commit_ready_for_transport(&session.action_log, boundary(), 3)
            .unwrap()
            .is_empty()
    );
    queue
        .enqueue(draft(ActionCommand::CaptureNow, Quantization::NextBar), 4)
        .unwrap();
    queue
        .enqueue(
            draft(ActionCommand::ExportProductMix, Quantization::Immediate),
            5,
        )
        .unwrap();
    assert_eq!(
        queue.ready_action_count_for_transport(CommitBoundary::Immediate),
        0
    );
    assert!(
        queue
            .commit_ready_for_transport(&session.action_log, boundary(), 6)
            .unwrap()
            .is_empty()
    );
    assert_eq!(queue.pending_actions().len(), 2);
    assert!(
        queue
            .pending_actions()
            .iter()
            .all(|action| action.status == ActionStatus::PendingCommit)
    );
    assert!(queue.history().is_empty());
}

#[test]
fn side_effect_missing_or_not_ready_precedes_exhaustion_and_does_not_relabel() {
    let session = session_at_sequence(u32::MAX);
    let mut queue = queue_after(&session);
    let id = queue
        .enqueue(
            draft(ActionCommand::ExportProductMix, Quantization::NextBar),
            3,
        )
        .unwrap();
    let before: Vec<_> = queue.pending_actions().into_iter().cloned().collect();
    for candidate in [ActionId(999), id] {
        assert_eq!(
            queue.commit_pending_after_side_effect(
                &session.action_log,
                candidate,
                boundary(),
                4,
                "generated"
            ),
            Ok(None)
        );
    }
    assert_eq!(queue.pending_actions(), before.iter().collect::<Vec<_>>());
    assert!(queue.history().is_empty());
}

#[test]
fn side_effect_rechecks_capacity_before_removing_eligible_pending_action() {
    let session = session_at_sequence(u32::MAX);
    let mut queue = queue_after(&session);
    let id = queue
        .enqueue(
            draft(ActionCommand::ExportProductMix, Quantization::Immediate),
            3,
        )
        .unwrap();
    let before: Vec<_> = queue.pending_actions().into_iter().cloned().collect();
    let error = queue
        .commit_pending_after_side_effect(&session.action_log, id, boundary(), 4, "generated")
        .unwrap_err();
    assert_eq!(error.requested_count, 1);
    assert_eq!(queue.pending_actions(), before.iter().collect::<Vec<_>>());
    assert!(queue.history().is_empty());
}

#[test]
fn side_effect_uses_last_absolute_sequence_and_leaves_other_pending_actions_untouched() {
    let session = session_at_sequence(u32::MAX - 1);
    let mut queue = queue_after(&session);
    queue
        .enqueue(
            draft(ActionCommand::TransportPlay, Quantization::Immediate),
            3,
        )
        .unwrap();
    let id = queue
        .enqueue(
            draft(ActionCommand::ExportProductMix, Quantization::Immediate),
            4,
        )
        .unwrap();
    queue
        .enqueue(draft(ActionCommand::CaptureNow, Quantization::NextBar), 5)
        .unwrap();
    let before: Vec<_> = queue
        .pending_actions()
        .into_iter()
        .filter(|action| action.id != id)
        .cloned()
        .collect();
    let committed = queue
        .commit_pending_after_side_effect(
            &session.action_log,
            id,
            boundary(),
            6,
            "generated artifact",
        )
        .unwrap()
        .unwrap();
    assert_eq!(committed.commit_sequence, u32::MAX);
    assert_eq!(committed.action_id, id);
    assert_eq!(queue.pending_actions(), before.iter().collect::<Vec<_>>());
    assert_eq!(queue.history().len(), 1);
    assert_eq!(
        queue.history()[0].result.as_ref().unwrap().summary,
        "generated artifact"
    );
}

#[test]
fn normal_batch_uses_history_maximum_and_readonly_count_does_not_mutate() {
    let session = session_at_sequence(9);
    let mut queue = queue_after(&session);
    queue
        .enqueue(draft(ActionCommand::CaptureNow, Quantization::NextBar), 3)
        .unwrap();
    let first = queue
        .enqueue(
            draft(ActionCommand::TransportPlay, Quantization::Immediate),
            4,
        )
        .unwrap();
    queue
        .enqueue(
            draft(ActionCommand::ExportProductMix, Quantization::Immediate),
            5,
        )
        .unwrap();
    let second = queue
        .enqueue(
            draft(ActionCommand::TransportPause, Quantization::Immediate),
            6,
        )
        .unwrap();
    queue
        .enqueue(draft(ActionCommand::CaptureNow, Quantization::NextBar), 7)
        .unwrap();
    let before: Vec<_> = queue.pending_actions().into_iter().cloned().collect();
    assert_eq!(
        queue.ready_action_count_for_transport(CommitBoundary::Immediate),
        2
    );
    assert_eq!(
        queue.ready_action_count_for_transport(CommitBoundary::Bar),
        4
    );
    assert_eq!(queue.pending_actions(), before.iter().collect::<Vec<_>>());
    let refs = queue
        .commit_ready_for_transport(&session.action_log, boundary(), 8)
        .unwrap();
    assert_eq!(
        refs.iter()
            .map(|item| (item.action_id, item.commit_sequence))
            .collect::<Vec<_>>(),
        vec![(first, 10), (second, 11)]
    );
    assert_eq!(queue.pending_actions().len(), 3);
    assert_eq!(queue.history().len(), 2);
}

#[test]
fn legacy_boundary_entrypoint_enforces_history_capacity_too() {
    let mut session = session_at_sequence(u32::MAX);
    session.action_log.commit_records[0].boundary = CommitBoundaryState {
        kind: CommitBoundary::Immediate,
        beat_index: 0,
        bar_index: 0,
        phrase_index: 0,
        scene_id: None,
    };
    let mut queue = queue_after(&session);
    queue
        .enqueue(
            draft(ActionCommand::TransportPlay, Quantization::Immediate),
            3,
        )
        .unwrap();
    assert!(
        queue
            .commit_ready(&session.action_log, CommitBoundary::Immediate, 4)
            .is_err()
    );
    assert_eq!(queue.pending_actions().len(), 1);
    assert!(queue.history().is_empty());
}
