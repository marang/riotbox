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

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut shell: JamShellState,
    launch: AppLaunch,
    mut audio_runtime: Option<&mut AudioRuntimeShell>,
    mut observer: Option<&mut UserSessionObserver>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        if let Some(audio_runtime) = audio_runtime.as_deref_mut() {
            let now = timestamp_now();
            let committed = shell
                .app
                .apply_audio_timing_snapshot(audio_runtime.timing_snapshot(), now);
            if !committed.is_empty() {
                shell.set_error_status("committed queued actions on transport boundary");
                if let Some(observer) = observer.as_deref_mut() {
                    observer.record_transport_commit(now, &committed, &shell)?;
                }
            }

            audio_runtime.update_transport_state(
                shell.app.runtime.transport.is_playing,
                shell.app.runtime.tr909_render.tempo_bpm,
                shell.app.runtime.transport.position_beats,
            );
            audio_runtime.update_tr909_render_state(&shell.app.runtime.tr909_render);
            audio_runtime.update_mc202_render_state(&shell.app.runtime.mc202_render);
            audio_runtime.update_w30_preview_render_state(&shell.app.runtime.w30_preview);
            audio_runtime.update_w30_resample_tap_state(&shell.app.runtime.w30_resample_tap);
            shell.app.set_audio_health(audio_runtime.health_snapshot());
        }

        terminal.draw(|frame| render_jam_shell(frame, &shell))?;

        if event::poll(INPUT_POLL)?
            && let Event::Key(key) = event::read()?
        {
            let key_label = key_code_label(key.code);
            let outcome = shell.handle_key_code(key.code);
            match outcome {
                ShellKeyOutcome::Quit => {
                    if let Some(observer) = observer.as_deref_mut() {
                        observer.record_key_event(
                            timestamp_now(),
                            &key_label,
                            shell_key_outcome_label(outcome),
                            &shell,
                        )?;
                    }
                    return Ok(());
                }
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
                ShellKeyOutcome::QueueSceneSelect => {
                    match shell.app.queue_scene_select(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status("queued scene select for next bar");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("scene transition already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status(scene_select_unavailable_status(&shell));
                        }
                    }
                }
                ShellKeyOutcome::QueueSceneRestore => {
                    match shell.app.queue_scene_restore(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status("queued scene restore for next bar");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("scene transition already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status("no restore scene available");
                        }
                    }
                }
                ShellKeyOutcome::QueueMc202RoleToggle => {
                    match shell.app.queue_mc202_role_toggle(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status("queued MC-202 role change for next phrase");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("MC-202 role change already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status("MC-202 role already set");
                        }
                    }
                }
                ShellKeyOutcome::QueueMc202GenerateFollower => {
                    match shell.app.queue_mc202_generate_follower(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status(
                                "queued MC-202 follower generation for next phrase",
                            );
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("MC-202 follower generation already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status("MC-202 follower already in state");
                        }
                    }
                }
                ShellKeyOutcome::QueueMc202GenerateAnswer => {
                    match shell.app.queue_mc202_generate_answer(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status(
                                "queued MC-202 answer generation for next phrase",
                            );
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("MC-202 answer generation already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status("MC-202 answer already in state");
                        }
                    }
                }
                ShellKeyOutcome::QueueTr909Fill => {
                    shell.app.queue_tr909_fill(timestamp_now());
                    shell.set_error_status("queued TR-909 fill for next bar");
                }
                ShellKeyOutcome::QueueTr909Reinforce => {
                    shell.app.queue_tr909_reinforce(timestamp_now());
                    shell.set_error_status("queued TR-909 reinforcement for next phrase");
                }
                ShellKeyOutcome::QueueTr909Slam => {
                    if shell.app.queue_tr909_slam_toggle(timestamp_now()) {
                        shell.set_error_status("queued TR-909 slam change for next beat");
                    } else {
                        shell.set_error_status("TR-909 slam change already queued");
                    }
                }
                ShellKeyOutcome::QueueTr909Takeover => {
                    match shell.app.queue_tr909_takeover(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status("queued TR-909 takeover for next phrase");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("TR-909 takeover change already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status("TR-909 takeover already active");
                        }
                    }
                }
                ShellKeyOutcome::QueueTr909SceneLock => {
                    match shell.app.queue_tr909_scene_lock(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status(
                                "queued TR-909 scene-lock variation for next phrase",
                            );
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("TR-909 takeover change already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status("TR-909 scene-lock variation already active");
                        }
                    }
                }
                ShellKeyOutcome::QueueTr909Release => {
                    match shell.app.queue_tr909_release(timestamp_now()) {
                        riotbox_app::jam_app::QueueControlResult::Enqueued => {
                            shell.set_error_status("queued TR-909 release for next phrase");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                            shell.set_error_status("TR-909 takeover change already queued");
                        }
                        riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                            shell.set_error_status("TR-909 takeover already released");
                        }
                    }
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
                ShellKeyOutcome::QueueW30TriggerPad => {
                    match shell.app.queue_w30_trigger_pad(timestamp_now()) {
                        Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                            shell.set_error_status("queued W-30 pad trigger for next beat");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                            shell.set_error_status("W-30 pad cue already queued");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                            shell.set_error_status("W-30 pad trigger already in state");
                        }
                        None => {
                            shell.set_error_status("no committed W-30 pad available to trigger")
                        }
                    }
                }
                ShellKeyOutcome::QueueW30StepFocus => {
                    match shell.app.queue_w30_step_focus(timestamp_now()) {
                        Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                            shell.set_error_status("queued W-30 focus step for next beat");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                            shell.set_error_status("W-30 pad cue already queued");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                            shell.set_error_status("W-30 focus already on the next stepped pad");
                        }
                        None => shell
                            .set_error_status("no promoted W-30 pads available to step through"),
                    }
                }
                ShellKeyOutcome::QueueW30SwapBank => {
                    match shell.app.queue_w30_swap_bank(timestamp_now()) {
                        Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                            shell.set_error_status("queued W-30 bank swap for next bar");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                            shell.set_error_status("W-30 pad cue already queued");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                            shell.set_error_status("W-30 bank swap already on the next bank");
                        }
                        None => {
                            shell.set_error_status("no alternate W-30 bank available to swap to")
                        }
                    }
                }
                ShellKeyOutcome::QueueW30BrowseSlicePool => {
                    match shell.app.queue_w30_browse_slice_pool(timestamp_now()) {
                        Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                            shell.set_error_status("queued W-30 slice-pool browse for next beat");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                            shell.set_error_status("W-30 pad cue already queued");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                            shell
                                .set_error_status("W-30 slice pool already on the current capture");
                        }
                        None => shell.set_error_status(
                            "no alternate capture in the current W-30 slice pool",
                        ),
                    }
                }
                ShellKeyOutcome::QueueW30ApplyDamageProfile => {
                    match shell.app.queue_w30_apply_damage_profile(timestamp_now()) {
                        Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                            shell.set_error_status("queued W-30 damage profile for next bar");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                            shell.set_error_status("W-30 pad cue already queued");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                            shell.set_error_status("W-30 damage profile already active");
                        }
                        None => shell.set_error_status("no W-30 pad available for damage profile"),
                    }
                }
                ShellKeyOutcome::QueueW30LoopFreeze => {
                    match shell.app.queue_w30_loop_freeze(timestamp_now()) {
                        Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                            shell.set_error_status("queued W-30 loop freeze for next phrase");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                            shell.set_error_status("W-30 pad cue already queued");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                            shell.set_error_status("W-30 loop freeze already in state");
                        }
                        None => shell.set_error_status("no committed W-30 pad available to freeze"),
                    }
                }
                ShellKeyOutcome::QueueW30LiveRecall => {
                    match shell.app.queue_w30_live_recall(timestamp_now()) {
                        Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                            shell.set_error_status("queued W-30 live recall for next bar");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                            shell.set_error_status("W-30 live recall already queued");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                            shell.set_error_status("W-30 live recall already in state");
                        }
                        None => shell.set_error_status(
                            "no pinned or promoted W-30 capture available to recall",
                        ),
                    }
                }
                ShellKeyOutcome::QueueW30Audition => {
                    match shell.app.queue_w30_audition(timestamp_now()) {
                        Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                            shell.set_error_status("queued W-30 audition for next bar");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                            shell.set_error_status("W-30 pad cue already queued");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                            shell.set_error_status("W-30 audition already in state");
                        }
                        None => {
                            shell.set_error_status("no W-30 or raw capture available to audition")
                        }
                    }
                }
                ShellKeyOutcome::QueueW30Resample => {
                    match shell.app.queue_w30_internal_resample(timestamp_now()) {
                        Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                            shell.set_error_status("queued W-30 internal resample for next phrase");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                            shell.set_error_status("W-30 internal resample already queued");
                        }
                        Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                            shell.set_error_status("W-30 internal resample already in state");
                        }
                        None => shell
                            .set_error_status("no committed W-30 capture available to resample"),
                    }
                }
                ShellKeyOutcome::TogglePinLatestCapture => {
                    match shell.app.toggle_pin_latest_capture() {
                        Some(true) => shell.set_error_status("pinned latest capture"),
                        Some(false) => shell.set_error_status("unpinned latest capture"),
                        None => shell.set_error_status("no capture available to pin"),
                    }
                }
                ShellKeyOutcome::LowerDrumBusLevel => {
                    let level = shell.app.adjust_drum_bus_level(-0.1);
                    shell.set_error_status(format!("drum bus level {:.2}", level));
                }
                ShellKeyOutcome::RaiseDrumBusLevel => {
                    let level = shell.app.adjust_drum_bus_level(0.1);
                    shell.set_error_status(format!("drum bus level {:.2}", level));
                }
                ShellKeyOutcome::UndoLast => {
                    if shell.app.undo_last_action(timestamp_now()).is_some() {
                        shell.set_error_status("undid most recent action");
                    } else {
                        shell.set_error_status("no undoable action available");
                    }
                }
                ShellKeyOutcome::RequestRefresh => match load_state(launch.mode.clone()) {
                    Ok(state) => shell.replace_app_state(state),
                    Err(error) => shell.set_error_status(format!("refresh failed: {error}")),
                },
            }

            if let Some(observer) = observer.as_deref_mut() {
                observer.record_key_event(
                    timestamp_now(),
                    &key_label,
                    shell_key_outcome_label(outcome),
                    &shell,
                )?;
            }
        }
    }
}

