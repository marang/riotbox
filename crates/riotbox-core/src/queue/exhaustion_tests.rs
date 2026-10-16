//! Checked allocation/exhaustion contract through the public queue interface.
use super::{ActionIdExhausted, ActionQueue, QueueEnqueueResult};
use crate::{
    action::{ActionCommand, ActionDraft, ActionStatus, ActorType, CommitBoundary, Quantization},
    ids::ActionId,
};

#[test]
fn exhausted_queue_does_not_enqueue_a_duplicate_action() {
    let mut queue = ActionQueue::new();
    queue.reserve_action_ids_after(Some(ActionId(u64::MAX - 1)));
    let draft = || {
        ActionDraft::new(
            ActorType::User,
            ActionCommand::TransportPlay,
            Quantization::Immediate,
            crate::action::ActionTarget::default(),
        )
    };
    let first = queue.enqueue(draft(), 1).expect("last unique ID");
    assert_eq!(first, ActionId(u64::MAX));
    assert_eq!(queue.enqueue(draft(), 2), Err(ActionIdExhausted));
    assert_eq!(
        queue.pending_actions().len(),
        1,
        "exhaustion must not queue a duplicate ID"
    );
}

fn transport_draft(command: ActionCommand) -> ActionDraft {
    ActionDraft::new(
        ActorType::User,
        command,
        Quantization::Immediate,
        crate::action::ActionTarget::default(),
    )
}

#[test]
fn new_and_default_preserve_their_distinct_first_ids() {
    let mut fresh = ActionQueue::new();
    assert_eq!(fresh.allocate_action_id(), Ok(ActionId(1)));
    assert_eq!(fresh.allocate_action_id(), Ok(ActionId(2)));
    let mut default = ActionQueue::default();
    assert_eq!(default.allocate_action_id(), Ok(ActionId(0)));
    assert_eq!(default.allocate_action_id(), Ok(ActionId(1)));
}

#[test]
fn direct_allocation_uses_the_last_ids_once_then_stays_exhausted() {
    let mut queue = ActionQueue::new();
    queue.reserve_action_ids_after(Some(ActionId(u64::MAX - 2)));
    assert_eq!(queue.allocate_action_id(), Ok(ActionId(u64::MAX - 1)));
    assert_eq!(queue.allocate_action_id(), Ok(ActionId(u64::MAX)));
    for history_max in [None, Some(ActionId(0)), Some(ActionId(u64::MAX - 1))] {
        assert_eq!(queue.allocate_action_id(), Err(ActionIdExhausted));
        queue.reserve_action_ids_after(history_max);
        assert_eq!(queue.allocate_action_id(), Err(ActionIdExhausted));
    }
    assert!(queue.pending_actions().is_empty());
    assert!(queue.history().is_empty());
}

#[test]
fn reserving_an_existing_max_id_is_exhausted_without_reopening() {
    for mut queue in [ActionQueue::new(), ActionQueue::default()] {
        queue.reserve_action_ids_after(Some(ActionId(u64::MAX)));
        for history_max in [
            None,
            Some(ActionId(0)),
            Some(ActionId(u64::MAX - 1)),
            Some(ActionId(u64::MAX)),
        ] {
            queue.reserve_action_ids_after(history_max);
            assert_eq!(
                queue.enqueue(transport_draft(ActionCommand::TransportPlay), 1),
                Err(ActionIdExhausted)
            );
            assert_eq!(queue.allocate_action_id(), Err(ActionIdExhausted));
            assert!(queue.pending_actions().is_empty());
            assert!(queue.history().is_empty());
        }
    }
}

#[test]
fn reservation_only_advances_an_available_cursor() {
    let mut queue = ActionQueue::default();
    queue.reserve_action_ids_after(None);
    assert_eq!(queue.allocate_action_id(), Ok(ActionId(0)));
    queue.reserve_action_ids_after(Some(ActionId(9)));
    queue.reserve_action_ids_after(Some(ActionId(2)));
    assert_eq!(queue.allocate_action_id(), Ok(ActionId(10)));
    queue.reserve_action_ids_after(None);
    assert_eq!(queue.allocate_action_id(), Ok(ActionId(11)));
}

