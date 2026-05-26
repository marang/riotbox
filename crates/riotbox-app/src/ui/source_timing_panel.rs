use ratatui::text::Line;

use super::{JamShellState, source_timing_grid_confirmed};

pub(super) fn source_timing_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let timing = &shell.app.jam_view.source.timing;
    match shell.app.source_graph.as_ref() {
        Some(graph) => vec![
            Line::from(format!(
                "ready {} | {} | c{:.2}",
                source_timing_cue_compact_label(shell),
                graph
                    .timing
                    .bpm_estimate
                    .map(|bpm| format!("{bpm:.1} BPM"))
                    .unwrap_or_else(|| "unknown".into()),
                graph.timing.bpm_confidence
            )),
            source_timing_grid_readiness_line(timing),
            Line::from(format!(
                "meter {} | hyp {} | anchors {}",
                graph
                    .timing
                    .meter_hint
                    .as_ref()
                    .map(|meter| format!("{}/{}", meter.beats_per_bar, meter.beat_unit))
                    .unwrap_or_else(|| "unknown".into()),
                graph.timing.hypotheses.len(),
                source_timing_anchor_display_label(timing)
            )),
            Line::from(format!(
                "mode {} | grid {} | trust {}",
                source_timing_degraded_policy_compact_label(&timing.degraded_policy),
                source_timing_grid_compact_label(&timing.grid_use),
                timing.quality,
            )),
            Line::from(format!(
                "act {} | warn {}",
                source_timing_action_compact_label(shell, &timing.actionability),
                source_timing_warning_compact_label(timing.primary_warning.as_deref()),
            )),
        ],
        None => vec![
            Line::from(format!(
                "readiness {} | trust {}",
                timing.cue, timing.quality
            )),
            Line::from(format!(
                "mode {} | grid {}",
                source_timing_degraded_policy_display_label(&timing.degraded_policy),
                timing.grid_use
            )),
            Line::from(format!(
                "warning {}",
                timing.primary_warning.as_deref().unwrap_or("none")
            )),
            Line::from(format!("action {}", timing.actionability)),
            Line::from("no timing information available"),
        ],
    }
}

fn source_timing_cue_compact_label(shell: &JamShellState) -> String {
    if source_timing_grid_confirmed(shell) {
        "grid confirmed".into()
    } else {
        shell.app.jam_view.source.timing.cue.clone()
    }
}

fn source_timing_action_compact_label(shell: &JamShellState, actionability: &str) -> &'static str {
    if source_timing_grid_confirmed(shell) {
        return "user confirmed";
    }
    match actionability {
        "confirm grid first" => "confirm grid",
        "grid can steer moves" => "grid steer",
        "listen first" => "listen first",
        "using safe fallback grid" => "fallback",
        "timing unavailable" => "unavailable",
        _ => "unknown",
    }
}

fn source_timing_warning_compact_label(warning: Option<&str>) -> &'static str {
    match warning {
        Some("ambiguous_downbeat") => "ambiguous",
        Some("phrase_uncertain") => "phrase",
        Some("low_timing_confidence") => "low_conf",
        Some("sparse_onsets") => "sparse",
        Some("weak_kick_anchor") => "weak_kick",
        Some("weak_backbeat_anchor") => "weak_backbeat",
        Some("drift_high") => "drift",
        Some(_) => "other",
        None => "none",
    }
}

pub(super) fn source_map_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let source_map = &shell.app.jam_view.source.source_map;
    let energy = source_map_compact_row(&source_map.energy_row, 18);
    let peaks = source_map_compact_row(&source_map.peak_row, 18);
    let bars = source_map_compact_row(&source_map.grid_row, 8);
    let play = source_map_compact_row(&source_map.playhead_row, 8);
    vec![
        Line::from(format!(
            "mode {} | {}",
            source_map.mode.label(),
            source_map.trust_label
        )),
        Line::from(source_map.current_region_label.clone()),
        Line::from(format!("energy {energy}")),
        Line::from(format!("peaks  {peaks}")),
        Line::from(format!(
            "b {bars} p {play} | {}",
            source_map_navigation_compact(&source_map.navigation_hint)
        )),
    ]
}

