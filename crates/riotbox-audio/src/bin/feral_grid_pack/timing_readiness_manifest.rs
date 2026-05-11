use riotbox_core::source_graph::{
    GrooveResidual, GrooveSubdivision, SourceTimingAnchor, SourceTimingAnchorType, TimingModel,
};

#[derive(Serialize)]
struct ManifestSourceTimingReadiness {
    schema: &'static str,
    schema_version: u32,
    source_id: String,
    policy_profile: &'static str,
    readiness: &'static str,
    requires_manual_confirm: bool,
    grid_use: &'static str,
    primary_bpm: Option<f32>,
    bpm_agrees_with_grid: Option<bool>,
    primary_downbeat_offset_beats: Option<u8>,
    beat_status: &'static str,
    downbeat_status: &'static str,
    confidence_result: &'static str,
    drift_status: &'static str,
    phrase_status: &'static str,
    anchor_evidence: ManifestSourceTimingAnchorEvidence,
    groove_evidence: ManifestSourceTimingGrooveEvidence,
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

#[derive(Clone, Debug, PartialEq, Serialize)]
struct ManifestSourceTimingGrooveEvidence {
    primary_groove_residual_count: usize,
    primary_max_abs_offset_ms: f32,
    primary_groove_preview: Vec<ManifestSourceTimingGrooveResidual>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct ManifestSourceTimingGrooveResidual {
    subdivision: &'static str,
    offset_ms: f32,
    confidence: f32,
}

impl ManifestSourceTimingGrooveEvidence {
    fn from_timing(timing: &TimingModel) -> Self {
        let groove = timing
            .primary_hypothesis()
            .map_or(&[][..], |hypothesis| hypothesis.groove.as_slice());
        Self {
            primary_groove_residual_count: groove.len(),
            primary_max_abs_offset_ms: groove
                .iter()
                .map(|residual| residual.offset_ms.abs())
                .fold(0.0_f32, f32::max),
            primary_groove_preview: groove
                .iter()
                .take(4)
                .map(ManifestSourceTimingGrooveResidual::from_residual)
                .collect(),
        }
    }
}

impl ManifestSourceTimingGrooveResidual {
    fn from_residual(residual: &GrooveResidual) -> Self {
        Self {
            subdivision: source_timing_groove_subdivision_label(residual.subdivision),
            offset_ms: residual.offset_ms,
            confidence: residual.confidence,
        }
    }
}

fn manifest_source_timing_readiness(
    report: &SourceTimingProbeReadinessReport,
    grid_bpm: GridBpmDecision,
    anchor_evidence: &ManifestSourceTimingAnchorEvidence,
    groove_evidence: &ManifestSourceTimingGrooveEvidence,
) -> ManifestSourceTimingReadiness {
    ManifestSourceTimingReadiness {
        schema: report.schema,
        schema_version: report.schema_version,
        source_id: report.source_id.clone(),
        policy_profile: SOURCE_TIMING_POLICY_PROFILE.name,
        readiness: readiness_status_label(report.readiness),
        requires_manual_confirm: report.requires_manual_confirm,
        grid_use: source_timing_grid_use(report),
        primary_bpm: report.primary_bpm,
        bpm_agrees_with_grid: source_timing_bpm_agrees(grid_bpm.source_delta_bpm),
        primary_downbeat_offset_beats: report.primary_downbeat_offset_beats,
        beat_status: beat_evidence_status_label(report.beat_status),
        downbeat_status: downbeat_evidence_status_label(report.downbeat_status),
        confidence_result: confidence_result_label(report.confidence_result),
        drift_status: drift_status_label(report.drift_status),
        phrase_status: phrase_status_label(report.phrase_status),
        anchor_evidence: anchor_evidence.clone(),
        groove_evidence: groove_evidence.clone(),
        alternate_evidence_count: report.alternate_evidence_count,
        warning_codes: report
            .warning_codes
            .iter()
            .map(|code| format!("{code:?}"))
            .collect(),
    }
}

fn source_timing_grid_use(report: &SourceTimingProbeReadinessReport) -> &'static str {
    if report.primary_bpm.is_none()
        || report.readiness == SourceTimingProbeReadinessStatus::Unavailable
    {
        return "unavailable";
    }
    if report.readiness == SourceTimingProbeReadinessStatus::Ready
        && !report.requires_manual_confirm
    {
        return "locked_grid";
    }
    if can_use_cautious_source_timing_bpm(report)
        && report.phrase_status == SourceTimingCandidatePhraseStatus::NotEnoughMaterial
    {
        return "short_loop_manual_confirm";
    }
    if report.requires_manual_confirm {
        return "manual_confirm_only";
    }
    "fallback_grid"
}

fn source_timing_groove_subdivision_label(subdivision: GrooveSubdivision) -> &'static str {
    match subdivision {
        GrooveSubdivision::Eighth => "eighth",
        GrooveSubdivision::Triplet => "triplet",
        GrooveSubdivision::Sixteenth => "sixteenth",
        GrooveSubdivision::ThirtySecond => "thirty_second",
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
