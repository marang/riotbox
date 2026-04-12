use std::{
    env,
    io::{self, stdout},
    path::PathBuf,
    time::Duration,
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
const DEFAULT_GRAPH_PATH: &str = "data/sessions/source-graph.json";
const DEFAULT_SIDECAR_PATH: &str = "python/sidecar/json_stdio_sidecar.py";

#[derive(Clone, Debug)]
enum LaunchMode {
    Load {
        session_path: PathBuf,
        source_graph_path: PathBuf,
    },
    Ingest {
        source_path: PathBuf,
        session_path: PathBuf,
        source_graph_path: PathBuf,
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
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_event_loop(&mut terminal, shell, mode);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut shell: JamShellState,
    mode: LaunchMode,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|frame| render_jam_shell(frame, &shell))?;

        if event::poll(Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
        {
            match shell.handle_key_code(key.code) {
                ShellKeyOutcome::Quit => return Ok(()),
                ShellKeyOutcome::Continue => {}
                ShellKeyOutcome::RequestRefresh => match load_state(mode.clone()) {
                    Ok(state) => shell.replace_app_state(state),
                    Err(error) => shell.set_error_status(format!("refresh failed: {error}")),
                },
            }
        }
    }
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<LaunchMode, String> {
    let mut args = args.into_iter();
    let mut source_path = None;
    let mut session_path = None;
    let mut source_graph_path = None;
    let mut sidecar_script_path = Some(PathBuf::from(DEFAULT_SIDECAR_PATH));
    let mut analysis_seed = 19_u64;
    let mut saw_session_flag = false;
    let mut saw_graph_flag = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--source" => source_path = Some(next_path(&mut args, "--source")?),
            "--session" => {
                saw_session_flag = true;
                session_path = Some(next_path(&mut args, "--session")?);
            }
            "--graph" => {
                saw_graph_flag = true;
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
    let source_graph_path = source_graph_path.unwrap_or_else(|| PathBuf::from(DEFAULT_GRAPH_PATH));

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
            if !saw_session_flag && !saw_graph_flag {
                return Err(help_text());
            }
            if saw_session_flag != saw_graph_flag {
                return Err(format!(
                    "load mode requires both --session and --graph\n\n{}",
                    help_text()
                ));
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
        "Usage:\n  riotbox-app --source <audio.wav> [--session <session.json>] [--graph <source-graph.json>] [--sidecar <script.py>] [--seed <n>]\n  riotbox-app --session <session.json> --graph <source-graph.json>\n\nDefaults:\n  --session {}\n  --graph {}\n  --sidecar {}",
        DEFAULT_SESSION_PATH, DEFAULT_GRAPH_PATH, DEFAULT_SIDECAR_PATH
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
                assert_eq!(source_graph_path, PathBuf::from("graph.json"));
                assert_eq!(analysis_seed, 19);
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
                assert_eq!(source_graph_path, PathBuf::from("graph.json"));
            }
            LaunchMode::Ingest { .. } => panic!("expected load mode"),
        }
    }

    #[test]
    fn parse_args_rejects_missing_paths_for_load_mode() {
        let error = parse_args(["--session".into(), "session.json".into()])
            .expect_err("missing graph should fail");

        assert!(error.contains("requires both --session and --graph"));
    }
}
