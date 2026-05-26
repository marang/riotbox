use ratatui::{
    style::Style,
    text::{Line, Span},
};
use riotbox_core::{
    source_graph::{CandidateType, EnergyClass, QualityClass, Section},
    view::jam::SourceTimingSummaryView,
};

use super::{
    JamShellState, scene_countdown_cue, style_confirmation_strong, style_low_emphasis,
    style_pending_cue, style_pending_detail,
};

pub(super) fn source_candidate_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) => {
            let scorecard = &shell.app.jam_view.source.feral_scorecard;
            let best_loop = graph
                .candidates
                .iter()
                .filter(|candidate| candidate.candidate_type == CandidateType::LoopCandidate)
                .max_by(|left, right| left.score.total_cmp(&right.score));
            let best_hook = graph
                .candidates
                .iter()
                .filter(|candidate| candidate.candidate_type == CandidateType::HookCandidate)
                .max_by(|left, right| left.score.total_cmp(&right.score));

            vec![
                Line::from(format!(
                    "feral {} | break {}",
                    scorecard.readiness, scorecard.break_rebuild_potential
                )),
                Line::from(format!(
                    "quote risk {} | support {}",
                    scorecard.quote_risk_count, scorecard.break_support_count
                )),
                Line::from(format!(
                    "hooks {} | capture {}",
                    scorecard.hook_fragment_count, scorecard.capture_candidate_count
                )),
                Line::from(format!("use {}", scorecard.top_reason)),
                Line::from(format!(
                    "feral warn {}",
                    if scorecard.warnings.is_empty() {
                        "none".into()
                    } else {
                        scorecard.warnings.join(", ")
                    }
                )),
                Line::from(format!(
                    "loops {} | hooks {}",
                    graph.loop_candidate_count(),
                    graph.hook_candidate_count()
                )),
                Line::from(format!(
                    "best loop {}",
                    best_loop
                        .map(|candidate| format!(
                            "{:.2} ({:.2})",
                            candidate.score, candidate.confidence
                        ))
                        .unwrap_or_else(|| "none".into())
                )),
                Line::from(format!(
                    "best hook {}",
                    best_hook
                        .map(|candidate| format!(
                            "{:.2} ({:.2})",
                            candidate.score, candidate.confidence
                        ))
                        .unwrap_or_else(|| "none".into())
                )),
                Line::from(format!("assets {}", graph.assets.len())),
            ]
        }
        None => vec![Line::from("no candidate information available")],
    }
}

pub(super) fn source_provenance_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) => vec![
            Line::from(format!("sidecar {}", graph.provenance.sidecar_version)),
            Line::from(format!(
                "providers {}",
                if graph.provenance.provider_set.is_empty() {
                    "none".into()
                } else {
                    graph.provenance.provider_set.join(", ")
                }
            )),
            Line::from(format!("seed {}", graph.provenance.analysis_seed)),
            Line::from(format!("generated {}", graph.provenance.generated_at)),
        ],
        None => vec![Line::from("no provenance available")],
    }
}

pub(super) fn source_warning_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) if !graph.analysis_summary.warnings.is_empty() => graph
            .analysis_summary
            .warnings
            .iter()
            .take(4)
            .flat_map(|warning| {
                [
                    Line::from(format!("{}: {}", warning.code, warning.message)),
                    Line::from(""),
                ]
            })
            .collect(),
        Some(_) => vec![Line::from("no source-graph warnings")],
        None => vec![Line::from("no warnings because no source graph is loaded")],
    }
}

pub(super) fn source_confidence_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) => vec![
            Line::from(format!(
                "overall {:.2} | break potential {}",
                graph.analysis_summary.overall_confidence,
                quality_label(&graph.analysis_summary.break_rebuild_potential)
            )),
            Line::from(format!(
                "timing {} | section quality {}",
                quality_label(&graph.analysis_summary.timing_quality),
                quality_label(&graph.analysis_summary.section_quality)
            )),
            Line::from(format!(
                "summary loops {} | hooks {}",
                graph.analysis_summary.loop_candidate_count,
                graph.analysis_summary.hook_candidate_count
            )),
            Line::from(format!("jam trust {}", trust_summary(shell).headline)),
        ],
        None => vec![Line::from("no confidence summary available")],
    }
}

pub(super) struct TrustSummary {
    pub(super) headline: &'static str,
    pub(super) overall_confidence: f32,
    pub(super) warning_count: usize,
    pub(super) timing_quality: &'static str,
    pub(super) section_quality: &'static str,
    pub(super) source_timing_warning: Option<String>,
}

