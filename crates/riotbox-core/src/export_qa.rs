use serde::{Deserialize, Serialize};

use crate::session::{ExportArtifactAudioMetrics, ExportArtifactRole, ExportArtifactSetEntry};

mod stem_package_non_silence;
pub use stem_package_non_silence::{
    StemPackageNonSilenceDeferredCheck, StemPackageNonSilenceDeferredCheckKind,
    StemPackageNonSilenceDeferredCheckStatus, StemPackageNonSilenceQaFailure,
    StemPackageNonSilenceQaFailureKind, StemPackageNonSilenceQaReport,
    StemPackageNonSilenceQaStatus, validate_stem_package_non_silence_evidence,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageArtifactSetQaReport {
    pub status: StemPackageArtifactSetQaStatus,
    pub claimed_roles: Vec<ExportArtifactRole>,
    pub checked_artifact_count: usize,
    pub failures: Vec<StemPackageArtifactSetQaFailure>,
    pub deferred_checks: Vec<StemPackageDeferredQaCheck>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageHashStabilityQaReport {
    pub status: StemPackageHashStabilityQaStatus,
    pub claimed_roles: Vec<ExportArtifactRole>,
    pub checked_artifact_count: usize,
    pub failures: Vec<StemPackageHashStabilityQaFailure>,
    pub deferred_checks: Vec<StemPackageHashStabilityDeferredCheck>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageArtifactSetQaPolicy {
    #[serde(default)]
    pub require_lineage_evidence: bool,
    #[serde(default)]
    pub require_fallback_comparison_evidence: bool,
}

impl StemPackageArtifactSetQaReport {
    #[must_use]
    pub fn passed_structure_only(&self) -> bool {
        self.status == StemPackageArtifactSetQaStatus::PassedStructureOnly
    }
}

impl StemPackageHashStabilityQaReport {
    #[must_use]
    pub fn passed_identity_only(&self) -> bool {
        self.status == StemPackageHashStabilityQaStatus::PassedIdentityOnly
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageArtifactSetQaStatus {
    PassedStructureOnly,
    Failed,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageHashStabilityQaStatus {
    PassedIdentityOnly,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageArtifactSetQaFailure {
    pub role: Option<ExportArtifactRole>,
    pub kind: StemPackageArtifactSetQaFailureKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageHashStabilityQaFailure {
    pub role: Option<ExportArtifactRole>,
    pub kind: StemPackageHashStabilityQaFailureKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageArtifactSetQaFailureKind {
    NoClaimedStemRoles,
    NonStemRoleClaimed,
    MissingRoleArtifact,
    DuplicateRoleArtifact,
    MissingArtifactLocation,
    MissingArtifactHash,
    MissingLineageEvidence,
    InvalidLineageEvidence,
    MissingFallbackComparisonEvidence,
    InvalidFallbackComparisonEvidence,
    InsufficientNonSilenceMetrics,
    SilentArtifactMetrics,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageHashStabilityQaFailureKind {
    NoClaimedStemRoles,
    NonStemRoleClaimed,
    MissingRoleArtifact,
    DuplicateRoleArtifact,
    MissingArtifactHash,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageDeferredQaCheck {
    pub check: StemPackageDeferredQaCheckKind,
    pub status: StemPackageDeferredQaCheckStatus,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageHashStabilityDeferredCheck {
    pub check: StemPackageHashStabilityDeferredCheckKind,
    pub status: StemPackageHashStabilityDeferredCheckStatus,
    pub reason: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageDeferredQaCheckKind {
    PerStemNonSilence,
    PerStemFallbackCollapse,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageHashStabilityDeferredCheckKind {
    RepeatedRenderHashComparison,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageDeferredQaCheckStatus {
    AspirationalUntilAudioEvidenceAttached,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageHashStabilityDeferredCheckStatus {
    AspirationalUntilPackageWriterExists,
}

pub fn validate_stem_package_artifact_set_evidence(
    artifact_set: &[ExportArtifactSetEntry],
    claimed_roles: &[ExportArtifactRole],
) -> StemPackageArtifactSetQaReport {
    validate_stem_package_artifact_set_evidence_with_policy(
        artifact_set,
        claimed_roles,
        StemPackageArtifactSetQaPolicy::default(),
    )
}

pub fn validate_stem_package_hash_stability_evidence(
    artifact_set: &[ExportArtifactSetEntry],
    claimed_roles: &[ExportArtifactRole],
) -> StemPackageHashStabilityQaReport {
    let mut failures = Vec::new();
    if claimed_roles.is_empty() {
        failures.push(hash_failure(
            None,
            StemPackageHashStabilityQaFailureKind::NoClaimedStemRoles,
        ));
    }

    for role in claimed_roles {
        validate_claimed_role_hash_stability(*role, artifact_set, &mut failures);
    }

    let status = if failures.is_empty() {
        StemPackageHashStabilityQaStatus::PassedIdentityOnly
    } else {
        StemPackageHashStabilityQaStatus::Failed
    };

    StemPackageHashStabilityQaReport {
        status,
        claimed_roles: claimed_roles.to_vec(),
        checked_artifact_count: artifact_set.len(),
        failures,
        deferred_checks: deferred_stem_hash_stability_checks(),
    }
}

pub fn validate_stem_package_artifact_set_evidence_with_policy(
    artifact_set: &[ExportArtifactSetEntry],
    claimed_roles: &[ExportArtifactRole],
    policy: StemPackageArtifactSetQaPolicy,
) -> StemPackageArtifactSetQaReport {
    let mut failures = Vec::new();
    if claimed_roles.is_empty() {
        failures.push(failure(
            None,
            StemPackageArtifactSetQaFailureKind::NoClaimedStemRoles,
        ));
    }

    for role in claimed_roles {
        validate_claimed_role(*role, artifact_set, policy, &mut failures);
    }

    let status = if failures.is_empty() {
        StemPackageArtifactSetQaStatus::PassedStructureOnly
    } else {
        StemPackageArtifactSetQaStatus::Failed
    };

    StemPackageArtifactSetQaReport {
        status,
        claimed_roles: claimed_roles.to_vec(),
        checked_artifact_count: artifact_set.len(),
        failures,
        deferred_checks: deferred_stem_audio_checks(artifact_set, claimed_roles),
    }
}

fn validate_claimed_role_hash_stability(
    role: ExportArtifactRole,
    artifact_set: &[ExportArtifactSetEntry],
    failures: &mut Vec<StemPackageHashStabilityQaFailure>,
) {
    if !role.is_stem_role() {
        failures.push(hash_failure(
            Some(role),
            StemPackageHashStabilityQaFailureKind::NonStemRoleClaimed,
        ));
        return;
    }

    let mut matches = artifact_set.iter().filter(|artifact| artifact.role == role);
    let Some(artifact) = matches.next() else {
        failures.push(hash_failure(
            Some(role),
            StemPackageHashStabilityQaFailureKind::MissingRoleArtifact,
        ));
        return;
    };
    if matches.next().is_some() {
        failures.push(hash_failure(
            Some(role),
            StemPackageHashStabilityQaFailureKind::DuplicateRoleArtifact,
        ));
        return;
    }
    if artifact.sha256.trim().is_empty() {
        failures.push(hash_failure(
            Some(role),
            StemPackageHashStabilityQaFailureKind::MissingArtifactHash,
        ));
    }
}

fn validate_claimed_role(
    role: ExportArtifactRole,
    artifact_set: &[ExportArtifactSetEntry],
    policy: StemPackageArtifactSetQaPolicy,
    failures: &mut Vec<StemPackageArtifactSetQaFailure>,
) {
    if !role.is_stem_role() {
        failures.push(failure(
            Some(role),
            StemPackageArtifactSetQaFailureKind::NonStemRoleClaimed,
        ));
        return;
    }

    let mut matches = artifact_set.iter().filter(|artifact| artifact.role == role);
    let Some(artifact) = matches.next() else {
        failures.push(failure(
            Some(role),
            StemPackageArtifactSetQaFailureKind::MissingRoleArtifact,
        ));
        return;
    };
    if matches.next().is_some() {
        failures.push(failure(
            Some(role),
            StemPackageArtifactSetQaFailureKind::DuplicateRoleArtifact,
        ));
        return;
    }

    validate_stem_artifact_identity(role, artifact, policy, failures);
}

fn validate_stem_artifact_identity(
    role: ExportArtifactRole,
    artifact: &ExportArtifactSetEntry,
    policy: StemPackageArtifactSetQaPolicy,
    failures: &mut Vec<StemPackageArtifactSetQaFailure>,
) {
    if artifact.location_identity().trim().is_empty() {
        failures.push(failure(
            Some(role),
            StemPackageArtifactSetQaFailureKind::MissingArtifactLocation,
        ));
    }
    if artifact.sha256.trim().is_empty() {
        failures.push(failure(
            Some(role),
            StemPackageArtifactSetQaFailureKind::MissingArtifactHash,
        ));
    }
    if policy.require_lineage_evidence && !artifact_has_lineage_evidence(artifact) {
        failures.push(failure(
            Some(role),
            StemPackageArtifactSetQaFailureKind::MissingLineageEvidence,
        ));
    }
    if policy.require_lineage_evidence && artifact_has_invalid_lineage_evidence(artifact) {
        failures.push(failure(
            Some(role),
            StemPackageArtifactSetQaFailureKind::InvalidLineageEvidence,
        ));
    }
    if policy.require_fallback_comparison_evidence {
        match &artifact.fallback_comparison {
            Some(comparison) if fallback_comparison_evidence_is_valid(comparison) => {}
            Some(_) => failures.push(failure(
                Some(role),
                StemPackageArtifactSetQaFailureKind::InvalidFallbackComparisonEvidence,
            )),
            None => failures.push(failure(
                Some(role),
                StemPackageArtifactSetQaFailureKind::MissingFallbackComparisonEvidence,
            )),
        }
    }
    if let Some(metrics) = &artifact.audio_metrics {
        validate_non_silence_metrics(role, metrics, failures);
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

fn fallback_comparison_evidence_is_valid(
    comparison: &crate::session::ExportArtifactFallbackComparisonEvidence,
) -> bool {
    !comparison.reference_identity.trim().is_empty()
        && (comparison.rms_difference_micros.is_some()
            || comparison.normalized_correlation_micros.is_some())
}

fn validate_non_silence_metrics(
    role: ExportArtifactRole,
    metrics: &ExportArtifactAudioMetrics,
    failures: &mut Vec<StemPackageArtifactSetQaFailure>,
) {
    if metrics_prove_silence(metrics) {
        failures.push(failure(
            Some(role),
            StemPackageArtifactSetQaFailureKind::SilentArtifactMetrics,
        ));
        return;
    }

    if !metrics_can_prove_activity(metrics) {
        failures.push(failure(
            Some(role),
            StemPackageArtifactSetQaFailureKind::InsufficientNonSilenceMetrics,
        ));
    }
}

pub(super) fn metrics_prove_silence(metrics: &ExportArtifactAudioMetrics) -> bool {
    if matches!(metrics.total_frame_count, Some(0)) {
        return true;
    }
    if matches!(
        (metrics.silent_frame_count, metrics.total_frame_count),
        (Some(silent), Some(total)) if total > 0 && silent >= total
    ) {
        return true;
    }
    matches!(
        (metrics.peak_amplitude_micros, metrics.rms_amplitude_micros),
        (Some(0), Some(0))
    )
}

pub(super) fn metrics_can_prove_activity(metrics: &ExportArtifactAudioMetrics) -> bool {
    matches!(metrics.peak_amplitude_micros, Some(value) if value > 0)
        || matches!(metrics.rms_amplitude_micros, Some(value) if value > 0)
        || matches!(
            (metrics.silent_frame_count, metrics.total_frame_count),
            (Some(silent), Some(total)) if total > 0 && silent < total
        )
}

fn failure(
    role: Option<ExportArtifactRole>,
    kind: StemPackageArtifactSetQaFailureKind,
) -> StemPackageArtifactSetQaFailure {
    StemPackageArtifactSetQaFailure { role, kind }
}

fn hash_failure(
    role: Option<ExportArtifactRole>,
    kind: StemPackageHashStabilityQaFailureKind,
) -> StemPackageHashStabilityQaFailure {
    StemPackageHashStabilityQaFailure { role, kind }
}

fn deferred_stem_audio_checks(
    artifact_set: &[ExportArtifactSetEntry],
    claimed_roles: &[ExportArtifactRole],
) -> Vec<StemPackageDeferredQaCheck> {
    let mut checks = Vec::new();
    if claimed_roles
        .iter()
        .filter(|role| role.is_stem_role())
        .any(|role| {
            artifact_set
                .iter()
                .find(|artifact| artifact.role == *role)
                .is_none_or(|artifact| artifact.audio_metrics.is_none())
        })
    {
        checks.push(StemPackageDeferredQaCheck {
            check: StemPackageDeferredQaCheckKind::PerStemNonSilence,
            status: StemPackageDeferredQaCheckStatus::AspirationalUntilAudioEvidenceAttached,
            reason: "stem export does not yet attach per-stem audio metrics".into(),
        });
    }
    checks.push(StemPackageDeferredQaCheck {
        check: StemPackageDeferredQaCheckKind::PerStemFallbackCollapse,
        status: StemPackageDeferredQaCheckStatus::AspirationalUntilAudioEvidenceAttached,
        reason: "stem export does not yet attach source-vs-fallback comparison metrics".into(),
    });
    checks
}

fn deferred_stem_hash_stability_checks() -> Vec<StemPackageHashStabilityDeferredCheck> {
    vec![StemPackageHashStabilityDeferredCheck {
        check: StemPackageHashStabilityDeferredCheckKind::RepeatedRenderHashComparison,
        status: StemPackageHashStabilityDeferredCheckStatus::AspirationalUntilPackageWriterExists,
        reason:
            "stem package writer does not yet render repeated package artifacts for hash comparison"
                .into(),
    }]
}

#[cfg(test)]
mod stem_package_evidence_tests;
#[cfg(test)]
mod stem_package_tests;
