use serde::{Deserialize, Serialize};

use crate::{
    export_readiness::{ExportScope, ProductExportBoundary, ProductExportRole},
    ids::{ActionId, ExportReceiptId},
    session::{ExportArtifactMediaType, ExportArtifactRole},
    stem_package_manifest::{StemPackageManifest, StemPackageManifestJsonIdentity},
};

pub const STEM_PACKAGE_PROOF_SCHEMA_ID: &str = "riotbox.stem_package_proof";
pub const STEM_PACKAGE_PROOF_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageProof {
    pub schema_id: String,
    pub schema_version: u32,
    pub package_id: String,
    pub export_scope: ExportScope,
    #[serde(default = "default_stem_package_proof_export_role")]
    pub export_role: ProductExportRole,
    #[serde(default = "default_stem_package_proof_boundary")]
    pub export_boundary: ProductExportBoundary,
    pub receipt_id: ExportReceiptId,
    pub created_by_action: ActionId,
    pub manifest_sha256: String,
    pub claimed_stem_roles: Vec<ExportArtifactRole>,
    pub manifest_identity: StemPackageManifestJsonIdentity,
    pub proof_identity: StemPackageManifestJsonIdentity,
}

impl StemPackageProof {
    pub fn from_manifest(manifest: &StemPackageManifest) -> Result<Self, StemPackageProofError> {
        let manifest_sha256 = manifest
            .normalized_json_sha256()
            .map_err(|_| StemPackageProofError::ManifestSerialization)?;
        Self::new(StemPackageProofInput {
            package_id: manifest.package_id.clone(),
            export_role: manifest.export_role,
            export_boundary: manifest.export_boundary,
            receipt_id: manifest.receipt_id.clone(),
            created_by_action: manifest.created_by_action,
            manifest_sha256,
            claimed_stem_roles: manifest.claimed_stem_roles.clone(),
            manifest_identity: manifest.manifest_identity.clone(),
            proof_identity: manifest.proof_identity.clone(),
        })
    }

