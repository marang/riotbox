use riotbox_core::source_graph::{SourceDescriptor, SourceGraph};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

pub const PROTOCOL_VERSION: &str = "0.1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PingPayload {
    pub request_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PongPayload {
    pub request_id: String,
    pub protocol_version: String,
    pub sidecar_version: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BuildSourceGraphStubPayload {
    pub request_id: String,
    pub source: SourceDescriptor,
    pub analysis_seed: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceGraphBuiltPayload {
    pub request_id: String,
    pub graph: SourceGraph,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SidecarErrorPayload {
    pub request_id: Option<String>,
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SidecarRequest {
    Ping(PingPayload),
    BuildSourceGraphStub(BuildSourceGraphStubPayload),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SidecarResponse {
    Pong(PongPayload),
    SourceGraphBuilt(SourceGraphBuiltPayload),
    Error(SidecarErrorPayload),
}

#[derive(Debug)]
pub enum ProtocolError {
    Serialize(serde_json::Error),
    Deserialize(serde_json::Error),
}

impl Display for ProtocolError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialize(error) => write!(f, "protocol serialization failed: {error}"),
            Self::Deserialize(error) => write!(f, "protocol deserialization failed: {error}"),
        }
    }
}

impl Error for ProtocolError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Serialize(error) | Self::Deserialize(error) => Some(error),
        }
    }
}

pub fn encode_json_line<T: Serialize>(message: &T) -> Result<String, ProtocolError> {
    let mut encoded = serde_json::to_string(message).map_err(ProtocolError::Serialize)?;
    encoded.push('\n');
    Ok(encoded)
}

pub fn decode_json_line<T: DeserializeOwned>(line: &str) -> Result<T, ProtocolError> {
    serde_json::from_str(line).map_err(ProtocolError::Deserialize)
}

#[cfg(test)]
mod tests {
    use riotbox_core::{
        ids::{AssetId, CandidateId, SourceId},
        source_graph::{
            AnalysisSummary, Asset, AssetType, Candidate, CandidateType, DecodeProfile,
            GraphProvenance, QualityClass, SourceGraph, SourceGraphVersion,
        },
    };

    use super::*;

    fn sample_source() -> SourceDescriptor {
        SourceDescriptor {
            source_id: SourceId::from("src-transport-1"),
            path: "fixtures/break.wav".into(),
            content_hash: "sha256:abc123".into(),
            duration_seconds: 92.5,
            sample_rate: 48_000,
            channel_count: 2,
            decode_profile: DecodeProfile::NormalizedStereo,
        }
    }

    fn sample_graph() -> SourceGraph {
        let mut graph = SourceGraph::new(
            sample_source(),
            GraphProvenance {
                sidecar_version: "0.1.0".into(),
                provider_set: vec!["beats".into(), "sections".into()],
                generated_at: "2026-04-12T19:00:00Z".into(),
                source_hash: "sha256:abc123".into(),
                analysis_seed: 9,
                run_notes: Some("stub".into()),
            },
        );
        graph.graph_version = SourceGraphVersion::V1;
        graph.assets.push(Asset {
            asset_id: AssetId::from("asset-1"),
            asset_type: AssetType::LoopWindow,
            start_seconds: 0.0,
            end_seconds: 4.0,
            start_bar: 1,
            end_bar: 2,
            confidence: 0.88,
            tags: vec!["loop".into()],
            source_refs: vec!["src-transport-1".into()],
        });
        graph.candidates.push(Candidate {
            candidate_id: CandidateId::from("cand-1"),
            candidate_type: CandidateType::LoopCandidate,
            asset_ref: AssetId::from("asset-1"),
            score: 0.91,
            confidence: 0.89,
            tags: vec!["prototype".into()],
            constraints: vec!["bar_aligned".into()],
            provenance_refs: vec!["provider:stub".into()],
        });
        graph.analysis_summary = AnalysisSummary {
            overall_confidence: 0.86,
            timing_quality: QualityClass::High,
            section_quality: QualityClass::Medium,
            loop_candidate_count: 1,
            hook_candidate_count: 0,
            break_rebuild_potential: QualityClass::Medium,
            warnings: vec![],
        };
        graph
    }

    #[test]
    fn request_roundtrips_through_ndjson() {
        let request = SidecarRequest::BuildSourceGraphStub(BuildSourceGraphStubPayload {
            request_id: "req-1".into(),
            source: sample_source(),
            analysis_seed: 9,
        });

        let encoded = encode_json_line(&request).expect("encode request");
        let decoded: SidecarRequest =
            decode_json_line(&encoded).expect("decode build source graph request");

        assert_eq!(decoded, request);
        assert!(encoded.ends_with('\n'));
    }

    #[test]
    fn response_roundtrips_through_ndjson() {
        let response = SidecarResponse::SourceGraphBuilt(SourceGraphBuiltPayload {
            request_id: "req-1".into(),
            graph: sample_graph(),
        });

        let encoded = encode_json_line(&response).expect("encode response");
        let decoded: SidecarResponse =
            decode_json_line(&encoded).expect("decode build source graph response");

        assert_eq!(decoded, response);
    }
}
