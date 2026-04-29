use riotbox_core::{
    TimestampMs,
    action::GhostMode,
    ghost::{GhostSuggestionDraftError, GhostWatchSuggestion},
    ids::ActionId,
    session::GhostSuggestionStatus,
};

use super::JamAppState;

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
                reason: "no current ghost suggestion".into(),
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
