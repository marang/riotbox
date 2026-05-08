fn source_timing_policy_failures(summary: &CorrelationSummary) -> Vec<String> {
    let Some(observer) = summary.observer_source_timing.as_ref() else {
        return Vec::new();
    };
    if observer.degraded_policy != "locked" {
        return Vec::new();
    }

    let mut failures = Vec::new();
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
