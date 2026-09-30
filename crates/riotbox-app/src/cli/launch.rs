use crate::cli::args::parse_args;
use crate::cli::daw_export_report::run_daw_export_readiness_report;
use crate::cli::daw_session_export::run_daw_session_host_import_proof_export_execute;
use crate::cli::daw_session_export::run_daw_session_writer_export_execute;
use crate::cli::daw_session_json_package::run_daw_session_audible_output_proof_apply;
use crate::cli::daw_session_json_package::run_daw_session_host_import_proof_apply;
use crate::cli::daw_session_json_package::run_daw_session_json_package_evidence_apply;
use crate::cli::daw_session_json_package::run_daw_session_json_package_execute;
use crate::cli::daw_session_writer_plan::run_daw_session_writer_plan;
use crate::cli::daw_session_writer_proof::run_daw_session_writer_proof_apply;
use crate::cli::daw_session_writer_proof::run_daw_session_writer_proof_execute;
use crate::cli::live_master_dawproject::run_live_master_dawproject_execute;
use crate::cli::live_master_recording::run_live_master_recording_execute;
use crate::cli::live_recording_report::run_live_recording_readiness_report;
use crate::cli::model::LaunchMode;
use crate::cli::stem_package_export::run_stem_package_local_ci_dry_run;
use crate::cli::stem_package_export::run_stem_package_local_ci_execute;
use crate::cli::stem_package_handoff::run_stem_package_source_matched_execute;
use crate::cli::stem_package_handoff::run_stem_package_w30_hook_execute;
use crate::cli::stem_package_report::run_stem_package_local_ci_report;
use crate::cli::terminal::run_terminal_ui;
use crate::cli::w30_hook_dawproject::run_w30_hook_dawproject_execute;
use crate::jam_app::JamAppError;
use crate::jam_app::JamAppState;
use crate::jam_app::SessionRecoverySurface;
use crate::ui::JamShellState;
use std::env;

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
    if matches!(launch.mode, LaunchMode::LiveMasterRecordingExecute { .. }) {
        run_live_master_recording_execute(&launch, &raw_args)?;
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
    if matches!(
        launch.mode,
        LaunchMode::DawSessionHostImportProofApply { .. }
    ) {
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
    if matches!(
        launch.mode,
        LaunchMode::DawSessionWriterExportExecute { .. }
    ) {
        run_daw_session_writer_export_execute(&launch, &raw_args)?;
        return Ok(());
    }
    if matches!(launch.mode, LaunchMode::W30HookDawprojectExecute { .. }) {
        run_w30_hook_dawproject_execute(&launch, &raw_args)?;
        return Ok(());
    }
    if matches!(launch.mode, LaunchMode::LiveMasterDawprojectExecute { .. }) {
        run_live_master_dawproject_execute(&launch, &raw_args)?;
        return Ok(());
    }
    if matches!(launch.mode, LaunchMode::DawSessionWriterPlan { .. }) {
        run_daw_session_writer_plan(&launch)?;
        return Ok(());
    }
    let state = load_state(launch.mode.clone())?;
    let shell = shell_for_loaded_state(state, &launch.mode);
    run_terminal_ui(shell, launch, &raw_args)?;
    Ok(())
}

pub(in crate::cli) fn shell_for_loaded_state(
    state: JamAppState,
    mode: &LaunchMode,
) -> JamShellState {
    let mut shell = JamShellState::new(state, mode.shell_launch_mode());
    refresh_recovery_surface_for_launch(&mut shell, mode);
    shell
}

pub(in crate::cli) fn refresh_recovery_surface_for_launch(
    shell: &mut JamShellState,
    mode: &LaunchMode,
) {
    shell.clear_recovery_surface();
    if let Some(recovery_surface) = recovery_surface_for_launch(mode) {
        shell.set_recovery_surface(recovery_surface);
    }
}

pub(in crate::cli) fn recovery_surface_for_launch(
    mode: &LaunchMode,
) -> Option<SessionRecoverySurface> {
    let LaunchMode::Load { session_path, .. } = mode else {
        return None;
    };

    JamAppState::scan_session_recovery_surface(session_path)
        .ok()
        .filter(SessionRecoverySurface::has_non_canonical_clues)
}

pub(in crate::cli) fn load_state(mode: LaunchMode) -> Result<JamAppState, JamAppError> {
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
        LaunchMode::LiveMasterRecordingExecute { .. } => Err(JamAppError::InvalidSession(
            "live master recording execute uses a non-interactive real-audio path".into(),
        )),
        LaunchMode::DawExportReadinessReport { .. } => Err(JamAppError::InvalidSession(
            "DAW export readiness report uses a non-interactive proof path".into(),
        )),
        LaunchMode::DawSessionJsonPackageExecute { .. } => Err(JamAppError::InvalidSession(
            "DAW session JSON package execute uses a non-interactive proof path".into(),
        )),
        LaunchMode::DawSessionJsonPackageEvidenceApply { .. } => Err(JamAppError::InvalidSession(
            "DAW session JSON package evidence apply uses a non-interactive proof path".into(),
        )),
        LaunchMode::DawSessionHostImportProofApply { .. } => Err(JamAppError::InvalidSession(
            "DAW session host import proof apply uses a non-interactive proof path".into(),
        )),
        LaunchMode::DawSessionHostImportProofExportExecute { .. } => {
            Err(JamAppError::InvalidSession(
                "DAW session host import proof export execute uses a non-interactive proof path"
                    .into(),
            ))
        }
        LaunchMode::DawSessionAudibleOutputProofApply { .. } => Err(JamAppError::InvalidSession(
            "DAW session audible output proof apply uses a non-interactive proof path".into(),
        )),
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
        LaunchMode::LiveMasterDawprojectExecute { .. } => Err(JamAppError::InvalidSession(
            "live-master DAWproject execute uses a non-interactive metadata-only export path"
                .into(),
        )),
        LaunchMode::DawSessionWriterPlan { .. } => Err(JamAppError::InvalidSession(
            "DAW session writer plan uses a non-interactive proof path".into(),
        )),
    }
}
