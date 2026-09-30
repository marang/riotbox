use crate::ui::first_run_capture::first_run_onramp_stage;
use crate::ui::footer_renderer::render_footer;
use crate::ui::help::render_help_overlay;
use crate::ui::jam_layout::render_action_rows;
use crate::ui::jam_layout::render_first_run_onramp_row;
use crate::ui::jam_layout::render_focus_row;
use crate::ui::jam_layout::render_inspect_detail_row;
use crate::ui::jam_layout::render_inspect_lane_row;
use crate::ui::jam_layout::render_overview_row;
use crate::ui::jam_layout::render_perform_row;
use crate::ui::scene_labels::next_action_line;
use crate::ui::scene_labels::now_line;
use crate::ui::screens::render_capture_body;
use crate::ui::screens::render_log_body;
use crate::ui::screens::render_source_body;
use crate::ui::shell_labels::screen_context_label;
use crate::ui::shell_state::JamShellState;
use crate::ui::shell_state::JamViewMode;
use crate::ui::shell_state::ShellScreen;
use crate::ui::source_trust_summary::source_timing_perform_risk;
use crate::ui::source_trust_summary::trust_summary;
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;

pub fn render_jam_shell(frame: &mut Frame<'_>, shell: &JamShellState) {
    let area = frame.area();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Min(17),
            Constraint::Length(6),
        ])
        .split(area);

    render_header(frame, rows[0], shell);
    render_screen_tabs(frame, rows[1], shell);
    match shell.active_screen {
        ShellScreen::Jam => render_jam_body(frame, rows[2], shell),
        ShellScreen::Log => render_log_body(frame, rows[2], shell),
        ShellScreen::Source => render_source_body(frame, rows[2], shell),
        ShellScreen::Capture => render_capture_body(frame, rows[2], shell),
    }
    render_footer(frame, rows[3], shell);

    if shell.show_help {
        render_help_overlay(frame, area, shell);
    }
}

pub(in crate::ui) fn render_jam_body(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    if first_run_onramp_stage(shell).is_some() {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Length(6),
                Constraint::Min(9),
            ])
            .split(area);

        render_overview_row(frame, rows[0], shell);
        render_first_run_onramp_row(frame, rows[1], shell);
        render_action_rows(frame, rows[2], shell);
    } else if shell.jam_mode == JamViewMode::Perform {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Length(8),
                Constraint::Min(8),
            ])
            .split(area);

        render_overview_row(frame, rows[0], shell);
        render_perform_row(frame, rows[1], shell);
        render_focus_row(frame, rows[2], shell);
    } else {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Length(9),
                Constraint::Min(9),
            ])
            .split(area);

        render_overview_row(frame, rows[0], shell);
        render_inspect_lane_row(frame, rows[1], shell);
        render_inspect_detail_row(frame, rows[2], shell);
    }
}

#[must_use]
pub fn render_jam_shell_snapshot(shell: &JamShellState, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("create snapshot terminal");
    terminal
        .draw(|frame| render_jam_shell(frame, shell))
        .expect("draw snapshot frame");
    let buffer = terminal.backend().buffer();
    let area = buffer.area;

    let mut lines = Vec::new();
    for y in 0..area.height {
        let mut line = String::new();
        for x in 0..area.width {
            line.push_str(buffer[(x, y)].symbol());
        }
        lines.push(line.trim_end().to_string());
    }

    lines.join("\n")
}

pub(in crate::ui) fn render_header(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let source = &shell.app.jam_view.source;
    let bpm_text = source
        .bpm_estimate
        .map(|bpm| format!("{bpm:.1} BPM"))
        .unwrap_or_else(|| "unknown BPM".into());
    let trust = trust_summary(shell);
    let perform_risk = source_timing_perform_risk(shell);

    let paragraph = Paragraph::new(vec![
        Line::from("Riotbox Jam"),
        Line::from(format!(
            "Mode {} | Screen {} | Source {} | {} | source {} | perform {} | feral {}",
            shell.launch_mode.label(),
            screen_context_label(shell),
            source.source_id,
            bpm_text,
            trust.headline,
            perform_risk.label(),
            source.feral_scorecard.readiness
        )),
        Line::from(format!(
            "Profile {} | Preset {}",
            shell
                .app
                .session
                .runtime_state
                .style
                .active_profile
                .map_or("none", |profile| profile.label()),
            shell
                .app
                .session
                .runtime_state
                .style
                .active_preset
                .map_or("none", |preset| preset.label())
        )),
        Line::from(format!(
            "Now {} | Next {}",
            now_line(shell),
            next_action_line(shell)
        )),
    ])
    .block(Block::default().title("Jam").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

pub(in crate::ui) fn render_screen_tabs(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let jam_label = if shell.active_screen == ShellScreen::Jam {
        "[1 Jam]"
    } else {
        "1 Jam"
    };
    let log_label = if shell.active_screen == ShellScreen::Log {
        "[2 Log]"
    } else {
        "2 Log"
    };
    let source_label = if shell.active_screen == ShellScreen::Source {
        "[3 Source]"
    } else {
        "3 Source"
    };
    let capture_label = if shell.active_screen == ShellScreen::Capture {
        "[4 Capture]"
    } else {
        "4 Capture"
    };

    let paragraph = Paragraph::new(vec![
        Line::from(format!(
            "Screens: {jam_label} | {log_label} | {source_label} | {capture_label} | Tab switch"
        )),
        Line::from(format!(
            "Purpose: {}",
            match shell.active_screen {
                ShellScreen::Jam => {
                    if shell.jam_mode == JamViewMode::Perform {
                        "instrument surface for immediate control and pending musical change"
                    } else {
                        "read-only inspect surface for lane detail, source structure, and diagnostics"
                    }
                }
                ShellScreen::Log => {
                    "trust surface for queued, committed, rejected, and undone actions"
                }
                ShellScreen::Source => {
                    "analysis structure surface for sections, candidates, and warnings"
                }
                ShellScreen::Capture => {
                    "capture surface for readiness, recent takes, and provenance"
                }
            }
        )),
    ])
    .block(Block::default().title("Navigation").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}
