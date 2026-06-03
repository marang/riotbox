use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    export_readiness::{
        ExportScope, ProductExportBoundary, ProductExportRole, UnsupportedExportScope,
    },
    ids::{ActionId, ExportReceiptId},
    session::{
        ArrangementExportPlacementReadinessBlocker, DawTempoMapReadinessBlocker,
        ExportArrangementPlacementRef, ExportArtifactLocation, ExportArtifactMediaType,
        ExportArtifactRole, ExportArtifactSetEntry, ExportDawTempoMapRef, ExportReceiptState,
    },
};

pub const DAW_SESSION_MANIFEST_SCHEMA_ID: &str = "riotbox.daw_session_manifest";
pub const DAW_SESSION_MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DawSessionManifest {
    pub schema_id: String,
    pub schema_version: u32,
    pub package_id: String,
    pub export_scope: ExportScope,
    #[serde(default = "default_daw_session_manifest_export_role")]
    pub export_role: ProductExportRole,
    #[serde(default = "default_daw_session_manifest_boundary")]
    pub export_boundary: ProductExportBoundary,
    pub receipt_id: ExportReceiptId,
    pub created_by_action: ActionId,
    pub placement_refs: Vec<ExportArrangementPlacementRef>,
    pub tempo_map_ref: ExportDawTempoMapRef,
    pub source_artifacts: Vec<DawSessionSourceArtifactIdentity>,
    pub planned_artifacts: Vec<DawSessionPlannedJsonIdentity>,
}

impl DawSessionManifest {
    pub fn new(input: DawSessionManifestInput) -> Result<Self, DawSessionManifestError> {
        let package_id = input.package_id;
        if package_id.trim().is_empty() {
            return Err(DawSessionManifestError::BlankPackageId);
        }
        if input.export_role != ProductExportRole::ArrangementManifest {
            return Err(DawSessionManifestError::UnexpectedExportRole {
                expected: ProductExportRole::ArrangementManifest,
                actual: input.export_role,
            });
        }
        if input.export_boundary != ProductExportBoundary::ArrangementDawPlacementContractV1 {
            return Err(DawSessionManifestError::UnexpectedExportBoundary {
                expected: ProductExportBoundary::ArrangementDawPlacementContractV1,
                actual: input.export_boundary,
            });
        }
        if input.placement_refs.is_empty() {
            return Err(DawSessionManifestError::MissingPlacementRefs);
        }

        validate_daw_session_source_artifacts(&input.source_artifacts)?;
        validate_daw_session_planned_artifacts(&input.planned_artifacts)?;

        Ok(Self {
            schema_id: DAW_SESSION_MANIFEST_SCHEMA_ID.into(),
            schema_version: DAW_SESSION_MANIFEST_SCHEMA_VERSION,
            package_id,
            export_scope: ExportScope::DawSession,
            export_role: input.export_role,
            export_boundary: input.export_boundary,
            receipt_id: input.receipt_id,
            created_by_action: input.created_by_action,
            placement_refs: input.placement_refs,
            tempo_map_ref: input.tempo_map_ref,
            source_artifacts: input.source_artifacts,
            planned_artifacts: input.planned_artifacts,
        })
    }

    pub fn from_receipt(
        receipt: &ExportReceiptState,
        planned_artifacts: Vec<DawSessionPlannedJsonIdentity>,
    ) -> Result<Self, DawSessionManifestBuildError> {
        if receipt.export_scope != ExportScope::DawSession {
            return Err(DawSessionManifestBuildError::NotDawSessionScope {
                export_scope: receipt.export_scope,
            });
        }
        if receipt.export_boundary != ProductExportBoundary::ArrangementDawPlacementContractV1 {
            return Err(DawSessionManifestBuildError::NotDawSessionBoundary {
                export_boundary: receipt.export_boundary,
            });
        }
        if receipt
            .unsupported_scopes
            .contains(&UnsupportedExportScope::DawExport)
        {
            return Err(DawSessionManifestBuildError::UnsupportedDawExportFlagPresent);
        }

        let placement_report = receipt.arrangement_export_placement_report();
        if !placement_report.ready() {
            return Err(DawSessionManifestBuildError::ArrangementPlacementBlocked {
                blockers: placement_report.blockers,
            });
        }
        let tempo_map_report = receipt.daw_tempo_map_report();
        if !tempo_map_report.ready() {
            return Err(DawSessionManifestBuildError::DawTempoMapBlocked {
                blockers: tempo_map_report.blockers,
            });
        }

        let source_artifacts = vec![
            source_artifact_for_role(receipt, ExportArtifactRole::ExportManifest)?,
            source_artifact_for_role(receipt, ExportArtifactRole::ProductExportProof)?,
        ];
        let tempo_map_ref = receipt.daw_tempo_map_ref.clone().ok_or(
            DawSessionManifestBuildError::DawTempoMapBlocked {
                blockers: vec![DawTempoMapReadinessBlocker::MissingTempoMapRef],
            },
        )?;

        Self::new(DawSessionManifestInput {
            package_id: receipt.pack_id.clone(),
            export_role: receipt.export_role,
            export_boundary: receipt.export_boundary,
            receipt_id: receipt.receipt_id.clone(),
            created_by_action: receipt.created_by_action,
            placement_refs: receipt.arrangement_placement_refs.clone(),
            tempo_map_ref,
            source_artifacts,
            planned_artifacts,
        })
        .map_err(DawSessionManifestBuildError::Manifest)
    }

