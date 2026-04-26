use crate::{
    ids::{AssetId, CandidateId, SceneId, SectionId, SourceId},
    transport::TransportClockState,
};
use serde::{Deserialize, Serialize};

pub type Confidence = f32;

#[must_use]
pub fn section_for_transport_bar<'a>(
    graph: &'a SourceGraph,
    transport: &TransportClockState,
) -> Option<&'a Section> {
    graph.sections.iter().find(|section| {
        let bar_index = transport.bar_index as u32;
        bar_index >= section.bar_start && bar_index <= section.bar_end
    })
}

#[must_use]
pub fn section_for_projected_scene<'a>(
    graph: &'a SourceGraph,
    scene_id: &SceneId,
) -> Option<&'a Section> {
    let scene_index = parse_projected_scene_index(scene_id.as_str())?;
    let sections = sorted_sections(graph);
    sections.get(scene_index).copied()
}

fn parse_projected_scene_index(scene_id: &str) -> Option<usize> {
    let mut parts = scene_id.splitn(3, '-');
    match (parts.next(), parts.next(), parts.next()) {
        (Some("scene"), Some(index), Some(_label)) => index.parse::<usize>().ok()?.checked_sub(1),
        _ => None,
    }
}

#[must_use]
pub fn sorted_sections(graph: &SourceGraph) -> Vec<&Section> {
    let mut sections = graph.sections.iter().collect::<Vec<_>>();
    sections.sort_by(|left, right| {
        left.bar_start
            .cmp(&right.bar_start)
            .then(left.bar_end.cmp(&right.bar_end))
            .then(left.section_id.as_str().cmp(right.section_id.as_str()))
    });
    sections
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceGraphVersion {
    V1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceGraph {
    pub graph_version: SourceGraphVersion,
    pub source: SourceDescriptor,
    pub timing: TimingModel,
    pub sections: Vec<Section>,
    pub assets: Vec<Asset>,
    pub candidates: Vec<Candidate>,
    pub relationships: Vec<Relationship>,
    pub analysis_summary: AnalysisSummary,
    pub provenance: GraphProvenance,
}

impl SourceGraph {
    #[must_use]
    pub fn new(source: SourceDescriptor, provenance: GraphProvenance) -> Self {
        Self {
            graph_version: SourceGraphVersion::V1,
            source,
            timing: TimingModel::default(),
            sections: Vec::new(),
            assets: Vec::new(),
            candidates: Vec::new(),
            relationships: Vec::new(),
            analysis_summary: AnalysisSummary::default(),
            provenance,
        }
    }

    #[must_use]
    pub fn candidate_count(&self, candidate_type: CandidateType) -> usize {
        self.candidates
            .iter()
            .filter(|candidate| candidate.candidate_type == candidate_type)
            .count()
    }

    #[must_use]
    pub fn loop_candidate_count(&self) -> usize {
        self.candidate_count(CandidateType::LoopCandidate)
    }

    #[must_use]
    pub fn hook_candidate_count(&self) -> usize {
        self.candidate_count(CandidateType::HookCandidate)
    }

    #[must_use]
    pub fn warnings(&self) -> Vec<String> {
        self.analysis_summary
            .warnings
            .iter()
            .map(|warning| warning.message.clone())
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceDescriptor {
    pub source_id: SourceId,
    pub path: String,
    pub content_hash: String,
    pub duration_seconds: f32,
    pub sample_rate: u32,
    pub channel_count: u16,
    pub decode_profile: DecodeProfile,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecodeProfile {
    Native,
    NormalizedStereo,
    NormalizedMono,
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct TimingModel {
    pub bpm_estimate: Option<f32>,
    pub bpm_confidence: Confidence,
    pub meter_hint: Option<MeterHint>,
    pub beat_grid: Vec<BeatPoint>,
    pub bar_grid: Vec<BarSpan>,
    pub phrase_grid: Vec<PhraseSpan>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeterHint {
    pub beats_per_bar: u8,
    pub beat_unit: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BeatPoint {
    pub beat_index: u32,
    pub time_seconds: f32,
    pub confidence: Confidence,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BarSpan {
    pub bar_index: u32,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub downbeat_confidence: Confidence,
    pub phrase_index: Option<u32>,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhraseSpan {
    pub phrase_index: u32,
    pub start_bar: u32,
    pub end_bar: u32,
    pub confidence: Confidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub section_id: SectionId,
    pub label_hint: SectionLabelHint,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub bar_start: u32,
    pub bar_end: u32,
    pub energy_class: EnergyClass,
    pub confidence: Confidence,
    pub tags: Vec<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectionLabelHint {
    Intro,
    Build,
    Drop,
    Break,
    Verse,
    Chorus,
    Bridge,
    Outro,
    Unknown,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnergyClass {
    Low,
    Medium,
    High,
    Peak,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    pub asset_id: AssetId,
    pub asset_type: AssetType,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub start_bar: u32,
    pub end_bar: u32,
    pub confidence: Confidence,
    pub tags: Vec<String>,
    pub source_refs: Vec<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    Slice,
    LoopWindow,
    HookFragment,
    DrumAnchor,
    PhraseFragment,
    TextureFragment,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    pub candidate_id: CandidateId,
    pub candidate_type: CandidateType,
    pub asset_ref: AssetId,
    pub score: f32,
    pub confidence: Confidence,
    pub tags: Vec<String>,
    pub constraints: Vec<String>,
    pub provenance_refs: Vec<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateType {
    KickAnchor,
    SnareAnchor,
    GhostHit,
    FillFragment,
    LoopCandidate,
    HookCandidate,
    AnswerCandidate,
    CaptureCandidate,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Relationship {
    pub relation_type: RelationshipType,
    pub from_id: String,
    pub to_id: String,
    pub weight: f32,
    pub notes: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipType {
    BelongsToSection,
    AlignsWithBar,
    VariantOf,
    SupportsBreakRebuild,
    HighQuoteRiskWith,
    GoodFollowupTo,
    CaptureParentOf,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub overall_confidence: Confidence,
    pub timing_quality: QualityClass,
    pub section_quality: QualityClass,
    pub loop_candidate_count: usize,
    pub hook_candidate_count: usize,
    pub break_rebuild_potential: QualityClass,
    pub warnings: Vec<AnalysisWarning>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QualityClass {
    Low,
    Medium,
    High,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnalysisWarning {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphProvenance {
    pub sidecar_version: String,
    pub provider_set: Vec<String>,
    pub generated_at: String,
    pub source_hash: String,
    pub analysis_seed: u64,
    pub run_notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_candidates_by_type() {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: SourceId::from("src-1"),
                path: "break.wav".into(),
                content_hash: "hash-1".into(),
                duration_seconds: 180.0,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "0.1.0".into(),
                provider_set: vec!["beats".into()],
                generated_at: "2026-04-12T18:00:00Z".into(),
                source_hash: "hash-1".into(),
                analysis_seed: 42,
                run_notes: None,
            },
        );

        graph.candidates.push(Candidate {
            candidate_id: CandidateId::from("cand-1"),
            candidate_type: CandidateType::LoopCandidate,
            asset_ref: AssetId::from("asset-1"),
            score: 0.8,
            confidence: 0.9,
            tags: vec![],
            constraints: vec![],
            provenance_refs: vec![],
        });
        graph.candidates.push(Candidate {
            candidate_id: CandidateId::from("cand-2"),
            candidate_type: CandidateType::HookCandidate,
            asset_ref: AssetId::from("asset-2"),
            score: 0.7,
            confidence: 0.8,
            tags: vec![],
            constraints: vec![],
            provenance_refs: vec![],
        });

        assert_eq!(graph.loop_candidate_count(), 1);
        assert_eq!(graph.hook_candidate_count(), 1);
    }

    #[test]
    fn source_graph_roundtrips_via_json() {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: SourceId::from("src-1"),
                path: "break.wav".into(),
                content_hash: "hash-1".into(),
                duration_seconds: 180.0,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "0.1.0".into(),
                provider_set: vec!["beats".into(), "hooks".into()],
                generated_at: "2026-04-12T18:00:00Z".into(),
                source_hash: "hash-1".into(),
                analysis_seed: 42,
                run_notes: Some("test".into()),
            },
        );
        graph.sections.push(Section {
            section_id: SectionId::from("section-a"),
            label_hint: SectionLabelHint::Drop,
            start_seconds: 0.0,
            end_seconds: 16.0,
            bar_start: 1,
            bar_end: 8,
            energy_class: EnergyClass::High,
            confidence: 0.9,
            tags: vec!["main".into()],
        });
        graph.assets.push(Asset {
            asset_id: AssetId::from("asset-a"),
            asset_type: AssetType::LoopWindow,
            start_seconds: 0.0,
            end_seconds: 4.0,
            start_bar: 1,
            end_bar: 2,
            confidence: 0.8,
            tags: vec!["loop".into()],
            source_refs: vec!["src-1".into()],
        });
        graph.candidates.push(Candidate {
            candidate_id: CandidateId::from("candidate-a"),
            candidate_type: CandidateType::LoopCandidate,
            asset_ref: AssetId::from("asset-a"),
            score: 0.88,
            confidence: 0.91,
            tags: vec!["useful".into()],
            constraints: vec!["bar_aligned".into()],
            provenance_refs: vec!["provider:beats".into()],
        });
        graph.relationships.push(Relationship {
            relation_type: RelationshipType::BelongsToSection,
            from_id: "asset-a".into(),
            to_id: "section-a".into(),
            weight: 1.0,
            notes: Some("primary loop".into()),
        });
        graph.analysis_summary = AnalysisSummary {
            overall_confidence: 0.87,
            timing_quality: QualityClass::High,
            section_quality: QualityClass::Medium,
            loop_candidate_count: 1,
            hook_candidate_count: 0,
            break_rebuild_potential: QualityClass::High,
            warnings: vec![AnalysisWarning {
                code: "low_hook_density".into(),
                message: "few hook fragments".into(),
            }],
        };

        let json = serde_json::to_string_pretty(&graph).expect("serialize source graph");
        let decoded: SourceGraph = serde_json::from_str(&json).expect("deserialize source graph");

        assert_eq!(decoded, graph);
    }
}
