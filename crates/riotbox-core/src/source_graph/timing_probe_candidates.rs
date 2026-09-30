// Public timing-candidate compatibility over private algorithm owners.
mod confidence_report;
mod downbeat_phase;
mod drift;
mod grid;
mod grid_use_policy;
mod groove;
mod hypothesis;
mod model;
mod onset_evidence;
mod period_scoring;
mod readiness_report;
mod types;

#[cfg(test)]
mod tests;

pub use confidence_report::source_timing_candidate_confidence_report;
pub use downbeat_phase::source_timing_probe_downbeat_evidence_report;
pub use grid_use_policy::{
    SourceTimingGridUse, SourceTimingPolicyLabels, SourceTimingReadinessLabels,
    source_timing_can_use_cautious_grid_bpm, source_timing_grid_use,
    source_timing_grid_use_from_timing_model, source_timing_policy_labels,
    source_timing_policy_labels_from_label, source_timing_readiness_labels,
    source_timing_readiness_report_labels,
};
pub use model::timing_model_from_probe_bpm_candidates;
pub use period_scoring::source_timing_probe_beat_evidence_report;
pub use readiness_report::source_timing_probe_readiness_report;
pub use types::{
    SourceTimingCandidateConfidenceReport, SourceTimingCandidateConfidenceResult,
    SourceTimingCandidateDriftStatus, SourceTimingCandidatePhraseStatus,
    SourceTimingProbeBeatEvidenceReport, SourceTimingProbeBeatEvidenceStatus,
    SourceTimingProbeBpmCandidateInput, SourceTimingProbeBpmCandidatePolicy,
    SourceTimingProbeDownbeatEvidenceReport, SourceTimingProbeDownbeatEvidenceStatus,
    SourceTimingProbeReadinessReport, SourceTimingProbeReadinessStatus,
};

const MIN_STABLE_DOWNBEAT_PHASE_SCORE: f32 = 0.30;

// Near-stable but phase-conflicted evidence is reviewable ambiguity, not a
// locked downbeat and not the same as flat weak timing.
const MIN_AMBIGUOUS_DOWNBEAT_PHASE_SCORE: f32 = MIN_STABLE_DOWNBEAT_PHASE_SCORE * 0.90;
