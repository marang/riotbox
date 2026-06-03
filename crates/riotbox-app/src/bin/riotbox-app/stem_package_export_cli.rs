use riotbox_core::{
    export_readiness::ProductExportDestinationKind,
    ids::ActionId,
    session::{
        ExportArtifactLocation, ExportArtifactMediaType, ExportArtifactSetEntry, ExportReceiptState,
    },
    stem_package_writer::{
        SUPPORTED_LOCAL_CI_PACKAGE_STEM_ROLES, StemPackageLocalWriterBoundary,
        StemPackageLocalWriterRequest, plan_stem_package_local_ci_package,
    },
};

fn run_stem_package_local_ci_dry_run(
    launch: &AppLaunch,
) -> Result<(), Box<dyn std::error::Error>> {
    let summary = stem_package_local_ci_dry_run_summary(launch)?;
    serde_json::to_writer_pretty(std::io::stdout(), &summary)?;
    println!();
    Ok(())
}

fn run_stem_package_local_ci_execute(
    launch: &AppLaunch,
    raw_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    write_stem_package_local_ci_execute_output(launch, raw_args, &mut stdout)
}

fn write_stem_package_local_ci_execute_output(
    launch: &AppLaunch,
    raw_args: &[String],
    output: &mut impl std::io::Write,
) -> Result<(), Box<dyn std::error::Error>> {
    let (summary, shell) = stem_package_local_ci_execute_summary(launch)?;
    if let Some(path) = launch.observer_path.as_deref() {
        let mut observer = UserSessionObserver::open(path)?;
        observer.record(json!({
            "event": "stem_package_local_ci_execute",
            "schema": "riotbox.user_session_observer.v1",
            "timestamp_ms": timestamp_now(),
            "opt_in": true,
            "capture_context": "non_interactive_cli",
            "raw_audio_recording": false,
            "realtime_callback_io": false,
            "argv": raw_args,
            "launch": launch_summary(launch),
            "summary": summary.clone(),
            "snapshot": observer_snapshot(&shell),
        }))?;
    }
    serde_json::to_writer_pretty(&mut *output, &summary)?;
    writeln!(output)?;
    Ok(())
}

fn stem_package_local_ci_dry_run_summary(launch: &AppLaunch) -> Result<Value, String> {
    let LaunchMode::StemPackageLocalCiDryRun {
        destination_path,
        claimed_stem_roles,
    } = &launch.mode
    else {
        return Err("not a stem package local CI dry-run launch".into());
    };

    let request = StemPackageLocalWriterRequest {
        created_by_action: ActionId(0),
        boundary: StemPackageLocalWriterBoundary::LocalCiPackageV1,
        destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
        destination_root: destination_path.to_string_lossy().into_owned(),
        claimed_stem_roles: claimed_stem_roles.clone(),
    };
    let unsupported_claimed_roles = claimed_stem_roles
        .iter()
        .copied()
        .filter(|role| !SUPPORTED_LOCAL_CI_PACKAGE_STEM_ROLES.contains(role))
        .map(export_artifact_role_label)
        .collect::<Vec<_>>();

    match plan_stem_package_local_ci_package(request) {
        Ok(plan) => Ok(json!({
            "mode": "stem_package_local_ci_dry_run",
            "status": "ready",
            "ready": true,
            "writes_files": false,
            "boundary": plan.boundary_id,
            "destination_path": plan.destination_root,
            "claimed_stem_roles": plan.claimed_stem_roles
                .iter()
                .copied()
                .map(export_artifact_role_label)
                .collect::<Vec<_>>(),
            "supported_roles": SUPPORTED_LOCAL_CI_PACKAGE_STEM_ROLES
                .iter()
                .copied()
                .map(export_artifact_role_label)
                .collect::<Vec<_>>(),
            "unsupported_claimed_roles": unsupported_claimed_roles,
            "readiness_blockers": [],
            "planned_artifacts": plan.artifacts
                .iter()
                .map(planned_stem_package_artifact_summary)
                .collect::<Vec<_>>(),
        })),
        Err(error) => Ok(json!({
            "mode": "stem_package_local_ci_dry_run",
            "status": "blocked",
            "ready": false,
            "writes_files": false,
            "boundary": StemPackageLocalWriterBoundary::LocalCiPackageV1.as_str(),
            "destination_path": destination_path,
            "claimed_stem_roles": claimed_stem_roles
                .iter()
                .copied()
                .map(export_artifact_role_label)
                .collect::<Vec<_>>(),
            "supported_roles": SUPPORTED_LOCAL_CI_PACKAGE_STEM_ROLES
                .iter()
                .copied()
                .map(export_artifact_role_label)
                .collect::<Vec<_>>(),
            "unsupported_claimed_roles": unsupported_claimed_roles,
            "readiness_blockers": [format!("{error:?}")],
            "planned_artifacts": [],
        })),
    }
}

fn stem_package_local_ci_execute_summary(
    launch: &AppLaunch,
) -> Result<(Value, JamShellState), Box<dyn std::error::Error>> {
    let LaunchMode::StemPackageLocalCiExecute {
        session_path,
        source_graph_path,
        destination_path,
        claimed_stem_roles,
    } = &launch.mode
    else {
        return Err("not a stem package local CI execute launch".into());
    };

    let mut state = JamAppState::from_json_files(session_path, source_graph_path.as_deref())?;
    let unsupported_claimed_roles = unsupported_claimed_roles(claimed_stem_roles);
    let summary = match state.commit_stem_package_export_local_ci_package(
        destination_path,
        timestamp_now(),
        claimed_stem_roles.clone(),
    ) {
        Ok(receipt) => {
            state.save()?;
            stem_package_local_ci_execute_ready_summary(
                session_path,
                destination_path,
                claimed_stem_roles,
                &unsupported_claimed_roles,
                &receipt,
            )
        }
        Err(error) => stem_package_local_ci_execute_blocked_summary(
            session_path,
            destination_path,
            claimed_stem_roles,
            &unsupported_claimed_roles,
            &error,
        ),
    };
    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    Ok((summary, shell))
}

