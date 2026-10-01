use riotbox_core::source_graph::{
    SourceTimingCandidateConfidenceResult, SourceTimingCandidateDriftStatus,
    SourceTimingCandidatePhraseStatus, SourceTimingProbeBeatEvidenceStatus,
    SourceTimingProbeDownbeatEvidenceStatus, SourceTimingProbeReadinessReport,
    SourceTimingProbeReadinessStatus, source_timing_grid_use,
    source_timing_readiness_report_labels,
};
use serde::Serialize;

use super::{
    grid_bpm_decision::{GridBpmDecision, source_timing_bpm_agrees},
    source_timing_policy_profile::SOURCE_TIMING_POLICY_PROFILE,
    timing_evidence::{ManifestSourceTimingAnchorEvidence, ManifestSourceTimingGrooveEvidence},
};

#[derive(Serialize)]
pub(super) struct ManifestSourceTimingReadiness {
    schema: &'static str,
    schema_version: u32,
    source_id: String,
    policy_profile: &'static str,
    readiness: &'static str,
    requires_manual_confirm: bool,
    cue: &'static str,
    actionability: &'static str,
    grid_use: &'static str,
    primary_bpm: Option<f32>,
    bpm_agrees_with_grid: Option<bool>,
    primary_downbeat_offset_beats: Option<u8>,
    primary_downbeat_score: Option<f32>,
    primary_downbeat_margin: Option<f32>,
    alternate_downbeat_phase_count: usize,
    beat_status: &'static str,
    downbeat_status: &'static str,
    confidence_result: &'static str,
    drift_status: &'static str,
    phrase_status: &'static str,
    primary_phrase_count: usize,
    primary_phrase_bar_count: usize,
    anchor_evidence: ManifestSourceTimingAnchorEvidence,
    groove_evidence: ManifestSourceTimingGrooveEvidence,
    alternate_evidence_count: usize,
    warning_codes: Vec<String>,
}

pub(super) fn manifest_source_timing_readiness(
    report: &SourceTimingProbeReadinessReport,
    grid_bpm: GridBpmDecision,
    anchor_evidence: &ManifestSourceTimingAnchorEvidence,
    groove_evidence: &ManifestSourceTimingGrooveEvidence,
) -> ManifestSourceTimingReadiness {
    let labels = source_timing_readiness_report_labels(report);
    ManifestSourceTimingReadiness {
        schema: report.schema,
        schema_version: report.schema_version,
        source_id: report.source_id.clone(),
        policy_profile: SOURCE_TIMING_POLICY_PROFILE.name,
        readiness: readiness_status_label(report.readiness),
        requires_manual_confirm: report.requires_manual_confirm,
        cue: labels.cue,
        actionability: labels.actionability,
        grid_use: source_timing_grid_use(report).label(),
        primary_bpm: report.primary_bpm,
        bpm_agrees_with_grid: source_timing_bpm_agrees(grid_bpm.source_delta_bpm),
        primary_downbeat_offset_beats: report.primary_downbeat_offset_beats,
        primary_downbeat_score: report.primary_downbeat_score,
        primary_downbeat_margin: report.primary_downbeat_margin,
        alternate_downbeat_phase_count: report.alternate_downbeat_phase_count,
        beat_status: beat_evidence_status_label(report.beat_status),
        downbeat_status: downbeat_evidence_status_label(report.downbeat_status),
        confidence_result: confidence_result_label(report.confidence_result),
        drift_status: drift_status_label(report.drift_status),
        phrase_status: phrase_status_label(report.phrase_status),
        primary_phrase_count: report.primary_phrase_count,
        primary_phrase_bar_count: report.primary_phrase_bar_count,
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

pub(super) fn readiness_status_label(status: SourceTimingProbeReadinessStatus) -> &'static str {
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

pub(super) fn downbeat_evidence_status_label(
    status: SourceTimingProbeDownbeatEvidenceStatus,
) -> &'static str {
    match status {
        SourceTimingProbeDownbeatEvidenceStatus::Unavailable => "unavailable",
        SourceTimingProbeDownbeatEvidenceStatus::Weak => "weak",
        SourceTimingProbeDownbeatEvidenceStatus::Stable => "stable",
        SourceTimingProbeDownbeatEvidenceStatus::Ambiguous => "ambiguous",
    }
}

pub(super) fn confidence_result_label(
    result: SourceTimingCandidateConfidenceResult,
) -> &'static str {
    match result {
        SourceTimingCandidateConfidenceResult::Degraded => "degraded",
        SourceTimingCandidateConfidenceResult::CandidateCautious => "candidate_cautious",
        SourceTimingCandidateConfidenceResult::CandidateAmbiguous => "candidate_ambiguous",
    }
}

pub(super) fn drift_status_label(status: SourceTimingCandidateDriftStatus) -> &'static str {
    match status {
        SourceTimingCandidateDriftStatus::Unavailable => "unavailable",
        SourceTimingCandidateDriftStatus::NotEnoughMaterial => "not_enough_material",
        SourceTimingCandidateDriftStatus::Stable => "stable",
        SourceTimingCandidateDriftStatus::High => "high",
    }
}

pub(super) fn phrase_status_label(status: SourceTimingCandidatePhraseStatus) -> &'static str {
    match status {
        SourceTimingCandidatePhraseStatus::Unavailable => "unavailable",
        SourceTimingCandidatePhraseStatus::NotEnoughMaterial => "not_enough_material",
        SourceTimingCandidatePhraseStatus::AmbiguousDownbeat => "ambiguous_downbeat",
        SourceTimingCandidatePhraseStatus::HighDrift => "high_drift",
        SourceTimingCandidatePhraseStatus::Stable => "stable",
    }
}
