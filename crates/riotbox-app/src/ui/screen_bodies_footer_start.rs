fn render_log_body(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(9)])
        .split(area);

    let summary_columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(rows[0]);

    let history = &shell.app.session.action_log.actions;
    let committed_count = history
        .iter()
        .filter(|action| action.status == riotbox_core::action::ActionStatus::Committed)
        .count();
    let rejected_count = history
        .iter()
        .filter(|action| action.status == riotbox_core::action::ActionStatus::Rejected)
        .count();
    let undone_count = history
        .iter()
        .filter(|action| action.status == riotbox_core::action::ActionStatus::Undone)
        .count();
    let ghost_count = history
        .iter()
        .filter(|action| action.actor == riotbox_core::action::ActorType::Ghost)
        .count();

    let counts = Paragraph::new(vec![
        Line::from(format!(
            "pending {}",
            shell.app.queue.pending_actions().len()
        )),
        Line::from(format!("committed {committed_count} | ghost {ghost_count}")),
        Line::from(format!("rejected {rejected_count} | undone {undone_count}")),
        Line::from(format!(
            "scene {} | {}",
            current_scene_compact_label(shell),
            shell
                .app
                .jam_view
                .scene
                .active_scene_energy
                .as_deref()
                .unwrap_or("unknown")
        )),
        Line::from(
            scene_history_trail_line(shell)
                .unwrap_or_else(|| format!("restore {}", restore_scene_label(shell))),
        ),
    ])
    .block(Block::default().title("Counts").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    let mc202_focus = Paragraph::new(mc202_log_lines(shell))
        .block(Block::default().title("MC-202 Lane").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    let w30_focus = Paragraph::new(w30_log_lines(shell))
        .block(Block::default().title("W-30 Lane").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    let mut render_focus_lines = vec![
        Line::from(tr909_log_header_line(shell)),
        Line::from(format!(
            "render {} | accent {}",
            shell.app.runtime_view.tr909_render_mode,
            shell.app.runtime_view.tr909_render_support_accent
        )),
    ];
    if let Some(reason) = tr909_compact_reason_line(shell) {
        render_focus_lines.push(Line::from(reason));
    }
    render_focus_lines.extend([
        Line::from(format!(
            "via {} | {} / {}",
            shell.app.runtime_view.tr909_render_routing,
            shell.app.runtime_view.tr909_render_profile,
            shell.app.runtime_view.tr909_render_support_context
        )),
        Line::from(format!(
            "{} | {} | {}",
            shell.app.runtime_view.tr909_render_pattern_adoption,
            shell.app.runtime_view.tr909_render_phrase_variation,
            shell.app.runtime_view.tr909_render_alignment
        )),
        Line::from(shell.app.runtime_view.tr909_render_mix_summary.clone()),
        Line::from(shell.app.runtime_view.tr909_render_alignment.clone()),
    ]);
    let render_focus = Paragraph::new(render_focus_lines)
        .block(
            Block::default()
                .title("TR-909 Render")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });

    let warnings = log_warning_lines(shell);
    let warnings_panel = Paragraph::new(warnings)
        .block(
            Block::default()
                .title("Warnings / Restore")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(counts, summary_columns[0]);
    frame.render_widget(mc202_focus, summary_columns[1]);
    frame.render_widget(w30_focus, summary_columns[2]);
    frame.render_widget(render_focus, summary_columns[3]);
    frame.render_widget(warnings_panel, summary_columns[4]);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(rows[1]);

    let pending_panel = Paragraph::new(pending_log_lines(shell))
        .block(
            Block::default()
                .title("Queued / Pending")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });
    let committed_panel = Paragraph::new(committed_log_lines(shell))
        .block(
            Block::default()
                .title("Accepted / Committed")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });
    let rejected_panel = Paragraph::new(exception_log_lines(shell))
        .block(
            Block::default()
                .title("Rejected / Undone")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(pending_panel, columns[0]);
    frame.render_widget(committed_panel, columns[1]);
    frame.render_widget(rejected_panel, columns[2]);
}

fn render_source_body(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(9),
            Constraint::Min(8),
        ])
        .split(area);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(40),
            Constraint::Percentage(35),
        ])
        .split(rows[0]);

    let identity = Paragraph::new(source_identity_lines(shell))
        .block(Block::default().title("Identity").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    let timing = Paragraph::new(source_timing_lines(shell))
        .block(Block::default().title("Timing").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    let source_map = Paragraph::new(source_map_lines(shell))
        .block(Block::default().title("Source Map").borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    frame.render_widget(identity, top[0]);
    frame.render_widget(timing, top[1]);
    frame.render_widget(source_map, top[2]);

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(rows[1]);

    let sections = List::new(source_section_items(shell))
        .block(Block::default().title("Sections").borders(Borders::ALL));
    let candidates = Paragraph::new(source_candidate_lines(shell))
        .block(Block::default().title("Candidates").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    let provenance = Paragraph::new(source_provenance_lines(shell))
        .block(Block::default().title("Provenance").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(sections, middle[0]);
    frame.render_widget(candidates, middle[1]);
    frame.render_widget(provenance, middle[2]);

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(rows[2]);

    let warnings = Paragraph::new(source_warning_lines(shell))
        .block(
            Block::default()
                .title("Source Graph Warnings")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });
    let confidence = Paragraph::new(source_confidence_lines(shell))
        .block(
            Block::default()
                .title("Confidence Summary")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(warnings, bottom[0]);
    frame.render_widget(confidence, bottom[1]);
}

fn render_capture_body(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(12)])
        .split(area);

    let summary = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(rows[0]);

    let readiness = Paragraph::new(capture_readiness_lines(shell))
        .block(Block::default().title("Readiness").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    let latest = Paragraph::new(capture_latest_lines(shell))
        .block(
            Block::default()
                .title("Latest Capture")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });
    let do_next = Paragraph::new(capture_do_next_lines(shell))
        .block(Block::default().title("Do Next").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(readiness, summary[0]);
    frame.render_widget(latest, summary[1]);
    frame.render_widget(do_next, summary[2]);

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(32),
            Constraint::Percentage(24),
            Constraint::Percentage(20),
            Constraint::Percentage(24),
        ])
        .split(rows[1]);

    let pending = Paragraph::new(pending_capture_lines(shell))
        .block(
            Block::default()
                .title("Pending Capture Cues")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });
    let recent = List::new(recent_capture_items(shell)).block(
        Block::default()
            .title("Recent Captures")
            .borders(Borders::ALL),
    );
    let provenance = Paragraph::new(capture_provenance_lines(shell))
        .block(Block::default().title("Provenance").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    let routing = Paragraph::new(capture_routing_lines(shell))
        .block(
            Block::default()
                .title("Advanced Routing")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(pending, bottom[0]);
    frame.render_widget(recent, bottom[1]);
    frame.render_widget(provenance, bottom[2]);
    frame.render_widget(routing, bottom[3]);
}
