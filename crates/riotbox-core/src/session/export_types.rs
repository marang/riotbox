use serde::{Deserialize, Serialize};

use crate::{
    TimestampMs,
    export_readiness::{
        ExportReadinessContract, ExportReadinessStatus, ExportScope, ProductExportBoundary,
        ProductExportRole, UnsupportedExportScope, default_export_scope,
        default_product_export_pack_id,
    },
    ids::{ActionId, CaptureId, ExportReceiptId, SourceId},
    source_graph::SourceGraphVersion,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportReceiptState {
    pub receipt_id: ExportReceiptId,
    pub created_by_action: ActionId,
    pub created_at: TimestampMs,
    #[serde(default = "default_export_scope")]
    pub export_scope: ExportScope,
    #[serde(default = "default_product_export_pack_id")]
    pub pack_id: String,
    pub export_role: ProductExportRole,
    pub export_boundary: ProductExportBoundary,
    pub artifact_path: String,
    pub proof_path: String,
    #[serde(default)]
    pub manifest_path: Option<String>,
    pub export_hash: String,
    pub normalized_manifest_hash: String,
    #[serde(default)]
    pub artifact_set: Vec<ExportArtifactSetEntry>,
    #[serde(default)]
    pub qa_gates: Vec<ExportReceiptQaGateResult>,
    pub readiness_status: ExportReadinessStatus,
    pub unsupported_scopes: Vec<UnsupportedExportScope>,
}

impl ExportReceiptState {
    #[must_use]
    pub fn from_readiness_contract(
        created_by_action: ActionId,
        created_at: TimestampMs,
        contract: &ExportReadinessContract,
        artifact_path: impl Into<String>,
        proof_path: impl Into<String>,
        manifest_path: Option<String>,
    ) -> Self {
        let artifact_path = artifact_path.into();
        Self {
            receipt_id: ExportReceiptId::new(format!("export-receipt-{created_by_action}")),
            created_by_action,
            created_at,
            export_scope: contract.export_scope,
            pack_id: contract.pack_id.clone(),
            export_role: contract.export_role,
            export_boundary: contract.boundary,
            artifact_path: artifact_path.clone(),
            proof_path: proof_path.into(),
            manifest_path,
            export_hash: contract.export_sha256.clone(),
            normalized_manifest_hash: contract.normalized_manifest_sha256.clone(),
            artifact_set: vec![ExportArtifactSetEntry::product_mix(
                artifact_path,
                contract.export_sha256.clone(),
                Some(contract.normalized_manifest_sha256.clone()),
            )],
            qa_gates: vec![ExportReceiptQaGateResult::product_export_reproducibility()],
            readiness_status: contract.status,
            unsupported_scopes: contract.unsupported_scopes.clone(),
        }
    }

    #[must_use]
    pub fn artifact_set_or_legacy(&self) -> Vec<ExportArtifactSetEntry> {
        if self.artifact_set.is_empty() && !self.artifact_path.trim().is_empty() {
            return vec![ExportArtifactSetEntry::product_mix(
                self.artifact_path.clone(),
                self.export_hash.clone(),
                Some(self.normalized_manifest_hash.clone()),
            )];
        }

        self.artifact_set.clone()
    }

    pub fn attach_artifact_source_graph_ref(
        &mut self,
        role: ExportArtifactRole,
        source_graph_ref: ExportArtifactSourceGraphRef,
    ) {
        for artifact in &mut self.artifact_set {
            if artifact.role == role {
                artifact.source_graph_ref = Some(source_graph_ref.clone());
            }
        }
    }

    pub fn attach_artifact_timing_grid_ref(
        &mut self,
        role: ExportArtifactRole,
        timing_grid_ref: ExportArtifactTimingGridRef,
    ) {
        for artifact in &mut self.artifact_set {
            if artifact.role == role {
                artifact.timing_grid_ref = Some(timing_grid_ref.clone());
            }
        }
    }
}

pub const PRODUCT_EXPORT_REPRODUCIBILITY_QA_GATE_ID: &str = "product_export_reproducibility_smoke";
pub const STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID: &str = "stem_package_artifact_set_evidence";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportReceiptQaGateResult {
    pub gate_id: String,
    pub status: ExportReceiptQaGateStatus,
    #[serde(default)]
    pub artifact_roles: Vec<ExportArtifactRole>,
    #[serde(default)]
    pub summary: Option<String>,
}

impl ExportReceiptQaGateResult {
    #[must_use]
    pub fn product_export_reproducibility() -> Self {
        Self {
            gate_id: PRODUCT_EXPORT_REPRODUCIBILITY_QA_GATE_ID.into(),
            status: ExportReceiptQaGateStatus::Passed,
            artifact_roles: vec![ExportArtifactRole::FullGridMix],
            summary: Some("product export proof and artifact hash accepted".into()),
        }
    }

    #[must_use]
    pub fn stem_package_artifact_set_evidence(
        report: &crate::export_qa::StemPackageArtifactSetQaReport,
    ) -> Self {
        let status = if report.status == crate::export_qa::StemPackageArtifactSetQaStatus::Failed {
            ExportReceiptQaGateStatus::Failed
        } else if report.deferred_checks.is_empty() {
            ExportReceiptQaGateStatus::Passed
        } else {
            ExportReceiptQaGateStatus::Deferred
        };

        Self {
            gate_id: STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID.into(),
            status,
            artifact_roles: report.claimed_roles.clone(),
            summary: Some(stem_package_artifact_set_summary(report)),
        }
    }
}

fn stem_package_artifact_set_summary(
    report: &crate::export_qa::StemPackageArtifactSetQaReport,
) -> String {
    match report.status {
        crate::export_qa::StemPackageArtifactSetQaStatus::PassedStructureOnly => format!(
            "stem package artifact-set structure accepted for {} claimed stem role(s); {} deferred QA check(s) remain",
            report.claimed_roles.len(),
            report.deferred_checks.len()
        ),
        crate::export_qa::StemPackageArtifactSetQaStatus::Failed => format!(
            "stem package artifact-set evidence failed with {} failure(s); {} deferred QA check(s) remain",
            report.failures.len(),
            report.deferred_checks.len()
        ),
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportReceiptQaGateStatus {
    Passed,
    Failed,
    Deferred,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportArtifactSetEntry {
    pub role: ExportArtifactRole,
    pub location: ExportArtifactLocation,
    pub media_type: ExportArtifactMediaType,
    pub sha256: String,
    #[serde(default)]
    pub normalized_manifest_hash: Option<String>,
    #[serde(default)]
    pub source_graph_ref: Option<ExportArtifactSourceGraphRef>,
    #[serde(default)]
    pub timing_grid_ref: Option<ExportArtifactTimingGridRef>,
    #[serde(default)]
    pub source_capture_refs: Vec<CaptureId>,
    #[serde(default)]
    pub lineage_capture_refs: Vec<CaptureId>,
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

impl ExportArtifactSetEntry {
    #[must_use]
    pub fn product_mix(
        path: impl Into<String>,
        sha256: impl Into<String>,
        normalized_manifest_hash: Option<String>,
    ) -> Self {
        Self {
            role: ExportArtifactRole::FullGridMix,
            location: ExportArtifactLocation::LocalPath { path: path.into() },
            media_type: ExportArtifactMediaType::AudioWav,
            sha256: sha256.into(),
            normalized_manifest_hash,
            source_graph_ref: None,
            timing_grid_ref: None,
            source_capture_refs: Vec::new(),
            lineage_capture_refs: Vec::new(),
            fallback_comparison: None,
            audio_metrics: None,
            sample_rate_hz: None,
            channel_count: None,
            duration_ms: None,
        }
    }

    #[must_use]
    pub fn product_export_proof(path: impl Into<String>, sha256: impl Into<String>) -> Self {
        Self::local_json_artifact(ExportArtifactRole::ProductExportProof, path, sha256)
    }

    #[must_use]
    pub fn stem_package_proof(path: impl Into<String>, sha256: impl Into<String>) -> Self {
        Self::local_json_artifact(ExportArtifactRole::ProductExportProof, path, sha256)
    }

    #[must_use]
    pub fn export_manifest(path: impl Into<String>, sha256: impl Into<String>) -> Self {
        Self::local_json_artifact(ExportArtifactRole::ExportManifest, path, sha256)
    }

    #[must_use]
    pub fn location_identity(&self) -> &str {
        match &self.location {
            ExportArtifactLocation::LocalPath { path }
            | ExportArtifactLocation::Uri { uri: path } => path,
        }
    }

    fn local_json_artifact(
        role: ExportArtifactRole,
        path: impl Into<String>,
        sha256: impl Into<String>,
    ) -> Self {
        Self {
            role,
            location: ExportArtifactLocation::LocalPath { path: path.into() },
            media_type: ExportArtifactMediaType::Json,
            sha256: sha256.into(),
            normalized_manifest_hash: None,
            source_graph_ref: None,
            timing_grid_ref: None,
            source_capture_refs: Vec::new(),
            lineage_capture_refs: Vec::new(),
            fallback_comparison: None,
            audio_metrics: None,
            sample_rate_hz: None,
            channel_count: None,
            duration_ms: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportArtifactSourceGraphRef {
    pub source_id: SourceId,
    pub graph_version: SourceGraphVersion,
    pub graph_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportArtifactTimingGridRef {
    pub source_id: SourceId,
    #[serde(default)]
    pub hypothesis_id: Option<String>,
    pub confirmed_by_action: ActionId,
    pub confirmed_at: TimestampMs,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportArtifactFallbackComparisonEvidence {
    pub comparison_kind: ExportArtifactFallbackComparisonKind,
    pub reference_identity: String,
    #[serde(default)]
    pub rms_difference_micros: Option<u32>,
    #[serde(default)]
    pub normalized_correlation_micros: Option<i32>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportArtifactFallbackComparisonKind {
    SourceVsFallback,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportArtifactRole {
    FullGridMix,
    StemDrums,
    StemBass,
    StemMusic,
    StemVocals,
    ProductExportProof,
    ExportManifest,
}

impl ExportArtifactRole {
    #[must_use]
    pub const fn is_stem_role(self) -> bool {
        matches!(
            self,
            Self::StemDrums | Self::StemBass | Self::StemMusic | Self::StemVocals
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExportArtifactLocation {
    LocalPath { path: String },
    Uri { uri: String },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportArtifactMediaType {
    AudioWav,
    Json,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportArtifactAudioMetrics {
    #[serde(default)]
    pub peak_milli_dbfs: Option<i32>,
    #[serde(default)]
    pub rms_milli_dbfs: Option<i32>,
    #[serde(default)]
    pub peak_amplitude_micros: Option<u32>,
    #[serde(default)]
    pub rms_amplitude_micros: Option<u32>,
    #[serde(default)]
    pub silent_frame_count: Option<u64>,
    #[serde(default)]
    pub total_frame_count: Option<u64>,
}

#[cfg(test)]
#[path = "export_types_tests.rs"]
mod export_types_tests;
