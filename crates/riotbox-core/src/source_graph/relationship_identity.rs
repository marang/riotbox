//! V1 wire endpoints stay strings; only the graph catalog can establish their kind.
use std::{collections::HashMap, error::Error, fmt};

use serde::{
    Deserialize, Deserializer, Serialize, Serializer, de::Error as _, ser::SerializeStruct,
};

use super::{
    AnalysisSummary, Asset, Candidate, GraphProvenance, PhraseAudioFeatures, Relationship,
    RelationshipType, Section, SourceDescriptor, SourceGraph, SourceGraphVersion,
    SourceMapEvidence, TimingModel, W30HookCandidateEvidence,
};
use crate::ids::{AssetId, CandidateId, SectionId, SourceId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphNodeRef {
    Source(SourceId),
    Section(SectionId),
    Asset(AssetId),
    Candidate(CandidateId),
}

impl GraphNodeRef {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Source(id) => id.as_str(),
            Self::Section(id) => id.as_str(),
            Self::Asset(id) => id.as_str(),
            Self::Candidate(id) => id.as_str(),
        }
    }
}

impl Serialize for GraphNodeRef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        validate_id(self.as_str()).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphNodeRefError {
    Malformed(String),
    Unknown(String),
    Ambiguous(String),
    KindMismatch(String),
}

impl fmt::Display for GraphNodeRefError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (reason, id) = match self {
            Self::Malformed(id) => ("malformed", id),
            Self::Unknown(id) => ("unknown", id),
            Self::Ambiguous(id) => ("ambiguous", id),
            Self::KindMismatch(id) => ("kind mismatch for", id),
        };
        write!(f, "{reason} Source Graph relationship endpoint {id:?}")
    }
}

impl Error for GraphNodeRefError {}

fn validate_id(id: &str) -> Result<(), GraphNodeRefError> {
    if id.trim().is_empty() || id.chars().any(char::is_control) {
        return Err(GraphNodeRefError::Malformed(id.to_owned()));
    }
    Ok(())
}

struct Catalog(HashMap<String, Option<GraphNodeRef>>);

impl Catalog {
    fn new(graph: &SourceGraph) -> Self {
        let nodes = std::iter::once(GraphNodeRef::Source(graph.source.source_id.clone()))
            .chain(
                graph
                    .sections
                    .iter()
                    .map(|s| GraphNodeRef::Section(s.section_id.clone())),
            )
            .chain(
                graph
                    .assets
                    .iter()
                    .map(|a| GraphNodeRef::Asset(a.asset_id.clone())),
            )
            .chain(
                graph
                    .candidates
                    .iter()
                    .map(|c| GraphNodeRef::Candidate(c.candidate_id.clone())),
            );
        let mut catalog = HashMap::new();
        for node in nodes {
            catalog
                .entry(node.as_str().to_owned())
                .and_modify(|entry| *entry = None)
                .or_insert(Some(node));
        }
        Self(catalog)
    }

    fn resolve(&self, id: &str) -> Result<GraphNodeRef, GraphNodeRefError> {
        validate_id(id)?;
        match self.0.get(id) {
            Some(Some(node)) => Ok(node.clone()),
            Some(None) => Err(GraphNodeRefError::Ambiguous(id.to_owned())),
            None => Err(GraphNodeRefError::Unknown(id.to_owned())),
        }
    }
}

impl SourceGraph {
    /// Validate typed in-memory edits against the same catalog used on ingest/restore.
    pub fn validate_relationship_endpoints(&self) -> Result<(), GraphNodeRefError> {
        let catalog = Catalog::new(self);
        for relation in &self.relationships {
            for endpoint in [&relation.from_id, &relation.to_id] {
                if catalog.resolve(endpoint.as_str())? != *endpoint {
                    return Err(GraphNodeRefError::KindMismatch(
                        endpoint.as_str().to_owned(),
                    ));
                }
            }
        }
        Ok(())
    }
}

impl Serialize for SourceGraph {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.validate_relationship_endpoints()
            .map_err(serde::ser::Error::custom)?;
        // Exhaustive destructuring makes a future graph-field addition update
        // this boundary explicitly. Keep the established V1 field order.
        let Self {
            graph_version,
            source,
            timing,
            source_map,
            phrase_audio_features,
            w30_hook_candidates,
            sections,
            assets,
            candidates,
            relationships,
            analysis_summary,
            provenance,
        } = self;
        let mut state = serializer.serialize_struct("SourceGraph", 12)?;
        state.serialize_field("graph_version", graph_version)?;
        state.serialize_field("source", source)?;
        state.serialize_field("timing", timing)?;
        state.serialize_field("source_map", source_map)?;
        state.serialize_field("phrase_audio_features", phrase_audio_features)?;
        state.serialize_field("w30_hook_candidates", w30_hook_candidates)?;
        state.serialize_field("sections", sections)?;
        state.serialize_field("assets", assets)?;
        state.serialize_field("candidates", candidates)?;
        state.serialize_field("relationships", relationships)?;
        state.serialize_field("analysis_summary", analysis_summary)?;
        state.serialize_field("provenance", provenance)?;
        state.end()
    }
}

// Deserialization-only DTOs preserve the V1 JSON shape without admitting an
// unresolved string endpoint into the product model. Serialization stays on the
// original SourceGraph field order, so existing canonical hashes do not change.
#[derive(Deserialize)]
#[cfg_attr(test, derive(Serialize))]
struct GraphWire {
    graph_version: SourceGraphVersion,
    source: SourceDescriptor,
    timing: TimingModel,
    #[serde(default)]
    source_map: SourceMapEvidence,
    #[serde(default)]
    phrase_audio_features: Vec<PhraseAudioFeatures>,
    #[serde(default)]
    w30_hook_candidates: Vec<W30HookCandidateEvidence>,
    sections: Vec<Section>,
    assets: Vec<Asset>,
    candidates: Vec<Candidate>,
    relationships: Vec<RelationshipWire>,
    analysis_summary: AnalysisSummary,
    provenance: GraphProvenance,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(Serialize))]
struct RelationshipWire {
    relation_type: RelationshipType,
    from_id: String,
    to_id: String,
    weight: f32,
    notes: Option<String>,
}

impl<'de> Deserialize<'de> for SourceGraph {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = GraphWire::deserialize(deserializer)?;
        let mut graph = Self {
            graph_version: wire.graph_version,
            source: wire.source,
            timing: wire.timing,
            source_map: wire.source_map,
            phrase_audio_features: wire.phrase_audio_features,
            w30_hook_candidates: wire.w30_hook_candidates,
            sections: wire.sections,
            assets: wire.assets,
            candidates: wire.candidates,
            relationships: Vec::new(),
            analysis_summary: wire.analysis_summary,
            provenance: wire.provenance,
        };
        let catalog = Catalog::new(&graph);
        graph.relationships = wire
            .relationships
            .into_iter()
            .map(|relation| {
                Ok(Relationship {
                    relation_type: relation.relation_type,
                    from_id: catalog
                        .resolve(&relation.from_id)
                        .map_err(D::Error::custom)?,
                    to_id: catalog.resolve(&relation.to_id).map_err(D::Error::custom)?,
                    weight: relation.weight,
                    notes: relation.notes,
                })
            })
            .collect::<Result<_, D::Error>>()?;
        Ok(graph)
    }
}

#[cfg(test)]
#[path = "relationship_identity_tests.rs"]
mod tests;
