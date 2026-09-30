use crate::ui::first_run_capture::first_run_onramp_stage;
use crate::ui::gestures::jam_action_label;
use crate::ui::scene_commit_cues::latest_landed_command;
use crate::ui::scene_commit_cues::scene_post_commit_cue_line;
use crate::ui::scene_labels::next_scene_jump_suggestion;
use crate::ui::scene_labels::restore_scene_now_compact_label;
use crate::ui::scene_timing::landed_scene_energy_delta;
use crate::ui::shell_state::JamShellState;
use crate::ui::shell_state::ShellScreen;
use crate::ui::styles::line_with_primary_keys;
use crate::ui::styles::style_confirmation_strong;
use crate::ui::styles::style_low_emphasis;
use ratatui::text::Line;
use ratatui::text::Span;
use riotbox_core::action::GhostMode;

pub(in crate::ui) fn jam_pending_landed_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let first_pending_line = shell
        .app
        .jam_view
        .pending_actions
        .first()
        .map(|action| {
            format!(
                "next 1 {} {} @ {}",
                action.actor,
                jam_action_label(&action.command),
                action.quantization
            )
        })
        .unwrap_or_else(|| "next 1 none".into());

    let second_pending_line = shell
        .app
        .jam_view
        .pending_actions
        .get(1)
        .map(|action| {
            let mut line = format!(
                "next 2 {} {} @ {}",
                action.actor,
                jam_action_label(&action.command),
                action.quantization
            );
            let more_pending = shell.app.jam_view.pending_actions.len().saturating_sub(2);
            if more_pending > 0 {
                line.push_str(&format!(" | +{more_pending} more"));
            }
            line
        })
        .unwrap_or_else(|| "next 2 none".into());

    vec![
        Line::from(first_pending_line),
        Line::from(second_pending_line),
        latest_landed_line(shell),
        scene_post_commit_cue_line(shell)
            .unwrap_or_else(|| Line::from(format!("status {}", shell.status_message))),
    ]
}

pub(in crate::ui) fn latest_landed_line(shell: &JamShellState) -> Line<'static> {
    if let Some(action) = shell.app.jam_view.recent_actions.first() {
        let mut spans = vec![
            Span::styled("landed ", style_low_emphasis()),
            Span::styled(format!("{} ", action.actor), style_low_emphasis()),
            Span::styled(
                jam_action_label(&action.command),
                style_confirmation_strong(),
            ),
        ];

        if let Some(energy_delta) = landed_scene_energy_delta(shell, action.command.as_str()) {
            spans.push(Span::styled(" | ", style_low_emphasis()));
            spans.push(Span::styled(energy_delta, style_confirmation_strong()));
        }

        Line::from(spans)
    } else {
        Line::from(Span::styled("landed none yet", style_low_emphasis()))
    }
}

pub(in crate::ui) fn latest_landed_text(shell: &JamShellState) -> String {
    if let Some(action) = shell.app.jam_view.recent_actions.first() {
        let mut line = format!(
            "landed {} {}",
            action.actor,
            jam_action_label(&action.command)
        );
        if let Some(energy_delta) = landed_scene_energy_delta(shell, action.command.as_str()) {
            line.push_str(&format!(" | {energy_delta}"));
        }
        line
    } else {
        "landed none yet".into()
    }
}

