use riotbox_core::{
    export_readiness::ProductExportDestinationKind,
    ids::ActionId,
    session::{ExportArtifactLocation, ExportArtifactMediaType},
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
