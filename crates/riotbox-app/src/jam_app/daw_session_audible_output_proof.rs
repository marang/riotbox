use std::path::{Path, PathBuf};

use riotbox_core::{
    export_readiness::ExportScope,
    session::{
        DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID, ExportReceiptQaGateResult, ExportReceiptState,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{JamAppError, product_export::sha256_file};

pub const DAW_SESSION_AUDIBLE_OUTPUT_PROOF_SCHEMA_ID: &str =
    "riotbox.daw_session_audible_output_proof";
pub const DAW_SESSION_AUDIBLE_OUTPUT_PROOF_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DawSessionAudibleOutputProofReport {
    pub status: DawSessionAudibleOutputProofReportStatus,
    pub ready: bool,
    pub writes_files: bool,
    pub proof_path: PathBuf,
    pub schema_id: Option<String>,
    pub schema_version: Option<u32>,
    pub package_dir: Option<PathBuf>,
    pub audible: Option<bool>,
    pub proof_blockers: Vec<String>,
    pub sha256: Option<String>,
    pub blockers: Vec<DawSessionAudibleOutputProofReportBlocker>,
}

impl DawSessionAudibleOutputProofReport {
    #[must_use]
    pub fn gate_blockers(&self) -> Vec<String> {
        self.blockers
            .iter()
            .map(|blocker| blocker.as_str().to_owned())
            .chain(
                self.proof_blockers
                    .iter()
                    .filter(|blocker| !blocker.trim().is_empty())
                    .cloned(),
            )
            .collect()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawSessionAudibleOutputProofReportStatus {
    Ready,
    Blocked,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawSessionAudibleOutputProofReportBlocker {
    MissingProofFile,
    InvalidProofJson,
    SchemaMismatch,
    SchemaVersionMismatch,
    MissingPackageIdentity,
    AudibleOutputNotProven,
    ProofBlockersPresent,
}

impl DawSessionAudibleOutputProofReportBlocker {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingProofFile => "missing_proof_file",
            Self::InvalidProofJson => "invalid_proof_json",
            Self::SchemaMismatch => "schema_mismatch",
            Self::SchemaVersionMismatch => "schema_version_mismatch",
            Self::MissingPackageIdentity => "missing_package_identity",
            Self::AudibleOutputNotProven => "audible_output_not_proven",
            Self::ProofBlockersPresent => "proof_blockers_present",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DawSessionAudibleOutputProofReceiptEvidenceError {
    NotDawSessionReceipt,
}

pub fn attach_daw_session_audible_output_proof_evidence_to_receipt(
    receipt: &mut ExportReceiptState,
    report: &DawSessionAudibleOutputProofReport,
) -> Result<(), DawSessionAudibleOutputProofReceiptEvidenceError> {
    if receipt.export_scope != ExportScope::DawSession {
        return Err(DawSessionAudibleOutputProofReceiptEvidenceError::NotDawSessionReceipt);
    }

    let blockers = report.gate_blockers();
    let gate = ExportReceiptQaGateResult::daw_session_audible_output_proof(report.ready, &blockers);
    receipt
        .qa_gates
        .retain(|gate| gate.gate_id != DAW_SESSION_AUDIBLE_OUTPUT_QA_GATE_ID);
    receipt.qa_gates.push(gate);

    Ok(())
}

pub fn daw_session_audible_output_proof_report(
    proof_path: impl AsRef<Path>,
) -> DawSessionAudibleOutputProofReport {
    let proof_path = proof_path.as_ref();
    if !proof_path.exists() {
        return missing_proof_report(proof_path);
    }

    let sha256 = sha256_file(proof_path).ok();
    let json = match std::fs::read(proof_path)
        .map_err(JamAppError::from)
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).map_err(JamAppError::from))
    {
        Ok(json) => json,
        Err(_) => {
            return invalid_proof_report(proof_path, sha256);
        }
    };

    let evidence =
        serde_json::from_value::<DawSessionAudibleOutputProofEvidence>(json.clone()).ok();
    let schema_id = json
        .get("schema_id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let schema_version = json
        .get("schema_version")
        .and_then(Value::as_u64)
        .and_then(|version| u32::try_from(version).ok());
    let package_dir = evidence
        .as_ref()
        .map(|evidence| evidence.package_dir.clone());
    let audible = evidence.as_ref().map(|evidence| evidence.audible);
    let proof_blockers = evidence
        .as_ref()
        .map(|evidence| evidence.blockers.clone())
        .unwrap_or_default();

    let mut blockers = Vec::new();
    if schema_id.as_deref() != Some(DAW_SESSION_AUDIBLE_OUTPUT_PROOF_SCHEMA_ID) {
        blockers.push(DawSessionAudibleOutputProofReportBlocker::SchemaMismatch);
    }
    if schema_version != Some(DAW_SESSION_AUDIBLE_OUTPUT_PROOF_SCHEMA_VERSION) {
        blockers.push(DawSessionAudibleOutputProofReportBlocker::SchemaVersionMismatch);
    }
    if package_dir
        .as_ref()
        .is_none_or(|package_dir| package_dir.as_os_str().is_empty())
    {
        blockers.push(DawSessionAudibleOutputProofReportBlocker::MissingPackageIdentity);
    }
    if audible != Some(true) {
        blockers.push(DawSessionAudibleOutputProofReportBlocker::AudibleOutputNotProven);
    }
    if proof_blockers
        .iter()
        .any(|blocker| !blocker.trim().is_empty())
    {
        blockers.push(DawSessionAudibleOutputProofReportBlocker::ProofBlockersPresent);
    }
    blockers.sort_by_key(|blocker| *blocker as u8);
    blockers.dedup();

    let ready = blockers.is_empty();
    DawSessionAudibleOutputProofReport {
        status: if ready {
            DawSessionAudibleOutputProofReportStatus::Ready
        } else {
            DawSessionAudibleOutputProofReportStatus::Blocked
        },
        ready,
        writes_files: false,
        proof_path: proof_path.to_path_buf(),
        schema_id,
        schema_version,
        package_dir,
        audible,
        proof_blockers,
        sha256,
        blockers,
    }
}

#[derive(Clone, Debug, Deserialize)]
struct DawSessionAudibleOutputProofEvidence {
    package_dir: PathBuf,
    audible: bool,
    #[serde(default)]
    blockers: Vec<String>,
}

fn missing_proof_report(proof_path: &Path) -> DawSessionAudibleOutputProofReport {
    blocked_report(
        proof_path,
        None,
        vec![DawSessionAudibleOutputProofReportBlocker::MissingProofFile],
    )
}

fn invalid_proof_report(
    proof_path: &Path,
    sha256: Option<String>,
) -> DawSessionAudibleOutputProofReport {
    blocked_report(
        proof_path,
        sha256,
        vec![DawSessionAudibleOutputProofReportBlocker::InvalidProofJson],
    )
}

fn blocked_report(
    proof_path: &Path,
    sha256: Option<String>,
    blockers: Vec<DawSessionAudibleOutputProofReportBlocker>,
) -> DawSessionAudibleOutputProofReport {
    DawSessionAudibleOutputProofReport {
        status: DawSessionAudibleOutputProofReportStatus::Blocked,
        ready: false,
        writes_files: false,
        proof_path: proof_path.to_path_buf(),
        schema_id: None,
        schema_version: None,
        package_dir: None,
        audible: None,
        proof_blockers: Vec::new(),
        sha256,
        blockers,
    }
}
