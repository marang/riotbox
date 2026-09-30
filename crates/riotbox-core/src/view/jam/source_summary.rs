use crate::source_graph::AssetType;
use crate::source_graph::CandidateType;
use crate::source_graph::QualityClass;
use crate::source_graph::RelationshipType;
use crate::source_graph::SourceGraph;
use crate::view::jam::source_map::SourceMapView;
use crate::view::jam::source_timing_summary::SourceTimingSummaryView;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SourceSummaryView {
    pub source_id: String,
    pub bpm_estimate: Option<f32>,
    pub bpm_confidence: f32,
    pub timing: SourceTimingSummaryView,
    pub source_map: SourceMapView,
    pub section_count: usize,
    pub loop_candidate_count: usize,
    pub hook_candidate_count: usize,
    pub feral_scorecard: FeralScorecardView,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeralScorecardView {
    pub readiness: String,
    pub break_rebuild_potential: String,
    pub hook_fragment_count: usize,
    pub break_support_count: usize,
    pub quote_risk_count: usize,
    pub capture_candidate_count: usize,
    pub top_reason: String,
    pub warnings: Vec<String>,
}

impl Default for FeralScorecardView {
    fn default() -> Self {
        Self {
            readiness: "unknown".into(),
            break_rebuild_potential: "unknown".into(),
            hook_fragment_count: 0,
            break_support_count: 0,
            quote_risk_count: 0,
            capture_candidate_count: 0,
            top_reason: "no feral source graph".into(),
            warnings: Vec::new(),
        }
    }
}

impl FeralScorecardView {
    #[must_use]
    pub fn from_graph(graph: &SourceGraph) -> Self {
        let break_rebuild_potential =
            quality_class_label(graph.analysis_summary.break_rebuild_potential).to_string();
        let hook_fragment_count = graph
            .assets
            .iter()
            .filter(|asset| asset.asset_type == AssetType::HookFragment)
            .count();
        let break_support_count = graph
            .relationships
            .iter()
            .filter(|relationship| {
                relationship.relation_type == RelationshipType::SupportsBreakRebuild
            })
            .count();
        let quote_risk_count = graph
            .relationships
            .iter()
            .filter(|relationship| {
                relationship.relation_type == RelationshipType::HighQuoteRiskWith
            })
            .count();
        let capture_candidate_count = graph
            .candidates
            .iter()
            .filter(|candidate| candidate.candidate_type == CandidateType::CaptureCandidate)
            .count();
        let readiness = feral_readiness(
            graph,
            hook_fragment_count,
            break_support_count,
            capture_candidate_count,
        )
        .to_string();
        let top_reason = feral_top_reason(
            graph.analysis_summary.break_rebuild_potential,
            hook_fragment_count,
            break_support_count,
            quote_risk_count,
            capture_candidate_count,
        )
        .to_string();
        let warnings = feral_scorecard_warnings(graph, hook_fragment_count, quote_risk_count);

        Self {
            readiness,
            break_rebuild_potential,
            hook_fragment_count,
            break_support_count,
            quote_risk_count,
            capture_candidate_count,
            top_reason,
            warnings,
        }
    }
}

fn feral_readiness(
    graph: &SourceGraph,
    hook_fragment_count: usize,
    break_support_count: usize,
    capture_candidate_count: usize,
) -> &'static str {
    if graph.has_feral_break_support_evidence() {
        "ready"
    } else if graph.analysis_summary.break_rebuild_potential == QualityClass::High
        && break_support_count == 0
    {
        "needs support"
    } else if graph.analysis_summary.break_rebuild_potential == QualityClass::High
        && hook_fragment_count == 0
        && capture_candidate_count == 0
        && graph.analysis_summary.hook_candidate_count == 0
        && graph.hook_candidate_count() == 0
    {
        "needs hook/capture"
    } else {
        "not ready"
    }
}

fn quality_class_label(quality: QualityClass) -> &'static str {
    match quality {
        QualityClass::Low => "low",
        QualityClass::Medium => "medium",
        QualityClass::High => "high",
        QualityClass::Unknown => "unknown",
    }
}

fn feral_top_reason(
    break_rebuild_potential: QualityClass,
    hook_fragment_count: usize,
    break_support_count: usize,
    quote_risk_count: usize,
    capture_candidate_count: usize,
) -> &'static str {
    if quote_risk_count > 0 && capture_candidate_count > 0 {
        "use capture before quoting"
    } else if quote_risk_count > 0 {
        "quote guard needed"
    } else if break_rebuild_potential == QualityClass::High && break_support_count > 0 {
        "break rebuild ready"
    } else if capture_candidate_count > 0 {
        "capture candidates ready"
    } else if hook_fragment_count > 0 {
        "hook fragments ready"
    } else {
        "feral evidence sparse"
    }
}

fn feral_scorecard_warnings(
    graph: &SourceGraph,
    hook_fragment_count: usize,
    quote_risk_count: usize,
) -> Vec<String> {
    let mut warnings = graph
        .analysis_summary
        .warnings
        .iter()
        .map(|warning| warning.code.clone())
        .collect::<Vec<_>>();

    if quote_risk_count > 0 {
        warnings.push(format!("quote risk {quote_risk_count}"));
    }

    if hook_fragment_count == 0 {
        warnings.push("no hook fragments".into());
    }

    warnings
}