pub(super) fn trust_summary(shell: &JamShellState) -> TrustSummary {
    match shell.app.source_graph.as_ref() {
        Some(graph) => {
            let overall = graph.analysis_summary.overall_confidence;
            let headline = if overall >= 0.8 {
                "strong"
            } else if overall >= 0.62 {
                "usable"
            } else {
                "tentative"
            };

            TrustSummary {
                headline,
                overall_confidence: overall,
                warning_count: graph.analysis_summary.warnings.len(),
                timing_quality: quality_label(&graph.analysis_summary.timing_quality),
                section_quality: quality_label(&graph.analysis_summary.section_quality),
                source_timing_warning: shell.app.jam_view.source.timing.primary_warning.clone(),
            }
        }
        None => TrustSummary {
            headline: "unknown",
            overall_confidence: 0.0,
            warning_count: 0,
            timing_quality: "unknown",
            section_quality: "unknown",
            source_timing_warning: None,
        },
    }
}

fn quality_label(quality: &QualityClass) -> &'static str {
    match quality {
        QualityClass::Low => "low",
        QualityClass::Medium => "medium",
        QualityClass::High => "high",
        QualityClass::Unknown => "unknown",
    }
}

pub(super) fn source_timing_readiness_line(shell: &JamShellState) -> Line<'static> {
    let timing = &shell.app.jam_view.source.timing;
    let cue = source_timing_cue_label(shell);
    let actionability = source_timing_actionability_label(shell);
    Line::from(vec![
        Span::styled("timing ", style_low_emphasis()),
        Span::styled(
            cue,
            source_timing_policy_cue_style(shell, &timing.degraded_policy),
        ),
        Span::styled(" | ", style_low_emphasis()),
        Span::raw(actionability),
        Span::styled(" | ", style_low_emphasis()),
        Span::raw(timing.grid_use.clone()),
        Span::styled(" | ", style_low_emphasis()),
        Span::raw(source_timing_downbeat_phase_chip(timing)),
    ])
}

pub(super) fn source_timing_performance_rail_line(shell: &JamShellState) -> Line<'static> {
    let timing = &shell.app.jam_view.source.timing;
    let cue = source_timing_cue_label(shell);
    if shell.app.source_graph.is_none()
        || matches!(timing.degraded_policy.as_str(), "disabled" | "unknown")
    {
        return Line::from(vec![
            Span::styled("timing ", style_low_emphasis()),
            Span::styled(
                cue,
                source_timing_policy_cue_style(shell, &timing.degraded_policy),
            ),
            Span::styled(" | ", style_low_emphasis()),
            Span::styled("no clock", style_low_emphasis()),
        ]);
    }

    let transport = &shell.app.runtime.transport;
    Line::from(vec![
        Span::styled("timing ", style_low_emphasis()),
        Span::styled(
            cue,
            source_timing_policy_cue_style(shell, &timing.degraded_policy),
        ),
        Span::styled(" ", style_low_emphasis()),
        Span::styled(
            scene_countdown_cue(transport.beat_index),
            style_pending_cue(),
        ),
        Span::styled(" next bar", style_pending_cue()),
    ])
}

fn source_timing_anchor_kind_compact(shell: &JamShellState) -> &'static str {
    let timing = &shell.app.jam_view.source.timing;
    if timing.primary_kick_anchor_count > 0 && timing.primary_backbeat_anchor_count > 0 {
        "kick+bb"
    } else if timing.primary_kick_anchor_count > 0 {
        "kick"
    } else if timing.primary_backbeat_anchor_count > 0 {
        "backbeat"
    } else if timing.primary_transient_anchor_count > 0 {
        "transient"
    } else {
        "no anchor"
    }
}

pub(super) fn source_timing_clock_line(shell: &JamShellState) -> String {
    source_timing_clock_label(shell, false)
}

pub(super) fn source_timing_clock_compact(shell: &JamShellState) -> String {
    source_timing_clock_label(shell, true)
}

pub(super) fn source_timing_help_line(shell: &JamShellState) -> Line<'static> {
    let timing = &shell.app.jam_view.source.timing;
    let cue = source_timing_cue_label(shell);
    let actionability = source_timing_actionability_label(shell);

    Line::from(vec![
        Span::raw("Timing: "),
        Span::styled(
            cue,
            source_timing_policy_cue_style(shell, &timing.degraded_policy),
        ),
        Span::styled(" | ", style_low_emphasis()),
        Span::raw(format!("grid {}", timing.grid_use)),
        Span::styled(" | ", style_low_emphasis()),
        Span::raw(source_timing_downbeat_phase_help(timing)),
        Span::styled(" | ", style_low_emphasis()),
        Span::raw(timing.quality.clone()),
        Span::styled(" | ", style_low_emphasis()),
        Span::styled(
            source_timing_anchor_kind_compact(shell),
            style_pending_detail(),
        ),
        Span::styled(" | ", style_low_emphasis()),
        Span::raw(source_timing_help_clock_compact(shell)),
        Span::styled(" | ", style_low_emphasis()),
        Span::raw(actionability),
    ])
}

fn source_timing_downbeat_phase_chip(timing: &SourceTimingSummaryView) -> String {
    let phase = timing
        .primary_downbeat_offset_beats
        .map_or_else(|| "p-".into(), |offset| format!("p{offset}"));
    format!(
        "{phase}:b{}/{}/{}",
        timing.beat_count, timing.bar_count, timing.phrase_count
    )
}

