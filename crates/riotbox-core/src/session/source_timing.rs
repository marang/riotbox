//! Session-owned timing confirmation, shared by policy and presentation.
use serde::{Deserialize, Serialize};

use crate::{
    TimestampMs,
    ids::{ActionId, SourceId},
    source_graph::SourceGraph,
};

use super::SessionFile;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SourceTimingRuntimeState {
    #[serde(default)]
    pub confirmed_grid: Option<SourceTimingGridConfirmationState>,
    #[serde(default)]
    pub confirmed_bpm: Option<f32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceTimingGridConfirmationState {
    pub source_id: SourceId,
    pub hypothesis_id: Option<String>,
    pub confirmed_by_action: ActionId,
    pub confirmed_at: TimestampMs,
}

#[must_use]
pub fn source_timing_confirmation_matches_graph(
    graph: &SourceGraph,
    session: &SessionFile,
) -> bool {
    session
        .runtime_state
        .source_timing
        .confirmed_grid
        .as_ref()
        .is_some_and(|confirmed| {
            confirmed.source_id == graph.source.source_id
                && confirmed.hypothesis_id.as_deref()
                    == graph.timing.primary_hypothesis_id.as_deref()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_graph::{DecodeProfile, GraphProvenance, SourceDescriptor};

    #[test]
    fn confirmation_matches_source_and_optional_hypothesis_without_view_dependency() {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: "source-a".into(),
                path: "metadata-only.wav".into(),
                content_hash: "synthetic-metadata".into(),
                duration_seconds: 8.0,
                sample_rate: 48_000,
                channel_count: 1,
                decode_profile: DecodeProfile::Native,
            },
            GraphProvenance {
                sidecar_version: "test".into(),
                provider_set: vec![],
                generated_at: "test".into(),
                source_hash: "synthetic-metadata".into(),
                analysis_seed: 1,
                run_notes: None,
            },
        );
        let mut session = SessionFile::new("session", "test", "test");
        assert!(!source_timing_confirmation_matches_graph(&graph, &session));
        for (source, hypothesis, primary, matches) in [
            ("source-a", None, None, true),
            ("source-b", None, None, false),
            ("source-a", Some("grid-a"), Some("grid-a"), true),
            ("source-a", Some("grid-a"), Some("grid-b"), false),
            ("source-a", None, Some("grid-a"), false),
            ("source-a", Some("grid-a"), None, false),
        ] {
            graph.timing.primary_hypothesis_id = primary.map(str::to_owned);
            session.runtime_state.source_timing.confirmed_grid =
                Some(SourceTimingGridConfirmationState {
                    source_id: source.into(),
                    hypothesis_id: hypothesis.map(str::to_owned),
                    confirmed_by_action: ActionId(7),
                    confirmed_at: 8,
                });
            assert_eq!(
                source_timing_confirmation_matches_graph(&graph, &session),
                matches
            );
            assert_eq!(
                crate::view::jam::source_timing_confirmation_matches_graph(&graph, &session),
                matches
            );
        }
    }

    #[test]
    fn moved_state_types_retain_their_wire_shape_and_defaults() {
        let empty: SourceTimingRuntimeState = serde_json::from_str("{}").unwrap();
        assert_eq!(empty, SourceTimingRuntimeState::default());
        let expected = serde_json::json!({
            "confirmed_grid": {
                "source_id": "source-a", "hypothesis_id": null,
                "confirmed_by_action": 7, "confirmed_at": 8,
            },
            "confirmed_bpm": 126.0,
        });
        let state: SourceTimingRuntimeState = serde_json::from_value(expected.clone()).unwrap();
        assert_eq!(serde_json::to_value(state).unwrap(), expected);
    }
}