#[test]
fn failed_enqueue_and_allocation_preserve_pending_actions_and_history() {
    let mut queue = ActionQueue::new();
    let committed_id = queue
        .enqueue(transport_draft(ActionCommand::TransportPlay), 1)
        .unwrap();
    assert_eq!(
        queue
            .commit_ready(
                &crate::session::ActionLog::default(),
                CommitBoundary::Immediate,
                2
            )
            .unwrap(),
        vec![committed_id]
    );
    let mut pending_draft = transport_draft(ActionCommand::CaptureNow);
    pending_draft.quantization = Quantization::NextBar;
    queue.enqueue(pending_draft, 3).unwrap();
    queue.reserve_action_ids_after(Some(ActionId(u64::MAX)));
    let pending_before: Vec<_> = queue.pending_actions().into_iter().cloned().collect();
    let history_before = queue.history().to_vec();

    assert_eq!(
        queue.enqueue(transport_draft(ActionCommand::TransportPause), 4),
        Err(ActionIdExhausted)
    );
    assert_eq!(
        queue.enqueue_if_no_pending_command(transport_draft(ActionCommand::TransportStop), 5),
        Err(ActionIdExhausted)
    );
    assert_eq!(queue.allocate_action_id(), Err(ActionIdExhausted));

    assert_eq!(
        queue.pending_actions(),
        pending_before.iter().collect::<Vec<_>>()
    );
    assert_eq!(queue.history(), history_before);
}

#[test]
fn duplicate_pending_command_precedes_exhaustion_until_the_pending_action_is_removed() {
    let mut queue = ActionQueue::new();
    queue.reserve_action_ids_after(Some(ActionId(u64::MAX - 1)));
    let draft = || transport_draft(ActionCommand::ExportProductMix);
    assert_eq!(
        queue.enqueue_if_no_pending_command(draft(), 1),
        Ok(QueueEnqueueResult::Enqueued(ActionId(u64::MAX)))
    );
    let pending_before: Vec<_> = queue.pending_actions().into_iter().cloned().collect();
    assert_eq!(
        queue.enqueue_if_no_pending_command(draft(), 2),
        Ok(QueueEnqueueResult::AlreadyPending {
            command: ActionCommand::ExportProductMix
        })
    );
    assert_eq!(
        queue.pending_actions(),
        pending_before.iter().collect::<Vec<_>>()
    );
    assert!(queue.history().is_empty());
    assert!(queue.reject(ActionId(u64::MAX), "owned export rejected"));
    let history_before = queue.history().to_vec();
    assert_eq!(
        queue.enqueue_if_no_pending_command(draft(), 3),
        Err(ActionIdExhausted)
    );
    assert!(queue.pending_actions().is_empty());
    assert_eq!(queue.history(), history_before);
}

#[test]
fn final_unique_action_still_commits_or_rejects_after_allocation_exhausts() {
    for commit in [true, false] {
        let mut queue = ActionQueue::new();
        queue.reserve_action_ids_after(Some(ActionId(u64::MAX - 1)));
        let id = queue
            .enqueue(transport_draft(ActionCommand::TransportPlay), 1)
            .unwrap();
        assert_eq!(id, ActionId(u64::MAX));
        if commit {
            assert_eq!(
                queue
                    .commit_ready(
                        &crate::session::ActionLog::default(),
                        CommitBoundary::Immediate,
                        2
                    )
                    .unwrap(),
                vec![id]
            );
            assert_eq!(queue.history()[0].status, ActionStatus::Committed);
        } else {
            assert!(queue.reject(id, "owned action rejected"));
            assert_eq!(queue.history()[0].status, ActionStatus::Rejected);
        }
        let history_before = queue.history().to_vec();
        assert!(queue.pending_actions().is_empty());
        assert_eq!(
            queue.enqueue(transport_draft(ActionCommand::TransportPause), 3),
            Err(ActionIdExhausted)
        );
        assert_eq!(queue.history(), history_before);
    }
}

#[test]
fn exhaustion_has_a_typed_standard_error_and_stable_message() {
    let error: &dyn std::error::Error = &ActionIdExhausted;
    assert_eq!(error.to_string(), "action ID space exhausted");
    assert!(error.source().is_none());
}
