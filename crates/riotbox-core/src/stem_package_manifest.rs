use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    export_readiness::{ExportScope, ProductExportBoundary, ProductExportRole},
    ids::{ActionId, ExportReceiptId},
    session::{
        ExportArtifactAudioMetrics, ExportArtifactFallbackComparisonEvidence,
        ExportArtifactLocation, ExportArtifactMediaType, ExportArtifactRole,
        ExportArtifactSetEntry, ExportArtifactSourceGraphRef, ExportArtifactTimingGridRef,
        ExportReceiptQaGateResult, ExportReceiptState, STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID,
    },
};

pub const STEM_PACKAGE_MANIFEST_SCHEMA_ID: &str = "riotbox.stem_package_manifest";
pub const STEM_PACKAGE_MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageManifest {
    pub schema_id: String,
    pub schema_version: u32,
    pub package_id: String,
    pub export_scope: ExportScope,
    #[serde(default = "default_stem_package_manifest_export_role")]
    pub export_role: ProductExportRole,
    #[serde(default = "default_stem_package_manifest_boundary")]
    pub export_boundary: ProductExportBoundary,
    pub receipt_id: ExportReceiptId,
    pub created_by_action: ActionId,
    pub claimed_stem_roles: Vec<ExportArtifactRole>,
    pub artifacts: Vec<StemPackageManifestArtifact>,
    pub manifest_identity: StemPackageManifestJsonIdentity,
    pub proof_identity: StemPackageManifestJsonIdentity,
    #[serde(default)]
    pub qa_gates: Vec<ExportReceiptQaGateResult>,
}

impl StemPackageManifest {
    pub fn new(input: StemPackageManifestInput) -> Result<Self, StemPackageManifestError> {
        let package_id = input.package_id;
        if package_id.trim().is_empty() {
            return Err(StemPackageManifestError::BlankPackageId);
        }

        validate_claimed_stem_roles(&input.claimed_stem_roles)?;
        validate_artifacts_match_claims(&input.claimed_stem_roles, &input.artifacts)?;
        input
            .manifest_identity
            .validate_role(ExportArtifactRole::ExportManifest)?;
        input
            .proof_identity
            .validate_role(ExportArtifactRole::ProductExportProof)?;

        Ok(Self {
            schema_id: STEM_PACKAGE_MANIFEST_SCHEMA_ID.into(),
            schema_version: STEM_PACKAGE_MANIFEST_SCHEMA_VERSION,
            package_id,
            export_scope: ExportScope::StemPackage,
            export_role: input.export_role,
            export_boundary: input.export_boundary,
            receipt_id: input.receipt_id,
            created_by_action: input.created_by_action,
            claimed_stem_roles: input.claimed_stem_roles,
            artifacts: input.artifacts,
            manifest_identity: input.manifest_identity,
            proof_identity: input.proof_identity,
            qa_gates: input.qa_gates,
        })
    }

