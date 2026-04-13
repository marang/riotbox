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
    QueueCaptureBar,
    UndoLast,
    Quit,
}

#[derive(Clone, Debug)]
pub struct JamShellState {
    pub app: JamAppState,
    pub launch_mode: ShellLaunchMode,
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
            show_help: false,
            status_message,
        }
    }

    pub fn handle_key_code(&mut self, code: KeyCode) -> ShellKeyOutcome {
        match code {
            KeyCode::Esc | KeyCode::Char('q') => ShellKeyOutcome::Quit,
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
            Constraint::Length(7),
            Constraint::Length(9),
            Constraint::Min(7),
            Constraint::Length(5),
        ])
        .split(area);

    render_header(frame, rows[0], shell);
    render_overview_row(frame, rows[1], shell);
    render_source_row(frame, rows[2], shell);
    render_action_rows(frame, rows[3], shell);
    render_footer(frame, rows[4], shell);

    if shell.show_help {
        render_help_overlay(frame, area, shell);
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
            "Mode {} | Source {} | {} | trust {}",
            shell.launch_mode.label(),
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
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
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

    frame.render_widget(pending, columns[0]);
    frame.render_widget(recent, columns[1]);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, shell: &JamShellState) {
    let mut lines = Vec::new();
    lines.push(Line::from(format!(
        "Keys: q quit | ? help | space play/pause | r {}",
        shell.launch_mode.refresh_verb()
    )));
    lines.push(Line::from(
        "Actions: m mutate scene | f TR-909 fill | c capture phrase | u undo",
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
        Line::from("space: play / pause transport"),
        Line::from(format!("r: {}", shell.launch_mode.refresh_verb())),
        Line::from("m: queue scene mutation on next bar"),
        Line::from("f: queue TR-909 fill on next bar"),
        Line::from("c: queue phrase capture on next phrase"),
        Line::from("u: undo most recent undoable action"),
        Line::from(""),
        Line::from(format!("Current mode: {}", shell.launch_mode.label())),
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
        assert!(rendered.contains("committed"));
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
            shell.handle_key_code(KeyCode::Char('m')),
            ShellKeyOutcome::QueueSceneMutation
        );
        assert_eq!(
            shell.handle_key_code(KeyCode::Char('f')),
            ShellKeyOutcome::QueueTr909Fill
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
}
