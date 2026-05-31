use super::*;

use crate::{
    ids::{CaptureId, SourceId},
    session::{
        ExportArtifactFallbackComparisonEvidence, ExportArtifactFallbackComparisonKind,
        ExportArtifactLocation, ExportArtifactMediaType, ExportArtifactSourceGraphRef,
    },
    source_graph::SourceGraphVersion,
};

#[test]
fn stem_package_gate_preserves_default_without_lineage_requirement() {
    let artifact_set = vec![stem_artifact()];

    let report = validate_stem_package_artifact_set_evidence(
        &artifact_set,
        &[ExportArtifactRole::StemDrums],
    );

    assert!(report.passed_structure_only());
    assert!(!report.failures.contains(&failure(
        Some(ExportArtifactRole::StemDrums),
        StemPackageArtifactSetQaFailureKind::MissingLineageEvidence,
    )));
}

#[test]
fn stem_package_gate_can_require_lineage_evidence() {
    let artifact_set = vec![stem_artifact()];

    let report = validate_stem_package_artifact_set_evidence_with_policy(
        &artifact_set,
        &[ExportArtifactRole::StemDrums],
        StemPackageArtifactSetQaPolicy {
            require_lineage_evidence: true,
            ..Default::default()
        },
    );

    assert_eq!(report.status, StemPackageArtifactSetQaStatus::Failed);
    assert!(report.failures.contains(&failure(
        Some(ExportArtifactRole::StemDrums),
        StemPackageArtifactSetQaFailureKind::MissingLineageEvidence,
    )));
}

#[test]
fn stem_package_gate_accepts_source_or_capture_lineage_evidence_when_required() {
    for artifact in [
        stem_artifact_with_source_graph_ref(),
        stem_artifact_with_source_capture_ref(),
        stem_artifact_with_lineage_capture_ref(),
    ] {
        let artifact_set = vec![artifact];

        let report = validate_stem_package_artifact_set_evidence_with_policy(
            &artifact_set,
            &[ExportArtifactRole::StemDrums],
            StemPackageArtifactSetQaPolicy {
                require_lineage_evidence: true,
                ..Default::default()
            },
        );

        assert!(report.passed_structure_only());
        assert!(!report.failures.contains(&failure(
            Some(ExportArtifactRole::StemDrums),
            StemPackageArtifactSetQaFailureKind::MissingLineageEvidence,
        )));
    }
}

#[test]
fn stem_package_gate_rejects_blank_lineage_identity_when_required() {
    for artifact in [
        stem_artifact_with_blank_source_graph_ref(),
        stem_artifact_with_blank_source_capture_ref(),
        stem_artifact_with_blank_lineage_capture_ref(),
    ] {
        let artifact_set = vec![artifact];

        let report = validate_stem_package_artifact_set_evidence_with_policy(
            &artifact_set,
            &[ExportArtifactRole::StemDrums],
            StemPackageArtifactSetQaPolicy {
                require_lineage_evidence: true,
                ..Default::default()
            },
        );

        assert_eq!(report.status, StemPackageArtifactSetQaStatus::Failed);
        assert!(report.failures.contains(&failure(
            Some(ExportArtifactRole::StemDrums),
            StemPackageArtifactSetQaFailureKind::InvalidLineageEvidence,
        )));
    }
}

#[test]
fn stem_package_gate_preserves_default_without_fallback_comparison_requirement() {
    let artifact_set = vec![stem_artifact()];

    let report = validate_stem_package_artifact_set_evidence(
        &artifact_set,
        &[ExportArtifactRole::StemDrums],
    );

    assert!(report.passed_structure_only());
    assert!(!report.failures.contains(&failure(
        Some(ExportArtifactRole::StemDrums),
        StemPackageArtifactSetQaFailureKind::MissingFallbackComparisonEvidence,
    )));
}

#[test]
fn stem_package_gate_can_require_fallback_comparison_evidence() {
    let artifact_set = vec![stem_artifact()];

    let report = validate_stem_package_artifact_set_evidence_with_policy(
        &artifact_set,
        &[ExportArtifactRole::StemDrums],
        StemPackageArtifactSetQaPolicy {
            require_fallback_comparison_evidence: true,
            ..Default::default()
        },
    );

    assert_eq!(report.status, StemPackageArtifactSetQaStatus::Failed);
    assert!(report.failures.contains(&failure(
        Some(ExportArtifactRole::StemDrums),
        StemPackageArtifactSetQaFailureKind::MissingFallbackComparisonEvidence,
    )));
}

#[test]
fn stem_package_gate_accepts_fallback_comparison_evidence_when_required() {
    let artifact_set = vec![stem_artifact_with_fallback_comparison()];

    let report = validate_stem_package_artifact_set_evidence_with_policy(
        &artifact_set,
        &[ExportArtifactRole::StemDrums],
        StemPackageArtifactSetQaPolicy {
            require_fallback_comparison_evidence: true,
            ..Default::default()
        },
    );

    assert!(report.passed_structure_only());
    assert!(!report.failures.contains(&failure(
        Some(ExportArtifactRole::StemDrums),
        StemPackageArtifactSetQaFailureKind::MissingFallbackComparisonEvidence,
    )));
}

