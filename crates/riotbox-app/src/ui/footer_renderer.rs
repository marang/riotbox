use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use riotbox_core::view::jam::SceneJumpAvailabilityView;

use super::{
    ADVANCED_GESTURES, GESTURE_CAPTURE, GESTURE_FILL, GESTURE_FOLLOW, GESTURE_HIT,
    GESTURE_SCENE_JUMP, GESTURE_UNDO, JamShellState, JamViewMode, LANE_GESTURES, ShellScreen,
    footer_ok_line, footer_scene_affordance_cue, footer_status_line, footer_warning_line,
    recovery_warning_line, render_gesture_items, spans_with_primary_gesture_keys,
    spans_with_primary_legend_keys, style_pending_cue, style_primary_control,
};

pub(super) fn render_footer(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let mut lines = Vec::new();
    let inspect_key_label =
        if shell.active_screen == ShellScreen::Jam && shell.jam_mode == JamViewMode::Inspect {
            "i return to perform"
        } else {
            "i jam inspect"
        };
    lines.push(footer_keys_line(
        inspect_key_label,
        shell.launch_mode.refresh_verb(),
    ));
    if shell.active_screen == ShellScreen::Jam && shell.jam_mode == JamViewMode::Inspect {
        lines.push(Line::from(
            "Inspect is read-only: use i to return, then queue actions from perform mode",
        ));
    } else {
        lines.push(footer_primary_line(&render_primary_gesture_items(shell)));
        if let Some(scene_cue) = footer_scene_affordance_cue(shell) {
            lines.push(footer_scene_line(&scene_cue));
        } else {
            lines.push(footer_advanced_line(&render_gesture_items(
                ADVANCED_GESTURES,
                " ",
            )));
        }
    }
    lines.push(footer_lane_ops_line(&render_gesture_items(
        LANE_GESTURES,
        " ",
    )));
    lines.push(footer_status_line(&format!(
        "Status: {} | jam {} | audio {} | sidecar {} | 909 render {} via {}",
        shell.status_message,
        shell.jam_mode.label(),
        shell.app.runtime_view.audio_status,
        shell.app.runtime_view.sidecar_status,
        shell.app.runtime_view.tr909_render_mode,
        shell.app.runtime_view.tr909_render_routing
    )));

    if let Some(recovery_warning) = recovery_warning_line(shell) {
        lines.push(footer_warning_line(&recovery_warning));
    } else if shell.app.runtime_view.runtime_warnings.is_empty()
        && shell.app.jam_view.warnings.is_empty()
    {
        lines.push(footer_ok_line(
            "Warnings clear | source trust stable enough for shell work",
        ));
    } else {
        for warning in shell
            .app
            .runtime_view
            .runtime_warnings
            .iter()
            .chain(shell.app.jam_view.warnings.iter())
            .take(2)
        {
            lines.push(footer_warning_line(warning));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Line::from("Footer").style(Style::default().add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn render_primary_gesture_items(shell: &JamShellState) -> String {
    let gestures = [
        ("y", scene_jump_primary_label(shell)),
        ("g", GESTURE_FOLLOW),
        ("f", GESTURE_FILL),
        ("c", GESTURE_CAPTURE),
        ("w", GESTURE_HIT),
        ("u", GESTURE_UNDO),
    ];

    render_gesture_items(&gestures, " ")
}

pub(super) fn render_help_primary_gesture_items(shell: &JamShellState) -> String {
    let gestures = [
        ("y", scene_jump_primary_label(shell)),
        ("g", GESTURE_FOLLOW),
        ("f", GESTURE_FILL),
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
        "q quit | ? help | 1-4 screens | Tab switch | {} | space play/pause | [ ] drum | r {}",
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
