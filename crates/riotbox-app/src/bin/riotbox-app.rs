use std::{
    env,
    io::{self, stdout},
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use riotbox_app::{
    jam_app::{JamAppError, JamAppState},
    ui::{JamShellState, ShellKeyOutcome, ShellLaunchMode, render_jam_shell},
};

const DEFAULT_SESSION_PATH: &str = "data/sessions/jam-session.json";
const DEFAULT_SIDECAR_PATH: &str = "python/sidecar/json_stdio_sidecar.py";
const UI_TICK: Duration = Duration::from_millis(200);

#[derive(Clone, Debug)]
enum LaunchMode {
    Load {
        session_path: PathBuf,
        source_graph_path: Option<PathBuf>,
    },
    Ingest {
        source_path: PathBuf,
        session_path: PathBuf,
        source_graph_path: Option<PathBuf>,
        sidecar_script_path: PathBuf,
        analysis_seed: u64,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mode = parse_args(env::args().skip(1))?;
    let state = load_state(mode.clone())?;
    run_terminal_ui(JamShellState::new(state, mode.shell_launch_mode()), mode)?;
    Ok(())
}

fn load_state(mode: LaunchMode) -> Result<JamAppState, JamAppError> {
    match mode {
        LaunchMode::Load {
            session_path,
            source_graph_path,
        } => JamAppState::from_json_files(session_path, source_graph_path),
        LaunchMode::Ingest {
            source_path,
            session_path,
            source_graph_path,
            sidecar_script_path,
            analysis_seed,
        } => JamAppState::analyze_source_file_to_json(
            source_path,
            session_path,
            source_graph_path,
            sidecar_script_path,
            analysis_seed,
        ),
    }
}

fn run_terminal_ui(
    shell: JamShellState,
    mode: LaunchMode,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ManagedTerminal::enter()?;
    run_event_loop(terminal.terminal_mut(), shell, mode)
}

struct ManagedTerminal {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl ManagedTerminal {
    fn enter() -> Result<Self, Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = stdout();

        if let Err(error) = execute!(stdout, EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(Box::new(error));
        }

        let backend = CrosstermBackend::new(stdout);
        let terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => {
                let _ = disable_raw_mode();
                let mut cleanup_stdout = io::stdout();
                let _ = execute!(cleanup_stdout, LeaveAlternateScreen);
                return Err(Box::new(error));
            }
        };

        Ok(Self { terminal })
    }

    fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<io::Stdout>> {
        &mut self.terminal
    }
}

impl Drop for ManagedTerminal {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut shell: JamShellState,
    mode: LaunchMode,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|frame| render_jam_shell(frame, &shell))?;

        if event::poll(UI_TICK)?
            && let Event::Key(key) = event::read()?
        {
            match shell.handle_key_code(key.code) {
                ShellKeyOutcome::Quit => return Ok(()),
                ShellKeyOutcome::Continue => {}
                ShellKeyOutcome::ToggleTransport => {
                    let next_is_playing = !shell.app.runtime.transport.is_playing;
                    shell.app.set_transport_playing(next_is_playing);
                    shell.set_error_status(if next_is_playing {
                        "transport started"
                    } else {
                        "transport paused"
                    });
                }
                ShellKeyOutcome::QueueSceneMutation => {
                    shell.app.queue_scene_mutation(timestamp_now());
                    shell.set_error_status("queued scene mutation for next bar");
                }
                ShellKeyOutcome::QueueTr909Fill => {
                    shell.app.queue_tr909_fill(timestamp_now());
                    shell.set_error_status("queued TR-909 fill for next bar");
                }
                ShellKeyOutcome::QueueTr909Reinforce => {
                    shell.app.queue_tr909_reinforce(timestamp_now());
                    shell.set_error_status("queued TR-909 reinforcement for next phrase");
                }
                ShellKeyOutcome::QueueCaptureBar => {
                    shell.app.queue_capture_bar(timestamp_now());
                    shell.set_error_status("queued capture for next phrase");
                }
                ShellKeyOutcome::PromoteLastCapture => {
                    if shell.app.queue_promote_last_capture(timestamp_now()) {
                        shell.set_error_status("queued promotion for latest capture");
                    } else {
                        shell.set_error_status("no promotable capture or W-30 target available");
                    }
                }
                ShellKeyOutcome::TogglePinLatestCapture => {
                    match shell.app.toggle_pin_latest_capture() {
                        Some(true) => shell.set_error_status("pinned latest capture"),
                        Some(false) => shell.set_error_status("unpinned latest capture"),
                        None => shell.set_error_status("no capture available to pin"),
                    }
                }
                ShellKeyOutcome::UndoLast => {
                    if shell.app.undo_last_action(timestamp_now()).is_some() {
                        shell.set_error_status("undid most recent action");
                    } else {
                        shell.set_error_status("no undoable action available");
                    }
                }
                ShellKeyOutcome::RequestRefresh => match load_state(mode.clone()) {
                    Ok(state) => shell.replace_app_state(state),
                    Err(error) => shell.set_error_status(format!("refresh failed: {error}")),
                },
            }
        } else {
            let delta_beats = tick_delta_beats(&shell);
            if !shell
                .app
                .advance_transport_by(delta_beats, timestamp_now())
                .is_empty()
            {
                shell.set_error_status("committed queued actions on transport boundary");
            }
        }
    }
}

