use std::path::{Path, PathBuf};

use riotbox_core::{
    daw_session_manifest::DAW_SESSION_MANIFEST_SCHEMA_ID,
    daw_session_proof::DAW_SESSION_PROOF_SCHEMA_ID,
    daw_session_tempo_map::DAW_SESSION_TEMPO_MAP_SCHEMA_ID,
    export_readiness::ExportScope,
    session::{
        DAW_SESSION_JSON_PACKAGE_QA_GATE_ID, ExportArtifactRole, ExportArtifactSetEntry,
        ExportReceiptQaGateResult, ExportReceiptState,
    },
};
use serde::Serialize;
use serde_json::Value;

use super::{
    JamAppError,
    daw_session_writer_plan::{
        DAW_SESSION_ARRANGEMENT_MANIFEST_FILE, DAW_SESSION_PACKAGE_DIR, DAW_SESSION_PROOF_FILE,
        DAW_SESSION_TEMPO_MAP_FILE,
    },
    product_export::sha256_file,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DawSessionJsonPackageReport {
    pub status: DawSessionJsonPackageReportStatus,
    pub ready: bool,
    pub writes_files: bool,
    pub package_dir: PathBuf,
    pub blockers: Vec<DawSessionJsonPackageReportBlocker>,
    pub artifacts: Vec<DawSessionJsonPackageArtifactReport>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawSessionJsonPackageReportStatus {
    Ready,
    Blocked,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawSessionJsonPackageReportBlocker {
    MissingManifestFile,
    MissingTempoMapFile,
    MissingProofFile,
    InvalidManifestJson,
    InvalidTempoMapJson,
    InvalidProofJson,
    ManifestSchemaMismatch,
    TempoMapSchemaMismatch,
    ProofSchemaMismatch,
    ProofManifestHashMismatch,
}

impl DawSessionJsonPackageReportBlocker {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingManifestFile => "missing_manifest_file",
            Self::MissingTempoMapFile => "missing_tempo_map_file",
            Self::MissingProofFile => "missing_proof_file",
            Self::InvalidManifestJson => "invalid_manifest_json",
            Self::InvalidTempoMapJson => "invalid_tempo_map_json",
            Self::InvalidProofJson => "invalid_proof_json",
            Self::ManifestSchemaMismatch => "manifest_schema_mismatch",
            Self::TempoMapSchemaMismatch => "tempo_map_schema_mismatch",
            Self::ProofSchemaMismatch => "proof_schema_mismatch",
            Self::ProofManifestHashMismatch => "proof_manifest_hash_mismatch",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DawSessionJsonPackageArtifactReport {
    pub role: DawSessionJsonPackageArtifactRole,
    pub path: PathBuf,
    pub schema_id: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawSessionJsonPackageArtifactRole {
    ArrangementManifest,
    TempoMap,
    DawSessionProof,
}

impl DawSessionJsonPackageArtifactRole {
    #[must_use]
    pub const fn receipt_role(self) -> ExportArtifactRole {
        match self {
            Self::ArrangementManifest => ExportArtifactRole::ExportManifest,
            Self::TempoMap => ExportArtifactRole::DawSessionTempoMap,
            Self::DawSessionProof => ExportArtifactRole::ProductExportProof,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DawSessionJsonPackageReceiptEvidenceError {
    NotDawSessionReceipt,
}

pub fn attach_daw_session_json_package_evidence_to_receipt(
    receipt: &mut ExportReceiptState,
    report: &DawSessionJsonPackageReport,
) -> Result<(), DawSessionJsonPackageReceiptEvidenceError> {
    if receipt.export_scope != ExportScope::DawSession {
        return Err(DawSessionJsonPackageReceiptEvidenceError::NotDawSessionReceipt);
    }

    let package_roles = [
        ExportArtifactRole::ExportManifest,
        ExportArtifactRole::DawSessionTempoMap,
        ExportArtifactRole::ProductExportProof,
    ];
    receipt
        .artifact_set
        .retain(|artifact| !package_roles.contains(&artifact.role));
    receipt.artifact_set.extend(
        report
            .artifacts
            .iter()
            .filter_map(receipt_artifact_from_report),
    );

    let blockers = report
        .blockers
        .iter()
        .map(|blocker| blocker.as_str().to_owned())
        .collect::<Vec<_>>();
    let gate = ExportReceiptQaGateResult::daw_session_json_package_integrity(
        report.ready,
        &blockers,
        package_roles.to_vec(),
    );
    receipt
        .qa_gates
        .retain(|gate| gate.gate_id != DAW_SESSION_JSON_PACKAGE_QA_GATE_ID);
    receipt.qa_gates.push(gate);

    Ok(())
}

fn receipt_artifact_from_report(
    artifact: &DawSessionJsonPackageArtifactReport,
) -> Option<ExportArtifactSetEntry> {
    let sha256 = artifact.sha256.clone()?;
    Some(match artifact.role {
        DawSessionJsonPackageArtifactRole::ArrangementManifest => {
            ExportArtifactSetEntry::export_manifest(artifact.path.to_string_lossy(), sha256)
        }
        DawSessionJsonPackageArtifactRole::TempoMap => {
            ExportArtifactSetEntry::daw_session_tempo_map(artifact.path.to_string_lossy(), sha256)
        }
        DawSessionJsonPackageArtifactRole::DawSessionProof => {
            ExportArtifactSetEntry::product_export_proof(artifact.path.to_string_lossy(), sha256)
        }
    })
}

pub fn daw_session_json_package_report(
    destination_root: impl AsRef<Path>,
) -> DawSessionJsonPackageReport {
    let package_dir = destination_root.as_ref().join(DAW_SESSION_PACKAGE_DIR);
    let manifest = inspect_json_artifact(
        &package_dir.join(DAW_SESSION_ARRANGEMENT_MANIFEST_FILE),
        DawSessionJsonPackageArtifactRole::ArrangementManifest,
        DAW_SESSION_MANIFEST_SCHEMA_ID,
        DawSessionJsonPackageReportBlocker::MissingManifestFile,
        DawSessionJsonPackageReportBlocker::InvalidManifestJson,
        DawSessionJsonPackageReportBlocker::ManifestSchemaMismatch,
    );
    let tempo_map = inspect_json_artifact(
        &package_dir.join(DAW_SESSION_TEMPO_MAP_FILE),
        DawSessionJsonPackageArtifactRole::TempoMap,
        DAW_SESSION_TEMPO_MAP_SCHEMA_ID,
        DawSessionJsonPackageReportBlocker::MissingTempoMapFile,
        DawSessionJsonPackageReportBlocker::InvalidTempoMapJson,
        DawSessionJsonPackageReportBlocker::TempoMapSchemaMismatch,
    );
    let proof = inspect_json_artifact(
        &package_dir.join(DAW_SESSION_PROOF_FILE),
        DawSessionJsonPackageArtifactRole::DawSessionProof,
        DAW_SESSION_PROOF_SCHEMA_ID,
        DawSessionJsonPackageReportBlocker::MissingProofFile,
        DawSessionJsonPackageReportBlocker::InvalidProofJson,
        DawSessionJsonPackageReportBlocker::ProofSchemaMismatch,
    );

    let mut blockers = Vec::new();
    blockers.extend(manifest.blockers);
    blockers.extend(tempo_map.blockers);
    blockers.extend(proof.blockers);

    if let (Some(manifest_sha), Some(proof_json)) = (&manifest.artifact.sha256, &proof.json)
        && proof_json
            .get("manifest_sha256")
            .and_then(Value::as_str)
            .map(|proof_manifest_sha| proof_manifest_sha != manifest_sha)
            .unwrap_or(true)
    {
        blockers.push(DawSessionJsonPackageReportBlocker::ProofManifestHashMismatch);
    }
    blockers.sort_by_key(|blocker| *blocker as u8);
    blockers.dedup();

    let ready = blockers.is_empty();
    DawSessionJsonPackageReport {
        status: if ready {
            DawSessionJsonPackageReportStatus::Ready
        } else {
            DawSessionJsonPackageReportStatus::Blocked
        },
        ready,
        writes_files: false,
        package_dir,
        blockers,
        artifacts: vec![manifest.artifact, tempo_map.artifact, proof.artifact],
    }
}

struct InspectedJsonArtifact {
    artifact: DawSessionJsonPackageArtifactReport,
    json: Option<Value>,
    blockers: Vec<DawSessionJsonPackageReportBlocker>,
}

fn inspect_json_artifact(
    path: &Path,
    role: DawSessionJsonPackageArtifactRole,
    expected_schema_id: &'static str,
    missing_blocker: DawSessionJsonPackageReportBlocker,
    invalid_blocker: DawSessionJsonPackageReportBlocker,
    schema_blocker: DawSessionJsonPackageReportBlocker,
) -> InspectedJsonArtifact {
    if !path.exists() {
        return InspectedJsonArtifact {
            artifact: artifact_report(role, path, None, None),
            json: None,
            blockers: vec![missing_blocker],
        };
    }

    let json = match std::fs::read(path)
        .map_err(JamAppError::from)
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).map_err(JamAppError::from))
    {
        Ok(json) => json,
        Err(_) => {
            return InspectedJsonArtifact {
                artifact: artifact_report(role, path, None, sha256_file(path).ok()),
                json: None,
                blockers: vec![invalid_blocker],
            };
        }
    };
    let schema_id = json
        .get("schema_id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let sha256 = sha256_file(path).ok();
    let mut blockers = Vec::new();
    if schema_id.as_deref() != Some(expected_schema_id) {
        blockers.push(schema_blocker);
    }

    InspectedJsonArtifact {
        artifact: artifact_report(role, path, schema_id, sha256),
        json: Some(json),
        blockers,
    }
}

fn artifact_report(
    role: DawSessionJsonPackageArtifactRole,
    path: &Path,
    schema_id: Option<String>,
    sha256: Option<String>,
) -> DawSessionJsonPackageArtifactReport {
    DawSessionJsonPackageArtifactReport {
        role,
        path: path.to_path_buf(),
        schema_id,
        sha256,
    }
}
