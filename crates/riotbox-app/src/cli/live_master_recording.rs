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
mod observation;
mod terminal_input;
use crate::cli::model::LiveRecordingInputMode;
use observation::{Observation, WindowStoppedSink};
use terminal_input::{RecordingInput, TerminalInput};

pub(super) fn run_live_master_recording_execute(
    launch: &AppLaunch,
    raw_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let input_mode = launch.mode.live_recording_input_mode();
    let mut observer = open_live_master_recording_observer(launch)?;
    let (execution, milestone_result) = {
        let sink = observer
            .as_mut()
            .map(|sink| sink as &mut dyn WindowStoppedSink);
        let mut observation = Observation::new(sink);
        let execution = live_master_recording_execute_summary(launch, &mut observation);
        (execution, observation.into_result())
    };
    let (mut summary, shell) = match execution {
        Ok(result) => result,
        Err(error) => {
            if let Some(observer) = observer.as_mut()
                && let Err(observer_error) = observer.record(json!({
                    "event": "live_master_recording_execute_failed",
                    "schema": "riotbox.user_session_observer.v1",
                    "timestamp_ms": timestamp_now(),
                    "opt_in": true,
                    "capture_context": input_mode.capture_context(),
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
    if milestone_result.is_err() {
        apply_live_master_observer_status(&mut summary, observer.is_some(), &milestone_result);
    }
    let observer_result = match observer.as_mut() {
        Some(observer) => observer.record(json!({
            "event": "live_master_recording_execute",
            "schema": "riotbox.user_session_observer.v1",
            "timestamp_ms": timestamp_now(),
            "opt_in": true,
            "capture_context": input_mode.capture_context(),
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
    let observer_result = milestone_result.and(observer_result);
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
    observation: &mut Observation<'_>,
) -> Result<(Value, JamShellState), Box<dyn std::error::Error>> {
    let LaunchMode::LiveMasterRecordingExecute {
        session_path,
        source_graph_path,
        ..
    } = &launch.mode
    else {
        return Err("not a live master recording execute launch".into());
    };
    let mut input = match launch.mode.live_recording_input_mode() {
        LiveRecordingInputMode::Disabled => None,
        LiveRecordingInputMode::TerminalFill => {
            // Brief before raw mode/device startup, never I/O during a take.
            eprintln!(
                "Recording input (Linux ASCII): f queues Fill; q/Ctrl-C or standalone Esc (50 ms) cancels while pending. Avoid paste/typeahead; unsupported key sequences abort. A queued Fill may miss this take; audio delivery is not confirmed."
            );
            Some(TerminalInput::open()?)
        }
    };
    let mut state = match JamAppState::from_json_files(session_path, source_graph_path.as_deref()) {
        Ok(state) => state,
        Err(error) => {
            let restoration = input.as_mut().map_or(Ok(()), TerminalInput::restore);
            return Err(with_terminal_restore_error(error.into(), restoration));
        }
    };
    // Borrow the single large facade, and return only a small summary through
    // this execution frame. Construct the shell only after runtime and terminal
    // cleanup, never move App/Shell through an extra resource-wrapper frame.
    let execution = (|| {
        let mut runtime =
            AudioRuntimeShell::start_default_output_with_render_states_and_source_monitor(
                state.runtime.tr909_render.clone(),
                state.runtime.mc202_render,
                state.runtime.w30_preview.clone(),
                state.runtime.w30_resample_tap.clone(),
                state.source_monitor_render_state(),
            )?;
        capture_lifecycle::execute(
            launch,
            &mut state,
            &mut runtime,
            observation,
            input.as_mut().map(|input| input as &mut dyn RecordingInput),
        )
    })();
    let restoration = input.as_mut().map_or(Ok(()), TerminalInput::restore);
    let summary = recording_result_after_terminal_restore(execution, restoration)?;
    Ok((summary, JamShellState::new(state, ShellLaunchMode::Load)))
}

fn with_terminal_restore_error(
    error: Box<dyn std::error::Error>,
    restoration: io::Result<()>,
) -> Box<dyn std::error::Error> {
    match restoration {
        Ok(()) => error,
        Err(restore_error) => {
            format!("{error}; additionally terminal restoration failed: {restore_error}").into()
        }
    }
}

fn recording_result_after_terminal_restore(
    execution: Result<Value, Box<dyn std::error::Error>>,
    restoration: io::Result<()>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match execution {
        Err(error) => Err(with_terminal_restore_error(error, restoration)),
        Ok(summary) => {
            // No successful stdout JSON without confirmed restoration. This
            // small reporting seam has no Session or file handles: cleanup
            // cannot erase or invalidate an already committed recording.
            if let Err(error) = restoration {
                return Err(format!(
                    "terminal restoration failed after recording execution (recording ready={}); any committed Session/receipt and published artifacts are retained: {error}",
                    summary["ready"],
                ).into());
            }
            Ok(summary)
        }
    }
}

#[cfg(test)]
mod terminal_result_tests {
    use super::recording_result_after_terminal_restore;
    use serde_json::json;
    use std::io;

    #[test]
    fn successful_restore_keeps_ready_and_blocked_recording_summaries_exact() {
        for ready in [false, true] {
            let summary = json!({"ready": ready, "receipt": {"receipt_id": "kept"}});
            assert_eq!(
                recording_result_after_terminal_restore(Ok(summary.clone()), Ok(())).unwrap(),
                summary
            );
        }
    }

    #[test]
    fn restore_failure_reports_recording_state_without_a_success_result() {
        for ready in [false, true] {
            let summary = json!({"ready": ready, "receipt": {"receipt_id": "kept"}});
            let error = recording_result_after_terminal_restore(
                Ok(summary),
                Err(io::Error::other("generated restore failure")),
            )
            .unwrap_err()
            .to_string();
            assert!(error.contains(&format!("recording ready={ready}")));
            assert!(
                error.contains("committed Session/receipt and published artifacts are retained")
            );
            assert!(error.contains("generated restore failure"));
        }
    }

    #[test]
    fn restore_failure_preserves_original_failure_cause() {
        let error = recording_result_after_terminal_restore(
            Err(io::Error::other("generated recording failure").into()),
            Err(io::Error::other("generated restore failure")),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("generated recording failure"));
        assert!(error.contains("generated restore failure"));
    }

    #[test]
    fn successful_restore_preserves_the_original_error_type() {
        let error = recording_result_after_terminal_restore(
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "generated load failure").into()),
            Ok(()),
        )
        .unwrap_err();
        assert_eq!(
            error.downcast_ref::<io::Error>().unwrap().kind(),
            io::ErrorKind::UnexpectedEof
        );
    }
}