#[test]
fn stem_package_gate_rejects_blank_fallback_comparison_identity_when_required() {
    let artifact_set = vec![stem_artifact_with_blank_fallback_comparison_identity()];

    let report = validate_stem_package_artifact_set_evidence_with_policy(
        &artifact_set,
        &[ExportArtifactRole::StemDrums],
        StemPackageArtifactSetQaPolicy {
            require_fallback_comparison_evidence: true,
            ..Default::default()
        },
    );

    assert_eq!(report.status, StemPackageArtifactSetQaStatus::Failed);
    assert!(report.failures.contains(&failure(
        Some(ExportArtifactRole::StemDrums),
        StemPackageArtifactSetQaFailureKind::InvalidFallbackComparisonEvidence,
    )));
}

#[test]
fn stem_package_gate_rejects_metricless_fallback_comparison_when_required() {
    let artifact_set = vec![stem_artifact_with_metricless_fallback_comparison()];

    let report = validate_stem_package_artifact_set_evidence_with_policy(
        &artifact_set,
        &[ExportArtifactRole::StemDrums],
        StemPackageArtifactSetQaPolicy {
            require_fallback_comparison_evidence: true,
            ..Default::default()
        },
    );

    assert_eq!(report.status, StemPackageArtifactSetQaStatus::Failed);
    assert!(report.failures.contains(&failure(
        Some(ExportArtifactRole::StemDrums),
        StemPackageArtifactSetQaFailureKind::InvalidFallbackComparisonEvidence,
    )));
}

fn stem_artifact() -> ExportArtifactSetEntry {
    ExportArtifactSetEntry {
        role: ExportArtifactRole::StemDrums,
        location: ExportArtifactLocation::LocalPath {
            path: "exports/stems/drums.wav".into(),
        },
        media_type: ExportArtifactMediaType::AudioWav,
        sha256: "a".into(),
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

fn stem_artifact_with_source_graph_ref() -> ExportArtifactSetEntry {
    let mut artifact = stem_artifact();
    artifact.source_graph_ref = Some(ExportArtifactSourceGraphRef {
        source_id: SourceId::from("src-1"),
        graph_version: SourceGraphVersion::V1,
        graph_hash: "graph-hash-1".into(),
    });
    artifact
}

fn stem_artifact_with_source_capture_ref() -> ExportArtifactSetEntry {
    let mut artifact = stem_artifact();
    artifact.source_capture_refs = vec![CaptureId::from("cap-source")];
    artifact
}

fn stem_artifact_with_lineage_capture_ref() -> ExportArtifactSetEntry {
    let mut artifact = stem_artifact();
    artifact.lineage_capture_refs = vec![CaptureId::from("cap-root")];
    artifact
}

fn stem_artifact_with_blank_source_graph_ref() -> ExportArtifactSetEntry {
    let mut artifact = stem_artifact();
    artifact.source_graph_ref = Some(ExportArtifactSourceGraphRef {
        source_id: SourceId::from(" "),
        graph_version: SourceGraphVersion::V1,
        graph_hash: " ".into(),
    });
    artifact
}

fn stem_artifact_with_blank_source_capture_ref() -> ExportArtifactSetEntry {
    let mut artifact = stem_artifact();
    artifact.source_capture_refs = vec![CaptureId::from(" ")];
    artifact
}

fn stem_artifact_with_blank_lineage_capture_ref() -> ExportArtifactSetEntry {
    let mut artifact = stem_artifact();
    artifact.lineage_capture_refs = vec![CaptureId::from(" ")];
    artifact
}

fn stem_artifact_with_fallback_comparison() -> ExportArtifactSetEntry {
    let mut artifact = stem_artifact();
    artifact.fallback_comparison = Some(ExportArtifactFallbackComparisonEvidence {
        comparison_kind: ExportArtifactFallbackComparisonKind::SourceVsFallback,
        reference_identity: "fallback://stem-drums".into(),
        rms_difference_micros: Some(125_000),
        normalized_correlation_micros: Some(420_000),
    });
    artifact
}

fn stem_artifact_with_blank_fallback_comparison_identity() -> ExportArtifactSetEntry {
    let mut artifact = stem_artifact_with_fallback_comparison();
    artifact
        .fallback_comparison
        .as_mut()
        .expect("fallback comparison")
        .reference_identity = " ".into();
    artifact
}

fn stem_artifact_with_metricless_fallback_comparison() -> ExportArtifactSetEntry {
    let mut artifact = stem_artifact_with_fallback_comparison();
    let comparison = artifact
        .fallback_comparison
        .as_mut()
        .expect("fallback comparison");
    comparison.rms_difference_micros = None;
    comparison.normalized_correlation_micros = None;
    artifact
}
