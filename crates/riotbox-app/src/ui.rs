use crossterm::event::KeyCode;
use ratatui::{
    Frame, Terminal,
    backend::TestBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use riotbox_core::source_graph::{
    DecodeProfile, EnergyClass, QualityClass, Section, SectionLabelHint,
};

use crate::jam_app::JamAppState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellScreen {
    Jam,
    Log,
    Source,
}

impl ShellScreen {
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Jam => "jam",
            Self::Log => "log",
            Self::Source => "source",
        }
    }

    #[must_use]
    pub const fn next(&self) -> Self {
        match self {
            Self::Jam => Self::Log,
            Self::Log => Self::Source,
            Self::Source => Self::Jam,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShellLaunchMode {
    Load,
    Ingest,
}

impl ShellLaunchMode {
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Load => "load",
            Self::Ingest => "ingest",
        }
    }

    #[must_use]
    pub const fn refresh_verb(&self) -> &'static str {
        match self {
            Self::Load => "reload session",
            Self::Ingest => "re-ingest source",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellKeyOutcome {
    Continue,
    RequestRefresh,
    ToggleTransport,
    QueueSceneMutation,
    QueueTr909Fill,
    QueueTr909Reinforce,
    QueueCaptureBar,
    UndoLast,
    Quit,
}

#[derive(Clone, Debug)]
pub struct JamShellState {
    pub app: JamAppState,
    pub launch_mode: ShellLaunchMode,
    pub active_screen: ShellScreen,
    pub show_help: bool,
    pub status_message: String,
}

impl JamShellState {
    #[must_use]
    pub fn new(app: JamAppState, launch_mode: ShellLaunchMode) -> Self {
        let status_message = match launch_mode {
            ShellLaunchMode::Load => "loaded session from disk".into(),
            ShellLaunchMode::Ingest => "ingested source into Jam shell".into(),
        };

        Self {
            app,
            launch_mode,
            active_screen: ShellScreen::Jam,
            show_help: false,
            status_message,
        }
    }

    pub fn handle_key_code(&mut self, code: KeyCode) -> ShellKeyOutcome {
        match code {
            KeyCode::Esc | KeyCode::Char('q') => ShellKeyOutcome::Quit,
            KeyCode::Tab | KeyCode::BackTab => {
                self.active_screen = self.active_screen.next();
                self.status_message = format!("switched to {} screen", self.active_screen.label());
                ShellKeyOutcome::Continue
            }
            KeyCode::Char('1') => {
                self.active_screen = ShellScreen::Jam;
                self.status_message = "switched to jam screen".into();
                ShellKeyOutcome::Continue
            }
            KeyCode::Char('2') => {
                self.active_screen = ShellScreen::Log;
                self.status_message = "switched to log screen".into();
                ShellKeyOutcome::Continue
            }
            KeyCode::Char('3') => {
                self.active_screen = ShellScreen::Source;
                self.status_message = "switched to source screen".into();
                ShellKeyOutcome::Continue
            }
            KeyCode::Char(' ') => {
                self.status_message = "transport toggle requested".into();
                ShellKeyOutcome::ToggleTransport
            }
            KeyCode::Char('?') | KeyCode::Char('h') => {
                self.show_help = !self.show_help;
                self.status_message = if self.show_help {
                    "help overlay opened".into()
                } else {
                    "help overlay closed".into()
                };
                ShellKeyOutcome::Continue
            }
            KeyCode::Char('r') => {
                self.status_message = format!("{} requested", self.launch_mode.refresh_verb());
                ShellKeyOutcome::RequestRefresh
            }
            KeyCode::Char('m') => {
                self.status_message = "queue scene mutation on next bar".into();
                ShellKeyOutcome::QueueSceneMutation
            }
            KeyCode::Char('f') => {
                self.status_message = "queue TR-909 fill on next bar".into();
                ShellKeyOutcome::QueueTr909Fill
            }
            KeyCode::Char('d') => {
                self.status_message = "queue TR-909 reinforcement on next phrase".into();
                ShellKeyOutcome::QueueTr909Reinforce
            }
            KeyCode::Char('c') => {
                self.status_message = "queue capture on next phrase".into();
                ShellKeyOutcome::QueueCaptureBar
            }
            KeyCode::Char('u') => {
                self.status_message = "undo most recent action requested".into();
                ShellKeyOutcome::UndoLast
            }
            _ => ShellKeyOutcome::Continue,
        }
    }

    pub fn replace_app_state(&mut self, app: JamAppState) {
        self.app = app;
        self.status_message = match self.launch_mode {
            ShellLaunchMode::Load => "reloaded session from disk".into(),
            ShellLaunchMode::Ingest => "re-ingested source into Jam shell".into(),
        };
    }

    pub fn set_error_status(&mut self, message: impl Into<String>) {
        self.status_message = message.into();
    }
}

pub fn render_jam_shell(frame: &mut Frame<'_>, shell: &JamShellState) {
    let area = frame.area();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Min(17),
            Constraint::Length(5),
        ])
        .split(area);

    render_header(frame, rows[0], shell);
    render_screen_tabs(frame, rows[1], shell);
    match shell.active_screen {
        ShellScreen::Jam => render_jam_body(frame, rows[2], shell),
        ShellScreen::Log => render_log_body(frame, rows[2], shell),
        ShellScreen::Source => render_source_body(frame, rows[2], shell),
    }
    render_footer(frame, rows[3], shell);

    if shell.show_help {
        render_help_overlay(frame, area, shell);
    }
}

