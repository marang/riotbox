use crate::cli::model::AppLaunch;
use crate::cli::model::LaunchMode;
use crate::cli::observer::UserSessionObserver;
use crate::cli::observer::launch_summary;
use crate::cli::observer::timestamp_now;
use crate::jam_app::JamAppState;
use crate::observer::observer_snapshot;
use crate::ui::{JamShellState, ShellLaunchMode};
use riotbox_audio::runtime::AudioRuntimeShell;
use serde_json::Value;
use serde_json::json;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
mod capture_lifecycle;

pub(super) fn run_live_master_recording_execute(
    launch: &AppLaunch,
    raw_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut observer = open_live_master_recording_observer(launch)?;
    let (mut summary, shell) = match live_master_recording_execute_summary(launch) {
        Ok(result) => result,
        Err(error) => {
            if let Some(observer) = observer.as_mut()
                && let Err(observer_error) = observer.record(json!({
                    "event": "live_master_recording_execute_failed",
                    "schema": "riotbox.user_session_observer.v1",
                    "timestamp_ms": timestamp_now(),
                    "opt_in": true,
                    "capture_context": "non_interactive_real_audio_cli",
                    "raw_audio_recording": false,
                    "live_master_recording": true,
                    "realtime_callback_io": false,
                    "argv": raw_args,
                    "launch": launch_summary(launch),
                    "failure_reason": error.to_string(),
                }))
            {
                eprintln!(
                    "live master recording observer could not record launch failure: {observer_error}"
                );
            }
            return Err(error);
        }
    };
    let observer_result = match observer.as_mut() {
        Some(observer) => observer.record(json!({
            "event": "live_master_recording_execute",
            "schema": "riotbox.user_session_observer.v1",
            "timestamp_ms": timestamp_now(),
            "opt_in": true,
            "capture_context": "non_interactive_real_audio_cli",
            "raw_audio_recording": false,
            "live_master_recording": true,
            "realtime_callback_io": false,
            "argv": raw_args,
            "launch": launch_summary(launch),
            "summary": summary.clone(),
            "snapshot": observer_snapshot(&shell),
        })),
        None => Ok(()),
    };
    apply_live_master_observer_status(&mut summary, observer.is_some(), &observer_result);
    if let Err(error) = observer_result {
        eprintln!(
            "live master recording completed, but the optional observer write failed: {error}"
        );
    }
    serde_json::to_writer_pretty(std::io::stdout(), &summary)?;
    println!();
    if summary.get("ready").and_then(Value::as_bool) == Some(true) {
        Ok(())
    } else {
        Err(summary
            .get("failure_reason")
            .and_then(Value::as_str)
            .unwrap_or("live master recording failed")
            .to_owned()
            .into())
    }
}

pub(super) fn apply_live_master_observer_status(
    summary: &mut Value,
    observer_requested: bool,
    observer_result: &io::Result<()>,
) {
    match (observer_requested, observer_result) {
        (true, Ok(())) => {
            summary["observer_events"] = Value::Bool(true);
            summary["observer_status"] = Value::String("recorded".into());
        }
        (true, Err(error)) => {
            summary["observer_events"] = Value::Bool(false);
            summary["observer_status"] = Value::String("write_failed".into());
            summary["observer_failure_reason"] = Value::String(error.to_string());
        }
        (false, _) => {
            summary["observer_events"] = Value::Bool(false);
            summary["observer_status"] = Value::String("not_requested".into());
        }
    }
}

fn open_live_master_recording_observer(
    launch: &AppLaunch,
) -> Result<Option<UserSessionObserver>, Box<dyn std::error::Error>> {
    let Some(observer_path) = launch.observer_path.as_deref() else {
        return Ok(None);
    };
    let LaunchMode::LiveMasterRecordingExecute {
        session_path,
        source_graph_path,
        destination_path,
        ..
    } = &launch.mode
    else {
        return Err("live master observer requires live master recording mode".into());
    };
    validate_live_master_observer_path(
        observer_path,
        session_path,
        source_graph_path.as_deref(),
        destination_path,
    )?;
    Ok(Some(UserSessionObserver::open_new(observer_path)?))
}

pub(super) fn validate_live_master_observer_path(
    observer_path: &Path,
    session_path: &Path,
    source_graph_path: Option<&Path>,
    destination_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let proof_path = crate::jam_app::live_master_recording_proof_path(destination_path)?;
    let mut protected_paths = vec![
        ("Session", fs::canonicalize(session_path)?),
        (
            "recording destination",
            canonical_future_path(destination_path, false)?,
        ),
        (
            "recording proof",
            canonical_future_path(&proof_path, false)?,
        ),
    ];
    if let Some(source_graph_path) = source_graph_path {
        protected_paths.push(("Source Graph", fs::canonicalize(source_graph_path)?));
    }
    let observer_exists = match fs::symlink_metadata(observer_path) {
        Ok(_) => true,
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Err(error) => return Err(error.into()),
    };
    let observer_identity = if observer_exists {
        fs::canonicalize(observer_path).unwrap_or_else(|_| observer_path.to_path_buf())
    } else {
        canonical_future_path(observer_path, true)?
    };
    if let Some((label, _)) = protected_paths
        .iter()
        .find(|(_, protected_path)| *protected_path == observer_identity)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("live master observer path aliases the {label} path"),
        )
        .into());
    }
    if observer_exists {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "live master observer path must be fresh and must not already exist",
        )
        .into());
    }
    Ok(())
}

fn canonical_future_path(path: &Path, create_parent: bool) -> io::Result<PathBuf> {
    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "live master path requires an explicit file name",
        )
    })?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if create_parent {
        fs::create_dir_all(parent)?;
    }
    Ok(fs::canonicalize(parent)?.join(file_name))
}

fn live_master_recording_execute_summary(
    launch: &AppLaunch,
) -> Result<(Value, JamShellState), Box<dyn std::error::Error>> {
    let LaunchMode::LiveMasterRecordingExecute {
        session_path,
        source_graph_path,
        ..
    } = &launch.mode
    else {
        return Err("not a live master recording execute launch".into());
    };

    let mut state = JamAppState::from_json_files(session_path, source_graph_path.as_deref())?;
    let mut runtime =
        AudioRuntimeShell::start_default_output_with_render_states_and_source_monitor(
            state.runtime.tr909_render.clone(),
            state.runtime.mc202_render,
            state.runtime.w30_preview.clone(),
            state.runtime.w30_resample_tap.clone(),
            state.source_monitor_render_state(),
        )?;
    let summary = capture_lifecycle::execute(launch, &mut state, &mut runtime)?;
    Ok((summary, JamShellState::new(state, ShellLaunchMode::Load)))
}
