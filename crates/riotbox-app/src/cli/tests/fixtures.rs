use riotbox_core::{
    ids::{AssetId, SourceId},
    source_graph::{
        AnalysisSummary, Candidate, CandidateType, DecodeProfile, GraphProvenance, QualityClass,
        SourceDescriptor, SourceGraph,
    },
};

pub(super) fn ghost_capture_candidate_graph() -> SourceGraph {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: SourceId::from("src-1"),
            path: "input.wav".into(),
            content_hash: "hash-1".into(),
            duration_seconds: 12.0,
            sample_rate: 44_100,
            channel_count: 2,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "0.1.0".into(),
            provider_set: vec!["decoded.wav_baseline".into()],
            generated_at: "2026-04-29T17:00:00Z".into(),
            source_hash: "hash-1".into(),
            analysis_seed: 1,
            run_notes: None,
        },
    );
    graph.candidates.push(Candidate {
        candidate_id: "capture-candidate-a".into(),
        candidate_type: CandidateType::CaptureCandidate,
        asset_ref: AssetId::from("asset-a"),
        score: 0.86,
        confidence: 0.88,
        tags: vec!["capture".into()],
        constraints: vec!["bar_aligned".into()],
        provenance_refs: vec!["provider:decoded.wav_baseline".into()],
    });
    graph.analysis_summary = AnalysisSummary {
        overall_confidence: 0.86,
        timing_quality: QualityClass::Medium,
        section_quality: QualityClass::Medium,
        loop_candidate_count: 0,
        hook_candidate_count: 0,
        break_rebuild_potential: QualityClass::Medium,
        warnings: Vec::new(),
    };
    graph
}