fn render_jam_body(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(9),
            Constraint::Min(7),
        ])
        .split(area);

    render_overview_row(frame, rows[0], shell);
    render_source_row(frame, rows[1], shell);
    render_action_rows(frame, rows[2], shell);
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

fn render_header(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let source = &shell.app.jam_view.source;
    let bpm_text = source
        .bpm_estimate
        .map(|bpm| format!("{bpm:.1} BPM"))
        .unwrap_or_else(|| "unknown BPM".into());
    let trust = trust_summary(shell);

    let paragraph = Paragraph::new(vec![
        Line::from("Riotbox Jam"),
        Line::from(format!(
            "Mode {} | Screen {} | Source {} | {} | trust {}",
            shell.launch_mode.label(),
            shell.active_screen.label(),
            source.source_id,
            bpm_text,
            trust.headline
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

fn render_screen_tabs(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
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

    let paragraph = Paragraph::new(vec![
        Line::from(format!(
            "Screens: {jam_label} | {log_label} | {source_label} | Tab switch"
        )),
        Line::from(format!(
            "Purpose: {}",
            match shell.active_screen {
                ShellScreen::Jam => {
                    "instrument surface for immediate control and pending musical change"
                }
                ShellScreen::Log => {
                    "trust surface for queued, committed, rejected, and undone actions"
                }
                ShellScreen::Source => {
                    "analysis structure surface for sections, candidates, and warnings"
                }
            }
        )),
    ])
    .block(Block::default().title("Navigation").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn render_overview_row(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let now = Paragraph::new(vec![
        Line::from(format!("Transport: {}", transport_label(shell))),
        Line::from(format!(
            "Beat {:.1} | scene {}",
            shell.app.jam_view.transport.position_beats,
            shell
                .app
                .jam_view
                .scene
                .active_scene
                .as_deref()
                .unwrap_or("none")
        )),
        Line::from(format!("Ghost: {}", ghost_label(shell))),
    ])
    .block(Block::default().title("Now").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    let next = Paragraph::new(vec![
        Line::from(primary_pending_line(shell)),
        Line::from(primary_recent_line(shell)),
        Line::from(format!("status {}", shell.status_message)),
    ])
    .block(Block::default().title("Next").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    let trust = trust_summary(shell);
    let trust_panel = Paragraph::new(vec![
        Line::from(format!(
            "overall {:.2} | warnings {}",
            trust.overall_confidence, trust.warning_count
        )),
        Line::from(format!(
            "timing {} | sections {}",
            trust.timing_quality, trust.section_quality
        )),
        Line::from(format!(
            "loops {} | hooks {}",
            shell.app.jam_view.source.loop_candidate_count,
            shell.app.jam_view.source.hook_candidate_count
        )),
    ])
    .block(Block::default().title("Trust").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    let lanes = Paragraph::new(vec![
        Line::from(format!(
            "MC-202: {}",
            shell
                .app
                .jam_view
                .lanes
                .mc202_role
                .as_deref()
                .unwrap_or("unset")
        )),
        Line::from(format!(
            "W-30: {}",
            shell
                .app
                .jam_view
                .lanes
                .w30_active_bank
                .as_deref()
                .unwrap_or("unset")
        )),
        Line::from(format!(
            "TR-909 slam: {}",
            if shell.app.jam_view.lanes.tr909_slam_enabled {
                "on"
            } else {
                "off"
            }
        )),
        Line::from(format!(
            "fill armed: {} | last bar {}",
            if shell.app.jam_view.lanes.tr909_fill_armed_next_bar {
                "yes"
            } else {
                "no"
            },
            shell
                .app
                .jam_view
                .lanes
                .tr909_last_fill_bar
                .map(|bar| bar.to_string())
                .unwrap_or_else(|| "-".into())
        )),
        Line::from(format!(
            "909 mode: {}",
            shell
                .app
                .jam_view
                .lanes
                .tr909_reinforcement_mode
                .as_deref()
                .unwrap_or("unset")
        )),
    ])
    .block(Block::default().title("Lanes").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    frame.render_widget(now, columns[0]);
    frame.render_widget(next, columns[1]);
    frame.render_widget(trust_panel, columns[2]);
    frame.render_widget(lanes, columns[3]);
}

fn render_source_row(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(46),
            Constraint::Percentage(34),
            Constraint::Percentage(20),
        ])
        .split(area);

    let source = Paragraph::new(source_detail_lines(shell))
        .block(Block::default().title("Source").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    let sections = List::new(section_items(shell))
        .block(Block::default().title("Sections").borders(Borders::ALL));
    let macros = Paragraph::new(macro_lines(shell))
        .block(Block::default().title("Macros").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(source, columns[0]);
    frame.render_widget(sections, columns[1]);
    frame.render_widget(macros, columns[2]);
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

fn render_log_body(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(10)])
        .split(area);

    let summary_columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
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
    ])
    .block(Block::default().title("Counts").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    let boundary = shell
        .app
        .runtime
        .last_commit_boundary
        .as_ref()
        .map(|boundary| {
            format!(
                "{:?} @ beat {} bar {} phrase {}",
                boundary.kind, boundary.beat_index, boundary.bar_index, boundary.phrase_index
            )
        })
        .unwrap_or_else(|| "no commit boundary yet".into());
    let current_scene = shell
        .app
        .runtime
        .transport
        .current_scene
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| "none".into());
    let focus = Paragraph::new(vec![
        Line::from(format!("scene {current_scene}")),
        Line::from(format!(
            "transport beat {:.1} | playing {}",
            shell.app.runtime.transport.position_beats, shell.app.runtime.transport.is_playing
        )),
        Line::from(format!("last boundary {boundary}")),
    ])
    .block(Block::default().title("Focus").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    let warnings = log_warning_lines(shell);
    let warnings_panel = Paragraph::new(warnings)
        .block(Block::default().title("Warnings").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(counts, summary_columns[0]);
    frame.render_widget(focus, summary_columns[1]);
    frame.render_widget(warnings_panel, summary_columns[2]);

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
            Constraint::Length(6),
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

fn render_footer(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let mut lines = Vec::new();
    lines.push(Line::from(format!(
        "Keys: q quit | ? help | 1 jam | 2 log | 3 source | Tab switch | space play/pause | r {}",
        shell.launch_mode.refresh_verb()
    )));
    lines.push(Line::from(
        "Actions: m mutate scene | f 909 fill | d 909 reinforce | c capture phrase | u undo",
    ));
    lines.push(Line::from(format!(
        "Status: {} | audio {} | sidecar {}",
        shell.status_message,
        shell.app.runtime_view.audio_status,
        shell.app.runtime_view.sidecar_status
    )));

    if shell.app.runtime_view.runtime_warnings.is_empty() && shell.app.jam_view.warnings.is_empty()
    {
        lines.push(Line::from(
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
            lines.push(Line::from(format!("Warning: {warning}")));
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

fn render_help_overlay(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let popup = centered_rect(60, 45, area);
    let help = Paragraph::new(vec![
        Line::from("Jam shell keys"),
        Line::from("q or Esc: quit"),
        Line::from("? or h: toggle help"),
        Line::from("1: Jam screen | 2: Log screen | 3: Source screen | Tab: next screen"),
        Line::from("space: play / pause transport"),
        Line::from(format!("r: {}", shell.launch_mode.refresh_verb())),
        Line::from("m: queue scene mutation on next bar"),
        Line::from("f: queue TR-909 fill on next bar"),
        Line::from("d: queue TR-909 reinforcement on next phrase"),
        Line::from("c: queue phrase capture on next phrase"),
        Line::from("u: undo most recent undoable action"),
        Line::from(""),
        Line::from(format!("Current mode: {}", shell.launch_mode.label())),
        Line::from(format!("Current screen: {}", shell.active_screen.label())),
        Line::from(shell.status_message.clone()),
    ])
    .block(Block::default().title("Help").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    frame.render_widget(Clear, popup);
    frame.render_widget(help, popup);
}

fn source_detail_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let source = &shell.app.jam_view.source;

    match shell.app.source_graph.as_ref() {
        Some(graph) => vec![
            Line::from(format!(
                "{} | {:.2}s | {} Hz | {} ch | {}",
                graph.source.path,
                graph.source.duration_seconds,
                graph.source.sample_rate,
                graph.source.channel_count,
                decode_profile_label(&graph.source.decode_profile)
            )),
            Line::from(format!(
                "tempo confidence {:.2} | timing {} | sections {}",
                source.bpm_confidence,
                quality_label(&graph.analysis_summary.timing_quality),
                quality_label(&graph.analysis_summary.section_quality)
            )),
            Line::from(format!(
                "sections {} | loops {} | hooks {} | warnings {}",
                graph.sections.len(),
                source.loop_candidate_count,
                source.hook_candidate_count,
                graph.analysis_summary.warnings.len()
            )),
            Line::from(format!(
                "overall confidence {:.2} | break potential {}",
                graph.analysis_summary.overall_confidence,
                quality_label(&graph.analysis_summary.break_rebuild_potential)
            )),
        ],
        None => vec![Line::from("No source graph loaded")],
    }
}

fn section_items(shell: &JamShellState) -> Vec<ListItem<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) if !graph.sections.is_empty() => graph
            .sections
            .iter()
            .take(4)
            .map(|section| {
                ListItem::new(format!(
                    "{} | bars {}-{} | {:.2}s-{:.2}s | {} | conf {:.2}",
                    section_label(section),
                    section.bar_start,
                    section.bar_end,
                    section.start_seconds,
                    section.end_seconds,
                    energy_label(section),
                    section.confidence
                ))
            })
            .collect(),
        Some(_) => vec![ListItem::new("no sections available")],
        None => vec![ListItem::new("no source graph loaded")],
    }
}

fn macro_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let macros = &shell.app.jam_view.macros;
    vec![
        Line::from(format!("retain {:.2}", macros.source_retain)),
        Line::from(format!("chaos {:.2}", macros.chaos)),
        Line::from(format!("mc202 {:.2}", macros.mc202_touch)),
        Line::from(format!(
            "w30 {:.2} | tr909 {:.2}",
            macros.w30_grit, macros.tr909_slam
        )),
    ]
}

fn section_label(section: &Section) -> &'static str {
    match section.label_hint {
        SectionLabelHint::Intro => "intro",
        SectionLabelHint::Build => "build",
        SectionLabelHint::Drop => "drop",
        SectionLabelHint::Break => "break",
        SectionLabelHint::Verse => "verse",
        SectionLabelHint::Chorus => "chorus",
        SectionLabelHint::Bridge => "bridge",
        SectionLabelHint::Outro => "outro",
        SectionLabelHint::Unknown => "unknown",
    }
}

fn decode_profile_label(profile: &DecodeProfile) -> String {
    match profile {
        DecodeProfile::Native => "native".into(),
        DecodeProfile::NormalizedStereo => "normalized_stereo".into(),
        DecodeProfile::NormalizedMono => "normalized_mono".into(),
        DecodeProfile::Custom(value) => value.clone(),
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
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

fn transport_label(shell: &JamShellState) -> &'static str {
    if shell.app.jam_view.transport.is_playing {
        "playing"
    } else {
        "idle"
    }
}

fn ghost_label(shell: &JamShellState) -> String {
    format!(
        "{} ({})",
        shell.app.jam_view.ghost.mode,
        if shell.app.jam_view.ghost.is_blocked {
            "blocked"
        } else {
            "clear"
        }
    )
}

fn now_line(shell: &JamShellState) -> String {
    let scene = shell
        .app
        .jam_view
        .scene
        .active_scene
        .as_deref()
        .unwrap_or("no scene");
    format!(
        "{} at beat {:.1} in {}",
        transport_label(shell),
        shell.app.jam_view.transport.position_beats,
        scene
    )
}

fn next_action_line(shell: &JamShellState) -> String {
    if let Some(action) = shell.app.jam_view.pending_actions.first() {
        format!(
            "{} {} @ {}",
            action.actor, action.command, action.quantization
        )
    } else {
        "no pending action queued".into()
    }
}

fn primary_pending_line(shell: &JamShellState) -> String {
    if let Some(action) = shell.app.jam_view.pending_actions.first() {
        format!(
            "queued {} {} @ {}",
            action.actor, action.command, action.quantization
        )
    } else {
        "queued no pending action".into()
    }
}

fn primary_recent_line(shell: &JamShellState) -> String {
    if let Some(action) = shell.app.jam_view.recent_actions.first() {
        format!(
            "recent {} {} [{}]",
            action.actor, action.command, action.status
        )
    } else {
        "recent no committed action yet".into()
    }
}

fn capture_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let capture = &shell.app.jam_view.capture;
    vec![
        Line::from(format!("captures {}", capture.capture_count)),
        Line::from(format!(
            "last {}",
            capture.last_capture_id.as_deref().unwrap_or("none")
        )),
        Line::from(format!(
            "target {}",
            capture
                .last_capture_target
                .as_deref()
                .unwrap_or("unassigned")
        )),
        Line::from(format!("origins {}", capture.last_capture_origin_count)),
        Line::from(
            capture
                .last_capture_notes
                .clone()
                .unwrap_or_else(|| "no capture note yet".into()),
        ),
    ]
}

fn pending_log_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let pending = shell.app.queue.pending_actions();
    if pending.is_empty() {
        return vec![Line::from("no queued or pending actions")];
    }

    let mut lines = Vec::new();
    for action in pending.into_iter().take(4) {
        lines.push(Line::from(format!(
            "{} {} {}",
            action.id, action.actor, action.command
        )));
        lines.push(Line::from(format!(
            "status {} | when {} | target {}",
            format!("{:?}", action.status).to_lowercase(),
            action.quantization,
            action_target_label(&action.target)
        )));
        lines.push(Line::from(format!(
            "requested {}{}",
            action.requested_at,
            action
                .explanation
                .as_ref()
                .map(|explanation| format!(" | {explanation}"))
                .unwrap_or_default()
        )));
        lines.push(Line::from(""));
    }

    lines
}

fn committed_log_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let committed: Vec<_> = shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .filter(|action| action.status == riotbox_core::action::ActionStatus::Committed)
        .take(4)
        .collect();

    if committed.is_empty() {
        return vec![Line::from("no committed actions yet")];
    }

    action_entry_lines(committed)
}

fn exception_log_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let exceptions: Vec<_> = shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .filter(|action| {
            matches!(
                action.status,
                riotbox_core::action::ActionStatus::Rejected
                    | riotbox_core::action::ActionStatus::Undone
                    | riotbox_core::action::ActionStatus::Failed
            )
        })
        .take(4)
        .collect();

    if exceptions.is_empty() {
        return vec![Line::from("no rejected, failed, or undone actions")];
    }

    action_entry_lines(exceptions)
}

fn action_entry_lines(actions: Vec<&riotbox_core::action::Action>) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for action in actions {
        lines.push(Line::from(format!(
            "{} {} {}",
            action.id, action.actor, action.command
        )));
        lines.push(Line::from(format!(
            "status {} | when {} | target {}",
            format!("{:?}", action.status).to_lowercase(),
            action.quantization,
            action_target_label(&action.target)
        )));
        lines.push(Line::from(format!(
            "requested {} | committed {}",
            action.requested_at,
            action
                .committed_at
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".into())
        )));
        if let Some(result) = &action.result {
            lines.push(Line::from(format!("result {}", result.summary)));
        } else if let Some(explanation) = &action.explanation {
            lines.push(Line::from(format!("note {explanation}")));
        }
        lines.push(Line::from(""));
    }

    lines
}

fn action_target_label(target: &riotbox_core::action::ActionTarget) -> String {
    let Some(scope) = &target.scope else {
        return "unset".into();
    };

    let detail = if let Some(scene_id) = &target.scene_id {
        scene_id.to_string()
    } else if let Some(bank_id) = &target.bank_id {
        match &target.pad_id {
            Some(pad_id) => format!("{bank_id}/{pad_id}"),
            None => bank_id.to_string(),
        }
    } else if let Some(loop_id) = &target.loop_id {
        loop_id.to_string()
    } else if let Some(object_id) = &target.object_id {
        object_id.clone()
    } else {
        String::new()
    };

    let scope = format!("{scope:?}").to_lowercase();
    if detail.is_empty() {
        scope
    } else {
        format!("{scope}:{detail}")
    }
}

fn log_warning_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let warnings: Vec<_> = shell
        .app
        .runtime_view
        .runtime_warnings
        .iter()
        .chain(shell.app.jam_view.warnings.iter())
        .take(3)
        .cloned()
        .collect();
    if warnings.is_empty() {
        return vec![Line::from("no active runtime or trust warnings")];
    }

    warnings
        .into_iter()
        .map(|warning| Line::from(format!("warning {warning}")))
        .collect()
}

fn source_identity_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) => vec![
            Line::from(format!("source {}", graph.source.source_id)),
            Line::from(graph.source.path.clone()),
            Line::from(format!(
                "{:.2}s | {} Hz | {} ch | {}",
                graph.source.duration_seconds,
                graph.source.sample_rate,
                graph.source.channel_count,
                decode_profile_label(&graph.source.decode_profile)
            )),
            Line::from(format!("hash {}", graph.source.content_hash)),
        ],
        None => vec![Line::from("no source graph loaded")],
    }
}

fn source_timing_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) => vec![
            Line::from(format!(
                "tempo {} | conf {:.2}",
                graph
                    .timing
                    .bpm_estimate
                    .map(|bpm| format!("{bpm:.1} BPM"))
                    .unwrap_or_else(|| "unknown".into()),
                graph.timing.bpm_confidence
            )),
            Line::from(format!(
                "meter {}",
                graph
                    .timing
                    .meter_hint
                    .as_ref()
                    .map(|meter| format!("{}/{}", meter.beats_per_bar, meter.beat_unit))
                    .unwrap_or_else(|| "unknown".into())
            )),
            Line::from(format!(
                "beats {} | bars {} | phrases {}",
                graph.timing.beat_grid.len(),
                graph.timing.bar_grid.len(),
                graph.timing.phrase_grid.len()
            )),
            Line::from(format!(
                "timing {} | sections {}",
                quality_label(&graph.analysis_summary.timing_quality),
                quality_label(&graph.analysis_summary.section_quality)
            )),
        ],
        None => vec![Line::from("no timing information available")],
    }
}

