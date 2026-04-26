use crossterm::event::KeyCode;
use ratatui::{
    Frame, Terminal,
    backend::TestBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use riotbox_audio::w30::W30PreviewRenderMode;
use riotbox_core::source_graph::{
    DecodeProfile, EnergyClass, QualityClass, Section, SectionLabelHint,
};
use riotbox_core::{
    action::{ActionCommand, ActionStatus},
    view::jam::{
        CaptureHandoffReadinessView, CaptureTargetKindView, SceneJumpAvailabilityView,
        W30PendingAuditionKind,
    },
};

use crate::jam_app::JamAppState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellScreen {
    Jam,
    Log,
    Source,
    Capture,
}

impl ShellScreen {
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Jam => "jam",
            Self::Log => "log",
            Self::Source => "source",
            Self::Capture => "capture",
        }
    }

    #[must_use]
    pub const fn next(&self) -> Self {
        match self {
            Self::Jam => Self::Log,
            Self::Log => Self::Source,
            Self::Source => Self::Capture,
            Self::Capture => Self::Jam,
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
pub enum JamViewMode {
    Perform,
    Inspect,
}

impl JamViewMode {
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Perform => "perform",
            Self::Inspect => "inspect",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellKeyOutcome {
    Continue,
    RequestRefresh,
    ToggleTransport,
    QueueSceneMutation,
    QueueSceneSelect,
    QueueSceneRestore,
    QueueMc202RoleToggle,
    QueueMc202GenerateFollower,
    QueueMc202GenerateAnswer,
    QueueTr909Fill,
    QueueTr909Reinforce,
    QueueTr909Slam,
    QueueTr909Takeover,
    QueueTr909SceneLock,
    QueueTr909Release,
    QueueCaptureBar,
    PromoteLastCapture,
    QueueW30TriggerPad,
    QueueW30StepFocus,
    QueueW30SwapBank,
    QueueW30BrowseSlicePool,
    QueueW30ApplyDamageProfile,
    QueueW30LoopFreeze,
    QueueW30LiveRecall,
    QueueW30Audition,
    QueueW30Resample,
    TogglePinLatestCapture,
    LowerDrumBusLevel,
    RaiseDrumBusLevel,
    UndoLast,
    Quit,
}

const GESTURE_MUTATE: &str = "mutate";
const GESTURE_SCENE_JUMP: &str = "scene jump";
const GESTURE_RESTORE: &str = "restore";
const GESTURE_VOICE: &str = "voice";
const GESTURE_FOLLOW: &str = "follow";
const GESTURE_ANSWER: &str = "answer";
const GESTURE_FILL: &str = "fill";
const GESTURE_PUSH: &str = "push";
const GESTURE_SLAM: &str = "slam";
const GESTURE_TAKEOVER: &str = "takeover";
const GESTURE_LOCK: &str = "lock";
const GESTURE_RELEASE: &str = "release";
const GESTURE_CAPTURE: &str = "capture";
const GESTURE_PROMOTE: &str = "promote";
const GESTURE_HIT: &str = "hit";
const GESTURE_NEXT_PAD: &str = "next pad";
const GESTURE_BANK: &str = "bank";
const GESTURE_BROWSE: &str = "browse";
const GESTURE_DAMAGE: &str = "damage";
const GESTURE_FREEZE: &str = "freeze";
const GESTURE_RECALL: &str = "recall";
const GESTURE_AUDITION: &str = "audition";
const GESTURE_RESAMPLE: &str = "resample";
const GESTURE_UNDO: &str = "undo";

const ADVANCED_GESTURES: &[(&str, &str)] = &[
    ("Y", GESTURE_RESTORE),
    ("a", GESTURE_ANSWER),
    ("b", GESTURE_VOICE),
    ("d", GESTURE_PUSH),
    ("t", GESTURE_TAKEOVER),
    ("k", GESTURE_LOCK),
];

const LANE_GESTURES: &[(&str, &str)] = &[
    ("l", GESTURE_RECALL),
    ("o", GESTURE_AUDITION),
    ("z", GESTURE_FREEZE),
    ("e", GESTURE_RESAMPLE),
    ("B", GESTURE_BANK),
    ("j", GESTURE_BROWSE),
];

const HELP_PRIMARY_CONFIRM_GESTURES: &[(&str, &str)] = &[
    ("c", GESTURE_CAPTURE),
    ("w", GESTURE_HIT),
    ("u", GESTURE_UNDO),
];

const HELP_ADVANCED_GESTURES_A: &[(&str, &str)] = &[
    ("Y", GESTURE_RESTORE),
    ("a", GESTURE_ANSWER),
    ("m", GESTURE_MUTATE),
    ("b", GESTURE_VOICE),
    ("d", GESTURE_PUSH),
];

const HELP_ADVANCED_GESTURES_B: &[(&str, &str)] = &[
    ("s", GESTURE_SLAM),
    ("t", GESTURE_TAKEOVER),
    ("k", GESTURE_LOCK),
    ("x", GESTURE_RELEASE),
];

const HELP_ADVANCED_GESTURES_C: &[(&str, &str)] = &[
    ("p", GESTURE_PROMOTE),
    ("n", GESTURE_NEXT_PAD),
    ("B", GESTURE_BANK),
    ("j", GESTURE_BROWSE),
];

const HELP_ADVANCED_GESTURES_D: &[(&str, &str)] = &[
    ("D", GESTURE_DAMAGE),
    ("z", GESTURE_FREEZE),
    ("l", GESTURE_RECALL),
    ("o", GESTURE_AUDITION),
    ("e", GESTURE_RESAMPLE),
];

fn render_gesture_items(items: &[(&str, &str)], separator: &str) -> String {
    items
        .iter()
        .map(|(key, label)| format!("{key}{separator}{label}"))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn queued_status_message(label: &str, boundary: &str) -> String {
    format!("queue {label} on {boundary}")
}

#[derive(Clone, Debug)]
pub struct JamShellState {
    pub app: JamAppState,
    pub launch_mode: ShellLaunchMode,
    pub active_screen: ShellScreen,
    pub jam_mode: JamViewMode,
    pub first_run_onramp: bool,
    pub show_help: bool,
    pub status_message: String,
}

impl JamShellState {
    #[must_use]
    pub fn new(app: JamAppState, launch_mode: ShellLaunchMode) -> Self {
        let first_run_onramp = matches!(launch_mode, ShellLaunchMode::Ingest)
            && app.session.action_log.actions.is_empty()
            && app.session.captures.is_empty();
        let status_message = match launch_mode {
            ShellLaunchMode::Load => "loaded session from disk".into(),
            ShellLaunchMode::Ingest => "ingested source into Jam shell".into(),
        };

        Self {
            app,
            launch_mode,
            active_screen: ShellScreen::Jam,
            jam_mode: JamViewMode::Perform,
            first_run_onramp,
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
            KeyCode::Char('4') => {
                self.active_screen = ShellScreen::Capture;
                self.status_message = "switched to capture screen".into();
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
            KeyCode::Char('i') => {
                if self.active_screen != ShellScreen::Jam {
                    self.status_message = "open Jam first if you want to use inspect".into();
                } else if first_run_onramp_stage(self).is_some() {
                    self.status_message =
                        "finish the first guided move before opening inspect".into();
                } else {
                    self.jam_mode = match self.jam_mode {
                        JamViewMode::Perform => JamViewMode::Inspect,
                        JamViewMode::Inspect => JamViewMode::Perform,
                    };
                    self.status_message = match self.jam_mode {
                        JamViewMode::Perform => "returned Jam to perform mode".into(),
                        JamViewMode::Inspect => {
                            "opened Jam inspect | press i to return to perform".into()
                        }
                    };
                }
                ShellKeyOutcome::Continue
            }
            KeyCode::Char('r') => {
                self.status_message = format!("{} requested", self.launch_mode.refresh_verb());
                ShellKeyOutcome::RequestRefresh
            }
            KeyCode::Char('m') => {
                self.status_message = queued_status_message(GESTURE_MUTATE, "next bar");
                ShellKeyOutcome::QueueSceneMutation
            }
            KeyCode::Char('y') => {
                self.status_message = queued_status_message(GESTURE_SCENE_JUMP, "next bar");
                ShellKeyOutcome::QueueSceneSelect
            }
            KeyCode::Char('Y') => {
                self.status_message = queued_status_message(GESTURE_RESTORE, "next bar");
                ShellKeyOutcome::QueueSceneRestore
            }
            KeyCode::Char('b') => {
                self.status_message = queued_status_message(GESTURE_VOICE, "next phrase");
                ShellKeyOutcome::QueueMc202RoleToggle
            }
            KeyCode::Char('g') => {
                self.status_message = queued_status_message(GESTURE_FOLLOW, "next phrase");
                ShellKeyOutcome::QueueMc202GenerateFollower
            }
            KeyCode::Char('a') => {
                self.status_message = queued_status_message(GESTURE_ANSWER, "next phrase");
                ShellKeyOutcome::QueueMc202GenerateAnswer
            }
            KeyCode::Char('f') => {
                self.status_message = queued_status_message(GESTURE_FILL, "next bar");
                ShellKeyOutcome::QueueTr909Fill
            }
            KeyCode::Char('d') => {
                self.status_message = queued_status_message(GESTURE_PUSH, "next phrase");
                ShellKeyOutcome::QueueTr909Reinforce
            }
            KeyCode::Char('s') => {
                self.status_message = queued_status_message(GESTURE_SLAM, "next beat");
                ShellKeyOutcome::QueueTr909Slam
            }
            KeyCode::Char('t') => {
                self.status_message = queued_status_message(GESTURE_TAKEOVER, "next phrase");
                ShellKeyOutcome::QueueTr909Takeover
            }
            KeyCode::Char('k') => {
                self.status_message = queued_status_message(GESTURE_LOCK, "next phrase");
                ShellKeyOutcome::QueueTr909SceneLock
            }
            KeyCode::Char('x') => {
                self.status_message = queued_status_message(GESTURE_RELEASE, "next phrase");
                ShellKeyOutcome::QueueTr909Release
            }
            KeyCode::Char('c') => {
                self.status_message = queued_status_message(GESTURE_CAPTURE, "next phrase");
                ShellKeyOutcome::QueueCaptureBar
            }
            KeyCode::Char('p') => {
                self.status_message = format!("queue {GESTURE_PROMOTE} for latest capture");
                ShellKeyOutcome::PromoteLastCapture
            }
            KeyCode::Char('w') => {
                self.status_message = queued_status_message(GESTURE_HIT, "next beat");
                ShellKeyOutcome::QueueW30TriggerPad
            }
            KeyCode::Char('n') => {
                self.status_message = queued_status_message(GESTURE_NEXT_PAD, "next beat");
                ShellKeyOutcome::QueueW30StepFocus
            }
            KeyCode::Char('B') => {
                self.status_message = queued_status_message(GESTURE_BANK, "next bar");
                ShellKeyOutcome::QueueW30SwapBank
            }
            KeyCode::Char('j') => {
                self.status_message = queued_status_message(GESTURE_BROWSE, "next beat");
                ShellKeyOutcome::QueueW30BrowseSlicePool
            }
            KeyCode::Char('D') => {
                self.status_message = queued_status_message(GESTURE_DAMAGE, "next bar");
                ShellKeyOutcome::QueueW30ApplyDamageProfile
            }
            KeyCode::Char('z') => {
                self.status_message = queued_status_message(GESTURE_FREEZE, "next phrase");
                ShellKeyOutcome::QueueW30LoopFreeze
            }
            KeyCode::Char('l') => {
                self.status_message = queued_status_message(GESTURE_RECALL, "next bar");
                ShellKeyOutcome::QueueW30LiveRecall
            }
            KeyCode::Char('o') => {
                self.status_message = queued_status_message(GESTURE_AUDITION, "next bar");
                ShellKeyOutcome::QueueW30Audition
            }
            KeyCode::Char('e') => {
                self.status_message = queued_status_message(GESTURE_RESAMPLE, "next phrase");
                ShellKeyOutcome::QueueW30Resample
            }
            KeyCode::Char('v') => {
                self.status_message = "toggle pin for latest capture".into();
                ShellKeyOutcome::TogglePinLatestCapture
            }
            KeyCode::Char('[') => {
                self.status_message = "lower drum bus level".into();
                ShellKeyOutcome::LowerDrumBusLevel
            }
            KeyCode::Char(']') => {
                self.status_message = "raise drum bus level".into();
                ShellKeyOutcome::RaiseDrumBusLevel
            }
            KeyCode::Char('u') => {
                self.status_message = "undo most recent action requested".into();
                ShellKeyOutcome::UndoLast
            }
            _ => ShellKeyOutcome::Continue,
        }
    }

    pub fn replace_app_state(&mut self, app: JamAppState) {
        self.first_run_onramp = matches!(self.launch_mode, ShellLaunchMode::Ingest)
            && app.session.action_log.actions.is_empty()
            && app.session.captures.is_empty();
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

fn render_jam_body(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
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
                Constraint::Length(7),
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
                Constraint::Length(7),
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
            screen_context_label(shell),
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
            "{} @ {:.1}",
            transport_label(shell),
            shell.app.jam_view.transport.position_beats
        )),
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
            Line::from("Goal: get one obvious first change before doing anything else"),
        ],
        Some(FirstRunOnrampStage::QueuedFirstMove) => vec![
            Line::from("Your first move is armed."),
            Line::from("Let transport cross the next bar so the fill can actually land."),
            Line::from("Then [2] confirm it in Log and decide: [c] capture it or [u] undo it."),
        ],
        Some(FirstRunOnrampStage::FirstResult) => vec![
            Line::from(format!("What changed: {}", latest_landed_text(shell))),
            Line::from("What next: [c] capture it or [u] undo it if it missed."),
            Line::from("Then try one more move: [y] jump or [g] follow."),
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

    let render_focus = Paragraph::new(vec![
        Line::from(format!(
            "{} | scene {} | {}",
            if shell.app.runtime.transport.is_playing {
                format!(
                    "running @ {:.1}",
                    shell.app.runtime.transport.position_beats
                )
            } else {
                format!(
                    "stopped @ {:.1}",
                    shell.app.runtime.transport.position_beats
                )
            },
            shell
                .app
                .runtime
                .transport
                .current_scene
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "none".into()),
            shell
                .app
                .runtime
                .last_commit_boundary
                .as_ref()
                .map(|boundary| {
                    format!(
                        "{:?} b{} p{}",
                        boundary.kind, boundary.bar_index, boundary.phrase_index
                    )
                })
                .unwrap_or_else(|| "boundary none".into())
        )),
        Line::from(format!(
            "render {} | accent {}",
            shell.app.runtime_view.tr909_render_mode,
            shell.app.runtime_view.tr909_render_support_accent
        )),
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
    ])
    .block(
        Block::default()
            .title("TR-909 Render")
            .borders(Borders::ALL),
    )
    .wrap(Wrap { trim: true });

    let warnings = log_warning_lines(shell);
    let warnings_panel = Paragraph::new(warnings)
        .block(Block::default().title("Warnings").borders(Borders::ALL))
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

    if shell.app.runtime_view.runtime_warnings.is_empty() && shell.app.jam_view.warnings.is_empty()
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

fn footer_status_line(status: &str) -> Line<'static> {
    Line::from(Span::styled(status.to_owned(), style_low_emphasis()))
}

fn footer_ok_line(message: &str) -> Line<'static> {
    Line::from(Span::styled(message.to_owned(), style_confirmation()))
}

fn footer_warning_line(warning: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("Warning:", style_warning_label()),
        Span::styled(format!(" {warning}"), style_warning_detail()),
    ])
}

fn spans_with_primary_gesture_keys(gestures: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    for (index, gesture) in gestures.split(" | ").enumerate() {
        if index > 0 {
            spans.push(Span::raw(" | "));
        }

        let Some((key, label)) = gesture.split_once(' ') else {
            spans.push(Span::styled(gesture.to_owned(), style_primary_control()));
            continue;
        };

        spans.push(Span::styled(key.to_owned(), style_primary_control()));
        spans.push(Span::raw(format!(" {label}")));
    }

    spans
}

fn spans_with_primary_legend_keys(legend: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    for (index, item) in legend.split(" | ").enumerate() {
        if index > 0 {
            spans.push(Span::raw(" | "));
        }

        let Some((key, label)) = item.split_once(' ') else {
            spans.push(Span::styled(item.to_owned(), style_primary_control()));
            continue;
        };

        if key == "[" && label.starts_with("] ") {
            spans.push(Span::styled("[ ]", style_primary_control()));
            spans.push(Span::raw(label[1..].to_owned()));
            continue;
        }

        spans.push(Span::styled(key.to_owned(), style_primary_control()));
        spans.push(Span::raw(format!(" {label}")));
    }

    spans
}

fn footer_scene_affordance_cue(shell: &JamShellState) -> Option<String> {
    if shell.active_screen != ShellScreen::Jam {
        return None;
    }

    if let Some((label, scene_id, boundary)) = pending_scene_transition(shell) {
        let scene = compact_scene_label(scene_id.as_str());
        let tick = scene_countdown_cue(shell.app.runtime.transport.beat_index);
        if let Some(direction) = compact_energy_delta_label(
            shell.app.jam_view.scene.active_scene_energy.as_deref(),
            scene_energy_label_for_scene_id(shell, scene_id.as_str()),
        ) {
            return Some(format!(
                "{label} {scene} @ {boundary} | {direction} {tick} | 2 trail"
            ));
        }
        return Some(format!(
            "{label} {scene} @ {boundary} | {tick} energy | 2 trail"
        ));
    }

    if show_restore_ready_cue(shell) {
        let restore_target = restore_scene_target_compact_label(shell);
        if let Some(direction) = restore_scene_energy_direction_label(shell) {
            return Some(format!(
                "restore {restore_target} ready | {direction} | Y brings back {restore_target}"
            ));
        }
        return Some(format!(
            "restore {restore_target} ready | Y brings back {restore_target}"
        ));
    }

    None
}

fn render_help_overlay(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let popup = centered_rect(60, 55, area);
    let mut lines = vec![
        Line::from("Jam shell keys"),
        line_with_primary_key_prefixes("q or Esc: quit"),
        line_with_primary_key_prefixes("? or h: toggle help"),
        line_with_primary_key_prefixes(
            "1: Jam screen | 2: Log screen | 3: Source screen | 4: Capture screen | Tab: next screen",
        ),
        line_with_primary_key_prefixes(
            "i: open inspect from Jam | press i again to return to perform",
        ),
    ];

    if let Some(stage) = first_run_onramp_stage(shell) {
        lines.push(Line::from(""));
        lines.push(Line::from("First run"));
        match stage {
            FirstRunOnrampStage::Start => {
                lines.push(line_with_primary_key_prefixes("space: start transport"));
                lines.push(line_with_primary_key_prefixes("f: queue one first fill"));
                lines.push(line_with_primary_key_prefixes(
                    "2: switch to Log and watch it land",
                ));
            }
            FirstRunOnrampStage::QueuedFirstMove => {
                lines.push(Line::from("let transport cross the next bar"));
                lines.push(line_with_primary_key_prefixes(
                    "2: confirm the first landed action in Log",
                ));
                lines.push(line_with_primary_key_prefixes("c: capture it | u: undo it"));
            }
            FirstRunOnrampStage::FirstResult => {
                lines.push(line_with_primary_key_prefixes(
                    "c: capture the first keeper",
                ));
                lines.push(line_with_primary_key_prefixes("u: undo it if it missed"));
                lines.push(line_with_primary_key_prefixes(
                    "y / g / w: try one more gesture",
                ));
            }
        }
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

    lines.extend([
        Line::from(""),
        Line::from("Primary gestures"),
        line_with_primary_key_prefixes(format!(
            "space: play / pause | {}",
            render_help_primary_gesture_items(shell)
        )),
        line_with_primary_key_prefixes(format!(
            "{} | 2: confirm in Log",
            render_gesture_items(HELP_PRIMARY_CONFIRM_GESTURES, ": ")
        )),
        Line::from(""),
        Line::from("After first loop: docs/jam_recipes.md -> Recipe 2 / Recipe 5"),
        Line::from(""),
        Line::from("Advanced / lane gestures"),
        line_with_primary_key_prefixes(format!("r: {}", shell.launch_mode.refresh_verb())),
        line_with_primary_key_prefixes(render_gesture_items(HELP_ADVANCED_GESTURES_A, ": ")),
        line_with_primary_key_prefixes(render_gesture_items(HELP_ADVANCED_GESTURES_B, ": ")),
        line_with_primary_key_prefixes(render_gesture_items(HELP_ADVANCED_GESTURES_C, ": ")),
        line_with_primary_key_prefixes(render_gesture_items(HELP_ADVANCED_GESTURES_D, ": ")),
        line_with_primary_key_prefixes("[ / ]: lower or raise drum bus | v: pin latest"),
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

fn pending_scene_help_lines(shell: &JamShellState) -> Option<Vec<Line<'static>>> {
    let (label, scene_id, boundary) = pending_scene_transition(shell)?;
    let scene = compact_scene_label(scene_id.as_str());

    Some(vec![
        Line::from(""),
        Line::from("Scene timing"),
        Line::from(format!("{label} {scene}: lands at {boundary}")),
        Line::from("Jam: read launch/restore, pulse, live/restore energy"),
        line_with_primary_key_prefixes("2: confirm the landed trail on Log"),
    ])
}

fn scene_restore_help_lines(shell: &JamShellState) -> Option<Vec<Line<'static>>> {
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

fn capture_help_lines(shell: &JamShellState) -> Option<Vec<Line<'static>>> {
    if shell.active_screen != ShellScreen::Capture {
        return None;
    }

    Some(vec![
        Line::from(""),
        Line::from("Capture path"),
        Line::from("Do Next: read capture -> promote -> hit"),
        line_with_primary_keys("hear ... stored means [p] promote, then [w] hit"),
        line_with_primary_key_prefixes("2: confirm promote, hit, and audition results in Log"),
    ])
}

fn screen_context_label(shell: &JamShellState) -> String {
    match shell.active_screen {
        ShellScreen::Jam => format!("jam/{}", shell.jam_mode.label()),
        _ => shell.active_screen.label().into(),
    }
}

fn mc202_perform_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let lanes = &shell.app.jam_view.lanes;
    let next = if let Some(role) = lanes.mc202_pending_role.as_deref() {
        format!("next voice {role}")
    } else if lanes.mc202_pending_answer_generation {
        "next answer".into()
    } else if lanes.mc202_pending_follower_generation {
        "next follow".into()
    } else {
        "next none".into()
    };

    vec![
        Line::from(format!(
            "current voice {}",
            lanes.mc202_role.as_deref().unwrap_or("unset")
        )),
        Line::from(next),
        Line::from(format!(
            "current phrase {}",
            lanes.mc202_phrase_ref.as_deref().unwrap_or("unset")
        )),
        Line::from(format!(
            "sound {} / {}",
            shell.app.runtime_view.mc202_render_mode,
            shell.app.runtime_view.mc202_render_phrase_shape
        )),
    ]
}

fn w30_perform_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let next = if w30_pending_cue_label(shell) != "idle" {
        format!("next {}", w30_pending_cue_label(shell))
    } else {
        format!("next {}", w30_operation_status_compact(shell))
    };

    vec![
        Line::from(format!("current pad {}", w30_target_compact(shell))),
        Line::from(format!(
            "current preview {}",
            w30_preview_mode_profile_compact(shell)
        )),
        Line::from(next),
    ]
}

fn tr909_perform_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let next = tr909_next_line(shell);

    vec![
        Line::from(format!(
            "current mode {}",
            if shell.app.jam_view.lanes.tr909_takeover_enabled {
                "takeover"
            } else {
                "support"
            }
        )),
        Line::from(format!(
            "current fill {} | slam {:.2}",
            if shell.app.jam_view.lanes.tr909_fill_armed_next_bar {
                "armed"
            } else {
                "idle"
            },
            shell.app.jam_view.macros.tr909_slam
        )),
        Line::from(format!("next {next}")),
    ]
}

