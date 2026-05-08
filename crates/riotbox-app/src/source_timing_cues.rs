pub fn source_timing_policy_cue_label(policy: &str) -> &'static str {
    match policy {
        "locked" => "grid locked",
        "manual_confirm" => "needs confirm",
        "cautious" => "listen first",
        "fallback_grid" => "fallback grid",
        "disabled" => "not available",
        _ => "unknown",
    }
}

pub fn source_timing_readiness_cue_label(
    readiness: &str,
    requires_manual_confirm: bool,
) -> &'static str {
    if requires_manual_confirm {
        return "needs confirm";
    }

    match readiness {
        "ready" => "grid locked",
        "needs_review" | "weak" => "listen first",
        "unavailable" => "not available",
        _ => "unknown",
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
    fn source_timing_readiness_cues_prioritize_manual_confirm() {
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
            source_timing_readiness_cue_label("surprise", false),
            "unknown"
        );
    }
}
