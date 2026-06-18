use ratatui::text::Line;
use riotbox_core::view::jam::CaptureTargetKindView;

use super::capture_handoff_readiness_label;
use crate::ui::{
    JamShellState, capture_or_recall_cue_label, w30_bank_manager_compact,
    w30_capture_lineage_compact, w30_damage_profile_compact, w30_loop_freeze_compact,
    w30_pending_cue_label, w30_preview_source_readiness, w30_resample_lineage_active,
    w30_resample_mix_log_compact, w30_resample_route_compact, w30_resample_source_compact,
    w30_resample_tap_compact, w30_slice_pool_compact, w30_slice_pool_relevant, w30_target_compact,
};

pub(in crate::ui) fn capture_routing_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let latest_promoted = shell
        .app
        .jam_view
        .capture
        .latest_w30_promoted_capture_label
        .as_deref()
        .unwrap_or("none");
    let pending_w30 = w30_pending_cue_label(shell);
    let bank_or_pool_line = if w30_slice_pool_relevant(shell) {
        format!(
            "bank/pad {} | pool {}",
            w30_target_compact(shell),
            w30_slice_pool_compact(shell)
        )
    } else {
        format!(
            "bank/pad {} | mgr {}",
            w30_target_compact(shell),
            w30_bank_manager_compact(shell)
        )
    };
    let mut lines = vec![
        Line::from(format!("pending W-30 cue {pending_w30}")),
        Line::from(bank_or_pool_line),
        Line::from({
            let mut line = format!(
                "preview {} | {}",
                shell.app.runtime_view.w30_preview_mode,
                shell.app.runtime_view.w30_preview_mix_summary,
            );
            if let Some(readiness) = w30_preview_source_readiness(shell) {
                line.push_str(" | ");
                line.push_str(readiness);
            }
            line
        }),
        Line::from(format!(
            "forge {} | tap {}",
            w30_damage_profile_compact(shell),
            w30_resample_tap_compact(shell),
        )),
    ];

    if w30_resample_lineage_active(shell) {
        lines.push(Line::from(format!(
            "tap {} | route {}",
            w30_resample_source_compact(shell),
            w30_resample_route_compact(shell),
        )));
        lines.push(Line::from(format!(
            "tap mix {}",
            w30_resample_mix_log_compact(shell)
        )));
        lines.push(Line::from(format!(
            "freeze {}",
            w30_loop_freeze_compact(shell)
        )));
        lines.push(Line::from(format!(
            "lineage {}",
            w30_capture_lineage_compact(shell)
        )));
    } else {
        let last_target = shell
            .app
            .jam_view
            .capture
            .last_capture_target
            .as_deref()
            .unwrap_or("unassigned");
        lines.push(Line::from(format!("route {last_target}")));
        lines.push(Line::from(
            shell
                .app
                .jam_view
                .capture
                .last_promotion_result
                .clone()
                .unwrap_or_else(|| "promotion result pending".into()),
        ));
        lines.push(Line::from(format!(
            "freeze {}",
            w30_loop_freeze_compact(shell)
        )));
        lines.push(Line::from(format!("latest promoted {latest_promoted}")));
        lines.push(Line::from(format!(
            "last lane capture {}",
            shell
                .app
                .session
                .runtime_state
                .lane_state
                .w30
                .last_capture
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "none".into())
        )));
        lines.push(Line::from(format!(
            "next shell cue {}",
            capture_or_recall_cue_label(shell)
        )));
        lines.push(Line::from(
            "audition and recall stay on the shared next-bar seam",
        ));
        return lines;
    }

    lines.push(Line::from(format!("latest promoted {latest_promoted}")));
    lines
}

pub(super) fn capture_heard_path_label(shell: &JamShellState) -> String {
    let capture = &shell.app.jam_view.capture;
    let Some(last_capture_id) = capture.last_capture_id.as_deref() else {
        return "[c] first, then [p]->[w]".into();
    };

    match (
        capture.last_capture_target_kind,
        capture.last_capture_target.as_deref(),
    ) {
        (Some(CaptureTargetKindView::W30Pad), Some(target)) => {
            format!(
                "{last_capture_id}->{target} [w]/[o] {}",
                capture_handoff_readiness_label(shell)
            )
        }
        (Some(CaptureTargetKindView::Scene), Some(target)) => {
            format!("{last_capture_id}->{target} ready")
        }
        (_, Some(target)) if target != "unassigned" => format!("{last_capture_id}->{target} ready"),
        _ => {
            let readiness = capture_handoff_readiness_label(shell);
            if readiness == "unavailable" {
                format!("{last_capture_id} unavailable: recapture")
            } else {
                format!("{last_capture_id} src: [o] raw -> [p]->[w]")
            }
        }
    }
}
