use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write, stdout},
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use riotbox_app::{
    jam_app::{JamAppError, JamAppState},
    ui::{JamShellState, ShellKeyOutcome, ShellLaunchMode, render_jam_shell},
};
use riotbox_audio::runtime::AudioRuntimeShell;
use riotbox_core::view::jam::SceneJumpAvailabilityView;
use serde_json::{Value, json};

const DEFAULT_SESSION_PATH: &str = "data/sessions/jam-session.json";
const DEFAULT_SIDECAR_PATH: &str = "python/sidecar/json_stdio_sidecar.py";
const INPUT_POLL: Duration = Duration::from_millis(50);

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
    let raw_args = env::args().collect::<Vec<_>>();
    let launch = parse_args(raw_args.iter().skip(1).cloned())?;
    let state = load_state(launch.mode.clone())?;
    run_terminal_ui(
        JamShellState::new(state, launch.mode.shell_launch_mode()),
        launch,
        &raw_args,
    )?;
    Ok(())
}

#[derive(Clone, Debug)]
struct AppLaunch {
    mode: LaunchMode,
    observer_path: Option<PathBuf>,
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
    mut shell: JamShellState,
    launch: AppLaunch,
    raw_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut observer = match launch.observer_path.as_deref() {
        Some(path) => {
            let mut observer = UserSessionObserver::open(path)?;
            observer.record_launch(raw_args, &launch, &shell)?;
            Some(observer)
        }
        None => None,
    };
    let mut terminal = ManagedTerminal::enter()?;
    let mut audio_runtime = match AudioRuntimeShell::start_default_output_with_render_states(
        shell.app.runtime.tr909_render.clone(),
        shell.app.runtime.mc202_render,
        shell.app.runtime.w30_preview.clone(),
        shell.app.runtime.w30_resample_tap.clone(),
    ) {
        Ok(runtime) => {
            runtime.update_transport_state(
                shell.app.runtime.transport.is_playing,
                shell.app.runtime.tr909_render.tempo_bpm,
                shell.app.runtime.transport.position_beats,
            );
            shell.app.set_audio_health(runtime.health_snapshot());
            if let Some(observer) = observer.as_mut() {
                observer.record_audio_runtime("started", None, &shell)?;
            }
            Some(runtime)
        }
        Err(error) => {
            shell.set_error_status(format!("audio unavailable: {error}"));
            if let Some(observer) = observer.as_mut() {
                observer.record_audio_runtime("unavailable", Some(&error.to_string()), &shell)?;
            }
            None
        }
    };
    run_event_loop(
        terminal.terminal_mut(),
        shell,
        launch,
        audio_runtime.as_mut(),
        observer.as_mut(),
    )
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

