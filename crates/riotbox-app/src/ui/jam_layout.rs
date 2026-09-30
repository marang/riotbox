use crate::ui::export_inspect::material_inspect_lines;
use crate::ui::first_run_capture::capture_lines;
use crate::ui::first_run_capture::first_run_onramp_compact_lines;
use crate::ui::first_run_capture::first_run_onramp_lines;
use crate::ui::first_run_capture::source_monitor_mode_compact_label;
use crate::ui::first_run_capture::source_monitor_route_compact_label;
use crate::ui::lane_diagnostics::mc202_log_lines;
use crate::ui::lane_diagnostics::w30_log_lines;
use crate::ui::lane_perform::mc202_perform_lines;
use crate::ui::lane_perform::tr909_inspect_lines;
use crate::ui::lane_perform::tr909_perform_lines;
use crate::ui::lane_perform::w30_perform_lines;
use crate::ui::perform_risk_cue_contract::PERFORM_RISK_BAR_LIVE_CUE;
use crate::ui::performer_cues::jam_pending_landed_lines;
use crate::ui::performer_cues::latest_landed_line;
use crate::ui::performer_cues::suggested_gesture_lines;
use crate::ui::scene_labels::next_action_line;
use crate::ui::scene_labels::next_scene_target_compact_label;
use crate::ui::scene_labels::scene_restore_contrast_line;
use crate::ui::scene_timing::queued_timing_rail_line;
use crate::ui::scene_timing::scene_pending_line;
use crate::ui::shell_labels::transport_label;
use crate::ui::shell_state::JamShellState;
use crate::ui::source_details::source_inspect_lines;
use crate::ui::source_trust_summary::SourceTimingPerformRisk;
use crate::ui::source_trust_summary::arrangement_taste_line;
use crate::ui::source_trust_summary::source_timing_clock_compact;
use crate::ui::source_trust_summary::source_timing_perform_risk;
use crate::ui::source_trust_summary::source_timing_performance_rail_line;
use crate::ui::source_trust_summary::source_timing_readiness_line;
use crate::ui::source_trust_summary::trust_summary;
use crate::ui::warnings::jam_diagnostic_lines;
use crate::ui::warnings::jam_warning_lines;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;

pub(in crate::ui) fn render_overview_row(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(area);

    let now = Paragraph::new(vec![
        Line::from(format!(
            "{} @ {:.1} | {}",
            transport_label(shell),
            shell.app.jam_view.transport.position_beats,
            source_timing_clock_compact(shell)
        )),
        source_timing_performance_rail_line(shell),
        Line::from(format!(
            "scene {} | energy {}",
            shell
                .app
                .jam_view
                .scene
                .active_scene
                .as_deref()
                .unwrap_or("none"),
            shell
                .app
                .jam_view
                .scene
                .active_scene_energy
                .as_deref()
                .unwrap_or("unknown")
        )),
        Line::from(format!(
            "source {} | next scene {}",
            shell.app.jam_view.source.source_id,
            next_scene_target_compact_label(shell)
        )),
        Line::from(scene_restore_contrast_line(shell)),
    ])
    .block(
        Block::default()
            .title(format!("Now | {}", source_monitor_perform_compact(shell)))
            .borders(Borders::ALL),
    )
    .wrap(Wrap { trim: true });

    let next = Paragraph::new(next_panel_lines(shell))
        .block(Block::default().title("Next").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    let trust = trust_summary(shell);
    let perform_risk = source_timing_perform_risk(shell);
    let perform_reason = trust
        .source_timing_warning
        .as_deref()
        .unwrap_or(shell.app.jam_view.source.timing.cue.as_str());
    let compact_risk_action = match perform_risk {
        SourceTimingPerformRisk::Trusted => "play grid",
        SourceTimingPerformRisk::Degraded | SourceTimingPerformRisk::Unavailable => {
            PERFORM_RISK_BAR_LIVE_CUE
        }
    };
    let trust_panel = Paragraph::new(vec![
        Line::from(format!("{} | {compact_risk_action}", perform_risk.label())),
        Line::from(format!("why {}", perform_reason.replace('_', " "))),
        Line::from(format!("risk {compact_risk_action}")),
        source_timing_readiness_line(shell),
        arrangement_taste_line(shell),
    ])
    .block(Block::default().title("Trust").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    frame.render_widget(now, columns[0]);
    frame.render_widget(next, columns[1]);
    frame.render_widget(trust_panel, columns[2]);
}

pub(in crate::ui) fn next_panel_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(next_action_line(shell)),
        scene_pending_line(shell),
    ];
    if let Some(timing_rail) = queued_timing_rail_line(shell) {
        lines.push(timing_rail);
        lines.push(latest_landed_line(shell));
    } else {
        lines.push(latest_landed_line(shell));
        lines.push(Line::from(format!("status {}", shell.status_message)));
    }
    lines
}

