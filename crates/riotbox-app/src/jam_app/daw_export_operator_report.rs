use std::path::{Path, PathBuf};

use riotbox_core::{
    export_readiness::ExportScope,
    ids::ExportReceiptId,
    session::{
        ArrangementExportPlacementReadinessReport, DawTempoMapReadinessReport, ExportReceiptState,
        SessionFile,
    },
};
use serde::Serialize;

use super::daw_export_proof_gates::{
    DawExportProofGateStatus, DawExportProofGatesSummary, default_daw_export_release_blockers,
    proof_gates_summary, release_blockers_for_receipt,
};
use super::product_export::{
    ExportReceiptArtifactPreflightError, preflight_export_receipt_artifacts,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DawExportOperatorReadinessReport {
    pub status: DawExportOperatorReadinessStatus,
    pub ready_for_next_gate: bool,
    pub writes_files: bool,
    pub developer_proof_status: DawExportDeveloperProofStatus,
    pub musician_export_readiness: &'static str,
    pub release_blockers: Vec<DawExportReleaseBlocker>,
    pub proof_gates: DawExportProofGatesSummary,
    pub proof_stack: DawExportProofStackSummary,
    pub readiness_blockers: Vec<DawExportReadinessBlocker>,
    pub daw_session_receipt_count: usize,
    pub receipt: Option<DawExportReceiptSummary>,
    pub arrangement_placement_readiness: Option<ArrangementExportPlacementReadinessReport>,
    pub daw_tempo_map_readiness: Option<DawTempoMapReadinessReport>,
    pub artifact_preflight: DawExportArtifactPreflightSummary,
}

impl DawExportOperatorReadinessReport {
    #[must_use]
    pub fn blocked_without_receipt(daw_session_receipt_count: usize) -> Self {
        Self {
            status: DawExportOperatorReadinessStatus::Blocked,
            ready_for_next_gate: false,
            writes_files: false,
            developer_proof_status: DawExportDeveloperProofStatus::NoDawSessionReceipt,
            musician_export_readiness: "not_final_daw_export_workflow",
            release_blockers: default_daw_export_release_blockers(),
            proof_gates: DawExportProofGatesSummary::missing(),
            proof_stack: DawExportProofStackSummary::missing_receipt(),
            readiness_blockers: vec![DawExportReadinessBlocker::NoDawSessionReceipt],
            daw_session_receipt_count,
            receipt: None,
            arrangement_placement_readiness: None,
            daw_tempo_map_readiness: None,
            artifact_preflight: DawExportArtifactPreflightSummary::blocked(
                DawExportArtifactPreflightBlocker::NoDawSessionReceipt,
            ),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawExportOperatorReadinessStatus {
    Blocked,
    ReadyForWriter,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawExportDeveloperProofStatus {
    NoDawSessionReceipt,
    ReceiptBlocked,
    ReadyForWriter,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DawExportProofStackSummary {
    pub status: DawExportProofStackStatus,
    pub all_required_proofs_passed: bool,
    pub missing_layers: Vec<DawExportProofLayer>,
}

impl DawExportProofStackSummary {
    #[must_use]
    pub fn missing_receipt() -> Self {
        Self {
            status: DawExportProofStackStatus::MissingReceipt,
            all_required_proofs_passed: false,
            missing_layers: DawExportProofLayer::all_required(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawExportProofStackStatus {
    MissingReceipt,
    Partial,
    CompleteDeveloperProofOnly,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawExportProofLayer {
    JsonPackageIntegrity,
    WriterProof,
    HostImportProof,
    AudibleOutputProof,
}

impl DawExportProofLayer {
    #[must_use]
    pub fn all_required() -> Vec<Self> {
        vec![
            Self::JsonPackageIntegrity,
            Self::WriterProof,
            Self::HostImportProof,
            Self::AudibleOutputProof,
        ]
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawExportReleaseBlocker {
    DeveloperProofOnly,
    DawWriterMissing,
    DawHostImportProofMissing,
    AudibleOutputProofMissing,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawExportReadinessBlocker {
    NoDawSessionReceipt,
    UnsupportedCommandBoundary,
    ArrangementPlacementBlocked,
    DawTempoMapBlocked,
    MissingArtifactIdentity,
    MissingLocalFiles,
    UnreadableLocalFiles,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DawExportReceiptSummary {
    pub receipt_id: ExportReceiptId,
    pub created_by_action: u64,
    pub export_scope: ExportScope,
    pub pack_id: String,
    pub export_role: riotbox_core::export_readiness::ProductExportRole,
    pub export_boundary: riotbox_core::export_readiness::ProductExportBoundary,
    pub artifact_path: String,
    pub proof_path: String,
    pub manifest_path: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DawExportArtifactPreflightSummary {
    pub status: DawExportArtifactPreflightStatus,
    pub blockers: Vec<DawExportArtifactPreflightBlocker>,
    pub missing_local_files: Vec<PathBuf>,
    pub unreadable_local_files: Vec<PathBuf>,
}

impl DawExportArtifactPreflightSummary {
    #[must_use]
    pub fn ready() -> Self {
        Self {
            status: DawExportArtifactPreflightStatus::Ready,
            blockers: Vec::new(),
            missing_local_files: Vec::new(),
            unreadable_local_files: Vec::new(),
        }
    }

    #[must_use]
    pub fn blocked(blocker: DawExportArtifactPreflightBlocker) -> Self {
        Self {
            status: DawExportArtifactPreflightStatus::Blocked,
            blockers: vec![blocker],
            missing_local_files: Vec::new(),
            unreadable_local_files: Vec::new(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawExportArtifactPreflightStatus {
    Ready,
    Blocked,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawExportArtifactPreflightBlocker {
    NoDawSessionReceipt,
    ArrangementPlacementBlocked,
    DawTempoMapBlocked,
    MissingArtifactIdentity,
    MissingLocalFiles,
    UnreadableLocalFiles,
}

#[must_use]
pub fn daw_export_operator_readiness_report(
    session: &SessionFile,
    base_dir: Option<&Path>,
) -> DawExportOperatorReadinessReport {
    let daw_session_receipt_count = session
        .export_receipts
        .iter()
        .filter(|receipt| receipt.export_scope == ExportScope::DawSession)
        .count();
    let Some(receipt) = session
        .export_receipts
        .iter()
        .rev()
        .find(|receipt| receipt.export_scope == ExportScope::DawSession)
    else {
        return DawExportOperatorReadinessReport::blocked_without_receipt(
            daw_session_receipt_count,
        );
    };

    report_for_receipt(receipt, base_dir, daw_session_receipt_count)
}

fn report_for_receipt(
    receipt: &ExportReceiptState,
    base_dir: Option<&Path>,
    daw_session_receipt_count: usize,
) -> DawExportOperatorReadinessReport {
    let arrangement_report = receipt.arrangement_export_placement_report();
    let tempo_map_report = receipt.daw_tempo_map_report();
    let artifact_preflight = artifact_preflight_summary(receipt, base_dir);
    let mut readiness_blockers = Vec::new();

    if receipt
        .unsupported_scopes
        .contains(&riotbox_core::export_readiness::UnsupportedExportScope::DawExport)
    {
        readiness_blockers.push(DawExportReadinessBlocker::UnsupportedCommandBoundary);
    }
    if !arrangement_report.ready() {
        readiness_blockers.push(DawExportReadinessBlocker::ArrangementPlacementBlocked);
    }
    if !tempo_map_report.ready() {
        readiness_blockers.push(DawExportReadinessBlocker::DawTempoMapBlocked);
    }
    for blocker in &artifact_preflight.blockers {
        match blocker {
            DawExportArtifactPreflightBlocker::NoDawSessionReceipt => {
                readiness_blockers.push(DawExportReadinessBlocker::NoDawSessionReceipt);
            }
            DawExportArtifactPreflightBlocker::ArrangementPlacementBlocked => {
                push_unique(
                    &mut readiness_blockers,
                    DawExportReadinessBlocker::ArrangementPlacementBlocked,
                );
            }
            DawExportArtifactPreflightBlocker::DawTempoMapBlocked => {
                push_unique(
                    &mut readiness_blockers,
                    DawExportReadinessBlocker::DawTempoMapBlocked,
                );
            }
            DawExportArtifactPreflightBlocker::MissingArtifactIdentity => {
                readiness_blockers.push(DawExportReadinessBlocker::MissingArtifactIdentity);
            }
            DawExportArtifactPreflightBlocker::MissingLocalFiles => {
                readiness_blockers.push(DawExportReadinessBlocker::MissingLocalFiles);
            }
            DawExportArtifactPreflightBlocker::UnreadableLocalFiles => {
                readiness_blockers.push(DawExportReadinessBlocker::UnreadableLocalFiles);
            }
        }
    }
    readiness_blockers.dedup();

    let ready_for_next_gate = readiness_blockers.is_empty();
    let proof_gates = proof_gates_summary(receipt);
    let proof_stack = daw_export_proof_stack_summary(&proof_gates);
    DawExportOperatorReadinessReport {
        status: if ready_for_next_gate {
            DawExportOperatorReadinessStatus::ReadyForWriter
        } else {
            DawExportOperatorReadinessStatus::Blocked
        },
        ready_for_next_gate,
        writes_files: false,
        developer_proof_status: if ready_for_next_gate {
            DawExportDeveloperProofStatus::ReadyForWriter
        } else {
            DawExportDeveloperProofStatus::ReceiptBlocked
        },
        musician_export_readiness: "not_final_daw_export_workflow",
        release_blockers: release_blockers_for_receipt(receipt),
        proof_gates,
        proof_stack,
        readiness_blockers,
        daw_session_receipt_count,
        receipt: Some(receipt_summary(receipt)),
        arrangement_placement_readiness: Some(arrangement_report),
        daw_tempo_map_readiness: Some(tempo_map_report),
        artifact_preflight,
    }
}

pub(crate) fn daw_export_proof_stack_summary(
    proof_gates: &DawExportProofGatesSummary,
) -> DawExportProofStackSummary {
    let mut missing_layers = Vec::new();
    if proof_gates.json_package_integrity.status != DawExportProofGateStatus::Passed {
        missing_layers.push(DawExportProofLayer::JsonPackageIntegrity);
    }
    if proof_gates.writer_proof.status != DawExportProofGateStatus::Passed {
        missing_layers.push(DawExportProofLayer::WriterProof);
    }
    if proof_gates.host_import_proof.status != DawExportProofGateStatus::Passed {
        missing_layers.push(DawExportProofLayer::HostImportProof);
    }
    if proof_gates.audible_output_proof.status != DawExportProofGateStatus::Passed {
        missing_layers.push(DawExportProofLayer::AudibleOutputProof);
    }
    let all_required_proofs_passed = missing_layers.is_empty();

    DawExportProofStackSummary {
        status: if all_required_proofs_passed {
            DawExportProofStackStatus::CompleteDeveloperProofOnly
        } else {
            DawExportProofStackStatus::Partial
        },
        all_required_proofs_passed,
        missing_layers,
    }
}

fn artifact_preflight_summary(
    receipt: &ExportReceiptState,
    base_dir: Option<&Path>,
) -> DawExportArtifactPreflightSummary {
    match preflight_export_receipt_artifacts(receipt, base_dir) {
        Ok(_) => DawExportArtifactPreflightSummary::ready(),
        Err(error) => artifact_preflight_error_summary(error),
    }
}

fn artifact_preflight_error_summary(
    error: ExportReceiptArtifactPreflightError,
) -> DawExportArtifactPreflightSummary {
    match error {
        ExportReceiptArtifactPreflightError::MissingArtifactPath { .. }
        | ExportReceiptArtifactPreflightError::MissingProofPath { .. }
        | ExportReceiptArtifactPreflightError::MissingArtifactSetPath { .. }
        | ExportReceiptArtifactPreflightError::MissingSessionFileSet { .. } => {
            DawExportArtifactPreflightSummary::blocked(
                DawExportArtifactPreflightBlocker::MissingArtifactIdentity,
            )
        }
        ExportReceiptArtifactPreflightError::MissingExportArtifact { path, .. }
        | ExportReceiptArtifactPreflightError::MissingProofArtifact { path, .. }
        | ExportReceiptArtifactPreflightError::MissingArtifactSetArtifact { path, .. } => {
            let mut summary = DawExportArtifactPreflightSummary::blocked(
                DawExportArtifactPreflightBlocker::MissingLocalFiles,
            );
            summary.missing_local_files.push(path);
            summary
        }
        ExportReceiptArtifactPreflightError::NotFile { path, .. }
        | ExportReceiptArtifactPreflightError::UnreadableArtifact { path, .. }
        | ExportReceiptArtifactPreflightError::ArtifactSetNotFile { path, .. }
        | ExportReceiptArtifactPreflightError::UnreadableArtifactSetArtifact { path, .. } => {
            let mut summary = DawExportArtifactPreflightSummary::blocked(
                DawExportArtifactPreflightBlocker::UnreadableLocalFiles,
            );
            summary.unreadable_local_files.push(path);
            summary
        }
        ExportReceiptArtifactPreflightError::ArrangementPlacementBlocked { .. } => {
            DawExportArtifactPreflightSummary::blocked(
                DawExportArtifactPreflightBlocker::ArrangementPlacementBlocked,
            )
        }
        ExportReceiptArtifactPreflightError::DawTempoMapBlocked { .. } => {
            DawExportArtifactPreflightSummary::blocked(
                DawExportArtifactPreflightBlocker::DawTempoMapBlocked,
            )
        }
    }
}

fn receipt_summary(receipt: &ExportReceiptState) -> DawExportReceiptSummary {
    DawExportReceiptSummary {
        receipt_id: receipt.receipt_id.clone(),
        created_by_action: receipt.created_by_action.0,
        export_scope: receipt.export_scope,
        pack_id: receipt.pack_id.clone(),
        export_role: receipt.export_role,
        export_boundary: receipt.export_boundary,
        artifact_path: receipt.artifact_path.clone(),
        proof_path: receipt.proof_path.clone(),
        manifest_path: receipt.manifest_path.clone(),
    }
}

fn push_unique<T: PartialEq>(values: &mut Vec<T>, value: T) {
    if !values.contains(&value) {
        values.push(value);
    }
}
