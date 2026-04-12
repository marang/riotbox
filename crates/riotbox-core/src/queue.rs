use std::collections::VecDeque;

use crate::{
    TimestampMs,
    action::{Action, ActionDraft, ActionResult, ActionStatus, CommitBoundary},
    ids::ActionId,
};

#[derive(Clone, Debug, Default)]
pub struct ActionQueue {
    next_id: u64,
    pending: VecDeque<Action>,
    history: Vec<Action>,
}

impl ActionQueue {
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_id: 1,
            pending: VecDeque::new(),
            history: Vec::new(),
        }
    }

    pub fn enqueue(&mut self, draft: ActionDraft, requested_at: TimestampMs) -> ActionId {
        let id = ActionId(self.next_id);
        self.next_id += 1;

        let action = Action {
            id,
            actor: draft.actor,
            command: draft.command,
            params: draft.params,
            target: draft.target,
            requested_at,
            quantization: draft.quantization,
            status: ActionStatus::Queued,
            committed_at: None,
            result: None,
            undo_policy: draft.undo_policy,
            explanation: draft.explanation,
        };

        self.pending.push_back(action);
        id
    }

    #[must_use]
    pub fn pending_actions(&self) -> Vec<&Action> {
        self.pending.iter().collect()
    }

    #[must_use]
    pub fn history(&self) -> &[Action] {
        &self.history
    }

    pub fn commit_ready(
        &mut self,
        boundary: CommitBoundary,
        committed_at: TimestampMs,
    ) -> Vec<ActionId> {
        let mut remaining = VecDeque::with_capacity(self.pending.len());
        let mut committed = Vec::new();

        while let Some(mut action) = self.pending.pop_front() {
            if action.quantization.is_ready_for(boundary) {
                action.status = ActionStatus::Committed;
                action.committed_at = Some(committed_at);
                action.result = Some(ActionResult {
                    accepted: true,
                    summary: format!("committed on {boundary:?} boundary"),
                });
                committed.push(action.id);
                self.history.push(action);
            } else {
                action.status = ActionStatus::PendingCommit;
                remaining.push_back(action);
            }
        }

        self.pending = remaining;
        committed
    }

    pub fn reject(&mut self, action_id: ActionId, reason: impl Into<String>) -> bool {
        let reason = reason.into();
        let mut remaining = VecDeque::with_capacity(self.pending.len());
        let mut rejected = false;

        while let Some(mut action) = self.pending.pop_front() {
            if action.id == action_id {
                action.status = ActionStatus::Rejected;
                action.result = Some(ActionResult {
                    accepted: false,
                    summary: reason.clone(),
                });
                self.history.push(action);
                rejected = true;
            } else {
                remaining.push_back(action);
            }
        }

        self.pending = remaining;
        rejected
    }
}

#[cfg(test)]
mod tests {
    use crate::action::{ActionCommand, ActorType, Quantization, TargetScope};

    use super::*;

    #[test]
    fn commits_only_actions_that_match_boundary() {
        let mut queue = ActionQueue::new();

        queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::CaptureNow,
                Quantization::NextBar,
                crate::action::ActionTarget {
                    scope: Some(TargetScope::LaneW30),
                    ..Default::default()
                },
            ),
            100,
        );

        queue.enqueue(
            ActionDraft::new(
                ActorType::Ghost,
                ActionCommand::GhostSetMode,
                Quantization::Immediate,
                crate::action::ActionTarget {
                    scope: Some(TargetScope::Ghost),
                    ..Default::default()
                },
            ),
            101,
        );

        let committed = queue.commit_ready(CommitBoundary::Immediate, 110);
        assert_eq!(committed.len(), 1);
        assert_eq!(queue.pending_actions().len(), 1);

        let committed = queue.commit_ready(CommitBoundary::Bar, 200);
        assert_eq!(committed.len(), 1);
        assert_eq!(queue.pending_actions().len(), 0);
        assert_eq!(queue.history().len(), 2);
    }

    #[test]
    fn can_reject_pending_action() {
        let mut queue = ActionQueue::new();
        let id = queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::MutateScene,
                Quantization::NextPhrase,
                crate::action::ActionTarget {
                    scope: Some(TargetScope::Scene),
                    ..Default::default()
                },
            ),
            100,
        );

        assert!(queue.reject(id, "scene locked"));
        assert_eq!(queue.pending_actions().len(), 0);
        assert_eq!(queue.history().len(), 1);
        assert_eq!(queue.history()[0].status, ActionStatus::Rejected);
    }
}
