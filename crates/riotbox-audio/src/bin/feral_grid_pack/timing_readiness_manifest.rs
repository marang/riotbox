use riotbox_core::source_graph::{SourceTimingAnchor, SourceTimingAnchorType, TimingModel};

#[derive(Serialize)]
struct ManifestSourceTimingReadiness {
    schema: &'static str,
    schema_version: u32,
    source_id: String,
    policy_profile: &'static str,
    readiness: &'static str,
    requires_manual_confirm: bool,
    primary_bpm: Option<f32>,
    bpm_agrees_with_grid: Option<bool>,
    primary_downbeat_offset_beats: Option<u8>,
    beat_status: &'static str,
    downbeat_status: &'static str,
    confidence_result: &'static str,
    drift_status: &'static str,
    phrase_status: &'static str,
    anchor_evidence: ManifestSourceTimingAnchorEvidence,
    alternate_evidence_count: usize,
    warning_codes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct ManifestSourceTimingAnchorEvidence {
    primary_anchor_count: usize,
    primary_kick_anchor_count: usize,
    primary_backbeat_anchor_count: usize,
    primary_transient_anchor_count: usize,
}

impl ManifestSourceTimingAnchorEvidence {
    fn from_timing(timing: &TimingModel) -> Self {
        let anchors = timing
            .primary_hypothesis()
            .map_or(&[][..], |hypothesis| hypothesis.anchors.as_slice());
        Self {
            primary_anchor_count: anchors.len(),
            primary_kick_anchor_count: count_source_timing_anchor_type(
                anchors,
                SourceTimingAnchorType::Kick,
            ),
            primary_backbeat_anchor_count: count_source_timing_anchor_type(
                anchors,
                SourceTimingAnchorType::Backbeat,
            ),
            primary_transient_anchor_count: count_source_timing_anchor_type(
                anchors,
                SourceTimingAnchorType::TransientCluster,
            ),
        }
    }
}

fn manifest_source_timing_readiness(
    report: &SourceTimingProbeReadinessReport,
    grid_bpm: GridBpmDecision,
    anchor_evidence: &ManifestSourceTimingAnchorEvidence,
) -> ManifestSourceTimingReadiness {
    ManifestSourceTimingReadiness {
        schema: report.schema,
        schema_version: report.schema_version,
        source_id: report.source_id.clone(),
        policy_profile: SOURCE_TIMING_POLICY_PROFILE.name,
        readiness: readiness_status_label(report.readiness),
        requires_manual_confirm: report.requires_manual_confirm,
        primary_bpm: report.primary_bpm,
        bpm_agrees_with_grid: source_timing_bpm_agrees(grid_bpm.source_delta_bpm),
        primary_downbeat_offset_beats: report.primary_downbeat_offset_beats,
        beat_status: beat_evidence_status_label(report.beat_status),
        downbeat_status: downbeat_evidence_status_label(report.downbeat_status),
        confidence_result: confidence_result_label(report.confidence_result),
        drift_status: drift_status_label(report.drift_status),
        phrase_status: phrase_status_label(report.phrase_status),
        anchor_evidence: anchor_evidence.clone(),
        alternate_evidence_count: report.alternate_evidence_count,
        warning_codes: report
            .warning_codes
            .iter()
            .map(|code| format!("{code:?}"))
            .collect(),
    }
}

fn count_source_timing_anchor_type(
    anchors: &[SourceTimingAnchor],
    anchor_type: SourceTimingAnchorType,
) -> usize {
    anchors
        .iter()
        .filter(|anchor| anchor.anchor_type == anchor_type)
        .count()
}

fn readiness_status_label(status: SourceTimingProbeReadinessStatus) -> &'static str {
    match status {
        SourceTimingProbeReadinessStatus::Unavailable => "unavailable",
        SourceTimingProbeReadinessStatus::Weak => "weak",
        SourceTimingProbeReadinessStatus::NeedsReview => "needs_review",
        SourceTimingProbeReadinessStatus::Ready => "ready",
    }
}

fn beat_evidence_status_label(status: SourceTimingProbeBeatEvidenceStatus) -> &'static str {
    match status {
        SourceTimingProbeBeatEvidenceStatus::Unavailable => "unavailable",
        SourceTimingProbeBeatEvidenceStatus::Weak => "weak",
        SourceTimingProbeBeatEvidenceStatus::Stable => "stable",
        SourceTimingProbeBeatEvidenceStatus::Ambiguous => "ambiguous",
    }
}

fn downbeat_evidence_status_label(
    status: SourceTimingProbeDownbeatEvidenceStatus,
) -> &'static str {
    match status {
        SourceTimingProbeDownbeatEvidenceStatus::Unavailable => "unavailable",
        SourceTimingProbeDownbeatEvidenceStatus::Weak => "weak",
        SourceTimingProbeDownbeatEvidenceStatus::Stable => "stable",
        SourceTimingProbeDownbeatEvidenceStatus::Ambiguous => "ambiguous",
    }
}

fn confidence_result_label(result: SourceTimingCandidateConfidenceResult) -> &'static str {
    match result {
        SourceTimingCandidateConfidenceResult::Degraded => "degraded",
        SourceTimingCandidateConfidenceResult::CandidateCautious => "candidate_cautious",
        SourceTimingCandidateConfidenceResult::CandidateAmbiguous => "candidate_ambiguous",
    }
}

fn drift_status_label(status: SourceTimingCandidateDriftStatus) -> &'static str {
    match status {
        SourceTimingCandidateDriftStatus::Unavailable => "unavailable",
        SourceTimingCandidateDriftStatus::NotEnoughMaterial => "not_enough_material",
        SourceTimingCandidateDriftStatus::Stable => "stable",
        SourceTimingCandidateDriftStatus::High => "high",
    }
}

fn phrase_status_label(status: SourceTimingCandidatePhraseStatus) -> &'static str {
    match status {
        SourceTimingCandidatePhraseStatus::Unavailable => "unavailable",
        SourceTimingCandidatePhraseStatus::NotEnoughMaterial => "not_enough_material",
        SourceTimingCandidatePhraseStatus::AmbiguousDownbeat => "ambiguous_downbeat",
        SourceTimingCandidatePhraseStatus::HighDrift => "high_drift",
        SourceTimingCandidatePhraseStatus::Stable => "stable",
    }
}
