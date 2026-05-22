use ratatui::text::Line;

use super::JamShellState;

pub(super) fn source_timing_lines(shell: &JamShellState) -> Vec<Line<'static>> {
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
            source_timing_grid_readiness_line(timing),
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
                "mode {} | grid {} | trust {}",
                source_timing_degraded_policy_display_label(&timing.degraded_policy),
                timing.grid_use,
                timing.quality,
            )),
            Line::from(format!(
                "action {} | warning {}",
                timing.actionability,
                timing.primary_warning.as_deref().unwrap_or("none"),
            )),
        ],
        None => vec![
            Line::from(format!(
                "readiness {} | trust {}",
                timing.cue, timing.quality
            )),
            Line::from(format!(
                "mode {} | grid {} | warning {}",
                source_timing_degraded_policy_display_label(&timing.degraded_policy),
                timing.grid_use,
                timing.primary_warning.as_deref().unwrap_or("none")
            )),
            Line::from(format!("action {}", timing.actionability)),
            Line::from("no timing information available"),
        ],
    }
}

fn source_timing_grid_readiness_line(
    timing: &riotbox_core::view::jam::SourceTimingSummaryView,
) -> Line<'static> {
    Line::from(format!(
        "beat {} | downbeat {} | phrase {}",
        source_timing_beat_display_label(timing),
        source_timing_downbeat_display_label(timing),
        source_timing_phrase_display_label(timing)
    ))
}

fn source_timing_phrase_display_label(
    timing: &riotbox_core::view::jam::SourceTimingSummaryView,
) -> String {
    format!(
        "{}({})",
        source_timing_status_display_label(&timing.phrase_status),
        timing.phrase_count
    )
}

fn source_timing_downbeat_display_label(
    timing: &riotbox_core::view::jam::SourceTimingSummaryView,
) -> String {
    let status = source_timing_status_display_label(&timing.downbeat_status);
    let offset = timing
        .primary_downbeat_offset_beats
        .map_or_else(|| "none".into(), |offset| offset.to_string());
    if timing.alternate_downbeat_phase_count == 0 {
        return format!("{status} off {offset}");
    }
    format!(
        "{status} off {offset} alt {} gap {}",
        timing.alternate_downbeat_phase_count,
        source_timing_optional_score_label(timing.primary_downbeat_score_gap)
    )
}

fn source_timing_optional_score_label(score: Option<f32>) -> String {
    score.map_or_else(|| "none".into(), |score| format!("{score:.3}"))
}

fn source_timing_status_display_label(status: &str) -> String {
    status.replace('_', " ")
}

fn source_timing_beat_display_label(
    timing: &riotbox_core::view::jam::SourceTimingSummaryView,
) -> String {
    if timing.beat_status == "grid" {
        format!("grid {}", timing.beat_count)
    } else {
        source_timing_status_display_label(&timing.beat_status)
    }
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
