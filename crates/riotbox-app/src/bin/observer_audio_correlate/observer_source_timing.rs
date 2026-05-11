#[derive(Debug, PartialEq)]
struct ObserverSourceTimingReadiness {
    source_id: String,
    cue: String,
    bpm_estimate: Option<f64>,
    bpm_confidence: f64,
    quality: String,
    degraded_policy: String,
    grid_use: String,
    beat_status: String,
    beat_count: u64,
    downbeat_status: String,
    bar_count: u64,
    phrase_status: String,
    phrase_count: u64,
    primary_hypothesis_id: Option<String>,
    hypothesis_count: u64,
    anchor_evidence: Option<SourceTimingAnchorEvidence>,
    groove_evidence: Option<SourceTimingGrooveEvidence>,
    primary_warning_code: Option<String>,
    warning_codes: Vec<String>,
}

fn collect_observer_source_timing(
    events: &[Value],
) -> (Option<ObserverSourceTimingReadiness>, bool) {
    let Some(source_timing) = events
        .iter()
        .filter_map(|event| event.get("snapshot"))
        .find_map(|snapshot| snapshot.get("source_timing"))
    else {
        return (None, false);
    };
    if source_timing.is_null() {
        return (None, false);
    }
    if !source_timing.is_object() {
        return (None, true);
    }

    let cue = match non_empty_string(source_timing, "cue") {
        Some(value) => value,
        None => return (None, true),
    };
    let degraded_policy = match non_empty_string(source_timing, "degraded_policy") {
        Some(value) => value,
        None => return (None, true),
    };
    let Some(expected_cue) = observer_source_timing_policy_cue(&degraded_policy) else {
        return (None, true);
    };
    if cue != expected_cue {
        return (None, true);
    }
    let grid_use = match non_empty_string(source_timing, "grid_use") {
        Some(value)
            if matches!(
                value.as_str(),
                "locked_grid"
                    | "short_loop_manual_confirm"
                    | "manual_confirm_only"
                    | "fallback_grid"
                    | "unavailable"
            ) =>
        {
            value
        }
        Some(_) | None => return (None, true),
    };
    if grid_use != observer_source_timing_expected_grid_use(source_timing, &degraded_policy) {
        return (None, true);
    }

    let evidence = ObserverSourceTimingReadiness {
        source_id: match non_empty_string(source_timing, "source_id") {
            Some(value) => value,
            None => return (None, true),
        },
        cue,
        bpm_estimate: match source_timing.get("bpm_estimate") {
            Some(value) if value.is_null() => None,
            Some(value) => match value.as_f64() {
                Some(value) => Some(value),
                None => return (None, true),
            },
            None => return (None, true),
        },
        bpm_confidence: match source_timing["bpm_confidence"].as_f64() {
            Some(value) => value,
            None => return (None, true),
        },
        quality: match non_empty_string(source_timing, "quality") {
            Some(value) => value,
            None => return (None, true),
        },
        degraded_policy,
        grid_use,
        beat_status: match non_empty_string(source_timing, "beat_status") {
            Some(value) if matches!(value.as_str(), "grid" | "tempo_only" | "unknown") => value,
            None => return (None, true),
            Some(_) => return (None, true),
        },
        beat_count: match source_timing["beat_count"].as_u64() {
            Some(value) => value,
            None => return (None, true),
        },
        downbeat_status: match non_empty_string(source_timing, "downbeat_status") {
            Some(value) if matches!(value.as_str(), "ambiguous" | "bar_locked" | "unknown") => {
                value
            }
            None => return (None, true),
            Some(_) => return (None, true),
        },
        bar_count: match source_timing["bar_count"].as_u64() {
            Some(value) => value,
            None => return (None, true),
        },
        phrase_status: match non_empty_string(source_timing, "phrase_status") {
            Some(value) if matches!(value.as_str(), "uncertain" | "phrase_locked" | "unknown") => {
                value
            }
            None => return (None, true),
            Some(_) => return (None, true),
        },
        phrase_count: match source_timing["phrase_count"].as_u64() {
            Some(value) => value,
            None => return (None, true),
        },
        primary_hypothesis_id: match source_timing.get("primary_hypothesis_id") {
            Some(value) if value.is_null() => None,
            Some(value) => match value.as_str().filter(|value| !value.is_empty()) {
                Some(value) => Some(value.to_string()),
                None => return (None, true),
            },
            None => return (None, true),
        },
        hypothesis_count: match source_timing["hypothesis_count"].as_u64() {
            Some(value) => value,
            None => return (None, true),
        },
        anchor_evidence: match collect_optional_source_timing_anchor_evidence(source_timing) {
            Ok(value) => value,
            Err(()) => return (None, true),
        },
        groove_evidence: match collect_optional_source_timing_groove_evidence(source_timing) {
            Ok(value) => value,
            Err(()) => return (None, true),
        },
        primary_warning_code: match source_timing.get("primary_warning_code") {
            Some(value) if value.is_null() => None,
            Some(value) => match value.as_str().filter(|value| !value.is_empty()) {
                Some(value) => Some(value.to_string()),
                None => return (None, true),
            },
            None => return (None, true),
        },
        warning_codes: match string_list(source_timing, "warning_codes") {
            Some(value) => value,
            None => return (None, true),
        },
    };

    (Some(evidence), false)
}

fn observer_source_timing_expected_grid_use(source_timing: &Value, degraded_policy: &str) -> String {
    let bpm_available = source_timing
        .get("bpm_estimate")
        .is_some_and(|value| value.as_f64().is_some());
    if !bpm_available || matches!(degraded_policy, "disabled" | "unknown") {
        return "unavailable".to_string();
    }

    if degraded_policy == "locked" {
        return "locked_grid".to_string();
    }

    if degraded_policy == "fallback_grid" {
        return "fallback_grid".to_string();
    }

    if degraded_policy == "cautious"
        && source_timing
            .get("beat_count")
            .and_then(Value::as_u64)
            .is_some_and(|count| count > 0)
        && source_timing
            .get("bar_count")
            .and_then(Value::as_u64)
            .is_some_and(|count| count > 0)
        && source_timing
            .get("phrase_count")
            .and_then(Value::as_u64)
            .is_some_and(|count| count == 0)
        && string_list(source_timing, "warning_codes")
            .is_some_and(|codes| codes.iter().any(|code| code == "phrase_uncertain"))
    {
        return "short_loop_manual_confirm".to_string();
    }

    "manual_confirm_only".to_string()
}

fn observer_source_timing_policy_cue(policy: &str) -> Option<&'static str> {
    match policy {
        "locked" | "manual_confirm" | "cautious" | "fallback_grid" | "disabled" | "unknown" => {
            Some(riotbox_app::source_timing_cues::source_timing_policy_cue_label(
                policy,
            ))
        }
        _ => None,
    }
}
