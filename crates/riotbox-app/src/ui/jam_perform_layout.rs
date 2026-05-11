fn render_overview_row(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
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
    .block(Block::default().title("Now").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    let next = Paragraph::new(next_panel_lines(shell))
        .block(Block::default().title("Next").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    let trust = trust_summary(shell);
    let trust_panel = Paragraph::new(vec![
        Line::from(format!(
            "{} ({:.2}) | warnings {}",
            trust.headline, trust.overall_confidence, trust.warning_count
        )),
        Line::from(format!(
            "timing {} | sections {}",
            trust.timing_quality, trust.section_quality
        )),
        source_timing_readiness_line(shell),
        Line::from(format!(
            "loops {} | hooks {}",
            shell.app.jam_view.source.loop_candidate_count,
            shell.app.jam_view.source.hook_candidate_count
        )),
        Line::from(primary_warning_line(shell)),
    ])
    .block(Block::default().title("Trust").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    frame.render_widget(now, columns[0]);
    frame.render_widget(next, columns[1]);
    frame.render_widget(trust_panel, columns[2]);
}

fn next_panel_lines(shell: &JamShellState) -> Vec<Line<'static>> {
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

fn style_primary_control() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

fn style_pending_cue() -> Style {
    Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
}

fn style_pending_detail() -> Style {
    Style::default().fg(Color::Yellow)
}

fn style_confirmation() -> Style {
    Style::default().fg(Color::Green)
}

fn style_confirmation_strong() -> Style {
    style_confirmation().add_modifier(Modifier::BOLD)
}

fn style_warning_label() -> Style {
    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
}

fn style_warning_detail() -> Style {
    Style::default().fg(Color::Yellow)
}

fn style_low_emphasis() -> Style {
    Style::default().fg(Color::DarkGray)
}

fn render_perform_row(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
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

fn render_first_run_onramp_row(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let lines = match first_run_onramp_stage(shell) {
        Some(FirstRunOnrampStage::Start) => vec![
            Line::from("1 [Space] start transport"),
            Line::from("2 [f] queue one first fill"),
            Line::from("3 [2] watch Log when it lands on the next bar"),
            source_timing_help_line(shell),
        ],
        Some(FirstRunOnrampStage::QueuedFirstMove) => vec![
            Line::from("Your first move is armed."),
            Line::from("Let transport cross the next bar so the fill can actually land."),
            Line::from("Then [2] confirm it in Log and decide: [c] capture it or [u] undo it."),
            source_timing_help_line(shell),
        ],
        Some(FirstRunOnrampStage::FirstResult) => vec![
            Line::from(format!("What changed: {}", latest_landed_text(shell))),
            Line::from("What next: [c] capture it or [u] undo it if it missed."),
            Line::from("Then try one more move: [y] jump or [g] follow."),
            source_timing_help_line(shell),
        ],
        None => Vec::new(),
    };

    let paragraph = Paragraph::new(lines)
        .block(Block::default().title("Start Here").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn render_action_rows(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
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

fn render_focus_row(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
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

fn render_inspect_lane_row(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
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

fn render_inspect_detail_row(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
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
