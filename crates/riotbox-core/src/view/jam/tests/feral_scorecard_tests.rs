use crate::source_graph::AnalysisSummary;
use crate::source_graph::AnalysisWarning;
use crate::source_graph::Asset;
use crate::source_graph::AssetType;
use crate::source_graph::Candidate;
use crate::source_graph::CandidateType;
use crate::source_graph::DecodeProfile;
use crate::source_graph::GraphProvenance;
use crate::source_graph::QualityClass;
use crate::source_graph::Relationship;
use crate::source_graph::RelationshipType;
use crate::source_graph::SourceDescriptor;
use crate::source_graph::SourceGraph;
use crate::view::jam::source_summary::FeralScorecardView;

#[test]
fn builds_feral_scorecard_from_source_graph_evidence() {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: "src-feral".into(),
            path: "input.wav".into(),
            content_hash: "hash-feral".into(),
            duration_seconds: 32.0,
            sample_rate: 48_000,
            channel_count: 2,
            decode_profile: DecodeProfile::NormalizedStereo,
        },
        GraphProvenance {
            sidecar_version: "0.1.0".into(),
            provider_set: vec!["fixture".into()],
            generated_at: "2026-04-26T13:20:00Z".into(),
            source_hash: "hash-feral".into(),
            analysis_seed: 329,
            run_notes: None,
        },
    );
    graph.assets.push(Asset {
        asset_id: "asset-hook".into(),
        asset_type: AssetType::HookFragment,
        start_seconds: 1.0,
        end_seconds: 2.0,
        start_bar: 1,
        end_bar: 1,
        confidence: 0.9,
        tags: vec!["hook".into()],
        source_refs: vec!["src-feral".into()],
    });
    graph.candidates.push(Candidate {
        candidate_id: "candidate-capture".into(),
        candidate_type: CandidateType::CaptureCandidate,
        asset_ref: "asset-hook".into(),
        score: 0.87,
        confidence: 0.81,
        tags: vec!["feral".into()],
        constraints: vec!["capture_first".into()],
        provenance_refs: vec!["fixture".into()],
    });
    graph.relationships.push(Relationship {
        relation_type: RelationshipType::SupportsBreakRebuild,
        from_id: crate::source_graph::GraphNodeRef::Asset("asset-hook".into()),
        to_id: crate::source_graph::GraphNodeRef::Section("section-break".into()),
        weight: 0.85,
        notes: None,
    });
    graph.relationships.push(Relationship {
        relation_type: RelationshipType::HighQuoteRiskWith,
        from_id: crate::source_graph::GraphNodeRef::Asset("asset-hook".into()),
        to_id: crate::source_graph::GraphNodeRef::Source("src-feral".into()),
        weight: 0.7,
        notes: None,
    });
    graph.analysis_summary = AnalysisSummary {
        overall_confidence: 0.82,
        timing_quality: QualityClass::High,
        section_quality: QualityClass::Medium,
        loop_candidate_count: 0,
        hook_candidate_count: 0,
        break_rebuild_potential: QualityClass::High,
        warnings: vec![AnalysisWarning {
            code: "fixture_warning".into(),
            message: "fixture warning".into(),
        }],
    };

    let scorecard = FeralScorecardView::from_graph(&graph);

    assert_eq!(scorecard.readiness, "ready");
    assert_eq!(scorecard.break_rebuild_potential, "high");
    assert_eq!(scorecard.hook_fragment_count, 1);
    assert_eq!(scorecard.break_support_count, 1);
    assert_eq!(scorecard.quote_risk_count, 1);
    assert_eq!(scorecard.capture_candidate_count, 1);
    assert_eq!(scorecard.top_reason, "use capture before quoting");
    assert_eq!(
        scorecard.warnings,
        vec!["fixture_warning".to_string(), "quote risk 1".to_string()]
    );
}

#[test]
fn feral_scorecard_readiness_uses_shared_break_support_evidence() {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: "src-feral".into(),
            path: "input.wav".into(),
            content_hash: "hash-feral".into(),
            duration_seconds: 32.0,
            sample_rate: 48_000,
            channel_count: 2,
            decode_profile: DecodeProfile::NormalizedStereo,
        },
        GraphProvenance {
            sidecar_version: "0.1.0".into(),
            provider_set: vec!["fixture".into()],
            generated_at: "2026-04-26T13:20:00Z".into(),
            source_hash: "hash-feral".into(),
            analysis_seed: 336,
            run_notes: None,
        },
    );
    graph.analysis_summary.break_rebuild_potential = QualityClass::High;
    graph.assets.push(Asset {
        asset_id: "asset-hook".into(),
        asset_type: AssetType::HookFragment,
        start_seconds: 1.0,
        end_seconds: 2.0,
        start_bar: 1,
        end_bar: 1,
        confidence: 0.9,
        tags: vec!["hook".into()],
        source_refs: vec!["src-feral".into()],
    });
    graph.candidates.push(Candidate {
        candidate_id: "candidate-capture".into(),
        candidate_type: CandidateType::CaptureCandidate,
        asset_ref: "asset-hook".into(),
        score: 0.87,
        confidence: 0.81,
        tags: vec!["feral".into()],
        constraints: vec!["capture_first".into()],
        provenance_refs: vec!["fixture".into()],
    });

    assert_eq!(
        FeralScorecardView::from_graph(&graph).readiness,
        "needs support"
    );

    graph.relationships.push(Relationship {
        relation_type: RelationshipType::SupportsBreakRebuild,
        from_id: crate::source_graph::GraphNodeRef::Asset("asset-hook".into()),
        to_id: crate::source_graph::GraphNodeRef::Section("section-break".into()),
        weight: 0.85,
        notes: None,
    });

    assert_eq!(FeralScorecardView::from_graph(&graph).readiness, "ready");
}
