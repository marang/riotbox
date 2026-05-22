use ratatui::text::{Line, Span};
use riotbox_core::view::jam::{CaptureSummaryView, W30PendingAuditionKind};

use super::JamShellState;
use crate::ui::{style_pending_cue, style_pending_detail};

pub(super) fn pending_w30_audition_do_next_lines(
    shell: &JamShellState,
) -> Option<Vec<Line<'static>>> {
    let pending = shell.app.jam_view.lanes.w30_pending_audition.as_ref()?;

    match pending.kind {
        W30PendingAuditionKind::RawCapture => Some(vec![
            capture_pending_intent_line(format!(
                "queued [o] audition raw @ {}",
                pending.quantization
            )),
            capture_pending_detail_line("wait, then hear raw preview"),
            capture_pending_detail_line(format!("target {}", pending.target)),
            capture_pending_detail_line("[2] confirm audition"),
        ]),
        W30PendingAuditionKind::Promoted => Some(vec![
            capture_pending_intent_line(format!(
                "queued [o] audition pad @ {}",
                pending.quantization
            )),
            capture_pending_detail_line("wait, then hear promoted preview"),
            capture_pending_detail_line(format!("target {}", pending.target)),
            capture_pending_detail_line("[2] confirm audition"),
        ]),
    }
}

pub(super) fn pending_capture_do_next_lines(
    capture: &CaptureSummaryView,
    handoff_readiness: &'static str,
) -> Option<Vec<Line<'static>>> {
    let pending = capture.pending_capture_items.first()?;

    if matches!(
        pending.command.as_str(),
        "capture.now" | "capture.loop" | "capture.bar_group" | "w30.capture_to_pad"
    ) {
        return Some(vec![
            capture_pending_intent_line(format!("queued [c] capture @ {}", pending.quantization)),
            capture_pending_detail_line("wait for commit"),
            capture_pending_detail_line("then [o] audition raw or [p] promote"),
            capture_pending_detail_line("[2] confirm capture"),
        ]);
    }

    if pending.command == "promote.capture_to_pad" {
        return Some(vec![
            capture_pending_intent_line(format!("queued [p] promote @ {}", pending.quantization)),
            capture_pending_detail_line(format!(
                "wait, then hear with [w] hit ({handoff_readiness})"
            )),
            capture_pending_detail_line(format!("target {}", pending.target)),
            capture_pending_detail_line("[2] confirm promotion"),
        ]);
    }

    if pending.command == "promote.capture_to_scene" {
        return Some(vec![
            capture_pending_intent_line(format!("queued scene promote @ {}", pending.quantization)),
            capture_pending_detail_line("wait for scene target"),
            capture_pending_detail_line(format!("target {}", pending.target)),
            capture_pending_detail_line("[2] confirm promotion"),
        ]);
    }

    if pending.command == "w30.loop_freeze" || pending.command == "promote.resample" {
        return Some(vec![
            capture_pending_intent_line(format!("queued W-30 reshape @ {}", pending.quantization)),
            capture_pending_detail_line("wait for phrase seam"),
            capture_pending_detail_line(format!("target {}", pending.target)),
            capture_pending_detail_line("[2] confirm result"),
        ]);
    }

    None
}

pub(in crate::ui) fn capture_pending_intent_line(message: impl Into<String>) -> Line<'static> {
    Line::from(Span::styled(message.into(), style_pending_cue()))
}

pub(in crate::ui) fn capture_pending_detail_line(message: impl Into<String>) -> Line<'static> {
    Line::from(Span::styled(message.into(), style_pending_detail()))
}