pub(in crate::ui) fn render_perform_row(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    let mc202 = Paragraph::new(mc202_perform_lines(shell))
        .block(Block::default().title("MC-202").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    let w30 = Paragraph::new(w30_perform_lines(shell))
        .block(Block::default().title("W-30").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    let tr909 = Paragraph::new(tr909_perform_lines(shell))
        .block(Block::default().title("TR-909").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(mc202, columns[0]);
    frame.render_widget(w30, columns[1]);
    frame.render_widget(tr909, columns[2]);
}

pub(in crate::ui) fn render_first_run_onramp_row(
    frame: &mut Frame<'_>,
    area: Rect,
    shell: &JamShellState,
) {
    let guidance = if area.width < 100 || area.height < 6 {
        first_run_onramp_compact_lines(shell)
    } else {
        first_run_onramp_lines(shell)
    };
    let lines = guidance.into_iter().map(Line::from).collect::<Vec<_>>();

    let paragraph = Paragraph::new(lines)
        .block(Block::default().title("Start Here").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

pub(in crate::ui) fn source_monitor_perform_compact(shell: &JamShellState) -> String {
    let mode = shell.app.session.runtime_state.source_monitor.mode;
    format!(
        "M {}>{}/{}",
        source_monitor_mode_compact_label(mode),
        source_monitor_mode_compact_label(mode.next()),
        source_monitor_route_compact_label(shell)
    )
}

pub(in crate::ui) fn render_action_rows(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(area);

    let pending_items = if shell.app.jam_view.pending_actions.is_empty() {
        vec![ListItem::new("no pending actions")]
    } else {
        shell
            .app
            .jam_view
            .pending_actions
            .iter()
            .map(|action| {
                ListItem::new(format!(
                    "{} {} {} @ {}",
                    action.id, action.actor, action.command, action.quantization
                ))
            })
            .collect()
    };

    let recent_items = if shell.app.jam_view.recent_actions.is_empty() {
        vec![ListItem::new("no committed actions yet")]
    } else {
        shell
            .app
            .jam_view
            .recent_actions
            .iter()
            .map(|action| {
                ListItem::new(format!(
                    "{} {} {} [{}]",
                    action.id, action.actor, action.command, action.status
                ))
            })
            .collect()
    };

    let pending =
        List::new(pending_items).block(Block::default().title("Pending").borders(Borders::ALL));
    let recent =
        List::new(recent_items).block(Block::default().title("Recent").borders(Borders::ALL));
    let capture = Paragraph::new(capture_lines(shell))
        .block(Block::default().title("Capture").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(pending, columns[0]);
    frame.render_widget(recent, columns[1]);
    frame.render_widget(capture, columns[2]);
}

pub(in crate::ui) fn render_focus_row(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(area);

    let pending = Paragraph::new(jam_pending_landed_lines(shell))
        .block(
            Block::default()
                .title("Pending / landed")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });
    let gestures = Paragraph::new(suggested_gesture_lines(shell))
        .block(
            Block::default()
                .title("Suggested gestures")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });
    let warnings = Paragraph::new(jam_warning_lines(shell))
        .block(
            Block::default()
                .title("Warnings / trust")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(pending, columns[0]);
    frame.render_widget(gestures, columns[1]);
    frame.render_widget(warnings, columns[2]);
}

pub(in crate::ui) fn render_inspect_lane_row(
    frame: &mut Frame<'_>,
    area: Rect,
    shell: &JamShellState,
) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    let mc202 = Paragraph::new(mc202_log_lines(shell))
        .block(
            Block::default()
                .title("MC-202 detail")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });
    let w30 = Paragraph::new(w30_log_lines(shell))
        .block(Block::default().title("W-30 detail").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    let tr909 = Paragraph::new(tr909_inspect_lines(shell))
        .block(
            Block::default()
                .title("TR-909 detail")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(mc202, columns[0]);
    frame.render_widget(w30, columns[1]);
    frame.render_widget(tr909, columns[2]);
}

pub(in crate::ui) fn render_inspect_detail_row(
    frame: &mut Frame<'_>,
    area: Rect,
    shell: &JamShellState,
) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(area);

    let source = Paragraph::new(source_inspect_lines(shell))
        .block(
            Block::default()
                .title("Source structure")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });
    let material = Paragraph::new(material_inspect_lines(shell))
        .block(
            Block::default()
                .title("Material flow")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });
    let diagnostics = Paragraph::new(jam_diagnostic_lines(shell))
        .block(Block::default().title("Diagnostics").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(source, columns[0]);
    frame.render_widget(material, columns[1]);
    frame.render_widget(diagnostics, columns[2]);
}