fn stem_package_local_ci_execute_ready_summary(
    session_path: &std::path::Path,
    destination_path: &std::path::Path,
    claimed_stem_roles: &[ExportArtifactRole],
    unsupported_claimed_roles: &[&'static str],
    receipt: &ExportReceiptState,
) -> Value {
    json!({
        "mode": "stem_package_local_ci_execute",
        "status": "ready",
        "ready": true,
        "writes_files": true,
        "boundary": StemPackageLocalWriterBoundary::LocalCiPackageV1.as_str(),
        "session_path": session_path,
        "destination_path": destination_path,
        "claimed_stem_roles": claimed_stem_roles
            .iter()
            .copied()
            .map(export_artifact_role_label)
            .collect::<Vec<_>>(),
        "supported_roles": supported_stem_role_labels(),
        "unsupported_claimed_roles": unsupported_claimed_roles,
        "readiness_blockers": [],
        "receipt": stem_package_receipt_summary(receipt),
    })
}

fn stem_package_local_ci_execute_blocked_summary(
    session_path: &std::path::Path,
    destination_path: &std::path::Path,
    claimed_stem_roles: &[ExportArtifactRole],
    unsupported_claimed_roles: &[&'static str],
    error: &JamAppError,
) -> Value {
    json!({
        "mode": "stem_package_local_ci_execute",
        "status": "blocked",
        "ready": false,
        "writes_files": false,
        "boundary": StemPackageLocalWriterBoundary::LocalCiPackageV1.as_str(),
        "session_path": session_path,
        "destination_path": destination_path,
        "claimed_stem_roles": claimed_stem_roles
            .iter()
            .copied()
            .map(export_artifact_role_label)
            .collect::<Vec<_>>(),
        "supported_roles": supported_stem_role_labels(),
        "unsupported_claimed_roles": unsupported_claimed_roles,
        "readiness_blockers": [error.to_string()],
        "receipt": null,
    })
}

fn stem_package_receipt_summary(receipt: &ExportReceiptState) -> Value {
    json!({
        "receipt_id": receipt.receipt_id,
        "created_by_action": receipt.created_by_action,
        "export_scope": receipt.export_scope.as_str(),
        "artifact_count": receipt.artifact_set.len(),
        "readiness_status": receipt.readiness_status,
        "unsupported_scopes": receipt.unsupported_scopes,
        "artifacts": receipt.artifact_set
            .iter()
            .map(export_receipt_artifact_summary)
            .collect::<Vec<_>>(),
    })
}

fn export_receipt_artifact_summary(artifact: &ExportArtifactSetEntry) -> Value {
    json!({
        "role": export_artifact_role_label(artifact.role),
        "location": artifact.location_identity(),
        "location_kind": match &artifact.location {
            ExportArtifactLocation::LocalPath { .. } => "local_path",
            ExportArtifactLocation::Uri { .. } => "uri",
        },
        "media_type": export_artifact_media_type_label(artifact.media_type),
        "sha256": artifact.sha256,
    })
}

fn planned_stem_package_artifact_summary(
    artifact: &riotbox_core::stem_package_writer::StemPackagePlannedArtifactIdentity,
) -> Value {
    json!({
        "role": export_artifact_role_label(artifact.role),
        "location": artifact.location_identity(),
        "location_kind": match artifact.location {
            ExportArtifactLocation::LocalPath { .. } => "local_path",
            ExportArtifactLocation::Uri { .. } => "uri",
        },
        "media_type": export_artifact_media_type_label(artifact.media_type),
    })
}

fn unsupported_claimed_roles(claimed_stem_roles: &[ExportArtifactRole]) -> Vec<&'static str> {
    claimed_stem_roles
        .iter()
        .copied()
        .filter(|role| !SUPPORTED_LOCAL_CI_PACKAGE_STEM_ROLES.contains(role))
        .map(export_artifact_role_label)
        .collect::<Vec<_>>()
}

fn supported_stem_role_labels() -> Vec<&'static str> {
    SUPPORTED_LOCAL_CI_PACKAGE_STEM_ROLES
        .iter()
        .copied()
        .map(export_artifact_role_label)
        .collect::<Vec<_>>()
}

fn export_artifact_role_label(role: ExportArtifactRole) -> &'static str {
    match role {
        ExportArtifactRole::FullGridMix => "full_grid_mix",
        ExportArtifactRole::StemDrums => "stem_drums",
        ExportArtifactRole::StemBass => "stem_bass",
        ExportArtifactRole::StemMusic => "stem_music",
        ExportArtifactRole::StemVocals => "stem_vocals",
        ExportArtifactRole::ProductExportProof => "product_export_proof",
        ExportArtifactRole::ExportManifest => "export_manifest",
    }
}

fn export_artifact_media_type_label(media_type: ExportArtifactMediaType) -> &'static str {
    match media_type {
        ExportArtifactMediaType::AudioWav => "audio_wav",
        ExportArtifactMediaType::Json => "json",
    }
}
