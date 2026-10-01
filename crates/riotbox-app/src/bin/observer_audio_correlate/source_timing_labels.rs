use super::report_model::SourceTimingEvidence;

pub(super) fn source_timing_readiness_cue(timing: &SourceTimingEvidence) -> &'static str {
    riotbox_app::source_timing_cues::source_timing_readiness_cue_label(
        &timing.readiness,
        timing.requires_manual_confirm,
    )
}

pub(super) fn source_timing_readiness_actionability(timing: &SourceTimingEvidence) -> &str {
    timing.actionability.as_deref().unwrap_or_else(|| {
        riotbox_app::source_timing_cues::source_timing_readiness_actionability_label(
            &timing.readiness,
            timing.requires_manual_confirm,
        )
    })
}