fn source_timing_downbeat_phase_help(timing: &SourceTimingSummaryView) -> String {
    let phase = timing
        .primary_downbeat_offset_beats
        .map_or_else(|| "phase none".into(), |offset| format!("phase {offset}"));
    if timing.alternate_downbeat_phase_count == 0 {
        if timing.downbeat_status == "ambiguous" {
            return format!("{phase} amb");
        }
        return phase;
    }
    format!(
        "{phase} alt{} gap {}",
        timing.alternate_downbeat_phase_count,
        source_timing_optional_score_label(timing.primary_downbeat_score_gap)
    )
}

fn source_timing_optional_score_label(score: Option<f32>) -> String {
    score.map_or_else(|| "none".into(), |score| format!("{score:.3}"))
}

fn source_timing_clock_label(shell: &JamShellState, compact: bool) -> String {
    if shell.app.source_graph.is_none() {
        return if compact {
            "clock unavailable".into()
        } else {
            "source clock unavailable".into()
        };
    }

    let timing = &shell.app.jam_view.source.timing;
    if source_timing_clock_unavailable(timing) {
        return if compact {
            "clock unavailable".into()
        } else {
            "source clock unavailable".into()
        };
    }

    let clock = &shell.app.runtime.transport;
    let (beat, bar, phrase) = source_timing_clock_components(
        timing,
        clock.beat_index,
        clock.bar_index,
        clock.phrase_index,
    );
    if compact {
        format!("source b{beat} bar{bar} p{phrase}")
    } else {
        format!("source clock beat {beat} | bar {bar} | phrase {phrase}")
    }
}

fn source_timing_help_clock_compact(shell: &JamShellState) -> String {
    if shell.app.source_graph.is_none() {
        return "clock unavailable".into();
    }

    let timing = &shell.app.jam_view.source.timing;
    if source_timing_clock_unavailable(timing) {
        return "clock unavailable".into();
    }

    let clock = &shell.app.runtime.transport;
    let (beat, bar, phrase) = source_timing_clock_components(
        timing,
        clock.beat_index,
        clock.bar_index,
        clock.phrase_index,
    );
    format!("b{beat} bar{bar} p{phrase}")
}

fn source_timing_clock_unavailable(timing: &SourceTimingSummaryView) -> bool {
    matches!(timing.degraded_policy.as_str(), "disabled" | "unknown")
        || (timing.beat_count == 0 && timing.bar_count == 0 && timing.phrase_count == 0)
}

fn source_timing_clock_components(
    timing: &SourceTimingSummaryView,
    beat_index: u64,
    bar_index: u64,
    phrase_index: u64,
) -> (String, String, String) {
    let beat = if timing.beat_count > 0 {
        beat_index.to_string()
    } else {
        "-".into()
    };
    let bar = if timing.bar_count > 0 {
        bar_index.to_string()
    } else {
        "-".into()
    };
    let phrase = if timing.phrase_count > 0 {
        phrase_index.to_string()
    } else {
        "-".into()
    };
    (beat, bar, phrase)
}

pub(super) fn source_timing_warning_line(shell: &JamShellState) -> String {
    let trust = trust_summary(shell);
    trust
        .source_timing_warning
        .map(|warning| format!("timing warning {warning}"))
        .unwrap_or_else(|| "timing warning none".into())
}

pub(super) fn source_timing_grid_confirmed(shell: &JamShellState) -> bool {
    let Some(graph) = shell.app.source_graph.as_ref() else {
        return false;
    };
    shell
        .app
        .session
        .runtime_state
        .source_timing
        .confirmed_grid
        .as_ref()
        .is_some_and(|confirmed| {
            confirmed.source_id == graph.source.source_id
                && confirmed.hypothesis_id.as_deref()
                    == graph.timing.primary_hypothesis_id.as_deref()
        })
}

fn source_timing_cue_label(shell: &JamShellState) -> String {
    if source_timing_grid_confirmed(shell) {
        "grid confirmed".into()
    } else {
        shell.app.jam_view.source.timing.cue.clone()
    }
}

fn source_timing_actionability_label(shell: &JamShellState) -> String {
    if source_timing_grid_confirmed(shell) {
        "user confirmed".into()
    } else {
        shell.app.jam_view.source.timing.actionability.clone()
    }
}

fn source_timing_policy_cue_style(shell: &JamShellState, policy: &str) -> Style {
    if source_timing_grid_confirmed(shell) {
        return style_confirmation_strong();
    }
    match policy {
        "locked" => style_confirmation_strong(),
        "manual_confirm" | "cautious" | "fallback_grid" => style_pending_cue(),
        "disabled" | "unknown" => style_low_emphasis(),
        _ => style_low_emphasis(),
    }
}

pub(super) fn energy_label(section: &Section) -> &'static str {
    match section.energy_class {
        EnergyClass::Low => "low",
        EnergyClass::Medium => "medium",
        EnergyClass::High => "high",
        EnergyClass::Peak => "peak",
        EnergyClass::Unknown => "unknown",
    }
}