fn timestamp_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn scene_select_unavailable_status(shell: &JamShellState) -> &'static str {
    match shell.app.jam_view.scene.scene_jump_availability {
        SceneJumpAvailabilityView::WaitingForMoreScenes => "scene jump waits for 2 scenes",
        SceneJumpAvailabilityView::Ready | SceneJumpAvailabilityView::Unknown => {
            "no next scene candidate available"
        }
    }
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<AppLaunch, String> {
    let mut args = args.into_iter();
    let mut source_path = None;
    let mut session_path = None;
    let mut source_graph_path = None;
    let mut sidecar_script_path = Some(PathBuf::from(DEFAULT_SIDECAR_PATH));
    let mut analysis_seed = 19_u64;
    let mut saw_session_flag = false;
    let mut observer_path = None;

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
            "--observer" => observer_path = Some(next_path(&mut args, "--observer")?),
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
    let mode = match source_path {
        Some(source_path) => LaunchMode::Ingest {
            source_path,
            session_path,
            source_graph_path,
            sidecar_script_path: sidecar_script_path
                .unwrap_or_else(|| PathBuf::from(DEFAULT_SIDECAR_PATH)),
            analysis_seed,
        },
        None => {
            if !saw_session_flag {
                return Err(help_text());
            }

            LaunchMode::Load {
                session_path,
                source_graph_path,
            }
        }
    };

    Ok(AppLaunch {
        mode,
        observer_path,
    })
}

