use super::*;

use crate::session::{
    ExportArtifactFallbackComparisonEvidence, ExportArtifactFallbackComparisonKind,
    ExportArtifactLocation, ExportArtifactMediaType, ExportArtifactSetEntry,
};

#[test]
fn stem_package_fallback_comparison_gate_passes_when_all_claimed_stems_have_metrics() {
    let artifact_set = vec![
        stem_artifact_with_fallback_comparison(
            ExportArtifactRole::StemDrums,
            "exports/stems/drums.wav",
            "drums-sha",
        ),
        stem_artifact_with_fallback_comparison(
            ExportArtifactRole::StemBass,
            "exports/stems/bass.wav",
            "bass-sha",
        ),
    ];

    let report = validate_stem_package_fallback_comparison_evidence(
        &artifact_set,
        &[ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
    );
    let gate = crate::session::ExportReceiptQaGateResult::stem_package_fallback_comparison(&report);

    assert!(report.passed());
    assert!(report.failures.is_empty());
    assert_eq!(
        gate.gate_id,
        crate::session::STEM_PACKAGE_FALLBACK_COMPARISON_QA_GATE_ID
    );
    assert_eq!(
        gate.status,
        crate::session::ExportReceiptQaGateStatus::Passed
    );
}

#[test]
fn stem_package_fallback_comparison_gate_fails_missing_invalid_duplicate_and_role_errors() {
    let artifact_set = vec![
        stem_artifact(
            ExportArtifactRole::StemDrums,
            "exports/stems/drums.wav",
            "drums-sha",
        ),
        stem_artifact_with_metricless_fallback_comparison(
            ExportArtifactRole::StemBass,
            "exports/stems/bass.wav",
            "bass-sha",
        ),
        stem_artifact_with_fallback_comparison(
            ExportArtifactRole::StemVocals,
            "exports/stems/vocals-a.wav",
            "vocals-a-sha",
        ),
        stem_artifact_with_fallback_comparison(
            ExportArtifactRole::StemVocals,
            "exports/stems/vocals-b.wav",
            "vocals-b-sha",
        ),
    ];

    let report = validate_stem_package_fallback_comparison_evidence(
        &artifact_set,
        &[
            ExportArtifactRole::StemDrums,
            ExportArtifactRole::StemBass,
            ExportArtifactRole::StemMusic,
            ExportArtifactRole::StemVocals,
            ExportArtifactRole::FullGridMix,
        ],
    );
    let gate = crate::session::ExportReceiptQaGateResult::stem_package_fallback_comparison(&report);

    assert_eq!(report.status, StemPackageFallbackComparisonQaStatus::Failed);
    assert!(
        report
            .failures
            .contains(&StemPackageFallbackComparisonQaFailure {
                role: Some(ExportArtifactRole::StemDrums),
                kind: StemPackageFallbackComparisonQaFailureKind::MissingFallbackComparisonEvidence,
            })
    );
    assert!(
        report
            .failures
            .contains(&StemPackageFallbackComparisonQaFailure {
                role: Some(ExportArtifactRole::StemBass),
                kind: StemPackageFallbackComparisonQaFailureKind::InvalidFallbackComparisonEvidence,
            })
    );
    assert!(
        report
            .failures
            .contains(&StemPackageFallbackComparisonQaFailure {
                role: Some(ExportArtifactRole::StemMusic),
                kind: StemPackageFallbackComparisonQaFailureKind::MissingRoleArtifact,
            })
    );
    assert!(
        report
            .failures
            .contains(&StemPackageFallbackComparisonQaFailure {
                role: Some(ExportArtifactRole::StemVocals),
                kind: StemPackageFallbackComparisonQaFailureKind::DuplicateRoleArtifact,
            })
    );
    assert!(
        report
            .failures
            .contains(&StemPackageFallbackComparisonQaFailure {
                role: Some(ExportArtifactRole::FullGridMix),
                kind: StemPackageFallbackComparisonQaFailureKind::NonStemRoleClaimed,
            })
    );
    assert_eq!(
        gate.status,
        crate::session::ExportReceiptQaGateStatus::Failed
    );
}

#[test]
fn stem_package_fallback_comparison_gate_fails_empty_claims_and_blank_reference_identity() {
    let report = validate_stem_package_fallback_comparison_evidence(&[], &[]);

    assert_eq!(report.status, StemPackageFallbackComparisonQaStatus::Failed);
    assert!(
        report
            .failures
            .contains(&StemPackageFallbackComparisonQaFailure {
                role: None,
                kind: StemPackageFallbackComparisonQaFailureKind::NoClaimedStemRoles,
            })
    );

    let artifact = stem_artifact_with_blank_fallback_reference_identity(
        ExportArtifactRole::StemDrums,
        "exports/stems/drums.wav",
        "drums-sha",
    );
    let report = validate_stem_package_fallback_comparison_evidence(
        &[artifact],
        &[ExportArtifactRole::StemDrums],
    );

    assert_eq!(report.status, StemPackageFallbackComparisonQaStatus::Failed);
    assert!(
        report
            .failures
            .contains(&StemPackageFallbackComparisonQaFailure {
                role: Some(ExportArtifactRole::StemDrums),
                kind: StemPackageFallbackComparisonQaFailureKind::InvalidFallbackComparisonEvidence,
            })
    );
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
        normalized_manifest_hash: None,
        source_graph_ref: None,
        timing_grid_ref: None,
        source_capture_refs: Vec::new(),
        lineage_capture_refs: Vec::new(),
        fallback_comparison: None,
        audio_metrics: None,
        sample_rate_hz: None,
        channel_count: None,
        duration_ms: None,
    }
}

fn stem_artifact_with_fallback_comparison(
    role: ExportArtifactRole,
    path: impl Into<String>,
    sha256: impl Into<String>,
) -> ExportArtifactSetEntry {
    let mut artifact = stem_artifact(role, path, sha256);
    artifact.fallback_comparison = Some(ExportArtifactFallbackComparisonEvidence {
        comparison_kind: ExportArtifactFallbackComparisonKind::SourceVsFallback,
        reference_identity: "fallback://stem".into(),
        rms_difference_micros: Some(125_000),
        normalized_correlation_micros: Some(420_000),
    });
    artifact
}

fn stem_artifact_with_blank_fallback_reference_identity(
    role: ExportArtifactRole,
    path: impl Into<String>,
    sha256: impl Into<String>,
) -> ExportArtifactSetEntry {
    let mut artifact = stem_artifact_with_fallback_comparison(role, path, sha256);
    artifact
        .fallback_comparison
        .as_mut()
        .expect("fallback comparison")
        .reference_identity = " ".into();
    artifact
}

fn stem_artifact_with_metricless_fallback_comparison(
    role: ExportArtifactRole,
    path: impl Into<String>,
    sha256: impl Into<String>,
) -> ExportArtifactSetEntry {
    let mut artifact = stem_artifact_with_fallback_comparison(role, path, sha256);
    let comparison = artifact
        .fallback_comparison
        .as_mut()
        .expect("fallback comparison");
    comparison.rms_difference_micros = None;
    comparison.normalized_correlation_micros = None;
    artifact
}
