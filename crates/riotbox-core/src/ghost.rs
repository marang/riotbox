use serde::{Deserialize, Serialize};

use crate::{
    action::{ActionCommand, ActionTarget, GhostMode, Quantization},
    session::GhostSuggestionRecord,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GhostWatchSuggestion {
    pub proposal_id: String,
    pub mode: GhostMode,
    pub tool_name: GhostWatchTool,
    pub summary: String,
    pub rationale: String,
    pub suggested_action: Option<GhostSuggestedAction>,
    pub confidence: GhostSuggestionConfidence,
    pub safety: GhostSuggestionSafety,
    pub blockers: Vec<GhostSuggestionBlocker>,
    pub created_at: String,
}

impl GhostWatchSuggestion {
    #[must_use]
    pub fn is_read_only(&self) -> bool {
        matches!(self.mode, GhostMode::Watch)
    }

    #[must_use]
    pub fn is_blocked(&self) -> bool {
        matches!(self.safety, GhostSuggestionSafety::Blocked) || !self.blockers.is_empty()
    }

    #[must_use]
    pub fn archive_record(&self) -> GhostSuggestionRecord {
        GhostSuggestionRecord {
            proposal_id: self.proposal_id.clone(),
            summary: self.summary.clone(),
            accepted: false,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GhostWatchTool {
    InspectJamState,
    InspectSourceSummary,
    InspectRecentActions,
    InspectHealth,
    SuggestCapture,
    SuggestSceneMutation,
    SuggestMacroShift,
    SuggestRestore,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GhostSuggestedAction {
    pub command: ActionCommand,
    pub target: ActionTarget,
    pub quantization: Quantization,
    pub intent: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GhostSuggestionConfidence {
    Low,
    Medium,
    High,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GhostSuggestionSafety {
    SafeToSuggest,
    NeedsAssistAcceptance,
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GhostSuggestionBlocker {
    pub kind: GhostSuggestionBlockerKind,
    pub object_id: Option<String>,
    pub reason: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GhostSuggestionBlockerKind {
    Lock,
    Budget,
    LowConfidence,
    MissingEvidence,
    PendingConflict,
}

#[cfg(test)]
mod tests {
    use crate::action::TargetScope;

    use super::*;

    #[test]
    fn watch_suggestion_roundtrips_via_json() {
        let suggestion = source_backed_capture_suggestion();

        let json = serde_json::to_string_pretty(&suggestion).expect("serialize suggestion");
        let decoded: GhostWatchSuggestion =
            serde_json::from_str(&json).expect("deserialize suggestion");

        assert_eq!(decoded, suggestion);
        assert!(decoded.is_read_only());
        assert!(!decoded.is_blocked());
        assert_eq!(decoded.tool_name, GhostWatchTool::SuggestCapture);
        assert_eq!(
            decoded
                .suggested_action
                .as_ref()
                .expect("suggested action")
                .command,
            ActionCommand::CaptureNow
        );
    }

    #[test]
    fn blocked_watch_suggestion_surfaces_lock_reason() {
        let mut suggestion = source_backed_capture_suggestion();
        suggestion.safety = GhostSuggestionSafety::Blocked;
        suggestion.blockers = vec![GhostSuggestionBlocker {
            kind: GhostSuggestionBlockerKind::Lock,
            object_id: Some("lane.w30".into()),
            reason: "W-30 lane is locked by the performer".into(),
        }];

        assert!(suggestion.is_blocked());
        assert_eq!(
            suggestion.blockers[0].kind,
            GhostSuggestionBlockerKind::Lock
        );
        assert_eq!(
            suggestion.blockers[0].object_id.as_deref(),
            Some("lane.w30")
        );
    }

    #[test]
    fn watch_suggestion_archives_without_queue_action_shape() {
        let suggestion = source_backed_capture_suggestion();

        let record = suggestion.archive_record();
        let record_json = serde_json::to_value(&record).expect("record json");

        assert_eq!(record.proposal_id, "ghost-watch-1");
        assert_eq!(record.summary, "capture the current source-backed hit");
        assert!(!record.accepted);
        assert!(record_json.get("command").is_none());
        assert!(record_json.get("status").is_none());
        assert!(record_json.get("quantization").is_none());
    }

    fn source_backed_capture_suggestion() -> GhostWatchSuggestion {
        GhostWatchSuggestion {
            proposal_id: "ghost-watch-1".into(),
            mode: GhostMode::Watch,
            tool_name: GhostWatchTool::SuggestCapture,
            summary: "capture the current source-backed hit".into(),
            rationale: "W-30 readiness is source-backed and no capture is pending".into(),
            suggested_action: Some(GhostSuggestedAction {
                command: ActionCommand::CaptureNow,
                target: ActionTarget {
                    scope: Some(TargetScope::LaneW30),
                    ..Default::default()
                },
                quantization: Quantization::NextBar,
                intent: "store a reusable W-30 pad candidate".into(),
            }),
            confidence: GhostSuggestionConfidence::Medium,
            safety: GhostSuggestionSafety::SafeToSuggest,
            blockers: Vec::new(),
            created_at: "2026-04-29T15:55:00Z".into(),
        }
    }
}
