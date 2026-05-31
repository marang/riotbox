use serde::{Deserialize, Serialize};

use crate::session::{ExportArtifactRole, ExportArtifactSetEntry};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageArtifactSetQaReport {
    pub status: StemPackageArtifactSetQaStatus,
    pub claimed_roles: Vec<ExportArtifactRole>,
    pub checked_artifact_count: usize,
    pub failures: Vec<StemPackageArtifactSetQaFailure>,
    pub deferred_checks: Vec<StemPackageDeferredQaCheck>,
}

impl StemPackageArtifactSetQaReport {
    #[must_use]
    pub fn passed_structure_only(&self) -> bool {
        self.status == StemPackageArtifactSetQaStatus::PassedStructureOnly
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StemPackageArtifactSetQaStatus {
    PassedStructureOnly,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageArtifactSetQaFailure {
    pub role: Option<ExportArtifactRole>,
    pub kind: StemPackageArtifactSetQaFailureKind,
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
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemPackageDeferredQaCheck {
    pub check: StemPackageDeferredQaCheckKind,
    pub status: StemPackageDeferredQaCheckStatus,
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
pub enum StemPackageDeferredQaCheckStatus {
    AspirationalUntilAudioEvidenceAttached,
}

pub fn validate_stem_package_artifact_set_evidence(
    artifact_set: &[ExportArtifactSetEntry],
    claimed_roles: &[ExportArtifactRole],
) -> StemPackageArtifactSetQaReport {
    let mut failures = Vec::new();
    if claimed_roles.is_empty() {
        failures.push(failure(
            None,
            StemPackageArtifactSetQaFailureKind::NoClaimedStemRoles,
        ));
    }

    for role in claimed_roles {
        validate_claimed_role(*role, artifact_set, &mut failures);
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
        deferred_checks: deferred_stem_audio_checks(),
    }
}

fn validate_claimed_role(
    role: ExportArtifactRole,
    artifact_set: &[ExportArtifactSetEntry],
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

    validate_stem_artifact_identity(role, artifact, failures);
}

fn validate_stem_artifact_identity(
    role: ExportArtifactRole,
    artifact: &ExportArtifactSetEntry,
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
}

fn failure(
    role: Option<ExportArtifactRole>,
    kind: StemPackageArtifactSetQaFailureKind,
) -> StemPackageArtifactSetQaFailure {
    StemPackageArtifactSetQaFailure { role, kind }
}

fn deferred_stem_audio_checks() -> Vec<StemPackageDeferredQaCheck> {
    vec![
        StemPackageDeferredQaCheck {
            check: StemPackageDeferredQaCheckKind::PerStemNonSilence,
            status: StemPackageDeferredQaCheckStatus::AspirationalUntilAudioEvidenceAttached,
            reason: "stem export does not yet attach per-stem audio metrics".into(),
        },
        StemPackageDeferredQaCheck {
            check: StemPackageDeferredQaCheckKind::PerStemFallbackCollapse,
            status: StemPackageDeferredQaCheckStatus::AspirationalUntilAudioEvidenceAttached,
            reason: "stem export does not yet attach source-vs-fallback comparison metrics".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::session::{ExportArtifactLocation, ExportArtifactMediaType};

    #[test]
    fn stem_package_gate_passes_structure_without_claiming_audio_evidence() {
        let artifact_set = vec![
            stem_artifact(
                ExportArtifactRole::StemDrums,
                "exports/stems/drums.wav",
                "a",
            ),
            stem_artifact(ExportArtifactRole::StemBass, "exports/stems/bass.wav", "b"),
        ];

        let report = validate_stem_package_artifact_set_evidence(
            &artifact_set,
            &[ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        );

        assert!(report.passed_structure_only());
        assert!(report.failures.is_empty());
        assert_eq!(report.checked_artifact_count, 2);
        assert_eq!(report.deferred_checks.len(), 2);
        assert!(report.deferred_checks.iter().any(|check| {
            check.check == StemPackageDeferredQaCheckKind::PerStemNonSilence
                && check.status
                    == StemPackageDeferredQaCheckStatus::AspirationalUntilAudioEvidenceAttached
        }));
    }

    #[test]
    fn stem_package_gate_fails_missing_role_and_non_stem_claims() {
        let artifact_set = vec![stem_artifact(
            ExportArtifactRole::StemDrums,
            "exports/stems/drums.wav",
            "a",
        )];

        let report = validate_stem_package_artifact_set_evidence(
            &artifact_set,
            &[
                ExportArtifactRole::StemDrums,
                ExportArtifactRole::StemBass,
                ExportArtifactRole::FullGridMix,
            ],
        );

        assert_eq!(report.status, StemPackageArtifactSetQaStatus::Failed);
        assert!(report.failures.contains(&failure(
            Some(ExportArtifactRole::StemBass),
            StemPackageArtifactSetQaFailureKind::MissingRoleArtifact,
        )));
        assert!(report.failures.contains(&failure(
            Some(ExportArtifactRole::FullGridMix),
            StemPackageArtifactSetQaFailureKind::NonStemRoleClaimed,
        )));
    }

    #[test]
    fn stem_package_gate_fails_missing_location_hash_and_duplicates() {
        let artifact_set = vec![
            ExportArtifactSetEntry {
                role: ExportArtifactRole::StemDrums,
                location: ExportArtifactLocation::LocalPath { path: " ".into() },
                media_type: ExportArtifactMediaType::AudioWav,
                sha256: " ".into(),
                audio_metrics: None,
                sample_rate_hz: None,
                channel_count: None,
                duration_ms: None,
            },
            stem_artifact(
                ExportArtifactRole::StemBass,
                "exports/stems/bass-a.wav",
                "b",
            ),
            stem_artifact(
                ExportArtifactRole::StemBass,
                "exports/stems/bass-b.wav",
                "c",
            ),
        ];

        let report = validate_stem_package_artifact_set_evidence(
            &artifact_set,
            &[ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        );

        assert!(report.failures.contains(&failure(
            Some(ExportArtifactRole::StemDrums),
            StemPackageArtifactSetQaFailureKind::MissingArtifactLocation,
        )));
        assert!(report.failures.contains(&failure(
            Some(ExportArtifactRole::StemDrums),
            StemPackageArtifactSetQaFailureKind::MissingArtifactHash,
        )));
        assert!(report.failures.contains(&failure(
            Some(ExportArtifactRole::StemBass),
            StemPackageArtifactSetQaFailureKind::DuplicateRoleArtifact,
        )));
    }

    fn stem_artifact(
        role: ExportArtifactRole,
        path: impl Into<String>,
        sha256: impl Into<String>,
    ) -> ExportArtifactSetEntry {
        ExportArtifactSetEntry {
            role,
            location: ExportArtifactLocation::LocalPath { path: path.into() },
            media_type: ExportArtifactMediaType::AudioWav,
            sha256: sha256.into(),
            audio_metrics: None,
            sample_rate_hz: None,
            channel_count: None,
            duration_ms: None,
        }
    }
}