    pub fn new(input: StemPackageProofInput) -> Result<Self, StemPackageProofError> {
        let package_id = input.package_id;
        if package_id.trim().is_empty() {
            return Err(StemPackageProofError::BlankPackageId);
        }
        let manifest_sha256 = input.manifest_sha256;
        if manifest_sha256.trim().is_empty() {
            return Err(StemPackageProofError::BlankManifestSha256);
        }

        validate_claimed_stem_roles(&input.claimed_stem_roles)?;
        validate_json_identity(&input.manifest_identity, ExportArtifactRole::ExportManifest)?;
        validate_json_identity(
            &input.proof_identity,
            ExportArtifactRole::ProductExportProof,
        )?;

        Ok(Self {
            schema_id: STEM_PACKAGE_PROOF_SCHEMA_ID.into(),
            schema_version: STEM_PACKAGE_PROOF_SCHEMA_VERSION,
            package_id,
            export_scope: ExportScope::StemPackage,
            export_role: input.export_role,
            export_boundary: input.export_boundary,
            receipt_id: input.receipt_id,
            created_by_action: input.created_by_action,
            manifest_sha256,
            claimed_stem_roles: input.claimed_stem_roles,
            manifest_identity: input.manifest_identity,
            proof_identity: input.proof_identity,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StemPackageProofInput {
    pub package_id: String,
    pub export_role: ProductExportRole,
    pub export_boundary: ProductExportBoundary,
    pub receipt_id: ExportReceiptId,
    pub created_by_action: ActionId,
    pub manifest_sha256: String,
    pub claimed_stem_roles: Vec<ExportArtifactRole>,
    pub manifest_identity: StemPackageManifestJsonIdentity,
    pub proof_identity: StemPackageManifestJsonIdentity,
}

#[must_use]
pub const fn default_stem_package_proof_export_role() -> ProductExportRole {
    ProductExportRole::PackageManifest
}

#[must_use]
pub const fn default_stem_package_proof_boundary() -> ProductExportBoundary {
    ProductExportBoundary::StemPackageLocalCiPackageV1
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StemPackageProofError {
    ManifestSerialization,
    BlankPackageId,
    BlankManifestSha256,
    EmptyClaimedStemRoles,
    NonStemClaimedRole {
        role: ExportArtifactRole,
    },
    DuplicateClaimedStemRole {
        role: ExportArtifactRole,
    },
    UnexpectedJsonIdentityRole {
        expected: ExportArtifactRole,
        actual: ExportArtifactRole,
    },
    BlankJsonIdentityLocation {
        role: ExportArtifactRole,
    },
    NonJsonIdentity {
        role: ExportArtifactRole,
        media_type: ExportArtifactMediaType,
    },
}

fn validate_claimed_stem_roles(
    claimed_stem_roles: &[ExportArtifactRole],
) -> Result<(), StemPackageProofError> {
    if claimed_stem_roles.is_empty() {
        return Err(StemPackageProofError::EmptyClaimedStemRoles);
    }

    for (index, role) in claimed_stem_roles.iter().enumerate() {
        if !role.is_stem_role() {
            return Err(StemPackageProofError::NonStemClaimedRole { role: *role });
        }
        if claimed_stem_roles[index + 1..].contains(role) {
            return Err(StemPackageProofError::DuplicateClaimedStemRole { role: *role });
        }
    }

    Ok(())
}

fn validate_json_identity(
    identity: &StemPackageManifestJsonIdentity,
    expected: ExportArtifactRole,
) -> Result<(), StemPackageProofError> {
    if identity.role != expected {
        return Err(StemPackageProofError::UnexpectedJsonIdentityRole {
            expected,
            actual: identity.role,
        });
    }
    if identity.location_identity().trim().is_empty() {
        return Err(StemPackageProofError::BlankJsonIdentityLocation {
            role: identity.role,
        });
    }
    if identity.media_type != ExportArtifactMediaType::Json {
        return Err(StemPackageProofError::NonJsonIdentity {
            role: identity.role,
            media_type: identity.media_type,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        export_readiness::{
            ProductExportBoundary, ProductExportRole, STEM_PACKAGE_LOCAL_CI_PACK_ID,
        },
        session::{ExportArtifactLocation, ExportArtifactSetEntry},
        stem_package_manifest::{
            StemPackageManifest, StemPackageManifestArtifact, StemPackageManifestInput,
        },
    };

    #[test]
    fn stem_package_proof_roundtrips_stable_schema_scope_roles_and_identity() {
        let proof = fixture_proof("manifest-sha");

        let json = serde_json::to_value(&proof).expect("serialize proof");

        assert_eq!(json["schema_id"], STEM_PACKAGE_PROOF_SCHEMA_ID);
        assert_eq!(json["schema_version"], STEM_PACKAGE_PROOF_SCHEMA_VERSION);
        assert_eq!(json["package_id"], STEM_PACKAGE_LOCAL_CI_PACK_ID);
        assert_eq!(json["export_scope"], "stem_package");
        assert_eq!(json["export_role"], "package_manifest");
        assert_eq!(json["export_boundary"], "stem_package_local_ci_package_v1");
        assert_eq!(json["manifest_sha256"], "manifest-sha");
        assert_eq!(
            json["claimed_stem_roles"],
            json!(["stem_drums", "stem_bass"])
        );
        assert_eq!(json["manifest_identity"]["role"], "export_manifest");
        assert_eq!(json["proof_identity"]["role"], "product_export_proof");
        assert!(json["manifest_identity"].get("sha256").is_none());
        assert!(json["proof_identity"].get("sha256").is_none());

        let roundtrip: StemPackageProof = serde_json::from_value(json).expect("deserialize proof");
        assert_eq!(roundtrip, proof);
    }

    #[test]
    fn stem_package_proof_serialized_value_changes_with_manifest_hash() {
        let proof = fixture_proof("manifest-sha-a");
        let changed = fixture_proof("manifest-sha-b");

        let bytes = serde_json::to_vec_pretty(&proof).expect("serialize proof");
        let repeated_bytes = serde_json::to_vec_pretty(&proof).expect("serialize proof again");
        let changed_bytes = serde_json::to_vec_pretty(&changed).expect("serialize changed proof");

        assert_eq!(bytes, repeated_bytes);
        assert_ne!(bytes, changed_bytes);
    }

    #[test]
    fn stem_package_proof_from_manifest_uses_normalized_manifest_hash() {
        let manifest = fixture_manifest();
        let proof = StemPackageProof::from_manifest(&manifest).expect("proof from manifest");
        let expected_manifest_hash = manifest
            .normalized_json_sha256()
            .expect("hash normalized manifest");

        assert_eq!(proof.package_id, manifest.package_id);
        assert_eq!(proof.export_role, manifest.export_role);
        assert_eq!(proof.export_boundary, manifest.export_boundary);
        assert_eq!(proof.receipt_id, manifest.receipt_id);
        assert_eq!(proof.created_by_action, manifest.created_by_action);
        assert_eq!(proof.manifest_sha256, expected_manifest_hash);
        assert_eq!(proof.claimed_stem_roles, manifest.claimed_stem_roles);
        assert_eq!(proof.manifest_identity, manifest.manifest_identity);
        assert_eq!(proof.proof_identity, manifest.proof_identity);
    }

    #[test]
    fn stem_package_proof_from_manifest_changes_hash_when_manifest_artifact_identity_changes() {
        let manifest = fixture_manifest();
        let mut changed = manifest.clone();
        changed.artifacts[0].sha256 = "changed-drums-sha".into();

        let proof = StemPackageProof::from_manifest(&manifest).expect("proof from manifest");
        let changed_proof =
            StemPackageProof::from_manifest(&changed).expect("changed proof from manifest");

        assert_ne!(proof.manifest_sha256, changed_proof.manifest_sha256);
    }

    #[test]
    fn stem_package_proof_rejects_blank_package_id_and_manifest_sha() {
        let err = StemPackageProof::new(proof_input(" ", "manifest-sha"))
            .expect_err("blank package id should fail");
        assert_eq!(err, StemPackageProofError::BlankPackageId);

        let err = StemPackageProof::new(proof_input("pkg", " "))
            .expect_err("blank manifest sha should fail");
        assert_eq!(err, StemPackageProofError::BlankManifestSha256);
    }

    #[test]
    fn stem_package_proof_rejects_empty_duplicate_and_non_stem_claims() {
        let mut input = proof_input("pkg", "manifest-sha");
        input.claimed_stem_roles.clear();
        let err = StemPackageProof::new(input).expect_err("empty claims should fail");
        assert_eq!(err, StemPackageProofError::EmptyClaimedStemRoles);

        let mut input = proof_input("pkg", "manifest-sha");
        input.claimed_stem_roles = vec![ExportArtifactRole::FullGridMix];
        let err = StemPackageProof::new(input).expect_err("non-stem claim should fail");
        assert_eq!(
            err,
            StemPackageProofError::NonStemClaimedRole {
                role: ExportArtifactRole::FullGridMix
            }
        );

        let mut input = proof_input("pkg", "manifest-sha");
        input.claimed_stem_roles =
            vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemDrums];
        let err = StemPackageProof::new(input).expect_err("duplicate claim should fail");
        assert_eq!(
            err,
            StemPackageProofError::DuplicateClaimedStemRole {
                role: ExportArtifactRole::StemDrums
            }
        );
    }

    #[test]
    fn stem_package_proof_rejects_wrong_or_invalid_json_identities() {
        let mut input = proof_input("pkg", "manifest-sha");
        input.manifest_identity = proof_identity("manifest.json", "manifest-id");
        let err = StemPackageProof::new(input).expect_err("wrong manifest role should fail");
        assert_eq!(
            err,
            StemPackageProofError::UnexpectedJsonIdentityRole {
                expected: ExportArtifactRole::ExportManifest,
                actual: ExportArtifactRole::ProductExportProof
            }
        );

        let mut input = proof_input("pkg", "manifest-sha");
        input.proof_identity.media_type = ExportArtifactMediaType::AudioWav;
        let err = StemPackageProof::new(input).expect_err("non-json proof should fail");
        assert_eq!(
            err,
            StemPackageProofError::NonJsonIdentity {
                role: ExportArtifactRole::ProductExportProof,
                media_type: ExportArtifactMediaType::AudioWav
            }
        );
    }

    fn fixture_proof(manifest_sha256: impl Into<String>) -> StemPackageProof {
        StemPackageProof::new(proof_input(STEM_PACKAGE_LOCAL_CI_PACK_ID, manifest_sha256))
            .expect("fixture proof")
    }

    fn fixture_manifest() -> StemPackageManifest {
        StemPackageManifest::new(StemPackageManifestInput {
            package_id: STEM_PACKAGE_LOCAL_CI_PACK_ID.into(),
            export_role: ProductExportRole::PackageManifest,
            export_boundary: ProductExportBoundary::StemPackageLocalCiPackageV1,
            receipt_id: ExportReceiptId::new("receipt-1"),
            created_by_action: ActionId(7),
            claimed_stem_roles: vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
            artifacts: vec![
                stem_artifact(ExportArtifactRole::StemDrums, "drums.wav", "drums-sha"),
                stem_artifact(ExportArtifactRole::StemBass, "bass.wav", "bass-sha"),
            ],
            manifest_identity: manifest_identity("manifest.json", "manifest-id"),
            proof_identity: proof_identity("proof.json", "proof-id"),
            qa_gates: Vec::new(),
        })
        .expect("fixture manifest")
    }

    fn stem_artifact(
        role: ExportArtifactRole,
        path: impl Into<String>,
        sha256: impl Into<String>,
    ) -> StemPackageManifestArtifact {
        StemPackageManifestArtifact::from_artifact_set_entry(&ExportArtifactSetEntry {
            role,
            location: ExportArtifactLocation::LocalPath { path: path.into() },
            media_type: ExportArtifactMediaType::AudioWav,
            sha256: sha256.into(),
            normalized_manifest_hash: None,
            source_graph_ref: None,
            timing_grid_ref: None,
            source_capture_refs: Vec::new(),
            lineage_capture_refs: Vec::new(),
            fallback_comparison: None,
            audio_metrics: None,
            sample_rate_hz: Some(48_000),
            channel_count: Some(2),
            duration_ms: Some(1000),
        })
        .expect("stem artifact")
    }

    fn proof_input(
        package_id: impl Into<String>,
        manifest_sha256: impl Into<String>,
    ) -> StemPackageProofInput {
        StemPackageProofInput {
            package_id: package_id.into(),
            export_role: ProductExportRole::PackageManifest,
            export_boundary: ProductExportBoundary::StemPackageLocalCiPackageV1,
            receipt_id: ExportReceiptId::new("receipt-1"),
            created_by_action: ActionId(7),
            manifest_sha256: manifest_sha256.into(),
            claimed_stem_roles: vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
            manifest_identity: manifest_identity("manifest.json", "manifest-id"),
            proof_identity: proof_identity("proof.json", "proof-id"),
        }
    }

    fn manifest_identity(
        path: impl Into<String>,
        sha256: impl Into<String>,
    ) -> StemPackageManifestJsonIdentity {
        json_identity(ExportArtifactRole::ExportManifest, path, sha256)
    }

    fn proof_identity(
        path: impl Into<String>,
        sha256: impl Into<String>,
    ) -> StemPackageManifestJsonIdentity {
        json_identity(ExportArtifactRole::ProductExportProof, path, sha256)
    }

    fn json_identity(
        role: ExportArtifactRole,
        path: impl Into<String>,
        _sha256: impl Into<String>,
    ) -> StemPackageManifestJsonIdentity {
        StemPackageManifestJsonIdentity {
            role,
            location: ExportArtifactLocation::LocalPath { path: path.into() },
            media_type: ExportArtifactMediaType::Json,
        }
    }
}
