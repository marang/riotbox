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

#[must_use]
pub fn source_timing_grid_use_from_timing_model(timing: &TimingModel) -> SourceTimingGridUse {
    if timing.bpm_estimate.is_none()
        || timing.effective_degraded_policy() == TimingDegradedPolicy::Disabled
    {
        return SourceTimingGridUse::Unavailable;
    }

    match timing.effective_degraded_policy() {
        TimingDegradedPolicy::Locked => SourceTimingGridUse::LockedGrid,
        TimingDegradedPolicy::Cautious if timing_model_has_short_loop_grid(timing) => {
            SourceTimingGridUse::ShortLoopManualConfirm
        }
        TimingDegradedPolicy::Cautious | TimingDegradedPolicy::ManualConfirm => {
            SourceTimingGridUse::ManualConfirmOnly
        }
        TimingDegradedPolicy::FallbackGrid => SourceTimingGridUse::FallbackGrid,
        TimingDegradedPolicy::Disabled | TimingDegradedPolicy::Unknown => {
            SourceTimingGridUse::Unavailable
        }
    }
}

fn source_timing_is_stable_short_loop_manual_confirm(
    report: &SourceTimingProbeReadinessReport,
) -> bool {
    source_timing_can_use_cautious_grid_bpm(report)
        && report.primary_bpm.is_some()
        && report.phrase_status == SourceTimingCandidatePhraseStatus::NotEnoughMaterial
}

fn timing_model_has_short_loop_grid(timing: &TimingModel) -> bool {
    timing.bpm_estimate.is_some()
        && !timing.beat_grid.is_empty()
        && !timing.bar_grid.is_empty()
        && timing.phrase_grid.is_empty()
        && timing
            .warnings
            .iter()
            .any(|warning| warning.code == TimingWarningCode::PhraseUncertain)
}
