use serde::{Deserialize, Serialize};

use crate::session::{ExportArtifactAudioMetrics, ExportArtifactRole, ExportArtifactSetEntry};

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
    InsufficientNonSilenceMetrics,
    SilentArtifactMetrics,
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
        deferred_checks: deferred_stem_audio_checks(artifact_set, claimed_roles),
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
    if let Some(metrics) = &artifact.audio_metrics {
        validate_non_silence_metrics(role, metrics, failures);
    }
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

fn metrics_prove_silence(metrics: &ExportArtifactAudioMetrics) -> bool {
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

fn metrics_can_prove_activity(metrics: &ExportArtifactAudioMetrics) -> bool {
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
    fn stem_package_gate_enforces_non_silence_when_metrics_exist() {
        let artifact_set = vec![stem_artifact_with_metrics(
            ExportArtifactRole::StemDrums,
            "exports/stems/drums.wav",
            "a",
            active_metrics(),
        )];

        let report = validate_stem_package_artifact_set_evidence(
            &artifact_set,
            &[ExportArtifactRole::StemDrums],
        );

        assert!(report.passed_structure_only());
        assert!(
            !report
                .deferred_checks
                .iter()
                .any(|check| check.check == StemPackageDeferredQaCheckKind::PerStemNonSilence)
        );
        assert!(report
            .deferred_checks
            .iter()
            .any(|check| check.check == StemPackageDeferredQaCheckKind::PerStemFallbackCollapse));
    }

    #[test]
    fn stem_package_gate_fails_metrics_that_prove_silence() {
        let artifact_set = vec![stem_artifact_with_metrics(
            ExportArtifactRole::StemDrums,
            "exports/stems/drums.wav",
            "a",
            silent_metrics(),
        )];

        let report = validate_stem_package_artifact_set_evidence(
            &artifact_set,
            &[ExportArtifactRole::StemDrums],
        );

        assert_eq!(report.status, StemPackageArtifactSetQaStatus::Failed);
        assert!(report.failures.contains(&failure(
            Some(ExportArtifactRole::StemDrums),
            StemPackageArtifactSetQaFailureKind::SilentArtifactMetrics,
        )));
    }

    #[test]
    fn stem_package_gate_fails_metrics_without_activity_evidence() {
        let artifact_set = vec![stem_artifact_with_metrics(
            ExportArtifactRole::StemDrums,
            "exports/stems/drums.wav",
            "a",
            ExportArtifactAudioMetrics {
                peak_milli_dbfs: Some(-12_000),
                rms_milli_dbfs: None,
                peak_amplitude_micros: None,
                rms_amplitude_micros: None,
                silent_frame_count: None,
                total_frame_count: None,
            },
        )];

        let report = validate_stem_package_artifact_set_evidence(
            &artifact_set,
            &[ExportArtifactRole::StemDrums],
        );

        assert_eq!(report.status, StemPackageArtifactSetQaStatus::Failed);
        assert!(report.failures.contains(&failure(
            Some(ExportArtifactRole::StemDrums),
            StemPackageArtifactSetQaFailureKind::InsufficientNonSilenceMetrics,
        )));
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
                source_graph_ref: None,
                source_capture_refs: Vec::new(),
                lineage_capture_refs: Vec::new(),
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
            source_graph_ref: None,
            source_capture_refs: Vec::new(),
            lineage_capture_refs: Vec::new(),
            audio_metrics: None,
            sample_rate_hz: None,
            channel_count: None,
            duration_ms: None,
        }
    }

    fn stem_artifact_with_metrics(
        role: ExportArtifactRole,
        path: impl Into<String>,
        sha256: impl Into<String>,
        metrics: ExportArtifactAudioMetrics,
    ) -> ExportArtifactSetEntry {
        let mut artifact = stem_artifact(role, path, sha256);
        artifact.audio_metrics = Some(metrics);
        artifact
    }

    fn active_metrics() -> ExportArtifactAudioMetrics {
        ExportArtifactAudioMetrics {
            peak_milli_dbfs: Some(-120),
            rms_milli_dbfs: Some(-6_000),
            peak_amplitude_micros: Some(986_000),
            rms_amplitude_micros: Some(125_000),
            silent_frame_count: Some(0),
            total_frame_count: Some(96_000),
        }
    }

    fn silent_metrics() -> ExportArtifactAudioMetrics {
        ExportArtifactAudioMetrics {
            peak_milli_dbfs: None,
            rms_milli_dbfs: None,
            peak_amplitude_micros: Some(0),
            rms_amplitude_micros: Some(0),
            silent_frame_count: Some(96_000),
            total_frame_count: Some(96_000),
        }
    }
}
