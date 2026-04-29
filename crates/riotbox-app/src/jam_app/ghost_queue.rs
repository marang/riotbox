use riotbox_core::{
    TimestampMs,
    action::{ActionStatus, ActorType, GhostMode},
    ghost::{GhostSuggestionDraftError, GhostWatchSuggestion},
    ids::ActionId,
    session::GhostSuggestionStatus,
};

use super::JamAppState;

pub const NO_CURRENT_GHOST_SUGGESTION_REASON: &str = "no current ghost suggestion";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GhostSuggestionQueueResult {
    Enqueued(ActionId),
    Rejected { reason: String },
}

impl JamAppState {
    pub fn set_current_ghost_suggestion(&mut self, suggestion: GhostWatchSuggestion) {
        if !self
            .session
            .ghost_state
            .suggestion_history
            .iter()
            .any(|record| record.proposal_id == suggestion.proposal_id)
        {
            self.session
                .ghost_state
                .suggestion_history
                .push(suggestion.archive_record());
        }

        self.runtime.current_ghost_suggestion = Some(suggestion);
        self.refresh_view();
    }

    pub fn clear_current_ghost_suggestion(&mut self) {
        self.runtime.current_ghost_suggestion = None;
        self.refresh_view();
    }

    pub fn accept_current_ghost_suggestion(
        &mut self,
        requested_at: TimestampMs,
    ) -> GhostSuggestionQueueResult {
        let Some(suggestion) = self.runtime.current_ghost_suggestion.clone() else {
            return GhostSuggestionQueueResult::Rejected {
                reason: NO_CURRENT_GHOST_SUGGESTION_REASON.into(),
            };
        };

        let result = self.queue_accepted_ghost_suggestion(&suggestion, requested_at);
        if matches!(result, GhostSuggestionQueueResult::Enqueued(_)) {
            self.runtime.current_ghost_suggestion = None;
        }
        result
    }

    pub fn reject_current_ghost_suggestion(&mut self) -> bool {
        let Some(suggestion) = self.runtime.current_ghost_suggestion.take() else {
            return false;
        };

        if !self
            .session
            .ghost_state
            .suggestion_history
            .iter()
            .any(|record| record.proposal_id == suggestion.proposal_id)
        {
            self.session
                .ghost_state
                .suggestion_history
                .push(suggestion.archive_record());
        }

        let rejected = self
            .session
            .ghost_state
            .reject_suggestion(&suggestion.proposal_id);
        self.refresh_view();
        rejected
    }

    pub fn queue_accepted_ghost_suggestion(
        &mut self,
        suggestion: &GhostWatchSuggestion,
        requested_at: TimestampMs,
    ) -> GhostSuggestionQueueResult {
        if matches!(
            self.session
                .ghost_state
                .suggestion_history
                .iter()
                .rev()
                .find(|record| record.proposal_id == suggestion.proposal_id)
                .map(|record| record.status()),
            Some(GhostSuggestionStatus::Accepted | GhostSuggestionStatus::Rejected)
        ) {
            return GhostSuggestionQueueResult::Rejected {
                reason: format!("ghost proposal {} already decided", suggestion.proposal_id),
            };
        }

        let draft = match suggestion.accepted_action_draft(self.session.ghost_state.mode) {
            Ok(draft) => draft,
            Err(error) => {
                return GhostSuggestionQueueResult::Rejected {
                    reason: ghost_queue_rejection_reason(error),
                };
            }
        };

        if !self.ghost_pending_action_budget_available() {
            return GhostSuggestionQueueResult::Rejected {
                reason: "ghost pending action budget exceeded".into(),
            };
        }

        if !self.ghost_phrase_action_budget_available() {
            return GhostSuggestionQueueResult::Rejected {
                reason: "ghost phrase action budget exceeded".into(),
            };
        }

        if !self
            .session
            .ghost_state
            .suggestion_history
            .iter()
            .any(|record| record.proposal_id == suggestion.proposal_id)
        {
            self.session
                .ghost_state
                .suggestion_history
                .push(suggestion.archive_record());
        }

        if !self
            .session
            .ghost_state
            .accept_suggestion(&suggestion.proposal_id)
        {
            return GhostSuggestionQueueResult::Rejected {
                reason: format!(
                    "ghost proposal {} could not be accepted",
                    suggestion.proposal_id
                ),
            };
        }

        let action_id = self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        GhostSuggestionQueueResult::Enqueued(action_id)
    }

    fn ghost_pending_action_budget_available(&self) -> bool {
        let max_pending = usize::from(self.session.ghost_state.budgets.max_pending_actions);
        let pending_ghost_actions = self
            .queue
            .pending_actions()
            .iter()
            .filter(|action| action.actor == ActorType::Ghost)
            .count();
        pending_ghost_actions < max_pending
    }

    fn ghost_phrase_action_budget_available(&self) -> bool {
        let max_actions = usize::from(self.session.ghost_state.budgets.max_actions_per_phrase);
        let current_phrase = self.runtime.transport.phrase_index;
        let committed_ghost_actions_in_phrase = self
            .session
            .action_log
            .commit_records
            .iter()
            .filter(|record| record.boundary.phrase_index == current_phrase)
            .filter(|record| {
                self.session.action_log.actions.iter().any(|action| {
                    action.id == record.action_id
                        && action.actor == ActorType::Ghost
                        && action.status == ActionStatus::Committed
                })
            })
            .count();
        let pending_ghost_actions = self
            .queue
            .pending_actions()
            .iter()
            .filter(|action| action.actor == ActorType::Ghost)
            .count();

        committed_ghost_actions_in_phrase + pending_ghost_actions < max_actions
    }
}

fn ghost_queue_rejection_reason(error: GhostSuggestionDraftError) -> String {
    match error {
        GhostSuggestionDraftError::AssistModeRequired => {
            format!("ghost accept requires {} mode", GhostMode::Assist)
        }
        GhostSuggestionDraftError::Blocked => "ghost proposal is blocked".into(),
        GhostSuggestionDraftError::MissingSuggestedAction => {
            "ghost proposal has no suggested action".into()
        }
    }
}
