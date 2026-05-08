fn source_timing_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let timing = &shell.app.jam_view.source.timing;
    match shell.app.source_graph.as_ref() {
        Some(graph) => vec![
            Line::from(format!(
                "readiness {} | {} | conf {:.2}",
                timing.cue,
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
                timing.primary_anchor_cue
            )),
            Line::from(format!(
                "mode {} | trust {}",
                source_timing_degraded_policy_display_label(&timing.degraded_policy),
                timing.quality,
            )),
            Line::from(format!(
                "warning {}",
                timing.primary_warning.as_deref().unwrap_or("none")
            )),
        ],
        None => vec![
            Line::from(format!(
                "readiness {} | trust {}",
                timing.cue, timing.quality
            )),
            Line::from(format!(
                "mode {} | warning {}",
                source_timing_degraded_policy_display_label(&timing.degraded_policy),
                timing.primary_warning.as_deref().unwrap_or("none")
            )),
            Line::from("no timing information available"),
        ],
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

fn source_timing_degraded_policy_display_label(policy: &str) -> &'static str {
    match policy {
        "locked" => "locked",
        "cautious" => "listen first",
        "manual_confirm" => "manual confirm",
        "fallback_grid" => "fallback grid",
        "disabled" => "disabled",
        _ => "unknown",
    }
}
