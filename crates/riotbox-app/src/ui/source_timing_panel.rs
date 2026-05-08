fn source_timing_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) => vec![
            Line::from(format!(
                "tempo {} | conf {:.2}",
                graph
                    .timing
                    .bpm_estimate
                    .map(|bpm| format!("{bpm:.1} BPM"))
                    .unwrap_or_else(|| "unknown".into()),
                graph.timing.bpm_confidence
            )),
            source_timing_cue_line(
                "cue",
                timing_degraded_policy_label(&graph.timing.effective_degraded_policy()),
                [
                    (
                        "quality",
                        timing_quality_label(&graph.timing.effective_timing_quality()),
                    ),
                    (
                        "policy",
                        timing_degraded_policy_label(&graph.timing.effective_degraded_policy()),
                    ),
                ],
            ),
            Line::from(format!(
                "meter {} | hyp {} primary {}",
                graph
                    .timing
                    .meter_hint
                    .as_ref()
                    .map(|meter| format!("{}/{}", meter.beats_per_bar, meter.beat_unit))
                    .unwrap_or_else(|| "unknown".into()),
                graph.timing.hypotheses.len(),
                graph
                    .timing
                    .primary_hypothesis_id
                    .as_deref()
                    .unwrap_or("none")
            )),
            Line::from(format!(
                "beats {} | bars {} | phrases {}",
                graph.timing.beat_grid.len(),
                graph.timing.bar_grid.len(),
                graph.timing.phrase_grid.len()
            )),
            Line::from(format!(
                "warnings {}",
                timing_warning_codes(&graph.timing.warnings)
            )),
        ],
        None => vec![Line::from("no timing information available")],
    }
}

fn timing_quality_label(quality: &TimingQuality) -> &'static str {
    match quality {
        TimingQuality::Low => "low",
        TimingQuality::Medium => "medium",
        TimingQuality::High => "high",
        TimingQuality::Unknown => "unknown",
    }
}

fn timing_degraded_policy_label(policy: &TimingDegradedPolicy) -> &'static str {
    match policy {
        TimingDegradedPolicy::Locked => "locked",
        TimingDegradedPolicy::Cautious => "cautious",
        TimingDegradedPolicy::ManualConfirm => "manual_confirm",
        TimingDegradedPolicy::FallbackGrid => "fallback_grid",
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
