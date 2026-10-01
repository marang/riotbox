use riotbox_core::source_graph::{
    SourceTimingProbeReadinessReport, SourceTimingProbeReadinessStatus,
    source_timing_can_use_cautious_grid_bpm,
};

use super::{args::Args, config::SOURCE_TIMING_BPM_MATCH_TOLERANCE};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GridBpmSource {
    UserOverride,
    SourceTiming,
    StaticDefault,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GridBpmDecisionReason {
    UserOverride,
    SourceTimingReady,
    SourceTimingNeedsReviewManualConfirm,
    SourceTimingRequiresManualConfirm,
    SourceTimingNotReady,
    SourceTimingMissingBpm,
    SourceTimingInvalidBpm,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GridBpmDecision {
    pub(super) bpm: f32,
    pub(super) source: GridBpmSource,
    pub(super) reason: GridBpmDecisionReason,
    pub(super) source_primary_bpm: Option<f32>,
    pub(super) source_delta_bpm: Option<f32>,
}

pub(super) fn choose_grid_bpm(
    args: &Args,
    timing_readiness: &SourceTimingProbeReadinessReport,
) -> GridBpmDecision {
    let source_primary_bpm = timing_readiness.primary_bpm;
    let usable_source_bpm = source_primary_bpm.filter(|bpm| bpm.is_finite() && *bpm > 0.0);
    let source_delta_bpm = usable_source_bpm.map(|bpm| (args.bpm - bpm).abs());

    if args.bpm_overridden {
        return GridBpmDecision {
            bpm: args.bpm,
            source: GridBpmSource::UserOverride,
            reason: GridBpmDecisionReason::UserOverride,
            source_primary_bpm,
            source_delta_bpm,
        };
    }

    if timing_readiness.readiness == SourceTimingProbeReadinessStatus::Ready
        && !timing_readiness.requires_manual_confirm
        && let Some(bpm) = usable_source_bpm
    {
        return GridBpmDecision {
            bpm,
            source: GridBpmSource::SourceTiming,
            reason: GridBpmDecisionReason::SourceTimingReady,
            source_primary_bpm,
            source_delta_bpm: Some(0.0),
        };
    }

    if source_timing_can_use_cautious_grid_bpm(timing_readiness)
        && let Some(bpm) = usable_source_bpm
    {
        return GridBpmDecision {
            bpm,
            source: GridBpmSource::SourceTiming,
            reason: GridBpmDecisionReason::SourceTimingNeedsReviewManualConfirm,
            source_primary_bpm,
            source_delta_bpm: Some(0.0),
        };
    }

    GridBpmDecision {
        bpm: args.bpm,
        source: GridBpmSource::StaticDefault,
        reason: static_default_reason(timing_readiness, usable_source_bpm),
        source_primary_bpm,
        source_delta_bpm,
    }
}

fn static_default_reason(
    timing_readiness: &SourceTimingProbeReadinessReport,
    usable_source_bpm: Option<f32>,
) -> GridBpmDecisionReason {
    match (timing_readiness.primary_bpm, usable_source_bpm) {
        (None, _) => GridBpmDecisionReason::SourceTimingMissingBpm,
        (Some(_), None) => GridBpmDecisionReason::SourceTimingInvalidBpm,
        (Some(_), Some(_)) if timing_readiness.requires_manual_confirm => {
            GridBpmDecisionReason::SourceTimingRequiresManualConfirm
        }
        (Some(_), Some(_)) => GridBpmDecisionReason::SourceTimingNotReady,
    }
}

pub(super) fn grid_bpm_source_label(source: GridBpmSource) -> &'static str {
    match source {
        GridBpmSource::UserOverride => "user_override",
        GridBpmSource::SourceTiming => "source_timing",
        GridBpmSource::StaticDefault => "static_default",
    }
}

pub(super) fn grid_bpm_decision_reason_label(reason: GridBpmDecisionReason) -> &'static str {
    match reason {
        GridBpmDecisionReason::UserOverride => "user_override",
        GridBpmDecisionReason::SourceTimingReady => "source_timing_ready",
        GridBpmDecisionReason::SourceTimingNeedsReviewManualConfirm => {
            "source_timing_needs_review_manual_confirm"
        }
        GridBpmDecisionReason::SourceTimingRequiresManualConfirm => {
            "source_timing_requires_manual_confirm"
        }
        GridBpmDecisionReason::SourceTimingNotReady => "source_timing_not_ready",
        GridBpmDecisionReason::SourceTimingMissingBpm => "source_timing_missing_bpm",
        GridBpmDecisionReason::SourceTimingInvalidBpm => "source_timing_invalid_bpm",
    }
}

pub(super) fn source_timing_bpm_agrees(delta_bpm: Option<f32>) -> Option<bool> {
    delta_bpm.map(|delta| delta <= SOURCE_TIMING_BPM_MATCH_TOLERANCE)
}
