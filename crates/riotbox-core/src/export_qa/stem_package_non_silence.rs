use serde::{Deserialize, Serialize};

use crate::session::{ExportArtifactRole, ExportArtifactSetEntry};

use super::{metrics_can_prove_activity, metrics_prove_silence};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageNonSilenceQaReport {
    pub status: StemPackageNonSilenceQaStatus,
    pub claimed_roles: Vec<ExportArtifactRole>,
    pub checked_artifact_count: usize,
    pub failures: Vec<StemPackageNonSilenceQaFailure>,
    pub deferred_checks: Vec<StemPackageNonSilenceDeferredCheck>,
}

impl StemPackageNonSilenceQaReport {
    #[must_use]
    pub fn passed(&self) -> bool {
        self.status == StemPackageNonSilenceQaStatus::Passed
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageNonSilenceQaStatus {
    Passed,
    Deferred,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageNonSilenceQaFailure {
    pub role: Option<ExportArtifactRole>,
    pub kind: StemPackageNonSilenceQaFailureKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageNonSilenceQaFailureKind {
    NoClaimedStemRoles,
    NonStemRoleClaimed,
    MissingRoleArtifact,
    DuplicateRoleArtifact,
    InsufficientNonSilenceMetrics,
    SilentArtifactMetrics,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageNonSilenceDeferredCheck {
    pub role: ExportArtifactRole,
    pub check: StemPackageNonSilenceDeferredCheckKind,
    pub status: StemPackageNonSilenceDeferredCheckStatus,
    pub reason: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageNonSilenceDeferredCheckKind {
    MissingAudioMetrics,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageNonSilenceDeferredCheckStatus {
    AspirationalUntilAudioEvidenceAttached,
}

pub fn validate_stem_package_non_silence_evidence(
    artifact_set: &[ExportArtifactSetEntry],
    claimed_roles: &[ExportArtifactRole],
) -> StemPackageNonSilenceQaReport {
    let mut failures = Vec::new();
    let mut deferred_checks = Vec::new();
    if claimed_roles.is_empty() {
        failures.push(non_silence_failure(
            None,
            StemPackageNonSilenceQaFailureKind::NoClaimedStemRoles,
        ));
    }

    for role in claimed_roles {
        validate_claimed_role_non_silence(*role, artifact_set, &mut failures, &mut deferred_checks);
    }

    let status = if !failures.is_empty() {
        StemPackageNonSilenceQaStatus::Failed
    } else if !deferred_checks.is_empty() {
        StemPackageNonSilenceQaStatus::Deferred
    } else {
        StemPackageNonSilenceQaStatus::Passed
    };

    StemPackageNonSilenceQaReport {
        status,
        claimed_roles: claimed_roles.to_vec(),
        checked_artifact_count: artifact_set.len(),
        failures,
        deferred_checks,
    }
}

fn validate_claimed_role_non_silence(
    role: ExportArtifactRole,
    artifact_set: &[ExportArtifactSetEntry],
    failures: &mut Vec<StemPackageNonSilenceQaFailure>,
    deferred_checks: &mut Vec<StemPackageNonSilenceDeferredCheck>,
) {
    if !role.is_stem_role() {
        failures.push(non_silence_failure(
            Some(role),
            StemPackageNonSilenceQaFailureKind::NonStemRoleClaimed,
        ));
        return;
    }

    let mut matches = artifact_set.iter().filter(|artifact| artifact.role == role);
    let Some(artifact) = matches.next() else {
        failures.push(non_silence_failure(
            Some(role),
            StemPackageNonSilenceQaFailureKind::MissingRoleArtifact,
        ));
        return;
    };
    if matches.next().is_some() {
        failures.push(non_silence_failure(
            Some(role),
            StemPackageNonSilenceQaFailureKind::DuplicateRoleArtifact,
        ));
        return;
    }

    match &artifact.audio_metrics {
        Some(metrics) if metrics_prove_silence(metrics) => failures.push(non_silence_failure(
            Some(role),
            StemPackageNonSilenceQaFailureKind::SilentArtifactMetrics,
        )),
        Some(metrics) if metrics_can_prove_activity(metrics) => {}
        Some(_) => failures.push(non_silence_failure(
            Some(role),
            StemPackageNonSilenceQaFailureKind::InsufficientNonSilenceMetrics,
        )),
        None => deferred_checks.push(StemPackageNonSilenceDeferredCheck {
            role,
            check: StemPackageNonSilenceDeferredCheckKind::MissingAudioMetrics,
            status:
                StemPackageNonSilenceDeferredCheckStatus::AspirationalUntilAudioEvidenceAttached,
            reason: "stem artifact does not yet carry audio metrics".into(),
        }),
    }
}

fn non_silence_failure(
    role: Option<ExportArtifactRole>,
    kind: StemPackageNonSilenceQaFailureKind,
) -> StemPackageNonSilenceQaFailure {
    StemPackageNonSilenceQaFailure { role, kind }
}
