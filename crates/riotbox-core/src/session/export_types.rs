use serde::{Deserialize, Serialize};

use crate::{
    TimestampMs,
    export_readiness::{
        ExportReadinessContract, ExportReadinessStatus, ProductExportBoundary, ProductExportRole,
        UnsupportedExportScope,
    },
    ids::{ActionId, CaptureId, ExportReceiptId, SourceId},
    source_graph::SourceGraphVersion,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportReceiptState {
    pub receipt_id: ExportReceiptId,
    pub created_by_action: ActionId,
    pub created_at: TimestampMs,
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
            )],
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
            )];
        }

        self.artifact_set.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportArtifactSetEntry {
    pub role: ExportArtifactRole,
    pub location: ExportArtifactLocation,
    pub media_type: ExportArtifactMediaType,
    pub sha256: String,
    #[serde(default)]
    pub source_graph_ref: Option<ExportArtifactSourceGraphRef>,
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
    pub fn product_mix(path: impl Into<String>, sha256: impl Into<String>) -> Self {
        Self {
            role: ExportArtifactRole::FullGridMix,
            location: ExportArtifactLocation::LocalPath { path: path.into() },
            media_type: ExportArtifactMediaType::AudioWav,
            sha256: sha256.into(),
            source_graph_ref: None,
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
    pub fn location_identity(&self) -> &str {
        match &self.location {
            ExportArtifactLocation::LocalPath { path }
            | ExportArtifactLocation::Uri { uri: path } => path,
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
mod tests {
    use super::*;

    use crate::{
        export_readiness::{EXPORT_READINESS_CONTRACT_SCHEMA, PRODUCT_EXPORT_PROOF_SCHEMA},
        session::SessionFile,
    };

    #[test]
    fn export_receipts_roundtrip_with_session_file() {
        let contract = ExportReadinessContract {
            schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
            status: ExportReadinessStatus::Reproducible,
            proof_schema: PRODUCT_EXPORT_PROOF_SCHEMA.into(),
            boundary: ProductExportBoundary::FeralGridGeneratedSupport,
            pack_id: crate::export_readiness::PRODUCT_EXPORT_PACK_ID.into(),
            export_role: ProductExportRole::FullGridMix,
            export_artifact: "run-a/full_grid_mix.wav".into(),
            source_sha256: "eeee".into(),
            export_sha256: "aaaa".into(),
            normalized_manifest_sha256: "dddd".into(),
            unsupported_scopes: vec![UnsupportedExportScope::StemPackage],
        };
        let mut session = SessionFile::new("session-export", "0.1.0", "2026-05-31T00:00:00Z");
        session
            .export_receipts
            .push(ExportReceiptState::from_readiness_contract(
                ActionId(7),
                900,
                &contract,
                "exports/full_grid_mix.wav",
                "exports/product_export_proof.json",
                Some("exports/manifest.json".into()),
            ));

        let json = serde_json::to_string_pretty(&session).expect("serialize session");
        let roundtrip: SessionFile = serde_json::from_str(&json).expect("deserialize session");

        assert_eq!(roundtrip.export_receipts.len(), 1);
        let receipt = &roundtrip.export_receipts[0];
        assert_eq!(
            receipt.receipt_id,
            ExportReceiptId::from("export-receipt-a-0007")
        );
        assert_eq!(receipt.created_by_action, ActionId(7));
        assert_eq!(receipt.export_role, ProductExportRole::FullGridMix);
        assert_eq!(
            receipt.export_boundary,
            ProductExportBoundary::FeralGridGeneratedSupport
        );
        assert_eq!(
            receipt.unsupported_scopes,
            vec![UnsupportedExportScope::StemPackage]
        );
        assert_eq!(
            receipt.artifact_set,
            vec![ExportArtifactSetEntry::product_mix(
                "exports/full_grid_mix.wav",
                "aaaa"
            )]
        );
    }

    #[test]
    fn missing_export_receipts_default_to_empty_for_older_sessions() {
        let session = SessionFile::new("old-session", "0.1.0", "2026-05-31T00:00:00Z");
        let mut json = serde_json::to_value(&session).expect("serialize session");
        json.as_object_mut()
            .expect("session json object")
            .remove("export_receipts");

        let session: SessionFile = serde_json::from_value(json).expect("deserialize older session");

        assert!(session.export_receipts.is_empty());
    }

    #[test]
    fn missing_artifact_set_defaults_to_empty_for_older_receipts() {
        let contract = ExportReadinessContract {
            schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
            status: ExportReadinessStatus::Reproducible,
            proof_schema: PRODUCT_EXPORT_PROOF_SCHEMA.into(),
            boundary: ProductExportBoundary::FeralGridGeneratedSupport,
            pack_id: crate::export_readiness::PRODUCT_EXPORT_PACK_ID.into(),
            export_role: ProductExportRole::FullGridMix,
            export_artifact: "run-a/full_grid_mix.wav".into(),
            source_sha256: "eeee".into(),
            export_sha256: "aaaa".into(),
            normalized_manifest_sha256: "dddd".into(),
            unsupported_scopes: vec![UnsupportedExportScope::StemPackage],
        };
        let mut receipt = ExportReceiptState::from_readiness_contract(
            ActionId(7),
            900,
            &contract,
            "exports/full_grid_mix.wav",
            "exports/product_export_proof.json",
            Some("exports/manifest.json".into()),
        );
        receipt.artifact_set.clear();
        let mut json = serde_json::to_value(&receipt).expect("serialize receipt");
        json.as_object_mut()
            .expect("receipt json object")
            .remove("artifact_set");

        let receipt: ExportReceiptState =
            serde_json::from_value(json).expect("deserialize older receipt");

        assert!(receipt.artifact_set.is_empty());
        assert_eq!(
            receipt.artifact_set_or_legacy(),
            vec![ExportArtifactSetEntry::product_mix(
                "exports/full_grid_mix.wav",
                "aaaa"
            )]
        );
    }

    #[test]
    fn artifact_set_entries_roundtrip_optional_audio_metrics() {
        let entry = ExportArtifactSetEntry {
            role: ExportArtifactRole::StemDrums,
            location: ExportArtifactLocation::LocalPath {
                path: "exports/stems/drums.wav".into(),
            },
            media_type: ExportArtifactMediaType::AudioWav,
            sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            source_graph_ref: None,
            source_capture_refs: Vec::new(),
            lineage_capture_refs: Vec::new(),
            fallback_comparison: None,
            audio_metrics: Some(ExportArtifactAudioMetrics {
                peak_milli_dbfs: Some(-120),
                rms_milli_dbfs: Some(-6_000),
                peak_amplitude_micros: Some(986_000),
                rms_amplitude_micros: Some(125_000),
                silent_frame_count: Some(0),
                total_frame_count: Some(96_000),
            }),
            sample_rate_hz: Some(48_000),
            channel_count: Some(2),
            duration_ms: Some(2_000),
        };

        let json = serde_json::to_string_pretty(&entry).expect("serialize artifact entry");
        let roundtrip: ExportArtifactSetEntry =
            serde_json::from_str(&json).expect("deserialize artifact entry");

        assert_eq!(roundtrip, entry);
    }

    #[test]
    fn artifact_set_entries_roundtrip_source_and_capture_lineage_refs() {
        let entry = ExportArtifactSetEntry {
            role: ExportArtifactRole::StemDrums,
            location: ExportArtifactLocation::LocalPath {
                path: "exports/stems/drums.wav".into(),
            },
            media_type: ExportArtifactMediaType::AudioWav,
            sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            source_graph_ref: Some(ExportArtifactSourceGraphRef {
                source_id: SourceId::from("src-1"),
                graph_version: SourceGraphVersion::V1,
                graph_hash: "graph-hash-1".into(),
            }),
            source_capture_refs: vec![CaptureId::from("cap-source")],
            lineage_capture_refs: vec![CaptureId::from("cap-root"), CaptureId::from("cap-print")],
            fallback_comparison: Some(ExportArtifactFallbackComparisonEvidence {
                comparison_kind: ExportArtifactFallbackComparisonKind::SourceVsFallback,
                reference_identity: "fallback://stem-drums".into(),
                rms_difference_micros: Some(125_000),
                normalized_correlation_micros: Some(420_000),
            }),
            audio_metrics: None,
            sample_rate_hz: Some(48_000),
            channel_count: Some(2),
            duration_ms: Some(2_000),
        };

        let json = serde_json::to_string_pretty(&entry).expect("serialize artifact entry");
        let roundtrip: ExportArtifactSetEntry =
            serde_json::from_str(&json).expect("deserialize artifact entry");

        assert_eq!(roundtrip, entry);
    }

    #[test]
    fn missing_optional_evidence_defaults_for_older_artifact_entries() {
        let mut json = serde_json::json!({
            "role": "stem_drums",
            "location": {
                "kind": "local_path",
                "path": "exports/stems/drums.wav"
            },
            "media_type": "audio_wav",
            "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        });
        json.as_object_mut()
            .expect("entry json object")
            .remove("audio_metrics");

        let entry: ExportArtifactSetEntry =
            serde_json::from_value(json).expect("deserialize older artifact entry");

        assert_eq!(entry.audio_metrics, None);
        assert_eq!(entry.source_graph_ref, None);
        assert!(entry.source_capture_refs.is_empty());
        assert!(entry.lineage_capture_refs.is_empty());
        assert_eq!(entry.fallback_comparison, None);
    }
}
