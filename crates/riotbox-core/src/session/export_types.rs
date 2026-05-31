use serde::{Deserialize, Serialize};

use crate::{
    TimestampMs,
    export_readiness::{
        ExportReadinessContract, ExportReadinessStatus, ProductExportBoundary, ProductExportRole,
        UnsupportedExportScope,
    },
    ids::{ActionId, ExportReceiptId},
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
        Self {
            receipt_id: ExportReceiptId::new(format!("export-receipt-{created_by_action}")),
            created_by_action,
            created_at,
            export_role: contract.export_role,
            export_boundary: contract.boundary,
            artifact_path: artifact_path.into(),
            proof_path: proof_path.into(),
            manifest_path,
            export_hash: contract.export_sha256.clone(),
            normalized_manifest_hash: contract.normalized_manifest_sha256.clone(),
            readiness_status: contract.status,
            unsupported_scopes: contract.unsupported_scopes.clone(),
        }
    }
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
}