fn source_section_items(shell: &JamShellState) -> Vec<ListItem<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) if !graph.sections.is_empty() => graph
            .sections
            .iter()
            .take(6)
            .map(|section| {
                ListItem::new(format!(
                    "{} | bars {}-{} | {:.2}s-{:.2}s | {} | conf {:.2}",
                    section_label(section),
                    section.bar_start,
                    section.bar_end,
                    section.start_seconds,
                    section.end_seconds,
                    energy_label(section),
                    section.confidence
                ))
            })
            .collect(),
        Some(_) => vec![ListItem::new("no sections available")],
        None => vec![ListItem::new("no source graph loaded")],
    }
}

fn source_candidate_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) => {
            let best_loop = graph
                .candidates
                .iter()
                .filter(|candidate| {
                    candidate.candidate_type
                        == riotbox_core::source_graph::CandidateType::LoopCandidate
                })
                .max_by(|left, right| left.score.total_cmp(&right.score));
            let best_hook = graph
                .candidates
                .iter()
                .filter(|candidate| {
                    candidate.candidate_type
                        == riotbox_core::source_graph::CandidateType::HookCandidate
                })
                .max_by(|left, right| left.score.total_cmp(&right.score));

            vec![
                Line::from(format!(
                    "loops {} | hooks {}",
                    graph.loop_candidate_count(),
                    graph.hook_candidate_count()
                )),
                Line::from(format!(
                    "best loop {}",
                    best_loop
                        .map(|candidate| format!(
                            "{:.2} ({:.2})",
                            candidate.score, candidate.confidence
                        ))
                        .unwrap_or_else(|| "none".into())
                )),
                Line::from(format!(
                    "best hook {}",
                    best_hook
                        .map(|candidate| format!(
                            "{:.2} ({:.2})",
                            candidate.score, candidate.confidence
                        ))
                        .unwrap_or_else(|| "none".into())
                )),
                Line::from(format!("assets {}", graph.assets.len())),
            ]
        }
        None => vec![Line::from("no candidate information available")],
    }
}