fn next_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing value for {flag}"))
}

fn help_text() -> String {
    format!(
        "Usage:\n  riotbox-app --source <audio.wav> [--session <session.json>] [--graph <source-graph.json>] [--sidecar <script.py>] [--seed <n>] [--observer <events.ndjson>]\n  riotbox-app --session <session.json> [--graph <source-graph.json>] [--observer <events.ndjson>]\n\nDefaults:\n  --session {}\n  --sidecar {}",
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

struct UserSessionObserver {
    writer: BufWriter<File>,
}

impl UserSessionObserver {
    fn open(path: &Path) -> io::Result<Self> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }

        let writer = BufWriter::new(
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)?,
        );
        Ok(Self { writer })
    }

    fn record_launch(
        &mut self,
        raw_args: &[String],
        launch: &AppLaunch,
        shell: &JamShellState,
    ) -> io::Result<()> {
        self.record(json!({
            "event": "observer_started",
            "schema": "riotbox.user_session_observer.v1",
            "timestamp_ms": timestamp_now(),
            "opt_in": true,
            "capture_context": "interactive_terminal",
            "raw_audio_recording": false,
            "realtime_callback_io": false,
            "argv": raw_args,
            "launch": launch_summary(launch),
            "snapshot": observer_snapshot(shell),
        }))
    }

    fn record_audio_runtime(
        &mut self,
        status: &str,
        error: Option<&str>,
        shell: &JamShellState,
    ) -> io::Result<()> {
        self.record(json!({
            "event": "audio_runtime",
            "timestamp_ms": timestamp_now(),
            "status": status,
            "error": error,
            "snapshot": observer_snapshot(shell),
        }))
    }

    fn record_key_event(
        &mut self,
        timestamp_ms: u64,
        key: &str,
        outcome: &str,
        shell: &JamShellState,
    ) -> io::Result<()> {
        self.record(json!({
            "event": "key_outcome",
            "timestamp_ms": timestamp_ms,
            "key": key,
            "outcome": outcome,
            "snapshot": observer_snapshot(shell),
        }))
    }

    fn record_transport_commit(
        &mut self,
        timestamp_ms: u64,
        committed: &[riotbox_core::queue::CommittedActionRef],
        shell: &JamShellState,
    ) -> io::Result<()> {
        self.record(json!({
            "event": "transport_commit",
            "timestamp_ms": timestamp_ms,
            "committed": committed.iter().map(|committed| {
                json!({
                    "action_id": committed.action_id.0,
                    "boundary": format!("{:?}", committed.boundary.kind),
                    "beat_index": committed.boundary.beat_index,
                    "bar_index": committed.boundary.bar_index,
                    "phrase_index": committed.boundary.phrase_index,
                    "scene_id": committed.boundary.scene_id.as_ref().map(ToString::to_string),
                    "commit_sequence": committed.commit_sequence,
                })
            }).collect::<Vec<_>>(),
            "snapshot": observer_snapshot(shell),
        }))
    }

    fn record(&mut self, event: Value) -> io::Result<()> {
        serde_json::to_writer(&mut self.writer, &event).map_err(io::Error::other)?;
        writeln!(self.writer)?;
        self.writer.flush()
    }
}

