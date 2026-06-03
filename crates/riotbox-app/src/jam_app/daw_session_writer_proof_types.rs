use std::path::PathBuf;

use serde::Serialize;

pub const DAW_SESSION_LOCAL_PROJECT_WRITER_BOUNDARY_ID: &str =
    "daw_session.local_project_writer_v1";
pub const DAW_SESSION_WRITER_PROOF_SCHEMA_ID: &str = "riotbox.daw_session_writer_proof";
pub const DAW_SESSION_WRITER_PROOF_SCHEMA_VERSION: u32 = 1;
pub const DAW_SESSION_LOCAL_PROJECT_SKELETON_SCHEMA_ID: &str =
    "riotbox.daw_session_local_project_skeleton";
pub const DAW_SESSION_LOCAL_PROJECT_SKELETON_SCHEMA_VERSION: u32 = 1;
pub const DAW_SESSION_WRITER_PACKAGE_DIR: &str = "daw_session_writer";
pub const DAW_SESSION_WRITER_PROJECT_SKELETON_FILE: &str = "local_project_skeleton.json";
pub const DAW_SESSION_WRITER_PROOF_FILE: &str = "writer_proof.json";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrittenDawSessionWriterProofSkeleton {
    pub package_dir: PathBuf,
    pub project_skeleton_path: PathBuf,
    pub proof_path: PathBuf,
    pub project_skeleton_sha256: String,
    pub proof_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DawSessionWriterProofReport {
    pub status: DawSessionWriterProofReportStatus,
    pub ready: bool,
    pub writes_files: bool,
    pub package_dir: PathBuf,
    pub proof_path: PathBuf,
    pub project_skeleton_path: PathBuf,
    pub boundary_id: Option<String>,
    pub schema_id: Option<String>,
    pub schema_version: Option<u32>,
    pub receipt_id: Option<String>,
    pub project_skeleton_sha256: Option<String>,
    pub proof_sha256: Option<String>,
    pub proof_blockers: Vec<String>,
    pub blockers: Vec<DawSessionWriterProofReportBlocker>,
}

impl DawSessionWriterProofReport {
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
pub enum DawSessionWriterProofReportStatus {
    Ready,
    Blocked,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DawSessionWriterProofReportBlocker {
    MissingProofFile,
    MissingProjectSkeletonFile,
    InvalidProofJson,
    InvalidProjectSkeletonJson,
    ProofSchemaMismatch,
    ProofSchemaVersionMismatch,
    BoundaryMismatch,
    ProjectSkeletonSchemaMismatch,
    ProjectSkeletonHashMismatch,
    WriterNotProven,
    ProofBlockersPresent,
}

impl DawSessionWriterProofReportBlocker {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingProofFile => "missing_proof_file",
            Self::MissingProjectSkeletonFile => "missing_project_skeleton_file",
            Self::InvalidProofJson => "invalid_proof_json",
            Self::InvalidProjectSkeletonJson => "invalid_project_skeleton_json",
            Self::ProofSchemaMismatch => "proof_schema_mismatch",
            Self::ProofSchemaVersionMismatch => "proof_schema_version_mismatch",
            Self::BoundaryMismatch => "boundary_mismatch",
            Self::ProjectSkeletonSchemaMismatch => "project_skeleton_schema_mismatch",
            Self::ProjectSkeletonHashMismatch => "project_skeleton_hash_mismatch",
            Self::WriterNotProven => "writer_not_proven",
            Self::ProofBlockersPresent => "proof_blockers_present",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DawSessionWriterProofReceiptEvidenceError {
    NotDawSessionReceipt,
    ReceiptIdentityMismatch,
}
