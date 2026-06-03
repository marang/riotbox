use riotbox_core::{
    export_readiness::{
        ARRANGEMENT_DAW_PLACEMENT_PACK_ID, ExportScope, ProductExportBoundary, ProductExportRole,
    },
    session::{DAW_SESSION_JSON_PACKAGE_QA_GATE_ID, ExportReceiptQaGateStatus, SessionFile},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DawSessionExportSurfaceGate {
    pub status: DawSessionExportSurfaceStatus,
    pub blockers: Vec<DawSessionExportSurfaceBlocker>,
}

impl DawSessionExportSurfaceGate {
    #[must_use]
    pub fn disabled_without_receipt() -> Self {
        Self {
            status: DawSessionExportSurfaceStatus::Disabled,
            blockers: vec![
                DawSessionExportSurfaceBlocker::NoDawSessionReceipt,
                DawSessionExportSurfaceBlocker::DeveloperProofOnly,
                DawSessionExportSurfaceBlocker::DawWriterMissing,
                DawSessionExportSurfaceBlocker::DawHostImportProofMissing,
                DawSessionExportSurfaceBlocker::AudibleOutputProofMissing,
            ],
        }
    }

    #[must_use]
    pub fn runnable(&self) -> bool {
        self.status == DawSessionExportSurfaceStatus::Runnable && self.blockers.is_empty()
    }

    #[must_use]
    pub fn musician_summary(&self) -> String {
        if self.runnable() {
            return "DAW session export is ready for musicians".into();
        }

        format!(
            "DAW session export is developer proof only; blockers: {}",
            self.blockers
                .iter()
                .map(|blocker| blocker.musician_label())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DawSessionExportSurfaceStatus {
    Disabled,
    Runnable,
}

impl DawSessionExportSurfaceStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Runnable => "runnable",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DawSessionExportSurfaceBlocker {
    NoDawSessionReceipt,
    DawReceiptReadinessBlocked,
    DawReceiptIdentityMissing,
    JsonPackageEvidenceMissing,
    JsonPackageIntegrityBlocked,
    DeveloperProofOnly,
    DawWriterMissing,
    DawHostImportProofMissing,
    AudibleOutputProofMissing,
}

impl DawSessionExportSurfaceBlocker {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoDawSessionReceipt => "no_daw_session_receipt",
            Self::DawReceiptReadinessBlocked => "receipt_readiness_blocked",
            Self::DawReceiptIdentityMissing => "receipt_identity_missing",
            Self::JsonPackageEvidenceMissing => "json_package_evidence_missing",
            Self::JsonPackageIntegrityBlocked => "json_package_integrity_blocked",
            Self::DeveloperProofOnly => "developer_proof_only",
            Self::DawWriterMissing => "daw_writer_missing",
            Self::DawHostImportProofMissing => "daw_host_import_proof_missing",
            Self::AudibleOutputProofMissing => "audible_output_proof_missing",
        }
    }

    #[must_use]
    pub const fn musician_label(self) -> &'static str {
        match self {
            Self::NoDawSessionReceipt => "DAW session receipt is missing",
            Self::DawReceiptReadinessBlocked => "DAW receipt readiness is still blocked",
            Self::DawReceiptIdentityMissing => {
                "DAW receipt identity is not the arrangement contract"
            }
            Self::JsonPackageEvidenceMissing => "DAW JSON package evidence is missing",
            Self::JsonPackageIntegrityBlocked => "DAW JSON package integrity is blocked",
            Self::DeveloperProofOnly => "DAW export is developer proof only",
            Self::DawWriterMissing => "DAW project/session writer is missing",
            Self::DawHostImportProofMissing => "DAW host import proof is missing",
            Self::AudibleOutputProofMissing => "audible export output proof is missing",
        }
    }
}

#[must_use]
pub fn daw_session_export_surface_gate_for_session(
    session: &SessionFile,
) -> DawSessionExportSurfaceGate {
    let Some(receipt) = session
        .export_receipts
        .iter()
        .rev()
        .find(|receipt| receipt.export_scope == ExportScope::DawSession)
    else {
        return DawSessionExportSurfaceGate::disabled_without_receipt();
    };

    let mut blockers = Vec::new();
    if !receipt.arrangement_export_placement_report().ready()
        || !receipt.daw_tempo_map_report().ready()
    {
        blockers.push(DawSessionExportSurfaceBlocker::DawReceiptReadinessBlocked);
    }
    if receipt.pack_id != ARRANGEMENT_DAW_PLACEMENT_PACK_ID
        || receipt.export_role != ProductExportRole::ArrangementManifest
        || receipt.export_boundary != ProductExportBoundary::ArrangementDawPlacementContractV1
    {
        blockers.push(DawSessionExportSurfaceBlocker::DawReceiptIdentityMissing);
    }

    match receipt
        .qa_gates
        .iter()
        .find(|gate| gate.gate_id == DAW_SESSION_JSON_PACKAGE_QA_GATE_ID)
    {
        Some(gate) if gate.status == ExportReceiptQaGateStatus::Passed => {}
        Some(_) => blockers.push(DawSessionExportSurfaceBlocker::JsonPackageIntegrityBlocked),
        None => blockers.push(DawSessionExportSurfaceBlocker::JsonPackageEvidenceMissing),
    }

    blockers.extend([
        DawSessionExportSurfaceBlocker::DeveloperProofOnly,
        DawSessionExportSurfaceBlocker::DawWriterMissing,
        DawSessionExportSurfaceBlocker::DawHostImportProofMissing,
        DawSessionExportSurfaceBlocker::AudibleOutputProofMissing,
    ]);

    DawSessionExportSurfaceGate {
        status: DawSessionExportSurfaceStatus::Disabled,
        blockers,
    }
}
