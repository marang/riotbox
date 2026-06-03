use serde::{Deserialize, Serialize};

use crate::{
    daw_session_manifest::{
        DawSessionManifest, DawSessionManifestError, DawSessionPlannedJsonIdentity,
        DawSessionSourceArtifactIdentity, validate_daw_session_planned_artifacts,
        validate_daw_session_source_artifacts,
    },
    export_readiness::{ExportScope, ProductExportBoundary, ProductExportRole},
    ids::{ActionId, ExportReceiptId},
    session::{ExportArrangementPlacementRef, ExportDawTempoMapRef},
};

pub const DAW_SESSION_PROOF_SCHEMA_ID: &str = "riotbox.daw_session_proof";
pub const DAW_SESSION_PROOF_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DawSessionProof {
    pub schema_id: String,
    pub schema_version: u32,
    pub package_id: String,
    pub export_scope: ExportScope,
    #[serde(default = "default_daw_session_proof_export_role")]
    pub export_role: ProductExportRole,
    #[serde(default = "default_daw_session_proof_boundary")]
    pub export_boundary: ProductExportBoundary,
    pub receipt_id: ExportReceiptId,
    pub created_by_action: ActionId,
    pub manifest_sha256: String,
    pub placement_refs: Vec<ExportArrangementPlacementRef>,
    pub tempo_map_ref: ExportDawTempoMapRef,
    pub source_artifacts: Vec<DawSessionSourceArtifactIdentity>,
    pub planned_artifacts: Vec<DawSessionPlannedJsonIdentity>,
}

impl DawSessionProof {
    pub fn from_manifest(manifest: &DawSessionManifest) -> Result<Self, DawSessionProofError> {
        let manifest_sha256 = manifest
            .normalized_json_sha256()
            .map_err(|_| DawSessionProofError::ManifestSerialization)?;
        Self::new(DawSessionProofInput {
            package_id: manifest.package_id.clone(),
            export_role: manifest.export_role,
            export_boundary: manifest.export_boundary,
            receipt_id: manifest.receipt_id.clone(),
            created_by_action: manifest.created_by_action,
            manifest_sha256,
            placement_refs: manifest.placement_refs.clone(),
            tempo_map_ref: manifest.tempo_map_ref.clone(),
            source_artifacts: manifest.source_artifacts.clone(),
            planned_artifacts: manifest.planned_artifacts.clone(),
        })
    }

    pub fn new(input: DawSessionProofInput) -> Result<Self, DawSessionProofError> {
        let package_id = input.package_id;
        if package_id.trim().is_empty() {
            return Err(DawSessionProofError::BlankPackageId);
        }
        let manifest_sha256 = input.manifest_sha256;
        if manifest_sha256.trim().is_empty() {
            return Err(DawSessionProofError::BlankManifestSha256);
        }
        if input.export_role != ProductExportRole::ArrangementManifest {
            return Err(DawSessionProofError::UnexpectedExportRole {
                expected: ProductExportRole::ArrangementManifest,
                actual: input.export_role,
            });
        }
        if input.export_boundary != ProductExportBoundary::ArrangementDawPlacementContractV1 {
            return Err(DawSessionProofError::UnexpectedExportBoundary {
                expected: ProductExportBoundary::ArrangementDawPlacementContractV1,
                actual: input.export_boundary,
            });
        }
        if input.placement_refs.is_empty() {
            return Err(DawSessionProofError::MissingPlacementRefs);
        }
        validate_daw_session_source_artifacts(&input.source_artifacts)
            .map_err(DawSessionProofError::InvalidSourceArtifacts)?;
        validate_daw_session_planned_artifacts(&input.planned_artifacts)
            .map_err(DawSessionProofError::InvalidPlannedArtifacts)?;

        Ok(Self {
            schema_id: DAW_SESSION_PROOF_SCHEMA_ID.into(),
            schema_version: DAW_SESSION_PROOF_SCHEMA_VERSION,
            package_id,
            export_scope: ExportScope::DawSession,
            export_role: input.export_role,
            export_boundary: input.export_boundary,
            receipt_id: input.receipt_id,
            created_by_action: input.created_by_action,
            manifest_sha256,
            placement_refs: input.placement_refs,
            tempo_map_ref: input.tempo_map_ref,
            source_artifacts: input.source_artifacts,
            planned_artifacts: input.planned_artifacts,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DawSessionProofInput {
    pub package_id: String,
    pub export_role: ProductExportRole,
    pub export_boundary: ProductExportBoundary,
    pub receipt_id: ExportReceiptId,
    pub created_by_action: ActionId,
    pub manifest_sha256: String,
    pub placement_refs: Vec<ExportArrangementPlacementRef>,
    pub tempo_map_ref: ExportDawTempoMapRef,
    pub source_artifacts: Vec<DawSessionSourceArtifactIdentity>,
    pub planned_artifacts: Vec<DawSessionPlannedJsonIdentity>,
}

#[must_use]
pub const fn default_daw_session_proof_export_role() -> ProductExportRole {
    ProductExportRole::ArrangementManifest
}

#[must_use]
pub const fn default_daw_session_proof_boundary() -> ProductExportBoundary {
    ProductExportBoundary::ArrangementDawPlacementContractV1
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DawSessionProofError {
    ManifestSerialization,
    BlankPackageId,
    BlankManifestSha256,
    UnexpectedExportRole {
        expected: ProductExportRole,
        actual: ProductExportRole,
    },
    UnexpectedExportBoundary {
        expected: ProductExportBoundary,
        actual: ProductExportBoundary,
    },
    MissingPlacementRefs,
    InvalidSourceArtifacts(DawSessionManifestError),
    InvalidPlannedArtifacts(DawSessionManifestError),
}

#[cfg(test)]
#[path = "daw_session_proof_tests.rs"]
mod daw_session_proof_tests;
