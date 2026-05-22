use riotbox_core::source_graph::{
    SourceTimingProbeReadinessStatus, source_timing_policy_labels_from_label,
    source_timing_readiness_labels,
};

pub fn source_timing_policy_cue_label(policy: &str) -> &'static str {
    source_timing_policy_labels_from_label(policy).cue
}

pub fn source_timing_policy_actionability_label(policy: &str) -> &'static str {
    source_timing_policy_labels_from_label(policy).actionability
}

pub fn source_timing_readiness_cue_label(
    readiness: &str,
    requires_manual_confirm: bool,
) -> &'static str {
    source_timing_readiness_status_from_label(readiness).map_or("unknown", |status| {
        source_timing_readiness_labels(status, requires_manual_confirm).cue
    })
}

pub fn source_timing_readiness_actionability_label(
    readiness: &str,
    requires_manual_confirm: bool,
) -> &'static str {
    source_timing_readiness_status_from_label(readiness).map_or("unknown", |status| {
        source_timing_readiness_labels(status, requires_manual_confirm).actionability
    })
}

fn source_timing_readiness_status_from_label(
    readiness: &str,
) -> Option<SourceTimingProbeReadinessStatus> {
    match readiness {
        "ready" => Some(SourceTimingProbeReadinessStatus::Ready),
        "needs_review" => Some(SourceTimingProbeReadinessStatus::NeedsReview),
        "weak" => Some(SourceTimingProbeReadinessStatus::Weak),
        "unavailable" => Some(SourceTimingProbeReadinessStatus::Unavailable),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_timing_policy_cues_match_musician_language() {
        assert_eq!(source_timing_policy_cue_label("locked"), "grid locked");
        assert_eq!(
            source_timing_policy_cue_label("manual_confirm"),
            "needs confirm"
        );
        assert_eq!(source_timing_policy_cue_label("cautious"), "listen first");
        assert_eq!(
            source_timing_policy_cue_label("fallback_grid"),
            "fallback grid"
        );
        assert_eq!(source_timing_policy_cue_label("disabled"), "not available");
        assert_eq!(source_timing_policy_cue_label("unknown"), "unknown");
    }

    #[test]
    fn source_timing_policy_actionability_matches_musician_language() {
        assert_eq!(
            source_timing_policy_actionability_label("locked"),
            "grid can steer moves"
        );
        assert_eq!(
            source_timing_policy_actionability_label("manual_confirm"),
            "confirm grid first"
        );
        assert_eq!(
            source_timing_policy_actionability_label("cautious"),
            "listen first"
        );
        assert_eq!(
            source_timing_policy_actionability_label("fallback_grid"),
            "using safe fallback grid"
        );
        assert_eq!(
            source_timing_policy_actionability_label("disabled"),
            "timing unavailable"
        );
        assert_eq!(
            source_timing_policy_actionability_label("unknown"),
            "timing trust unknown"
        );
    }

    #[test]
    fn source_timing_readiness_cues_prioritize_unavailable_then_manual_confirm() {
        assert_eq!(
            source_timing_readiness_cue_label("ready", false),
            "grid locked"
        );
        assert_eq!(
            source_timing_readiness_cue_label("ready", true),
            "needs confirm"
        );
        assert_eq!(
            source_timing_readiness_cue_label("weak", false),
            "listen first"
        );
        assert_eq!(
            source_timing_readiness_cue_label("needs_review", false),
            "listen first"
        );
        assert_eq!(
            source_timing_readiness_cue_label("unavailable", false),
            "not available"
        );
        assert_eq!(
            source_timing_readiness_cue_label("unavailable", true),
            "not available"
        );
        assert_eq!(
            source_timing_readiness_cue_label("surprise", false),
            "unknown"
        );
    }

    #[test]
    fn source_timing_readiness_actionability_prioritizes_unavailable_then_manual_confirm() {
        assert_eq!(
            source_timing_readiness_actionability_label("ready", false),
            "grid can steer moves"
        );
        assert_eq!(
            source_timing_readiness_actionability_label("ready", true),
            "confirm grid first"
        );
        assert_eq!(
            source_timing_readiness_actionability_label("weak", false),
            "listen first"
        );
        assert_eq!(
            source_timing_readiness_actionability_label("needs_review", false),
            "listen first"
        );
        assert_eq!(
            source_timing_readiness_actionability_label("unavailable", false),
            "timing unavailable"
        );
        assert_eq!(
            source_timing_readiness_actionability_label("unavailable", true),
            "timing unavailable"
        );
        assert_eq!(
            source_timing_readiness_actionability_label("surprise", false),
            "unknown"
        );
    }
}
