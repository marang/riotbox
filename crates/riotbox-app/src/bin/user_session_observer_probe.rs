use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    path::Path,
};

use crossterm::event::KeyCode;
use riotbox_app::{
    jam_app::{JamAppState, SourceMapNavigationIntent},
    observer::{compact_commit, key_code_label, observer_snapshot, shell_key_outcome_label},
    ui::{JamShellState, ShellKeyOutcome, ShellLaunchMode},
};
use riotbox_audio::runtime::{AudioRuntimeHealth, AudioRuntimeLifecycle};
use riotbox_core::{
    action::{ActionCommand, CommitBoundary},
    ids::SceneId,
    queue::{ActionQueue, CommittedActionRef},
    session::SessionFile,
    transport::{
        CommitBoundaryState, DEFAULT_BARS_PER_PHRASE, DEFAULT_BEATS_PER_BAR, TransportClockState,
        TransportGridPosition,
    },
};
use serde_json::{Value, json};

#[path = "user_session_observer_probe/cli.rs"]
mod cli;
#[path = "user_session_observer_probe/probe_scenarios.rs"]
mod probe_scenarios;
#[path = "user_session_observer_probe/source_map_navigation_control.rs"]
mod source_map_navigation_control;

use cli::{Args, print_help};
use probe_scenarios::{
    write_feral_grid_fallback_jam_observer, write_feral_grid_jam_observer,
    write_feral_grid_locked_jam_observer, write_first_playable_jam_observer,
    write_interrupted_session_recovery_observer, write_missing_target_recovery_observer,
    write_p014_scene_movement_observer, write_recipe2_mc202_observer,
    write_source_timing_confirmation_observer, write_source_transport_map_capture_observer,
    write_stage_style_jam_observer, write_stage_style_restore_diversity_observer,
};
use source_map_navigation_control::apply_source_map_navigation;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(std::env::args().skip(1))?;
    if args.show_help {
        print_help();
        return Ok(());
    }

    match args.probe.as_str() {
        "recipe2-mc202" => write_recipe2_mc202_observer(&args.observer_path)?,
        "first-playable-jam" => write_first_playable_jam_observer(&args.observer_path)?,
        "stage-style-jam" => write_stage_style_jam_observer(&args.observer_path)?,
        "stage-style-restore-diversity" => {
            write_stage_style_restore_diversity_observer(&args.observer_path)?
        }
        "interrupted-session-recovery" => {
            write_interrupted_session_recovery_observer(&args.observer_path)?
        }
        "missing-target-recovery" => write_missing_target_recovery_observer(&args.observer_path)?,
        "feral-grid-jam" => write_feral_grid_jam_observer(&args.observer_path)?,
        "feral-grid-jam-fallback" => write_feral_grid_fallback_jam_observer(&args.observer_path)?,
        "feral-grid-jam-locked" => write_feral_grid_locked_jam_observer(&args.observer_path)?,
        "source-timing-confirmation" => {
            write_source_timing_confirmation_observer(&args.observer_path)?
        }
        "source-transport-map-capture" => {
            write_source_transport_map_capture_observer(&args.observer_path)?
        }
        "p014-scene-movement" => write_p014_scene_movement_observer(&args.observer_path)?,
        other => {
            return Err(format!(
                "unknown probe {other:?}; supported probes: recipe2-mc202, first-playable-jam, stage-style-jam, stage-style-restore-diversity, interrupted-session-recovery, missing-target-recovery, feral-grid-jam, feral-grid-jam-fallback, feral-grid-jam-locked, source-timing-confirmation, source-transport-map-capture, p014-scene-movement"
            )
            .into());
        }
    }

    Ok(())
}

fn probe_shell(session_id: &str) -> JamShellState {
    let mut session = SessionFile::new(session_id, "0.1.0", "2026-04-30T00:00:00Z");
    session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
    session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());

    JamShellState::new(
        JamAppState::from_parts(session, None, ActionQueue::new()),
        ShellLaunchMode::Ingest,
    )
}

