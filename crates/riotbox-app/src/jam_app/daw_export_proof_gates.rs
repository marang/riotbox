use riotbox_core::session::{
    DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID, DAW_SESSION_HOST_IMPORT_QA_GATE_ID,
    DAW_SESSION_JSON_PACKAGE_QA_GATE_ID, DAW_SESSION_WRITER_QA_GATE_ID, ExportArtifactMediaType,
    ExportArtifactRole, ExportReceiptQaGateResult, ExportReceiptQaGateStatus, ExportReceiptState,
};
use serde::Serialize;

use super::daw_export_operator_report::DawExportReleaseBlocker;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DawExportProofGatesSummary {
    pub json_package_integrity: DawExportProofGateSummary,
    pub writer_proof: DawExportProofGateSummary,
    pub host_import_proof: DawExportProofGateSummary,
    pub audible_output_proof: DawExportProofGateSummary,
}

impl DawExportProofGatesSummary {
    #[must_use]
    pub fn missing() -> Self {
        Self {
            json_package_integrity: DawExportProofGateSummary::missing(
                DAW_SESSION_JSON_PACKAGE_QA_GATE_ID,
            ),
            writer_proof: DawExportProofGateSummary::missing(DAW_SESSION_WRITER_QA_GATE_ID),
            host_import_proof: DawExportProofGateSummary::missing(
                DAW_SESSION_HOST_IMPORT_QA_GATE_ID,
            ),
            audible_output_proof: DawExportProofGateSummary::missing(
                DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID,
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DawExportProofGateSummary {
    pub gate_id: &'static str,
    pub status: DawExportProofGateStatus,
    pub summary: Option<String>,
    pub artifact_roles: Vec<ExportArtifactRole>,
    pub artifact_available: bool,
    pub artifacts: Vec<DawExportProofArtifactSummary>,
}

impl DawExportProofGateSummary {
    #[must_use]
    pub fn missing(gate_id: &'static str) -> Self {
        Self {
            gate_id,
            status: DawExportProofGateStatus::Missing,
            summary: None,
            artifact_roles: Vec::new(),
            artifact_available: false,
            artifacts: Vec::new(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawExportProofGateStatus {
    Missing,
    Passed,
    Failed,
    Deferred,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DawExportProofArtifactSummary {
    pub role: ExportArtifactRole,
    pub media_type: ExportArtifactMediaType,
    pub location: String,
    pub sha256: String,
}

pub fn default_daw_export_release_blockers() -> Vec<DawExportReleaseBlocker> {
    vec![
        DawExportReleaseBlocker::DeveloperProofOnly,
        DawExportReleaseBlocker::DawWriterMissing,
        DawExportReleaseBlocker::DawHostImportProofMissing,
        DawExportReleaseBlocker::AudibleOutputProofMissing,
    ]
}

pub fn release_blockers_for_receipt(receipt: &ExportReceiptState) -> Vec<DawExportReleaseBlocker> {
    let mut blockers = vec![DawExportReleaseBlocker::DeveloperProofOnly];
    if !gate_passed(receipt, DAW_SESSION_WRITER_QA_GATE_ID) {
        blockers.push(DawExportReleaseBlocker::DawWriterMissing);
    }
    if !gate_passed(receipt, DAW_SESSION_HOST_IMPORT_QA_GATE_ID) {
        blockers.push(DawExportReleaseBlocker::DawHostImportProofMissing);
    }
    if !gate_passed(receipt, DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID) {
        blockers.push(DawExportReleaseBlocker::AudibleOutputProofMissing);
    }
    blockers
}

pub fn proof_gates_summary(receipt: &ExportReceiptState) -> DawExportProofGatesSummary {
    DawExportProofGatesSummary {
        json_package_integrity: proof_gate_summary(
            receipt,
            DAW_SESSION_JSON_PACKAGE_QA_GATE_ID,
            &[
                ExportArtifactRole::ExportManifest,
                ExportArtifactRole::DawSessionTempoMap,
                ExportArtifactRole::ProductExportProof,
            ],
        ),
        writer_proof: proof_gate_summary(
            receipt,
            DAW_SESSION_WRITER_QA_GATE_ID,
            &[ExportArtifactRole::DawSessionWriterProof],
        ),
        host_import_proof: proof_gate_summary(receipt, DAW_SESSION_HOST_IMPORT_QA_GATE_ID, &[]),
        audible_output_proof: proof_gate_summary(
            receipt,
            DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID,
            &[],
        ),
    }
}

fn proof_gate_summary(
    receipt: &ExportReceiptState,
    gate_id: &'static str,
    artifact_roles: &[ExportArtifactRole],
) -> DawExportProofGateSummary {
    let artifacts = receipt
        .artifact_set
        .iter()
        .filter(|artifact| artifact_roles.contains(&artifact.role))
        .map(|artifact| DawExportProofArtifactSummary {
            role: artifact.role,
            media_type: artifact.media_type,
            location: artifact.location_identity().to_owned(),
            sha256: artifact.sha256.clone(),
        })
        .collect::<Vec<_>>();

    let Some(gate) = receipt.qa_gates.iter().find(|gate| gate.gate_id == gate_id) else {
        return DawExportProofGateSummary {
            gate_id,
            status: DawExportProofGateStatus::Missing,
            summary: None,
            artifact_roles: artifact_roles.to_vec(),
            artifact_available: !artifacts.is_empty(),
            artifacts,
        };
    };

    gate_summary(gate_id, gate, artifact_roles, artifacts)
}

fn gate_summary(
    gate_id: &'static str,
    gate: &ExportReceiptQaGateResult,
    fallback_artifact_roles: &[ExportArtifactRole],
    artifacts: Vec<DawExportProofArtifactSummary>,
) -> DawExportProofGateSummary {
    DawExportProofGateSummary {
        gate_id,
        status: proof_gate_status(gate.status),
        summary: gate.summary.clone(),
        artifact_roles: if gate.artifact_roles.is_empty() {
            fallback_artifact_roles.to_vec()
        } else {
            gate.artifact_roles.clone()
        },
        artifact_available: !artifacts.is_empty(),
        artifacts,
    }
}

fn proof_gate_status(status: ExportReceiptQaGateStatus) -> DawExportProofGateStatus {
    match status {
        ExportReceiptQaGateStatus::Passed => DawExportProofGateStatus::Passed,
        ExportReceiptQaGateStatus::Failed => DawExportProofGateStatus::Failed,
        ExportReceiptQaGateStatus::Deferred => DawExportProofGateStatus::Deferred,
    }
}

fn gate_passed(receipt: &ExportReceiptState, gate_id: &str) -> bool {
    receipt
        .qa_gates
        .iter()
        .any(|gate| gate.gate_id == gate_id && gate.status == ExportReceiptQaGateStatus::Passed)
}
