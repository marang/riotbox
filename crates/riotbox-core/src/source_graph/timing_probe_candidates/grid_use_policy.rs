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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceTimingReadinessLabels {
    pub cue: &'static str,
    pub actionability: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceTimingPolicyLabels {
    pub cue: &'static str,
    pub actionability: &'static str,
}

#[must_use]
pub fn source_timing_policy_labels(policy: TimingDegradedPolicy) -> SourceTimingPolicyLabels {
    match policy {
        TimingDegradedPolicy::Locked => SourceTimingPolicyLabels {
            cue: "grid locked",
            actionability: "grid can steer moves",
        },
        TimingDegradedPolicy::ManualConfirm => SourceTimingPolicyLabels {
            cue: "needs confirm",
            actionability: "confirm grid first",
        },
        TimingDegradedPolicy::Cautious => SourceTimingPolicyLabels {
            cue: "listen first",
            actionability: "listen first",
        },
        TimingDegradedPolicy::FallbackGrid => SourceTimingPolicyLabels {
            cue: "fallback grid",
            actionability: "using safe fallback grid",
        },
        TimingDegradedPolicy::Disabled => SourceTimingPolicyLabels {
            cue: "not available",
            actionability: "timing unavailable",
        },
        TimingDegradedPolicy::Unknown => SourceTimingPolicyLabels {
            cue: "unknown",
            actionability: "timing trust unknown",
        },
    }
}

#[must_use]
pub fn source_timing_policy_labels_from_label(policy: &str) -> SourceTimingPolicyLabels {
    match policy {
        "locked" => source_timing_policy_labels(TimingDegradedPolicy::Locked),
        "manual_confirm" => source_timing_policy_labels(TimingDegradedPolicy::ManualConfirm),
        "cautious" => source_timing_policy_labels(TimingDegradedPolicy::Cautious),
        "fallback_grid" => source_timing_policy_labels(TimingDegradedPolicy::FallbackGrid),
        "disabled" => source_timing_policy_labels(TimingDegradedPolicy::Disabled),
        _ => source_timing_policy_labels(TimingDegradedPolicy::Unknown),
    }
}

#[must_use]
pub fn source_timing_readiness_labels(
    readiness: SourceTimingProbeReadinessStatus,
    requires_manual_confirm: bool,
) -> SourceTimingReadinessLabels {
    if readiness == SourceTimingProbeReadinessStatus::Unavailable {
        return SourceTimingReadinessLabels {
            cue: "not available",
            actionability: "timing unavailable",
        };
    }

    if requires_manual_confirm {
        return SourceTimingReadinessLabels {
            cue: "needs confirm",
            actionability: "confirm grid first",
        };
    }

    match readiness {
        SourceTimingProbeReadinessStatus::Ready => SourceTimingReadinessLabels {
            cue: "grid locked",
            actionability: "grid can steer moves",
        },
        SourceTimingProbeReadinessStatus::NeedsReview | SourceTimingProbeReadinessStatus::Weak => {
            SourceTimingReadinessLabels {
                cue: "listen first",
                actionability: "listen first",
            }
        }
        SourceTimingProbeReadinessStatus::Unavailable => unreachable!(),
    }
}

#[must_use]
pub fn source_timing_readiness_report_labels(
    report: &SourceTimingProbeReadinessReport,
) -> SourceTimingReadinessLabels {
    source_timing_readiness_labels(report.readiness, report.requires_manual_confirm)
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
    let beat_count = timing_model_primary_or_top_level_beat_count(timing);
    let bar_count = timing_model_primary_or_top_level_bar_count(timing);
    let phrase_count = timing_model_primary_or_top_level_phrase_count(timing);

    timing.bpm_estimate.is_some()
        && beat_count > 0
        && bar_count > 0
        && phrase_count == 0
        && timing
            .warnings
            .iter()
            .any(|warning| warning.code == TimingWarningCode::PhraseUncertain)
}

fn timing_model_primary_or_top_level_beat_count(timing: &TimingModel) -> usize {
    timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.beat_grid.len())
        .filter(|count| *count > 0)
        .unwrap_or(timing.beat_grid.len())
}

fn timing_model_primary_or_top_level_bar_count(timing: &TimingModel) -> usize {
    timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.bar_grid.len())
        .filter(|count| *count > 0)
        .unwrap_or(timing.bar_grid.len())
}

fn timing_model_primary_or_top_level_phrase_count(timing: &TimingModel) -> usize {
    timing
        .primary_hypothesis()
        .map(|hypothesis| hypothesis.phrase_grid.len())
        .filter(|count| *count > 0)
        .unwrap_or(timing.phrase_grid.len())
}