fn source_provenance_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) => vec![
            Line::from(format!("sidecar {}", graph.provenance.sidecar_version)),
            Line::from(format!(
                "providers {}",
                if graph.provenance.provider_set.is_empty() {
                    "none".into()
                } else {
                    graph.provenance.provider_set.join(", ")
                }
            )),
            Line::from(format!("seed {}", graph.provenance.analysis_seed)),
            Line::from(format!("generated {}", graph.provenance.generated_at)),
        ],
        None => vec![Line::from("no provenance available")],
    }
}

fn source_warning_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) if !graph.analysis_summary.warnings.is_empty() => graph
            .analysis_summary
            .warnings
            .iter()
            .take(4)
            .flat_map(|warning| {
                [
                    Line::from(format!("{}: {}", warning.code, warning.message)),
                    Line::from(""),
                ]
            })
            .collect(),
        Some(_) => vec![Line::from("no source-graph warnings")],
        None => vec![Line::from("no warnings because no source graph is loaded")],
    }
}

fn source_confidence_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    match shell.app.source_graph.as_ref() {
        Some(graph) => vec![
            Line::from(format!(
                "overall {:.2} | break potential {}",
                graph.analysis_summary.overall_confidence,
                quality_label(&graph.analysis_summary.break_rebuild_potential)
            )),
            Line::from(format!(
                "timing {} | section quality {}",
                quality_label(&graph.analysis_summary.timing_quality),
                quality_label(&graph.analysis_summary.section_quality)
            )),
            Line::from(format!(
                "summary loops {} | hooks {}",
                graph.analysis_summary.loop_candidate_count,
                graph.analysis_summary.hook_candidate_count
            )),
            Line::from(format!("jam trust {}", trust_summary(shell).headline)),
        ],
        None => vec![Line::from("no confidence summary available")],
    }
}

