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
            KeyCode::Char('P') => {
                self.status_message = queued_status_message(GESTURE_PRESSURE, "next phrase");
                ShellKeyOutcome::QueueMc202GeneratePressure
            }
            KeyCode::Char('I') => {
                self.status_message = queued_status_message(GESTURE_INSTIGATE, "next phrase");
                ShellKeyOutcome::QueueMc202GenerateInstigator
            }
            KeyCode::Char('G') => {
                self.status_message = queued_status_message(GESTURE_PHRASE, "next phrase");
                ShellKeyOutcome::QueueMc202MutatePhrase
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
            KeyCode::Char('<') => {
                self.status_message = "lower MC-202 touch".into();
                ShellKeyOutcome::LowerMc202Touch
            }
            KeyCode::Char('>') => {
                self.status_message = "raise MC-202 touch".into();
                ShellKeyOutcome::RaiseMc202Touch
            }
            KeyCode::Enter => {
                self.status_message = "accept ghost suggestion requested".into();
                ShellKeyOutcome::AcceptCurrentGhostSuggestion
            }
            KeyCode::Char('N') => {
                self.status_message = "reject ghost suggestion requested".into();
                ShellKeyOutcome::RejectCurrentGhostSuggestion
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
            "Mode {} | Screen {} | Source {} | {} | trust {} | feral {}",
            shell.launch_mode.label(),
            screen_context_label(shell),
            source.source_id,
            bpm_text,
            trust.headline,
            source.feral_scorecard.readiness
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
