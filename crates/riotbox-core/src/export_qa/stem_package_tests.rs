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
    assert!(
        report
            .deferred_checks
            .iter()
            .any(|check| check.check == StemPackageDeferredQaCheckKind::PerStemFallbackCollapse)
    );
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
            fallback_comparison: None,
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
        fallback_comparison: None,
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