fn source_map_compact_row(row: &str, width: usize) -> String {
    let chars = row.chars().collect::<Vec<_>>();
    if chars.len() <= width {
        return row.into();
    }

    (0..width)
        .map(|index| {
            let start = index * chars.len() / width;
            let end = ((index + 1) * chars.len()).div_ceil(width);
            source_map_best_bucket_char(&chars[start..end.min(chars.len())])
        })
        .collect()
}

fn source_map_best_bucket_char(chars: &[char]) -> char {
    ['^', '|', '█', '▇', '▅', '▂', '▁', '.']
        .into_iter()
        .find(|candidate| chars.contains(candidate))
        .unwrap_or(' ')
}

fn source_map_navigation_compact(navigation_hint: &str) -> &'static str {
    if navigation_hint.contains("Up/Down") {
        "nav b/p"
    } else if navigation_hint.contains("Left/Right") {
        "nav bar"
    } else {
        "nav -"
    }
}

fn source_timing_grid_readiness_line(
    timing: &riotbox_core::view::jam::SourceTimingSummaryView,
) -> Line<'static> {
    Line::from(format!(
        "beat {} | bars {} | phase {} | phr {}",
        source_timing_beat_display_label(timing),
        timing.bar_count,
        source_timing_phase_compact_label(timing),
        source_timing_phrase_compact_label(timing)
    ))
}

fn source_timing_phase_compact_label(
    timing: &riotbox_core::view::jam::SourceTimingSummaryView,
) -> String {
    let phase = timing
        .primary_downbeat_offset_beats
        .map_or_else(|| "p-".into(), |offset| format!("p{offset}"));
    if timing.downbeat_status == "ambiguous" {
        format!("{phase} amb")
    } else {
        phase
    }
}

fn source_timing_phrase_compact_label(
    timing: &riotbox_core::view::jam::SourceTimingSummaryView,
) -> String {
    match timing.phrase_status.as_str() {
        "unknown" => timing.phrase_count.to_string(),
        "uncertain" => format!("u{}", timing.phrase_count),
        "phrase_locked" => format!("p{}", timing.phrase_count),
        _ => format!(
            "{}{}",
            source_timing_status_display_label(&timing.phrase_status),
            timing.phrase_count
        ),
    }
}

fn source_timing_anchor_display_label(
    timing: &riotbox_core::view::jam::SourceTimingSummaryView,
) -> &'static str {
    if timing.primary_kick_anchor_count > 0 && timing.primary_backbeat_anchor_count > 0 {
        "kick+bb"
    } else if timing.primary_kick_anchor_count > 0 {
        "kick"
    } else if timing.primary_backbeat_anchor_count > 0 {
        "backbeat"
    } else if timing.primary_transient_anchor_count > 0 {
        "transient"
    } else {
        "none"
    }
}

fn source_timing_status_display_label(status: &str) -> String {
    status.replace('_', " ")
}

fn source_timing_beat_display_label(
    timing: &riotbox_core::view::jam::SourceTimingSummaryView,
) -> String {
    if timing.beat_status == "grid" {
        format!("grid {}", timing.beat_count)
    } else if timing.beat_status == "tempo_only" {
        "tempo".into()
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

fn source_timing_degraded_policy_compact_label(policy: &str) -> &'static str {
    match policy {
        "manual_confirm" => "manual",
        _ => source_timing_degraded_policy_display_label(policy),
    }
}

fn source_timing_grid_compact_label(grid_use: &str) -> &'static str {
    match grid_use {
        "locked_grid" => "locked",
        "manual_confirm_only" => "manual",
        "short_loop_manual_confirm" => "short loop",
        "fallback_grid" => "fallback",
        "unavailable" => "unavailable",
        _ => "unknown",
    }
}