pub(in crate::ui) fn suggested_gesture_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    if let Some(suggestion) = shell.app.runtime.current_ghost_suggestion.as_ref() {
        let ghost_action_line = if !matches!(shell.app.session.ghost_state.mode, GhostMode::Assist)
        {
            "[Enter] needs Assist  [N] reject"
        } else if suggestion.is_blocked() {
            "[Enter] blocked  [N] reject"
        } else if suggestion.suggested_action.is_none() {
            "[Enter] no action  [N] reject"
        } else {
            "[Enter] accept  [N] reject"
        };
        return vec![
            Line::from(format!("ghost: {}", suggestion.summary)),
            line_with_primary_keys(ghost_action_line),
            line_with_primary_keys("[2] log  [?] help"),
        ];
    }

    if !shell.app.jam_view.transport.is_playing {
        return vec![
            line_with_primary_keys("[Space] play"),
            line_with_primary_keys(format!("{}  [f] fill", next_scene_jump_suggestion(shell))),
            line_with_primary_keys("[c] capture"),
        ];
    }

    if !shell.app.jam_view.pending_actions.is_empty() {
        return vec![
            Line::from("let it land"),
            line_with_primary_keys("[2] log  [u] undo"),
            line_with_primary_keys("[c] capture if good"),
        ];
    }

    if show_restore_readiness_cue(shell) {
        let third_line = ghost_assist_request_line(shell)
            .unwrap_or_else(|| line_with_primary_keys("[c] capture"));
        return vec![
            line_with_primary_keys("[y] jump first"),
            line_with_primary_keys("[Y] restore waits for one landed jump"),
            third_line,
        ];
    }

    if show_restore_ready_cue(shell) {
        return vec![
            line_with_primary_keys(format!(
                "[Y] restore {}",
                restore_scene_now_compact_label(shell)
            )),
            line_with_primary_keys("[y] jump  [c] capture"),
            line_with_primary_keys("[2] trail  [u] undo"),
        ];
    }

    if shell.app.jam_view.source.feral_scorecard.readiness == "ready" {
        let third_line = ghost_assist_request_line(shell)
            .unwrap_or_else(|| line_with_primary_keys("[c] capture if it bites"));
        return vec![
            line_with_primary_keys("feral ready: [j] browse  [f] fill"),
            line_with_primary_keys("[g] follow  [a] answer"),
            third_line,
        ];
    }

    if !shell.app.jam_view.recent_actions.is_empty() {
        return vec![
            Line::from(format!("what changed: {}", latest_landed_text(shell))),
            line_with_primary_keys("what next: [c] capture  [u] undo"),
            line_with_primary_keys(format!(
                "then try: {}  [g] follow",
                next_scene_jump_suggestion(shell)
            )),
        ];
    }

    let third_line = ghost_assist_request_line(shell)
        .unwrap_or_else(|| line_with_primary_keys("[c] capture  [w] hit"));

    vec![
        line_with_primary_keys(format!("{}  [g] follow", next_scene_jump_suggestion(shell))),
        line_with_primary_keys("[a] answer  [f] fill"),
        third_line,
    ]
}

pub(in crate::ui) fn ghost_assist_request_line(shell: &JamShellState) -> Option<Line<'static>> {
    ghost_assist_request_is_useful(shell)
        .then(|| line_with_primary_keys("ghost assist: [Enter] ask"))
}

pub(in crate::ui) fn ghost_assist_request_is_useful(shell: &JamShellState) -> bool {
    shell.active_screen == ShellScreen::Jam
        && first_run_onramp_stage(shell).is_none()
        && shell.app.jam_view.transport.is_playing
        && shell.app.jam_view.pending_actions.is_empty()
        && shell.app.jam_view.recent_actions.is_empty()
        && shell
            .app
            .can_refresh_current_ghost_suggestion_from_jam_state()
}

pub(in crate::ui) fn show_restore_readiness_cue(shell: &JamShellState) -> bool {
    let recent_command_allows_readiness =
        matches!(latest_landed_command(shell), None | Some("undo.last"));

    shell.app.jam_view.transport.is_playing
        && shell.app.jam_view.pending_actions.is_empty()
        && recent_command_allows_readiness
        && shell
            .app
            .session
            .runtime_state
            .scene_state
            .restore_scene
            .is_none()
        && shell.app.session.runtime_state.scene_state.scenes.len() > 1
}

pub(in crate::ui) fn show_restore_ready_cue(shell: &JamShellState) -> bool {
    shell.app.jam_view.transport.is_playing
        && shell.app.jam_view.pending_actions.is_empty()
        && shell
            .app
            .session
            .runtime_state
            .scene_state
            .restore_scene
            .is_some()
}