fn record_probe_start(
    writer: &mut NdjsonWriter,
    shell: &mut JamShellState,
    path: &Path,
    probe: &str,
    source_path: &str,
    session_path: &str,
) -> io::Result<()> {
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
            probe,
            "--observer",
            path.display().to_string(),
        ],
        "launch": {
            "mode": "ingest",
            "source_path": source_path,
            "session_path": session_path,
            "source_graph_path": null,
            "sidecar_script_path": null,
            "analysis_seed": 19,
            "observer_path": path.display().to_string(),
            "probe": probe,
        },
        "snapshot": observer_snapshot(shell),
    }))?;
    shell.app.set_audio_health(headless_audio_health());
    writer.record(json!({
        "event": "audio_runtime",
        "timestamp_ms": 10,
        "status": "started",
        "error": null,
        "host": "headless-probe",
        "snapshot": observer_snapshot(shell),
    }))
}

fn apply_probe_key(
    shell: &mut JamShellState,
    writer: &mut NdjsonWriter,
    timestamp_ms: u64,
    key: KeyCode,
) -> io::Result<()> {
    let outcome = shell.handle_key_code(key);
    let mut immediate_committed = Vec::new();

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
        ShellKeyOutcome::QueueSourceMonitorMode(mode) => {
            match shell.app.queue_source_monitor_mode(mode, timestamp_ms) {
                riotbox_app::jam_app::QueueControlResult::Enqueued => {
                    let transport = shell.app.runtime.transport.clone();
                    let boundary = transport.boundary_state(CommitBoundary::Immediate);
                    immediate_committed = shell
                        .app
                        .commit_ready_actions(boundary.clone(), timestamp_ms);
                    require_immediate_commit(
                        shell,
                        &immediate_committed,
                        &boundary,
                        ActionCommand::SourceMonitorSetMode,
                        "source monitor mode",
                    )?;
                    shell.set_error_status(format!(
                        "monitor {} landed | route {}",
                        shell.app.runtime_view.source_monitor_mode,
                        shell.app.runtime_view.source_monitor_audio_route
                    ));
                }
                riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                    shell.set_error_status("source monitor change already queued");
                }
                riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                    shell.set_error_status(format!("monitor already {mode}"));
                }
            }
        }
        ShellKeyOutcome::QueuePerformancePreset(preset_id) => {
            match shell.app.queue_performance_preset(preset_id, timestamp_ms) {
                riotbox_app::jam_app::QueueControlResult::Enqueued => {
                    let transport = shell.app.runtime.transport.clone();
                    let boundary = transport.boundary_state(CommitBoundary::Immediate);
                    immediate_committed = shell
                        .app
                        .commit_ready_actions(boundary.clone(), timestamp_ms);
                    require_immediate_commit(
                        shell,
                        &immediate_committed,
                        &boundary,
                        ActionCommand::PresetActivate,
                        "performance preset",
                    )?;
                    shell.set_error_status(format!("{} landed", preset_id.label()));
                }
                riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                    shell.set_error_status("performance preset already queued");
                }
                riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                    shell.set_error_status(format!("{} already active", preset_id.label()));
                }
            }
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
        ShellKeyOutcome::QueueTr909Fill => {
            shell.app.queue_tr909_fill(timestamp_ms);
            shell.set_error_status("queued TR-909 fill for next bar");
        }
        ShellKeyOutcome::QueueTr909Slam => {
            if shell.app.queue_tr909_slam_toggle(timestamp_ms) {
                shell.set_error_status("queued TR-909 slam change for next beat");
            } else {
                shell.set_error_status("TR-909 slam change already queued");
            }
        }
        ShellKeyOutcome::QueueSceneSelect => match shell.app.queue_scene_select(timestamp_ms) {
            riotbox_app::jam_app::QueueControlResult::Enqueued => {
                shell.set_error_status("queued scene select for next bar");
            }
            riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                shell.set_error_status("scene transition already queued");
            }
            riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                shell.set_error_status("no next scene available");
            }
        },
        ShellKeyOutcome::QueueSceneRestore => match shell.app.queue_scene_restore(timestamp_ms) {
            riotbox_app::jam_app::QueueControlResult::Enqueued => {
                shell.set_error_status("queued scene restore for next bar");
            }
            riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                shell.set_error_status("scene transition already queued");
            }
            riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                shell.set_error_status("no restore scene available");
            }
        },
        ShellKeyOutcome::QueueTr909Reinforce => {
            shell.app.queue_tr909_reinforce(timestamp_ms);
            shell.set_error_status("queued TR-909 reinforcement for next phrase");
        }
        ShellKeyOutcome::QueueTr909SceneLock => {
            match shell.app.queue_tr909_scene_lock(timestamp_ms) {
                riotbox_app::jam_app::QueueControlResult::Enqueued => {
                    shell.set_error_status("queued TR-909 scene lock for next phrase");
                }
                riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                    shell.set_error_status("TR-909 takeover change already queued");
                }
                riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                    shell.set_error_status("TR-909 scene lock already in state");
                }
            }
        }
        ShellKeyOutcome::QueueTr909Release => match shell.app.queue_tr909_release(timestamp_ms) {
            riotbox_app::jam_app::QueueControlResult::Enqueued => {
                shell.set_error_status("queued TR-909 release for next phrase");
            }
            riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                shell.set_error_status("TR-909 takeover change already queued");
            }
            riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                shell.set_error_status("TR-909 takeover already released");
            }
        },
        ShellKeyOutcome::QueueCaptureBar => {
            shell.app.queue_capture_bar(timestamp_ms);
            shell.set_error_status("queued capture for next phrase");
        }
        ShellKeyOutcome::PromoteLastCapture => {
            if shell.app.queue_promote_last_capture(timestamp_ms) {
                shell.set_error_status("queued promotion for latest capture");
            } else {
                shell.set_error_status("no promotable capture or W-30 target available");
            }
        }
        ShellKeyOutcome::QueueW30Audition => match shell.app.queue_w30_audition(timestamp_ms) {
            Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                shell.set_error_status("queued W-30 audition for next bar");
            }
            Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                shell.set_error_status("W-30 pad cue already queued");
            }
            Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                shell.set_error_status("W-30 audition already in state");
            }
            None => shell.set_error_status("no W-30 or raw capture available to audition"),
        },
        ShellKeyOutcome::QueueW30LiveRecall => {
            match shell.app.queue_w30_live_recall(timestamp_ms) {
                Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                    shell.set_error_status("queued W-30 live recall for next bar");
                }
                Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                    shell.set_error_status("W-30 live recall already queued");
                }
                Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                    shell.set_error_status("W-30 live recall already in state");
                }
                None => {
                    shell
                        .set_error_status("no pinned or promoted W-30 capture available to recall");
                }
            }
        }
        ShellKeyOutcome::QueueW30TriggerPad => {
            match shell.app.queue_w30_trigger_pad(timestamp_ms) {
                Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                    shell.set_error_status("queued W-30 pad trigger for next beat");
                }
                Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                    shell.set_error_status("W-30 pad cue already queued");
                }
                Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                    shell.set_error_status("W-30 pad trigger already in state");
                }
                None => shell.set_error_status("no committed W-30 pad available to trigger"),
            }
        }
        ShellKeyOutcome::QueueW30ApplyDamageProfile => {
            match shell.app.queue_w30_apply_damage_profile(timestamp_ms) {
                Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                    shell.set_error_status("queued W-30 damage for next bar");
                }
                Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                    shell.set_error_status("W-30 damage already queued");
                }
                Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                    shell.set_error_status("W-30 damage already in state");
                }
                None => shell.set_error_status("no committed W-30 pad available to damage"),
            }
        }
        ShellKeyOutcome::QueueW30HookTurnaround => {
            match shell.app.queue_w30_hook_turnaround(timestamp_ms) {
                Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                    shell.set_error_status("queued W-30 hook turnaround for next bar");
                }
                Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                    shell.set_error_status("W-30 hook turnaround already queued");
                }
                Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                    shell.set_error_status("W-30 hook turnaround already active");
                }
                None => shell.set_error_status("no committed W-30 pad available to turn around"),
            }
        }
        ShellKeyOutcome::QueueW30PitchDive => match shell.app.queue_w30_pitch_dive(timestamp_ms) {
            Some(riotbox_app::jam_app::QueueControlResult::Enqueued) => {
                shell.set_error_status("queued W-30 pitch dive for next bar");
            }
            Some(riotbox_app::jam_app::QueueControlResult::AlreadyPending) => {
                shell.set_error_status("W-30 pitch dive already queued");
            }
            Some(riotbox_app::jam_app::QueueControlResult::AlreadyInState) => {
                shell.set_error_status("W-30 pitch dive already active");
            }
            None => shell.set_error_status("no committed W-30 pad available to pitch-dive"),
        },
        ShellKeyOutcome::RaiseMc202Touch => {
            let touch = shell.app.adjust_mc202_touch(0.08);
            shell.set_error_status(format!("MC-202 touch {touch:.2}"));
        }
        ShellKeyOutcome::ConfirmSourceTimingGrid => {
            match shell
                .app
                .queue_source_timing_grid_confirmation(timestamp_ms)
            {
                riotbox_app::jam_app::QueueControlResult::Enqueued => {
                    let transport = shell.app.runtime.transport.clone();
                    immediate_committed = shell.app.commit_ready_actions(
                        CommitBoundaryState {
                            kind: CommitBoundary::Immediate,
                            beat_index: transport.beat_index,
                            bar_index: transport.bar_index,
                            phrase_index: transport.phrase_index,
                            scene_id: transport.current_scene,
                        },
                        timestamp_ms,
                    );
                    if immediate_committed.is_empty() {
                        shell.set_error_status("source timing grid confirmation queued");
                    } else {
                        shell.set_error_status("confirmed source timing grid");
                    }
                }
                riotbox_app::jam_app::QueueControlResult::AlreadyPending => {
                    shell.set_error_status("source timing grid trust change already queued");
                }
                riotbox_app::jam_app::QueueControlResult::AlreadyInState => {
                    if shell.app.source_graph.is_some() {
                        shell.set_error_status("source timing grid already confirmed");
                    } else {
                        shell.set_error_status("no source timing grid available to confirm");
                    }
                }
            }
        }
        ShellKeyOutcome::NavigateSourceMapPreviousBar => {
            immediate_committed = apply_source_map_navigation(
                shell,
                SourceMapNavigationIntent::PreviousBar,
                timestamp_ms,
            );
        }
        ShellKeyOutcome::NavigateSourceMapNextBar => {
            immediate_committed = apply_source_map_navigation(
                shell,
                SourceMapNavigationIntent::NextBar,
                timestamp_ms,
            );
        }
        ShellKeyOutcome::NavigateSourceMapPreviousPhrase => {
            immediate_committed = apply_source_map_navigation(
                shell,
                SourceMapNavigationIntent::PreviousPhrase,
                timestamp_ms,
            );
        }
        ShellKeyOutcome::NavigateSourceMapNextPhrase => {
            immediate_committed = apply_source_map_navigation(
                shell,
                SourceMapNavigationIntent::NextPhrase,
                timestamp_ms,
            );
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
    }))?;

    if !immediate_committed.is_empty() {
        writer.record(json!({
            "event": "transport_commit",
            "timestamp_ms": timestamp_ms,
            "committed": immediate_committed.iter().map(compact_commit).collect::<Vec<_>>(),
            "snapshot": observer_snapshot(shell),
        }))?;
    }

    Ok(())
}

