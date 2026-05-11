fn source_timing_policy_failures(summary: &CorrelationSummary) -> Vec<String> {
    let mut failures = source_timing_grid_use_policy_failures(summary);

    let Some(observer) = summary.observer_source_timing.as_ref() else {
        return failures;
    };
    if observer.degraded_policy != "locked" {
        return failures;
    }

    if summary.grid_bpm_source != "source_timing" {
        failures.push(format!(
            "source_timing_policy.locked_observer_grid_bpm_source={}",
            summary.grid_bpm_source
        ));
    }
    if summary.grid_bpm_decision_reason != "source_timing_ready" {
        failures.push(format!(
            "source_timing_policy.locked_observer_grid_bpm_decision_reason={}",
            summary.grid_bpm_decision_reason
        ));
    }
    match summary.source_timing.as_ref() {
        Some(source_timing) if source_timing.requires_manual_confirm => {
            failures.push("source_timing_policy.locked_observer_requires_manual_confirm=true".into());
        }
        Some(source_timing) if source_timing.readiness != "ready" => {
            failures.push(format!(
                "source_timing_policy.locked_observer_readiness={}",
                source_timing.readiness
            ));
        }
        Some(_) => {}
        None => failures.push("source_timing_policy.locked_observer_missing_manifest_timing".into()),
    }
    failures
}

fn source_timing_grid_use_policy_failures(summary: &CorrelationSummary) -> Vec<String> {
    let Some(source_timing) = summary.source_timing.as_ref() else {
        return Vec::new();
    };
    let Some(grid_use) = source_timing.grid_use.as_deref() else {
        return Vec::new();
    };

    let expected = match (
        summary.grid_bpm_source.as_str(),
        summary.grid_bpm_decision_reason.as_str(),
    ) {
        ("source_timing", "source_timing_ready") => Some("locked_grid"),
        ("source_timing", "source_timing_needs_review_manual_confirm") => {
            Some("short_loop_manual_confirm")
        }
        ("static_default", _) if grid_use == "locked_grid" => Some("not_locked_grid"),
        ("static_default", _) if grid_use == "short_loop_manual_confirm" => {
            Some("not_short_loop_manual_confirm")
        }
        _ => None,
    };

    match expected {
        Some(expected) if grid_use != expected => vec![format!(
            "source_timing_policy.grid_use={grid_use} expected={expected} grid_bpm_source={} grid_bpm_decision_reason={}",
            summary.grid_bpm_source, summary.grid_bpm_decision_reason
        )],
        _ => Vec::new(),
    }
}
