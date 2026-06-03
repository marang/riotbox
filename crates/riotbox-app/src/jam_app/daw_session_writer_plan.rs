use std::path::{Path, PathBuf};

use riotbox_core::{
    export_readiness::ExportScope,
    session::{ExportDawTempoMapRef, ExportReceiptState, SessionFile},
};
use serde::Serialize;

use super::daw_export_operator_report::{
    DawExportOperatorReadinessReport, DawExportReadinessBlocker,
    daw_export_operator_readiness_report,
};

pub const DAW_SESSION_WRITER_PLAN_BOUNDARY_ID: &str = "daw_session.writer_plan_v1";
pub const DAW_SESSION_PACKAGE_DIR: &str = "daw_session";
pub const DAW_SESSION_ARRANGEMENT_MANIFEST_FILE: &str = "arrangement_manifest.json";
pub const DAW_SESSION_TEMPO_MAP_FILE: &str = "tempo_map.json";
pub const DAW_SESSION_PROOF_FILE: &str = "daw_session_proof.json";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DawSessionWriterPlan {
    pub status: DawSessionWriterPlanStatus,
    pub ready_for_writer: bool,
    pub writes_files: bool,
    pub boundary_id: &'static str,
    pub destination_root: Option<String>,
    pub readiness_blockers: Vec<DawSessionWriterPlanBlocker>,
    pub writer_blockers: Vec<DawSessionWriterReleaseBlocker>,
    pub receipt: Option<DawSessionWriterReceiptSummary>,
    pub placement_refs: Vec<riotbox_core::session::ExportArrangementPlacementRef>,
    pub tempo_map_ref: Option<ExportDawTempoMapRef>,
    pub source_artifacts: Vec<DawSessionSourceArtifactRef>,
    pub planned_artifacts: Vec<DawSessionPlannedArtifact>,
    pub operator_readiness: DawExportOperatorReadinessReport,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawSessionWriterPlanStatus {
    Blocked,
    ReadyForWriter,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawSessionWriterPlanBlocker {
    NoDawSessionReceipt,
    MissingDestinationRoot,
    UnsupportedCommandBoundary,
    ArrangementPlacementBlocked,
    DawTempoMapBlocked,
    MissingArtifactIdentity,
    MissingLocalFiles,
    UnreadableLocalFiles,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawSessionWriterReleaseBlocker {
    DawWriterMissing,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DawSessionWriterReceiptSummary {
    pub receipt_id: riotbox_core::ids::ExportReceiptId,
    pub created_by_action: u64,
    pub pack_id: String,
    pub export_role: riotbox_core::export_readiness::ProductExportRole,
    pub export_boundary: riotbox_core::export_readiness::ProductExportBoundary,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DawSessionSourceArtifactRef {
    pub role: DawSessionSourceArtifactRole,
    pub path: PathBuf,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawSessionSourceArtifactRole {
    ArrangementManifest,
    ProductExportProof,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DawSessionPlannedArtifact {
    pub role: DawSessionPlannedArtifactRole,
    pub media_type: DawSessionPlannedArtifactMediaType,
    pub path: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawSessionPlannedArtifactRole {
    ArrangementManifest,
    TempoMap,
    DawSessionProof,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawSessionPlannedArtifactMediaType {
    Json,
}

#[must_use]
pub fn daw_session_writer_plan(
    session: &SessionFile,
    session_base_dir: Option<&Path>,
    destination_root: impl AsRef<Path>,
) -> DawSessionWriterPlan {
    let operator_readiness = daw_export_operator_readiness_report(session, session_base_dir);
    let receipt = latest_daw_session_receipt(session);
    let destination_root = normalize_destination_root(destination_root.as_ref());
    let mut readiness_blockers = operator_readiness
        .readiness_blockers
        .iter()
        .copied()
        .map(daw_readiness_blocker)
        .collect::<Vec<_>>();
    if destination_root.is_none() {
        push_unique(
            &mut readiness_blockers,
            DawSessionWriterPlanBlocker::MissingDestinationRoot,
        );
    }
    readiness_blockers.dedup();

    let ready_for_writer = readiness_blockers.is_empty();
    let planned_artifacts = destination_root
        .as_deref()
        .map(planned_artifacts)
        .unwrap_or_default();
    let source_artifacts = receipt
        .map(|receipt| source_artifacts(receipt, session_base_dir))
        .unwrap_or_default();

    DawSessionWriterPlan {
        status: if ready_for_writer {
            DawSessionWriterPlanStatus::ReadyForWriter
        } else {
            DawSessionWriterPlanStatus::Blocked
        },
        ready_for_writer,
        writes_files: false,
        boundary_id: DAW_SESSION_WRITER_PLAN_BOUNDARY_ID,
        destination_root,
        readiness_blockers,
        writer_blockers: vec![DawSessionWriterReleaseBlocker::DawWriterMissing],
        receipt: receipt.map(receipt_summary),
        placement_refs: receipt
            .map(|receipt| receipt.arrangement_placement_refs.clone())
            .unwrap_or_default(),
        tempo_map_ref: receipt.and_then(|receipt| receipt.daw_tempo_map_ref.clone()),
        source_artifacts,
        planned_artifacts,
        operator_readiness,
    }
}

fn latest_daw_session_receipt(session: &SessionFile) -> Option<&ExportReceiptState> {
    session
        .export_receipts
        .iter()
        .rev()
        .find(|receipt| receipt.export_scope == ExportScope::DawSession)
}

fn normalize_destination_root(destination_root: &Path) -> Option<String> {
    let destination_root = destination_root.to_string_lossy();
    let destination_root = destination_root.trim().trim_end_matches('/');
    if destination_root.is_empty() {
        None
    } else {
        Some(destination_root.to_owned())
    }
}

fn planned_artifacts(destination_root: &str) -> Vec<DawSessionPlannedArtifact> {
    vec![
        planned_json(
            DawSessionPlannedArtifactRole::ArrangementManifest,
            package_path(destination_root, DAW_SESSION_ARRANGEMENT_MANIFEST_FILE),
        ),
        planned_json(
            DawSessionPlannedArtifactRole::TempoMap,
            package_path(destination_root, DAW_SESSION_TEMPO_MAP_FILE),
        ),
        planned_json(
            DawSessionPlannedArtifactRole::DawSessionProof,
            package_path(destination_root, DAW_SESSION_PROOF_FILE),
        ),
    ]
}

fn planned_json(role: DawSessionPlannedArtifactRole, path: String) -> DawSessionPlannedArtifact {
    DawSessionPlannedArtifact {
        role,
        media_type: DawSessionPlannedArtifactMediaType::Json,
        path,
    }
}

fn package_path(destination_root: &str, file_name: &str) -> String {
    format!("{destination_root}/{DAW_SESSION_PACKAGE_DIR}/{file_name}")
}

fn source_artifacts(
    receipt: &ExportReceiptState,
    session_base_dir: Option<&Path>,
) -> Vec<DawSessionSourceArtifactRef> {
    [
        (
            DawSessionSourceArtifactRole::ArrangementManifest,
            receipt.artifact_path.as_str(),
        ),
        (
            DawSessionSourceArtifactRole::ProductExportProof,
            receipt.proof_path.as_str(),
        ),
    ]
    .into_iter()
    .filter_map(|(role, path)| source_artifact(role, path, session_base_dir))
    .collect()
}

fn source_artifact(
    role: DawSessionSourceArtifactRole,
    path: &str,
    session_base_dir: Option<&Path>,
) -> Option<DawSessionSourceArtifactRef> {
    let path = path.trim();
    if path.is_empty() {
        return None;
    }
    let path = Path::new(path);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        session_base_dir.map(|base_dir| base_dir.join(path))?
    };
    Some(DawSessionSourceArtifactRef { role, path })
}

fn receipt_summary(receipt: &ExportReceiptState) -> DawSessionWriterReceiptSummary {
    DawSessionWriterReceiptSummary {
        receipt_id: receipt.receipt_id.clone(),
        created_by_action: receipt.created_by_action.0,
        pack_id: receipt.pack_id.clone(),
        export_role: receipt.export_role,
        export_boundary: receipt.export_boundary,
    }
}

fn daw_readiness_blocker(blocker: DawExportReadinessBlocker) -> DawSessionWriterPlanBlocker {
    match blocker {
        DawExportReadinessBlocker::NoDawSessionReceipt => {
            DawSessionWriterPlanBlocker::NoDawSessionReceipt
        }
        DawExportReadinessBlocker::UnsupportedCommandBoundary => {
            DawSessionWriterPlanBlocker::UnsupportedCommandBoundary
        }
        DawExportReadinessBlocker::ArrangementPlacementBlocked => {
            DawSessionWriterPlanBlocker::ArrangementPlacementBlocked
        }
        DawExportReadinessBlocker::DawTempoMapBlocked => {
            DawSessionWriterPlanBlocker::DawTempoMapBlocked
        }
        DawExportReadinessBlocker::MissingArtifactIdentity => {
            DawSessionWriterPlanBlocker::MissingArtifactIdentity
        }
        DawExportReadinessBlocker::MissingLocalFiles => {
            DawSessionWriterPlanBlocker::MissingLocalFiles
        }
        DawExportReadinessBlocker::UnreadableLocalFiles => {
            DawSessionWriterPlanBlocker::UnreadableLocalFiles
        }
    }
}

fn push_unique<T: PartialEq>(values: &mut Vec<T>, value: T) {
    if !values.contains(&value) {
        values.push(value);
    }
}