struct TrustSummary {
    headline: &'static str,
    overall_confidence: f32,
    warning_count: usize,
    timing_quality: &'static str,
    section_quality: &'static str,
}

fn trust_summary(shell: &JamShellState) -> TrustSummary {
    match shell.app.source_graph.as_ref() {
        Some(graph) => {
            let overall = graph.analysis_summary.overall_confidence;
            let headline = if overall >= 0.8 {
                "strong"
            } else if overall >= 0.62 {
                "usable"
            } else {
                "tentative"
            };

            TrustSummary {
                headline,
                overall_confidence: overall,
                warning_count: graph.analysis_summary.warnings.len(),
                timing_quality: quality_label(&graph.analysis_summary.timing_quality),
                section_quality: quality_label(&graph.analysis_summary.section_quality),
            }
        }
        None => TrustSummary {
            headline: "unknown",
            overall_confidence: 0.0,
            warning_count: 0,
            timing_quality: "unknown",
            section_quality: "unknown",
        },
    }
}

fn quality_label(quality: &QualityClass) -> &'static str {
    match quality {
        QualityClass::Low => "low",
        QualityClass::Medium => "medium",
        QualityClass::High => "high",
        QualityClass::Unknown => "unknown",
    }
}

fn energy_label(section: &Section) -> &'static str {
    match section.energy_class {
        EnergyClass::Low => "low",
        EnergyClass::Medium => "medium",
        EnergyClass::High => "high",
        EnergyClass::Peak => "peak",
        EnergyClass::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use riotbox_core::{
        action::{
            Action, ActionCommand, ActionDraft, ActionParams, ActionResult, ActionStatus,
            ActionTarget, ActorType, GhostMode, Quantization, TargetScope, UndoPolicy,
        },
        ids::{ActionId, AssetId, BankId, SceneId, SectionId, SourceId},
        queue::ActionQueue,
        session::SessionFile,
        source_graph::{
            AnalysisSummary, AnalysisWarning, Asset, AssetType, Candidate, CandidateType,
            DecodeProfile, EnergyClass, GraphProvenance, QualityClass, Section, SectionLabelHint,
            SourceDescriptor, SourceGraph,
        },
    };

    use super::*;

    fn sample_shell_state() -> JamShellState {
        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-12T00:00:00Z");
        session.runtime_state.transport.position_beats = 32.0;
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-a"));
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-a"));
        session.runtime_state.macro_state.source_retain = 0.7;
        session.runtime_state.macro_state.chaos = 0.2;
        session.runtime_state.macro_state.mc202_touch = 0.8;
        session.runtime_state.macro_state.w30_grit = 0.5;
        session.runtime_state.macro_state.tr909_slam = 0.9;
        session.runtime_state.lane_state.mc202.role = Some("leader".into());
        session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
        session.ghost_state.mode = GhostMode::Assist;
        session.runtime_state.lane_state.tr909.fill_armed_next_bar = true;
        session.runtime_state.lane_state.tr909.last_fill_bar = Some(6);
        session.runtime_state.lane_state.tr909.reinforcement_mode = Some("hybrid".into());
        session.action_log.actions.push(Action {
            id: ActionId(1),
            actor: ActorType::User,
            command: ActionCommand::CaptureNow,
            params: ActionParams::Capture { bars: Some(2) },
            target: ActionTarget {
                scope: Some(TargetScope::LaneW30),
                ..Default::default()
            },
            requested_at: 100,
            quantization: Quantization::NextBar,
            status: ActionStatus::Committed,
            committed_at: Some(120),
            result: Some(ActionResult {
                accepted: true,
                summary: "captured".into(),
            }),
            undo_policy: UndoPolicy::Undoable,
            explanation: Some("capture opener".into()),
        });
        session.action_log.actions.push(Action {
            id: ActionId(2),
            actor: ActorType::Ghost,
            command: ActionCommand::MutateScene,
            params: ActionParams::Mutation {
                intensity: 0.4,
                target_id: Some("scene-a".into()),
            },
            target: ActionTarget {
                scope: Some(TargetScope::Scene),
                scene_id: Some(SceneId::from("scene-a")),
                ..Default::default()
            },
            requested_at: 125,
            quantization: Quantization::NextPhrase,
            status: ActionStatus::Rejected,
            committed_at: None,
            result: Some(ActionResult {
                accepted: false,
                summary: "scene lock blocked ghost mutation".into(),
            }),
            undo_policy: UndoPolicy::NotUndoable {
                reason: "rejected actions do not create undo state".into(),
            },
            explanation: Some("ghost suggestion rejected".into()),
        });
        session.action_log.actions.push(Action {
            id: ActionId(3),
            actor: ActorType::User,
            command: ActionCommand::UndoLast,
            params: ActionParams::Empty,
            target: ActionTarget {
                scope: Some(TargetScope::Session),
                ..Default::default()
            },
            requested_at: 140,
            quantization: Quantization::Immediate,
            status: ActionStatus::Undone,
            committed_at: Some(140),
            result: Some(ActionResult {
                accepted: true,
                summary: "undid most recent musical action".into(),
            }),
            undo_policy: UndoPolicy::NotUndoable {
                reason: "undo markers are not undoable".into(),
            },
            explanation: Some("user trust correction".into()),
        });

        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: SourceId::from("src-1"),
                path: "fixtures/input.wav".into(),
                content_hash: "hash-1".into(),
                duration_seconds: 12.0,
                sample_rate: 44_100,
                channel_count: 2,
                decode_profile: DecodeProfile::Native,
            },
            GraphProvenance {
                sidecar_version: "0.1.0".into(),
                provider_set: vec!["decoded.wav_baseline".into()],
                generated_at: "2026-04-12T00:00:00Z".into(),
                source_hash: "hash-1".into(),
                analysis_seed: 1,
                run_notes: Some("test".into()),
            },
        );
        graph.timing.bpm_estimate = Some(126.0);
        graph.timing.bpm_confidence = 0.76;
        graph.sections.push(Section {
            section_id: SectionId::from("section-a"),
            label_hint: SectionLabelHint::Intro,
            start_seconds: 0.0,
            end_seconds: 4.0,
            bar_start: 1,
            bar_end: 2,
            energy_class: EnergyClass::Medium,
            confidence: 0.71,
            tags: vec!["decoded_wave".into()],
        });
        graph.sections.push(Section {
            section_id: SectionId::from("section-b"),
            label_hint: SectionLabelHint::Drop,
            start_seconds: 4.0,
            end_seconds: 12.0,
            bar_start: 3,
            bar_end: 6,
            energy_class: EnergyClass::High,
            confidence: 0.83,
            tags: vec!["decoded_wave".into()],
        });
        graph.assets.push(Asset {
            asset_id: AssetId::from("asset-a"),
            asset_type: AssetType::LoopWindow,
            start_seconds: 0.0,
            end_seconds: 4.0,
            start_bar: 1,
            end_bar: 2,
            confidence: 0.79,
            tags: vec!["loop".into()],
            source_refs: vec!["src-1".into()],
        });
        graph.candidates.push(Candidate {
            candidate_id: "cand-loop".into(),
            candidate_type: CandidateType::LoopCandidate,
            asset_ref: AssetId::from("asset-a"),
            score: 0.84,
            confidence: 0.78,
            tags: vec!["decoded_wave".into()],
            constraints: vec!["bar_aligned".into()],
            provenance_refs: vec!["provider:decoded.wav_baseline".into()],
        });
        graph.analysis_summary = AnalysisSummary {
            overall_confidence: 0.74,
            timing_quality: QualityClass::Medium,
            section_quality: QualityClass::High,
            loop_candidate_count: 1,
            hook_candidate_count: 0,
            break_rebuild_potential: QualityClass::High,
            warnings: vec![AnalysisWarning {
                code: "wav_baseline_only".into(),
                message: "decoded-source baseline used WAV metadata and simple energy heuristics"
                    .into(),
            }],
        };

        let mut queue = ActionQueue::new();
        queue.enqueue(
            ActionDraft::new(
                ActorType::Ghost,
                ActionCommand::MutateScene,
                Quantization::NextBar,
                ActionTarget {
                    scope: Some(TargetScope::Scene),
                    ..Default::default()
                },
            ),
            130,
        );

        let app = JamAppState::from_parts(session, Some(graph), queue);
        JamShellState::new(app, ShellLaunchMode::Ingest)
    }

    #[test]
    fn renders_more_musical_jam_shell_snapshot() {
        let shell = sample_shell_state();
        let rendered = render_jam_shell_snapshot(&shell, 100, 30);

        assert!(rendered.contains("trust usable"));
        assert!(rendered.contains("scene-a"));
        assert!(rendered.contains("queued"));
        assert!(rendered.contains("ghost"));
        assert!(rendered.contains("overall"));
        assert!(rendered.contains("warnings"));
        assert!(rendered.contains("recent"));
        assert!(rendered.contains("[commit"));
        assert!(rendered.contains("Capture"));
    }

    #[test]
    fn shell_state_handles_help_refresh_and_action_keys() {
        let mut shell = sample_shell_state();

        assert_eq!(
            shell.handle_key_code(KeyCode::Char('?')),
            ShellKeyOutcome::Continue
        );
        assert!(shell.show_help);
        assert_eq!(shell.status_message, "help overlay opened");

        assert_eq!(
            shell.handle_key_code(KeyCode::Char('r')),
            ShellKeyOutcome::RequestRefresh
        );
        assert_eq!(shell.status_message, "re-ingest source requested");
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('2')),
            ShellKeyOutcome::Continue
        );
        assert_eq!(shell.active_screen, ShellScreen::Log);
        assert_eq!(shell.status_message, "switched to log screen");
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('3')),
            ShellKeyOutcome::Continue
        );
        assert_eq!(shell.active_screen, ShellScreen::Source);
        assert_eq!(shell.status_message, "switched to source screen");
        assert_eq!(
            shell.handle_key_code(KeyCode::Tab),
            ShellKeyOutcome::Continue
        );
        assert_eq!(shell.active_screen, ShellScreen::Jam);
        assert_eq!(shell.status_message, "switched to jam screen");
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('m')),
            ShellKeyOutcome::QueueSceneMutation
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('f')),
            ShellKeyOutcome::QueueTr909Fill
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('d')),
            ShellKeyOutcome::QueueTr909Reinforce
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('c')),
            ShellKeyOutcome::QueueCaptureBar
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('u')),
            ShellKeyOutcome::UndoLast
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char(' ')),
            ShellKeyOutcome::ToggleTransport
        );

        assert_eq!(shell.handle_key_code(KeyCode::Esc), ShellKeyOutcome::Quit);
    }

    #[test]
    fn renders_log_shell_snapshot_with_action_trust_history() {
        let mut shell = sample_shell_state();
        shell.active_screen = ShellScreen::Log;
        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("[2 Log]"));
        assert!(rendered.contains("Queued / Pending"));
        assert!(rendered.contains("Accepted / Committed"));
        assert!(rendered.contains("Rejected / Undone"));
        assert!(rendered.contains("ghost"));
        assert!(rendered.contains("mutate.scene"));
        assert!(rendered.contains("scene lock blocked ghost"));
        assert!(rendered.contains("mutation"));
        assert!(rendered.contains("undid most recent musical"));
    }

    #[test]
    fn renders_source_shell_snapshot_with_analysis_structure() {
        let mut shell = sample_shell_state();
        shell.active_screen = ShellScreen::Source;
        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("[3 Source]"));
        assert!(rendered.contains("Identity"));
        assert!(rendered.contains("Timing"));
        assert!(rendered.contains("Sections"));
        assert!(rendered.contains("Candidates"));
        assert!(rendered.contains("Provenance"));
        assert!(rendered.contains("Source Graph Warnings"));
        assert!(rendered.contains("decoded.wav_baseline"));
        assert!(rendered.contains("fixtures/input.wav"));
        assert!(rendered.contains("wav_baseline_only"));
    }
}