fn launch_summary(launch: &AppLaunch) -> Value {
    match &launch.mode {
        LaunchMode::Load {
            session_path,
            source_graph_path,
        } => json!({
            "mode": "load",
            "session_path": session_path,
            "source_graph_path": source_graph_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::Ingest {
            source_path,
            session_path,
            source_graph_path,
            sidecar_script_path,
            analysis_seed,
        } => json!({
            "mode": "ingest",
            "source_path": source_path,
            "session_path": session_path,
            "source_graph_path": source_graph_path,
            "sidecar_script_path": sidecar_script_path,
            "analysis_seed": analysis_seed,
            "observer_path": launch.observer_path,
        }),
    }
}

fn observer_snapshot(shell: &JamShellState) -> Value {
    let transport = &shell.app.runtime.transport;
    let runtime = &shell.app.runtime_view;
    json!({
        "status_message": shell.status_message,
        "active_screen": shell.active_screen.label(),
        "jam_mode": shell.jam_mode.label(),
        "show_help": shell.show_help,
        "transport": {
            "is_playing": transport.is_playing,
            "position_beats": transport.position_beats,
            "beat_index": transport.beat_index,
            "bar_index": transport.bar_index,
            "phrase_index": transport.phrase_index,
            "current_scene": transport.current_scene.as_ref().map(ToString::to_string),
        },
        "queue": {
            "pending_count": shell.app.queue.pending_actions().len(),
            "queue_history_count": shell.app.queue.history().len(),
            "session_log_count": shell.app.session.action_log.actions.len(),
            "pending": shell.app.queue.pending_actions().into_iter().map(compact_action).collect::<Vec<_>>(),
            "recent_history": shell.app.queue.history().iter().rev().take(5).map(compact_action).collect::<Vec<_>>(),
        },
        "runtime": {
            "audio_status": runtime.audio_status,
            "audio_callback_count": runtime.audio_callback_count,
            "audio_last_error": runtime.audio_last_error,
            "sidecar_status": runtime.sidecar_status,
            "tr909_mode": runtime.tr909_render_mode,
            "tr909_routing": runtime.tr909_render_routing,
            "tr909_profile": runtime.tr909_render_profile,
            "tr909_support_context": runtime.tr909_render_support_context,
            "tr909_support_accent": runtime.tr909_render_support_accent,
            "mc202_mode": runtime.mc202_render_mode,
            "mc202_routing": runtime.mc202_render_routing,
            "mc202_phrase_shape": runtime.mc202_render_phrase_shape,
            "w30_preview_mode": runtime.w30_preview_mode,
            "w30_preview_target": runtime.w30_preview_target_summary,
            "w30_resample_tap_mode": runtime.w30_resample_tap_mode,
            "warnings": runtime.runtime_warnings,
        }
    })
}

fn compact_action(action: &riotbox_core::action::Action) -> Value {
    json!({
        "id": action.id.0,
        "command": action.command.as_str(),
        "actor": action.actor.to_string(),
        "quantization": action.quantization.to_string(),
        "status": format!("{:?}", action.status),
        "requested_at": action.requested_at,
        "committed_at": action.committed_at,
        "result": action.result.as_ref().map(|result| result.summary.clone()),
    })
}

fn key_code_label(code: KeyCode) -> String {
    match code {
        KeyCode::Char(' ') => "space".into(),
        KeyCode::Char(character) => character.to_string(),
        KeyCode::Enter => "enter".into(),
        KeyCode::Esc => "escape".into(),
        KeyCode::Tab => "tab".into(),
        KeyCode::BackTab => "backtab".into(),
        other => format!("{other:?}"),
    }
}

fn shell_key_outcome_label(outcome: ShellKeyOutcome) -> &'static str {
    match outcome {
        ShellKeyOutcome::Continue => "continue",
        ShellKeyOutcome::RequestRefresh => "request_refresh",
        ShellKeyOutcome::ToggleTransport => "toggle_transport",
        ShellKeyOutcome::QueueSceneMutation => "queue_scene_mutation",
        ShellKeyOutcome::QueueSceneSelect => "queue_scene_select",
        ShellKeyOutcome::QueueSceneRestore => "queue_scene_restore",
        ShellKeyOutcome::QueueMc202RoleToggle => "queue_mc202_role_toggle",
        ShellKeyOutcome::QueueMc202GenerateFollower => "queue_mc202_generate_follower",
        ShellKeyOutcome::QueueMc202GenerateAnswer => "queue_mc202_generate_answer",
        ShellKeyOutcome::QueueTr909Fill => "queue_tr909_fill",
        ShellKeyOutcome::QueueTr909Reinforce => "queue_tr909_reinforce",
        ShellKeyOutcome::QueueTr909Slam => "queue_tr909_slam",
        ShellKeyOutcome::QueueTr909Takeover => "queue_tr909_takeover",
        ShellKeyOutcome::QueueTr909SceneLock => "queue_tr909_scene_lock",
        ShellKeyOutcome::QueueTr909Release => "queue_tr909_release",
        ShellKeyOutcome::QueueCaptureBar => "queue_capture_bar",
        ShellKeyOutcome::PromoteLastCapture => "promote_last_capture",
        ShellKeyOutcome::QueueW30TriggerPad => "queue_w30_trigger_pad",
        ShellKeyOutcome::QueueW30StepFocus => "queue_w30_step_focus",
        ShellKeyOutcome::QueueW30SwapBank => "queue_w30_swap_bank",
        ShellKeyOutcome::QueueW30BrowseSlicePool => "queue_w30_browse_slice_pool",
        ShellKeyOutcome::QueueW30ApplyDamageProfile => "queue_w30_apply_damage_profile",
        ShellKeyOutcome::QueueW30LoopFreeze => "queue_w30_loop_freeze",
        ShellKeyOutcome::QueueW30LiveRecall => "queue_w30_live_recall",
        ShellKeyOutcome::QueueW30Audition => "queue_w30_audition",
        ShellKeyOutcome::QueueW30Resample => "queue_w30_resample",
        ShellKeyOutcome::TogglePinLatestCapture => "toggle_pin_latest_capture",
        ShellKeyOutcome::LowerDrumBusLevel => "lower_drum_bus_level",
        ShellKeyOutcome::RaiseDrumBusLevel => "raise_drum_bus_level",
        ShellKeyOutcome::UndoLast => "undo_last",
        ShellKeyOutcome::Quit => "quit",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riotbox_core::{ids::SceneId, queue::ActionQueue, session::SessionFile};

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

        assert_eq!(mode.observer_path, None);
        match mode.mode {
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

        match mode.mode {
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

        match mode.mode {
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

        match mode.mode {
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

    #[test]
    fn parse_args_accepts_observer_path() {
        let launch = parse_args([
            "--session".into(),
            "session.json".into(),
            "--observer".into(),
            "artifacts/audio_qa/live/events.ndjson".into(),
        ])
        .expect("parse observer path");

        assert_eq!(
            launch.observer_path,
            Some(PathBuf::from("artifacts/audio_qa/live/events.ndjson"))
        );
    }

    #[test]
    fn user_session_observer_writes_launch_and_key_events() {
        let temp = tempfile::tempdir().expect("tempdir");
        let observer_path = temp.path().join("observer/events.ndjson");
        let launch = AppLaunch {
            mode: LaunchMode::Load {
                session_path: PathBuf::from("session.json"),
                source_graph_path: None,
            },
            observer_path: Some(observer_path.clone()),
        };
        let shell = JamShellState::new(
            JamAppState::from_parts(
                SessionFile::new("session-1", "0.1.0", "2026-04-26T00:00:00Z"),
                None,
                ActionQueue::new(),
            ),
            ShellLaunchMode::Load,
        );
        let mut observer = UserSessionObserver::open(&observer_path).expect("open observer");

        observer
            .record_launch(
                &[
                    "riotbox-app".into(),
                    "--session".into(),
                    "session.json".into(),
                    "--observer".into(),
                    observer_path.display().to_string(),
                ],
                &launch,
                &shell,
            )
            .expect("record launch");
        observer
            .record_key_event(123, "space", "toggle_transport", &shell)
            .expect("record key");
        drop(observer);

        let content = fs::read_to_string(observer_path).expect("read observer");

        assert!(content.contains("\"event\":\"observer_started\""));
        assert!(content.contains("\"event\":\"key_outcome\""));
        assert!(content.contains("\"outcome\":\"toggle_transport\""));
        assert!(content.contains("\"raw_audio_recording\":false"));
        assert!(content.contains("\"realtime_callback_io\":false"));
    }

    #[test]
    fn scene_select_unavailable_status_explains_waiting_for_scene_material() {
        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-25T00:00:00Z");
        session.runtime_state.scene_state.scenes = vec![SceneId::from("scene-01-intro")];
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));

        let shell = JamShellState::new(
            JamAppState::from_parts(session, None, ActionQueue::new()),
            ShellLaunchMode::Load,
        );

        assert_eq!(
            scene_select_unavailable_status(&shell),
            "scene jump waits for 2 scenes"
        );
    }
}
