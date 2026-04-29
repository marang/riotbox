use riotbox_core::{
    action::{ActionCommand, ActionTarget, GhostMode, Quantization, TargetScope},
    ghost::{
        GhostSuggestedAction, GhostSuggestionBlocker, GhostSuggestionBlockerKind,
        GhostSuggestionConfidence, GhostSuggestionSafety, GhostWatchSuggestion, GhostWatchTool,
    },
    queue::ActionQueue,
    session::{GhostSuggestionStatus, SessionFile},
    source_graph::{CandidateType, SourceGraph},
};

use super::JamAppState;

impl JamAppState {
    pub fn can_refresh_current_ghost_suggestion_from_jam_state(&self) -> bool {
        self.next_jam_state_ghost_suggestion().is_some()
    }

    pub fn refresh_current_ghost_suggestion_from_jam_state(&mut self) -> bool {
        let Some(suggestion) = self.next_jam_state_ghost_suggestion() else {
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

        self.runtime.current_ghost_suggestion = Some(suggestion);
        self.refresh_view();
        true
    }

    fn next_jam_state_ghost_suggestion(&self) -> Option<GhostWatchSuggestion> {
        if self.runtime.current_ghost_suggestion.is_some() {
            return None;
        }

        let suggestion = source_backed_capture_suggestion(
            &self.session,
            &self.queue,
            self.source_graph.as_ref(),
        )?;

        if ghost_suggestion_is_decided(&self.session, &suggestion.proposal_id) {
            return None;
        }

        Some(suggestion)
    }
}

fn source_backed_capture_suggestion(
    session: &SessionFile,
    queue: &ActionQueue,
    graph: Option<&SourceGraph>,
) -> Option<GhostWatchSuggestion> {
    if !matches!(session.ghost_state.mode, GhostMode::Assist) {
        return None;
    }

    let graph = graph?;
    if !source_graph_has_ghost_capture_evidence(graph)
        || !session.captures.is_empty()
        || !queue.pending_actions().is_empty()
    {
        return None;
    }

    let source_id = graph.source.source_id.to_string();
    let blockers = ghost_capture_blockers(session);
    let safety = if blockers.is_empty() {
        GhostSuggestionSafety::NeedsAssistAcceptance
    } else {
        GhostSuggestionSafety::Blocked
    };

    Some(GhostWatchSuggestion {
        proposal_id: format!("ghost-jam-capture-{source_id}"),
        mode: session.ghost_state.mode,
        tool_name: GhostWatchTool::SuggestCapture,
        summary: "capture the current source-backed hit".into(),
        rationale: ghost_capture_rationale(graph),
        suggested_action: Some(GhostSuggestedAction {
            command: ActionCommand::CaptureNow,
            target: ActionTarget {
                scope: Some(TargetScope::LaneW30),
                ..Default::default()
            },
            quantization: Quantization::NextBar,
            intent: "store a reusable W-30 pad candidate".into(),
        }),
        confidence: ghost_capture_confidence(graph),
        safety,
        blockers,
        created_at: graph.provenance.generated_at.clone(),
    })
}

fn source_graph_has_ghost_capture_evidence(graph: &SourceGraph) -> bool {
    graph.has_feral_break_support_evidence()
        || graph
            .candidates
            .iter()
            .any(|candidate| candidate.candidate_type == CandidateType::CaptureCandidate)
}

fn ghost_capture_blockers(session: &SessionFile) -> Vec<GhostSuggestionBlocker> {
    if session
        .runtime_state
        .lock_state
        .locked_object_ids
        .iter()
        .any(|object_id| object_id == "lane.w30")
    {
        return vec![GhostSuggestionBlocker {
            kind: GhostSuggestionBlockerKind::Lock,
            object_id: Some("lane.w30".into()),
            reason: "W-30 lane is locked by the performer".into(),
        }];
    }

    Vec::new()
}

fn ghost_capture_confidence(graph: &SourceGraph) -> GhostSuggestionConfidence {
    if graph.analysis_summary.overall_confidence >= 0.8 {
        GhostSuggestionConfidence::High
    } else if graph.analysis_summary.overall_confidence >= 0.55 {
        GhostSuggestionConfidence::Medium
    } else {
        GhostSuggestionConfidence::Low
    }
}

fn ghost_capture_rationale(graph: &SourceGraph) -> String {
    if graph.has_feral_break_support_evidence() {
        "source graph has break-rebuild evidence for a reusable W-30 capture".into()
    } else {
        "source graph has capture-candidate evidence for a reusable W-30 pad".into()
    }
}

fn ghost_suggestion_is_decided(session: &SessionFile, proposal_id: &str) -> bool {
    matches!(
        session
            .ghost_state
            .suggestion_history
            .iter()
            .rev()
            .find(|record| record.proposal_id == proposal_id)
            .map(|record| record.status()),
        Some(GhostSuggestionStatus::Accepted | GhostSuggestionStatus::Rejected)
    )
}
