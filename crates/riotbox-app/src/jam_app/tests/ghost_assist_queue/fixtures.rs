use crate::jam_app::tests::fixtures::session_source::sample_graph;
use riotbox_core::action::ActionCommand;
use riotbox_core::action::ActionTarget;
use riotbox_core::action::GhostMode;
use riotbox_core::action::Quantization;
use riotbox_core::action::TargetScope;
use riotbox_core::ghost::GhostSuggestedAction;
use riotbox_core::ghost::GhostSuggestionConfidence;
use riotbox_core::ghost::GhostSuggestionSafety;
use riotbox_core::ghost::GhostWatchSuggestion;
use riotbox_core::ghost::GhostWatchTool;
use riotbox_core::source_graph::Candidate;
use riotbox_core::source_graph::CandidateType;
use riotbox_core::source_graph::SourceGraph;

pub(in crate::jam_app::tests) fn ghost_fill_suggestion() -> GhostWatchSuggestion {
    GhostWatchSuggestion {
        proposal_id: "ghost-fill-1".into(),
        mode: GhostMode::Watch,
        tool_name: GhostWatchTool::SuggestMacroShift,
        summary: "add a drum answer".into(),
        rationale: "TR-909 has room for a next-bar support move".into(),
        suggested_action: Some(GhostSuggestedAction {
            command: ActionCommand::Tr909FillNext,
            target: ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
            quantization: Quantization::NextBar,
            intent: "add a next-bar drum answer".into(),
        }),
        confidence: GhostSuggestionConfidence::Medium,
        safety: GhostSuggestionSafety::SafeToSuggest,
        blockers: Vec::new(),
        created_at: "2026-04-29T17:00:00Z".into(),
    }
}

pub(in crate::jam_app::tests) fn ghost_destructive_takeover_suggestion(
    proposal_id: &str,
) -> GhostWatchSuggestion {
    let mut suggestion = ghost_fill_suggestion();
    suggestion.proposal_id = proposal_id.into();
    suggestion.tool_name = GhostWatchTool::SuggestSceneMutation;
    suggestion.summary = "take over the current scene".into();
    suggestion.rationale = "TR-909 can safely carry the next phrase".into();
    suggestion.suggested_action = Some(GhostSuggestedAction {
        command: ActionCommand::Tr909Takeover,
        target: ActionTarget {
            scope: Some(TargetScope::LaneTr909),
            ..Default::default()
        },
        quantization: Quantization::NextPhrase,
        intent: "take over the next phrase with TR-909".into(),
    });
    suggestion
}

pub(in crate::jam_app::tests) fn ghost_capture_candidate_graph() -> SourceGraph {
    let mut graph = sample_graph();
    graph.candidates.push(Candidate {
        candidate_id: "capture-candidate-a".into(),
        candidate_type: CandidateType::CaptureCandidate,
        asset_ref: "asset-a".into(),
        score: 0.86,
        confidence: 0.88,
        tags: vec!["capture".into(), "feral".into()],
        constraints: vec!["bar_aligned".into()],
        provenance_refs: vec!["provider:decoded.wav_baseline".into()],
    });
    graph
}
