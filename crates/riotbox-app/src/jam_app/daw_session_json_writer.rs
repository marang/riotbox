use std::{
    fs,
    path::{Path, PathBuf},
};

use riotbox_core::{
    daw_session_manifest::{
        DawSessionManifest, DawSessionPlannedArtifactRole as CoreDawSessionPlannedArtifactRole,
        DawSessionPlannedJsonIdentity,
    },
    daw_session_proof::DawSessionProof,
    daw_session_tempo_map::DawSessionTempoMap,
    export_readiness::ExportScope,
    session::{ExportArtifactLocation, ExportArtifactMediaType, ExportReceiptState, SessionFile},
};

use super::{
    JamAppError,
    daw_session_writer_plan::{
        DAW_SESSION_ARRANGEMENT_MANIFEST_FILE, DAW_SESSION_PACKAGE_DIR, DAW_SESSION_PROOF_FILE,
        DAW_SESSION_TEMPO_MAP_FILE, DawSessionPlannedArtifact, DawSessionPlannedArtifactRole,
        daw_session_writer_plan,
    },
    product_export::sha256_file,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrittenDawSessionJsonPackage {
    pub package_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub tempo_map_path: PathBuf,
    pub proof_path: PathBuf,
    pub manifest_sha256: String,
    pub tempo_map_sha256: String,
    pub proof_sha256: String,
    pub proof_manifest_sha256: String,
}

pub fn write_daw_session_json_package(
    session: &SessionFile,
    session_base_dir: Option<&Path>,
    destination_root: impl AsRef<Path>,
) -> Result<WrittenDawSessionJsonPackage, JamAppError> {
    let destination_root = destination_root.as_ref();
    let plan = daw_session_writer_plan(session, session_base_dir, destination_root);
    if !plan.ready_for_writer || !plan.payload_preview.ready {
        return Err(JamAppError::InvalidSession(format!(
            "DAW session JSON writer blocked: plan={:?} payload={:?}",
            plan.readiness_blockers, plan.payload_preview.blockers
        )));
    }

    let receipt = latest_daw_session_receipt(session).ok_or_else(|| {
        JamAppError::InvalidSession("DAW session JSON writer requires a DAW session receipt".into())
    })?;
    let final_package_dir = destination_root.join(DAW_SESSION_PACKAGE_DIR);
    if final_package_dir.exists() {
        return Err(JamAppError::InvalidSession(format!(
            "DAW session package destination already exists: {}",
            final_package_dir.display()
        )));
    }

    let staging_root = destination_root.join(format!(
        ".daw_session_staging_{}",
        receipt.created_by_action.0
    ));
    if staging_root.exists() {
        return Err(JamAppError::InvalidSession(format!(
            "DAW session staging destination already exists: {}",
            staging_root.display()
        )));
    }
    let staging_package_dir = staging_root.join(DAW_SESSION_PACKAGE_DIR);
    fs::create_dir_all(&staging_package_dir)?;

    let planned_identities = plan
        .planned_artifacts
        .iter()
        .map(planned_identity)
        .collect::<Vec<_>>();
    let manifest = DawSessionManifest::from_receipt(receipt, planned_identities)
        .map_err(|error| JamAppError::InvalidSession(format!("{error:?}")))?;
    let tempo_map = DawSessionTempoMap::from_receipt(receipt)
        .map_err(|error| JamAppError::InvalidSession(format!("{error:?}")))?;
    let proof = DawSessionProof::from_manifest(&manifest)
        .map_err(|error| JamAppError::InvalidSession(format!("{error:?}")))?;
    let manifest_hash = manifest.normalized_json_sha256()?;
    if proof.manifest_sha256 != manifest_hash {
        return Err(JamAppError::InvalidSession(
            "DAW session proof manifest hash does not match manifest payload".into(),
        ));
    }

    let staging_manifest_path = staging_package_dir.join(DAW_SESSION_ARRANGEMENT_MANIFEST_FILE);
    let staging_tempo_map_path = staging_package_dir.join(DAW_SESSION_TEMPO_MAP_FILE);
    let staging_proof_path = staging_package_dir.join(DAW_SESSION_PROOF_FILE);
    fs::write(&staging_manifest_path, manifest.normalized_json_bytes()?)?;
    fs::write(&staging_tempo_map_path, tempo_map.normalized_json_bytes()?)?;
    fs::write(&staging_proof_path, serde_json::to_vec_pretty(&proof)?)?;

    fs::rename(&staging_package_dir, &final_package_dir)?;
    fs::remove_dir_all(&staging_root)?;

    let final_manifest_path = final_package_dir.join(DAW_SESSION_ARRANGEMENT_MANIFEST_FILE);
    let final_tempo_map_path = final_package_dir.join(DAW_SESSION_TEMPO_MAP_FILE);
    let final_proof_path = final_package_dir.join(DAW_SESSION_PROOF_FILE);
    let manifest_sha256 = sha256_file(&final_manifest_path)?;
    let tempo_map_sha256 = sha256_file(&final_tempo_map_path)?;
    let proof_sha256 = sha256_file(&final_proof_path)?;
    if manifest_sha256 != manifest_hash {
        return Err(JamAppError::InvalidSession(
            "written DAW session manifest hash changed after promotion".into(),
        ));
    }

    Ok(WrittenDawSessionJsonPackage {
        package_dir: final_package_dir,
        manifest_path: final_manifest_path,
        tempo_map_path: final_tempo_map_path,
        proof_path: final_proof_path,
        manifest_sha256,
        tempo_map_sha256,
        proof_sha256,
        proof_manifest_sha256: proof.manifest_sha256,
    })
}

fn latest_daw_session_receipt(session: &SessionFile) -> Option<&ExportReceiptState> {
    session
        .export_receipts
        .iter()
        .rev()
        .find(|receipt| receipt.export_scope == ExportScope::DawSession)
}

fn planned_identity(planned: &DawSessionPlannedArtifact) -> DawSessionPlannedJsonIdentity {
    DawSessionPlannedJsonIdentity {
        role: match planned.role {
            DawSessionPlannedArtifactRole::ArrangementManifest => {
                CoreDawSessionPlannedArtifactRole::ArrangementManifest
            }
            DawSessionPlannedArtifactRole::TempoMap => CoreDawSessionPlannedArtifactRole::TempoMap,
            DawSessionPlannedArtifactRole::DawSessionProof => {
                CoreDawSessionPlannedArtifactRole::DawSessionProof
            }
        },
        location: ExportArtifactLocation::LocalPath {
            path: planned.path.clone(),
        },
        media_type: ExportArtifactMediaType::Json,
    }
}
