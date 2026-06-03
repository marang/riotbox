use std::{
    fs,
    path::{Path, PathBuf},
};

use riotbox_core::{
    export_readiness::ExportScope,
    session::{
        DAW_SESSION_JSON_PACKAGE_QA_GATE_ID, DAW_SESSION_WRITER_QA_GATE_ID, ExportArtifactRole,
        ExportArtifactSetEntry, ExportReceiptQaGateResult, ExportReceiptQaGateStatus,
        ExportReceiptState, SessionFile,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    JamAppError,
    daw_session_package_report::{
        DawSessionJsonPackageArtifactReport, DawSessionJsonPackageArtifactRole,
        daw_session_json_package_report,
    },
    daw_session_writer_plan::daw_session_writer_plan,
    daw_session_writer_proof_types::*,
    product_export::sha256_file,
};

pub fn write_daw_session_writer_proof_skeleton(
    session: &SessionFile,
    session_base_dir: Option<&Path>,
    destination_root: impl AsRef<Path>,
) -> Result<WrittenDawSessionWriterProofSkeleton, JamAppError> {
    let destination_root = destination_root.as_ref();
    let plan = daw_session_writer_plan(session, session_base_dir, destination_root);
    if !plan.ready_for_writer {
        return Err(JamAppError::InvalidSession(format!(
            "DAW session writer proof blocked: {:?}",
            plan.readiness_blockers
        )));
    }
    let receipt = latest_daw_session_receipt(session).ok_or_else(|| {
        JamAppError::InvalidSession(
            "DAW session writer proof requires a DAW session receipt".into(),
        )
    })?;
    if !daw_json_package_gate_passed(receipt) {
        return Err(JamAppError::InvalidSession(
            "DAW session writer proof requires passed JSON package evidence".into(),
        ));
    }
    let json_package_report = daw_session_json_package_report(destination_root);
    if !json_package_report.ready {
        return Err(JamAppError::InvalidSession(format!(
            "DAW session writer proof requires ready JSON package report: {:?}",
            json_package_report.blockers
        )));
    }
    validate_json_package_report_matches_receipt(receipt, &json_package_report.artifacts)?;

    let final_package_dir = destination_root.join(DAW_SESSION_WRITER_PACKAGE_DIR);
    if final_package_dir.exists() {
        return Err(JamAppError::InvalidSession(format!(
            "DAW session writer proof destination already exists: {}",
            final_package_dir.display()
        )));
    }
    let staging_root = destination_root.join(format!(
        ".daw_session_writer_staging_{}",
        receipt.created_by_action.0
    ));
    if staging_root.exists() {
        return Err(JamAppError::InvalidSession(format!(
            "DAW session writer proof staging destination already exists: {}",
            staging_root.display()
        )));
    }
    let staging_package_dir = staging_root.join(DAW_SESSION_WRITER_PACKAGE_DIR);
    fs::create_dir_all(&staging_package_dir)?;

    let manifest = package_artifact(
        &json_package_report.artifacts,
        DawSessionJsonPackageArtifactRole::ArrangementManifest,
    )?;
    let tempo_map = package_artifact(
        &json_package_report.artifacts,
        DawSessionJsonPackageArtifactRole::TempoMap,
    )?;
    let daw_proof = package_artifact(
        &json_package_report.artifacts,
        DawSessionJsonPackageArtifactRole::DawSessionProof,
    )?;
    let final_project_skeleton_path =
        final_package_dir.join(DAW_SESSION_WRITER_PROJECT_SKELETON_FILE);
    let project_skeleton = DawSessionLocalProjectSkeleton {
        schema_id: DAW_SESSION_LOCAL_PROJECT_SKELETON_SCHEMA_ID.into(),
        schema_version: DAW_SESSION_LOCAL_PROJECT_SKELETON_SCHEMA_VERSION,
        boundary_id: DAW_SESSION_LOCAL_PROJECT_WRITER_BOUNDARY_ID.into(),
        receipt_id: receipt.receipt_id.as_str().into(),
        source_json_package_dir: json_package_report.package_dir.clone(),
        arrangement_manifest_path: manifest.path.clone(),
        tempo_map_path: tempo_map.path.clone(),
        daw_session_proof_path: daw_proof.path.clone(),
        host_imported: false,
        audible_output_proven: false,
    };
    let staging_project_skeleton_path =
        staging_package_dir.join(DAW_SESSION_WRITER_PROJECT_SKELETON_FILE);
    fs::write(
        &staging_project_skeleton_path,
        serde_json::to_vec_pretty(&project_skeleton)?,
    )?;
    let project_skeleton_sha256 = sha256_file(&staging_project_skeleton_path)?;

    let proof = DawSessionWriterProofEvidence {
        schema_id: DAW_SESSION_WRITER_PROOF_SCHEMA_ID.into(),
        schema_version: DAW_SESSION_WRITER_PROOF_SCHEMA_VERSION,
        boundary_id: DAW_SESSION_LOCAL_PROJECT_WRITER_BOUNDARY_ID.into(),
        receipt_id: receipt.receipt_id.as_str().into(),
        package_dir: final_package_dir.clone(),
        project_skeleton_path: final_project_skeleton_path.clone(),
        project_skeleton_sha256: project_skeleton_sha256.clone(),
        json_package_manifest_sha256: manifest.sha256.clone().ok_or_else(|| {
            JamAppError::InvalidSession("missing DAW JSON manifest sha256".into())
        })?,
        json_package_tempo_map_sha256: tempo_map.sha256.clone().ok_or_else(|| {
            JamAppError::InvalidSession("missing DAW JSON tempo map sha256".into())
        })?,
        json_package_proof_sha256: daw_proof
            .sha256
            .clone()
            .ok_or_else(|| JamAppError::InvalidSession("missing DAW JSON proof sha256".into()))?,
        written: true,
        blockers: Vec::new(),
    };
    let staging_proof_path = staging_package_dir.join(DAW_SESSION_WRITER_PROOF_FILE);
    fs::write(&staging_proof_path, serde_json::to_vec_pretty(&proof)?)?;

    fs::rename(&staging_package_dir, &final_package_dir)?;
    fs::remove_dir_all(&staging_root)?;

    let project_skeleton_path = final_package_dir.join(DAW_SESSION_WRITER_PROJECT_SKELETON_FILE);
    let proof_path = final_package_dir.join(DAW_SESSION_WRITER_PROOF_FILE);
    let promoted_project_skeleton_sha256 = sha256_file(&project_skeleton_path)?;
    if promoted_project_skeleton_sha256 != project_skeleton_sha256 {
        return Err(JamAppError::InvalidSession(
            "written DAW session project skeleton hash changed after promotion".into(),
        ));
    }
    let proof_sha256 = sha256_file(&proof_path)?;

    Ok(WrittenDawSessionWriterProofSkeleton {
        package_dir: final_package_dir,
        project_skeleton_path,
        proof_path,
        project_skeleton_sha256,
        proof_sha256,
    })
}

pub fn daw_session_writer_proof_report(
    destination_root: impl AsRef<Path>,
) -> DawSessionWriterProofReport {
    let package_dir = destination_root
        .as_ref()
        .join(DAW_SESSION_WRITER_PACKAGE_DIR);
    let proof_path = package_dir.join(DAW_SESSION_WRITER_PROOF_FILE);
    let project_skeleton_path = package_dir.join(DAW_SESSION_WRITER_PROJECT_SKELETON_FILE);

    let mut blockers = Vec::new();
    if !proof_path.exists() {
        blockers.push(DawSessionWriterProofReportBlocker::MissingProofFile);
    }
    if !project_skeleton_path.exists() {
        blockers.push(DawSessionWriterProofReportBlocker::MissingProjectSkeletonFile);
    }

    let proof_sha256 = sha256_file(&proof_path).ok();
    let proof_json = read_json(&proof_path).map_err(|_| {
        blockers.push(DawSessionWriterProofReportBlocker::InvalidProofJson);
    });
    let skeleton_sha256 = sha256_file(&project_skeleton_path).ok();
    let skeleton_json = read_json(&project_skeleton_path).map_err(|_| {
        blockers.push(DawSessionWriterProofReportBlocker::InvalidProjectSkeletonJson);
    });

    let evidence = proof_json.as_ref().ok().and_then(|json| {
        serde_json::from_value::<DawSessionWriterProofEvidence>(json.clone()).ok()
    });
    let schema_id = proof_json
        .as_ref()
        .ok()
        .and_then(|json| json.get("schema_id"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let schema_version = proof_json
        .as_ref()
        .ok()
        .and_then(|json| json.get("schema_version"))
        .and_then(Value::as_u64)
        .and_then(|version| u32::try_from(version).ok());
    let boundary_id = proof_json
        .as_ref()
        .ok()
        .and_then(|json| json.get("boundary_id"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let receipt_id = proof_json
        .as_ref()
        .ok()
        .and_then(|json| json.get("receipt_id"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let proof_blockers = evidence
        .as_ref()
        .map(|evidence| evidence.blockers.clone())
        .unwrap_or_default();

    if schema_id.as_deref() != Some(DAW_SESSION_WRITER_PROOF_SCHEMA_ID) {
        blockers.push(DawSessionWriterProofReportBlocker::ProofSchemaMismatch);
    }
    if schema_version != Some(DAW_SESSION_WRITER_PROOF_SCHEMA_VERSION) {
        blockers.push(DawSessionWriterProofReportBlocker::ProofSchemaVersionMismatch);
    }
    if boundary_id.as_deref() != Some(DAW_SESSION_LOCAL_PROJECT_WRITER_BOUNDARY_ID) {
        blockers.push(DawSessionWriterProofReportBlocker::BoundaryMismatch);
    }
    if evidence.as_ref().map(|evidence| evidence.written) != Some(true) {
        blockers.push(DawSessionWriterProofReportBlocker::WriterNotProven);
    }
    if proof_blockers
        .iter()
        .any(|blocker| !blocker.trim().is_empty())
    {
        blockers.push(DawSessionWriterProofReportBlocker::ProofBlockersPresent);
    }
    if skeleton_json
        .as_ref()
        .ok()
        .and_then(|json| json.get("schema_id"))
        .and_then(Value::as_str)
        != Some(DAW_SESSION_LOCAL_PROJECT_SKELETON_SCHEMA_ID)
    {
        blockers.push(DawSessionWriterProofReportBlocker::ProjectSkeletonSchemaMismatch);
    }
    if let (Some(evidence), Some(actual_skeleton_sha256)) = (&evidence, &skeleton_sha256)
        && evidence.project_skeleton_sha256 != *actual_skeleton_sha256
    {
        blockers.push(DawSessionWriterProofReportBlocker::ProjectSkeletonHashMismatch);
    }
    blockers.sort_by_key(|blocker| *blocker as u8);
    blockers.dedup();

    let ready = blockers.is_empty();
    DawSessionWriterProofReport {
        status: if ready {
            DawSessionWriterProofReportStatus::Ready
        } else {
            DawSessionWriterProofReportStatus::Blocked
        },
        ready,
        writes_files: false,
        package_dir,
        proof_path,
        project_skeleton_path,
        boundary_id,
        schema_id,
        schema_version,
        receipt_id,
        project_skeleton_sha256: skeleton_sha256,
        proof_sha256,
        proof_blockers,
        blockers,
    }
}

pub fn attach_daw_session_writer_proof_evidence_to_receipt(
    receipt: &mut ExportReceiptState,
    report: &DawSessionWriterProofReport,
) -> Result<(), DawSessionWriterProofReceiptEvidenceError> {
    if receipt.export_scope != ExportScope::DawSession {
        return Err(DawSessionWriterProofReceiptEvidenceError::NotDawSessionReceipt);
    }
    if report.ready && report.receipt_id.as_deref() != Some(receipt.receipt_id.as_str()) {
        return Err(DawSessionWriterProofReceiptEvidenceError::ReceiptIdentityMismatch);
    }

    receipt
        .artifact_set
        .retain(|artifact| artifact.role != ExportArtifactRole::DawSessionWriterProof);
    if report.ready
        && let Some(sha256) = &report.proof_sha256
    {
        receipt
            .artifact_set
            .push(ExportArtifactSetEntry::daw_session_writer_proof(
                report.proof_path.to_string_lossy(),
                sha256.clone(),
            ));
    }

    let blockers = report.gate_blockers();
    let gate = ExportReceiptQaGateResult::daw_session_writer_proof(
        report.ready,
        &blockers,
        vec![ExportArtifactRole::DawSessionWriterProof],
    );
    receipt
        .qa_gates
        .retain(|gate| gate.gate_id != DAW_SESSION_WRITER_QA_GATE_ID);
    receipt.qa_gates.push(gate);

    Ok(())
}

fn latest_daw_session_receipt(session: &SessionFile) -> Option<&ExportReceiptState> {
    session
        .export_receipts
        .iter()
        .rev()
        .find(|receipt| receipt.export_scope == ExportScope::DawSession)
}

fn daw_json_package_gate_passed(receipt: &ExportReceiptState) -> bool {
    receipt.qa_gates.iter().any(|gate| {
        gate.gate_id == DAW_SESSION_JSON_PACKAGE_QA_GATE_ID
            && gate.status == ExportReceiptQaGateStatus::Passed
    })
}

fn package_artifact(
    artifacts: &[DawSessionJsonPackageArtifactReport],
    role: DawSessionJsonPackageArtifactRole,
) -> Result<&DawSessionJsonPackageArtifactReport, JamAppError> {
    artifacts
        .iter()
        .find(|artifact| artifact.role == role && artifact.sha256.is_some())
        .ok_or_else(|| {
            JamAppError::InvalidSession(format!(
                "missing ready DAW JSON package artifact: {role:?}"
            ))
        })
}

fn validate_json_package_report_matches_receipt(
    receipt: &ExportReceiptState,
    artifacts: &[DawSessionJsonPackageArtifactReport],
) -> Result<(), JamAppError> {
    for (package_role, receipt_role) in [
        (
            DawSessionJsonPackageArtifactRole::ArrangementManifest,
            ExportArtifactRole::ExportManifest,
        ),
        (
            DawSessionJsonPackageArtifactRole::TempoMap,
            ExportArtifactRole::DawSessionTempoMap,
        ),
        (
            DawSessionJsonPackageArtifactRole::DawSessionProof,
            ExportArtifactRole::ProductExportProof,
        ),
    ] {
        let package_artifact = package_artifact(artifacts, package_role)?;
        let receipt_artifact = receipt
            .artifact_set
            .iter()
            .find(|artifact| artifact.role == receipt_role)
            .ok_or_else(|| {
                JamAppError::InvalidSession(format!(
                    "DAW session writer proof missing receipt artifact evidence: {receipt_role:?}"
                ))
            })?;
        let package_sha = package_artifact
            .sha256
            .as_deref()
            .ok_or_else(|| JamAppError::InvalidSession("missing package sha256".into()))?;
        if receipt_artifact.location_identity() != package_artifact.path.to_string_lossy()
            || receipt_artifact.sha256 != package_sha
        {
            return Err(JamAppError::InvalidSession(format!(
                "DAW session writer proof package evidence mismatch for {receipt_role:?}"
            )));
        }
    }

    Ok(())
}

fn read_json(path: &Path) -> Result<Value, JamAppError> {
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DawSessionLocalProjectSkeleton {
    schema_id: String,
    schema_version: u32,
    boundary_id: String,
    receipt_id: String,
    source_json_package_dir: PathBuf,
    arrangement_manifest_path: PathBuf,
    tempo_map_path: PathBuf,
    daw_session_proof_path: PathBuf,
    host_imported: bool,
    audible_output_proven: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct DawSessionWriterProofEvidence {
    schema_id: String,
    schema_version: u32,
    boundary_id: String,
    receipt_id: String,
    package_dir: PathBuf,
    project_skeleton_path: PathBuf,
    project_skeleton_sha256: String,
    json_package_manifest_sha256: String,
    json_package_tempo_map_sha256: String,
    json_package_proof_sha256: String,
    written: bool,
    #[serde(default)]
    blockers: Vec<String>,
}
