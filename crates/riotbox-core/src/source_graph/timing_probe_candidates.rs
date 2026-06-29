use super::{
    BarSpan, BeatPoint, Confidence, GrooveResidual, GrooveSubdivision, MeterHint, PhraseSpan,
    SourceTimingAnchor, SourceTimingAnchorType, SourceTimingProbeDiagnosticInput,
    SourceTimingProbeDiagnosticPolicy, TimingDegradedPolicy, TimingDriftReport, TimingHypothesis,
    TimingHypothesisKind, TimingModel, TimingQuality, TimingWarning, TimingWarningCode,
    timing_model_from_probe_diagnostics,
};

const MIN_STABLE_DOWNBEAT_PHASE_SCORE: f32 = 0.30;
// Near-stable but phase-conflicted evidence is reviewable ambiguity, not a
// locked downbeat and not the same as flat weak timing.
const MIN_AMBIGUOUS_DOWNBEAT_PHASE_SCORE: f32 = MIN_STABLE_DOWNBEAT_PHASE_SCORE * 0.90;

include!("timing_probe_candidates/types.rs");
include!("timing_probe_candidates/confidence_report.rs");
include!("timing_probe_candidates/period_scoring.rs");
include!("timing_probe_candidates/drift.rs");
include!("timing_probe_candidates/groove.rs");
include!("timing_probe_candidates/phrase.rs");
include!("timing_probe_candidates/model.rs");
include!("timing_probe_candidates/hypothesis.rs");
include!("timing_probe_candidates/downbeat_phase.rs");
include!("timing_probe_candidates/grid.rs");
include!("timing_probe_candidates/readiness_report.rs");
include!("timing_probe_candidates/grid_use_policy.rs");

#[cfg(test)]
#[path = "probe_candidate_tests.rs"]
mod probe_candidate_tests;
