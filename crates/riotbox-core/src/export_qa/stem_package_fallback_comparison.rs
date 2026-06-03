use serde::{Deserialize, Serialize};

use crate::session::{ExportArtifactRole, ExportArtifactSetEntry};

use super::fallback_comparison_evidence_is_valid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageFallbackComparisonQaReport {
    pub status: StemPackageFallbackComparisonQaStatus,
    pub claimed_roles: Vec<ExportArtifactRole>,
    pub checked_artifact_count: usize,
    pub failures: Vec<StemPackageFallbackComparisonQaFailure>,
}

impl StemPackageFallbackComparisonQaReport {
    #[must_use]
    pub fn passed(&self) -> bool {
        self.status == StemPackageFallbackComparisonQaStatus::Passed
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageFallbackComparisonQaStatus {
    Passed,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageFallbackComparisonQaFailure {
    pub role: Option<ExportArtifactRole>,
    pub kind: StemPackageFallbackComparisonQaFailureKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageFallbackComparisonQaFailureKind {
    NoClaimedStemRoles,
    NonStemRoleClaimed,
    MissingRoleArtifact,
    DuplicateRoleArtifact,
    MissingFallbackComparisonEvidence,
    InvalidFallbackComparisonEvidence,
}

pub fn validate_stem_package_fallback_comparison_evidence(
    artifact_set: &[ExportArtifactSetEntry],
    claimed_roles: &[ExportArtifactRole],
) -> StemPackageFallbackComparisonQaReport {
    let mut failures = Vec::new();
    if claimed_roles.is_empty() {
        failures.push(fallback_comparison_failure(
            None,
            StemPackageFallbackComparisonQaFailureKind::NoClaimedStemRoles,
        ));
    }

    for role in claimed_roles {
        validate_claimed_role_fallback_comparison(*role, artifact_set, &mut failures);
    }

    let status = if failures.is_empty() {
        StemPackageFallbackComparisonQaStatus::Passed
    } else {
        StemPackageFallbackComparisonQaStatus::Failed
    };

    StemPackageFallbackComparisonQaReport {
        status,
        claimed_roles: claimed_roles.to_vec(),
        checked_artifact_count: artifact_set.len(),
        failures,
    }
}

fn validate_claimed_role_fallback_comparison(
    role: ExportArtifactRole,
    artifact_set: &[ExportArtifactSetEntry],
    failures: &mut Vec<StemPackageFallbackComparisonQaFailure>,
) {
    if !role.is_stem_role() {
        failures.push(fallback_comparison_failure(
            Some(role),
            StemPackageFallbackComparisonQaFailureKind::NonStemRoleClaimed,
        ));
        return;
    }

    let mut matches = artifact_set.iter().filter(|artifact| artifact.role == role);
    let Some(artifact) = matches.next() else {
        failures.push(fallback_comparison_failure(
            Some(role),
            StemPackageFallbackComparisonQaFailureKind::MissingRoleArtifact,
        ));
        return;
    };
    if matches.next().is_some() {
        failures.push(fallback_comparison_failure(
            Some(role),
            StemPackageFallbackComparisonQaFailureKind::DuplicateRoleArtifact,
        ));
        return;
    }

    match &artifact.fallback_comparison {
        Some(comparison) if fallback_comparison_evidence_is_valid(comparison) => {}
        Some(_) => failures.push(fallback_comparison_failure(
            Some(role),
            StemPackageFallbackComparisonQaFailureKind::InvalidFallbackComparisonEvidence,
        )),
        None => failures.push(fallback_comparison_failure(
            Some(role),
            StemPackageFallbackComparisonQaFailureKind::MissingFallbackComparisonEvidence,
        )),
    }
}

fn fallback_comparison_failure(
    role: Option<ExportArtifactRole>,
    kind: StemPackageFallbackComparisonQaFailureKind,
) -> StemPackageFallbackComparisonQaFailure {
    StemPackageFallbackComparisonQaFailure { role, kind }
}
