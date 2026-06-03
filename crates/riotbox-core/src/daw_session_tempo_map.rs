use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    TimestampMs,
    export_readiness::{
        ExportScope, ProductExportBoundary, ProductExportRole, UnsupportedExportScope,
    },
    ids::{ActionId, ExportReceiptId, SourceId},
    session::{DawTempoMapReadinessBlocker, ExportReceiptState},
};

pub const DAW_SESSION_TEMPO_MAP_SCHEMA_ID: &str = "riotbox.daw_session_tempo_map";
pub const DAW_SESSION_TEMPO_MAP_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DawSessionTempoMap {
    pub schema_id: String,
    pub schema_version: u32,
    pub package_id: String,
    pub export_scope: ExportScope,
    #[serde(default = "default_daw_session_tempo_map_export_role")]
    pub export_role: ProductExportRole,
    #[serde(default = "default_daw_session_tempo_map_boundary")]
    pub export_boundary: ProductExportBoundary,
    pub receipt_id: ExportReceiptId,
    pub created_by_action: ActionId,
    pub source_id: SourceId,
    #[serde(default)]
    pub hypothesis_id: Option<String>,
    pub confirmed_by_action: ActionId,
    pub confirmed_at: TimestampMs,
    pub start_beat: u64,
    pub end_beat: u64,
    pub bpm_micros: u32,
}

impl DawSessionTempoMap {
    pub fn new(input: DawSessionTempoMapInput) -> Result<Self, DawSessionTempoMapError> {
        let package_id = input.package_id;
        if package_id.trim().is_empty() {
            return Err(DawSessionTempoMapError::BlankPackageId);
        }
        if input.export_role != ProductExportRole::ArrangementManifest {
            return Err(DawSessionTempoMapError::UnexpectedExportRole {
                expected: ProductExportRole::ArrangementManifest,
                actual: input.export_role,
            });
        }
        if input.export_boundary != ProductExportBoundary::ArrangementDawPlacementContractV1 {
            return Err(DawSessionTempoMapError::UnexpectedExportBoundary {
                expected: ProductExportBoundary::ArrangementDawPlacementContractV1,
                actual: input.export_boundary,
            });
        }
        if input.source_id.as_str().trim().is_empty() {
            return Err(DawSessionTempoMapError::BlankSourceRef);
        }
        if input.end_beat <= input.start_beat {
            return Err(DawSessionTempoMapError::InvalidBeatRange);
        }
        if input.bpm_micros == 0 {
            return Err(DawSessionTempoMapError::InvalidTempo);
        }

        Ok(Self {
            schema_id: DAW_SESSION_TEMPO_MAP_SCHEMA_ID.into(),
            schema_version: DAW_SESSION_TEMPO_MAP_SCHEMA_VERSION,
            package_id,
            export_scope: ExportScope::DawSession,
            export_role: input.export_role,
            export_boundary: input.export_boundary,
            receipt_id: input.receipt_id,
            created_by_action: input.created_by_action,
            source_id: input.source_id,
            hypothesis_id: input.hypothesis_id,
            confirmed_by_action: input.confirmed_by_action,
            confirmed_at: input.confirmed_at,
            start_beat: input.start_beat,
            end_beat: input.end_beat,
            bpm_micros: input.bpm_micros,
        })
    }

    pub fn from_receipt(
        receipt: &ExportReceiptState,
    ) -> Result<Self, DawSessionTempoMapBuildError> {
        if receipt.export_scope != ExportScope::DawSession {
            return Err(DawSessionTempoMapBuildError::NotDawSessionScope {
                export_scope: receipt.export_scope,
            });
        }
        if receipt.export_boundary != ProductExportBoundary::ArrangementDawPlacementContractV1 {
            return Err(DawSessionTempoMapBuildError::NotDawSessionBoundary {
                export_boundary: receipt.export_boundary,
            });
        }
        if receipt.export_role != ProductExportRole::ArrangementManifest {
            return Err(DawSessionTempoMapBuildError::UnexpectedExportRole {
                export_role: receipt.export_role,
            });
        }
        if receipt
            .unsupported_scopes
            .contains(&UnsupportedExportScope::DawExport)
        {
            return Err(DawSessionTempoMapBuildError::UnsupportedDawExportFlagPresent);
        }

        let report = receipt.daw_tempo_map_report();
        if !report.ready() {
            return Err(DawSessionTempoMapBuildError::DawTempoMapBlocked {
                blockers: report.blockers,
            });
        }
        let tempo_map = receipt.daw_tempo_map_ref.clone().ok_or(
            DawSessionTempoMapBuildError::DawTempoMapBlocked {
                blockers: vec![DawTempoMapReadinessBlocker::MissingTempoMapRef],
            },
        )?;

        Self::new(DawSessionTempoMapInput {
            package_id: receipt.pack_id.clone(),
            export_role: receipt.export_role,
            export_boundary: receipt.export_boundary,
            receipt_id: receipt.receipt_id.clone(),
            created_by_action: receipt.created_by_action,
            source_id: tempo_map.source_id,
            hypothesis_id: tempo_map.hypothesis_id,
            confirmed_by_action: tempo_map.confirmed_by_action,
            confirmed_at: tempo_map.confirmed_at,
            start_beat: tempo_map.start_beat,
            end_beat: tempo_map.end_beat,
            bpm_micros: tempo_map.bpm_micros,
        })
        .map_err(DawSessionTempoMapBuildError::TempoMap)
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
pub struct DawSessionTempoMapInput {
    pub package_id: String,
    pub export_role: ProductExportRole,
    pub export_boundary: ProductExportBoundary,
    pub receipt_id: ExportReceiptId,
    pub created_by_action: ActionId,
    pub source_id: SourceId,
    pub hypothesis_id: Option<String>,
    pub confirmed_by_action: ActionId,
    pub confirmed_at: TimestampMs,
    pub start_beat: u64,
    pub end_beat: u64,
    pub bpm_micros: u32,
}

#[must_use]
pub const fn default_daw_session_tempo_map_export_role() -> ProductExportRole {
    ProductExportRole::ArrangementManifest
}

#[must_use]
pub const fn default_daw_session_tempo_map_boundary() -> ProductExportBoundary {
    ProductExportBoundary::ArrangementDawPlacementContractV1
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DawSessionTempoMapError {
    BlankPackageId,
    UnexpectedExportRole {
        expected: ProductExportRole,
        actual: ProductExportRole,
    },
    UnexpectedExportBoundary {
        expected: ProductExportBoundary,
        actual: ProductExportBoundary,
    },
    BlankSourceRef,
    InvalidBeatRange,
    InvalidTempo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DawSessionTempoMapBuildError {
    NotDawSessionScope {
        export_scope: ExportScope,
    },
    NotDawSessionBoundary {
        export_boundary: ProductExportBoundary,
    },
    UnexpectedExportRole {
        export_role: ProductExportRole,
    },
    UnsupportedDawExportFlagPresent,
    DawTempoMapBlocked {
        blockers: Vec<DawTempoMapReadinessBlocker>,
    },
    TempoMap(DawSessionTempoMapError),
}

#[cfg(test)]
#[path = "daw_session_tempo_map_tests.rs"]
mod daw_session_tempo_map_tests;
