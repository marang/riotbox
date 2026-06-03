use riotbox_core::{
    daw_session_manifest::{
        DAW_SESSION_MANIFEST_SCHEMA_ID, DAW_SESSION_MANIFEST_SCHEMA_VERSION, DawSessionManifest,
        DawSessionPlannedArtifactRole as CoreDawSessionPlannedArtifactRole,
        DawSessionPlannedJsonIdentity,
    },
    daw_session_proof::{
        DAW_SESSION_PROOF_SCHEMA_ID, DAW_SESSION_PROOF_SCHEMA_VERSION, DawSessionProof,
    },
    session::{ExportArtifactLocation, ExportArtifactMediaType, ExportReceiptState},
};
use serde::Serialize;

use super::daw_session_writer_plan::{
    DawSessionPlannedArtifact, DawSessionPlannedArtifactRole, DawSessionWriterPlanBlocker,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DawSessionPayloadPreview {
    pub status: DawSessionPayloadPreviewStatus,
    pub ready: bool,
    pub blockers: Vec<DawSessionPayloadPreviewBlocker>,
    pub errors: Vec<String>,
    pub manifest: Option<DawSessionManifestPreview>,
    pub proof: Option<DawSessionProofPreview>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawSessionPayloadPreviewStatus {
    Blocked,
    Ready,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawSessionPayloadPreviewBlocker {
    NoDawSessionReceipt,
    MissingDestinationRoot,
    UnsupportedCommandBoundary,
    ArrangementPlacementBlocked,
    DawTempoMapBlocked,
    MissingArtifactIdentity,
    MissingLocalFiles,
    UnreadableLocalFiles,
    ManifestPayloadInvalid,
    ManifestHashUnavailable,
    ProofPayloadInvalid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DawSessionManifestPreview {
    pub schema_id: &'static str,
    pub schema_version: u32,
    pub planned_path: String,
    pub normalized_json_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DawSessionProofPreview {
    pub schema_id: &'static str,
    pub schema_version: u32,
    pub planned_path: String,
    pub manifest_sha256: String,
}

pub(super) fn payload_preview(
    receipt: Option<&ExportReceiptState>,
    planned_artifacts: &[DawSessionPlannedArtifact],
    readiness_blockers: &[DawSessionWriterPlanBlocker],
) -> DawSessionPayloadPreview {
    if !readiness_blockers.is_empty() {
        return blocked_payload_preview(
            readiness_blockers
                .iter()
                .copied()
                .map(payload_blocker_from_plan_blocker)
                .collect(),
            Vec::new(),
        );
    }

    let Some(receipt) = receipt else {
        return blocked_payload_preview(
            vec![DawSessionPayloadPreviewBlocker::NoDawSessionReceipt],
            Vec::new(),
        );
    };

    let planned_identities = planned_artifacts
        .iter()
        .map(planned_identity)
        .collect::<Vec<_>>();
    let manifest = match DawSessionManifest::from_receipt(receipt, planned_identities) {
        Ok(manifest) => manifest,
        Err(error) => {
            return blocked_payload_preview(
                vec![DawSessionPayloadPreviewBlocker::ManifestPayloadInvalid],
                vec![format!("{error:?}")],
            );
        }
    };
    let manifest_hash = match manifest.normalized_json_sha256() {
        Ok(hash) => hash,
        Err(error) => {
            return blocked_payload_preview(
                vec![DawSessionPayloadPreviewBlocker::ManifestHashUnavailable],
                vec![error.to_string()],
            );
        }
    };
    let proof = match DawSessionProof::from_manifest(&manifest) {
        Ok(proof) => proof,
        Err(error) => {
            return blocked_payload_preview(
                vec![DawSessionPayloadPreviewBlocker::ProofPayloadInvalid],
                vec![format!("{error:?}")],
            );
        }
    };

    DawSessionPayloadPreview {
        status: DawSessionPayloadPreviewStatus::Ready,
        ready: true,
        blockers: Vec::new(),
        errors: Vec::new(),
        manifest: Some(DawSessionManifestPreview {
            schema_id: DAW_SESSION_MANIFEST_SCHEMA_ID,
            schema_version: DAW_SESSION_MANIFEST_SCHEMA_VERSION,
            planned_path: planned_path(
                planned_artifacts,
                DawSessionPlannedArtifactRole::ArrangementManifest,
            ),
            normalized_json_sha256: manifest_hash,
        }),
        proof: Some(DawSessionProofPreview {
            schema_id: DAW_SESSION_PROOF_SCHEMA_ID,
            schema_version: DAW_SESSION_PROOF_SCHEMA_VERSION,
            planned_path: planned_path(
                planned_artifacts,
                DawSessionPlannedArtifactRole::DawSessionProof,
            ),
            manifest_sha256: proof.manifest_sha256,
        }),
    }
}

fn blocked_payload_preview(
    blockers: Vec<DawSessionPayloadPreviewBlocker>,
    errors: Vec<String>,
) -> DawSessionPayloadPreview {
    DawSessionPayloadPreview {
        status: DawSessionPayloadPreviewStatus::Blocked,
        ready: false,
        blockers,
        errors,
        manifest: None,
        proof: None,
    }
}

fn payload_blocker_from_plan_blocker(
    blocker: DawSessionWriterPlanBlocker,
) -> DawSessionPayloadPreviewBlocker {
    match blocker {
        DawSessionWriterPlanBlocker::NoDawSessionReceipt => {
            DawSessionPayloadPreviewBlocker::NoDawSessionReceipt
        }
        DawSessionWriterPlanBlocker::MissingDestinationRoot => {
            DawSessionPayloadPreviewBlocker::MissingDestinationRoot
        }
        DawSessionWriterPlanBlocker::UnsupportedCommandBoundary => {
            DawSessionPayloadPreviewBlocker::UnsupportedCommandBoundary
        }
        DawSessionWriterPlanBlocker::ArrangementPlacementBlocked => {
            DawSessionPayloadPreviewBlocker::ArrangementPlacementBlocked
        }
        DawSessionWriterPlanBlocker::DawTempoMapBlocked => {
            DawSessionPayloadPreviewBlocker::DawTempoMapBlocked
        }
        DawSessionWriterPlanBlocker::MissingArtifactIdentity => {
            DawSessionPayloadPreviewBlocker::MissingArtifactIdentity
        }
        DawSessionWriterPlanBlocker::MissingLocalFiles => {
            DawSessionPayloadPreviewBlocker::MissingLocalFiles
        }
        DawSessionWriterPlanBlocker::UnreadableLocalFiles => {
            DawSessionPayloadPreviewBlocker::UnreadableLocalFiles
        }
    }
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

fn planned_path(
    planned_artifacts: &[DawSessionPlannedArtifact],
    role: DawSessionPlannedArtifactRole,
) -> String {
    planned_artifacts
        .iter()
        .find(|artifact| artifact.role == role)
        .map(|artifact| artifact.path.clone())
        .unwrap_or_default()
}
