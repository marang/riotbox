use serde::{Deserialize, Serialize};

use crate::session::{ExportArtifactRole, ExportArtifactSetEntry};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageLineageQaReport {
    pub status: StemPackageLineageQaStatus,
    pub claimed_roles: Vec<ExportArtifactRole>,
    pub checked_artifact_count: usize,
    pub failures: Vec<StemPackageLineageQaFailure>,
}

impl StemPackageLineageQaReport {
    #[must_use]
    pub fn passed(&self) -> bool {
        self.status == StemPackageLineageQaStatus::Passed
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageLineageQaStatus {
    Passed,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageLineageQaFailure {
    pub role: Option<ExportArtifactRole>,
    pub kind: StemPackageLineageQaFailureKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageLineageQaFailureKind {
    NoClaimedStemRoles,
    NonStemRoleClaimed,
    MissingRoleArtifact,
    DuplicateRoleArtifact,
    MissingLineageEvidence,
    InvalidLineageEvidence,
}

pub fn validate_stem_package_lineage_evidence(
    artifact_set: &[ExportArtifactSetEntry],
    claimed_roles: &[ExportArtifactRole],
) -> StemPackageLineageQaReport {
    let mut failures = Vec::new();
    if claimed_roles.is_empty() {
        failures.push(lineage_failure(
            None,
            StemPackageLineageQaFailureKind::NoClaimedStemRoles,
        ));
    }

    for role in claimed_roles {
        validate_claimed_role_lineage(*role, artifact_set, &mut failures);
    }

    let status = if failures.is_empty() {
        StemPackageLineageQaStatus::Passed
    } else {
        StemPackageLineageQaStatus::Failed
    };

    StemPackageLineageQaReport {
        status,
        claimed_roles: claimed_roles.to_vec(),
        checked_artifact_count: artifact_set.len(),
        failures,
    }
}

fn validate_claimed_role_lineage(
    role: ExportArtifactRole,
    artifact_set: &[ExportArtifactSetEntry],
    failures: &mut Vec<StemPackageLineageQaFailure>,
) {
    if !role.is_stem_role() {
        failures.push(lineage_failure(
            Some(role),
            StemPackageLineageQaFailureKind::NonStemRoleClaimed,
        ));
        return;
    }

    let mut matches = artifact_set.iter().filter(|artifact| artifact.role == role);
    let Some(artifact) = matches.next() else {
        failures.push(lineage_failure(
            Some(role),
            StemPackageLineageQaFailureKind::MissingRoleArtifact,
        ));
        return;
    };
    if matches.next().is_some() {
        failures.push(lineage_failure(
            Some(role),
            StemPackageLineageQaFailureKind::DuplicateRoleArtifact,
        ));
        return;
    }

    if !artifact_has_lineage_evidence(artifact) {
        failures.push(lineage_failure(
            Some(role),
            StemPackageLineageQaFailureKind::MissingLineageEvidence,
        ));
    }
    if artifact_has_invalid_lineage_evidence(artifact) {
        failures.push(lineage_failure(
            Some(role),
            StemPackageLineageQaFailureKind::InvalidLineageEvidence,
        ));
    }
}

fn artifact_has_lineage_evidence(artifact: &ExportArtifactSetEntry) -> bool {
    artifact.source_graph_ref.is_some()
        || !artifact.source_capture_refs.is_empty()
        || !artifact.lineage_capture_refs.is_empty()
}

fn artifact_has_invalid_lineage_evidence(artifact: &ExportArtifactSetEntry) -> bool {
    artifact
        .source_graph_ref
        .as_ref()
        .is_some_and(|source_graph| {
            source_graph.source_id.as_str().trim().is_empty()
                || source_graph.graph_hash.trim().is_empty()
        })
        || artifact
            .source_capture_refs
            .iter()
            .any(|capture_id| capture_id.as_str().trim().is_empty())
        || artifact
            .lineage_capture_refs
            .iter()
            .any(|capture_id| capture_id.as_str().trim().is_empty())
}

fn lineage_failure(
    role: Option<ExportArtifactRole>,
    kind: StemPackageLineageQaFailureKind,
) -> StemPackageLineageQaFailure {
    StemPackageLineageQaFailure { role, kind }
}
