use super::*;

use crate::{
    ids::{CaptureId, SourceId},
    session::{
        ExportArtifactLocation, ExportArtifactMediaType, ExportArtifactSetEntry,
        ExportArtifactSourceGraphRef,
    },
    source_graph::SourceGraphVersion,
};

#[test]
fn stem_package_lineage_gate_passes_when_all_claimed_stems_have_core_lineage() {
    let artifact_set = vec![
        stem_artifact_with_source_graph(
            ExportArtifactRole::StemDrums,
            "exports/stems/drums.wav",
            "drums-sha",
        ),
        stem_artifact_with_capture_refs(
            ExportArtifactRole::StemBass,
            "exports/stems/bass.wav",
            "bass-sha",
        ),
    ];

    let report = validate_stem_package_lineage_evidence(
        &artifact_set,
        &[ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
    );
    let gate = crate::session::ExportReceiptQaGateResult::stem_package_lineage(&report);

    assert!(report.passed());
    assert!(report.failures.is_empty());
    assert_eq!(
        gate.gate_id,
        crate::session::STEM_PACKAGE_LINEAGE_QA_GATE_ID
    );
    assert_eq!(
        gate.status,
        crate::session::ExportReceiptQaGateStatus::Passed
    );
}

#[test]
fn stem_package_lineage_gate_fails_missing_invalid_duplicate_and_role_errors() {
    let mut invalid_source_graph = stem_artifact_with_source_graph(
        ExportArtifactRole::StemDrums,
        "exports/stems/drums.wav",
        "drums-sha",
    );
    invalid_source_graph.source_graph_ref = Some(ExportArtifactSourceGraphRef {
        source_id: SourceId::new(" "),
        graph_version: SourceGraphVersion::V1,
        graph_hash: " ".into(),
    });

    let artifact_set = vec![
        invalid_source_graph,
        stem_artifact(
            ExportArtifactRole::StemBass,
            "exports/stems/bass.wav",
            "bass-sha",
        ),
        stem_artifact_with_capture_refs(
            ExportArtifactRole::StemVocals,
            "exports/stems/vocals-a.wav",
            "vocals-a-sha",
        ),
        stem_artifact_with_capture_refs(
            ExportArtifactRole::StemVocals,
            "exports/stems/vocals-b.wav",
            "vocals-b-sha",
        ),
    ];

    let report = validate_stem_package_lineage_evidence(
        &artifact_set,
        &[
            ExportArtifactRole::StemDrums,
            ExportArtifactRole::StemBass,
            ExportArtifactRole::StemMusic,
            ExportArtifactRole::StemVocals,
            ExportArtifactRole::FullGridMix,
        ],
    );
    let gate = crate::session::ExportReceiptQaGateResult::stem_package_lineage(&report);

    assert_eq!(report.status, StemPackageLineageQaStatus::Failed);
    assert!(report.failures.contains(&StemPackageLineageQaFailure {
        role: Some(ExportArtifactRole::StemDrums),
        kind: StemPackageLineageQaFailureKind::InvalidLineageEvidence,
    }));
    assert!(report.failures.contains(&StemPackageLineageQaFailure {
        role: Some(ExportArtifactRole::StemBass),
        kind: StemPackageLineageQaFailureKind::MissingLineageEvidence,
    }));
    assert!(report.failures.contains(&StemPackageLineageQaFailure {
        role: Some(ExportArtifactRole::StemMusic),
        kind: StemPackageLineageQaFailureKind::MissingRoleArtifact,
    }));
    assert!(report.failures.contains(&StemPackageLineageQaFailure {
        role: Some(ExportArtifactRole::StemVocals),
        kind: StemPackageLineageQaFailureKind::DuplicateRoleArtifact,
    }));
    assert!(report.failures.contains(&StemPackageLineageQaFailure {
        role: Some(ExportArtifactRole::FullGridMix),
        kind: StemPackageLineageQaFailureKind::NonStemRoleClaimed,
    }));
    assert_eq!(
        gate.status,
        crate::session::ExportReceiptQaGateStatus::Failed
    );
}

#[test]
fn stem_package_lineage_gate_fails_empty_claims_and_blank_capture_refs() {
    let report = validate_stem_package_lineage_evidence(&[], &[]);

    assert_eq!(report.status, StemPackageLineageQaStatus::Failed);
    assert!(report.failures.contains(&StemPackageLineageQaFailure {
        role: None,
        kind: StemPackageLineageQaFailureKind::NoClaimedStemRoles,
    }));

    let mut artifact = stem_artifact(
        ExportArtifactRole::StemDrums,
        "exports/stems/drums.wav",
        "drums-sha",
    );
    artifact.source_capture_refs.push(CaptureId::new(" "));

    let report =
        validate_stem_package_lineage_evidence(&[artifact], &[ExportArtifactRole::StemDrums]);

    assert_eq!(report.status, StemPackageLineageQaStatus::Failed);
    assert!(report.failures.contains(&StemPackageLineageQaFailure {
        role: Some(ExportArtifactRole::StemDrums),
        kind: StemPackageLineageQaFailureKind::InvalidLineageEvidence,
    }));
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

fn stem_artifact_with_source_graph(
    role: ExportArtifactRole,
    path: impl Into<String>,
    sha256: impl Into<String>,
) -> ExportArtifactSetEntry {
    let mut artifact = stem_artifact(role, path, sha256);
    artifact.source_graph_ref = Some(ExportArtifactSourceGraphRef {
        source_id: SourceId::new("source-1"),
        graph_version: SourceGraphVersion::V1,
        graph_hash: "graph-sha".into(),
    });
    artifact
}

fn stem_artifact_with_capture_refs(
    role: ExportArtifactRole,
    path: impl Into<String>,
    sha256: impl Into<String>,
) -> ExportArtifactSetEntry {
    let mut artifact = stem_artifact(role, path, sha256);
    artifact
        .source_capture_refs
        .push(CaptureId::new("capture-1"));
    artifact
}
