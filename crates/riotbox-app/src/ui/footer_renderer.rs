use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use riotbox_core::view::jam::SceneJumpAvailabilityView;

use super::{
    ADVANCED_GESTURES, GESTURE_CAPTURE, GESTURE_FILL, GESTURE_HIT, GESTURE_SCENE_JUMP,
    GESTURE_UNDO, JamShellState, JamViewMode, LANE_GESTURES, ShellScreen, footer_ok_line,
    footer_scene_affordance_cue, footer_status_line, footer_warning_line, recovery_warning_line,
    render_gesture_items, source_monitor_route_compact_label, spans_with_primary_gesture_keys,
    spans_with_primary_legend_keys, style_pending_cue, style_primary_control,
};

pub(super) fn render_footer(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let content_rows = usize::from(area.height.saturating_sub(2));
    let inspect_key_label =
        if shell.active_screen == ShellScreen::Jam && shell.jam_mode == JamViewMode::Inspect {
            "i return to perform"
        } else {
            "i jam inspect"
        };
    let keys_line = if area.width < 100 {
        footer_compact_keys_line(shell.launch_mode.refresh_verb())
    } else {
        footer_keys_line(inspect_key_label, shell.launch_mode.refresh_verb())
    };
    let primary_line = if shell.active_screen == ShellScreen::Jam
        && shell.jam_mode == JamViewMode::Inspect
    {
        Line::from("Inspect is read-only: use i to return, then queue actions from perform mode")
    } else {
        footer_primary_line(&render_primary_gesture_items(shell))
    };
    let secondary_line =
        if shell.active_screen == ShellScreen::Jam && shell.jam_mode == JamViewMode::Inspect {
            None
        } else if let Some(scene_cue) = footer_scene_affordance_cue(shell) {
            Some(footer_scene_line(&scene_cue))
        } else {
            Some(footer_advanced_line(&render_gesture_items(
                ADVANCED_GESTURES,
                " ",
            )))
        };
    let lane_line = footer_lane_ops_line(&render_gesture_items(LANE_GESTURES, " "));
    let status_line = footer_status_line(&format!(
        "Status: audio {} | monitor {}/{} | {}",
        shell.app.runtime_view.audio_status,
        shell.app.runtime_view.source_monitor_mode,
        source_monitor_route_compact_label(shell),
        shell.status_message,
    ));
    let recovery_warning = recovery_warning_line(shell);
    let health_line = if let Some(recovery_warning) = recovery_warning.as_deref() {
        footer_warning_line(recovery_warning)
    } else if let Some(warning) = shell
        .app
        .runtime_view
        .runtime_warnings
        .iter()
        .chain(shell.app.jam_view.warnings.iter())
        .next()
    {
        footer_warning_line(warning)
    } else {
        footer_ok_line("Warnings clear | source trust stable enough for shell work")
    };

    let mut lines = vec![keys_line, primary_line];
    if content_rows >= 6 {
        if let Some(secondary_line) = secondary_line {
            lines.push(secondary_line);
        }
        lines.push(lane_line);
        lines.push(status_line);
        lines.push(health_line);
    } else if content_rows >= 5 {
        if let Some(secondary_line) = secondary_line {
            lines.push(secondary_line);
        }
        lines.push(status_line);
        lines.push(health_line);
    } else if area.width >= 100 && recovery_warning.is_none() {
        if let Some(secondary_line) = secondary_line {
            lines.push(secondary_line);
        }
        lines.push(combine_footer_health(status_line, health_line));
    } else {
        lines.push(status_line);
        lines.push(health_line);
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(Line::from("Footer").style(Style::default().add_modifier(Modifier::BOLD)))
            .borders(Borders::ALL),
    );

    frame.render_widget(paragraph, area);
}

fn combine_footer_health(
    mut status_line: Line<'static>,
    health_line: Line<'static>,
) -> Line<'static> {
    status_line.spans.push(Span::raw(" | "));
    status_line.spans.extend(health_line.spans);
    status_line
}

fn footer_compact_keys_line(refresh_verb: &str) -> Line<'static> {
    let legend = format!(
        "q quit | ? help | 1-4 screens | space play/pause | M monitor | r {}",
        compact_refresh_verb(refresh_verb),
    );
    let mut spans = vec![Span::raw("Keys: ")];
    spans.extend(spans_with_primary_legend_keys(&legend));
    Line::from(spans)
}

fn render_primary_gesture_items(shell: &JamShellState) -> String {
    let gestures = [
        ("w", GESTURE_HIT),
        ("f", GESTURE_FILL),
        ("s", "slam"),
        ("y", scene_jump_primary_label(shell)),
        ("c", GESTURE_CAPTURE),
        ("u", GESTURE_UNDO),
    ];

    render_gesture_items(&gestures, " ")
}

pub(super) fn render_help_primary_gesture_items(shell: &JamShellState) -> String {
    let gestures = [
        ("w", GESTURE_HIT),
        ("f", GESTURE_FILL),
        ("s", "slam"),
        ("y", scene_jump_primary_label(shell)),
    ];

    render_gesture_items(&gestures, ": ")
}

fn scene_jump_primary_label(shell: &JamShellState) -> &'static str {
    match shell.app.jam_view.scene.scene_jump_availability {
        SceneJumpAvailabilityView::WaitingForMoreScenes => "jump waits",
        SceneJumpAvailabilityView::Ready | SceneJumpAvailabilityView::Unknown => GESTURE_SCENE_JUMP,
    }
}

pub(super) fn footer_keys_line(inspect_key_label: &str, refresh_verb: &str) -> Line<'static> {
    let legend = format!(
        "q quit | ? help | 1-4 screens | Tab switch | {} | space play/pause | M monitor | [ ] drum | r {}",
        compact_inspect_key_label(inspect_key_label),
        compact_refresh_verb(refresh_verb),
    );
    let mut spans = vec![Span::raw("Keys: ")];
    spans.extend(spans_with_primary_legend_keys(&legend));
    Line::from(spans)
}

fn compact_inspect_key_label(inspect_key_label: &str) -> &str {
    match inspect_key_label {
        "i jam inspect" => "i inspect",
        "i return to perform" => "i perform",
        _ => inspect_key_label,
    }
}

fn compact_refresh_verb(refresh_verb: &str) -> &str {
    match refresh_verb {
        "re-ingest source" => "re-ingest",
        "reload session" => "reload",
        _ => refresh_verb,
    }
}

pub(super) fn footer_primary_line(gestures: &str) -> Line<'static> {
    let mut spans = vec![
        Span::styled("Primary:", style_primary_control()),
        Span::raw(" "),
    ];
    spans.extend(spans_with_primary_gesture_keys(gestures));
    Line::from(spans)
}

pub(super) fn footer_advanced_line(gestures: &str) -> Line<'static> {
    let mut spans = vec![Span::raw("Advanced: ")];
    spans.extend(spans_with_primary_gesture_keys(gestures));
    spans.push(Span::raw(" | more in ? help"));
    Line::from(spans)
}

pub(super) fn footer_lane_ops_line(gestures: &str) -> Line<'static> {
    let mut spans = vec![Span::raw("Lane ops: ")];
    spans.extend(spans_with_primary_gesture_keys(gestures));
    Line::from(spans)
}

pub(super) fn footer_scene_line(scene_cue: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("Scene:", style_pending_cue()),
        Span::styled(format!(" {scene_cue}"), style_pending_cue()),
    ])
}
