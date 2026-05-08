fn format_observer_source_timing(summary: &CorrelationSummary) -> String {
    if summary.observer_source_timing_malformed {
        return "malformed".to_string();
    }
    summary.observer_source_timing.as_ref().map_or_else(
        || "unknown".to_string(),
        |timing| {
            format!(
                "{} cue={} quality={} policy={} bpm={} confidence={:.3} beat={}({}) downbeat={}({}) phrase={}({}) warning={}",
                timing.source_id,
                timing.cue,
                timing.quality,
                timing.degraded_policy,
                format_optional_f64(timing.bpm_estimate),
                timing.bpm_confidence,
                timing.beat_status,
                timing.beat_count,
                timing.downbeat_status,
                timing.bar_count,
                timing.phrase_status,
                timing.phrase_count,
                timing
                    .primary_warning_code
                    .as_deref()
                    .unwrap_or("none")
            )
        },
    )
}
