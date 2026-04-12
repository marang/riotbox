use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::jam_app::JamAppState;

pub fn render_jam_shell(frame: &mut Frame<'_>, state: &JamAppState) {
    let area = frame.area();
    let columns = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Min(8),
            Constraint::Length(5),
        ])
        .split(area);

    render_header(frame, columns[0], state);
    render_status_row(frame, columns[1], state);
    render_macro_row(frame, columns[2], state);
    render_action_rows(frame, columns[3], state);
    render_footer(frame, columns[4], state);
}

fn render_header(frame: &mut Frame<'_>, area: Rect, state: &JamAppState) {
    let source = &state.jam_view.source;
    let bpm_text = source
        .bpm_estimate
        .map(|bpm| format!("{bpm:.1} BPM"))
        .unwrap_or_else(|| "unknown BPM".into());
    let scene_text = state
        .jam_view
        .scene
        .active_scene
        .as_deref()
        .unwrap_or("no active scene");

    let paragraph = Paragraph::new(vec![
        Line::from("Riotbox Jam"),
        Line::from(format!(
            "Source {} | {} | sections {} | loops {} | hooks {}",
            source.source_id,
            bpm_text,
            source.section_count,
            source.loop_candidate_count,
            source.hook_candidate_count
        )),
        Line::from(format!(
            "Transport {} at beat {:.1} | scene {}",
            if state.jam_view.transport.is_playing {
                "playing"
            } else {
                "idle"
            },
            state.jam_view.transport.position_beats,
            scene_text
        )),
    ])
    .block(Block::default().title("Jam").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn render_status_row(frame: &mut Frame<'_>, area: Rect, state: &JamAppState) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(area);

    let runtime = Paragraph::new(vec![
        Line::from(format!("Audio: {}", state.runtime_view.audio_status)),
        Line::from(format!("Sidecar: {}", state.runtime_view.sidecar_status)),
        Line::from(format!(
            "Ghost: {} ({})",
            state.jam_view.ghost.mode,
            if state.jam_view.ghost.is_blocked {
                "blocked"
            } else {
                "clear"
            }
        )),
    ])
    .block(Block::default().title("Runtime").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    let lanes = Paragraph::new(vec![
        Line::from(format!(
            "MC-202 role: {}",
            state
                .jam_view
                .lanes
                .mc202_role
                .as_deref()
                .unwrap_or("unset")
        )),
        Line::from(format!(
            "W-30 bank: {}",
            state
                .jam_view
                .lanes
                .w30_active_bank
                .as_deref()
                .unwrap_or("unset")
        )),
        Line::from(format!(
            "TR-909 slam: {}",
            if state.jam_view.lanes.tr909_slam_enabled {
                "on"
            } else {
                "off"
            }
        )),
    ])
    .block(Block::default().title("Lanes").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    let source = Paragraph::new(vec![
        Line::from(format!("Source refs: {}", state.session.source_refs.len())),
        Line::from(format!(
            "Graph refs: {}",
            state.session.source_graph_refs.len()
        )),
        Line::from(format!(
            "Ghost suggestions: {}",
            state.jam_view.ghost.suggestion_count
        )),
    ])
    .block(Block::default().title("Session").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    frame.render_widget(runtime, columns[0]);
    frame.render_widget(lanes, columns[1]);
    frame.render_widget(source, columns[2]);
}

fn render_macro_row(frame: &mut Frame<'_>, area: Rect, state: &JamAppState) {
    let macros = &state.jam_view.macros;
    let paragraph = Paragraph::new(vec![
        Line::from(format!(
            "retain {:.2} | chaos {:.2} | mc202 {:.2}",
            macros.source_retain, macros.chaos, macros.mc202_touch
        )),
        Line::from(format!(
            "w30 grit {:.2} | tr909 slam {:.2}",
            macros.w30_grit, macros.tr909_slam
        )),
    ])
    .block(Block::default().title("Macros").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn render_action_rows(frame: &mut Frame<'_>, area: Rect, state: &JamAppState) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let pending_items = if state.jam_view.pending_actions.is_empty() {
        vec![ListItem::new("no pending actions")]
    } else {
        state
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

    let recent_items = if state.jam_view.recent_actions.is_empty() {
        vec![ListItem::new("no committed actions yet")]
    } else {
        state
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

fn render_footer(frame: &mut Frame<'_>, area: Rect, state: &JamAppState) {
    let warning_lines =
        if state.runtime_view.runtime_warnings.is_empty() && state.jam_view.warnings.is_empty() {
            vec![Line::from("Warnings clear | q to quit")]
        } else {
            let mut lines = Vec::new();
            for warning in state
                .runtime_view
                .runtime_warnings
                .iter()
                .chain(state.jam_view.warnings.iter())
            {
                lines.push(Line::from(warning.clone()));
            }
            lines.push(Line::from("q to quit"));
            lines
        };

    let paragraph = Paragraph::new(warning_lines)
        .block(
            Block::default()
                .title(Line::from("Warnings").style(Style::default().add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend};
    use riotbox_core::{
        ids::SourceId,
        queue::ActionQueue,
        session::SessionFile,
        source_graph::{DecodeProfile, GraphProvenance, SourceDescriptor, SourceGraph},
    };

    use crate::jam_app::JamAppState;

    use super::*;

    fn sample_state() -> JamAppState {
        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-12T00:00:00Z");
        session.runtime_state.transport.position_beats = 32.0;
        session.runtime_state.macro_state.source_retain = 0.7;
        session.runtime_state.macro_state.chaos = 0.2;
        session.runtime_state.macro_state.mc202_touch = 0.8;
        session.runtime_state.macro_state.w30_grit = 0.5;
        session.runtime_state.macro_state.tr909_slam = 0.9;
        session.runtime_state.lane_state.mc202.role = Some("leader".into());
        session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
        session.ghost_state.mode = riotbox_core::action::GhostMode::Assist;

        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: SourceId::from("src-1"),
                path: "input.wav".into(),
                content_hash: "hash-1".into(),
                duration_seconds: 2.0,
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
        graph.sections.push(riotbox_core::source_graph::Section {
            section_id: "section-a".into(),
            label_hint: riotbox_core::source_graph::SectionLabelHint::Intro,
            start_seconds: 0.0,
            end_seconds: 2.0,
            bar_start: 1,
            bar_end: 2,
            energy_class: riotbox_core::source_graph::EnergyClass::Medium,
            confidence: 0.7,
            tags: vec!["decoded_wave".into()],
        });
        graph
            .analysis_summary
            .warnings
            .push(riotbox_core::source_graph::AnalysisWarning {
                code: "wav_baseline_only".into(),
                message: "decoded-source baseline used WAV metadata and simple energy heuristics"
                    .into(),
            });

        JamAppState::from_parts(session, Some(graph), ActionQueue::new())
    }

    #[test]
    fn renders_minimal_jam_shell_into_test_backend() {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).expect("create terminal");
        let state = sample_state();

        terminal
            .draw(|frame| render_jam_shell(frame, &state))
            .expect("draw frame");

        let backend = terminal.backend();
        let buffer = backend.buffer();
        let rendered = buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains("Riotbox Jam"));
        assert!(rendered.contains("Runtime"));
        assert!(rendered.contains("Pending"));
        assert!(rendered.contains("Warnings"));
    }
}
