mod pending_capture;
mod routing;

use ratatui::{text::Line, widgets::ListItem};
use riotbox_core::{
    action::ActionStatus,
    view::jam::{CaptureHandoffReadinessView, CaptureTargetKindView},
};

use super::{JamShellState, transport_label};

#[cfg(test)]
pub(super) use pending_capture::{capture_pending_detail_line, capture_pending_intent_line};
use pending_capture::{pending_capture_do_next_lines, pending_w30_audition_do_next_lines};
use routing::capture_heard_path_label;
pub(super) use routing::capture_routing_lines;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FirstRunOnrampStage {
    Start,
    QueuedFirstMove,
    FirstResult,
}

pub(super) fn first_run_onramp_stage(shell: &JamShellState) -> Option<FirstRunOnrampStage> {
    if !shell.first_run_onramp {
        return None;
    }

    let committed_count = shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .filter(|action| action.status == ActionStatus::Committed)
        .count();
    let has_pending = !shell.app.jam_view.pending_actions.is_empty();
    let capture_count = shell.app.jam_view.capture.capture_count;

    if capture_count > 0 || committed_count > 1 {
        return None;
    }

    if committed_count == 0 {
        return Some(if has_pending {
            FirstRunOnrampStage::QueuedFirstMove
        } else {
            FirstRunOnrampStage::Start
        });
    }

    Some(FirstRunOnrampStage::FirstResult)
}

pub(super) fn capture_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let capture = &shell.app.jam_view.capture;
    vec![
        Line::from(format!("captures {}", capture.capture_count)),
        Line::from(format!(
            "last {}",
            capture.last_capture_id.as_deref().unwrap_or("none")
        )),
        Line::from(format!(
            "target {}",
            capture
                .last_capture_target
                .as_deref()
                .unwrap_or("unassigned")
        )),
        Line::from(format!("origins {}", capture.last_capture_origin_count)),
        Line::from(
            capture
                .last_capture_notes
                .clone()
                .unwrap_or_else(|| "no capture note yet".into()),
        ),
    ]
}

pub(super) fn capture_readiness_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let pending_capture_count = shell.app.jam_view.capture.pending_capture_count;
    let bank = shell
        .app
        .jam_view
        .lanes
        .w30_active_bank
        .as_deref()
        .unwrap_or("unset");

    vec![
        Line::from(format!(
            "transport {} | beat {:.1}",
            transport_label(shell),
            shell.app.jam_view.transport.position_beats
        )),
        Line::from(format!("pending capture actions {pending_capture_count}")),
        Line::from(format!("w30 bank {bank}")),
        Line::from(format!(
            "last lane capture {}",
            shell
                .app
                .jam_view
                .capture
                .last_capture_id
                .as_deref()
                .unwrap_or("none")
        )),
    ]
}

pub(super) fn capture_latest_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let capture = &shell.app.jam_view.capture;
    vec![
        Line::from(format!("captures total {}", capture.capture_count)),
        Line::from(format!(
            "pinned {} | promoted {}",
            capture.pinned_capture_count, capture.promoted_capture_count
        )),
        Line::from(format!("hear {}", capture_heard_path_label(shell))),
        Line::from(format!(
            "latest {}",
            capture.last_capture_id.as_deref().unwrap_or("none")
        )),
        Line::from(format!(
            "target {}",
            capture
                .last_capture_target
                .as_deref()
                .unwrap_or("unassigned")
        )),
        Line::from(format!("origin refs {}", capture.last_capture_origin_count)),
        Line::from(
            capture
                .last_promotion_result
                .clone()
                .or_else(|| capture.last_capture_notes.clone())
                .unwrap_or_else(|| "no capture note yet".into()),
        ),
    ]
}

pub(super) fn capture_do_next_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let capture = &shell.app.jam_view.capture;
    let handoff_readiness = capture_handoff_readiness_label(shell);
    if let Some(lines) = pending_capture_do_next_lines(capture, handoff_readiness) {
        return lines;
    }
    if let Some(lines) = pending_w30_audition_do_next_lines(shell) {
        return lines;
    }

    let Some(last_capture_id) = capture.last_capture_id.as_deref() else {
        return vec![
            Line::from("1 [c] capture phrase"),
            Line::from("2 [p] promote keeper"),
            Line::from("3 [w] hit promoted pad"),
            Line::from("use Log to confirm"),
        ];
    };

    match (
        capture.last_capture_target_kind,
        capture.last_capture_target.as_deref(),
    ) {
        (Some(CaptureTargetKindView::W30Pad), Some(target)) => {
            if handoff_readiness == "fallback" {
                vec![
                    Line::from(format!("fallback: [w]/[o] safe {target}")),
                    Line::from("[3] Source shows why"),
                    Line::from("[c] new capture can become src"),
                    Line::from(format!("source {last_capture_id}")),
                ]
            } else {
                vec![
                    Line::from(format!("hear now: [w] hit {target} ({handoff_readiness})")),
                    Line::from("or [o] audition same pad"),
                    Line::from("[b]/[s] browse or swap"),
                    Line::from(format!("source {last_capture_id}")),
                ]
            }
        }
        (Some(CaptureTargetKindView::Scene), Some(target)) => vec![
            Line::from(format!("scene target {target}")),
            Line::from("use Jam scene controls"),
            Line::from("[2] confirm action trail"),
            Line::from(format!("source {last_capture_id}")),
        ],
        _ => vec![
            Line::from(format!(
                "1 hear it: [o] raw {last_capture_id} ({handoff_readiness})"
            )),
            Line::from(format!("2 keep it: [p] promote {last_capture_id}")),
            Line::from(format!(
                "3 play it: [w] hit after promote ({handoff_readiness})"
            )),
            Line::from(capture_handoff_help_line(handoff_readiness)),
        ],
    }
}

fn capture_handoff_readiness_label(shell: &JamShellState) -> &'static str {
    match shell.app.jam_view.capture.last_capture_handoff_readiness {
        Some(CaptureHandoffReadinessView::Source) => "src",
        Some(CaptureHandoffReadinessView::Fallback) | None => "fallback",
    }
}

fn capture_handoff_help_line(handoff_readiness: &str) -> &'static str {
    if handoff_readiness == "fallback" {
        "if still fallback: [3] Source"
    } else {
        "[2] confirm result"
    }
}

pub(super) fn capture_provenance_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let lines = &shell.app.jam_view.capture.latest_capture_provenance_lines;
    if lines.is_empty() {
        return vec![Line::from("no captured material yet")];
    }

    lines.iter().cloned().map(Line::from).collect()
}

pub(super) fn pending_capture_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let pending = &shell.app.jam_view.capture.pending_capture_items;
    if pending.is_empty() {
        return vec![Line::from("no queued capture actions")];
    }

    let action = &pending[0];
    let mut lines = vec![
        Line::from(format!("next {} {}", action.actor, action.command)),
        Line::from(format!(
            "when {} | target {}",
            action.quantization, action.target
        )),
    ];
    if let Some(explanation) = &action.explanation {
        lines.push(Line::from(format!("note {explanation}")));
    }

    let overflow_count = pending.len().saturating_sub(1);
    if overflow_count > 0 {
        lines.push(Line::from(format!("+{overflow_count} more in [2] Log")));
    }

    lines
}

pub(super) fn recent_capture_items(shell: &JamShellState) -> Vec<ListItem<'static>> {
    let rows = &shell.app.jam_view.capture.recent_capture_rows;
    if rows.is_empty() {
        return vec![ListItem::new("no captures stored yet")];
    }

    rows.iter().cloned().map(ListItem::new).collect()
}
