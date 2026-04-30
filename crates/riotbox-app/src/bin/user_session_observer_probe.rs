use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};

use crossterm::event::KeyCode;
use riotbox_app::{
    jam_app::JamAppState,
    observer::{compact_commit, key_code_label, observer_snapshot, shell_key_outcome_label},
    ui::{JamShellState, ShellKeyOutcome, ShellLaunchMode},
};
use riotbox_audio::runtime::{AudioRuntimeHealth, AudioRuntimeLifecycle};
use riotbox_core::{
    action::CommitBoundary, ids::SceneId, queue::ActionQueue, session::SessionFile,
    transport::CommitBoundaryState,
};
use serde_json::{Value, json};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(std::env::args().skip(1))?;
    if args.show_help {
        print_help();
        return Ok(());
    }

    match args.probe.as_str() {
        "recipe2-mc202" => write_recipe2_mc202_observer(&args.observer_path)?,
        other => {
            return Err(format!("unknown probe {other:?}; supported probes: recipe2-mc202").into());
        }
    }

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct Args {
    probe: String,
    observer_path: PathBuf,
    show_help: bool,
}

impl Args {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut probe = None;
        let mut observer_path = None;
        let mut show_help = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => show_help = true,
                "--probe" => {
                    probe = Some(
                        args.next()
                            .ok_or_else(|| "--probe requires a value".to_string())?,
                    );
                }
                "--observer" => {
                    observer_path = Some(PathBuf::from(
                        args.next()
                            .ok_or_else(|| "--observer requires a path".to_string())?,
                    ));
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        if show_help {
            return Ok(Self {
                probe: String::new(),
                observer_path: PathBuf::new(),
                show_help,
            });
        }

        Ok(Self {
            probe: probe.ok_or_else(|| "--probe is required".to_string())?,
            observer_path: observer_path.ok_or_else(|| "--observer is required".to_string())?,
            show_help,
        })
    }
}

fn print_help() {
    println!(
        "Usage:\n  user_session_observer_probe --probe recipe2-mc202 --observer <events.ndjson>"
    );
}

fn write_recipe2_mc202_observer(path: &Path) -> io::Result<()> {
    let mut writer = NdjsonWriter::open(path)?;
    let mut shell = recipe_probe_shell();

    writer.record(json!({
        "event": "observer_started",
        "schema": "riotbox.user_session_observer.v1",
        "timestamp_ms": 0,
        "opt_in": true,
        "capture_context": "headless_probe",
        "raw_audio_recording": false,
        "realtime_callback_io": false,
        "argv": [
            "user_session_observer_probe",
            "--probe",
            "recipe2-mc202",
            "--observer",
            path.display().to_string(),
        ],
        "launch": {
            "mode": "ingest",
            "source_path": "synthetic-recipe2-mc202-probe.wav",
            "session_path": "headless-recipe2-session.json",
            "source_graph_path": null,
            "sidecar_script_path": null,
            "analysis_seed": 19,
            "observer_path": path.display().to_string(),
            "probe": "recipe2-mc202",
        },
        "snapshot": observer_snapshot(&shell),
    }))?;
    shell.app.set_audio_health(headless_audio_health());
    writer.record(json!({
        "event": "audio_runtime",
        "timestamp_ms": 10,
        "status": "started",
        "error": null,
        "host": "headless-probe",
        "snapshot": observer_snapshot(&shell),
    }))?;

    apply_recipe_key(&mut shell, &mut writer, 100, KeyCode::Char(' '))?;
    apply_recipe_key(&mut shell, &mut writer, 300, KeyCode::Char('g'))?;
    commit_phrase(&mut shell, &mut writer, 400, 1)?;
    apply_recipe_key(&mut shell, &mut writer, 500, KeyCode::Char('a'))?;
    commit_phrase(&mut shell, &mut writer, 600, 2)?;
    apply_recipe_key(&mut shell, &mut writer, 700, KeyCode::Char('P'))?;
    commit_phrase(&mut shell, &mut writer, 800, 3)?;
    apply_recipe_key(&mut shell, &mut writer, 900, KeyCode::Char('I'))?;
    commit_phrase(&mut shell, &mut writer, 1_000, 4)?;
    apply_recipe_key(&mut shell, &mut writer, 1_100, KeyCode::Char('G'))?;
    commit_phrase(&mut shell, &mut writer, 1_200, 5)?;
    apply_recipe_key(&mut shell, &mut writer, 1_300, KeyCode::Char('>'))?;

    Ok(())
}

fn recipe_probe_shell() -> JamShellState {
    JamShellState::new(
        JamAppState::from_parts(
            SessionFile::new("recipe2-mc202-probe", "0.1.0", "2026-04-30T00:00:00Z"),
            None,
            ActionQueue::new(),
        ),
        ShellLaunchMode::Ingest,
    )
}

