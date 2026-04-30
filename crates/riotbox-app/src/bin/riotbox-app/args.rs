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
            "mc202_mix": runtime.mc202_render_mix_summary,
            "w30_preview_mode": runtime.w30_preview_mode,
            "w30_preview_target": runtime.w30_preview_target_summary,
            "w30_resample_tap_mode": runtime.w30_resample_tap_mode,
            "warnings": runtime.runtime_warnings,
        },
        "recovery": recovery_observer_snapshot(shell),
    })
}

fn recovery_observer_snapshot(shell: &JamShellState) -> Value {
    let Some(surface) = shell.recovery_surface.as_ref() else {
        return json!({
            "present": false,
            "has_manual_candidates": false,
            "selected_candidate": null,
            "candidate_count": 0,
            "candidates": [],
        });
    };

    json!({
        "present": true,
        "headline": surface.headline,
        "safety_note": surface.safety_note,
        "target_path": surface.target_path,
        "has_manual_candidates": surface.has_manual_candidates(),
        "selected_candidate": surface.selected_candidate,
        "candidate_count": surface.candidates.len(),
        "candidates": surface.candidates.iter().map(|candidate| {
            json!({
                "path": candidate.path,
                "kind": candidate.kind_label,
                "status": candidate.status_label,
                "artifact_availability": candidate.artifact_availability_label,
                "replay_readiness": candidate.replay_readiness_label,
                "payload_readiness": candidate.payload_readiness_label,
                "replay_suffix": candidate.replay_suffix_label,
                "replay_unsupported": candidate.replay_unsupported_label,
                "guidance": candidate.guidance.as_ref().map(|guidance| guidance.help_label()),
                "trust": format!("{:?}", candidate.trust),
                "action_hint": candidate.action_hint,
            })
        }).collect::<Vec<_>>(),
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
        ShellKeyOutcome::QueueMc202GeneratePressure => "queue_mc202_generate_pressure",
        ShellKeyOutcome::QueueMc202GenerateInstigator => "queue_mc202_generate_instigator",
        ShellKeyOutcome::QueueMc202MutatePhrase => "queue_mc202_mutate_phrase",
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
        ShellKeyOutcome::LowerMc202Touch => "lower_mc202_touch",
        ShellKeyOutcome::RaiseMc202Touch => "raise_mc202_touch",
        ShellKeyOutcome::AcceptCurrentGhostSuggestion => "accept_current_ghost_suggestion",
        ShellKeyOutcome::RejectCurrentGhostSuggestion => "reject_current_ghost_suggestion",
        ShellKeyOutcome::UndoLast => "undo_last",
        ShellKeyOutcome::Quit => "quit",
    }
}
