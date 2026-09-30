use crate::ui::first_run_capture::first_run_onramp_lines;
use crate::ui::first_run_capture::first_run_onramp_stage;
use crate::ui::first_run_capture::source_monitor_route_help_label;
use crate::ui::footer_renderer::render_help_primary_gesture_items;
use crate::ui::gestures::HELP_ADVANCED_GESTURES_A;
use crate::ui::gestures::HELP_ADVANCED_GESTURES_B;
use crate::ui::gestures::HELP_ADVANCED_GESTURES_C;
use crate::ui::gestures::HELP_ADVANCED_GESTURES_D;
use crate::ui::gestures::HELP_PRIMARY_CONFIRM_GESTURES;
use crate::ui::gestures::render_gesture_items;
use crate::ui::performer_cues::ghost_assist_request_is_useful;
use crate::ui::performer_cues::show_restore_readiness_cue;
use crate::ui::performer_cues::show_restore_ready_cue;
use crate::ui::recovery_prompt::recovery_help_lines;
use crate::ui::scene_labels::compact_scene_label;
use crate::ui::scene_labels::restore_scene_energy_direction_label;
use crate::ui::scene_labels::restore_scene_target_compact_label;
use crate::ui::scene_timing::pending_scene_transition;
use crate::ui::shell_state::JamShellState;
use crate::ui::shell_state::ShellScreen;
use crate::ui::source_trust_summary::source_timing_help_line;
use crate::ui::styles::line_with_primary_key_prefixes;
use crate::ui::styles::line_with_primary_keys;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;
use riotbox_core::action::GhostMode;
use riotbox_core::view::jam::ArrangementSceneContractReadinessView;

pub(in crate::ui) fn render_help_overlay(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let popup = centered_rect(100, 100, area);
    let mut lines = vec![
        Line::from("Jam shell keys"),
        line_with_primary_key_prefixes("q: quit | Esc / ? / h: close help"),
        line_with_primary_key_prefixes(
            "1: Jam screen | 2: Log screen | 3: Source screen | 4: Capture screen | Tab: next screen",
        ),
        line_with_primary_key_prefixes(
            "i: open inspect from Jam | press i again to return to perform",
        ),
        Line::from(""),
        Line::from("Primary gestures"),
        line_with_primary_key_prefixes("F: activate Feral Break Alpha"),
        line_with_primary_key_prefixes(format!(
            "M: monitor {} -> {} | {}",
            shell.app.session.runtime_state.source_monitor.mode,
            shell.app.session.runtime_state.source_monitor.mode.next(),
            source_monitor_route_help_label(shell),
        )),
        line_with_primary_key_prefixes(format!(
            "space: play / pause | {}",
            render_help_primary_gesture_items(shell)
        )),
        line_with_primary_key_prefixes(
            "y: request next scene (when ready) | Y: restore prior scene",
        ),
        line_with_primary_key_prefixes(format!(
            "{} | 2: action trail (optional)",
            render_gesture_items(HELP_PRIMARY_CONFIRM_GESTURES, ": ")
        )),
    ];

    if first_run_onramp_stage(shell).is_some() {
        lines.push(Line::from(""));
        lines.push(Line::from("First run"));
        lines.push(source_timing_help_line(shell));
        lines.extend(
            first_run_onramp_lines(shell)
                .into_iter()
                .map(line_with_primary_key_prefixes),
        );
        lines.push(Line::from(
            "After first loop: Recipe 16 taste/proof | Recipe 2/5 gestures/sources",
        ));
    }

    if let Some(recovery_help_lines) = recovery_help_lines(shell) {
        lines.extend(recovery_help_lines);
    }
    if let Some(scene_help_lines) = pending_scene_help_lines(shell) {
        lines.extend(scene_help_lines);
    }
    if let Some(scene_restore_help_lines) = scene_restore_help_lines(shell) {
        lines.extend(scene_restore_help_lines);
    }
    if let Some(capture_help_lines) = capture_help_lines(shell) {
        lines.extend(capture_help_lines);
    }
    if let Some(ghost_help_lines) = ghost_help_lines(shell) {
        lines.extend(ghost_help_lines);
    }
    lines.extend(arrangement_help_lines(shell));

    lines.extend([
        Line::from(""),
        Line::from("Advanced / lane gestures"),
        line_with_primary_key_prefixes(format!("r: {}", shell.launch_mode.refresh_verb())),
        line_with_primary_key_prefixes(render_gesture_items(HELP_ADVANCED_GESTURES_A, ": ")),
        line_with_primary_key_prefixes(render_gesture_items(HELP_ADVANCED_GESTURES_B, ": ")),
        line_with_primary_key_prefixes(render_gesture_items(HELP_ADVANCED_GESTURES_C, ": ")),
        line_with_primary_key_prefixes(render_gesture_items(HELP_ADVANCED_GESTURES_D, ": ")),
        line_with_primary_key_prefixes("[ / ]: drum bus | < / >: MC-202 touch | v: pin latest"),
        Line::from(""),
        Line::from(format!("Current mode: {}", shell.launch_mode.label())),
        Line::from(format!("Jam view: {}", shell.jam_mode.label())),
        Line::from(format!("Current screen: {}", shell.active_screen.label())),
        Line::from(shell.status_message.clone()),
    ]);

    let help = Paragraph::new(lines)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(Clear, popup);
    frame.render_widget(help, popup);
}

