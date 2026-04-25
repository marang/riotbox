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
use riotbox_audio::runtime::AudioRuntimeShell;
use riotbox_core::view::jam::SceneJumpAvailabilityView;

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
    mut shell: JamShellState,
    mode: LaunchMode,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ManagedTerminal::enter()?;
    let mut audio_runtime = match AudioRuntimeShell::start_default_output_with_render_states(
        shell.app.runtime.tr909_render.clone(),
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
            Some(runtime)
        }
        Err(error) => {
            shell.set_error_status(format!("audio unavailable: {error}"));
            None
        }
    };
    run_event_loop(terminal.terminal_mut(), shell, mode, audio_runtime.as_mut())
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
    mut audio_runtime: Option<&mut AudioRuntimeShell>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        if let Some(audio_runtime) = audio_runtime.as_deref_mut() {
            if !shell
                .app
                .apply_audio_timing_snapshot(audio_runtime.timing_snapshot(), timestamp_now())
                .is_empty()
            {
                shell.set_error_status("committed queued actions on transport boundary");
            }

            audio_runtime.update_transport_state(
                shell.app.runtime.transport.is_playing,
                shell.app.runtime.tr909_render.tempo_bpm,
                shell.app.runtime.transport.position_beats,
            );
            audio_runtime.update_tr909_render_state(&shell.app.runtime.tr909_render);
            audio_runtime.update_w30_preview_render_state(&shell.app.runtime.w30_preview);
            audio_runtime.update_w30_resample_tap_state(&shell.app.runtime.w30_resample_tap);
            shell.app.set_audio_health(audio_runtime.health_snapshot());
        }

        terminal.draw(|frame| render_jam_shell(frame, &shell))?;

        if event::poll(INPUT_POLL)?
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
                ShellKeyOutcome::RequestRefresh => match load_state(mode.clone()) {
                    Ok(state) => shell.replace_app_state(state),
                    Err(error) => shell.set_error_status(format!("refresh failed: {error}")),
                },
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
