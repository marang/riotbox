use crate::cli::event_loop::run_event_loop;
use crate::cli::model::AppLaunch;
use crate::cli::observer::UserSessionObserver;
use crate::ui::JamShellState;
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use riotbox_audio::runtime::AudioRuntimeError;
use riotbox_audio::runtime::AudioRuntimeHealth;
use riotbox_audio::runtime::AudioRuntimeLifecycle;
use riotbox_audio::runtime::AudioRuntimeShell;
use std::io;
use std::io::stdout;

pub(in crate::cli) fn run_terminal_ui(
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
    let mut audio_runtime =
        start_audio_runtime_for_shell(&mut shell, observer.as_mut(), "started")?;
    run_event_loop(
        terminal.terminal_mut(),
        shell,
        launch,
        &mut audio_runtime,
        observer.as_mut(),
    )
}

pub(in crate::cli) fn start_audio_runtime_for_shell(
    shell: &mut JamShellState,
    observer: Option<&mut UserSessionObserver>,
    success_state: &str,
) -> Result<Option<AudioRuntimeShell>, Box<dyn std::error::Error>> {
    let audio_runtime =
        match AudioRuntimeShell::start_default_output_with_render_states_and_source_monitor(
            shell.app.runtime.tr909_render.clone(),
            shell.app.runtime.mc202_render,
            shell.app.runtime.w30_preview.clone(),
            shell.app.runtime.w30_resample_tap.clone(),
            shell.app.source_monitor_render_state(),
        ) {
            Ok(runtime) => {
                runtime.update_transport_state(
                    shell.app.runtime.transport.is_playing,
                    shell.app.runtime.tr909_render.tempo_bpm,
                    shell.app.runtime.transport.position_beats,
                );
                shell.app.set_audio_health(runtime.health_snapshot());
                if let Some(observer) = observer {
                    observer.record_audio_runtime(success_state, None, shell)?;
                }
                Some(runtime)
            }
            Err(error) => {
                shell
                    .app
                    .set_audio_health(audio_start_failure_health(&error));
                shell.set_error_status(format!("audio unavailable: {error}"));
                if let Some(observer) = observer {
                    observer.record_audio_runtime(
                        "unavailable",
                        Some(&error.to_string()),
                        shell,
                    )?;
                }
                None
            }
        };
    Ok(audio_runtime)
}

pub(in crate::cli) fn audio_start_failure_health(error: &AudioRuntimeError) -> AudioRuntimeHealth {
    AudioRuntimeHealth {
        lifecycle: AudioRuntimeLifecycle::Faulted,
        output: None,
        callback_count: 0,
        max_callback_gap_micros: None,
        callback_scratch_overflow_count: 0,
        stream_error_count: 1,
        last_stream_error: Some(error.to_string()),
    }
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