pub(in crate::ui) fn arrangement_help_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        Line::from("Jam taste / proof"),
        Line::from(format!("Taste now: {}", arrangement_taste_help_text(shell))),
        Line::from(format!("Proof now: {}", arrangement_proof_help_text(shell))),
        Line::from("Taste is confidence language, not autonomous arranger proof"),
    ]
}

pub(in crate::ui) fn arrangement_taste_help_text(shell: &JamShellState) -> &'static str {
    match shell.app.jam_view.scene.arrangement_contract.readiness {
        ArrangementSceneContractReadinessView::Ready => {
            "scene-ready; trusted grid can steer manual scene moves"
        }
        ArrangementSceneContractReadinessView::NeedsTimingConfirmation => {
            "cautious; confirm grid before trusting scene moves"
        }
        ArrangementSceneContractReadinessView::FallbackTimingOnly => "sketch; fallback timing only",
        ArrangementSceneContractReadinessView::NeedsSceneMaterial => "waiting; needs two scenes",
        ArrangementSceneContractReadinessView::NeedsTimingEvidence => "unknown; timing unavailable",
        ArrangementSceneContractReadinessView::MissingSourceGraph => "unknown; load source graph",
    }
}

pub(in crate::ui) fn arrangement_proof_help_text(shell: &JamShellState) -> &'static str {
    let contract = &shell.app.jam_view.scene.arrangement_contract;

    if contract.has_pending_scene_transition {
        "pending scene move; wait for commit plus output evidence"
    } else if contract.has_landed_movement {
        "landed movement; inspect replay/output trail before trusting audio"
    } else {
        "none yet; no landed audible move has output evidence in this run"
    }
}

pub(in crate::ui) fn pending_scene_help_lines(shell: &JamShellState) -> Option<Vec<Line<'static>>> {
    let (_kind, label, scene_id, boundary) = pending_scene_transition(shell)?;
    let scene = compact_scene_label(scene_id.as_str());

    Some(vec![
        Line::from(""),
        Line::from("Scene timing"),
        Line::from(format!("{label} {scene}: lands at {boundary}")),
        Line::from("Jam: read launch/restore, pulse, live/restore energy"),
        line_with_primary_key_prefixes("2: confirm the landed trail on Log"),
    ])
}

pub(in crate::ui) fn scene_restore_help_lines(shell: &JamShellState) -> Option<Vec<Line<'static>>> {
    if show_restore_readiness_cue(shell) {
        return Some(vec![
            Line::from(""),
            Line::from("Scene restore"),
            Line::from("Y waits for one landed jump"),
            Line::from("land one jump, then Y can restore the last scene"),
        ]);
    }

    if show_restore_ready_cue(shell) {
        let restore_target = restore_scene_target_compact_label(shell);
        let direction = restore_scene_energy_direction_label(shell)
            .map(|direction| format!(" ({direction})"))
            .unwrap_or_default();
        return Some(vec![
            Line::from(""),
            Line::from("Scene restore"),
            Line::from(format!("Y is live now for {restore_target}{direction}")),
            Line::from(format!(
                "press Y to bring {restore_target} back on the next bar"
            )),
        ]);
    }

    None
}

pub(in crate::ui) fn capture_help_lines(shell: &JamShellState) -> Option<Vec<Line<'static>>> {
    if shell.active_screen != ShellScreen::Capture {
        return None;
    }

    Some(vec![
        Line::from(""),
        Line::from("Capture path"),
        Line::from("Do Next: read capture -> promote -> hit"),
        line_with_primary_keys("src/artifact means audible; unavailable means recapture"),
        line_with_primary_keys("hear only backed W-30: [o] raw, [p] promote, [w] hit"),
        line_with_primary_key_prefixes("2: confirm promote, hit, and audition results in Log"),
    ])
}

pub(in crate::ui) fn ghost_help_lines(shell: &JamShellState) -> Option<Vec<Line<'static>>> {
    if let Some(suggestion) = shell.app.runtime.current_ghost_suggestion.as_ref() {
        let accept_line = if !matches!(shell.app.session.ghost_state.mode, GhostMode::Assist) {
            "Enter: accept only works in Assist mode"
        } else if suggestion.is_blocked() {
            "Enter: blocked by Ghost safety"
        } else if suggestion.suggested_action.is_none() {
            "Enter: no queueable Ghost action"
        } else {
            "Enter: accept and queue the Ghost move"
        };

        return Some(vec![
            Line::from(""),
            Line::from("Ghost suggestion"),
            Line::from(format!("current: {}", suggestion.summary)),
            line_with_primary_key_prefixes(accept_line),
            line_with_primary_key_prefixes("N: reject and clear the suggestion"),
        ]);
    }

    if !ghost_assist_request_is_useful(shell) {
        return None;
    }

    Some(vec![
        Line::from(""),
        Line::from("Ghost Assist"),
        line_with_primary_key_prefixes("Enter: ask Ghost for the current best move"),
        line_with_primary_key_prefixes("Enter again: queue it | N: reject it"),
    ])
}

pub(in crate::ui) fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