fn tick_delta_beats(shell: &JamShellState) -> f64 {
    let bpm = shell
        .app
        .jam_view
        .source
        .bpm_estimate
        .map(f64::from)
        .filter(|bpm| *bpm > 0.0)
        .unwrap_or(120.0);
    bpm * UI_TICK.as_secs_f64() / 60.0
}

fn timestamp_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<LaunchMode, String> {
    let mut args = args.into_iter();
    let mut source_path = None;
    let mut session_path = None;
    let mut source_graph_path = None;
    let mut sidecar_script_path = Some(PathBuf::from(DEFAULT_SIDECAR_PATH));
    let mut analysis_seed = 19_u64;
    let mut saw_session_flag = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--source" => source_path = Some(next_path(&mut args, "--source")?),
            "--session" => {
                saw_session_flag = true;
                session_path = Some(next_path(&mut args, "--session")?);
            }
            "--graph" => {
                source_graph_path = Some(next_path(&mut args, "--graph")?);
            }
            "--sidecar" => sidecar_script_path = Some(next_path(&mut args, "--sidecar")?),
            "--seed" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                analysis_seed = value
                    .parse::<u64>()
                    .map_err(|_| format!("invalid seed value: {value}"))?;
            }
            "--help" | "-h" => return Err(help_text()),
            other => return Err(format!("unknown argument: {other}\n\n{}", help_text())),
        }
    }

    let session_path = session_path.unwrap_or_else(|| PathBuf::from(DEFAULT_SESSION_PATH));
    match source_path {
        Some(source_path) => Ok(LaunchMode::Ingest {
            source_path,
            session_path,
            source_graph_path,
            sidecar_script_path: sidecar_script_path
                .unwrap_or_else(|| PathBuf::from(DEFAULT_SIDECAR_PATH)),
            analysis_seed,
        }),
        None => {
            if !saw_session_flag {
                return Err(help_text());
            }

            Ok(LaunchMode::Load {
                session_path,
                source_graph_path,
            })
        }
    }
}

fn next_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing value for {flag}"))
}

fn help_text() -> String {
    format!(
        "Usage:\n  riotbox-app --source <audio.wav> [--session <session.json>] [--graph <source-graph.json>] [--sidecar <script.py>] [--seed <n>]\n  riotbox-app --session <session.json> [--graph <source-graph.json>]\n\nDefaults:\n  --session {}\n  --sidecar {}",
        DEFAULT_SESSION_PATH, DEFAULT_SIDECAR_PATH
    )
}

impl LaunchMode {
    fn shell_launch_mode(&self) -> ShellLaunchMode {
        match self {
            Self::Load { .. } => ShellLaunchMode::Load,
            Self::Ingest { .. } => ShellLaunchMode::Ingest,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_builds_ingest_mode() {
        let mode = parse_args([
            "--source".into(),
            "input.wav".into(),
            "--session".into(),
            "session.json".into(),
            "--graph".into(),
            "graph.json".into(),
        ])
        .expect("parse ingest mode");

        match mode {
            LaunchMode::Ingest {
                source_path,
                session_path,
                source_graph_path,
                analysis_seed,
                ..
            } => {
                assert_eq!(source_path, PathBuf::from("input.wav"));
                assert_eq!(session_path, PathBuf::from("session.json"));
                assert_eq!(source_graph_path, Some(PathBuf::from("graph.json")));
                assert_eq!(analysis_seed, 19);
            }
            LaunchMode::Load { .. } => panic!("expected ingest mode"),
        }
    }

    #[test]
    fn parse_args_defaults_ingest_to_embedded_graph_storage() {
        let mode = parse_args([
            "--source".into(),
            "input.wav".into(),
            "--session".into(),
            "session.json".into(),
        ])
        .expect("parse ingest mode");

        match mode {
            LaunchMode::Ingest {
                source_graph_path, ..
            } => {
                assert_eq!(source_graph_path, None);
            }
            LaunchMode::Load { .. } => panic!("expected ingest mode"),
        }
    }

    #[test]
    fn parse_args_builds_load_mode() {
        let mode = parse_args([
            "--session".into(),
            "session.json".into(),
            "--graph".into(),
            "graph.json".into(),
        ])
        .expect("parse load mode");

        match mode {
            LaunchMode::Load {
                session_path,
                source_graph_path,
            } => {
                assert_eq!(session_path, PathBuf::from("session.json"));
                assert_eq!(source_graph_path, Some(PathBuf::from("graph.json")));
            }
            LaunchMode::Ingest { .. } => panic!("expected load mode"),
        }
    }

    #[test]
    fn parse_args_allows_session_only_for_load_mode() {
        let mode =
            parse_args(["--session".into(), "session.json".into()]).expect("session-only load");

        match mode {
            LaunchMode::Load {
                session_path,
                source_graph_path,
            } => {
                assert_eq!(session_path, PathBuf::from("session.json"));
                assert_eq!(source_graph_path, None);
            }
            LaunchMode::Ingest { .. } => panic!("expected load mode"),
        }
    }
}
