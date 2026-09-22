use super::GraphWire;
use crate::source_graph::{
    Asset, AssetType, Candidate, CandidateType, DecodeProfile, GraphNodeRef, GraphNodeRefError,
    GraphProvenance, Relationship, RelationshipType, SourceDescriptor, SourceGraph,
};

fn graph() -> SourceGraph {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: "source".into(),
            path: "metadata-only.wav".into(),
            content_hash: "test".into(),
            duration_seconds: 1.0,
            sample_rate: 48_000,
            channel_count: 2,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "test".into(),
            provider_set: vec![],
            generated_at: "test".into(),
            source_hash: "test".into(),
            analysis_seed: 0,
            run_notes: None,
        },
    );
    // IDs are opaque, not a prefix-based kind discriminator.
    graph.assets.push(Asset {
        asset_id: "section-looking-asset".into(),
        asset_type: AssetType::LoopWindow,
        start_seconds: 0.0,
        end_seconds: 1.0,
        start_bar: 1,
        end_bar: 1,
        confidence: 1.0,
        tags: vec![],
        source_refs: vec![],
    });
    graph.candidates.push(Candidate {
        candidate_id: "candidate".into(),
        candidate_type: CandidateType::LoopCandidate,
        asset_ref: "section-looking-asset".into(),
        score: 1.0,
        confidence: 1.0,
        tags: vec![],
        constraints: vec![],
        provenance_refs: vec![],
    });
    graph.relationships.push(Relationship {
        relation_type: RelationshipType::VariantOf,
        from_id: GraphNodeRef::Candidate("candidate".into()),
        to_id: GraphNodeRef::Asset("section-looking-asset".into()),
        weight: 0.75,
        notes: None,
    });
    graph
}

#[test]
fn legacy_endpoint_strings_keep_bytes_and_resolve_kind_from_the_catalog() {
    let graph = graph();
    let bytes = serde_json::to_vec(&graph).unwrap();
    let wire: GraphWire = serde_json::from_slice(&bytes).unwrap();
    // The frozen V1 field order and raw-string relationship serializer.
    assert_eq!(serde_json::to_vec(&wire).unwrap(), bytes);
    let restored: SourceGraph = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored, graph);
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
}

#[test]
fn unknown_malformed_ambiguous_and_non_string_legacy_endpoints_are_rejected() {
    let baseline = serde_json::to_value(graph()).unwrap();
    for (endpoint, expected) in [
        ("missing", "unknown"),
        ("", "malformed"),
        ("bad\nname", "malformed"),
    ] {
        let mut wire = baseline.clone();
        wire["relationships"][0]["to_id"] = endpoint.into();
        let error = serde_json::from_value::<SourceGraph>(wire).unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
    }
    let mut wire = baseline.clone();
    wire["source"]["source_id"] = "section-looking-asset".into();
    assert!(
        serde_json::from_value::<SourceGraph>(wire)
            .unwrap_err()
            .to_string()
            .contains("ambiguous")
    );
    let mut wire = baseline;
    wire["relationships"][0]["to_id"] = serde_json::json!({"Asset": "section-looking-asset"});
    assert!(serde_json::from_value::<SourceGraph>(wire).is_err());
}

#[test]
fn in_memory_wrong_kind_and_dangling_endpoint_cannot_be_persisted() {
    let mut graph = graph();
    graph.relationships[0].to_id = GraphNodeRef::Source("section-looking-asset".into());
    assert_eq!(
        graph.validate_relationship_endpoints(),
        Err(GraphNodeRefError::KindMismatch(
            "section-looking-asset".into()
        ))
    );
    assert!(serde_json::to_vec(&graph).is_err());
    graph.relationships[0].to_id = GraphNodeRef::Source("source".into());
    assert!(serde_json::to_vec(&graph).is_ok());
    graph.candidates.clear();
    assert!(matches!(
        graph.validate_relationship_endpoints(),
        Err(GraphNodeRefError::Unknown(_))
    ));
}

#[test]
fn duplicate_same_kind_nodes_are_not_resolved_by_first_match() {
    let mut graph = graph();
    graph.assets.push(graph.assets[0].clone());
    assert!(matches!(
        graph.validate_relationship_endpoints(),
        Err(GraphNodeRefError::Ambiguous(_))
    ));
}
