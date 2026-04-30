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
            "committed": committed.iter().map(compact_commit).collect::<Vec<_>>(),
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