fn tr909_next_line(shell: &JamShellState) -> String {
    use riotbox_core::action::ActionCommand::{
        Tr909FillNext, Tr909ReinforceBreak, Tr909Release, Tr909SceneLock, Tr909SetSlam,
        Tr909Takeover,
    };

    shell
        .app
        .queue
        .pending_actions()
        .iter()
        .find_map(|action| match action.command {
            Tr909FillNext => Some("fill".into()),
            Tr909ReinforceBreak => Some("push".into()),
            Tr909SetSlam => Some("slam".into()),
            Tr909Takeover => Some("takeover".into()),
            Tr909SceneLock => Some("lock".into()),
            Tr909Release => Some("release".into()),
            _ => None,
        })
        .unwrap_or_else(|| {
            if shell.app.jam_view.lanes.tr909_fill_armed_next_bar {
                "fill armed".into()
            } else {
                "none".into()
            }
        })
}

fn tr909_inspect_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let render = &shell.app.runtime_view;
    let last_boundary = shell
        .app
        .runtime
        .last_commit_boundary
        .as_ref()
        .map(|boundary| {
            format!(
                "{:?} b{} p{}",
                boundary.kind, boundary.bar_index, boundary.phrase_index
            )
        })
        .unwrap_or_else(|| "none".into());

    vec![
        Line::from(format!(
            "mode {} | next {}",
            render.tr909_render_mode,
            tr909_next_line(shell)
        )),
        Line::from(format!(
            "profile {} | context {} | accent {} | route {}",
            render.tr909_render_profile,
            render.tr909_render_support_context,
            render.tr909_render_support_accent,
            render.tr909_render_routing
        )),
        Line::from(format!(
            "{} | {}",
            render.tr909_render_pattern_adoption, render.tr909_render_phrase_variation
        )),
        Line::from(render.tr909_render_mix_summary.clone()),
        Line::from(format!(
            "{} | boundary {last_boundary}",
            render.tr909_render_alignment
        )),
    ]
}

fn jam_pending_landed_lines(shell: &JamShellState) -> Vec<Line<'static>> {
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

fn latest_landed_line(shell: &JamShellState) -> Line<'static> {
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

fn latest_landed_text(shell: &JamShellState) -> String {
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

fn scene_history_trail_line(shell: &JamShellState) -> Option<String> {
    let trail = shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .filter(|action| {
            action.status == ActionStatus::Committed
                && matches!(
                    action.command,
                    ActionCommand::SceneLaunch | ActionCommand::SceneRestore
                )
        })
        .take(3)
        .map(|action| {
            let verb = match action.command {
                ActionCommand::SceneLaunch => "jump",
                ActionCommand::SceneRestore => "restore",
                _ => unreachable!("scene trail filter only matches launch/restore"),
            };
            let scene = action
                .result
                .as_ref()
                .and_then(|result| result.summary.split_whitespace().nth(2))
                .or(action
                    .target
                    .scene_id
                    .as_ref()
                    .map(|scene_id| scene_id.as_str()))
                .map(compact_scene_label)
                .unwrap_or_else(|| "none".into());
            format!("{verb} {scene}")
        })
        .collect::<Vec<_>>();

    if trail.is_empty() {
        None
    } else {
        Some(format!("trail {}", trail.join(" <- ")))
    }
}

fn latest_landed_command(shell: &JamShellState) -> Option<&str> {
    shell
        .app
        .jam_view
        .recent_actions
        .first()
        .map(|action| action.command.as_str())
}

fn scene_post_commit_cue_line(shell: &JamShellState) -> Option<Line<'static>> {
    let command = latest_landed_command(shell)?;
    if !matches!(command, "scene.launch" | "scene.restore") {
        return None;
    }

    let current_scene = current_scene_target_compact_label(shell);
    let restore_scene = restore_scene_target_compact_label(shell);
    let next_scene_key = if command == "scene.launch" {
        ("[Y]", " restore ")
    } else {
        ("[y]", " jump ")
    };

    let mut spans = vec![
        Span::styled("scene ", style_low_emphasis()),
        Span::styled(current_scene, style_confirmation_strong()),
        Span::styled(" | restore ", style_low_emphasis()),
        Span::styled(restore_scene, style_pending_detail()),
    ];

    if shell.app.runtime_view.tr909_render_support_accent == "scene" {
        spans.push(Span::styled(" | ", style_low_emphasis()));
        spans.push(Span::styled("909 lift", style_pending_detail()));
    }

    spans.extend([
        Span::styled(" | next ", style_low_emphasis()),
        Span::styled(next_scene_key.0, style_primary_control()),
        Span::raw(next_scene_key.1),
        Span::styled("[c]", style_primary_control()),
        Span::raw(" capture"),
    ]);

    Some(Line::from(spans))
}

fn suggested_gesture_lines(shell: &JamShellState) -> Vec<Line<'static>> {
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
        return vec![
            line_with_primary_keys("[y] jump first"),
            line_with_primary_keys("[Y] restore waits for one landed jump"),
            line_with_primary_keys("[c] capture"),
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

    vec![
        line_with_primary_keys(format!("{}  [g] follow", next_scene_jump_suggestion(shell))),
        line_with_primary_keys("[a] answer  [f] fill"),
        line_with_primary_keys("[c] capture  [w] hit"),
    ]
}

fn line_with_primary_keys(text: impl Into<String>) -> Line<'static> {
    let text = text.into();
    let mut spans = Vec::new();
    let mut rest = text.as_str();

    while let Some(start) = rest.find('[') {
        let (prefix, key_and_tail) = rest.split_at(start);
        if !prefix.is_empty() {
            spans.push(Span::raw(prefix.to_owned()));
        }

        let Some(end) = key_and_tail.find(']') else {
            spans.push(Span::raw(key_and_tail.to_owned()));
            return Line::from(spans);
        };
        let key_end = end + 1;
        let (key, tail) = key_and_tail.split_at(key_end);
        spans.push(Span::styled(key.to_owned(), style_primary_control()));
        rest = tail;
    }

    if !rest.is_empty() || spans.is_empty() {
        spans.push(Span::raw(rest.to_owned()));
    }

    Line::from(spans)
}

fn line_with_primary_key_prefixes(text: impl Into<String>) -> Line<'static> {
    let text = text.into();
    let mut spans = Vec::new();

    for (index, segment) in text.split(" | ").enumerate() {
        if index > 0 {
            spans.push(Span::raw(" | "));
        }

        let Some(colon) = segment.find(':') else {
            spans.push(Span::raw(segment.to_owned()));
            continue;
        };

        let (key, detail) = segment.split_at(colon);
        spans.push(Span::styled(key.to_owned(), style_primary_control()));
        spans.push(Span::raw(detail.to_owned()));
    }

    Line::from(spans)
}

fn show_restore_readiness_cue(shell: &JamShellState) -> bool {
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

fn show_restore_ready_cue(shell: &JamShellState) -> bool {
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

fn jam_warning_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let trust = trust_summary(shell);
    let readiness = if trust.headline == "strong" || trust.headline == "usable" {
        "ready"
    } else {
        "tentative"
    };

    vec![
        Line::from(format!("trust {} | {}", trust.headline, readiness)),
        Line::from(primary_warning_line(shell)),
        Line::from(format!(
            "audio {} | sidecar {}",
            shell.app.runtime_view.audio_status, shell.app.runtime_view.sidecar_status
        )),
    ]
}

fn primary_warning_line(shell: &JamShellState) -> String {
    shell
        .app
        .runtime_view
        .runtime_warnings
        .iter()
        .chain(shell.app.jam_view.warnings.iter())
        .next()
        .map(|warning| warning.to_string())
        .unwrap_or_else(|| "no major warning".into())
}

fn source_inspect_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let source = &shell.app.jam_view.source;
    let first_section = shell
        .app
        .source_graph
        .as_ref()
        .and_then(|graph| graph.sections.first())
        .map(section_compact_label)
        .unwrap_or_else(|| "first none".into());
    let second_section = shell
        .app
        .source_graph
        .as_ref()
        .and_then(|graph| graph.sections.get(1))
        .map(section_compact_label)
        .unwrap_or_else(|| "next none".into());

    vec![
        Line::from(format!(
            "tempo {:.1} | trust {}",
            source.bpm_estimate.unwrap_or(0.0),
            trust_summary(shell).headline
        )),
        Line::from(format!(
            "sections {} | loops {} | hooks {}",
            source.section_count, source.loop_candidate_count, source.hook_candidate_count
        )),
        Line::from(first_section),
        Line::from(second_section),
        source_warning_lines(shell)
            .into_iter()
            .next()
            .unwrap_or_else(|| Line::from("warnings clear")),
    ]
}

fn material_inspect_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let capture = &shell.app.jam_view.capture;
    vec![
        Line::from(format!(
            "captures {} | pending {}",
            capture.capture_count, capture.pending_capture_count
        )),
        Line::from(format!("w30 {}", w30_target_compact(shell))),
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
        Line::from(format!(
            "notes {}",
            capture
                .last_capture_notes
                .as_deref()
                .unwrap_or("no capture note yet")
        )),
    ]
}

fn jam_diagnostic_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let last_boundary = shell
        .app
        .runtime
        .last_commit_boundary
        .as_ref()
        .map(|boundary| {
            format!(
                "{:?} b{} p{}",
                boundary.kind, boundary.bar_index, boundary.phrase_index
            )
        })
        .unwrap_or_else(|| "none".into());

    vec![
        Line::from(format!(
            "audio {} | sidecar {}",
            shell.app.runtime_view.audio_status, shell.app.runtime_view.sidecar_status
        )),
        Line::from(format!(
            "transport {} @ {:.1}",
            transport_label(shell),
            shell.app.runtime.transport.position_beats
        )),
        Line::from(format!("last boundary {last_boundary}")),
        Line::from(format!(
            "pending {} | landed {}",
            shell.app.jam_view.pending_actions.len(),
            shell.app.jam_view.recent_actions.len()
        )),
        Line::from(primary_warning_line(shell)),
    ]
}

fn mc202_pending_role_label(shell: &JamShellState) -> &'static str {
    if shell.app.jam_view.lanes.mc202_pending_role.is_some() {
        "voice queued"
    } else if shell.app.jam_view.lanes.mc202_pending_answer_generation {
        "answer queued"
    } else if shell.app.jam_view.lanes.mc202_pending_follower_generation {
        "follow queued"
    } else {
        "stable"
    }
}

fn jam_action_label(command: &str) -> String {
    match command {
        "mutate.scene" => GESTURE_MUTATE.into(),
        "scene.launch" => GESTURE_SCENE_JUMP.into(),
        "scene.restore" => GESTURE_RESTORE.into(),
        "mc202.set_role" => GESTURE_VOICE.into(),
        "mc202.generate_follower" => GESTURE_FOLLOW.into(),
        "mc202.generate_answer" => GESTURE_ANSWER.into(),
        "tr909.fill_next" => GESTURE_FILL.into(),
        "tr909.reinforce_break" => GESTURE_PUSH.into(),
        "tr909.set_slam" => GESTURE_SLAM.into(),
        "tr909.takeover" => GESTURE_TAKEOVER.into(),
        "tr909.scene_lock" => GESTURE_LOCK.into(),
        "tr909.release" => GESTURE_RELEASE.into(),
        "capture.now" | "capture.loop" | "capture.bar_group" => GESTURE_CAPTURE.into(),
        "promote.capture_to_pad" | "promote.capture_to_scene" => GESTURE_PROMOTE.into(),
        "w30.trigger_pad" => GESTURE_HIT.into(),
        "w30.step_focus" => GESTURE_NEXT_PAD.into(),
        "w30.swap_bank" => GESTURE_BANK.into(),
        "w30.browse_slice_pool" => GESTURE_BROWSE.into(),
        "w30.apply_damage_profile" => GESTURE_DAMAGE.into(),
        "w30.loop_freeze" => GESTURE_FREEZE.into(),
        "w30.live_recall" => GESTURE_RECALL.into(),
        "w30.audition_raw_capture" => GESTURE_AUDITION.into(),
        "w30.audition_promoted" => GESTURE_AUDITION.into(),
        "promote.resample" => GESTURE_RESAMPLE.into(),
        _ => command.to_string(),
    }
}

fn mc202_log_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let lanes = &shell.app.jam_view.lanes;
    let last_mc202_action = shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .find(|action| {
            matches!(
                action.command,
                riotbox_core::action::ActionCommand::Mc202SetRole
                    | riotbox_core::action::ActionCommand::Mc202GenerateFollower
                    | riotbox_core::action::ActionCommand::Mc202GenerateAnswer
            )
        })
        .map(|action| action.command.to_string())
        .unwrap_or_else(|| "none".into());

    vec![
        Line::from(format!(
            "role {} | next {}",
            lanes.mc202_role.as_deref().unwrap_or("unset"),
            lanes.mc202_pending_role.as_deref().unwrap_or("none")
        )),
        Line::from(format!(
            "phrase {} | gen {}",
            lanes.mc202_phrase_ref.as_deref().unwrap_or("unset"),
            if lanes.mc202_pending_answer_generation {
                "queued answer"
            } else if lanes.mc202_pending_follower_generation {
                "queued"
            } else {
                "idle"
            }
        )),
        Line::from(format!(
            "touch {:.2} | last {}",
            shell.app.jam_view.macros.mc202_touch, last_mc202_action
        )),
        Line::from(format!(
            "render {} | {}",
            shell.app.runtime_view.mc202_render_routing,
            shell.app.runtime_view.mc202_render_mix_summary
        )),
        Line::from(format!("diagnostic {}", mc202_pending_role_label(shell))),
    ]
}

fn w30_log_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let lanes = &shell.app.jam_view.lanes;
    let recent = last_committed_w30_action(shell);
    let recent_label = recent
        .map(|action| short_w30_action_label(&action.command))
        .unwrap_or("none");
    let lineage_active = w30_resample_lineage_active(shell);
    let slice_pool_relevant = w30_slice_pool_relevant(shell);

    vec![
        Line::from(format!(
            "bank {}/{}",
            lanes.w30_active_bank.as_deref().unwrap_or("unset"),
            lanes.w30_focused_pad.as_deref().unwrap_or("unset")
        )),
        Line::from(format!(
            "cue {} | {recent_label}",
            w30_pending_cue_label(shell)
        )),
        Line::from(format!("prev {}", w30_preview_log_compact(shell))),
        Line::from(if lineage_active {
            format!("tapmix {}", w30_resample_mix_log_compact(shell))
        } else {
            format!(
                "mix {} {}",
                w30_mix_log_compact(shell),
                w30_operation_status_compact(shell),
            )
        }),
        if lineage_active {
            Line::from(w30_resample_log_focus_compact(shell))
        } else {
            if slice_pool_relevant {
                Line::from(format!("pool {}", w30_slice_pool_log_compact(shell)))
            } else {
                Line::from(w30_capture_log_compact(shell))
            }
        },
    ]
}

fn w30_preview_mode_profile_compact(shell: &JamShellState) -> String {
    let render = &shell.app.runtime.w30_preview;
    let mode = match render.mode {
        W30PreviewRenderMode::Idle => "idle",
        W30PreviewRenderMode::LiveRecall => "recall",
        W30PreviewRenderMode::RawCaptureAudition => "audition raw",
        W30PreviewRenderMode::PromotedAudition => "audition",
    };
    let profile = match render.source_profile {
        None => "unset",
        Some(riotbox_audio::w30::W30PreviewSourceProfile::PinnedRecall) => "pinned",
        Some(riotbox_audio::w30::W30PreviewSourceProfile::PromotedRecall) => "promoted",
        Some(riotbox_audio::w30::W30PreviewSourceProfile::SlicePoolBrowse) => "browse",
        Some(riotbox_audio::w30::W30PreviewSourceProfile::RawCaptureAudition) => "raw",
        Some(riotbox_audio::w30::W30PreviewSourceProfile::PromotedAudition) => "audition",
    };

    if matches!(render.mode, W30PreviewRenderMode::RawCaptureAudition) {
        return format!(
            "{mode}/{}",
            w30_preview_source_suffix(render).unwrap_or("fallback")
        );
    }

    if matches!(render.mode, W30PreviewRenderMode::PromotedAudition) {
        return format!(
            "{mode}/{}",
            w30_preview_source_suffix(render).unwrap_or("fallback")
        );
    }

    if matches!(render.mode, W30PreviewRenderMode::LiveRecall) {
        return format!(
            "{mode}/{profile}/{}",
            w30_preview_source_suffix(render).unwrap_or("fallback")
        );
    }

    format!("{mode}/{profile}")
}

fn w30_preview_log_compact(shell: &JamShellState) -> String {
    let render = &shell.app.runtime.w30_preview;
    if matches!(render.mode, W30PreviewRenderMode::RawCaptureAudition) {
        return format!(
            "raw/{}",
            w30_preview_source_suffix(render).unwrap_or("fallback")
        );
    }

    if matches!(render.mode, W30PreviewRenderMode::PromotedAudition) {
        return format!(
            "audition/{}",
            w30_preview_source_suffix(render).unwrap_or("fallback")
        );
    }

    if matches!(render.mode, W30PreviewRenderMode::LiveRecall) {
        return format!(
            "recall/{}",
            w30_preview_source_suffix(render).unwrap_or("fallback")
        );
    }

    w30_preview_mode_profile_compact(shell)
}

fn w30_preview_source_suffix(
    render: &riotbox_audio::w30::W30PreviewRenderState,
) -> Option<&'static str> {
    if matches!(render.mode, W30PreviewRenderMode::Idle) {
        return None;
    }

    if render.source_window_preview.is_some() {
        Some("src")
    } else {
        Some("fallback")
    }
}

fn w30_preview_source_readiness(shell: &JamShellState) -> Option<&'static str> {
    let render = &shell.app.runtime.w30_preview;
    if matches!(render.mode, W30PreviewRenderMode::RawCaptureAudition) {
        return match w30_preview_source_suffix(render)? {
            "src" => Some("source-backed"),
            "fallback" => Some("fallback"),
            _ => None,
        };
    }

    if render.source_window_preview.is_some() {
        Some("source-backed")
    } else {
        None
    }
}

fn w30_target_compact(shell: &JamShellState) -> String {
    format!(
        "{}/{}",
        shell
            .app
            .jam_view
            .lanes
            .w30_active_bank
            .as_deref()
            .unwrap_or("unset"),
        shell
            .app
            .jam_view
            .lanes
            .w30_focused_pad
            .as_deref()
            .unwrap_or("unset")
    )
}

fn w30_resample_tap_compact(shell: &JamShellState) -> String {
    let tap = &shell.app.runtime.w30_resample_tap;
    if matches!(tap.mode, riotbox_audio::w30::W30ResampleTapMode::Idle) {
        return "idle/silent".into();
    }

    let profile = match tap.source_profile {
        None => "unset",
        Some(riotbox_audio::w30::W30ResampleTapSourceProfile::RawCapture) => "raw",
        Some(riotbox_audio::w30::W30ResampleTapSourceProfile::PromotedCapture) => "promoted",
        Some(riotbox_audio::w30::W30ResampleTapSourceProfile::PinnedCapture) => "pinned",
    };

    format!("ready/{profile} g{}", tap.generation_depth)
}

fn w30_capture_lineage_compact(shell: &JamShellState) -> String {
    let Some(capture_id) = shell
        .app
        .session
        .runtime_state
        .lane_state
        .w30
        .last_capture
        .as_ref()
    else {
        return "lineage none".into();
    };

    let Some(capture) = shell
        .app
        .session
        .captures
        .iter()
        .find(|capture| &capture.capture_id == capture_id)
    else {
        return format!("lineage missing {capture_id}");
    };

    let lineage_chain = if capture.lineage_capture_refs.is_empty() {
        capture.capture_id.to_string()
    } else {
        format!(
            "{}>{}",
            capture
                .lineage_capture_refs
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(">"),
            capture.capture_id
        )
    };

    format!("{lineage_chain} | g{}", capture.resample_generation_depth)
}

fn w30_resample_route_compact(shell: &JamShellState) -> &'static str {
    match shell.app.runtime.w30_resample_tap.routing {
        riotbox_audio::w30::W30ResampleTapRouting::Silent => "silent",
        riotbox_audio::w30::W30ResampleTapRouting::InternalCaptureTap => "internal",
    }
}

fn w30_resample_source_compact(shell: &JamShellState) -> String {
    let tap = &shell.app.runtime.w30_resample_tap;
    match tap.source_capture_id.as_deref() {
        Some(capture_id) => format!(
            "src {capture_id} g{}/l{}",
            tap.generation_depth, tap.lineage_capture_count
        ),
        None => format!(
            "src unset g{}/l{}",
            tap.generation_depth, tap.lineage_capture_count
        ),
    }
}

fn w30_resample_log_focus_compact(shell: &JamShellState) -> String {
    let tap = &shell.app.runtime.w30_resample_tap;
    let capture_id = tap.source_capture_id.as_deref().unwrap_or("unset");
    let route = match tap.routing {
        riotbox_audio::w30::W30ResampleTapRouting::Silent => "sil",
        riotbox_audio::w30::W30ResampleTapRouting::InternalCaptureTap => "int",
    };

    format!(
        "tap {capture_id} g{}/l{} {route}",
        tap.generation_depth, tap.lineage_capture_count
    )
}

fn w30_resample_lineage_active(shell: &JamShellState) -> bool {
    let tap = &shell.app.runtime.w30_resample_tap;
    tap.generation_depth > 0 || tap.lineage_capture_count > 0
}

fn w30_mix_log_compact(shell: &JamShellState) -> String {
    format!(
        "{:.2}/{:.2}",
        shell.app.runtime.w30_preview.music_bus_level, shell.app.runtime.w30_preview.grit_level
    )
}

fn w30_resample_mix_log_compact(shell: &JamShellState) -> String {
    format!(
        "{:.2}/{:.2}",
        shell.app.runtime.w30_resample_tap.music_bus_level,
        shell.app.runtime.w30_resample_tap.grit_level
    )
}

fn w30_capture_compact(shell: &JamShellState) -> String {
    shell
        .app
        .session
        .runtime_state
        .lane_state
        .w30
        .last_capture
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| "none".into())
}

fn w30_capture_log_compact(shell: &JamShellState) -> String {
    let capture_id = w30_capture_compact(shell);
    if let Some(source_window) = w30_latest_capture_source_window_compact(shell) {
        source_window
    } else {
        format!("cap {capture_id} | {}", w30_trigger_compact(shell))
    }
}