    pub fn normalized_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec_pretty(self)
    }

    pub fn normalized_json_sha256(&self) -> Result<String, serde_json::Error> {
        let bytes = self.normalized_json_bytes()?;
        let mut digest = Sha256::new();
        digest.update(&bytes);
        Ok(format!("{:x}", digest.finalize()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DawSessionManifestInput {
    pub package_id: String,
    pub export_role: ProductExportRole,
    pub export_boundary: ProductExportBoundary,
    pub receipt_id: ExportReceiptId,
    pub created_by_action: ActionId,
    pub placement_refs: Vec<ExportArrangementPlacementRef>,
    pub tempo_map_ref: ExportDawTempoMapRef,
    pub source_artifacts: Vec<DawSessionSourceArtifactIdentity>,
    pub planned_artifacts: Vec<DawSessionPlannedJsonIdentity>,
}

#[must_use]
pub const fn default_daw_session_manifest_export_role() -> ProductExportRole {
    ProductExportRole::ArrangementManifest
}

#[must_use]
pub const fn default_daw_session_manifest_boundary() -> ProductExportBoundary {
    ProductExportBoundary::ArrangementDawPlacementContractV1
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DawSessionSourceArtifactIdentity {
    pub role: ExportArtifactRole,
    pub location: ExportArtifactLocation,
    pub media_type: ExportArtifactMediaType,
    pub sha256: String,
}

impl DawSessionSourceArtifactIdentity {
    pub fn from_artifact_set_entry(
        entry: &ExportArtifactSetEntry,
    ) -> Result<Self, DawSessionManifestError> {
        if entry.role != ExportArtifactRole::ExportManifest
            && entry.role != ExportArtifactRole::ProductExportProof
        {
            return Err(DawSessionManifestError::UnexpectedSourceArtifactRole { role: entry.role });
        }
        if entry.location_identity().trim().is_empty() {
            return Err(DawSessionManifestError::BlankArtifactLocation);
        }
        if entry.sha256.trim().is_empty() {
            return Err(DawSessionManifestError::BlankArtifactSha256 { role: entry.role });
        }
        if entry.media_type != ExportArtifactMediaType::Json {
            return Err(DawSessionManifestError::NonJsonArtifact {
                role: entry.role,
                media_type: entry.media_type,
            });
        }

        Ok(Self {
            role: entry.role,
            location: entry.location.clone(),
            media_type: entry.media_type,
            sha256: entry.sha256.clone(),
        })
    }

    #[must_use]
    pub fn location_identity(&self) -> &str {
        location_identity(&self.location)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DawSessionPlannedArtifactRole {
    ArrangementManifest,
    TempoMap,
    DawSessionProof,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DawSessionPlannedJsonIdentity {
    pub role: DawSessionPlannedArtifactRole,
    pub location: ExportArtifactLocation,
    pub media_type: ExportArtifactMediaType,
}

impl DawSessionPlannedJsonIdentity {
    #[must_use]
    pub fn local_path(role: DawSessionPlannedArtifactRole, path: impl Into<String>) -> Self {
        Self {
            role,
            location: ExportArtifactLocation::LocalPath { path: path.into() },
            media_type: ExportArtifactMediaType::Json,
        }
    }

    #[must_use]
    pub fn location_identity(&self) -> &str {
        location_identity(&self.location)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DawSessionManifestError {
    BlankPackageId,
    UnexpectedExportRole {
        expected: ProductExportRole,
        actual: ProductExportRole,
    },
    UnexpectedExportBoundary {
        expected: ProductExportBoundary,
        actual: ProductExportBoundary,
    },
    MissingPlacementRefs,
    MissingSourceArtifact {
        role: ExportArtifactRole,
    },
    DuplicateSourceArtifact {
        role: ExportArtifactRole,
    },
    UnexpectedSourceArtifactRole {
        role: ExportArtifactRole,
    },
    BlankArtifactLocation,
    BlankArtifactSha256 {
        role: ExportArtifactRole,
    },
    NonJsonArtifact {
        role: ExportArtifactRole,
        media_type: ExportArtifactMediaType,
    },
    MissingPlannedArtifact {
        role: DawSessionPlannedArtifactRole,
    },
    DuplicatePlannedArtifact {
        role: DawSessionPlannedArtifactRole,
    },
    BlankPlannedArtifactLocation {
        role: DawSessionPlannedArtifactRole,
    },
    NonJsonPlannedArtifact {
        role: DawSessionPlannedArtifactRole,
        media_type: ExportArtifactMediaType,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DawSessionManifestBuildError {
    NotDawSessionScope {
        export_scope: ExportScope,
    },
    NotDawSessionBoundary {
        export_boundary: ProductExportBoundary,
    },
    UnsupportedDawExportFlagPresent,
    ArrangementPlacementBlocked {
        blockers: Vec<ArrangementExportPlacementReadinessBlocker>,
    },
    DawTempoMapBlocked {
        blockers: Vec<DawTempoMapReadinessBlocker>,
    },
    MissingSourceArtifact {
        role: ExportArtifactRole,
    },
    MultipleSourceArtifacts {
        role: ExportArtifactRole,
    },
    Manifest(DawSessionManifestError),
}

fn source_artifact_for_role(
    receipt: &ExportReceiptState,
    role: ExportArtifactRole,
) -> Result<DawSessionSourceArtifactIdentity, DawSessionManifestBuildError> {
    let mut matches = receipt
        .artifact_set
        .iter()
        .filter(|entry| entry.role == role);
    let entry = matches
        .next()
        .ok_or(DawSessionManifestBuildError::MissingSourceArtifact { role })?;
    if matches.next().is_some() {
        return Err(DawSessionManifestBuildError::MultipleSourceArtifacts { role });
    }

    DawSessionSourceArtifactIdentity::from_artifact_set_entry(entry)
        .map_err(DawSessionManifestBuildError::Manifest)
}

pub(crate) fn validate_daw_session_source_artifacts(
    source_artifacts: &[DawSessionSourceArtifactIdentity],
) -> Result<(), DawSessionManifestError> {
    for role in [
        ExportArtifactRole::ExportManifest,
        ExportArtifactRole::ProductExportProof,
    ] {
        let matches = source_artifacts
            .iter()
            .filter(|artifact| artifact.role == role)
            .count();
        match matches {
            0 => return Err(DawSessionManifestError::MissingSourceArtifact { role }),
            1 => {}
            _ => return Err(DawSessionManifestError::DuplicateSourceArtifact { role }),
        }
    }

    for artifact in source_artifacts {
        if artifact.role != ExportArtifactRole::ExportManifest
            && artifact.role != ExportArtifactRole::ProductExportProof
        {
            return Err(DawSessionManifestError::UnexpectedSourceArtifactRole {
                role: artifact.role,
            });
        }
        if artifact.location_identity().trim().is_empty() {
            return Err(DawSessionManifestError::BlankArtifactLocation);
        }
        if artifact.sha256.trim().is_empty() {
            return Err(DawSessionManifestError::BlankArtifactSha256 {
                role: artifact.role,
            });
        }
        if artifact.media_type != ExportArtifactMediaType::Json {
            return Err(DawSessionManifestError::NonJsonArtifact {
                role: artifact.role,
                media_type: artifact.media_type,
            });
        }
    }

    Ok(())
}

pub(crate) fn validate_daw_session_planned_artifacts(
    planned_artifacts: &[DawSessionPlannedJsonIdentity],
) -> Result<(), DawSessionManifestError> {
    for role in [
        DawSessionPlannedArtifactRole::ArrangementManifest,
        DawSessionPlannedArtifactRole::TempoMap,
        DawSessionPlannedArtifactRole::DawSessionProof,
    ] {
        let matches = planned_artifacts
            .iter()
            .filter(|artifact| artifact.role == role)
            .count();
        match matches {
            0 => return Err(DawSessionManifestError::MissingPlannedArtifact { role }),
            1 => {}
            _ => return Err(DawSessionManifestError::DuplicatePlannedArtifact { role }),
        }
    }

    for artifact in planned_artifacts {
        if artifact.location_identity().trim().is_empty() {
            return Err(DawSessionManifestError::BlankPlannedArtifactLocation {
                role: artifact.role,
            });
        }
        if artifact.media_type != ExportArtifactMediaType::Json {
            return Err(DawSessionManifestError::NonJsonPlannedArtifact {
                role: artifact.role,
                media_type: artifact.media_type,
            });
        }
    }

    Ok(())
}

#[must_use]
fn location_identity(location: &ExportArtifactLocation) -> &str {
    match location {
        ExportArtifactLocation::LocalPath { path } | ExportArtifactLocation::Uri { uri: path } => {
            path
        }
    }
}

#[cfg(test)]
#[path = "daw_session_manifest_tests.rs"]
mod daw_session_manifest_tests;
