use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write, stdout},
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use crate::{
    jam_app::{JamAppError, JamAppState, SessionRecoverySurface},
    observer::{compact_commit, key_code_label, observer_snapshot, shell_key_outcome_label},
    ui::{JamShellState, ShellKeyOutcome, ShellLaunchMode, render_jam_shell},
};
use riotbox_audio::runtime::{
    AudioRuntimeError, AudioRuntimeHealth, AudioRuntimeLifecycle, AudioRuntimeShell,
};
use riotbox_core::{
    action::{ActionCommand, SourceMonitorMode},
    queue::CommittedActionRef,
    session::ExportArtifactRole,
    style::PerformancePresetId,
    view::jam::SceneJumpAvailabilityView,
};
use serde_json::{Value, json};
use riotbox_sidecar::path::bundled_sidecar_script_path;

const DEFAULT_SESSION_PATH: &str = "data/sessions/jam-session.json";
const DEFAULT_SIDECAR_PATH: &str = "repository-bundled sidecar (CWD-independent)";
const INPUT_POLL: Duration = Duration::from_millis(50);

#[derive(Clone, Debug)]
enum LaunchMode {
    Load {
        session_path: PathBuf,
        source_graph_path: Option<PathBuf>,
        product_mix_export_handoff: Option<ProductMixExportHandoff>,
    },
    Ingest {
        source_path: PathBuf,
        session_path: PathBuf,
        source_graph_path: Option<PathBuf>,
        sidecar_script_path: PathBuf,
        analysis_seed: u64,
        explicit_source_bpm: Option<f32>,
        explicit_source_downbeat_seconds: Option<f32>,
        product_mix_export_handoff: Option<ProductMixExportHandoff>,
    },
    StemPackageLocalCiDryRun {
        destination_path: PathBuf,
        claimed_stem_roles: Vec<ExportArtifactRole>,
    },
    StemPackageLocalCiExecute {
        session_path: PathBuf,
        source_graph_path: Option<PathBuf>,
        destination_path: PathBuf,
        claimed_stem_roles: Vec<ExportArtifactRole>,
    },
    StemPackageSourceMatchedExecute {
        session_path: PathBuf,
        source_graph_path: Option<PathBuf>,
        handoff_proof_path: PathBuf,
        destination_path: PathBuf,
    },
    StemPackageW30HookExecute {
        session_path: PathBuf,
        source_graph_path: Option<PathBuf>,
        destination_path: PathBuf,
    },
    StemPackageLocalCiReport {
        session_path: PathBuf,
    },
    LiveRecordingReadinessReport {
        session_path: PathBuf,
    },
    DawExportReadinessReport {
        session_path: PathBuf,
    },
    DawSessionJsonPackageExecute {
        session_path: PathBuf,
        destination_path: PathBuf,
    },
    DawSessionJsonPackageEvidenceApply {
        session_path: PathBuf,
        destination_path: PathBuf,
    },
    DawSessionHostImportProofApply {
        session_path: PathBuf,
        proof_path: PathBuf,
    },
    DawSessionHostImportProofExportExecute {
        session_path: PathBuf,
        proof_path: PathBuf,
    },
    DawSessionAudibleOutputProofApply {
        session_path: PathBuf,
        proof_path: PathBuf,
    },
    DawSessionWriterProofExecute {
        session_path: PathBuf,
        destination_path: PathBuf,
    },
    DawSessionWriterProofApply {
        session_path: PathBuf,
        destination_path: PathBuf,
    },
    DawSessionWriterExportExecute {
        session_path: PathBuf,
        destination_path: PathBuf,
    },
    W30HookDawprojectExecute {
        session_path: PathBuf,
        destination_path: PathBuf,
    },
    DawSessionWriterPlan {
        session_path: PathBuf,
        destination_path: PathBuf,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProductMixExportHandoff {
    proof_path: PathBuf,
    destination_path: PathBuf,
}

impl LaunchMode {
    fn product_mix_export_handoff(&self) -> Option<&ProductMixExportHandoff> {
        match self {
            Self::Load {
                product_mix_export_handoff,
                ..
            }
            | Self::Ingest {
                product_mix_export_handoff,
                ..
            } => product_mix_export_handoff.as_ref(),
            _ => None,
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let raw_args = env::args().collect::<Vec<_>>();
    let launch = parse_args(raw_args.iter().skip(1).cloned())?;
    if matches!(launch.mode, LaunchMode::StemPackageLocalCiDryRun { .. }) {
        run_stem_package_local_ci_dry_run(&launch)?;
        return Ok(());
    }
    if matches!(launch.mode, LaunchMode::StemPackageLocalCiExecute { .. }) {
        run_stem_package_local_ci_execute(&launch, &raw_args)?;
        return Ok(());
    }
    if matches!(
        launch.mode,
        LaunchMode::StemPackageSourceMatchedExecute { .. }
    ) {
        run_stem_package_source_matched_execute(&launch, &raw_args)?;
        return Ok(());
    }
    if matches!(launch.mode, LaunchMode::StemPackageW30HookExecute { .. }) {
        run_stem_package_w30_hook_execute(&launch, &raw_args)?;
        return Ok(());
    }
    if matches!(launch.mode, LaunchMode::StemPackageLocalCiReport { .. }) {
        run_stem_package_local_ci_report(&launch)?;
        return Ok(());
    }
    if matches!(launch.mode, LaunchMode::LiveRecordingReadinessReport { .. }) {
        run_live_recording_readiness_report(&launch)?;
        return Ok(());
    }
    if matches!(launch.mode, LaunchMode::DawExportReadinessReport { .. }) {
        run_daw_export_readiness_report(&launch)?;
        return Ok(());
    }
    if matches!(launch.mode, LaunchMode::DawSessionJsonPackageExecute { .. }) {
        run_daw_session_json_package_execute(&launch)?;
        return Ok(());
    }
    if matches!(
        launch.mode,
        LaunchMode::DawSessionJsonPackageEvidenceApply { .. }
    ) {
        run_daw_session_json_package_evidence_apply(&launch)?;
        return Ok(());
    }
    if matches!(launch.mode, LaunchMode::DawSessionHostImportProofApply { .. }) {
        run_daw_session_host_import_proof_apply(&launch)?;
        return Ok(());
    }
    if matches!(
        launch.mode,
        LaunchMode::DawSessionHostImportProofExportExecute { .. }
    ) {
        run_daw_session_host_import_proof_export_execute(&launch, &raw_args)?;
        return Ok(());
    }
    if matches!(
        launch.mode,
        LaunchMode::DawSessionAudibleOutputProofApply { .. }
    ) {
        run_daw_session_audible_output_proof_apply(&launch)?;
        return Ok(());
    }
    if matches!(launch.mode, LaunchMode::DawSessionWriterProofExecute { .. }) {
        run_daw_session_writer_proof_execute(&launch)?;
        return Ok(());
    }
    if matches!(launch.mode, LaunchMode::DawSessionWriterProofApply { .. }) {
        run_daw_session_writer_proof_apply(&launch)?;
        return Ok(());
    }
    if matches!(launch.mode, LaunchMode::DawSessionWriterExportExecute { .. }) {
        run_daw_session_writer_export_execute(&launch, &raw_args)?;
        return Ok(());
    }
    if matches!(launch.mode, LaunchMode::W30HookDawprojectExecute { .. }) {
        run_w30_hook_dawproject_execute(&launch, &raw_args)?;
        return Ok(());
    }
    if matches!(launch.mode, LaunchMode::DawSessionWriterPlan { .. }) {
        run_daw_session_writer_plan(&launch)?;
        return Ok(());
    }
    let state = load_state(launch.mode.clone())?;
    let shell = shell_for_loaded_state(state, &launch.mode);
    run_terminal_ui(
        shell,
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

fn shell_for_loaded_state(state: JamAppState, mode: &LaunchMode) -> JamShellState {
    let mut shell = JamShellState::new(state, mode.shell_launch_mode());
    refresh_recovery_surface_for_launch(&mut shell, mode);
    shell
}

fn refresh_recovery_surface_for_launch(shell: &mut JamShellState, mode: &LaunchMode) {
    shell.clear_recovery_surface();
    if let Some(recovery_surface) = recovery_surface_for_launch(mode) {
        shell.set_recovery_surface(recovery_surface);
    }
}

fn recovery_surface_for_launch(mode: &LaunchMode) -> Option<SessionRecoverySurface> {
    let LaunchMode::Load { session_path, .. } = mode else {
        return None;
    };

    JamAppState::scan_session_recovery_surface(session_path)
        .ok()
        .filter(SessionRecoverySurface::has_non_canonical_clues)
}

fn load_state(mode: LaunchMode) -> Result<JamAppState, JamAppError> {
    match mode {
        LaunchMode::Load {
            session_path,
            source_graph_path,
            ..
        } => JamAppState::from_json_files(session_path, source_graph_path),
        LaunchMode::Ingest {
            source_path,
            session_path,
            source_graph_path,
            sidecar_script_path,
            analysis_seed,
            explicit_source_bpm,
            explicit_source_downbeat_seconds,
            ..
        } => JamAppState::analyze_source_file_to_json_with_source_timing_confirmation(
            source_path,
            session_path,
            source_graph_path,
            sidecar_script_path,
            analysis_seed,
            explicit_source_bpm,
            explicit_source_downbeat_seconds,
        ),
        LaunchMode::StemPackageLocalCiDryRun { .. } => Err(JamAppError::InvalidSession(
            "stem package local CI dry-run does not load app state".into(),
        )),
        LaunchMode::StemPackageLocalCiExecute { .. } => Err(JamAppError::InvalidSession(
            "stem package local CI execute uses a non-interactive proof path".into(),
        )),
        LaunchMode::StemPackageSourceMatchedExecute { .. } => Err(JamAppError::InvalidSession(
            "source-matched stem package execute uses a non-interactive proof path".into(),
        )),
        LaunchMode::StemPackageW30HookExecute { .. } => Err(JamAppError::InvalidSession(
            "W-30 hook stem package execute uses a non-interactive proof path".into(),
        )),
        LaunchMode::StemPackageLocalCiReport { .. } => Err(JamAppError::InvalidSession(
            "stem package local CI report uses a non-interactive proof path".into(),
        )),
        LaunchMode::LiveRecordingReadinessReport { .. } => Err(JamAppError::InvalidSession(
            "live recording readiness report uses a non-interactive proof path".into(),
        )),
        LaunchMode::DawExportReadinessReport { .. } => Err(JamAppError::InvalidSession(
            "DAW export readiness report uses a non-interactive proof path".into(),
        )),
        LaunchMode::DawSessionJsonPackageExecute { .. } => Err(JamAppError::InvalidSession(
            "DAW session JSON package execute uses a non-interactive proof path".into(),
        )),
        LaunchMode::DawSessionJsonPackageEvidenceApply { .. } => Err(
            JamAppError::InvalidSession(
                "DAW session JSON package evidence apply uses a non-interactive proof path".into(),
            ),
        ),
        LaunchMode::DawSessionHostImportProofApply { .. } => Err(JamAppError::InvalidSession(
            "DAW session host import proof apply uses a non-interactive proof path".into(),
        )),
        LaunchMode::DawSessionHostImportProofExportExecute { .. } => Err(
            JamAppError::InvalidSession(
                "DAW session host import proof export execute uses a non-interactive proof path"
                    .into(),
            ),
        ),
        LaunchMode::DawSessionAudibleOutputProofApply { .. } => Err(
            JamAppError::InvalidSession(
                "DAW session audible output proof apply uses a non-interactive proof path".into(),
            ),
        ),
        LaunchMode::DawSessionWriterProofExecute { .. } => Err(JamAppError::InvalidSession(
            "DAW session writer proof execute uses a non-interactive proof path".into(),
        )),
        LaunchMode::DawSessionWriterProofApply { .. } => Err(JamAppError::InvalidSession(
            "DAW session writer proof apply uses a non-interactive proof path".into(),
        )),
        LaunchMode::DawSessionWriterExportExecute { .. } => Err(JamAppError::InvalidSession(
            "DAW session writer export execute uses a non-interactive proof path".into(),
        )),
        LaunchMode::W30HookDawprojectExecute { .. } => Err(JamAppError::InvalidSession(
            "W-30 DAWproject execute uses a non-interactive export path".into(),
        )),
        LaunchMode::DawSessionWriterPlan { .. } => Err(JamAppError::InvalidSession(
            "DAW session writer plan uses a non-interactive proof path".into(),
        )),
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
    let mut audio_runtime = start_audio_runtime_for_shell(
        &mut shell,
        observer.as_mut(),
        "started",
    )?;
    run_event_loop(
        terminal.terminal_mut(),
        shell,
        launch,
        &mut audio_runtime,
        observer.as_mut(),
    )
}

fn start_audio_runtime_for_shell(
    shell: &mut JamShellState,
    observer: Option<&mut UserSessionObserver>,
    success_state: &str,
) -> Result<Option<AudioRuntimeShell>, Box<dyn std::error::Error>> {
    let audio_runtime = match AudioRuntimeShell::start_default_output_with_render_states_and_source_monitor(
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
            shell.app.set_audio_health(audio_start_failure_health(&error));
            shell.set_error_status(format!("audio unavailable: {error}"));
            if let Some(observer) = observer {
                observer.record_audio_runtime("unavailable", Some(&error.to_string()), shell)?;
            }
            None
        }
    };
    Ok(audio_runtime)
}

fn audio_start_failure_health(error: &AudioRuntimeError) -> AudioRuntimeHealth {
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