fn w30_latest_capture_source_window_compact(shell: &JamShellState) -> Option<String> {
    let capture_id = shell
        .app
        .session
        .runtime_state
        .lane_state
        .w30
        .last_capture
        .as_ref()?;
    let capture = shell
        .app
        .session
        .captures
        .iter()
        .find(|capture| &capture.capture_id == capture_id)?;
    let source_window = capture.source_window.as_ref()?;

    Some(format_source_window_log_compact(source_window))
}

fn format_source_window_span(source_window: &riotbox_core::session::CaptureSourceWindow) -> String {
    format!(
        "{:.2}-{:.2}s",
        source_window.start_seconds, source_window.end_seconds
    )
}

fn format_source_window_log_compact(
    source_window: &riotbox_core::session::CaptureSourceWindow,
) -> String {
    format!(
        "win {} {}",
        format_source_window_span(source_window),
        source_window.source_id
    )
}

fn current_w30_slice_pool(shell: &JamShellState) -> Vec<&riotbox_core::session::CaptureRef> {
    let Some((active_bank, focused_pad)) = current_w30_lane_target(shell) else {
        return Vec::new();
    };

    shell
        .app
        .session
        .captures
        .iter()
        .filter(|capture| {
            matches!(
                capture.assigned_target.as_ref(),
                Some(riotbox_core::session::CaptureTarget::W30Pad { bank_id, pad_id })
                    if bank_id.as_str() == active_bank && pad_id.as_str() == focused_pad
            )
        })
        .collect()
}

fn current_w30_slice_pool_position(
    shell: &JamShellState,
    pool: &[&riotbox_core::session::CaptureRef],
) -> Option<usize> {
    let last_capture = shell
        .app
        .session
        .runtime_state
        .lane_state
        .w30
        .last_capture
        .as_ref()?;
    pool.iter()
        .position(|capture| &capture.capture_id == last_capture)
}

fn w30_slice_pool_relevant(shell: &JamShellState) -> bool {
    shell
        .app
        .jam_view
        .lanes
        .w30_pending_slice_pool_capture_id
        .is_some()
        || current_w30_slice_pool(shell).len() > 1
}

fn w30_slice_pool_compact(shell: &JamShellState) -> String {
    let pool = current_w30_slice_pool(shell);
    if pool.is_empty() {
        return "none".into();
    }

    let current_index =
        current_w30_slice_pool_position(shell, &pool).unwrap_or_else(|| pool.len() - 1);
    let current_capture = pool[current_index].capture_id.to_string();
    let next_capture = shell
        .app
        .jam_view
        .lanes
        .w30_pending_slice_pool_capture_id
        .clone()
        .or_else(|| {
            (pool.len() > 1).then(|| {
                pool[(current_index + 1) % pool.len()]
                    .capture_id
                    .to_string()
            })
        })
        .unwrap_or_else(|| "hold".into());

    format!(
        "{current_capture} {}/{} -> {next_capture}",
        current_index + 1,
        pool.len()
    )
}

fn w30_slice_pool_log_compact(shell: &JamShellState) -> String {
    let pool = current_w30_slice_pool(shell);
    if pool.is_empty() {
        return "none".into();
    }

    let current_index =
        current_w30_slice_pool_position(shell, &pool).unwrap_or_else(|| pool.len() - 1);
    let next_capture = shell
        .app
        .jam_view
        .lanes
        .w30_pending_slice_pool_capture_id
        .clone()
        .or_else(|| {
            (pool.len() > 1).then(|| {
                pool[(current_index + 1) % pool.len()]
                    .capture_id
                    .to_string()
            })
        })
        .unwrap_or_else(|| "hold".into());

    format!("{}/{} -> {next_capture}", current_index + 1, pool.len())
}

fn w30_trigger_compact(shell: &JamShellState) -> String {
    let render = &shell.app.runtime.w30_preview;
    if render.trigger_revision == 0 {
        if matches!(render.mode, W30PreviewRenderMode::Idle) {
            "unset".into()
        } else {
            "pending".into()
        }
    } else {
        format!(
            "r{}@{:.2}",
            render.trigger_revision, render.trigger_velocity
        )
    }
}

fn w30_action_target_compact(action: &riotbox_core::action::Action) -> Option<String> {
    action
        .target
        .bank_id
        .as_ref()
        .zip(action.target.pad_id.as_ref())
        .map(|(bank_id, pad_id)| format!("{bank_id}/{pad_id}"))
}

fn current_w30_lane_target(shell: &JamShellState) -> Option<(&str, &str)> {
    let lanes = &shell.app.jam_view.lanes;
    Some((
        lanes.w30_active_bank.as_deref()?,
        lanes.w30_focused_pad.as_deref()?,
    ))
}

fn latest_committed_w30_action_for_current_target(
    shell: &JamShellState,
    command: riotbox_core::action::ActionCommand,
) -> Option<&riotbox_core::action::Action> {
    let (active_bank, focused_pad) = current_w30_lane_target(shell)?;
    shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .find(|action| {
            action.status == riotbox_core::action::ActionStatus::Committed
                && action.command == command
                && action
                    .target
                    .bank_id
                    .as_ref()
                    .map(|bank_id| bank_id.as_str())
                    == Some(active_bank)
                && action.target.pad_id.as_ref().map(|pad_id| pad_id.as_str()) == Some(focused_pad)
        })
}

fn w30_bank_manager_compact(shell: &JamShellState) -> String {
    if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_bank_swap_target
        .as_deref()
    {
        format!("next {target}")
    } else if let Some(target) = latest_committed_w30_action_for_current_target(
        shell,
        riotbox_core::action::ActionCommand::W30SwapBank,
    )
    .and_then(w30_action_target_compact)
    {
        target
    } else {
        "idle".into()
    }
}

fn w30_bank_manager_status_compact(shell: &JamShellState) -> &'static str {
    if shell
        .app
        .jam_view
        .lanes
        .w30_pending_bank_swap_target
        .is_some()
    {
        "next-swap"
    } else if latest_committed_w30_action_for_current_target(
        shell,
        riotbox_core::action::ActionCommand::W30SwapBank,
    )
    .is_some()
    {
        "swap"
    } else {
        "idle"
    }
}

fn w30_damage_profile_compact(shell: &JamShellState) -> String {
    if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_damage_profile_target
        .as_deref()
    {
        format!("next {target}")
    } else if let Some(target) = latest_committed_w30_action_for_current_target(
        shell,
        riotbox_core::action::ActionCommand::W30ApplyDamageProfile,
    )
    .and_then(w30_action_target_compact)
    {
        target
    } else {
        "idle".into()
    }
}

fn w30_damage_profile_status_compact(shell: &JamShellState) -> &'static str {
    if shell
        .app
        .jam_view
        .lanes
        .w30_pending_damage_profile_target
        .is_some()
    {
        "next-shred"
    } else if latest_committed_w30_action_for_current_target(
        shell,
        riotbox_core::action::ActionCommand::W30ApplyDamageProfile,
    )
    .is_some()
    {
        "shred"
    } else {
        "idle"
    }
}

fn w30_loop_freeze_compact(shell: &JamShellState) -> String {
    if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_loop_freeze_target
        .as_deref()
    {
        format!("next {target}")
    } else if let Some(target) = latest_committed_w30_action_for_current_target(
        shell,
        riotbox_core::action::ActionCommand::W30LoopFreeze,
    )
    .and_then(w30_action_target_compact)
    {
        target
    } else {
        "idle".into()
    }
}

fn w30_loop_freeze_status_compact(shell: &JamShellState) -> &'static str {
    if shell
        .app
        .jam_view
        .lanes
        .w30_pending_loop_freeze_target
        .is_some()
    {
        "next-freeze"
    } else if latest_committed_w30_action_for_current_target(
        shell,
        riotbox_core::action::ActionCommand::W30LoopFreeze,
    )
    .is_some()
    {
        "freeze"
    } else {
        "idle"
    }
}

fn w30_operation_status_compact(shell: &JamShellState) -> String {
    let operations = [
        w30_bank_manager_status_compact(shell),
        w30_damage_profile_status_compact(shell),
        w30_loop_freeze_status_compact(shell),
    ]
    .into_iter()
    .filter(|status| *status != "idle")
    .collect::<Vec<_>>();

    if operations.is_empty() {
        "idle".into()
    } else {
        operations.join("+")
    }
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

fn next_scene_jump_suggestion(shell: &JamShellState) -> String {
    let Some(scene_id) = shell.app.jam_view.scene.next_scene.as_deref() else {
        match shell.app.jam_view.scene.scene_jump_availability {
            SceneJumpAvailabilityView::WaitingForMoreScenes => {
                return "[y] jump waits for 2 scenes".into();
            }
            SceneJumpAvailabilityView::Ready | SceneJumpAvailabilityView::Unknown => {
                return "[y] jump".into();
            }
        }
    };

    let scene = compact_scene_label(scene_id);
    match compact_energy_delta_label(
        shell.app.jam_view.scene.active_scene_energy.as_deref(),
        shell.app.jam_view.scene.next_scene_energy.as_deref(),
    ) {
        Some(direction) => format!("[y] jump {scene} ({direction})"),
        None => format!("[y] jump {scene}"),
    }
}

fn next_scene_target_compact_label(shell: &JamShellState) -> String {
    let Some(scene_id) = shell.app.jam_view.scene.next_scene.as_deref() else {
        match shell.app.jam_view.scene.scene_jump_availability {
            SceneJumpAvailabilityView::WaitingForMoreScenes => {
                return "waits for 2 scenes".into();
            }
            SceneJumpAvailabilityView::Ready | SceneJumpAvailabilityView::Unknown => {
                return "none".into();
            }
        }
    };

    let scene = compact_scene_label(scene_id);
    if let Some(energy) = shell.app.jam_view.scene.next_scene_energy.as_deref() {
        return format!("{scene}/{}", compact_energy_label(energy));
    }
    scene
}

fn current_scene_compact_label(shell: &JamShellState) -> String {
    let scene_id = current_scene_id(shell).unwrap_or_else(|| "none".into());

    compact_scene_label(scene_id.as_str())
}

fn current_scene_target_compact_label(shell: &JamShellState) -> String {
    let scene = current_scene_compact_label(shell);
    if let Some(energy) = shell.app.jam_view.scene.active_scene_energy.as_deref() {
        return format!("{scene}/{}", compact_energy_label(energy));
    }
    scene
}

fn current_scene_id(shell: &JamShellState) -> Option<String> {
    shell.app.jam_view.scene.active_scene.clone()
}

fn scene_restore_contrast_line(shell: &JamShellState) -> String {
    let current_scene = current_scene_compact_label(shell);
    let current_energy = shell
        .app
        .jam_view
        .scene
        .active_scene_energy
        .as_deref()
        .map(compact_energy_label)
        .unwrap_or("unk");
    let restore_scene = compact_scene_label(restore_scene_label(shell).as_str());
    let restore_energy = shell
        .app
        .jam_view
        .scene
        .restore_scene_energy
        .as_deref()
        .map(compact_energy_label);

    format!(
        "live {current_scene}/{current_energy} <> restore {} | ghost {}",
        match restore_energy {
            Some(restore_energy) => format!("{restore_scene}/{restore_energy}"),
            None => restore_scene,
        },
        ghost_label(shell)
    )
}

fn compact_scene_label(scene_id: &str) -> String {
    let mut parts = scene_id.splitn(3, '-');
    match (parts.next(), parts.next(), parts.next()) {
        (Some("scene"), Some(index), Some(label))
            if index.chars().all(|ch| ch.is_ascii_digit()) =>
        {
            label.to_string()
        }
        _ => scene_id.to_string(),
    }
}

fn restore_scene_label(shell: &JamShellState) -> String {
    shell
        .app
        .jam_view
        .scene
        .restore_scene
        .clone()
        .unwrap_or_else(|| "none".into())
}

fn scene_energy_label_for_scene_id<'a>(
    shell: &'a JamShellState,
    scene_id: &str,
) -> Option<&'a str> {
    let graph = shell.app.source_graph.as_ref()?;
    let scene_index = parse_projected_scene_index(scene_id)?;
    let mut sections = graph.sections.iter().collect::<Vec<_>>();
    sections.sort_by(|left, right| {
        left.bar_start
            .cmp(&right.bar_start)
            .then(left.bar_end.cmp(&right.bar_end))
            .then(left.section_id.as_str().cmp(right.section_id.as_str()))
    });
    sections
        .get(scene_index)
        .map(|section| energy_label(section))
}

fn parse_projected_scene_index(scene_id: &str) -> Option<usize> {
    let mut parts = scene_id.splitn(3, '-');
    match (parts.next(), parts.next()) {
        (Some("scene"), Some(index)) => index.parse::<usize>().ok()?.checked_sub(1),
        _ => None,
    }
}

fn energy_rank(label: &str) -> Option<u8> {
    match label {
        "low" => Some(0),
        "medium" => Some(1),
        "high" => Some(2),
        "peak" => Some(3),
        _ => None,
    }
}

fn compact_energy_label(label: &str) -> &'static str {
    match label {
        "low" => "low",
        "medium" => "med",
        "high" => "high",
        "peak" => "peak",
        _ => "unk",
    }
}

fn energy_delta_label(from: Option<&str>, to: Option<&str>) -> Option<&'static str> {
    let from = energy_rank(from?)?;
    let to = energy_rank(to?)?;

    Some(match to.cmp(&from) {
        std::cmp::Ordering::Greater => "energy rise",
        std::cmp::Ordering::Less => "energy drop",
        std::cmp::Ordering::Equal => "energy hold",
    })
}

fn compact_energy_delta_label(from: Option<&str>, to: Option<&str>) -> Option<&'static str> {
    let from = energy_rank(from?)?;
    let to = energy_rank(to?)?;

    Some(match to.cmp(&from) {
        std::cmp::Ordering::Greater => "rise",
        std::cmp::Ordering::Less => "drop",
        std::cmp::Ordering::Equal => "hold",
    })
}

fn restore_scene_energy_direction_label(shell: &JamShellState) -> Option<&'static str> {
    compact_energy_delta_label(
        shell.app.jam_view.scene.active_scene_energy.as_deref(),
        shell.app.jam_view.scene.restore_scene_energy.as_deref(),
    )
}

fn restore_scene_now_compact_label(shell: &JamShellState) -> String {
    let scene = compact_scene_label(restore_scene_label(shell).as_str());
    match restore_scene_energy_direction_label(shell) {
        Some(direction) => format!("{scene} now ({direction})"),
        None => format!("{scene} now"),
    }
}

fn restore_scene_target_compact_label(shell: &JamShellState) -> String {
    let scene = compact_scene_label(restore_scene_label(shell).as_str());
    if let Some(energy) = shell.app.jam_view.scene.restore_scene_energy.as_deref() {
        return format!("{scene}/{}", compact_energy_label(energy));
    }
    scene
}

fn quantization_boundary_label(quantization: riotbox_core::action::Quantization) -> &'static str {
    match quantization {
        riotbox_core::action::Quantization::Immediate => "immediately",
        riotbox_core::action::Quantization::NextBeat => "next beat",
        riotbox_core::action::Quantization::NextHalfBar => "next half bar",
        riotbox_core::action::Quantization::NextBar => "next bar",
        riotbox_core::action::Quantization::NextPhrase => "next phrase",
        riotbox_core::action::Quantization::NextScene => "next scene",
    }
}

fn pending_scene_transition(shell: &JamShellState) -> Option<(&'static str, String, String)> {
    shell
        .app
        .queue
        .pending_actions()
        .iter()
        .find(|action| {
            matches!(
                action.command,
                riotbox_core::action::ActionCommand::SceneLaunch
                    | riotbox_core::action::ActionCommand::SceneRestore
            )
        })
        .and_then(|action| {
            let label = match action.command {
                riotbox_core::action::ActionCommand::SceneLaunch => "launch",
                riotbox_core::action::ActionCommand::SceneRestore => "restore",
                _ => unreachable!("scene transition scan only matches launch and restore"),
            };
            action
                .target
                .scene_id
                .as_ref()
                .map(ToString::to_string)
                .or_else(|| match &action.params {
                    riotbox_core::action::ActionParams::Scene {
                        scene_id: Some(scene_id),
                    } => Some(scene_id.to_string()),
                    _ => None,
                })
                .map(|scene_id| {
                    (
                        label,
                        scene_id,
                        quantization_boundary_label(action.quantization).into(),
                    )
                })
        })
}

fn scene_pending_line(shell: &JamShellState) -> Line<'static> {
    let Some((label, scene_id, boundary)) = pending_scene_transition(shell) else {
        return Line::from(Span::styled("scene transition idle", style_low_emphasis()));
    };

    let mut spans = vec![
        Span::styled(label, style_pending_cue()),
        Span::styled(" -> ", style_low_emphasis()),
        Span::styled(scene_id.clone(), style_pending_detail()),
        Span::styled(" @ ", style_low_emphasis()),
        Span::styled(boundary, style_pending_cue()),
    ];

    if let Some(energy_delta) = energy_delta_label(
        shell.app.jam_view.scene.active_scene_energy.as_deref(),
        scene_energy_label_for_scene_id(shell, scene_id.as_str()),
    ) {
        spans.push(Span::styled(" | ", style_low_emphasis()));
        spans.push(Span::styled(energy_delta, style_confirmation_strong()));
    }

    Line::from(spans)
}

fn landed_scene_energy_delta(shell: &JamShellState, command: &str) -> Option<&'static str> {
    if !matches!(command, "scene.launch" | "scene.restore") {
        return None;
    }

    energy_delta_label(
        shell.app.jam_view.scene.restore_scene_energy.as_deref(),
        shell.app.jam_view.scene.active_scene_energy.as_deref(),
    )
}

fn scene_commit_pulse_line(shell: &JamShellState) -> Option<Line<'static>> {
    pending_scene_transition(shell)?;
    let transport = &shell.app.runtime.transport;
    let countdown = scene_countdown_cue(transport.beat_index);

    Some(timing_rail_line(
        "pulse",
        countdown,
        None,
        format!(
            "b{} | b{} | p{}",
            transport.beat_index, transport.bar_index, transport.phrase_index
        ),
    ))
}

fn queued_timing_rail_line(shell: &JamShellState) -> Option<Line<'static>> {
    if let Some(scene_line) = scene_commit_pulse_line(shell) {
        return Some(scene_line);
    }

    let pending_actions = shell.app.queue.pending_actions();
    let action = pending_actions.first()?;
    let transport = &shell.app.runtime.transport;
    let countdown = quantization_countdown_cue(
        action.quantization,
        transport.beat_index,
        transport.bar_index,
    );

    Some(timing_rail_line(
        "wait",
        countdown,
        Some(quantization_boundary_label(action.quantization)),
        format!(
            "| b{} | bar{} | p{}",
            transport.beat_index, transport.bar_index, transport.phrase_index
        ),
    ))
}

fn timing_rail_line(
    prefix: &'static str,
    countdown: String,
    boundary: Option<&'static str>,
    tail: String,
) -> Line<'static> {
    let mut spans = vec![
        Span::styled(format!("{prefix} "), style_low_emphasis()),
        Span::styled(countdown, style_pending_cue()),
    ];

    if let Some(boundary) = boundary {
        spans.push(Span::styled(" ", style_low_emphasis()));
        spans.push(Span::styled(boundary, style_pending_cue()));
    }

    spans.push(Span::styled(format!(" {tail}"), style_low_emphasis()));

    Line::from(spans)
}

fn scene_countdown_cue(beat_index: u64) -> String {
    let slot = ((beat_index.saturating_sub(1) % 4) + 1) as usize;
    let mut chars = ['-'; 4];
    for ch in chars.iter_mut().take(slot.saturating_sub(1)) {
        *ch = '=';
    }
    chars[slot - 1] = '>';
    format!("[{}]", chars.iter().collect::<String>())
}

fn quantization_countdown_cue(
    quantization: riotbox_core::action::Quantization,
    beat_index: u64,
    bar_index: u64,
) -> String {
    match quantization {
        riotbox_core::action::Quantization::Immediate => "[now]".into(),
        riotbox_core::action::Quantization::NextBeat => "[>]".into(),
        riotbox_core::action::Quantization::NextHalfBar => {
            let slot = (((beat_index.saturating_sub(1) % 4) / 2) + 1) as usize;
            ascii_countdown(slot, 2)
        }
        riotbox_core::action::Quantization::NextBar => scene_countdown_cue(beat_index),
        riotbox_core::action::Quantization::NextPhrase => {
            let slot = ((bar_index.saturating_sub(1) % 8) + 1) as usize;
            ascii_countdown(slot, 8)
        }
        riotbox_core::action::Quantization::NextScene => "[scene]".into(),
    }
}

fn ascii_countdown(slot: usize, width: usize) -> String {
    let slot = slot.clamp(1, width);
    let mut chars = vec!['-'; width];
    for ch in chars.iter_mut().take(slot.saturating_sub(1)) {
        *ch = '=';
    }
    chars[slot - 1] = '>';
    format!("[{}]", chars.iter().collect::<String>())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FirstRunOnrampStage {
    Start,
    QueuedFirstMove,
    FirstResult,
}

fn first_run_onramp_stage(shell: &JamShellState) -> Option<FirstRunOnrampStage> {
    if !shell.first_run_onramp {
        return None;
    }

    let committed_count = shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .filter(|action| action.status == riotbox_core::action::ActionStatus::Committed)
        .count();
    let has_pending = !shell.app.jam_view.pending_actions.is_empty();
    let capture_count = shell.app.jam_view.capture.capture_count;

    if capture_count > 0 || committed_count > 1 {
        return None;
    }

    if committed_count == 0 {
        return Some(if has_pending {
            FirstRunOnrampStage::QueuedFirstMove
        } else {
            FirstRunOnrampStage::Start
        });
    }

    Some(FirstRunOnrampStage::FirstResult)
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

fn capture_readiness_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let pending_capture_count = shell.app.jam_view.capture.pending_capture_count;
    let bank = shell
        .app
        .jam_view
        .lanes
        .w30_active_bank
        .as_deref()
        .unwrap_or("unset");

    vec![
        Line::from(format!(
            "transport {} | beat {:.1}",
            transport_label(shell),
            shell.app.jam_view.transport.position_beats
        )),
        Line::from(format!("pending capture actions {pending_capture_count}")),
        Line::from(format!("w30 bank {bank}")),
        Line::from(format!(
            "last lane capture {}",
            shell
                .app
                .jam_view
                .capture
                .last_capture_id
                .as_deref()
                .unwrap_or("none")
        )),
    ]
}

fn capture_latest_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let capture = &shell.app.jam_view.capture;
    vec![
        Line::from(format!("captures total {}", capture.capture_count)),
        Line::from(format!(
            "pinned {} | promoted {}",
            capture.pinned_capture_count, capture.promoted_capture_count
        )),
        Line::from(format!("hear {}", capture_heard_path_label(shell))),
        Line::from(format!(
            "latest {}",
            capture.last_capture_id.as_deref().unwrap_or("none")
        )),
        Line::from(format!(
            "target {}",
            capture
                .last_capture_target
                .as_deref()
                .unwrap_or("unassigned")
        )),
        Line::from(format!("origin refs {}", capture.last_capture_origin_count)),
        Line::from(
            capture
                .last_promotion_result
                .clone()
                .or_else(|| capture.last_capture_notes.clone())
                .unwrap_or_else(|| "no capture note yet".into()),
        ),
    ]
}

