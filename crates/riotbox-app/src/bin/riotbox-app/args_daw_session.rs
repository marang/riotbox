struct DawSessionModeArgs<'a> {
    json_package_execute: bool,
    json_package_evidence_apply: bool,
    host_import_proof_apply: bool,
    audible_output_proof_apply: bool,
    writer_proof_execute: bool,
    writer_proof_apply: bool,
    writer_plan: bool,
    source_path_present: bool,
    source_graph_path_present: bool,
    saw_session_flag: bool,
    saw_sidecar_flag: bool,
    saw_seed_flag: bool,
    observer_path_present: bool,
    stem_package_destination_path_present: bool,
    claimed_stem_roles_empty: bool,
    session_path: Option<&'a PathBuf>,
    destination_path: Option<&'a PathBuf>,
    host_import_proof_path: Option<&'a PathBuf>,
    audible_output_proof_path: Option<&'a PathBuf>,
}

fn parse_daw_session_mode_args(args: DawSessionModeArgs<'_>) -> Result<Option<AppLaunch>, String> {
    if args.json_package_execute {
        reject_daw_session_destination_mode_conflicts(
            &args,
            "DAW session JSON package execute reads only an explicit session and destination and cannot be combined with source/graph/observer/sidecar/seed/stem arguments",
            ProofArgPolicy::NoProofArgs,
        )?;
        return Ok(Some(AppLaunch {
            mode: LaunchMode::DawSessionJsonPackageExecute {
                session_path: required_daw_session(args.session_path, args.saw_session_flag, "DAW session JSON package execute requires --session <session.json>")?,
                destination_path: required_daw_destination(
                    args.destination_path,
                    "DAW session JSON package execute requires --daw-session-destination <dir>",
                )?,
            },
            observer_path: None,
        }));
    }

    if args.json_package_evidence_apply {
        reject_daw_session_destination_mode_conflicts(
            &args,
            "DAW session JSON package evidence apply reads only an explicit session and destination and cannot be combined with source/graph/observer/sidecar/seed/stem arguments",
            ProofArgPolicy::NoProofArgs,
        )?;
        return Ok(Some(AppLaunch {
            mode: LaunchMode::DawSessionJsonPackageEvidenceApply {
                session_path: required_daw_session(args.session_path, args.saw_session_flag, "DAW session JSON package evidence apply requires --session <session.json>")?,
                destination_path: required_daw_destination(
                    args.destination_path,
                    "DAW session JSON package evidence apply requires --daw-session-destination <dir>",
                )?,
            },
            observer_path: None,
        }));
    }

    if args.host_import_proof_apply {
        reject_daw_session_proof_mode_conflicts(
            &args,
            "DAW session host import proof apply reads only an explicit session and proof file and cannot be combined with source/graph/observer/sidecar/seed/destination/stem arguments",
            ProofKind::HostImport,
        )?;
        return Ok(Some(AppLaunch {
            mode: LaunchMode::DawSessionHostImportProofApply {
                session_path: required_daw_session(args.session_path, args.saw_session_flag, "DAW session host import proof apply requires --session <session.json>")?,
                proof_path: required_proof_path(
                    args.host_import_proof_path,
                    "DAW session host import proof apply requires --daw-session-host-import-proof <proof.json>",
                )?,
            },
            observer_path: None,
        }));
    }

    if args.audible_output_proof_apply {
        reject_daw_session_proof_mode_conflicts(
            &args,
            "DAW session audible output proof apply reads only an explicit session and proof file and cannot be combined with source/graph/observer/sidecar/seed/destination/stem arguments",
            ProofKind::AudibleOutput,
        )?;
        return Ok(Some(AppLaunch {
            mode: LaunchMode::DawSessionAudibleOutputProofApply {
                session_path: required_daw_session(args.session_path, args.saw_session_flag, "DAW session audible output proof apply requires --session <session.json>")?,
                proof_path: required_proof_path(
                    args.audible_output_proof_path,
                    "DAW session audible output proof apply requires --daw-session-audible-output-proof <proof.json>",
                )?,
            },
            observer_path: None,
        }));
    }

    if args.writer_proof_execute {
        reject_daw_session_destination_mode_conflicts(
            &args,
            "DAW session writer proof execute reads only an explicit session and destination and cannot be combined with source/graph/observer/sidecar/seed/stem arguments",
            ProofArgPolicy::NoProofArgs,
        )?;
        return Ok(Some(AppLaunch {
            mode: LaunchMode::DawSessionWriterProofExecute {
                session_path: required_daw_session(args.session_path, args.saw_session_flag, "DAW session writer proof execute requires --session <session.json>")?,
                destination_path: required_daw_destination(
                    args.destination_path,
                    "DAW session writer proof execute requires --daw-session-destination <dir>",
                )?,
            },
            observer_path: None,
        }));
    }

    if args.writer_proof_apply {
        reject_daw_session_destination_mode_conflicts(
            &args,
            "DAW session writer proof apply reads only an explicit session and destination and cannot be combined with source/graph/observer/sidecar/seed/stem arguments",
            ProofArgPolicy::NoProofArgs,
        )?;
        return Ok(Some(AppLaunch {
            mode: LaunchMode::DawSessionWriterProofApply {
                session_path: required_daw_session(args.session_path, args.saw_session_flag, "DAW session writer proof apply requires --session <session.json>")?,
                destination_path: required_daw_destination(
                    args.destination_path,
                    "DAW session writer proof apply requires --daw-session-destination <dir>",
                )?,
            },
            observer_path: None,
        }));
    }

    if args.writer_plan {
        reject_daw_session_destination_mode_conflicts(
            &args,
            "DAW session writer plan reads only an explicit session and destination and cannot be combined with source/graph/observer/sidecar/seed/stem arguments",
            ProofArgPolicy::NoProofArgs,
        )?;
        return Ok(Some(AppLaunch {
            mode: LaunchMode::DawSessionWriterPlan {
                session_path: required_daw_session(args.session_path, args.saw_session_flag, "DAW session writer plan requires --session <session.json>")?,
                destination_path: required_daw_destination(
                    args.destination_path,
                    "DAW session writer plan requires --daw-session-destination <dir>",
                )?,
            },
            observer_path: None,
        }));
    }

    reject_standalone_daw_session_args(&args)?;
    Ok(None)
}