fn apply_recipe_key(
    shell: &mut JamShellState,
    writer: &mut NdjsonWriter,
    timestamp_ms: u64,
    key: KeyCode,
) -> io::Result<()> {
    let outcome = shell.handle_key_code(key);

    match outcome {
        ShellKeyOutcome::ToggleTransport => {
            let next_is_playing = !shell.app.runtime.transport.is_playing;
            shell.app.set_transport_playing(next_is_playing);
            shell.set_error_status(if next_is_playing {
                "transport started"
            } else {
                "transport paused"
            });
        }
        ShellKeyOutcome::QueueMc202GenerateFollower => {
            match shell.app.queue_mc202_generate_follower(timestamp_ms) {
                riotbox_app::jam_app::QueueControlResult::Enqueued => {
                    shell.set_error_status("queued MC-202 follower generation for next phrase");
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
            match shell.app.queue_mc202_generate_answer(timestamp_ms) {
                riotbox_app::jam_app::QueueControlResult::Enqueued => {
                    shell.set_error_status("queued MC-202 answer generation for next phrase");
                }
                riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                    shell.set_error_status("MC-202 answer generation already queued");
                }
                riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                    shell.set_error_status("MC-202 answer already in state");
                }
            }
        }
        ShellKeyOutcome::QueueMc202GeneratePressure => {
            match shell.app.queue_mc202_generate_pressure(timestamp_ms) {
                riotbox_app::jam_app::QueueControlResult::Enqueued => {
                    shell.set_error_status("queued MC-202 pressure generation for next phrase");
                }
                riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                    shell.set_error_status("MC-202 pressure generation already queued");
                }
                riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                    shell.set_error_status("MC-202 pressure already in state");
                }
            }
        }
        ShellKeyOutcome::QueueMc202GenerateInstigator => {
            match shell.app.queue_mc202_generate_instigator(timestamp_ms) {
                riotbox_app::jam_app::QueueControlResult::Enqueued => {
                    shell.set_error_status("queued MC-202 instigator generation for next phrase");
                }
                riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                    shell.set_error_status("MC-202 phrase control already queued");
                }
                riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                    shell.set_error_status("MC-202 instigator already in state");
                }
            }
        }
        ShellKeyOutcome::QueueMc202MutatePhrase => {
            match shell.app.queue_mc202_mutate_phrase(timestamp_ms) {
                riotbox_app::jam_app::QueueControlResult::Enqueued => {
                    shell.set_error_status("queued MC-202 phrase mutation for next phrase");
                }
                riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                    shell.set_error_status("MC-202 phrase control already queued");
                }
                riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                    shell.set_error_status("set an MC-202 voice before mutating phrase");
                }
            }
        }
        ShellKeyOutcome::RaiseMc202Touch => {
            let touch = shell.app.adjust_mc202_touch(0.08);
            shell.set_error_status(format!("MC-202 touch {touch:.2}"));
        }
        other => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected recipe key outcome: {other:?}"),
            ));
        }
    }

    writer.record(json!({
        "event": "key_outcome",
        "timestamp_ms": timestamp_ms,
        "key": key_code_label(key),
        "outcome": shell_key_outcome_label(outcome),
        "status": shell.status_message,
        "snapshot": observer_snapshot(shell),
    }))
}

fn commit_phrase(
    shell: &mut JamShellState,
    writer: &mut NdjsonWriter,
    timestamp_ms: u64,
    phrase_index: u64,
) -> io::Result<()> {
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: phrase_index * 16,
            bar_index: phrase_index * 4,
            phrase_index,
            scene_id: Some(SceneId::from("scene-1")),
        },
        timestamp_ms,
    );
    if committed.len() != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "expected one committed Recipe 2 action at phrase {phrase_index}, got {}",
                committed.len()
            ),
        ));
    }

    writer.record(json!({
        "event": "transport_commit",
        "timestamp_ms": timestamp_ms,
        "committed": committed.iter().map(compact_commit).collect::<Vec<_>>(),
        "snapshot": observer_snapshot(shell),
    }))
}

fn headless_audio_health() -> AudioRuntimeHealth {
    AudioRuntimeHealth {
        lifecycle: AudioRuntimeLifecycle::Running,
        output: None,
        callback_count: 0,
        max_callback_gap_micros: None,
        stream_error_count: 0,
        last_stream_error: None,
    }
}

struct NdjsonWriter {
    writer: BufWriter<File>,
}

impl NdjsonWriter {
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

    fn record(&mut self, event: Value) -> io::Result<()> {
        serde_json::to_writer(&mut self.writer, &event).map_err(io::Error::other)?;
        writeln!(self.writer)?;
        self.writer.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_required_probe_args() {
        let args = Args::parse([
            "--probe".into(),
            "recipe2-mc202".into(),
            "--observer".into(),
            "events.ndjson".into(),
        ])
        .expect("parse args");

        assert_eq!(args.probe, "recipe2-mc202");
        assert_eq!(args.observer_path, PathBuf::from("events.ndjson"));
        assert!(!args.show_help);
    }

    #[test]
    fn writes_recipe2_mc202_observer_stream() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("events.ndjson");

        write_recipe2_mc202_observer(&path).expect("write observer");

        let events = fs::read_to_string(path).expect("read observer");
        assert!(events.contains(r#""schema":"riotbox.user_session_observer.v1""#));
        assert!(events.contains(r#""capture_context":"headless_probe""#));
        assert!(events.contains(r#""snapshot":{"#));
        assert!(events.contains(r#""transport":{"#));
        assert!(events.contains(r#""queue":{"#));
        assert!(events.contains(r#""runtime":{"#));
        assert!(events.contains(r#""recovery":{"#));
        assert!(events.contains(r#""outcome":"queue_mc202_generate_follower""#));
        assert!(events.contains(r#""outcome":"queue_mc202_generate_answer""#));
        assert!(events.contains(r#""outcome":"queue_mc202_generate_pressure""#));
        assert!(events.contains(r#""outcome":"queue_mc202_generate_instigator""#));
        assert!(events.contains(r#""outcome":"queue_mc202_mutate_phrase""#));
        assert!(events.contains(r#""outcome":"raise_mc202_touch""#));
        assert_eq!(events.matches(r#""boundary":"Phrase""#).count(), 5);
    }
}