fn capture_do_next_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let capture = &shell.app.jam_view.capture;
    let handoff_readiness = capture_handoff_readiness_label(shell);
    if let Some(lines) = pending_capture_do_next_lines(capture, handoff_readiness) {
        return lines;
    }
    if let Some(lines) = pending_w30_audition_do_next_lines(shell) {
        return lines;
    }

    let Some(last_capture_id) = capture.last_capture_id.as_deref() else {
        return vec![
            Line::from("1 [c] capture phrase"),
            Line::from("2 [p] promote keeper"),
            Line::from("3 [w] hit promoted pad"),
            Line::from("use Log to confirm"),
        ];
    };

    match (
        capture.last_capture_target_kind,
        capture.last_capture_target.as_deref(),
    ) {
        (Some(CaptureTargetKindView::W30Pad), Some(target)) => vec![
            Line::from(format!("hear now: [w] hit {target} ({handoff_readiness})")),
            Line::from("or [o] audition same pad"),
            Line::from("[b]/[s] browse or swap"),
            Line::from(format!("source {last_capture_id}")),
        ],
        (Some(CaptureTargetKindView::Scene), Some(target)) => vec![
            Line::from(format!("scene target {target}")),
            Line::from("use Jam scene controls"),
            Line::from("[2] confirm action trail"),
            Line::from(format!("source {last_capture_id}")),
        ],
        _ => vec![
            Line::from(format!("1 hear it: [o] audition raw {last_capture_id}")),
            Line::from(format!("2 keep it: [p] promote {last_capture_id}")),
            Line::from(format!(
                "3 play it: [w] hit after promote ({handoff_readiness})"
            )),
            Line::from("[2] confirm result"),
        ],
    }
}

fn capture_handoff_readiness_label(shell: &JamShellState) -> &'static str {
    match shell.app.jam_view.capture.last_capture_handoff_readiness {
        Some(CaptureHandoffReadinessView::Source) => "src",
        Some(CaptureHandoffReadinessView::Fallback) | None => "fallback",
    }
}

fn pending_w30_audition_do_next_lines(shell: &JamShellState) -> Option<Vec<Line<'static>>> {
    let pending = shell.app.jam_view.lanes.w30_pending_audition.as_ref()?;

    match pending.kind {
        W30PendingAuditionKind::RawCapture => Some(vec![
            capture_pending_intent_line(format!(
                "queued [o] audition raw @ {}",
                pending.quantization
            )),
            capture_pending_detail_line("wait, then hear raw preview"),
            capture_pending_detail_line(format!("target {}", pending.target)),
            capture_pending_detail_line("[2] confirm audition"),
        ]),
        W30PendingAuditionKind::Promoted => Some(vec![
            capture_pending_intent_line(format!(
                "queued [o] audition pad @ {}",
                pending.quantization
            )),
            capture_pending_detail_line("wait, then hear promoted preview"),
            capture_pending_detail_line(format!("target {}", pending.target)),
            capture_pending_detail_line("[2] confirm audition"),
        ]),
    }
}

fn pending_capture_do_next_lines(
    capture: &riotbox_core::view::jam::CaptureSummaryView,
    handoff_readiness: &'static str,
) -> Option<Vec<Line<'static>>> {
    let pending = capture.pending_capture_items.first()?;

    if matches!(
        pending.command.as_str(),
        "capture.now" | "capture.loop" | "capture.bar_group" | "w30.capture_to_pad"
    ) {
        return Some(vec![
            capture_pending_intent_line(format!("queued [c] capture @ {}", pending.quantization)),
            capture_pending_detail_line("wait for commit"),
            capture_pending_detail_line("then [o] audition raw or [p] promote"),
            capture_pending_detail_line("[2] confirm capture"),
        ]);
    }

    if pending.command == "promote.capture_to_pad" {
        return Some(vec![
            capture_pending_intent_line(format!("queued [p] promote @ {}", pending.quantization)),
            capture_pending_detail_line(format!(
                "wait, then hear with [w] hit ({handoff_readiness})"
            )),
            capture_pending_detail_line(format!("target {}", pending.target)),
            capture_pending_detail_line("[2] confirm promotion"),
        ]);
    }

    if pending.command == "promote.capture_to_scene" {
        return Some(vec![
            capture_pending_intent_line(format!("queued scene promote @ {}", pending.quantization)),
            capture_pending_detail_line("wait for scene target"),
            capture_pending_detail_line(format!("target {}", pending.target)),
            capture_pending_detail_line("[2] confirm promotion"),
        ]);
    }

    if pending.command == "w30.loop_freeze" || pending.command == "promote.resample" {
        return Some(vec![
            capture_pending_intent_line(format!("queued W-30 reshape @ {}", pending.quantization)),
            capture_pending_detail_line("wait for phrase seam"),
            capture_pending_detail_line(format!("target {}", pending.target)),
            capture_pending_detail_line("[2] confirm result"),
        ]);
    }

    None
}

fn capture_pending_intent_line(message: impl Into<String>) -> Line<'static> {
    Line::from(Span::styled(message.into(), style_pending_cue()))
}

fn capture_pending_detail_line(message: impl Into<String>) -> Line<'static> {
    Line::from(Span::styled(message.into(), style_pending_detail()))
}

fn capture_provenance_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let lines = &shell.app.jam_view.capture.latest_capture_provenance_lines;
    if lines.is_empty() {
        return vec![Line::from("no captured material yet")];
    }

    lines.iter().cloned().map(Line::from).collect()
}

fn pending_capture_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let pending = &shell.app.jam_view.capture.pending_capture_items;
    if pending.is_empty() {
        return vec![Line::from("no queued capture actions")];
    }

    let action = &pending[0];
    let mut lines = vec![
        Line::from(format!("next {} {}", action.actor, action.command)),
        Line::from(format!(
            "when {} | target {}",
            action.quantization, action.target
        )),
    ];
    if let Some(explanation) = &action.explanation {
        lines.push(Line::from(format!("note {explanation}")));
    }

    let overflow_count = pending.len().saturating_sub(1);
    if overflow_count > 0 {
        lines.push(Line::from(format!("+{overflow_count} more in [2] Log")));
    }

    lines
}

fn recent_capture_items(shell: &JamShellState) -> Vec<ListItem<'static>> {
    let rows = &shell.app.jam_view.capture.recent_capture_rows;
    if rows.is_empty() {
        return vec![ListItem::new("no captures stored yet")];
    }

    rows.iter().cloned().map(ListItem::new).collect()
}

fn capture_routing_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let latest_promoted = shell
        .app
        .jam_view
        .capture
        .latest_w30_promoted_capture_label
        .as_deref()
        .unwrap_or("none");
    let pending_w30 = w30_pending_cue_label(shell);
    let bank_or_pool_line = if w30_slice_pool_relevant(shell) {
        format!(
            "bank/pad {} | pool {}",
            w30_target_compact(shell),
            w30_slice_pool_compact(shell)
        )
    } else {
        format!(
            "bank/pad {} | mgr {}",
            w30_target_compact(shell),
            w30_bank_manager_compact(shell)
        )
    };
    let mut lines = vec![
        Line::from(format!("pending W-30 cue {pending_w30}")),
        Line::from(bank_or_pool_line),
        Line::from({
            let mut line = format!(
                "preview {} | {}",
                shell.app.runtime_view.w30_preview_mode,
                shell.app.runtime_view.w30_preview_mix_summary,
            );
            if let Some(readiness) = w30_preview_source_readiness(shell) {
                line.push_str(" | ");
                line.push_str(readiness);
            }
            line
        }),
        Line::from(format!(
            "forge {} | tap {}",
            w30_damage_profile_compact(shell),
            w30_resample_tap_compact(shell),
        )),
    ];

    if w30_resample_lineage_active(shell) {
        lines.push(Line::from(format!(
            "tap {} | route {}",
            w30_resample_source_compact(shell),
            w30_resample_route_compact(shell),
        )));
        lines.push(Line::from(format!(
            "tap mix {}",
            w30_resample_mix_log_compact(shell)
        )));
        lines.push(Line::from(format!(
            "freeze {}",
            w30_loop_freeze_compact(shell)
        )));
        lines.push(Line::from(format!(
            "lineage {}",
            w30_capture_lineage_compact(shell)
        )));
    } else {
        let last_target = shell
            .app
            .jam_view
            .capture
            .last_capture_target
            .as_deref()
            .unwrap_or("unassigned");
        lines.push(Line::from(format!("route {last_target}")));
        lines.push(Line::from(
            shell
                .app
                .jam_view
                .capture
                .last_promotion_result
                .clone()
                .unwrap_or_else(|| "promotion result pending".into()),
        ));
        lines.push(Line::from(format!(
            "freeze {}",
            w30_loop_freeze_compact(shell)
        )));
        lines.push(Line::from(format!("latest promoted {latest_promoted}")));
        lines.push(Line::from(format!(
            "last lane capture {}",
            shell
                .app
                .session
                .runtime_state
                .lane_state
                .w30
                .last_capture
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "none".into())
        )));
        lines.push(Line::from(format!(
            "next shell cue {}",
            capture_or_recall_cue_label(shell)
        )));
        lines.push(Line::from(
            "audition and recall stay on the shared next-bar seam",
        ));
        return lines;
    }

    lines.push(Line::from(format!("latest promoted {latest_promoted}")));
    lines
}

fn capture_heard_path_label(shell: &JamShellState) -> String {
    let capture = &shell.app.jam_view.capture;
    let Some(last_capture_id) = capture.last_capture_id.as_deref() else {
        return "[c] first, then [p]->[w]".into();
    };

    match (
        capture.last_capture_target_kind,
        capture.last_capture_target.as_deref(),
    ) {
        (Some(CaptureTargetKindView::W30Pad), Some(target)) => {
            format!(
                "{last_capture_id}->{target} [w]/[o] {}",
                capture_handoff_readiness_label(shell)
            )
        }
        (Some(CaptureTargetKindView::Scene), Some(target)) => {
            format!("{last_capture_id}->{target} ready")
        }
        (_, Some(target)) if target != "unassigned" => format!("{last_capture_id}->{target} ready"),
        _ => format!(
            "{last_capture_id} stored {} [o] raw or [p]->[w]",
            capture_handoff_readiness_label(shell)
        ),
    }
}

fn capture_or_recall_cue_label(shell: &JamShellState) -> String {
    shell
        .app
        .jam_view
        .pending_actions
        .iter()
        .find(|action| {
            matches!(
                action.command.as_str(),
                "w30.trigger_pad"
                    | "w30.step_focus"
                    | "w30.swap_bank"
                    | "w30.apply_damage_profile"
                    | "w30.loop_freeze"
                    | "w30.live_recall"
                    | "w30.audition_raw_capture"
                    | "w30.audition_promoted"
                    | "promote.resample"
            )
        })
        .or_else(|| {
            shell
                .app
                .jam_view
                .pending_actions
                .iter()
                .find(|action| is_capture_command_view(action.command.as_str()))
        })
        .map(|action| format!("{} @ {}", action.command, action.quantization))
        .unwrap_or_else(|| "no capture cue queued".into())
}