#[derive(Copy, Clone)]
enum ProofArgPolicy {
    NoProofArgs,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum ProofKind {
    HostImport,
    AudibleOutput,
}

fn reject_daw_session_destination_mode_conflicts(
    args: &DawSessionModeArgs<'_>,
    message: &str,
    _: ProofArgPolicy,
) -> Result<(), String> {
    if args.source_path_present
        || args.source_graph_path_present
        || args.saw_sidecar_flag
        || args.saw_seed_flag
        || args.observer_path_present
        || args.stem_package_destination_path_present
        || args.host_import_proof_path.is_some()
        || args.audible_output_proof_path.is_some()
        || !args.claimed_stem_roles_empty
    {
        return Err(message.into());
    }

    Ok(())
}

fn reject_daw_session_proof_mode_conflicts(
    args: &DawSessionModeArgs<'_>,
    message: &str,
    proof_kind: ProofKind,
) -> Result<(), String> {
    let other_proof_present = match proof_kind {
        ProofKind::HostImport => args.audible_output_proof_path.is_some(),
        ProofKind::AudibleOutput => args.host_import_proof_path.is_some(),
    };
    if args.source_path_present
        || args.source_graph_path_present
        || args.saw_sidecar_flag
        || args.saw_seed_flag
        || args.observer_path_present
        || args.stem_package_destination_path_present
        || args.destination_path.is_some()
        || other_proof_present
        || !args.claimed_stem_roles_empty
    {
        return Err(message.into());
    }

    Ok(())
}

fn reject_standalone_daw_session_args(args: &DawSessionModeArgs<'_>) -> Result<(), String> {
    if args.destination_path.is_some() {
        return Err(
            "--daw-session-destination requires --daw-session-writer-plan, --daw-session-json-package-execute, --daw-session-json-package-evidence-apply, --daw-session-writer-proof-execute, or --daw-session-writer-proof-apply"
                .into(),
        );
    }
    if args.host_import_proof_path.is_some() {
        return Err(
            "--daw-session-host-import-proof requires --daw-session-host-import-proof-apply"
                .into(),
        );
    }
    if args.audible_output_proof_path.is_some() {
        return Err(
            "--daw-session-audible-output-proof requires --daw-session-audible-output-proof-apply"
                .into(),
        );
    }

    Ok(())
}

fn required_daw_session(
    session_path: Option<&PathBuf>,
    saw_session_flag: bool,
    message: &str,
) -> Result<PathBuf, String> {
    session_path
        .filter(|_| saw_session_flag)
        .cloned()
        .ok_or_else(|| message.to_string())
}

fn required_daw_destination(
    destination_path: Option<&PathBuf>,
    message: &str,
) -> Result<PathBuf, String> {
    destination_path.cloned().ok_or_else(|| message.to_string())
}

fn required_proof_path(proof_path: Option<&PathBuf>, message: &str) -> Result<PathBuf, String> {
    proof_path.cloned().ok_or_else(|| message.to_string())
}
