fn parse_args(args: impl IntoIterator<Item = String>) -> Result<AppLaunch, String> {
    let mut args = args.into_iter();
    let mut source_path = None;
    let mut session_path = None;
    let mut source_graph_path = None;
    let mut sidecar_script_path = Some(PathBuf::from(DEFAULT_SIDECAR_PATH));
    let mut analysis_seed = 19_u64;
    let mut saw_session_flag = false;
    let mut saw_sidecar_flag = false;
    let mut saw_seed_flag = false;
    let mut observer_path = None;
    let mut stem_package_local_ci_dry_run = false;
    let mut stem_package_local_ci_execute = false;
    let mut stem_package_local_ci_report = false;
    let mut daw_export_readiness_report = false;
    let mut daw_session_json_package_execute = false;
    let mut daw_session_json_package_evidence_apply = false;
    let mut daw_session_host_import_proof_apply = false;
    let mut daw_session_audible_output_proof_apply = false;
    let mut daw_session_writer_proof_execute = false;
    let mut daw_session_writer_proof_apply = false;
    let mut daw_session_writer_plan = false;
    let mut stem_package_destination_path = None;
    let mut daw_session_destination_path = None;
    let mut daw_session_host_import_proof_path = None;
    let mut daw_session_audible_output_proof_path = None;
    let mut claimed_stem_roles = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--stem-package-local-ci-dry-run" => stem_package_local_ci_dry_run = true,
            "--stem-package-local-ci-execute" => stem_package_local_ci_execute = true,
            "--stem-package-local-ci-report" => stem_package_local_ci_report = true,
            "--daw-export-readiness-report" => daw_export_readiness_report = true,
            "--daw-session-json-package-execute" => daw_session_json_package_execute = true,
            "--daw-session-json-package-evidence-apply" => {
                daw_session_json_package_evidence_apply = true;
            }
            "--daw-session-host-import-proof-apply" => {
                daw_session_host_import_proof_apply = true;
            }
            "--daw-session-audible-output-proof-apply" => {
                daw_session_audible_output_proof_apply = true;
            }
            "--daw-session-writer-proof-execute" => {
                daw_session_writer_proof_execute = true;
            }
            "--daw-session-writer-proof-apply" => {
                daw_session_writer_proof_apply = true;
            }
            "--daw-session-writer-plan" => daw_session_writer_plan = true,
            "--stem-package-destination" => {
                stem_package_destination_path =
                    Some(next_path(&mut args, "--stem-package-destination")?);
            }
            "--daw-session-destination" => {
                daw_session_destination_path =
                    Some(next_path(&mut args, "--daw-session-destination")?);
            }
            "--daw-session-host-import-proof" => {
                daw_session_host_import_proof_path =
                    Some(next_path(&mut args, "--daw-session-host-import-proof")?);
            }
            "--daw-session-audible-output-proof" => {
                daw_session_audible_output_proof_path = Some(next_path(
                    &mut args,
                    "--daw-session-audible-output-proof",
                )?);
            }
            "--stem-role" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --stem-role".to_string())?;
                claimed_stem_roles.push(parse_export_artifact_role(&value)?);
            }
            "--stem-roles" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --stem-roles".to_string())?;
                for role in value.split(',') {
                    let role = role.trim();
                    if !role.is_empty() {
                        claimed_stem_roles.push(parse_export_artifact_role(role)?);
                    }
                }
            }
            "--source" => source_path = Some(next_path(&mut args, "--source")?),
            "--session" => {
                saw_session_flag = true;
                session_path = Some(next_path(&mut args, "--session")?);
            }
            "--graph" => {
                source_graph_path = Some(next_path(&mut args, "--graph")?);
            }
            "--sidecar" => {
                saw_sidecar_flag = true;
                sidecar_script_path = Some(next_path(&mut args, "--sidecar")?);
            }
            "--observer" => observer_path = Some(next_path(&mut args, "--observer")?),
            "--seed" => {
                saw_seed_flag = true;
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

    let stem_package_mode_count = [
        stem_package_local_ci_dry_run,
        stem_package_local_ci_execute,
        stem_package_local_ci_report,
    ]
    .into_iter()
    .filter(|enabled| *enabled)
    .count();
    if stem_package_mode_count > 1 {
        return Err(
            "stem package local CI dry-run, execute, and report modes cannot be combined".into(),
        );
    }
    if daw_export_readiness_report && stem_package_mode_count > 0 {
        return Err("DAW export readiness report cannot be combined with stem package modes".into());
    }
    let daw_session_mode_count = [
        daw_session_json_package_execute,
        daw_session_json_package_evidence_apply,
        daw_session_host_import_proof_apply,
        daw_session_audible_output_proof_apply,
        daw_session_writer_proof_execute,
        daw_session_writer_proof_apply,
        daw_session_writer_plan,
    ]
    .into_iter()
    .filter(|enabled| *enabled)
    .count();
    if daw_session_mode_count > 1 {
        return Err(
            "DAW session JSON package execute, evidence apply, host import proof apply, audible output proof apply, writer proof execute/apply, and writer plan modes cannot be combined"
                .into(),
        );
    }
    if daw_session_mode_count > 0 && (stem_package_mode_count > 0 || daw_export_readiness_report) {
        return Err(
            "DAW session modes cannot be combined with stem package modes or DAW readiness report"
                .into(),
        );
    }

    if stem_package_local_ci_dry_run {
        if source_path.is_some()
            || session_path.is_some()
            || source_graph_path.is_some()
            || observer_path.is_some()
            || saw_sidecar_flag
            || saw_seed_flag
            || daw_session_destination_path.is_some()
            || daw_session_host_import_proof_path.is_some()
            || daw_session_audible_output_proof_path.is_some()
        {
            return Err(
                "stem package local CI dry-run cannot be combined with source/session/graph/sidecar/seed/observer/DAW destination launch arguments"
                    .into(),
            );
        }
        let destination_path = stem_package_destination_path.ok_or_else(|| {
            "stem package local CI dry-run requires --stem-package-destination <dir>".to_string()
        })?;
        if claimed_stem_roles.is_empty() {
            return Err("stem package local CI dry-run requires at least one --stem-role".into());
        }

        return Ok(AppLaunch {
            mode: LaunchMode::StemPackageLocalCiDryRun {
                destination_path,
                claimed_stem_roles,
            },
            observer_path: None,
        });
    }
    if stem_package_local_ci_execute {
        if source_path.is_some()
            || saw_sidecar_flag
            || saw_seed_flag
            || daw_session_destination_path.is_some()
            || daw_session_host_import_proof_path.is_some()
            || daw_session_audible_output_proof_path.is_some()
        {
            return Err(
                "stem package local CI execute cannot be combined with source/sidecar/seed/DAW destination launch arguments"
                    .into(),
            );
        }
        let session_path = session_path.filter(|_| saw_session_flag).ok_or_else(|| {
            "stem package local CI execute requires --session <session.json>".to_string()
        })?;
        let destination_path = stem_package_destination_path.ok_or_else(|| {
            "stem package local CI execute requires --stem-package-destination <dir>".to_string()
        })?;
        if claimed_stem_roles.is_empty() {
            return Err("stem package local CI execute requires at least one --stem-role".into());
        }

        return Ok(AppLaunch {
            mode: LaunchMode::StemPackageLocalCiExecute {
                session_path,
                source_graph_path,
                destination_path,
                claimed_stem_roles,
            },
            observer_path,
        });
    }
    if stem_package_local_ci_report {
        if source_path.is_some()
            || source_graph_path.is_some()
            || saw_sidecar_flag
            || saw_seed_flag
            || observer_path.is_some()
            || stem_package_destination_path.is_some()
            || daw_session_destination_path.is_some()
            || daw_session_host_import_proof_path.is_some()
            || daw_session_audible_output_proof_path.is_some()
            || !claimed_stem_roles.is_empty()
        {
            return Err(
                "stem package local CI report reads only an explicit session and cannot be combined with source/graph/observer/sidecar/seed/destination/role arguments"
                    .into(),
            );
        }
        let session_path = session_path.filter(|_| saw_session_flag).ok_or_else(|| {
            "stem package local CI report requires --session <session.json>".to_string()
        })?;

        return Ok(AppLaunch {
            mode: LaunchMode::StemPackageLocalCiReport { session_path },
            observer_path: None,
        });
    }
    if daw_export_readiness_report {
        if source_path.is_some()
            || source_graph_path.is_some()
            || saw_sidecar_flag
            || saw_seed_flag
            || observer_path.is_some()
            || stem_package_destination_path.is_some()
            || daw_session_destination_path.is_some()
            || daw_session_host_import_proof_path.is_some()
            || daw_session_audible_output_proof_path.is_some()
            || !claimed_stem_roles.is_empty()
        {
            return Err(
                "DAW export readiness report reads only an explicit session and cannot be combined with source/graph/observer/sidecar/seed/destination/role arguments"
                    .into(),
            );
        }
        let session_path = session_path.filter(|_| saw_session_flag).ok_or_else(|| {
            "DAW export readiness report requires --session <session.json>".to_string()
        })?;

        return Ok(AppLaunch {
            mode: LaunchMode::DawExportReadinessReport { session_path },
            observer_path: None,
        });
    }
    if let Some(launch) = parse_daw_session_mode_args(DawSessionModeArgs {
        json_package_execute: daw_session_json_package_execute,
        json_package_evidence_apply: daw_session_json_package_evidence_apply,
        host_import_proof_apply: daw_session_host_import_proof_apply,
        audible_output_proof_apply: daw_session_audible_output_proof_apply,
        writer_proof_execute: daw_session_writer_proof_execute,
        writer_proof_apply: daw_session_writer_proof_apply,
        writer_plan: daw_session_writer_plan,
        source_path_present: source_path.is_some(),
        source_graph_path_present: source_graph_path.is_some(),
        saw_session_flag,
        saw_sidecar_flag,
        saw_seed_flag,
        observer_path_present: observer_path.is_some(),
        stem_package_destination_path_present: stem_package_destination_path.is_some(),
        claimed_stem_roles_empty: claimed_stem_roles.is_empty(),
        session_path: session_path.as_ref(),
        destination_path: daw_session_destination_path.as_ref(),
        host_import_proof_path: daw_session_host_import_proof_path.as_ref(),
        audible_output_proof_path: daw_session_audible_output_proof_path.as_ref(),
    })? {
        return Ok(launch);
    }
    if stem_package_destination_path.is_some() || !claimed_stem_roles.is_empty() {
        return Err(
            "--stem-package-destination, --stem-role, and --stem-roles require --stem-package-local-ci-dry-run or --stem-package-local-ci-execute"
                .into(),
        );
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