fn w30_pending_cue_label(shell: &JamShellState) -> String {
    if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_trigger_target
        .as_deref()
    {
        format!("trigger {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_focus_step_target
        .as_deref()
    {
        format!("step {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_audition
        .as_ref()
        .map(|pending| pending.target.as_str())
    {
        format!("audition {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_bank_swap_target
        .as_deref()
    {
        format!("bank {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_slice_pool_target
        .as_deref()
    {
        format!("browse {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_damage_profile_target
        .as_deref()
    {
        format!("damage shred {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_loop_freeze_target
        .as_deref()
    {
        format!("freeze {target}")
    } else if let Some(target) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_recall_target
        .as_deref()
    {
        format!("recall {target}")
    } else if let Some(capture_id) = shell
        .app
        .jam_view
        .lanes
        .w30_pending_resample_capture_id
        .as_deref()
    {
        format!("resample {capture_id}")
    } else {
        "idle".into()
    }
}

fn last_committed_w30_action(shell: &JamShellState) -> Option<&riotbox_core::action::Action> {
    shell
        .app
        .session
        .action_log
        .actions
        .iter()
        .rev()
        .find(|action| {
            action.status == riotbox_core::action::ActionStatus::Committed
                && matches!(
                    action.command,
                    riotbox_core::action::ActionCommand::W30TriggerPad
                        | riotbox_core::action::ActionCommand::W30StepFocus
                        | riotbox_core::action::ActionCommand::W30SwapBank
                        | riotbox_core::action::ActionCommand::W30BrowseSlicePool
                        | riotbox_core::action::ActionCommand::W30ApplyDamageProfile
                        | riotbox_core::action::ActionCommand::W30LoopFreeze
                        | riotbox_core::action::ActionCommand::W30LiveRecall
                        | riotbox_core::action::ActionCommand::W30AuditionRawCapture
                        | riotbox_core::action::ActionCommand::W30AuditionPromoted
                        | riotbox_core::action::ActionCommand::PromoteResample
                )
        })
}

fn short_w30_action_label(command: &riotbox_core::action::ActionCommand) -> &'static str {
    match command {
        riotbox_core::action::ActionCommand::W30TriggerPad => "trigger",
        riotbox_core::action::ActionCommand::W30StepFocus => "step",
        riotbox_core::action::ActionCommand::W30SwapBank => "bank",
        riotbox_core::action::ActionCommand::W30BrowseSlicePool => "browse",
        riotbox_core::action::ActionCommand::W30ApplyDamageProfile => "damage",
        riotbox_core::action::ActionCommand::W30LoopFreeze => "freeze",
        riotbox_core::action::ActionCommand::W30LiveRecall => "recall",
        riotbox_core::action::ActionCommand::W30AuditionRawCapture => "audition raw",
        riotbox_core::action::ActionCommand::W30AuditionPromoted => "audition",
        riotbox_core::action::ActionCommand::PromoteResample => "resample",
        _ => "other",
    }
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

fn is_capture_command_view(command: &str) -> bool {
    matches!(
        command,
        "capture.now"
            | "capture.loop"
            | "capture.bar_group"
            | "w30.capture_to_pad"
            | "promote.capture_to_pad"
            | "promote.capture_to_scene"
            | "w30.loop_freeze"
            | "promote.resample"
    )
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

fn section_compact_label(section: &Section) -> String {
    format!(
        "{} bars {}-{}",
        section_label_hint_compact(&section.label_hint),
        section.bar_start,
        section.bar_end
    )
}

fn section_label_hint_compact(label_hint: &SectionLabelHint) -> &'static str {
    match label_hint {
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
    use crate::test_support::{scene_energy_for_label, scene_label_hint};
    use riotbox_core::{
        TimestampMs,
        action::{
            Action, ActionCommand, ActionDraft, ActionParams, ActionResult, ActionStatus,
            ActionTarget, ActorType, GhostMode, Quantization, TargetScope, UndoPolicy,
        },
        ids::{ActionId, AssetId, BankId, CaptureId, PadId, SceneId, SectionId, SourceId},
        queue::ActionQueue,
        session::{SessionFile, Tr909ReinforcementModeState, Tr909TakeoverProfileState},
        source_graph::{
            AnalysisSummary, AnalysisWarning, Asset, AssetType, Candidate, CandidateType,
            DecodeProfile, EnergyClass, GraphProvenance, QualityClass, Section, SectionLabelHint,
            SourceDescriptor, SourceGraph,
        },
        transport::CommitBoundaryState,
    };
    use serde::Deserialize;

    use super::*;

    #[test]
    fn footer_keys_line_styles_top_legend_key_tokens() {
        let line = footer_keys_line("i jam inspect", "re-ingest source");
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert_eq!(
            rendered,
            "Keys: q quit | ? help | 1-4 screens | Tab switch | i inspect | space play/pause | [ ] drum | r re-ingest"
        );
        assert_eq!(line.spans[1].content.as_ref(), "q");
        assert_eq!(line.spans[1].style.fg, Some(Color::Cyan));
        assert_eq!(line.spans[4].content.as_ref(), "?");
        assert_eq!(line.spans[4].style.fg, Some(Color::Cyan));
        assert_eq!(line.spans[7].content.as_ref(), "1-4");
        assert_eq!(line.spans[7].style.fg, Some(Color::Cyan));
        assert_eq!(line.spans[10].content.as_ref(), "Tab");
        assert_eq!(line.spans[10].style.fg, Some(Color::Cyan));
        assert_eq!(line.spans[13].content.as_ref(), "i");
        assert_eq!(line.spans[13].style.fg, Some(Color::Cyan));
        assert_eq!(line.spans[16].content.as_ref(), "space");
        assert_eq!(line.spans[16].style.fg, Some(Color::Cyan));
        assert_eq!(line.spans[19].content.as_ref(), "[ ]");
        assert_eq!(line.spans[19].style.fg, Some(Color::Cyan));
        assert_eq!(line.spans[22].content.as_ref(), "r");
        assert_eq!(line.spans[22].style.fg, Some(Color::Cyan));
    }

    #[test]
    fn footer_keys_line_compacts_load_mode_labels() {
        let line = footer_keys_line("i return to perform", "reload session");
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert_eq!(
            rendered,
            "Keys: q quit | ? help | 1-4 screens | Tab switch | i perform | space play/pause | [ ] drum | r reload"
        );
    }

    #[test]
    fn footer_line_styles_define_first_visual_hierarchy() {
        let primary = footer_primary_line("y scene jump | g follow | f fill");
        let primary_text = primary
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert_eq!(primary_text, "Primary: y scene jump | g follow | f fill");
        assert_eq!(primary.spans[0].content.as_ref(), "Primary:");
        assert_eq!(primary.spans[0].style.fg, Some(Color::Cyan));
        assert!(
            primary.spans[0].style.add_modifier.contains(Modifier::BOLD),
            "{primary:?}"
        );
        assert_eq!(primary.spans[2].content.as_ref(), "y");
        assert_eq!(primary.spans[2].style.fg, Some(Color::Cyan));
        assert_eq!(primary.spans[5].content.as_ref(), "g");
        assert_eq!(primary.spans[5].style.fg, Some(Color::Cyan));
        assert_eq!(primary.spans[8].content.as_ref(), "f");
        assert_eq!(primary.spans[8].style.fg, Some(Color::Cyan));

        let scene = footer_scene_line("launch drop @ next bar | rise [===>] | 2 trail");
        assert_eq!(scene.spans[0].content.as_ref(), "Scene:");
        assert_eq!(scene.spans[0].style.fg, Some(Color::Yellow));
        assert!(
            scene.spans[0].style.add_modifier.contains(Modifier::BOLD),
            "{scene:?}"
        );
        assert_eq!(scene.spans[1].style.fg, Some(Color::Yellow));

        let status = footer_status_line("Status: playing");
        assert_eq!(status.spans[0].style.fg, Some(Color::DarkGray));

        let ok = footer_ok_line("Warnings clear");
        assert_eq!(ok.spans[0].style.fg, Some(Color::Green));

        let warning = footer_warning_line("tempo weak");
        assert_eq!(warning.spans[0].content.as_ref(), "Warning:");
        assert_eq!(warning.spans[0].style.fg, Some(Color::Red));
        assert!(
            warning.spans[0].style.add_modifier.contains(Modifier::BOLD),
            "{warning:?}"
        );
        assert_eq!(warning.spans[1].style.fg, Some(Color::Yellow));
    }

    #[test]
    fn footer_advanced_line_styles_gesture_key_prefixes() {
        let line = footer_advanced_line("Y restore | a answer | b voice | d push");
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert_eq!(
            rendered,
            "Advanced: Y restore | a answer | b voice | d push | more in ? help"
        );
        assert_eq!(line.spans[1].content.as_ref(), "Y");
        assert_eq!(line.spans[1].style.fg, Some(Color::Cyan));
        assert_eq!(line.spans[4].content.as_ref(), "a");
        assert_eq!(line.spans[4].style.fg, Some(Color::Cyan));
        assert_eq!(line.spans[7].content.as_ref(), "b");
        assert_eq!(line.spans[7].style.fg, Some(Color::Cyan));
        assert_eq!(line.spans[10].content.as_ref(), "d");
        assert_eq!(line.spans[10].style.fg, Some(Color::Cyan));
    }

    #[test]
    fn footer_lane_ops_line_styles_gesture_key_prefixes() {
        let line = footer_lane_ops_line("t trigger | s step | x swap | z freeze");
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert_eq!(rendered, "Lane ops: t trigger | s step | x swap | z freeze");
        assert_eq!(line.spans[1].content.as_ref(), "t");
        assert_eq!(line.spans[1].style.fg, Some(Color::Cyan));
        assert_eq!(line.spans[4].content.as_ref(), "s");
        assert_eq!(line.spans[4].style.fg, Some(Color::Cyan));
        assert_eq!(line.spans[7].content.as_ref(), "x");
        assert_eq!(line.spans[7].style.fg, Some(Color::Cyan));
        assert_eq!(line.spans[10].content.as_ref(), "z");
        assert_eq!(line.spans[10].style.fg, Some(Color::Cyan));
    }

    #[test]
    fn suggested_gesture_key_tokens_use_primary_control_style() {
        let line = line_with_primary_keys("what next: [c] capture  [u] undo");
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert_eq!(rendered, "what next: [c] capture  [u] undo");
        assert_eq!(line.spans[0].content.as_ref(), "what next: ");
        assert_eq!(line.spans[1].content.as_ref(), "[c]");
        assert_eq!(line.spans[1].style.fg, Some(Color::Cyan));
        assert!(
            line.spans[1].style.add_modifier.contains(Modifier::BOLD),
            "{line:?}"
        );
        assert_eq!(line.spans[3].content.as_ref(), "[u]");
        assert_eq!(line.spans[3].style.fg, Some(Color::Cyan));
        assert!(
            line.spans[3].style.add_modifier.contains(Modifier::BOLD),
            "{line:?}"
        );
    }

    #[test]
    fn suggested_gesture_lines_style_start_key_token() {
        let shell = sample_shell_state();
        let lines = suggested_gesture_lines(&shell);

        assert_eq!(lines[0].spans[0].content.as_ref(), "[Space]");
        assert_eq!(lines[0].spans[0].style.fg, Some(Color::Cyan));
        assert!(
            lines[0].spans[0]
                .style
                .add_modifier
                .contains(Modifier::BOLD),
            "{:?}",
            lines[0]
        );
    }

    #[test]
    fn help_key_prefixes_use_primary_control_style() {
        let line = line_with_primary_key_prefixes("space: play / pause | y: jump | Tab: next");
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert_eq!(rendered, "space: play / pause | y: jump | Tab: next");
        assert_eq!(line.spans[0].content.as_ref(), "space");
        assert_eq!(line.spans[0].style.fg, Some(Color::Cyan));
        assert!(
            line.spans[0].style.add_modifier.contains(Modifier::BOLD),
            "{line:?}"
        );
        assert_eq!(line.spans[3].content.as_ref(), "y");
        assert_eq!(line.spans[3].style.fg, Some(Color::Cyan));
        assert_eq!(line.spans[6].content.as_ref(), "Tab");
        assert_eq!(line.spans[6].style.fg, Some(Color::Cyan));
    }

    #[test]
    fn help_primary_gesture_line_styles_key_prefixes_without_rewriting_text() {
        let shell = sample_shell_state();
        let line = line_with_primary_key_prefixes(format!(
            "space: play / pause | {}",
            render_help_primary_gesture_items(&shell)
        ));
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert_eq!(
            rendered,
            "space: play / pause | y: scene jump | g: follow | f: fill"
        );
        assert_eq!(line.spans[0].content.as_ref(), "space");
        assert_eq!(line.spans[0].style.fg, Some(Color::Cyan));
        assert_eq!(line.spans[3].content.as_ref(), "y");
        assert_eq!(line.spans[3].style.fg, Some(Color::Cyan));
        assert_eq!(line.spans[6].content.as_ref(), "g");
        assert_eq!(line.spans[6].style.fg, Some(Color::Cyan));
        assert_eq!(line.spans[9].content.as_ref(), "f");
        assert_eq!(line.spans[9].style.fg, Some(Color::Cyan));
    }

    #[test]
    fn capture_pending_do_next_styles_define_pending_hierarchy() {
        let intent = capture_pending_intent_line("queued [c] capture @ next_phrase");
        assert_eq!(
            intent.spans[0].content.as_ref(),
            "queued [c] capture @ next_phrase"
        );
        assert_eq!(intent.spans[0].style.fg, Some(Color::Yellow));
        assert!(
            intent.spans[0].style.add_modifier.contains(Modifier::BOLD),
            "{intent:?}"
        );

        let detail = capture_pending_detail_line("wait for commit");
        assert_eq!(detail.spans[0].content.as_ref(), "wait for commit");
        assert_eq!(detail.spans[0].style.fg, Some(Color::Yellow));
        assert!(
            !detail.spans[0].style.add_modifier.contains(Modifier::BOLD),
            "{detail:?}"
        );
    }

    #[derive(Debug, Deserialize)]
    struct Mc202RegressionFixture {
        name: String,
        initial_role: String,
        action: Mc202RegressionAction,
        requested_at: TimestampMs,
        committed_at: TimestampMs,
        boundary: Mc202RegressionBoundary,
        expected: Mc202RegressionExpected,
    }

    #[derive(Debug, Deserialize)]
    struct SceneRegressionFixture {
        name: String,
        section_labels: Vec<String>,
        action: SceneRegressionAction,
        #[serde(default)]
        initial_active_scene: Option<String>,
        #[serde(default)]
        initial_current_scene: Option<String>,
        #[serde(default)]
        initial_restore_scene: Option<String>,
        #[serde(default)]
        tr909_reinforcement_mode: Option<Tr909ReinforcementModeState>,
        #[serde(default)]
        tr909_pattern_ref: Option<String>,
        #[serde(default)]
        requested_at: Option<TimestampMs>,
        #[serde(default)]
        committed_at: Option<TimestampMs>,
        #[serde(default)]
        boundary: Option<SceneRegressionBoundary>,
        expected: SceneRegressionExpected,
    }

    #[derive(Clone, Copy, Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum SceneRegressionAction {
        ProjectCandidates,
        SelectNextScene,
        RestoreScene,
    }

    #[derive(Debug, Deserialize)]
    struct SceneRegressionBoundary {
        kind: SceneRegressionBoundaryKind,
        beat_index: u64,
        bar_index: u64,
        phrase_index: u64,
        scene_id: Option<String>,
    }

    #[derive(Clone, Copy, Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum SceneRegressionBoundaryKind {
        Immediate,
        Beat,
        HalfBar,
        Bar,
        Phrase,
        Scene,
    }

    #[derive(Debug, Deserialize)]
    struct SceneRegressionExpected {
        active_scene: String,
        #[allow(dead_code)]
        current_scene: String,
        #[allow(dead_code)]
        scenes: Vec<String>,
        #[serde(default)]
        result_summary: Option<String>,
        jam_contains: Vec<String>,
        log_contains: Vec<String>,
    }

    #[derive(Clone, Copy, Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum Mc202RegressionAction {
        SetRole,
        GenerateFollower,
        GenerateAnswer,
    }

    #[derive(Debug, Deserialize)]
    struct Mc202RegressionBoundary {
        kind: Mc202RegressionBoundaryKind,
        beat_index: u64,
        bar_index: u64,
        phrase_index: u64,
        scene_id: Option<String>,
    }

    #[derive(Clone, Copy, Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum Mc202RegressionBoundaryKind {
        Immediate,
        Beat,
        HalfBar,
        Bar,
        Phrase,
        Scene,
    }

    #[derive(Debug, Deserialize)]
    struct Mc202RegressionExpected {
        role: String,
        phrase_ref: String,
        touch: f32,
        result_summary: String,
        jam_contains: Vec<String>,
        log_contains: Vec<String>,
    }

    #[derive(Debug, Deserialize)]
    struct W30RegressionFixture {
        name: String,
        action: W30RegressionAction,
        capture_bank: String,
        capture_pad: String,
        capture_pinned: bool,
        #[serde(default)]
        source_window: Option<W30RegressionSourceWindow>,
        #[serde(default = "default_true")]
        capture_assigned: bool,
        #[serde(default)]
        extra_captures: Vec<W30RegressionCapture>,
        #[serde(default)]
        initial_active_bank: Option<String>,
        #[serde(default)]
        initial_focused_pad: Option<String>,
        #[serde(default)]
        initial_last_capture: Option<String>,
        #[serde(default)]
        initial_preview_mode: Option<String>,
        #[serde(default)]
        initial_w30_grit: Option<f32>,
        requested_at: TimestampMs,
        committed_at: TimestampMs,
        boundary: W30RegressionBoundary,
        expected: W30RegressionExpected,
    }

    #[derive(Debug, Deserialize)]
    struct W30RegressionSourceWindow {
        source_id: String,
        start_seconds: f32,
        end_seconds: f32,
        start_frame: u64,
        end_frame: u64,
    }

    #[derive(Debug, Deserialize)]
    struct W30RegressionCapture {
        capture_id: String,
        bank: String,
        pad: String,
        pinned: bool,
        #[serde(default)]
        notes: Option<String>,
    }

    #[derive(Clone, Copy, Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum W30RegressionAction {
        LiveRecall,
        RawCaptureAudition,
        PromotedAudition,
        TriggerPad,
        SwapBank,
        ApplyDamageProfile,
        LoopFreeze,
        BrowseSlicePool,
    }

    fn default_true() -> bool {
        true
    }

    #[derive(Debug, Deserialize)]
    struct W30RegressionBoundary {
        kind: W30RegressionBoundaryKind,
        beat_index: u64,
        bar_index: u64,
        phrase_index: u64,
        scene_id: Option<String>,
    }

    #[derive(Clone, Copy, Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum W30RegressionBoundaryKind {
        Immediate,
        Beat,
        HalfBar,
        Bar,
        Phrase,
        Scene,
    }

    #[derive(Debug, Deserialize)]
    struct W30RegressionExpected {
        #[serde(default)]
        jam_contains: Vec<String>,
        capture_contains: Vec<String>,
        log_contains: Vec<String>,
    }

    fn w30_preview_mode_state(value: &str) -> riotbox_core::session::W30PreviewModeState {
        match value {
            "live_recall" => riotbox_core::session::W30PreviewModeState::LiveRecall,
            "raw_capture_audition" => {
                riotbox_core::session::W30PreviewModeState::RawCaptureAudition
            }
            "promoted_audition" => riotbox_core::session::W30PreviewModeState::PromotedAudition,
            other => panic!("unsupported W-30 preview mode fixture value: {other}"),
        }
    }

    impl Mc202RegressionBoundary {
        fn to_commit_boundary_state(&self) -> CommitBoundaryState {
            CommitBoundaryState {
                kind: match self.kind {
                    Mc202RegressionBoundaryKind::Immediate => {
                        riotbox_core::action::CommitBoundary::Immediate
                    }
                    Mc202RegressionBoundaryKind::Beat => riotbox_core::action::CommitBoundary::Beat,
                    Mc202RegressionBoundaryKind::HalfBar => {
                        riotbox_core::action::CommitBoundary::HalfBar
                    }
                    Mc202RegressionBoundaryKind::Bar => riotbox_core::action::CommitBoundary::Bar,
                    Mc202RegressionBoundaryKind::Phrase => {
                        riotbox_core::action::CommitBoundary::Phrase
                    }
                    Mc202RegressionBoundaryKind::Scene => {
                        riotbox_core::action::CommitBoundary::Scene
                    }
                },
                beat_index: self.beat_index,
                bar_index: self.bar_index,
                phrase_index: self.phrase_index,
                scene_id: self.scene_id.clone().map(SceneId::from),
            }
        }
    }

    impl SceneRegressionBoundary {
        fn to_commit_boundary_state(&self) -> CommitBoundaryState {
            CommitBoundaryState {
                kind: match self.kind {
                    SceneRegressionBoundaryKind::Immediate => {
                        riotbox_core::action::CommitBoundary::Immediate
                    }
                    SceneRegressionBoundaryKind::Beat => riotbox_core::action::CommitBoundary::Beat,
                    SceneRegressionBoundaryKind::HalfBar => {
                        riotbox_core::action::CommitBoundary::HalfBar
                    }
                    SceneRegressionBoundaryKind::Bar => riotbox_core::action::CommitBoundary::Bar,
                    SceneRegressionBoundaryKind::Phrase => {
                        riotbox_core::action::CommitBoundary::Phrase
                    }
                    SceneRegressionBoundaryKind::Scene => {
                        riotbox_core::action::CommitBoundary::Scene
                    }
                },
                beat_index: self.beat_index,
                bar_index: self.bar_index,
                phrase_index: self.phrase_index,
                scene_id: self.scene_id.clone().map(SceneId::from),
            }
        }
    }

    fn scene_regression_graph(section_labels: &[String]) -> SourceGraph {
        let mut graph = sample_shell_state()
            .app
            .source_graph
            .clone()
            .expect("sample shell source graph");
        graph.sections.clear();

        for (index, label) in section_labels.iter().enumerate() {
            let bar_start = (index as u32 * 8) + 1;
            graph.sections.push(riotbox_core::source_graph::Section {
                section_id: riotbox_core::ids::SectionId::from(format!("section-{index}")),
                label_hint: scene_label_hint(label),
                start_seconds: index as f32 * 16.0,
                end_seconds: (index + 1) as f32 * 16.0,
                bar_start,
                bar_end: bar_start + 7,
                energy_class: scene_energy_for_label(label),
                confidence: 0.9,
                tags: vec![label.clone()],
            });
        }

        graph
    }

    fn seed_scene_fixture_state(shell: &mut JamShellState, fixture: &SceneRegressionFixture) {
        if let Some(current_scene) = fixture.initial_current_scene.as_deref() {
            shell.app.session.runtime_state.transport.current_scene =
                Some(SceneId::from(current_scene));
        }
        if let Some(active_scene) = fixture.initial_active_scene.as_deref() {
            shell.app.session.runtime_state.scene_state.active_scene =
                Some(SceneId::from(active_scene));
        }
        if let Some(restore_scene) = fixture.initial_restore_scene.as_deref() {
            shell.app.session.runtime_state.scene_state.restore_scene =
                Some(SceneId::from(restore_scene));
        }
        if let Some(reinforcement_mode) = fixture.tr909_reinforcement_mode {
            shell
                .app
                .session
                .runtime_state
                .lane_state
                .tr909
                .takeover_enabled = false;
            shell
                .app
                .session
                .runtime_state
                .lane_state
                .tr909
                .takeover_profile = None;
            shell
                .app
                .session
                .runtime_state
                .lane_state
                .tr909
                .reinforcement_mode = Some(reinforcement_mode);
        }
        if let Some(pattern_ref) = fixture.tr909_pattern_ref.as_deref() {
            shell.app.session.runtime_state.lane_state.tr909.pattern_ref = Some(pattern_ref.into());
        }
        shell.app.refresh_view();
    }

    impl W30RegressionBoundary {
        fn to_commit_boundary_state(&self) -> CommitBoundaryState {
            CommitBoundaryState {
                kind: match self.kind {
                    W30RegressionBoundaryKind::Immediate => {
                        riotbox_core::action::CommitBoundary::Immediate
                    }
                    W30RegressionBoundaryKind::Beat => riotbox_core::action::CommitBoundary::Beat,
                    W30RegressionBoundaryKind::HalfBar => {
                        riotbox_core::action::CommitBoundary::HalfBar
                    }
                    W30RegressionBoundaryKind::Bar => riotbox_core::action::CommitBoundary::Bar,
                    W30RegressionBoundaryKind::Phrase => {
                        riotbox_core::action::CommitBoundary::Phrase
                    }
                    W30RegressionBoundaryKind::Scene => riotbox_core::action::CommitBoundary::Scene,
                },
                beat_index: self.beat_index,
                bar_index: self.bar_index,
                phrase_index: self.phrase_index,
                scene_id: self.scene_id.clone().map(SceneId::from),
            }
        }
    }

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
        session.runtime_state.mixer_state.drum_level = 0.82;
        session.runtime_state.mixer_state.music_level = 0.64;
        session.runtime_state.lane_state.mc202.role = Some("leader".into());
        session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
        session.runtime_state.lane_state.tr909.takeover_enabled = true;
        session.runtime_state.lane_state.tr909.takeover_profile =
            Some(Tr909TakeoverProfileState::ControlledPhraseTakeover);
        session.runtime_state.lane_state.tr909.pattern_ref = Some("scene-a-main".into());
        session.ghost_state.mode = GhostMode::Assist;
        session.runtime_state.lane_state.tr909.last_fill_bar = Some(6);
        session.runtime_state.lane_state.tr909.reinforcement_mode =
            Some(Tr909ReinforcementModeState::Takeover);
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
        queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::Tr909FillNext,
                Quantization::NextBar,
                ActionTarget {
                    scope: Some(TargetScope::LaneTr909),
                    ..Default::default()
                },
            ),
            130,
        );
        let mut promote_draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::PromoteCaptureToPad,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(BankId::from("bank-a")),
                pad_id: Some(PadId::from("pad-01")),
                ..Default::default()
            },
        );
        promote_draft.params = ActionParams::Promotion {
            capture_id: Some("cap-01".into()),
            destination: Some("w30:bank-a/pad-01".into()),
        };
        promote_draft.explanation = Some("promote keeper capture into the live pad".into());
        queue.enqueue(promote_draft, 131);

        session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
        session.captures.push(riotbox_core::session::CaptureRef {
            capture_id: "cap-01".into(),
            capture_type: riotbox_core::session::CaptureType::Pad,
            source_origin_refs: vec!["asset-a".into(), "src-1".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-01.wav".into(),
            assigned_target: None,
            is_pinned: false,
            notes: Some("keeper capture".into()),
        });

        let app = JamAppState::from_parts(session, Some(graph), queue);
        JamShellState::new(app, ShellLaunchMode::Ingest)
    }

    fn first_run_shell_state() -> JamShellState {
        let sample_shell = sample_shell_state();
        let mut session = sample_shell.app.session.clone();
        session.action_log.actions.clear();
        session.captures.clear();
        session.runtime_state.lane_state.w30.last_capture = None;

        let app = JamAppState::from_parts(
            session,
            sample_shell.app.source_graph.clone(),
            ActionQueue::new(),
        );
        JamShellState::new(app, ShellLaunchMode::Ingest)
    }

    fn first_result_shell_state() -> JamShellState {
        let mut shell = first_run_shell_state();
        shell.app.session.action_log.actions.push(Action {
            id: ActionId(1),
            actor: ActorType::User,
            command: ActionCommand::Tr909FillNext,
            params: ActionParams::Empty,
            target: ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
            requested_at: 200,
            quantization: Quantization::NextBar,
            status: ActionStatus::Committed,
            committed_at: Some(220),
            result: Some(ActionResult {
                accepted: true,
                summary: "committed fill on next bar".into(),
            }),
            undo_policy: UndoPolicy::Undoable,
            explanation: Some("first committed fill".into()),
        });

        shell.app.refresh_view();
        shell
    }

    fn sample_shell_without_pending_queue() -> JamShellState {
        let sample_shell = sample_shell_state();
        JamShellState::new(
            JamAppState::from_parts(
                sample_shell.app.session.clone(),
                sample_shell.app.source_graph.clone(),
                ActionQueue::new(),
            ),
            ShellLaunchMode::Ingest,
        )
    }

    fn scene_post_commit_shell_state(
        command: ActionCommand,
        active_scene: &str,
        restore_scene: &str,
    ) -> JamShellState {
        let sample_shell = sample_shell_state();
        let mut session = sample_shell.app.session.clone();
        session.action_log.actions.clear();
        session.runtime_state.transport.current_scene = Some(SceneId::from(active_scene));
        session.runtime_state.scene_state.active_scene = Some(SceneId::from(active_scene));
        session.runtime_state.scene_state.restore_scene = Some(SceneId::from(restore_scene));
        session.runtime_state.lane_state.tr909.takeover_enabled = false;
        session.runtime_state.lane_state.tr909.takeover_profile = None;
        session.runtime_state.lane_state.tr909.reinforcement_mode =
            Some(Tr909ReinforcementModeState::SourceSupport);
        session.runtime_state.lane_state.tr909.pattern_ref = Some("scene-support".into());
        session.action_log.actions.push(Action {
            id: ActionId(1),
            actor: ActorType::User,
            command,
            params: ActionParams::Scene {
                scene_id: Some(SceneId::from(active_scene)),
            },
            target: ActionTarget {
                scope: Some(TargetScope::Scene),
                scene_id: Some(SceneId::from(active_scene)),
                ..Default::default()
            },
            requested_at: 300,
            quantization: Quantization::NextBar,
            status: ActionStatus::Committed,
            committed_at: Some(320),
            result: Some(ActionResult {
                accepted: true,
                summary: format!("scene {active_scene} landed"),
            }),
            undo_policy: UndoPolicy::Undoable,
            explanation: Some(format!("landed {active_scene} scene move")),
        });

        let mut shell = JamShellState::new(
            JamAppState::from_parts(
                session,
                sample_shell.app.source_graph.clone(),
                ActionQueue::new(),
            ),
            ShellLaunchMode::Load,
        );
        shell.app.set_transport_playing(true);
        shell
    }

    #[test]
    fn renders_more_musical_jam_shell_snapshot() {
        let shell = sample_shell_state();
        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("trust usable"));
        assert!(rendered.contains("scene scene-a | energy med"));
        assert!(rendered.contains("ghost"));
        assert!(rendered.contains("warnings"));
        assert!(rendered.contains("MC-202"));
        assert!(rendered.contains("W-30"));
        assert!(rendered.contains("TR-909"));
        assert!(rendered.contains("Suggested gestures"));
        assert!(rendered.contains("Pending / landed"));
        assert!(rendered.contains("next fill"));
        assert!(rendered.contains("wait [===>] next bar"), "{rendered}");
        assert!(
            rendered.contains("Primary: y scene jump | g follow | f fill"),
            "{rendered}"
        );
        assert!(
            rendered.contains("Advanced: Y restore | a answer | b voice | d push"),
            "{rendered}"
        );
        assert!(!rendered.contains("Sections"), "{rendered}");
    }

    #[test]
    fn renders_jam_shell_inspect_snapshot() {
        let mut shell = sample_shell_state();
        shell.jam_mode = JamViewMode::Inspect;
        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("Screen jam/inspect"), "{rendered}");
        assert!(rendered.contains("MC-202 detail"), "{rendered}");
        assert!(rendered.contains("W-30 detail"), "{rendered}");
        assert!(rendered.contains("TR-909 detail"), "{rendered}");
        assert!(rendered.contains("accent off"), "{rendered}");
        assert!(rendered.contains("Source structure"), "{rendered}");
        assert!(rendered.contains("Material flow"), "{rendered}");
        assert!(rendered.contains("Diagnostics"), "{rendered}");
        assert!(!rendered.contains("Suggested gestures"), "{rendered}");
    }

    #[test]
    fn renders_jam_shell_with_scene_brain_summary() {
        let sample_shell = sample_shell_state();
        let mut session = sample_shell.app.session.clone();
        session.runtime_state.scene_state.scenes = vec![
            SceneId::from("scene-01-intro"),
            SceneId::from("scene-02-drop"),
        ];
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
        let mut shell = JamShellState::new(
            JamAppState::from_parts(
                session,
                sample_shell.app.source_graph.clone(),
                ActionQueue::new(),
            ),
            ShellLaunchMode::Load,
        );
        assert_eq!(
            shell.app.queue_scene_select(300),
            crate::jam_app::QueueControlResult::Enqueued
        );

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("idle @ 32.0"));
        assert!(rendered.contains("scene-01-intro"));
        assert!(rendered.contains("energy medium"));
        assert!(
            rendered.contains("source src-1 | next scene drop/high"),
            "{rendered}"
        );
        assert!(rendered.contains("scene-01-intro"));
        assert!(rendered.contains("live intro/med <> restore none"));
        assert!(rendered.contains("launch ->"), "{rendered}");
        assert!(rendered.contains("@ next bar"), "{rendered}");
        assert!(
            rendered.contains("pulse [===>] b32 | b8 | p1"),
            "{rendered}"
        );
        assert!(
            rendered.contains("Scene: launch drop @ next bar | rise [===>] | 2 trail"),
            "{rendered}"
        );
        assert!(rendered.contains("energy rise"), "{rendered}");
    }

    #[test]
    fn scene_pending_line_styles_define_intent_hierarchy() {
        let sample_shell = sample_shell_state();
        let mut session = sample_shell.app.session.clone();
        session.runtime_state.scene_state.scenes = vec![
            SceneId::from("scene-01-intro"),
            SceneId::from("scene-02-drop"),
        ];
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
        let mut shell = JamShellState::new(
            JamAppState::from_parts(
                session,
                sample_shell.app.source_graph.clone(),
                ActionQueue::new(),
            ),
            ShellLaunchMode::Load,
        );
        assert_eq!(
            shell.app.queue_scene_select(300),
            crate::jam_app::QueueControlResult::Enqueued
        );

        let line = scene_pending_line(&shell);
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert_eq!(rendered, "launch -> scene-02-drop @ next bar | energy rise");
        assert_eq!(line.spans[0].content.as_ref(), "launch");
        assert_eq!(line.spans[0].style.fg, Some(Color::Yellow));
        assert!(
            line.spans[0].style.add_modifier.contains(Modifier::BOLD),
            "{line:?}"
        );
        assert_eq!(line.spans[2].content.as_ref(), "scene-02-drop");
        assert_eq!(line.spans[2].style.fg, Some(Color::Yellow));
        assert_eq!(line.spans[4].content.as_ref(), "next bar");
        assert_eq!(line.spans[4].style.fg, Some(Color::Yellow));
        assert!(
            line.spans[4].style.add_modifier.contains(Modifier::BOLD),
            "{line:?}"
        );
        assert_eq!(line.spans[6].content.as_ref(), "energy rise");
        assert_eq!(line.spans[6].style.fg, Some(Color::Green));
        assert!(
            line.spans[6].style.add_modifier.contains(Modifier::BOLD),
            "{line:?}"
        );
    }

    #[test]
    fn renders_jam_shell_with_pending_scene_restore_summary() {
        let graph = sample_shell_state()
            .app
            .source_graph
            .clone()
            .expect("sample shell source graph");
        let mut session = sample_shell_state().app.session.clone();
        session.runtime_state.scene_state.scenes = vec![
            SceneId::from("scene-01-drop"),
            SceneId::from("scene-02-intro"),
        ];
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-drop"));
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-drop"));
        session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));

        let mut shell = JamShellState::new(
            JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
            ShellLaunchMode::Load,
        );
        shell.app.session.runtime_state.scene_state.restore_scene =
            Some(SceneId::from("scene-02-intro"));
        assert_eq!(
            shell.app.queue_scene_restore(300),
            crate::jam_app::QueueControlResult::Enqueued
        );

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("scene-01-drop"), "{rendered}");
        assert!(rendered.contains("energy medium"), "{rendered}");
        assert!(
            rendered.contains("live drop/med <> restore intro/high"),
            "{rendered}"
        );
        assert!(
            rendered.contains("restore -> scene-02-intro @ next bar"),
            "{rendered}"
        );
        assert!(rendered.contains("energy rise"), "{rendered}");
        assert!(
            rendered.contains("pulse [===>] b32 | b8 | p1"),
            "{rendered}"
        );
        assert!(
            rendered.contains("restore intro @ next bar | rise [===>] | 2 trail"),
            "{rendered}"
        );
    }

    #[test]
    fn renders_log_shell_with_pending_scene_restore_summary() {
        let graph = scene_regression_graph(&["drop".into(), "break".into()]);
        let mut session = sample_shell_state().app.session.clone();
        session.runtime_state.scene_state.scenes = vec![
            SceneId::from("scene-01-drop"),
            SceneId::from("scene-02-break"),
        ];
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-02-break"));
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-02-break"));
        session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));

        let mut shell = JamShellState::new(
            JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
            ShellLaunchMode::Load,
        );
        assert_eq!(
            shell.app.queue_scene_restore(300),
            crate::jam_app::QueueControlResult::Enqueued
        );
        shell.active_screen = ShellScreen::Log;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("restore scene-01-drop"), "{rendered}");
        assert!(
            rendered.contains("requested 300 | restore scene"),
            "{rendered}"
        );
        assert!(rendered.contains("scene-01-drop on next bar"), "{rendered}");
    }

    #[test]
    fn renders_jam_shell_with_pending_mc202_role_change() {
        let mut shell = sample_shell_state();
        assert_eq!(
            shell.app.queue_mc202_role_toggle(200),
            crate::jam_app::QueueControlResult::Enqueued
        );

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("current voice leader"));
        assert!(rendered.contains("next voice follower"));
    }

    #[test]
    fn renders_jam_shell_with_pending_mc202_follower_generation() {
        let first_run_shell = first_run_shell_state();
        let mut shell = JamShellState::new(first_run_shell.app, ShellLaunchMode::Load);
        shell.app.set_transport_playing(true);
        assert_eq!(
            shell.app.queue_mc202_generate_follower(200),
            crate::jam_app::QueueControlResult::Enqueued
        );

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("next follow"));
        assert!(
            rendered.contains("wait [=======>] next phrase"),
            "{rendered}"
        );
    }

    #[test]
    fn renders_jam_shell_with_pending_mc202_answer_generation() {
        let mut shell = sample_shell_state();
        assert_eq!(
            shell.app.queue_mc202_generate_answer(200),
            crate::jam_app::QueueControlResult::Enqueued
        );

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("next answer"));
    }

    #[test]
    fn renders_jam_shell_with_two_promoted_pending_actions_and_queue_summary() {
        let first_run_shell = first_run_shell_state();
        let mut shell = JamShellState::new(first_run_shell.app, ShellLaunchMode::Load);
        shell.app.queue_scene_mutation(200);
        shell.app.queue_tr909_fill(201);
        shell.app.queue_capture_bar(202);

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("next 1 user mutate"), "{rendered}");
        assert!(rendered.contains("next 2 user fill"), "{rendered}");
        assert!(rendered.contains("+1 more"), "{rendered}");
        assert!(!rendered.contains("more queued"), "{rendered}");
    }

    #[test]
    fn quantization_countdown_cues_match_boundary_widths() {
        assert_eq!(
            quantization_countdown_cue(Quantization::NextBeat, 32, 8),
            "[>]"
        );
        assert_eq!(
            quantization_countdown_cue(Quantization::NextHalfBar, 3, 1),
            "[=>]"
        );
        assert_eq!(
            quantization_countdown_cue(Quantization::NextBar, 32, 8),
            "[===>]"
        );
        assert_eq!(
            quantization_countdown_cue(Quantization::NextPhrase, 32, 8),
            "[=======>]"
        );
    }

    #[test]
    fn queued_timing_rail_styles_define_boundary_hierarchy() {
        let shell = sample_shell_state();
        let line = queued_timing_rail_line(&shell).expect("queued timing rail");
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert_eq!(rendered, "wait [===>] next bar | b32 | bar8 | p1");
        assert_eq!(line.spans[0].content.as_ref(), "wait ");
        assert_eq!(line.spans[0].style.fg, Some(Color::DarkGray));
        assert_eq!(line.spans[1].content.as_ref(), "[===>]");
        assert_eq!(line.spans[1].style.fg, Some(Color::Yellow));
        assert!(
            line.spans[1].style.add_modifier.contains(Modifier::BOLD),
            "{line:?}"
        );
        assert_eq!(line.spans[3].content.as_ref(), "next bar");
        assert_eq!(line.spans[3].style.fg, Some(Color::Yellow));
        assert!(
            line.spans[3].style.add_modifier.contains(Modifier::BOLD),
            "{line:?}"
        );
        assert_eq!(line.spans[4].style.fg, Some(Color::DarkGray));
    }

    #[test]
    fn queued_scene_timing_rail_styles_pulse_hierarchy() {
        let sample_shell = sample_shell_state();
        let mut session = sample_shell.app.session.clone();
        session.runtime_state.scene_state.scenes = vec![
            SceneId::from("scene-01-intro"),
            SceneId::from("scene-02-drop"),
        ];
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
        let mut shell = JamShellState::new(
            JamAppState::from_parts(
                session,
                sample_shell.app.source_graph.clone(),
                ActionQueue::new(),
            ),
            ShellLaunchMode::Load,
        );
        assert_eq!(
            shell.app.queue_scene_select(300),
            crate::jam_app::QueueControlResult::Enqueued
        );

        let line = queued_timing_rail_line(&shell).expect("scene timing rail");
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert_eq!(rendered, "pulse [===>] b32 | b8 | p1");
        assert_eq!(line.spans[0].content.as_ref(), "pulse ");
        assert_eq!(line.spans[0].style.fg, Some(Color::DarkGray));
        assert_eq!(line.spans[1].content.as_ref(), "[===>]");
        assert_eq!(line.spans[1].style.fg, Some(Color::Yellow));
        assert!(
            line.spans[1].style.add_modifier.contains(Modifier::BOLD),
            "{line:?}"
        );
        assert_eq!(line.spans[2].style.fg, Some(Color::DarkGray));
    }

    #[test]
    fn renders_jam_shell_with_first_run_onramp() {
        let shell = first_run_shell_state();
        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("Start Here"), "{rendered}");
        assert!(rendered.contains("1 [Space] start transport"), "{rendered}");
        assert!(
            rendered.contains("2 [f] queue one first fill"),
            "{rendered}"
        );
        assert!(
            rendered.contains("3 [2] watch Log when it lands on the next bar"),
            "{rendered}"
        );
    }

    #[test]
    fn renders_jam_shell_with_queued_first_move_guidance() {
        let mut shell = first_run_shell_state();
        shell.app.queue_tr909_fill(200);

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("Your first move is armed."), "{rendered}");
        assert!(rendered.contains("next bar"), "{rendered}");
        assert!(rendered.contains("confirm it in Log"), "{rendered}");
        assert!(rendered.contains("[c] capture"), "{rendered}");
    }

    #[test]
    fn renders_jam_shell_with_first_result_guidance() {
        let shell = first_result_shell_state();
        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(
            rendered.contains("What changed: landed user fill"),
            "{rendered}"
        );
        assert!(
            rendered.contains("What next: [c] capture it or [u] undo it if it missed."),
            "{rendered}"
        );
        assert!(
            rendered.contains("Then try one more move: [y] jump or [g] follow."),
            "{rendered}"
        );
    }

    #[test]
    fn next_panel_promotes_timing_rail_above_landed_history() {
        let mut shell = first_result_shell_state();
        shell.app.queue_tr909_fill(240);

        let line_texts = next_panel_lines(&shell)
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>();

        assert_eq!(line_texts[0], "user tr909.fill_next @ next_bar");
        assert_eq!(line_texts[1], "scene transition idle");
        assert!(
            line_texts[2].starts_with("wait [===>] next bar"),
            "{line_texts:?}"
        );
        assert_eq!(line_texts[3], "landed user fill");
    }

    #[test]
    fn renders_jam_shell_with_post_commit_next_step_cue() {
        let first_result_shell = first_result_shell_state();
        let mut shell = JamShellState::new(first_result_shell.app, ShellLaunchMode::Load);
        shell.app.set_transport_playing(true);
        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(
            rendered.contains("what changed: landed user fill"),
            "{rendered}"
        );
        assert!(
            rendered.contains("what next: [c] capture  [u] undo"),
            "{rendered}"
        );
        assert!(
            rendered.contains("then try: [y] jump intro (hold)"),
            "{rendered}"
        );
    }

    #[test]
    fn renders_jam_shell_with_single_scene_jump_waiting_cue() {
        let sample_shell = sample_shell_state();
        let mut session = sample_shell.app.session.clone();
        session.runtime_state.scene_state.scenes = vec![SceneId::from("scene-01-intro")];
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));

        let shell = JamShellState::new(
            JamAppState::from_parts(
                session,
                sample_shell.app.source_graph.clone(),
                ActionQueue::new(),
            ),
            ShellLaunchMode::Load,
        );
        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(
            rendered.contains("source src-1 | next scene waits for 2")
                && rendered.contains("scenes"),
            "{rendered}"
        );
        assert!(
            rendered.contains("[y] jump waits for 2 scenes"),
            "{rendered}"
        );
        assert!(
            rendered.contains("Primary: y jump waits | g follow | f fill"),
            "{rendered}"
        );

        let mut shell = shell;
        shell.show_help = true;
        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(
            rendered.contains("space: play / pause | y: jump waits | g: follow | f: fill"),
            "{rendered}"
        );
    }

    #[test]
    fn renders_scene_jump_post_commit_guidance() {
        let shell = scene_post_commit_shell_state(
            ActionCommand::SceneLaunch,
            "scene-02-break",
            "scene-01-drop",
        );
        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(
            rendered.contains("scene break/high | restore drop/med"),
            "{rendered}"
        );
        assert!(
            rendered.contains("live break/high <> restore drop/med"),
            "{rendered}"
        );
        assert!(
            rendered.contains("landed user scene jump | energy rise"),
            "{rendered}"
        );
        assert!(rendered.contains("909 lift"), "{rendered}");
        assert!(rendered.contains("next [Y]"), "{rendered}");
        assert!(rendered.contains("restore [c] capture"), "{rendered}");
        assert!(rendered.contains("[c] capture"), "{rendered}");
    }

    #[test]
    fn scene_post_commit_cue_styles_define_performance_hierarchy() {
        let shell = scene_post_commit_shell_state(
            ActionCommand::SceneLaunch,
            "scene-02-break",
            "scene-01-drop",
        );
        let line = scene_post_commit_cue_line(&shell).expect("scene post-commit cue");
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert_eq!(
            rendered,
            "scene break/high | restore drop/med | 909 lift | next [Y] restore [c] capture"
        );
        assert_eq!(line.spans[0].style.fg, Some(Color::DarkGray));
        assert_eq!(line.spans[1].content.as_ref(), "break/high");
        assert_eq!(line.spans[1].style.fg, Some(Color::Green));
        assert!(
            line.spans[1].style.add_modifier.contains(Modifier::BOLD),
            "{line:?}"
        );
        assert_eq!(line.spans[3].content.as_ref(), "drop/med");
        assert_eq!(line.spans[3].style.fg, Some(Color::Yellow));
        assert_eq!(line.spans[5].content.as_ref(), "909 lift");
        assert_eq!(line.spans[5].style.fg, Some(Color::Yellow));
        assert_eq!(line.spans[7].content.as_ref(), "[Y]");
        assert_eq!(line.spans[7].style.fg, Some(Color::Cyan));
        assert!(
            line.spans[7].style.add_modifier.contains(Modifier::BOLD),
            "{line:?}"
        );
        assert_eq!(line.spans[9].content.as_ref(), "[c]");
        assert_eq!(line.spans[9].style.fg, Some(Color::Cyan));
        assert!(
            line.spans[9].style.add_modifier.contains(Modifier::BOLD),
            "{line:?}"
        );
    }

    #[test]
    fn latest_landed_line_styles_define_result_hierarchy() {
        let shell = scene_post_commit_shell_state(
            ActionCommand::SceneLaunch,
            "scene-02-break",
            "scene-01-drop",
        );
        let line = latest_landed_line(&shell);
        let rendered = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert_eq!(rendered, "landed user scene jump | energy rise");
        assert_eq!(latest_landed_text(&shell), rendered);
        assert_eq!(line.spans[0].content.as_ref(), "landed ");
        assert_eq!(line.spans[0].style.fg, Some(Color::DarkGray));
        assert_eq!(line.spans[1].content.as_ref(), "user ");
        assert_eq!(line.spans[1].style.fg, Some(Color::DarkGray));
        assert_eq!(line.spans[2].content.as_ref(), "scene jump");
        assert_eq!(line.spans[2].style.fg, Some(Color::Green));
        assert!(
            line.spans[2].style.add_modifier.contains(Modifier::BOLD),
            "{line:?}"
        );
        assert_eq!(line.spans[4].content.as_ref(), "energy rise");
        assert_eq!(line.spans[4].style.fg, Some(Color::Green));
        assert!(
            line.spans[4].style.add_modifier.contains(Modifier::BOLD),
            "{line:?}"
        );
    }

    #[test]
    fn renders_scene_restore_post_commit_guidance() {
        let shell = scene_post_commit_shell_state(
            ActionCommand::SceneRestore,
            "scene-01-drop",
            "scene-02-break",
        );
        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(
            rendered.contains("scene drop/med | restore break/high"),
            "{rendered}"
        );
        assert!(
            rendered.contains("live drop/med <> restore break/high"),
            "{rendered}"
        );
        assert!(
            rendered.contains("landed user restore | energy drop"),
            "{rendered}"
        );
        assert!(rendered.contains("909 lift"), "{rendered}");
        assert!(rendered.contains("next [y]"), "{rendered}");
        assert!(rendered.contains("jump [c] capture"), "{rendered}");
        assert!(rendered.contains("[c] capture"), "{rendered}");
    }

    #[test]
    fn omits_scene_post_commit_tr909_lift_without_scene_accent() {
        let mut shell = scene_post_commit_shell_state(
            ActionCommand::SceneRestore,
            "scene-01-drop",
            "scene-02-break",
        );
        shell
            .app
            .session
            .runtime_state
            .lane_state
            .tr909
            .reinforcement_mode = None;
        shell.app.session.runtime_state.lane_state.tr909.pattern_ref = None;
        shell.app.refresh_view();
        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(
            rendered.contains("scene drop/med | restore break/high | next"),
            "{rendered}"
        );
        assert!(!rendered.contains("909 lift"), "{rendered}");
    }

    #[test]
    fn renders_help_overlay_with_first_run_guidance() {
        let mut shell = first_run_shell_state();
        shell.show_help = true;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("First run"), "{rendered}");
        assert!(rendered.contains("space: start transport"), "{rendered}");
        assert!(rendered.contains("f: queue one first fill"), "{rendered}");
        assert!(
            rendered.contains("2: switch to Log and watch it land"),
            "{rendered}"
        );
        assert!(
            rendered.contains("After first loop: docs/jam_recipes.md -> Recipe 2 / Recipe 5"),
            "{rendered}"
        );
    }

    #[test]
    fn renders_help_overlay_with_pending_scene_jump_cue() {
        let mut shell = sample_shell_state();
        assert_eq!(
            shell.app.queue_scene_select(300),
            crate::jam_app::QueueControlResult::Enqueued
        );
        shell.show_help = true;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("Scene timing"), "{rendered}");
        assert!(
            rendered.contains("launch intro: lands at next bar"),
            "{rendered}"
        );
        assert!(
            rendered.contains("Jam: read launch/restore, pulse, live/restore energy"),
            "{rendered}"
        );
        assert!(
            rendered.contains("2: confirm the landed trail on Log"),
            "{rendered}"
        );
    }

    #[test]
    fn renders_help_overlay_with_pending_scene_restore_cue() {
        let graph = scene_regression_graph(&["drop".into(), "break".into()]);
        let mut session = sample_shell_state().app.session.clone();
        session.runtime_state.scene_state.scenes = vec![
            SceneId::from("scene-01-drop"),
            SceneId::from("scene-02-break"),
        ];
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-02-break"));
        session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));

        let mut shell = JamShellState::new(
            JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
            ShellLaunchMode::Load,
        );
        shell.app.set_transport_playing(true);
        assert_eq!(
            shell.app.queue_scene_restore(300),
            crate::jam_app::QueueControlResult::Enqueued
        );
        shell.show_help = true;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("Scene timing"), "{rendered}");
        assert!(
            rendered.contains("restore drop: lands at next bar"),
            "{rendered}"
        );
        assert!(
            rendered.contains("2: confirm the landed trail on Log"),
            "{rendered}"
        );
    }

    #[test]
    fn renders_help_overlay_with_capture_path_cue() {
        let mut shell = sample_shell_state();
        shell.active_screen = ShellScreen::Capture;
        shell.show_help = true;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("Capture path"), "{rendered}");
        assert!(
            rendered.contains("Do Next: read capture -> promote -> hit"),
            "{rendered}"
        );
        assert!(
            rendered.contains("hear ... stored means [p] promote, then [w] hit"),
            "{rendered}"
        );
        assert!(
            rendered.contains("2: confirm promote, hit, and audition results in Log"),
            "{rendered}"
        );
    }

    #[test]
    fn renders_jam_shell_with_restore_readiness_cue() {
        let graph = scene_regression_graph(&["intro".into(), "drop".into()]);
        let mut session = sample_shell_state().app.session.clone();
        session.runtime_state.scene_state.scenes = vec![
            SceneId::from("scene-01-intro"),
            SceneId::from("scene-02-drop"),
        ];
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
        session.runtime_state.scene_state.restore_scene = None;

        let mut shell = JamShellState::new(
            JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
            ShellLaunchMode::Load,
        );
        shell.app.set_transport_playing(true);

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("[y] jump first"), "{rendered}");
        assert!(
            rendered.contains("[Y] restore waits for one landed"),
            "{rendered}"
        );
        assert!(rendered.contains("jump"), "{rendered}");
    }

    #[test]
    fn renders_help_overlay_with_restore_readiness_cue() {
        let graph = scene_regression_graph(&["intro".into(), "drop".into()]);
        let mut session = sample_shell_state().app.session.clone();
        session.runtime_state.scene_state.scenes = vec![
            SceneId::from("scene-01-intro"),
            SceneId::from("scene-02-drop"),
        ];
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
        session.runtime_state.scene_state.restore_scene = None;

        let mut shell = JamShellState::new(
            JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
            ShellLaunchMode::Load,
        );
        shell.app.set_transport_playing(true);
        shell.show_help = true;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("Scene restore"), "{rendered}");
        assert!(
            rendered.contains("Y waits for one landed jump"),
            "{rendered}"
        );
        assert!(
            rendered.contains("land one jump, then Y can restore the last scene"),
            "{rendered}"
        );
    }

    #[test]
    fn renders_jam_shell_with_restore_ready_cue() {
        let graph = scene_regression_graph(&["drop".into(), "break".into()]);
        let mut session = sample_shell_state().app.session.clone();
        session.runtime_state.scene_state.scenes = vec![
            SceneId::from("scene-01-drop"),
            SceneId::from("scene-02-break"),
        ];
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-02-break"));
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-02-break"));
        session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));

        let mut shell = JamShellState::new(
            JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
            ShellLaunchMode::Load,
        );
        shell.app.set_transport_playing(true);

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(
            rendered.contains("[Y] restore drop now (rise)"),
            "{rendered}"
        );
        assert!(
            rendered.contains("Scene: restore drop/high ready | rise | Y brings back drop/high"),
            "{rendered}"
        );
    }

    #[test]
    fn renders_help_overlay_with_restore_ready_cue() {
        let graph = scene_regression_graph(&["drop".into(), "break".into()]);
        let mut session = sample_shell_state().app.session.clone();
        session.runtime_state.scene_state.scenes = vec![
            SceneId::from("scene-01-drop"),
            SceneId::from("scene-02-break"),
        ];
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-02-break"));
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-02-break"));
        session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));

        let mut shell = JamShellState::new(
            JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
            ShellLaunchMode::Load,
        );
        shell.app.set_transport_playing(true);
        shell.show_help = true;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("Scene restore"), "{rendered}");
        assert!(
            rendered.contains("Y is live now for drop/high (rise)"),
            "{rendered}"
        );
        assert!(
            rendered.contains("press Y to bring drop/high back on the next bar"),
            "{rendered}"
        );
    }

    fn mc202_committed_shell_state(fixture: &Mc202RegressionFixture) -> JamShellState {
        let sample_shell = sample_shell_state();
        let mut session = sample_shell.app.session.clone();
        session.action_log.actions.clear();
        session.captures.clear();
        session.runtime_state.lane_state.w30.last_capture = None;
        session.runtime_state.lane_state.mc202.role = Some(fixture.initial_role.clone());
        session.runtime_state.lane_state.mc202.phrase_ref = None;
        session.runtime_state.macro_state.mc202_touch = 0.4;

        let mut shell = JamShellState::new(
            JamAppState::from_parts(
                session,
                sample_shell.app.source_graph.clone(),
                ActionQueue::new(),
            ),
            ShellLaunchMode::Load,
        );

        let queue_result = match fixture.action {
            Mc202RegressionAction::SetRole => {
                shell.app.queue_mc202_role_toggle(fixture.requested_at)
            }
            Mc202RegressionAction::GenerateFollower => shell
                .app
                .queue_mc202_generate_follower(fixture.requested_at),
            Mc202RegressionAction::GenerateAnswer => {
                shell.app.queue_mc202_generate_answer(fixture.requested_at)
            }
        };
        assert_eq!(
            queue_result,
            crate::jam_app::QueueControlResult::Enqueued,
            "{} did not enqueue",
            fixture.name
        );

        let committed = shell.app.commit_ready_actions(
            fixture.boundary.to_commit_boundary_state(),
            fixture.committed_at,
        );
        assert_eq!(
            committed.len(),
            1,
            "{} did not commit exactly one action",
            fixture.name
        );
        assert_eq!(
            shell
                .app
                .session
                .runtime_state
                .lane_state
                .mc202
                .role
                .as_deref(),
            Some(fixture.expected.role.as_str()),
            "{} role drifted",
            fixture.name
        );
        assert_eq!(
            shell
                .app
                .session
                .runtime_state
                .lane_state
                .mc202
                .phrase_ref
                .as_deref(),
            Some(fixture.expected.phrase_ref.as_str()),
            "{} phrase ref drifted",
            fixture.name
        );
        assert_eq!(
            shell.app.session.runtime_state.macro_state.mc202_touch, fixture.expected.touch,
            "{} touch drifted",
            fixture.name
        );
        assert_eq!(
            shell
                .app
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some(fixture.expected.result_summary.as_str()),
            "{} result summary drifted",
            fixture.name
        );

        shell
    }

    fn scene_committed_shell_state(fixture: &SceneRegressionFixture) -> JamShellState {
        let sample_shell = sample_shell_state();
        let graph = scene_regression_graph(&fixture.section_labels);
        let mut session = sample_shell.app.session.clone();
        session.runtime_state.transport.current_scene = None;
        session.runtime_state.scene_state.active_scene = None;
        session.runtime_state.scene_state.scenes.clear();

        let mut shell = JamShellState::new(
            JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
            ShellLaunchMode::Load,
        );
        seed_scene_fixture_state(&mut shell, fixture);

        match fixture.action {
            SceneRegressionAction::ProjectCandidates => {}
            SceneRegressionAction::SelectNextScene => {
                assert_eq!(
                    shell.app.queue_scene_select(
                        fixture.requested_at.expect("scene select requested_at")
                    ),
                    crate::jam_app::QueueControlResult::Enqueued,
                    "{} did not enqueue",
                    fixture.name
                );

                let committed = shell.app.commit_ready_actions(
                    fixture
                        .boundary
                        .as_ref()
                        .expect("scene select boundary")
                        .to_commit_boundary_state(),
                    fixture.committed_at.expect("scene select committed_at"),
                );
                assert_eq!(
                    committed.len(),
                    1,
                    "{} did not commit exactly one action",
                    fixture.name
                );
            }
            SceneRegressionAction::RestoreScene => {
                assert_eq!(
                    shell.app.queue_scene_restore(
                        fixture.requested_at.expect("scene restore requested_at")
                    ),
                    crate::jam_app::QueueControlResult::Enqueued,
                    "{} did not enqueue",
                    fixture.name
                );

                let committed = shell.app.commit_ready_actions(
                    fixture
                        .boundary
                        .as_ref()
                        .expect("scene restore boundary")
                        .to_commit_boundary_state(),
                    fixture.committed_at.expect("scene restore committed_at"),
                );
                assert_eq!(
                    committed.len(),
                    1,
                    "{} did not commit exactly one action",
                    fixture.name
                );
            }
        }

        assert_eq!(
            shell.app.jam_view.scene.active_scene.as_deref(),
            Some(fixture.expected.active_scene.as_str()),
            "{} active scene drifted",
            fixture.name
        );
        if let Some(expected_summary) = &fixture.expected.result_summary {
            assert_eq!(
                shell
                    .app
                    .session
                    .action_log
                    .actions
                    .last()
                    .and_then(|action| action.result.as_ref())
                    .map(|result| result.summary.as_str()),
                Some(expected_summary.as_str()),
                "{} result summary drifted",
                fixture.name
            );
        }

        shell
    }

    #[test]
    fn mc202_fixture_backed_shell_regressions_hold() {
        let fixtures: Vec<Mc202RegressionFixture> =
            serde_json::from_str(include_str!("../tests/fixtures/mc202_regression.json"))
                .expect("parse MC-202 regression fixtures");

        for fixture in fixtures {
            let mut shell = mc202_committed_shell_state(&fixture);
            shell.active_screen = ShellScreen::Jam;
            let jam_rendered = render_jam_shell_snapshot(&shell, 120, 34);
            for needle in &fixture.expected.jam_contains {
                assert!(
                    jam_rendered.contains(needle),
                    "{} jam snapshot missing {needle}\n{jam_rendered}",
                    fixture.name,
                    jam_rendered = jam_rendered
                );
            }

            shell.active_screen = ShellScreen::Log;
            let log_rendered = render_jam_shell_snapshot(&shell, 120, 34);
            for needle in &fixture.expected.log_contains {
                assert!(
                    log_rendered.contains(needle),
                    "{} log snapshot missing {needle}",
                    fixture.name
                );
            }
        }
    }

    #[test]
    fn scene_fixture_backed_shell_regressions_hold() {
        let fixtures: Vec<SceneRegressionFixture> =
            serde_json::from_str(include_str!("../tests/fixtures/scene_regression.json"))
                .expect("parse Scene Brain regression fixtures");

        for fixture in fixtures {
            let mut shell = scene_committed_shell_state(&fixture);
            shell.active_screen = ShellScreen::Jam;
            let jam_rendered = render_jam_shell_snapshot(&shell, 120, 34);
            for needle in &fixture.expected.jam_contains {
                assert!(
                    jam_rendered.contains(needle),
                    "{} jam snapshot missing {needle}\n{jam_rendered}",
                    fixture.name,
                    jam_rendered = jam_rendered
                );
            }

            shell.active_screen = ShellScreen::Log;
            let log_rendered = render_jam_shell_snapshot(&shell, 120, 34);
            for needle in &fixture.expected.log_contains {
                assert!(
                    log_rendered.contains(needle),
                    "{} log snapshot missing {needle}\n{log_rendered}",
                    fixture.name,
                    log_rendered = log_rendered
                );
            }
        }
    }

    fn w30_committed_shell_state(fixture: &W30RegressionFixture) -> JamShellState {
        let sample_shell = sample_shell_state();
        let mut session = sample_shell.app.session.clone();
        session.action_log.actions.clear();
        session.runtime_state.macro_state.w30_grit = fixture.initial_w30_grit.unwrap_or(0.0);
        session.runtime_state.lane_state.w30.active_bank = Some(BankId::from(
            fixture
                .initial_active_bank
                .clone()
                .unwrap_or_else(|| fixture.capture_bank.clone()),
        ));
        session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from(
            fixture
                .initial_focused_pad
                .clone()
                .unwrap_or_else(|| fixture.capture_pad.clone()),
        ));
        session.runtime_state.lane_state.w30.last_capture =
            fixture.initial_last_capture.clone().map(CaptureId::from);
        session.runtime_state.lane_state.w30.preview_mode = fixture
            .initial_preview_mode
            .as_deref()
            .map(w30_preview_mode_state);
        session.captures[0].assigned_target =
            fixture
                .capture_assigned
                .then(|| riotbox_core::session::CaptureTarget::W30Pad {
                    bank_id: fixture.capture_bank.clone().into(),
                    pad_id: fixture.capture_pad.clone().into(),
                });
        session.captures[0].is_pinned = fixture.capture_pinned;
        session.captures[0].source_window = fixture.source_window.as_ref().map(|source_window| {
            riotbox_core::session::CaptureSourceWindow {
                source_id: SourceId::from(source_window.source_id.clone()),
                start_seconds: source_window.start_seconds,
                end_seconds: source_window.end_seconds,
                start_frame: source_window.start_frame,
                end_frame: source_window.end_frame,
            }
        });
        for extra in &fixture.extra_captures {
            session.captures.push(riotbox_core::session::CaptureRef {
                capture_id: extra.capture_id.clone().into(),
                capture_type: riotbox_core::session::CaptureType::Pad,
                source_origin_refs: vec!["fixture-extra".into()],
                source_window: None,
                lineage_capture_refs: Vec::new(),
                resample_generation_depth: 0,
                created_from_action: None,
                storage_path: format!("captures/{}.wav", extra.capture_id),
                assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                    bank_id: extra.bank.clone().into(),
                    pad_id: extra.pad.clone().into(),
                }),
                is_pinned: extra.pinned,
                notes: extra.notes.clone(),
            });
        }

        let mut shell = JamShellState::new(
            JamAppState::from_parts(
                session,
                sample_shell.app.source_graph.clone(),
                ActionQueue::new(),
            ),
            ShellLaunchMode::Load,
        );

        let queue_result = match fixture.action {
            W30RegressionAction::LiveRecall => {
                shell.app.queue_w30_live_recall(fixture.requested_at)
            }
            W30RegressionAction::RawCaptureAudition => {
                shell.app.queue_w30_audition(fixture.requested_at)
            }
            W30RegressionAction::PromotedAudition => {
                shell.app.queue_w30_promoted_audition(fixture.requested_at)
            }
            W30RegressionAction::TriggerPad => {
                shell.app.queue_w30_trigger_pad(fixture.requested_at)
            }
            W30RegressionAction::SwapBank => shell.app.queue_w30_swap_bank(fixture.requested_at),
            W30RegressionAction::ApplyDamageProfile => shell
                .app
                .queue_w30_apply_damage_profile(fixture.requested_at),
            W30RegressionAction::LoopFreeze => {
                shell.app.queue_w30_loop_freeze(fixture.requested_at)
            }
            W30RegressionAction::BrowseSlicePool => {
                shell.app.queue_w30_browse_slice_pool(fixture.requested_at)
            }
        };
        assert_eq!(
            queue_result,
            Some(crate::jam_app::QueueControlResult::Enqueued),
            "{} did not enqueue",
            fixture.name
        );

        let committed = shell.app.commit_ready_actions(
            fixture.boundary.to_commit_boundary_state(),
            fixture.committed_at,
        );
        assert_eq!(
            committed.len(),
            1,
            "{} did not commit exactly one action",
            fixture.name
        );

        shell
    }

    #[test]
    fn w30_fixture_backed_shell_regressions_hold() {
        let fixtures: Vec<W30RegressionFixture> =
            serde_json::from_str(include_str!("../tests/fixtures/w30_regression.json"))
                .expect("parse W-30 regression fixtures");

        for fixture in fixtures {
            let mut shell = w30_committed_shell_state(&fixture);
            shell.active_screen = ShellScreen::Jam;
            let jam_rendered = render_jam_shell_snapshot(&shell, 120, 34);
            for needle in &fixture.expected.jam_contains {
                assert!(
                    jam_rendered.contains(needle),
                    "{} jam snapshot missing {needle}\n{jam_rendered}",
                    fixture.name,
                    jam_rendered = jam_rendered
                );
            }

            shell.active_screen = ShellScreen::Capture;
            let capture_rendered = render_jam_shell_snapshot(&shell, 120, 34);
            for needle in &fixture.expected.capture_contains {
                assert!(
                    capture_rendered.contains(needle),
                    "{} capture snapshot missing {needle}\n{capture_rendered}",
                    fixture.name,
                    capture_rendered = capture_rendered
                );
            }

            shell.active_screen = ShellScreen::Log;
            let log_rendered = render_jam_shell_snapshot(&shell, 120, 34);
            for needle in &fixture.expected.log_contains {
                assert!(
                    log_rendered.contains(needle),
                    "{} log snapshot missing {needle}\n{log_rendered}",
                    fixture.name,
                    log_rendered = log_rendered
                );
            }
        }
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
            shell.handle_key_code(KeyCode::Char('4')),
            ShellKeyOutcome::Continue
        );
        assert_eq!(shell.active_screen, ShellScreen::Capture);
        assert_eq!(shell.status_message, "switched to capture screen");
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('i')),
            ShellKeyOutcome::Continue
        );
        assert_eq!(
            shell.status_message,
            "open Jam first if you want to use inspect"
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Tab),
            ShellKeyOutcome::Continue
        );
        assert_eq!(shell.active_screen, ShellScreen::Jam);
        assert_eq!(shell.status_message, "switched to jam screen");
        assert_eq!(shell.jam_mode, JamViewMode::Perform);
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('i')),
            ShellKeyOutcome::Continue
        );
        assert_eq!(shell.jam_mode, JamViewMode::Inspect);
        assert_eq!(
            shell.status_message,
            "opened Jam inspect | press i to return to perform"
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('i')),
            ShellKeyOutcome::Continue
        );
        assert_eq!(shell.jam_mode, JamViewMode::Perform);
        assert_eq!(shell.status_message, "returned Jam to perform mode");
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('m')),
            ShellKeyOutcome::QueueSceneMutation
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('y')),
            ShellKeyOutcome::QueueSceneSelect
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('Y')),
            ShellKeyOutcome::QueueSceneRestore
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('b')),
            ShellKeyOutcome::QueueMc202RoleToggle
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('g')),
            ShellKeyOutcome::QueueMc202GenerateFollower
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('a')),
            ShellKeyOutcome::QueueMc202GenerateAnswer
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
            shell.handle_key_code(KeyCode::Char('s')),
            ShellKeyOutcome::QueueTr909Slam
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('t')),
            ShellKeyOutcome::QueueTr909Takeover
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('k')),
            ShellKeyOutcome::QueueTr909SceneLock
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('x')),
            ShellKeyOutcome::QueueTr909Release
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('c')),
            ShellKeyOutcome::QueueCaptureBar
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('p')),
            ShellKeyOutcome::PromoteLastCapture
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('w')),
            ShellKeyOutcome::QueueW30TriggerPad
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('n')),
            ShellKeyOutcome::QueueW30StepFocus
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('B')),
            ShellKeyOutcome::QueueW30SwapBank
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('j')),
            ShellKeyOutcome::QueueW30BrowseSlicePool
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('D')),
            ShellKeyOutcome::QueueW30ApplyDamageProfile
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('z')),
            ShellKeyOutcome::QueueW30LoopFreeze
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('l')),
            ShellKeyOutcome::QueueW30LiveRecall
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('o')),
            ShellKeyOutcome::QueueW30Audition
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('e')),
            ShellKeyOutcome::QueueW30Resample
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('v')),
            ShellKeyOutcome::TogglePinLatestCapture
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('[')),
            ShellKeyOutcome::LowerDrumBusLevel
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char(']')),
            ShellKeyOutcome::RaiseDrumBusLevel
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
    fn first_run_shell_blocks_jam_inspect_toggle() {
        let mut shell = first_run_shell_state();

        assert_eq!(shell.jam_mode, JamViewMode::Perform);
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('i')),
            ShellKeyOutcome::Continue
        );
        assert_eq!(shell.jam_mode, JamViewMode::Perform);
        assert_eq!(
            shell.status_message,
            "finish the first guided move before opening inspect"
        );
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
        assert!(rendered.contains("MC-202 Lane"));
        assert!(rendered.contains("W-30 Lane"));
        assert!(rendered.contains("role leader"));
        assert!(rendered.contains("cue idle"));
        assert!(rendered.contains("cue idle | none"));
        assert!(rendered.contains("prev recall/fallback"));
        assert!(rendered.contains("mix 0.64/0.50 idle"));
        assert!(rendered.contains("cap cap-01 | pending"));
        assert!(rendered.contains("ghost"));
        assert!(rendered.contains("mutate.scene"));
        assert!(rendered.contains("TR-909 Render"));
        assert!(rendered.contains("accent off"));
        assert!(rendered.contains("takeover"));
        assert!(rendered.contains("scene lock blocked ghost"));
        assert!(rendered.contains("undid most recent musical"));
    }

    #[test]
    fn renders_log_shell_snapshot_with_scene_brain_diagnostics() {
        let mut shell = sample_shell_state();
        assert_eq!(
            shell.app.queue_scene_select(300),
            crate::jam_app::QueueControlResult::Enqueued
        );
        shell.active_screen = ShellScreen::Log;
        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("Counts"));
        assert!(rendered.contains("scene scene-a | medium"));
        assert!(rendered.contains("restore none"));
        assert!(rendered.contains("pending"));
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

    #[test]
    fn renders_capture_shell_snapshot_with_capture_context() {
        let mut shell = sample_shell_state();
        shell.active_screen = ShellScreen::Capture;
        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("[4 Capture]"));
        assert!(rendered.contains("Readiness"));
        assert!(rendered.contains("Latest Capture"));
        assert!(rendered.contains("Do Next"));
        assert!(rendered.contains("Provenance"));
        assert!(rendered.contains("Pending Capture Cues"));
        assert!(rendered.contains("Recent Captures"));
        assert!(rendered.contains("Advanced Routing"));
        assert!(rendered.contains("cap-01"));
        assert!(rendered.contains("promote keeper capture"));
        assert!(rendered.contains("promotion result pending"));
        assert!(rendered.contains("captures total 1"));
        assert!(rendered.contains("pinned 0 | promoted 0"));
        assert!(
            rendered.contains("queued [p] promote @ next_bar"),
            "{rendered}"
        );
        assert!(
            rendered.contains("wait, then hear with [w] hit"),
            "{rendered}"
        );
        assert!(
            rendered.contains("target lanew30:bank-a/pad-01"),
            "{rendered}"
        );
        assert!(rendered.contains("pending W-30 cue idle"));
        assert!(
            rendered.contains("hear cap-01 stored fallback [o] raw"),
            "{rendered}"
        );
        assert!(rendered.contains("[p]->[w]"), "{rendered}");
        assert!(
            rendered.contains("forge idle | tap ready/raw"),
            "{rendered}"
        );
        assert!(rendered.contains("g0"), "{rendered}");
        assert!(rendered.contains("latest promoted none"));
    }

    #[test]
    fn renders_capture_provenance_source_window_when_available() {
        let mut shell = sample_shell_state();
        shell.app.session.captures[0].source_window =
            Some(riotbox_core::session::CaptureSourceWindow {
                source_id: SourceId::from("src-1"),
                start_seconds: 1.25,
                end_seconds: 3.75,
                start_frame: 60_000,
                end_frame: 180_000,
            });
        shell.app.refresh_view();
        shell.active_screen = ShellScreen::Capture;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("win src-1 1.25-3.75s"), "{rendered}");
    }

    #[test]
    fn renders_recent_capture_source_window_shorthand_when_available() {
        let mut shell = sample_shell_state();
        shell.app.session.captures[0].source_window =
            Some(riotbox_core::session::CaptureSourceWindow {
                source_id: SourceId::from("src-1"),
                start_seconds: 1.25,
                end_seconds: 3.75,
                start_frame: 60_000,
                end_frame: 180_000,
            });
        shell.app.refresh_view();
        shell.active_screen = ShellScreen::Capture;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("cap-01 | 1.25-3.75s"), "{rendered}");
    }

    #[test]
    fn source_window_formatters_keep_surface_shapes_stable() {
        let source_window = riotbox_core::session::CaptureSourceWindow {
            source_id: SourceId::from("src-1"),
            start_seconds: 1.25,
            end_seconds: 3.75,
            start_frame: 60_000,
            end_frame: 180_000,
        };

        assert_eq!(format_source_window_span(&source_window), "1.25-3.75s");
        assert_eq!(
            format_source_window_log_compact(&source_window),
            "win 1.25-3.75s src-1"
        );
    }

    #[test]
    fn renders_log_w30_source_window_when_available() {
        let mut shell = sample_shell_state();
        shell.app.session.captures[0].source_window =
            Some(riotbox_core::session::CaptureSourceWindow {
                source_id: SourceId::from("src-1"),
                start_seconds: 1.25,
                end_seconds: 3.75,
                start_frame: 60_000,
                end_frame: 180_000,
            });
        shell.active_screen = ShellScreen::Log;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("win 1.25-3.75s src-1"), "{rendered}");
        assert_eq!(w30_capture_log_compact(&shell), "win 1.25-3.75s src-1");
    }

    #[test]
    fn renders_capture_do_next_with_pending_capture_state() {
        let first_run_shell = first_run_shell_state();
        let mut shell = JamShellState::new(first_run_shell.app, ShellLaunchMode::Load);
        shell.app.queue_capture_bar(240);
        shell.active_screen = ShellScreen::Capture;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(
            rendered.contains("queued [c] capture @ next_phrase"),
            "{rendered}"
        );
        assert!(
            rendered.contains("then [o] audition raw or [p] promote"),
            "{rendered}"
        );
        assert!(rendered.contains("[2] confirm capture"), "{rendered}");
    }

    #[test]
    fn renders_capture_shell_snapshot_with_raw_capture_audition_cue() {
        let mut shell = sample_shell_without_pending_queue();
        shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
        shell.app.refresh_view();
        assert_eq!(
            shell.app.queue_w30_audition(260),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        shell.active_screen = ShellScreen::Capture;

        let rendered = render_jam_shell_snapshot(&shell, 160, 34);

        assert!(rendered.contains("pending W-30 cue audition"), "{rendered}");
        assert!(rendered.contains("bank-a/pad-01"), "{rendered}");
        assert!(rendered.contains("queued [o] audition raw @"), "{rendered}");
        assert!(
            rendered.contains("wait, then hear raw preview"),
            "{rendered}"
        );
        assert!(
            rendered.contains("hear cap-01 stored fallback [o] raw or [p]->[w]"),
            "{rendered}"
        );

        shell.active_screen = ShellScreen::Log;
        let rendered_log = render_jam_shell_snapshot(&shell, 160, 34);
        assert!(
            rendered_log.contains("w30.audition_raw_capture"),
            "{rendered_log}"
        );
    }

    #[test]
    fn committed_raw_capture_audition_surfaces_source_fallback_readiness() {
        let mut shell = sample_shell_without_pending_queue();
        shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
        shell.app.refresh_view();
        assert_eq!(
            shell.app.queue_w30_audition(260),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        shell.app.commit_ready_actions(
            CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Bar,
                beat_index: 33,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            320,
        );
        shell.active_screen = ShellScreen::Jam;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("current preview audition"), "{rendered}");
        assert!(rendered.contains("raw/fallback"), "{rendered}");

        shell.active_screen = ShellScreen::Capture;
        let rendered = render_jam_shell_snapshot(&shell, 120, 34);
        assert!(rendered.contains("| fallback"), "{rendered}");
    }

    #[test]
    fn source_backed_raw_capture_audition_compact_label_uses_src_cue() {
        let mut shell = sample_shell_without_pending_queue();
        shell.app.session.runtime_state.lane_state.w30.preview_mode =
            Some(riotbox_core::session::W30PreviewModeState::RawCaptureAudition);
        shell.app.refresh_view();
        shell.app.runtime.w30_preview.source_window_preview =
            Some(riotbox_audio::w30::W30PreviewSampleWindow {
                source_start_frame: 0,
                source_end_frame: 64,
                sample_count: 64,
                samples: [0.0; riotbox_audio::w30::W30_PREVIEW_SAMPLE_WINDOW_LEN],
            });

        assert_eq!(w30_preview_mode_profile_compact(&shell), "audition raw/src");
        assert_eq!(w30_preview_source_readiness(&shell), Some("source-backed"));
    }

    #[test]
    fn source_backed_promoted_and_recall_compact_labels_use_src_cue() {
        let mut shell = sample_shell_without_pending_queue();
        let sample_window = riotbox_audio::w30::W30PreviewSampleWindow {
            source_start_frame: 0,
            source_end_frame: 64,
            sample_count: 64,
            samples: [0.0; riotbox_audio::w30::W30_PREVIEW_SAMPLE_WINDOW_LEN],
        };

        shell.app.runtime.w30_preview.mode = W30PreviewRenderMode::PromotedAudition;
        shell.app.runtime.w30_preview.source_profile =
            Some(riotbox_audio::w30::W30PreviewSourceProfile::PromotedAudition);
        shell.app.runtime.w30_preview.source_window_preview = Some(sample_window.clone());

        assert_eq!(w30_preview_mode_profile_compact(&shell), "audition/src");
        assert_eq!(w30_preview_log_compact(&shell), "audition/src");
        assert_eq!(w30_preview_source_readiness(&shell), Some("source-backed"));

        shell.app.runtime.w30_preview.mode = W30PreviewRenderMode::LiveRecall;
        shell.app.runtime.w30_preview.source_profile =
            Some(riotbox_audio::w30::W30PreviewSourceProfile::PromotedRecall);
        shell.app.runtime.w30_preview.source_window_preview = Some(sample_window);

        assert_eq!(
            w30_preview_mode_profile_compact(&shell),
            "recall/promoted/src"
        );
        assert_eq!(w30_preview_log_compact(&shell), "recall/src");
        assert_eq!(w30_preview_source_readiness(&shell), Some("source-backed"));
    }

    #[test]
    fn renders_capture_pending_cues_panel_as_first_item_with_log_overflow() {
        let first_run_shell = first_run_shell_state();
        let mut shell = JamShellState::new(first_run_shell.app, ShellLaunchMode::Load);
        shell.app.queue_capture_bar(240);
        shell.app.queue_capture_bar(241);

        let lines = pending_capture_lines(&shell);
        let rendered: Vec<String> = lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect();

        assert_eq!(rendered[0], "next user capture.bar_group");
        assert_eq!(rendered[1], "when next_phrase | target lanew30");
        assert_eq!(rendered[2], "note capture next phrase into W-30 path");
        assert_eq!(rendered[3], "+1 more in [2] Log");
        assert_eq!(rendered.len(), 4);
    }

    #[test]
    fn renders_capture_shell_snapshot_with_w30_live_recall_cue() {
        let mut shell = sample_shell_state();
        shell.app.session.captures[0].assigned_target =
            Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-03".into(),
            });
        shell.app.session.captures[0].is_pinned = true;
        shell.app.refresh_view();
        assert_eq!(
            shell.app.queue_w30_live_recall(200),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        shell.active_screen = ShellScreen::Capture;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("pending W-30 cue"));
        assert!(rendered.contains("recall"));
    }

    #[test]
    fn renders_capture_shell_snapshot_with_w30_trigger_cue() {
        let mut shell = sample_shell_state();
        shell.app.session.captures[0].assigned_target =
            Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-a".into(),
                pad_id: "pad-01".into(),
            });
        shell.app.refresh_view();
        assert_eq!(
            shell.app.queue_w30_trigger_pad(205),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        shell.active_screen = ShellScreen::Capture;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("pending W-30 cue"));
        assert!(rendered.contains("trigger"));
        assert!(rendered.contains("bank-a/pad-01"));
    }

    #[test]
    fn renders_capture_shell_snapshot_with_w30_step_cue() {
        let mut shell = sample_shell_state();
        shell.app.session.captures[0].assigned_target =
            Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-a".into(),
                pad_id: "pad-01".into(),
            });
        shell
            .app
            .session
            .captures
            .push(riotbox_core::session::CaptureRef {
                capture_id: "cap-02".into(),
                capture_type: riotbox_core::session::CaptureType::Pad,
                source_origin_refs: vec!["asset-b".into()],
                source_window: None,
                lineage_capture_refs: Vec::new(),
                resample_generation_depth: 0,
                created_from_action: None,
                storage_path: "captures/cap-02.wav".into(),
                assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                    bank_id: "bank-b".into(),
                    pad_id: "pad-04".into(),
                }),
                is_pinned: false,
                notes: Some("secondary".into()),
            });
        shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
        shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
        shell.app.refresh_view();
        assert_eq!(
            shell.app.queue_w30_step_focus(207),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        shell.active_screen = ShellScreen::Capture;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("pending W-30 cue"));
        assert!(rendered.contains("step"));
        assert!(rendered.contains("bank-b/pad-04"));
    }

    #[test]
    fn renders_capture_shell_snapshot_with_w30_bank_swap_cue() {
        let mut shell = sample_shell_state();
        shell.app.session.captures[0].assigned_target =
            Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-a".into(),
                pad_id: "pad-01".into(),
            });
        shell
            .app
            .session
            .captures
            .push(riotbox_core::session::CaptureRef {
                capture_id: "cap-02".into(),
                capture_type: riotbox_core::session::CaptureType::Pad,
                source_origin_refs: vec!["asset-b".into()],
                source_window: None,
                lineage_capture_refs: Vec::new(),
                resample_generation_depth: 0,
                created_from_action: None,
                storage_path: "captures/cap-02.wav".into(),
                assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                    bank_id: "bank-b".into(),
                    pad_id: "pad-01".into(),
                }),
                is_pinned: false,
                notes: Some("bank b".into()),
            });
        shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
        shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
        shell.app.refresh_view();
        assert_eq!(
            shell.app.queue_w30_swap_bank(208),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        shell.active_screen = ShellScreen::Capture;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("pending W-30 cue"));
        assert!(rendered.contains("bank"));
        assert!(rendered.contains("bank-b/pad-01"));
        assert!(rendered.contains("pending W-30 cue bank"), "{rendered}");
        assert!(rendered.contains("mgr next bank-b/pad-01"), "{rendered}");
    }

    #[test]
    fn renders_capture_shell_snapshot_with_w30_slice_pool_browse_cue() {
        let mut shell = sample_shell_state();
        shell.app.session.captures[0].assigned_target =
            Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-a".into(),
                pad_id: "pad-01".into(),
            });
        shell
            .app
            .session
            .captures
            .push(riotbox_core::session::CaptureRef {
                capture_id: "cap-02".into(),
                capture_type: riotbox_core::session::CaptureType::Pad,
                source_origin_refs: vec!["asset-b".into()],
                source_window: None,
                lineage_capture_refs: vec!["cap-01".into()],
                resample_generation_depth: 0,
                created_from_action: None,
                storage_path: "captures/cap-02.wav".into(),
                assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                    bank_id: "bank-a".into(),
                    pad_id: "pad-01".into(),
                }),
                is_pinned: false,
                notes: Some("alt slice".into()),
            });
        shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
        shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
        shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
        shell.app.refresh_view();
        assert_eq!(
            shell.app.queue_w30_browse_slice_pool(209),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        shell.active_screen = ShellScreen::Capture;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("pending W-30 cue"));
        assert!(rendered.contains("browse"));
        assert!(rendered.contains("bank-a/pad-01"), "{rendered}");
        assert!(rendered.contains("bank/pad bank-a/pad-01"), "{rendered}");
        assert!(rendered.contains("pool cap-01 1/2 -> cap-02"), "{rendered}");
    }

    #[test]
    fn renders_log_shell_snapshot_with_committed_w30_slice_pool_browse_diagnostics() {
        let mut shell = sample_shell_state();
        shell.app.session.captures[0].assigned_target =
            Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-a".into(),
                pad_id: "pad-01".into(),
            });
        shell
            .app
            .session
            .captures
            .push(riotbox_core::session::CaptureRef {
                capture_id: "cap-02".into(),
                capture_type: riotbox_core::session::CaptureType::Pad,
                source_origin_refs: vec!["asset-b".into()],
                source_window: None,
                lineage_capture_refs: vec!["cap-01".into()],
                resample_generation_depth: 0,
                created_from_action: None,
                storage_path: "captures/cap-02.wav".into(),
                assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                    bank_id: "bank-a".into(),
                    pad_id: "pad-01".into(),
                }),
                is_pinned: false,
                notes: Some("alt slice".into()),
            });
        shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
        shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
        shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
        shell.app.refresh_view();
        assert_eq!(
            shell.app.queue_w30_browse_slice_pool(320),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        shell.app.commit_ready_actions(
            riotbox_core::transport::CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Beat,
                beat_index: 42,
                bar_index: 11,
                phrase_index: 3,
                scene_id: Some("scene-1".into()),
            },
            420,
        );
        shell.active_screen = ShellScreen::Log;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("cue idle | browse"), "{rendered}");
        assert!(rendered.contains("bank bank-a/pad-01"), "{rendered}");
        assert!(rendered.contains("tap cap-02 g0/l1 int"), "{rendered}");
    }

    #[test]
    fn renders_capture_shell_snapshot_with_w30_damage_profile_cue() {
        let mut shell = sample_shell_state();
        shell.app.session.captures[0].assigned_target =
            Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-a".into(),
                pad_id: "pad-01".into(),
            });
        shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
        shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
        shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
        shell.app.refresh_view();
        assert_eq!(
            shell.app.queue_w30_apply_damage_profile(210),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        shell.active_screen = ShellScreen::Capture;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("pending W-30 cue"));
        assert!(rendered.contains("damage"));
        assert!(rendered.contains("bank-a/pad-01"));
        assert!(rendered.contains("next bank-a/pad-01"), "{rendered}");
    }

    #[test]
    fn renders_w30_bank_manager_and_damage_profile_diagnostics_across_shell_surfaces() {
        let mut shell = sample_shell_state();
        shell.app.queue = ActionQueue::new();
        shell.app.session.captures[0].assigned_target =
            Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-a".into(),
                pad_id: "pad-01".into(),
            });
        shell
            .app
            .session
            .captures
            .push(riotbox_core::session::CaptureRef {
                capture_id: "cap-02".into(),
                capture_type: riotbox_core::session::CaptureType::Pad,
                source_origin_refs: vec!["asset-b".into()],
                source_window: None,
                lineage_capture_refs: Vec::new(),
                resample_generation_depth: 0,
                created_from_action: None,
                storage_path: "captures/cap-02.wav".into(),
                assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                    bank_id: "bank-b".into(),
                    pad_id: "pad-01".into(),
                }),
                is_pinned: false,
                notes: Some("bank b".into()),
            });
        shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
        shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
        shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
        shell.app.refresh_view();

        assert_eq!(
            shell.app.queue_w30_swap_bank(208),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        let committed = shell.app.commit_ready_actions(
            CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Bar,
                beat_index: 17,
                bar_index: 5,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-a")),
            },
            220,
        );
        assert_eq!(committed.len(), 1);

        assert_eq!(
            shell.app.queue_w30_apply_damage_profile(222),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        let committed = shell.app.commit_ready_actions(
            CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Bar,
                beat_index: 21,
                bar_index: 6,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-a")),
            },
            240,
        );
        assert_eq!(committed.len(), 1);

        let jam_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        assert!(
            jam_rendered.contains("current pad bank-b/pad-01"),
            "{jam_rendered}"
        );
        assert!(jam_rendered.contains("next swap+shred"), "{jam_rendered}");

        shell.active_screen = ShellScreen::Capture;
        let capture_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        assert!(
            capture_rendered.contains("mgr bank-b/pad-01"),
            "{capture_rendered}"
        );
        assert!(
            capture_rendered.contains("forge bank-b/pad-01"),
            "{capture_rendered}"
        );

        shell.active_screen = ShellScreen::Log;
        let log_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        assert!(
            log_rendered.contains("bank bank-b/pad-01"),
            "{log_rendered}"
        );
        assert!(log_rendered.contains("cue idle | damage"), "{log_rendered}");
        assert!(log_rendered.contains("mix 0.64/0.82"), "{log_rendered}");
        assert!(log_rendered.contains("swap+shred"), "{log_rendered}");
    }

    #[test]
    fn w30_operation_diagnostics_follow_current_lane_target() {
        let mut shell = sample_shell_state();
        shell.app.queue = ActionQueue::new();
        shell.app.session.captures[0].assigned_target =
            Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-a".into(),
                pad_id: "pad-01".into(),
            });
        shell
            .app
            .session
            .captures
            .push(riotbox_core::session::CaptureRef {
                capture_id: "cap-02".into(),
                capture_type: riotbox_core::session::CaptureType::Pad,
                source_origin_refs: vec!["asset-b".into()],
                source_window: None,
                lineage_capture_refs: Vec::new(),
                resample_generation_depth: 0,
                created_from_action: None,
                storage_path: "captures/cap-02.wav".into(),
                assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                    bank_id: "bank-b".into(),
                    pad_id: "pad-01".into(),
                }),
                is_pinned: false,
                notes: Some("bank b".into()),
            });
        shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-b".into());
        shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
        shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-02".into());
        shell.app.refresh_view();

        assert_eq!(
            shell.app.queue_w30_swap_bank(208),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        let committed = shell.app.commit_ready_actions(
            CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Bar,
                beat_index: 17,
                bar_index: 5,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-a")),
            },
            220,
        );
        assert_eq!(committed.len(), 1);

        assert_eq!(
            shell.app.queue_w30_apply_damage_profile(222),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        let committed = shell.app.commit_ready_actions(
            CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Bar,
                beat_index: 21,
                bar_index: 6,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-a")),
            },
            240,
        );
        assert_eq!(committed.len(), 1);

        assert_eq!(
            shell.app.queue_w30_loop_freeze(245),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        let committed = shell.app.commit_ready_actions(
            CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Phrase,
                beat_index: 29,
                bar_index: 8,
                phrase_index: 3,
                scene_id: Some(SceneId::from("scene-a")),
            },
            260,
        );
        assert_eq!(committed.len(), 1);

        shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-c".into());
        shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
        shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
        shell.app.refresh_view();

        let jam_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        assert!(
            jam_rendered.contains("current pad bank-c/pad-01"),
            "{jam_rendered}"
        );
        assert!(jam_rendered.contains("next idle"), "{jam_rendered}");

        shell.active_screen = ShellScreen::Capture;
        let capture_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        assert!(
            capture_rendered.contains("bank/pad bank-c/pad-01"),
            "{capture_rendered}"
        );
        assert!(capture_rendered.contains("mgr idle"), "{capture_rendered}");
        assert!(
            capture_rendered.contains("forge idle"),
            "{capture_rendered}"
        );
        assert!(
            capture_rendered.contains("freeze idle"),
            "{capture_rendered}"
        );

        shell.active_screen = ShellScreen::Log;
        let log_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        assert!(
            log_rendered.contains("mix 0.64/0.82 idle"),
            "{log_rendered}"
        );
    }

    #[test]
    fn renders_capture_shell_snapshot_with_w30_audition_cue() {
        let mut shell = sample_shell_without_pending_queue();
        shell.app.session.captures[0].assigned_target =
            Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-03".into(),
            });
        shell.app.refresh_view();
        assert_eq!(
            shell.app.queue_w30_promoted_audition(210),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        shell.active_screen = ShellScreen::Capture;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("pending W-30 cue"));
        assert!(rendered.contains("audition"));
        assert!(rendered.contains("[w]/[o]"), "{rendered}");
        assert!(rendered.contains("queued [o] audition pad @"), "{rendered}");
        assert!(
            rendered.contains("wait, then hear promoted preview"),
            "{rendered}"
        );
        assert_eq!(
            shell
                .app
                .jam_view
                .capture
                .latest_w30_promoted_capture_label
                .as_deref(),
            Some("cap-01 -> bank-b/pad-03")
        );
        assert!(rendered.contains("latest promoted cap-01 ->"), "{rendered}");
        assert!(rendered.contains("cap-01"));
    }

    #[test]
    fn renders_capture_heard_path_for_scene_targets_without_w30_audition_keys() {
        let mut shell = sample_shell_without_pending_queue();
        shell.app.session.captures[0].assigned_target =
            Some(riotbox_core::session::CaptureTarget::Scene("drop-1".into()));
        shell.app.refresh_view();
        shell.active_screen = ShellScreen::Capture;

        assert_eq!(
            shell.app.jam_view.capture.last_capture_target_kind,
            Some(CaptureTargetKindView::Scene)
        );
        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(
            rendered.contains("hear cap-01->scene drop-1 ready"),
            "{rendered}"
        );
        assert!(rendered.contains("scene target scene drop-1"), "{rendered}");
    }

    #[test]
    fn renders_capture_handoff_source_readiness_for_w30_targets() {
        let mut shell = sample_shell_without_pending_queue();
        shell.app.session.captures[0].assigned_target =
            Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-03".into(),
            });
        shell.app.session.captures[0].source_window =
            Some(riotbox_core::session::CaptureSourceWindow {
                source_id: SourceId::from("src-1"),
                start_seconds: 1.25,
                end_seconds: 3.75,
                start_frame: 60_000,
                end_frame: 180_000,
            });
        shell.app.refresh_view();
        shell.active_screen = ShellScreen::Capture;

        assert_eq!(
            shell.app.jam_view.capture.last_capture_handoff_readiness,
            Some(CaptureHandoffReadinessView::Source)
        );
        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(
            rendered.contains("hear cap-01->pad bank-b/pad-03"),
            "{rendered}"
        );
        assert!(rendered.contains("[w]/[o] src"), "{rendered}");
        assert!(
            rendered.contains("hear now: [w] hit pad bank-b/pad-03"),
            "{rendered}"
        );
        assert!(rendered.contains("(src)"), "{rendered}");
    }

    #[test]
    fn renders_capture_shell_snapshot_with_w30_resample_cue() {
        let mut shell = sample_shell_state();
        shell.app.session.captures[0].assigned_target =
            Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-03".into(),
            });
        shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-b".into());
        shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-03".into());
        shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
        shell.app.refresh_view();
        assert_eq!(
            shell.app.queue_w30_internal_resample(215),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        shell.active_screen = ShellScreen::Capture;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("pending W-30 cue"));
        assert!(rendered.contains("+1 more in [2] Log"));
        assert!(rendered.contains("resample"));
        assert!(rendered.contains("cap-01"));
    }

    #[test]
    fn renders_capture_shell_snapshot_with_committed_w30_resample_lineage_diagnostics() {
        let mut shell = sample_shell_state();
        shell.app.queue = ActionQueue::new();
        shell.app.session.captures[0].assigned_target =
            Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-03".into(),
            });
        shell.app.session.captures[0].lineage_capture_refs = vec!["cap-root".into()];
        shell.app.session.captures[0].resample_generation_depth = 1;
        shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-b".into());
        shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-03".into());
        shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
        shell.app.refresh_view();
        assert_eq!(
            shell.app.queue_w30_internal_resample(220),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        let committed = shell.app.commit_ready_actions(
            CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Phrase,
                beat_index: 33,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-a")),
            },
            240,
        );
        assert_eq!(committed.len(), 1);
        shell.active_screen = ShellScreen::Capture;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(
            rendered.contains("forge idle | tap ready/raw"),
            "{rendered}"
        );
        assert!(rendered.contains("g2"), "{rendered}");
        assert!(rendered.contains("lineage"));
        assert!(
            rendered.contains("cap-root>cap-01>cap-02 | g2"),
            "{rendered}"
        );
        assert!(rendered.contains("tap src cap-02 g2/l2 |"), "{rendered}");
        assert!(rendered.contains("route internal"), "{rendered}");
        assert!(rendered.contains("tap mix 0.64/0.50"), "{rendered}");
        assert!(
            rendered.matches("latest promoted").count() <= 1,
            "{rendered}"
        );
    }

    #[test]
    fn renders_log_shell_snapshot_with_committed_w30_audition_diagnostics() {
        let mut shell = sample_shell_state();
        shell.app.queue = ActionQueue::new();
        shell.app.session.captures[0].assigned_target =
            Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-03".into(),
            });
        shell.app.refresh_view();
        assert_eq!(
            shell.app.queue_w30_promoted_audition(220),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        let committed = shell.app.commit_ready_actions(
            CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Bar,
                beat_index: 33,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-a")),
            },
            240,
        );
        assert_eq!(committed.len(), 1);
        shell.active_screen = ShellScreen::Log;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("W-30 Lane"));
        assert!(rendered.contains("cue idle"));
        assert!(rendered.contains("auditioned cap-01"));
        assert!(rendered.contains("bank-b"));
        assert!(rendered.contains("pad-03"));
        assert!(rendered.contains("cue idle | audition"));
        assert!(rendered.contains("prev audition/fallback"));
        assert!(rendered.contains("mix 0.64/0.68"));
        assert!(rendered.contains("cap cap-01 | pending"), "{rendered}");
    }

    #[test]
    fn renders_log_shell_snapshot_with_committed_w30_trigger_preview_diagnostics() {
        let mut shell = sample_shell_state();
        shell.app.queue = ActionQueue::new();
        shell.app.session.captures[0].assigned_target =
            Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-a".into(),
                pad_id: "pad-01".into(),
            });
        shell.app.refresh_view();
        assert_eq!(
            shell.app.queue_w30_trigger_pad(230),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        let committed = shell.app.commit_ready_actions(
            CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Beat,
                beat_index: 34,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-a")),
            },
            250,
        );
        assert_eq!(committed.len(), 1);
        shell.active_screen = ShellScreen::Log;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("W-30 Lane"));
        assert!(rendered.contains("cue idle | trigger"));
        assert!(rendered.contains("prev recall/fallback"));
        assert!(rendered.contains("mix 0.64/0.69"));
        assert!(rendered.contains("cap cap-01 | r1@0.84"), "{rendered}");
    }

    #[test]
    fn renders_log_shell_snapshot_with_committed_w30_resample_lineage_diagnostics() {
        let mut shell = sample_shell_state();
        shell.app.queue = ActionQueue::new();
        shell.app.session.captures[0].assigned_target =
            Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-03".into(),
            });
        shell.app.session.captures[0].lineage_capture_refs = vec!["cap-root".into()];
        shell.app.session.captures[0].resample_generation_depth = 1;
        shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-b".into());
        shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-03".into());
        shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
        shell.app.refresh_view();
        assert_eq!(
            shell.app.queue_w30_internal_resample(245),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        let committed = shell.app.commit_ready_actions(
            CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Phrase,
                beat_index: 34,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-a")),
            },
            260,
        );
        assert_eq!(committed.len(), 1);
        shell.active_screen = ShellScreen::Log;

        let rendered = render_jam_shell_snapshot(&shell, 120, 34);

        assert!(rendered.contains("W-30 Lane"));
        assert!(rendered.contains("cue idle | resample"));
        assert!(rendered.contains("tapmix 0.64/0.50"), "{rendered}");
        assert!(rendered.contains("tap cap-02 g2/l2 int"), "{rendered}");
    }

    #[test]
    fn renders_w30_resample_lab_diagnostics_across_shell_surfaces() {
        let mut shell = sample_shell_state();
        shell.app.queue = ActionQueue::new();
        shell.app.session.captures[0].assigned_target =
            Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-03".into(),
            });
        shell.app.session.captures[0].lineage_capture_refs = vec!["cap-root".into()];
        shell.app.session.captures[0].resample_generation_depth = 1;
        shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-b".into());
        shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-03".into());
        shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
        shell.app.refresh_view();

        assert_eq!(
            shell.app.queue_w30_internal_resample(265),
            Some(crate::jam_app::QueueControlResult::Enqueued)
        );
        let committed = shell.app.commit_ready_actions(
            CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Phrase,
                beat_index: 36,
                bar_index: 10,
                phrase_index: 3,
                scene_id: Some(SceneId::from("scene-a")),
            },
            280,
        );
        assert_eq!(committed.len(), 1);

        let jam_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        assert!(
            jam_rendered.contains("current pad bank-b/pad-03"),
            "{jam_rendered}"
        );
        assert!(jam_rendered.contains("next idle"), "{jam_rendered}");

        shell.active_screen = ShellScreen::Capture;
        let capture_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        assert!(
            capture_rendered.contains("tap src cap-02 g2/l2 |"),
            "{capture_rendered}"
        );
        assert!(
            capture_rendered.contains("route internal"),
            "{capture_rendered}"
        );
        assert!(
            capture_rendered.contains("tap mix 0.64/0.50"),
            "{capture_rendered}"
        );

        shell.active_screen = ShellScreen::Log;
        let log_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        assert!(
            log_rendered.contains("tap cap-02 g2/l2 int"),
            "{log_rendered}"
        );
        assert!(log_rendered.contains("tapmix 0.64/0.50"), "{log_rendered}");
    }
}
