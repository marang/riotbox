#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceTimingGridUse {
    LockedGrid,
    ShortLoopManualConfirm,
    ManualConfirmOnly,
    FallbackGrid,
    Unavailable,
}

impl SourceTimingGridUse {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::LockedGrid => "locked_grid",
            Self::ShortLoopManualConfirm => "short_loop_manual_confirm",
            Self::ManualConfirmOnly => "manual_confirm_only",
            Self::FallbackGrid => "fallback_grid",
            Self::Unavailable => "unavailable",
        }
    }
}

#[must_use]
pub fn source_timing_grid_use(report: &SourceTimingProbeReadinessReport) -> SourceTimingGridUse {
    if report.primary_bpm.is_none()
        || report.readiness == SourceTimingProbeReadinessStatus::Unavailable
    {
        return SourceTimingGridUse::Unavailable;
    }
    if report.readiness == SourceTimingProbeReadinessStatus::Ready
        && !report.requires_manual_confirm
    {
        return SourceTimingGridUse::LockedGrid;
    }
    if source_timing_is_stable_short_loop_manual_confirm(report) {
        return SourceTimingGridUse::ShortLoopManualConfirm;
    }
    if report.requires_manual_confirm {
        return SourceTimingGridUse::ManualConfirmOnly;
    }
    SourceTimingGridUse::FallbackGrid
}

#[must_use]
pub fn source_timing_can_use_cautious_grid_bpm(
    report: &SourceTimingProbeReadinessReport,
) -> bool {
    report.readiness == SourceTimingProbeReadinessStatus::NeedsReview
        && report.requires_manual_confirm
        && report.beat_status == SourceTimingProbeBeatEvidenceStatus::Stable
        && report.downbeat_status == SourceTimingProbeDownbeatEvidenceStatus::Stable
        && report.confidence_result == SourceTimingCandidateConfidenceResult::CandidateCautious
        && report.alternate_evidence_count == 0
}

fn source_timing_is_stable_short_loop_manual_confirm(
    report: &SourceTimingProbeReadinessReport,
) -> bool {
    source_timing_can_use_cautious_grid_bpm(report)
        && report.primary_bpm.is_some()
        && report.phrase_status == SourceTimingCandidatePhraseStatus::NotEnoughMaterial
}
