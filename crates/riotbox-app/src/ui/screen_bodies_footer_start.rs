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
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(rows[0]);

    let identity = Paragraph::new(source_identity_lines(shell))
        .block(Block::default().title("Identity").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    let timing = Paragraph::new(source_timing_lines(shell))
        .block(Block::default().title("Timing").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(identity, top[0]);
    frame.render_widget(timing, top[1]);

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

fn render_footer(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
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

fn render_help_primary_gesture_items(shell: &JamShellState) -> String {
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

fn footer_keys_line(inspect_key_label: &str, refresh_verb: &str) -> Line<'static> {
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

fn footer_primary_line(gestures: &str) -> Line<'static> {
    let mut spans = vec![
        Span::styled("Primary:", style_primary_control()),
        Span::raw(" "),
    ];
    spans.extend(spans_with_primary_gesture_keys(gestures));
    Line::from(spans)
}

fn footer_advanced_line(gestures: &str) -> Line<'static> {
    let mut spans = vec![Span::raw("Advanced: ")];
    spans.extend(spans_with_primary_gesture_keys(gestures));
    spans.push(Span::raw(" | more in ? help"));
    Line::from(spans)
}

fn footer_lane_ops_line(gestures: &str) -> Line<'static> {
    let mut spans = vec![Span::raw("Lane ops: ")];
    spans.extend(spans_with_primary_gesture_keys(gestures));
    Line::from(spans)
}

fn footer_scene_line(scene_cue: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("Scene:", style_pending_cue()),
        Span::styled(format!(" {scene_cue}"), style_pending_cue()),
    ])
}
