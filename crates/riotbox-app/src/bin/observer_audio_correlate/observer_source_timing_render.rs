fn format_observer_source_timing(summary: &CorrelationSummary) -> String {
    if summary.observer_source_timing_malformed {
        return "malformed".to_string();
    }
    summary.observer_source_timing.as_ref().map_or_else(
        || "unknown".to_string(),
        |timing| {
            format!(
                "{} cue={} actionability={} grid_use={} quality={} policy={} bpm={} confidence={:.3} beat={}({}) downbeat={}({}) offset={} downbeat_score={} downbeat_gap={} downbeat_alts={} phrase={}({}) anchors={} anchor_cue=\"{}\" groove={} warning={}",
                timing.source_id,
                timing.cue,
                timing.actionability,
                timing.grid_use,
                timing.quality,
                timing.degraded_policy,
                format_optional_f64(timing.bpm_estimate),
                timing.bpm_confidence,
                timing.beat_status,
                timing.beat_count,
                timing.downbeat_status,
                timing.bar_count,
                timing
                    .primary_downbeat_offset_beats
                    .map_or_else(|| "none".to_string(), |offset| offset.to_string()),
                format_optional_f64(timing.primary_downbeat_score),
                format_optional_f64(timing.primary_downbeat_score_gap),
                timing.alternate_downbeat_phase_count,
                timing.phrase_status,
                timing.phrase_count,
                format_source_timing_anchor_counts(timing.anchor_evidence.as_ref()),
                timing.primary_anchor_cue,
                format_source_timing_groove_counts(timing.groove_evidence.as_ref()),
                timing
                    .primary_warning_code
                    .as_deref()
                    .unwrap_or("none")
            )
        },
    )
}
