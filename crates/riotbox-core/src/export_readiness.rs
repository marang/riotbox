use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PRODUCT_EXPORT_PROOF_SCHEMA: &str = "riotbox.product_export_reproducibility.v1";
pub const EXPORT_READINESS_CONTRACT_SCHEMA: &str = "riotbox.export_readiness_contract.v1";
pub const PRODUCT_EXPORT_PACK_ID: &str = "feral-grid-demo";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductExportReproducibilityProof {
    pub schema: String,
    pub schema_version: u32,
    pub boundary: String,
    pub pack_id: String,
    pub export_role: String,
    pub export_artifact: String,
    pub source_sha256: String,
    pub export_sha256: String,
    pub normalized_manifest_sha256: String,
    pub audio_artifact_sha256: BTreeMap<String, String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportReadinessStatus {
    Reproducible,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductExportBoundary {
    FeralGridGeneratedSupport,
}

impl ProductExportBoundary {
    #[must_use]
    pub const fn as_proof_str(self) -> &'static str {
        match self {
            Self::FeralGridGeneratedSupport => "feral-grid generated-support export",
        }
    }

    fn parse(value: &str) -> Result<Self, ExportReadinessError> {
        match value {
            "feral-grid generated-support export" => Ok(Self::FeralGridGeneratedSupport),
            other => Err(ExportReadinessError::UnsupportedBoundary(other.to_owned())),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductExportRole {
    FullGridMix,
}

impl ProductExportRole {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FullGridMix => "full_grid_mix",
        }
    }

    fn parse(value: &str) -> Result<Self, ExportReadinessError> {
        match value {
            "full_grid_mix" => Ok(Self::FullGridMix),
            other => Err(ExportReadinessError::UnsupportedExportRole(
                other.to_owned(),
            )),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportScope {
    ProductMix,
}

impl ExportScope {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProductMix => "product_mix",
        }
    }

    #[must_use]
    pub const fn musician_label(self) -> &'static str {
        match self {
            Self::ProductMix => "product mix export",
        }
    }
}

#[must_use]
pub const fn default_export_scope() -> ExportScope {
    ExportScope::ProductMix
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductExportDestinationKind {
    LocalArtifactDirectory,
    LocalFilePath,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnsupportedExportScope {
    StemPackage,
    LiveRecording,
    DawExport,
    HostAudioSoak,
}

impl UnsupportedExportScope {
    #[must_use]
    pub const fn musician_label(self) -> &'static str {
        match self {
            Self::StemPackage => "stem package export",
            Self::LiveRecording => "live recording export",
            Self::DawExport => "DAW export",
            Self::HostAudioSoak => "host-audio soak",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportReadinessContract {
    pub schema: String,
    pub status: ExportReadinessStatus,
    pub proof_schema: String,
    #[serde(default = "default_export_scope")]
    pub export_scope: ExportScope,
    pub boundary: ProductExportBoundary,
    pub pack_id: String,
    pub export_role: ProductExportRole,
    pub export_artifact: String,
    pub source_sha256: String,
    pub export_sha256: String,
    pub normalized_manifest_sha256: String,
    pub unsupported_scopes: Vec<UnsupportedExportScope>,
}

impl ExportReadinessContract {
    pub fn from_product_export_proof(
        proof: &ProductExportReproducibilityProof,
    ) -> Result<Self, ExportReadinessError> {
        if proof.schema != PRODUCT_EXPORT_PROOF_SCHEMA {
            return Err(ExportReadinessError::UnsupportedSchema(
                proof.schema.clone(),
            ));
        }
        if proof.schema_version != 1 {
            return Err(ExportReadinessError::UnsupportedSchemaVersion(
                proof.schema_version,
            ));
        }
        if proof.pack_id != PRODUCT_EXPORT_PACK_ID {
            return Err(ExportReadinessError::UnsupportedPackId(
                proof.pack_id.clone(),
            ));
        }

        let boundary = ProductExportBoundary::parse(&proof.boundary)?;
        let export_role = ProductExportRole::parse(&proof.export_role)?;
        let export_hash = proof
            .audio_artifact_sha256
            .get(export_role.as_str())
            .ok_or(ExportReadinessError::MissingExportHash(export_role))?;
        if export_hash != &proof.export_sha256 {
            return Err(ExportReadinessError::MismatchedExportHash {
                export_role,
                proof_hash: proof.export_sha256.clone(),
                artifact_hash: export_hash.clone(),
            });
        }

        Ok(Self {
            schema: EXPORT_READINESS_CONTRACT_SCHEMA.to_owned(),
            status: ExportReadinessStatus::Reproducible,
            proof_schema: proof.schema.clone(),
            export_scope: ExportScope::ProductMix,
            boundary,
            pack_id: proof.pack_id.clone(),
            export_role,
            export_artifact: proof.export_artifact.clone(),
            source_sha256: proof.source_sha256.clone(),
            export_sha256: proof.export_sha256.clone(),
            normalized_manifest_sha256: proof.normalized_manifest_sha256.clone(),
            unsupported_scopes: default_unsupported_export_scopes(),
        })
    }

    #[must_use]
    pub fn unsupported_scope_labels(&self) -> Vec<&'static str> {
        self.unsupported_scopes
            .iter()
            .map(|scope| scope.musician_label())
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportReadinessError {
    UnsupportedSchema(String),
    UnsupportedSchemaVersion(u32),
    UnsupportedPackId(String),
    UnsupportedBoundary(String),
    UnsupportedExportRole(String),
    MissingExportHash(ProductExportRole),
    MismatchedExportHash {
        export_role: ProductExportRole,
        proof_hash: String,
        artifact_hash: String,
    },
}

#[must_use]
pub fn default_unsupported_export_scopes() -> Vec<UnsupportedExportScope> {
    vec![
        UnsupportedExportScope::StemPackage,
        UnsupportedExportScope::LiveRecording,
        UnsupportedExportScope::DawExport,
        UnsupportedExportScope::HostAudioSoak,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_proof() -> ProductExportReproducibilityProof {
        serde_json::from_str(include_str!(
            "../tests/fixtures/export_readiness/product_export_proof.json"
        ))
        .expect("parse product export proof fixture")
    }

    #[test]
    fn builds_readiness_contract_from_product_export_proof_fixture() {
        let proof = fixture_proof();
        let contract = ExportReadinessContract::from_product_export_proof(&proof)
            .expect("build export readiness contract");

        assert_eq!(contract.schema, EXPORT_READINESS_CONTRACT_SCHEMA);
        assert_eq!(contract.status, ExportReadinessStatus::Reproducible);
        assert_eq!(
            contract.boundary.as_proof_str(),
            "feral-grid generated-support export"
        );
        assert_eq!(contract.export_scope.as_str(), "product_mix");
        assert_eq!(contract.export_role.as_str(), "full_grid_mix");
        assert_eq!(contract.export_sha256, proof.export_sha256);
        assert_eq!(
            contract.normalized_manifest_sha256,
            proof.normalized_manifest_sha256
        );
    }

    #[test]
    fn contract_keeps_unsupported_export_scope_flags_visible() {
        let contract = ExportReadinessContract::from_product_export_proof(&fixture_proof())
            .expect("build export readiness contract");

        assert_eq!(
            contract.unsupported_scopes,
            vec![
                UnsupportedExportScope::StemPackage,
                UnsupportedExportScope::LiveRecording,
                UnsupportedExportScope::DawExport,
                UnsupportedExportScope::HostAudioSoak,
            ]
        );
        assert_eq!(
            contract.unsupported_scope_labels(),
            vec![
                "stem package export",
                "live recording export",
                "DAW export",
                "host-audio soak"
            ]
        );
    }

    #[test]
    fn rejects_export_proof_with_mismatched_artifact_hash() {
        let mut proof = fixture_proof();
        proof.export_sha256 =
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".into();

        let error = ExportReadinessContract::from_product_export_proof(&proof)
            .expect_err("mismatched export hash should fail");

        assert_eq!(
            error,
            ExportReadinessError::MismatchedExportHash {
                export_role: ProductExportRole::FullGridMix,
                proof_hash: "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                    .into(),
                artifact_hash: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .into(),
            }
        );
    }

    #[test]
    fn rejects_unknown_export_boundary_instead_of_creating_shadow_truth() {
        let mut proof = fixture_proof();
        proof.boundary = "stem package export".into();

        let error = ExportReadinessContract::from_product_export_proof(&proof)
            .expect_err("unknown boundary should fail");

        assert_eq!(
            error,
            ExportReadinessError::UnsupportedBoundary("stem package export".into())
        );
    }

    #[test]
    fn rejects_unknown_pack_id_instead_of_generalizing_the_export_boundary() {
        let mut proof = fixture_proof();
        proof.pack_id = "future-stem-package".into();

        let error = ExportReadinessContract::from_product_export_proof(&proof)
            .expect_err("unknown pack should fail");

        assert_eq!(
            error,
            ExportReadinessError::UnsupportedPackId("future-stem-package".into())
        );
    }
}
