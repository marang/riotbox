fn source_timing_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) => vec![
            Line::from(format!(
                "readiness {} | {} | conf {:.2}",
                crate::source_timing_cues::source_timing_policy_cue_label(
                    timing_degraded_policy_machine_label(&graph.timing.effective_degraded_policy())
                ),
                graph
                    .timing
                    .bpm_estimate
                    .map(|bpm| format!("{bpm:.1} BPM"))
                    .unwrap_or_else(|| "unknown".into()),
                graph.timing.bpm_confidence
            )),
            source_timing_grid_readiness_line(graph),
            Line::from(format!(
                "meter {} | hypotheses {} | {}",
                graph
                    .timing
                    .meter_hint
                    .as_ref()
                    .map(|meter| format!("{}/{}", meter.beats_per_bar, meter.beat_unit))
                    .unwrap_or_else(|| "unknown".into()),
                graph.timing.hypotheses.len(),
                source_timing_anchor_compact(shell)
            )),
            Line::from(format!(
                "mode {} | trust {}",
                timing_degraded_policy_display_label(&graph.timing.effective_degraded_policy()),
                timing_quality_label(&graph.timing.effective_timing_quality()),
            )),
            Line::from(format!(
                "warnings {}",
                timing_warning_codes(&graph.timing.warnings)
            )),
        ],
        None => vec![Line::from("no timing information available")],
    }
}

fn source_timing_grid_readiness_line(
    graph: &riotbox_core::source_graph::SourceGraph,
) -> Line<'static> {
    Line::from(format!(
        "beat {} | downbeat {} | phrase {}",
        beat_grid_status_label(graph),
        downbeat_status_label(graph),
        phrase_status_label(graph)
    ))
}

fn beat_grid_status_label(graph: &riotbox_core::source_graph::SourceGraph) -> String {
    if !graph.timing.beat_grid.is_empty() {
        return format!("grid {}", graph.timing.beat_grid.len());
    }
    if graph.timing.bpm_estimate.is_some() {
        return "tempo only".into();
    }
    "unknown".into()
}

fn downbeat_status_label(graph: &riotbox_core::source_graph::SourceGraph) -> &'static str {
    if graph
        .timing
        .warnings
        .iter()
        .any(|warning| warning.code == TimingWarningCode::AmbiguousDownbeat)
    {
        return "ambiguous";
    }
    if !graph.timing.bar_grid.is_empty() {
        return "bar locked";
    }
    "unknown"
}

fn phrase_status_label(graph: &riotbox_core::source_graph::SourceGraph) -> &'static str {
    if graph
        .timing
        .warnings
        .iter()
        .any(|warning| warning.code == TimingWarningCode::PhraseUncertain)
    {
        return "uncertain";
    }
    if !graph.timing.phrase_grid.is_empty() {
        return "phrase locked";
    }
    "unknown"
}

fn timing_quality_label(quality: &TimingQuality) -> &'static str {
    match quality {
        TimingQuality::Low => "low",
        TimingQuality::Medium => "medium",
        TimingQuality::High => "high",
        TimingQuality::Unknown => "unknown",
    }
}

fn timing_degraded_policy_machine_label(policy: &TimingDegradedPolicy) -> &'static str {
    match policy {
        TimingDegradedPolicy::Locked => "locked",
        TimingDegradedPolicy::Cautious => "cautious",
        TimingDegradedPolicy::ManualConfirm => "manual_confirm",
        TimingDegradedPolicy::FallbackGrid => "fallback_grid",
        TimingDegradedPolicy::Disabled => "disabled",
        TimingDegradedPolicy::Unknown => "unknown",
    }
}

fn timing_degraded_policy_display_label(policy: &TimingDegradedPolicy) -> &'static str {
    match policy {
        TimingDegradedPolicy::Locked => "locked",
        TimingDegradedPolicy::Cautious => "listen first",
        TimingDegradedPolicy::ManualConfirm => "manual confirm",
        TimingDegradedPolicy::FallbackGrid => "fallback grid",
        TimingDegradedPolicy::Disabled => "disabled",
        TimingDegradedPolicy::Unknown => "unknown",
    }
}

fn timing_warning_codes(warnings: &[riotbox_core::source_graph::TimingWarning]) -> String {
    if warnings.is_empty() {
        return "none".into();
    }

    warnings
        .iter()
        .map(|warning| timing_warning_code_label(&warning.code))
        .collect::<Vec<_>>()
        .join(", ")
}

fn timing_warning_code_label(code: &TimingWarningCode) -> &'static str {
    match code {
        TimingWarningCode::WeakKickAnchor => "weak_kick_anchor",
        TimingWarningCode::WeakBackbeatAnchor => "weak_backbeat_anchor",
        TimingWarningCode::AmbiguousDownbeat => "ambiguous_downbeat",
        TimingWarningCode::HalfTimePossible => "half_time_possible",
        TimingWarningCode::DoubleTimePossible => "double_time_possible",
        TimingWarningCode::DriftHigh => "drift_high",
        TimingWarningCode::PhraseUncertain => "phrase_uncertain",
        TimingWarningCode::LowTimingConfidence => "low_timing_confidence",
    }
}