    pub fn from_receipt(
        receipt: &ExportReceiptState,
    ) -> Result<Self, StemPackageManifestBuildError> {
        if receipt.export_scope != ExportScope::StemPackage {
            return Err(StemPackageManifestBuildError::NotStemPackageScope {
                export_scope: receipt.export_scope,
            });
        }

        let claimed_stem_roles = claimed_stem_roles_from_receipt(receipt)?;
        let artifacts = receipt
            .artifact_set
            .iter()
            .filter(|entry| entry.role.is_stem_role())
            .map(StemPackageManifestArtifact::from_artifact_set_entry)
            .collect::<Result<Vec<_>, _>>()?;
        let manifest_identity =
            json_identity_for_role(receipt, ExportArtifactRole::ExportManifest)?;
        let proof_identity =
            json_identity_for_role(receipt, ExportArtifactRole::ProductExportProof)?;

        Self::new(StemPackageManifestInput {
            package_id: receipt.pack_id.clone(),
            export_role: receipt.export_role,
            export_boundary: receipt.export_boundary,
            receipt_id: receipt.receipt_id.clone(),
            created_by_action: receipt.created_by_action,
            claimed_stem_roles,
            artifacts,
            manifest_identity,
            proof_identity,
            qa_gates: receipt.qa_gates.clone(),
        })
        .map_err(StemPackageManifestBuildError::Manifest)
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
pub struct StemPackageManifestInput {
    pub package_id: String,
    pub export_role: ProductExportRole,
    pub export_boundary: ProductExportBoundary,
    pub receipt_id: ExportReceiptId,
    pub created_by_action: ActionId,
    pub claimed_stem_roles: Vec<ExportArtifactRole>,
    pub artifacts: Vec<StemPackageManifestArtifact>,
    pub manifest_identity: StemPackageManifestJsonIdentity,
    pub proof_identity: StemPackageManifestJsonIdentity,
    pub qa_gates: Vec<ExportReceiptQaGateResult>,
}

#[must_use]
pub const fn default_stem_package_manifest_export_role() -> ProductExportRole {
    ProductExportRole::PackageManifest
}

#[must_use]
pub const fn default_stem_package_manifest_boundary() -> ProductExportBoundary {
    ProductExportBoundary::StemPackageLocalCiPackageV1
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageManifestArtifact {
    pub role: ExportArtifactRole,
    pub location: ExportArtifactLocation,
    pub media_type: ExportArtifactMediaType,
    pub sha256: String,
    #[serde(default)]
    pub source_graph_ref: Option<ExportArtifactSourceGraphRef>,
    #[serde(default)]
    pub timing_grid_ref: Option<ExportArtifactTimingGridRef>,
    #[serde(default)]
    pub source_capture_refs: Vec<crate::ids::CaptureId>,
    #[serde(default)]
    pub lineage_capture_refs: Vec<crate::ids::CaptureId>,
    #[serde(default)]
    pub fallback_comparison: Option<ExportArtifactFallbackComparisonEvidence>,
    #[serde(default)]
    pub audio_metrics: Option<ExportArtifactAudioMetrics>,
    #[serde(default)]
    pub sample_rate_hz: Option<u32>,
    #[serde(default)]
    pub channel_count: Option<u16>,
    #[serde(default)]
    pub duration_ms: Option<u64>,
}

impl StemPackageManifestArtifact {
    pub fn from_artifact_set_entry(
        entry: &ExportArtifactSetEntry,
    ) -> Result<Self, StemPackageManifestError> {
        if !entry.role.is_stem_role() {
            return Err(StemPackageManifestError::NonStemArtifactRole { role: entry.role });
        }
        if entry.location_identity().trim().is_empty() {
            return Err(StemPackageManifestError::BlankArtifactLocation { role: entry.role });
        }
        if entry.sha256.trim().is_empty() {
            return Err(StemPackageManifestError::BlankArtifactSha256 { role: entry.role });
        }
        if entry.media_type != ExportArtifactMediaType::AudioWav {
            return Err(StemPackageManifestError::NonWavStemArtifact {
                role: entry.role,
                media_type: entry.media_type,
            });
        }

        Ok(Self {
            role: entry.role,
            location: entry.location.clone(),
            media_type: entry.media_type,
            sha256: entry.sha256.clone(),
            source_graph_ref: entry.source_graph_ref.clone(),
            timing_grid_ref: entry.timing_grid_ref.clone(),
            source_capture_refs: entry.source_capture_refs.clone(),
            lineage_capture_refs: entry.lineage_capture_refs.clone(),
            fallback_comparison: entry.fallback_comparison.clone(),
            audio_metrics: entry.audio_metrics.clone(),
            sample_rate_hz: entry.sample_rate_hz,
            channel_count: entry.channel_count,
            duration_ms: entry.duration_ms,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageManifestJsonIdentity {
    pub role: ExportArtifactRole,
    pub location: ExportArtifactLocation,
    pub media_type: ExportArtifactMediaType,
}

impl StemPackageManifestJsonIdentity {
    pub fn from_artifact_set_entry(
        entry: &ExportArtifactSetEntry,
    ) -> Result<Self, StemPackageManifestError> {
        if entry.location_identity().trim().is_empty() {
            return Err(StemPackageManifestError::BlankArtifactLocation { role: entry.role });
        }
        if entry.media_type != ExportArtifactMediaType::Json {
            return Err(StemPackageManifestError::NonJsonIdentity {
                role: entry.role,
                media_type: entry.media_type,
            });
        }

        Ok(Self {
            role: entry.role,
            location: entry.location.clone(),
            media_type: entry.media_type,
        })
    }

    fn validate_role(&self, expected: ExportArtifactRole) -> Result<(), StemPackageManifestError> {
        if self.role != expected {
            return Err(StemPackageManifestError::UnexpectedJsonIdentityRole {
                expected,
                actual: self.role,
            });
        }
        if self.location_identity().trim().is_empty() {
            return Err(StemPackageManifestError::BlankArtifactLocation { role: self.role });
        }
        if self.media_type != ExportArtifactMediaType::Json {
            return Err(StemPackageManifestError::NonJsonIdentity {
                role: self.role,
                media_type: self.media_type,
            });
        }
        Ok(())
    }

    #[must_use]
    pub fn location_identity(&self) -> &str {
        match &self.location {
            ExportArtifactLocation::LocalPath { path }
            | ExportArtifactLocation::Uri { uri: path } => path,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StemPackageManifestError {
    BlankPackageId,
    EmptyClaimedStemRoles,
    NonStemClaimedRole {
        role: ExportArtifactRole,
    },
    DuplicateClaimedStemRole {
        role: ExportArtifactRole,
    },
    EmptyArtifacts,
    NonStemArtifactRole {
        role: ExportArtifactRole,
    },
    UnclaimedStemArtifact {
        role: ExportArtifactRole,
    },
    MissingClaimedStemArtifact {
        role: ExportArtifactRole,
    },
    DuplicateStemArtifact {
        role: ExportArtifactRole,
    },
    BlankArtifactLocation {
        role: ExportArtifactRole,
    },
    BlankArtifactSha256 {
        role: ExportArtifactRole,
    },
    NonWavStemArtifact {
        role: ExportArtifactRole,
        media_type: ExportArtifactMediaType,
    },
    UnexpectedJsonIdentityRole {
        expected: ExportArtifactRole,
        actual: ExportArtifactRole,
    },
    NonJsonIdentity {
        role: ExportArtifactRole,
        media_type: ExportArtifactMediaType,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StemPackageManifestBuildError {
    NotStemPackageScope { export_scope: ExportScope },
    MissingStemPackageArtifactSetQaGate,
    MissingJsonIdentity { role: ExportArtifactRole },
    MultipleJsonIdentities { role: ExportArtifactRole },
    Manifest(StemPackageManifestError),
}

impl From<StemPackageManifestError> for StemPackageManifestBuildError {
    fn from(value: StemPackageManifestError) -> Self {
        Self::Manifest(value)
    }
}

fn claimed_stem_roles_from_receipt(
    receipt: &ExportReceiptState,
) -> Result<Vec<ExportArtifactRole>, StemPackageManifestBuildError> {
    receipt
        .qa_gates
        .iter()
        .find(|gate| gate.gate_id == STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID)
        .map(|gate| gate.artifact_roles.clone())
        .ok_or(StemPackageManifestBuildError::MissingStemPackageArtifactSetQaGate)
}

fn json_identity_for_role(
    receipt: &ExportReceiptState,
    role: ExportArtifactRole,
) -> Result<StemPackageManifestJsonIdentity, StemPackageManifestBuildError> {
    let mut matches = receipt
        .artifact_set
        .iter()
        .filter(|entry| entry.role == role);
    let entry = matches
        .next()
        .ok_or(StemPackageManifestBuildError::MissingJsonIdentity { role })?;
    if matches.next().is_some() {
        return Err(StemPackageManifestBuildError::MultipleJsonIdentities { role });
    }

    StemPackageManifestJsonIdentity::from_artifact_set_entry(entry).map_err(Into::into)
}

fn validate_claimed_stem_roles(
    claimed_stem_roles: &[ExportArtifactRole],
) -> Result<(), StemPackageManifestError> {
    if claimed_stem_roles.is_empty() {
        return Err(StemPackageManifestError::EmptyClaimedStemRoles);
    }

    for (index, role) in claimed_stem_roles.iter().enumerate() {
        if !role.is_stem_role() {
            return Err(StemPackageManifestError::NonStemClaimedRole { role: *role });
        }
        if claimed_stem_roles[index + 1..].contains(role) {
            return Err(StemPackageManifestError::DuplicateClaimedStemRole { role: *role });
        }
    }

    Ok(())
}

fn validate_artifacts_match_claims(
    claimed_stem_roles: &[ExportArtifactRole],
    artifacts: &[StemPackageManifestArtifact],
) -> Result<(), StemPackageManifestError> {
    if artifacts.is_empty() {
        return Err(StemPackageManifestError::EmptyArtifacts);
    }

    for artifact in artifacts {
        if !artifact.role.is_stem_role() {
            return Err(StemPackageManifestError::NonStemArtifactRole {
                role: artifact.role,
            });
        }
        if !claimed_stem_roles.contains(&artifact.role) {
            return Err(StemPackageManifestError::UnclaimedStemArtifact {
                role: artifact.role,
            });
        }
        if artifact.location_identity().trim().is_empty() {
            return Err(StemPackageManifestError::BlankArtifactLocation {
                role: artifact.role,
            });
        }
        if artifact.sha256.trim().is_empty() {
            return Err(StemPackageManifestError::BlankArtifactSha256 {
                role: artifact.role,
            });
        }
        if artifact.media_type != ExportArtifactMediaType::AudioWav {
            return Err(StemPackageManifestError::NonWavStemArtifact {
                role: artifact.role,
                media_type: artifact.media_type,
            });
        }
    }

    for role in claimed_stem_roles {
        let matches = artifacts
            .iter()
            .filter(|artifact| artifact.role == *role)
            .count();
        match matches {
            0 => {
                return Err(StemPackageManifestError::MissingClaimedStemArtifact { role: *role });
            }
            1 => {}
            _ => return Err(StemPackageManifestError::DuplicateStemArtifact { role: *role }),
        }
    }

    Ok(())
}

impl StemPackageManifestArtifact {
    #[must_use]
    pub fn location_identity(&self) -> &str {
        match &self.location {
            ExportArtifactLocation::LocalPath { path }
            | ExportArtifactLocation::Uri { uri: path } => path,
        }
    }
}

#[cfg(test)]
#[path = "stem_package_manifest_from_receipt_tests.rs"]
mod stem_package_manifest_from_receipt_tests;
#[cfg(test)]
#[path = "stem_package_manifest_tests.rs"]
mod stem_package_manifest_tests;