fn require_immediate_commit(
    shell: &JamShellState,
    committed: &[CommittedActionRef],
    expected_boundary: &CommitBoundaryState,
    expected_command: ActionCommand,
    label: &str,
) -> io::Result<()> {
    let [committed] = committed else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{label} key must commit exactly one action immediately, got {}",
                committed.len()
            ),
        ));
    };
    if committed.boundary != *expected_boundary
        || committed.boundary.kind != CommitBoundary::Immediate
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{label} key committed at {:?}, expected current-clock {:?}",
                committed.boundary, expected_boundary
            ),
        ));
    }

    let command = shell
        .app
        .queue
        .history()
        .iter()
        .find(|action| action.id == committed.action_id)
        .map(|action| action.command);
    if command != Some(expected_command) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{label} key committed unexpected action {command:?}, expected {expected_command:?}",
            ),
        ));
    }

    Ok(())
}

fn commit_boundary(
    shell: &mut JamShellState,
    writer: &mut NdjsonWriter,
    timestamp_ms: u64,
    kind: CommitBoundary,
    index: u64,
    expected_count: usize,
) -> io::Result<()> {
    let beats_per_phrase = DEFAULT_BEATS_PER_BAR.saturating_mul(DEFAULT_BARS_PER_PHRASE);
    let grid_position = source_aware_grid_position(shell, index.saturating_mul(beats_per_phrase));
    let scene_id = Some(SceneId::from("scene-1"));
    shell.app.update_transport_clock(TransportClockState {
        is_playing: shell.app.runtime.transport.is_playing,
        position_beats: grid_position.beat_cursor as f64,
        beat_index: grid_position.beat_cursor,
        bar_index: grid_position.bar_index,
        phrase_index: grid_position.phrase_index,
        current_scene: scene_id.clone(),
    });
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind,
            beat_index: grid_position.beat_cursor,
            bar_index: grid_position.bar_index,
            phrase_index: grid_position.phrase_index,
            scene_id,
        },
        timestamp_ms,
    );
    record_boundary_commit(
        shell,
        writer,
        timestamp_ms,
        committed,
        kind,
        index,
        expected_count,
    )
}

fn source_aware_grid_position(shell: &JamShellState, position_beats: u64) -> TransportGridPosition {
    shell
        .app
        .source_graph
        .as_ref()
        .and_then(|graph| graph.timing.primary_hypothesis())
        .and_then(|hypothesis| hypothesis.transport_grid_position(position_beats as f64))
        .unwrap_or_else(|| {
            TransportGridPosition::from_zero_based_position_beats(
                position_beats as f64,
                DEFAULT_BEATS_PER_BAR,
                DEFAULT_BARS_PER_PHRASE,
            )
        })
}

fn commit_boundary_for_scene(
    shell: &mut JamShellState,
    writer: &mut NdjsonWriter,
    timestamp_ms: u64,
    boundary: CommitBoundaryState,
    expected_count: usize,
) -> io::Result<()> {
    let kind = boundary.kind;
    let phrase_index = boundary.phrase_index;
    let committed = shell.app.commit_ready_actions(boundary, timestamp_ms);
    record_boundary_commit(
        shell,
        writer,
        timestamp_ms,
        committed,
        kind,
        phrase_index,
        expected_count,
    )
}

fn record_boundary_commit(
    shell: &mut JamShellState,
    writer: &mut NdjsonWriter,
    timestamp_ms: u64,
    committed: Vec<riotbox_core::queue::CommittedActionRef>,
    kind: CommitBoundary,
    index: u64,
    expected_count: usize,
) -> io::Result<()> {
    if committed.len() != expected_count {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "expected {expected_count} committed action(s) at {kind:?} index {index}, got {}",
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
        callback_scratch_overflow_count: 0,
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
#[path = "user_session_observer_probe/source_transport_map_capture_tests.rs"]
mod source_transport_map_capture_tests;
#[cfg(test)]
#[path = "user_session_observer_probe/tests.rs"]
mod tests;
