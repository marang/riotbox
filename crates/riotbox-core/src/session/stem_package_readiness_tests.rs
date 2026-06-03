use super::*;

use crate::{
    export_qa::{
        StemPackageArtifactSetQaPolicy, validate_stem_package_artifact_set_evidence_with_policy,
        validate_stem_package_fallback_comparison_evidence, validate_stem_package_lineage_evidence,
        validate_stem_package_non_silence_evidence,
    },
    export_readiness::{
        EXPORT_READINESS_CONTRACT_SCHEMA, ProductExportBoundary, ProductExportRole,
        STEM_PACKAGE_LOCAL_CI_PACK_ID,
    },
    ids::{ActionId, SourceId},
    session::{
        ExportArtifactAudioMetrics, ExportArtifactFallbackComparisonEvidence,
        ExportArtifactFallbackComparisonKind, ExportArtifactLocation, ExportArtifactMediaType,
        ExportArtifactSourceGraphRef,
    },
    source_graph::SourceGraphVersion,
};

fn stem_package_receipt() -> ExportReceiptState {
    let contract = ExportReadinessContract {
        schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: crate::stem_package_proof::STEM_PACKAGE_PROOF_SCHEMA_ID.into(),
        export_scope: ExportScope::StemPackage,
        boundary: ProductExportBoundary::StemPackageLocalCiPackageV1,
        pack_id: STEM_PACKAGE_LOCAL_CI_PACK_ID.into(),
        export_role: ProductExportRole::PackageManifest,
        export_artifact: "run-a/stem_package_manifest.json".into(),
        source_sha256: "eeee".into(),
        export_sha256: "aaaa".into(),
        normalized_manifest_sha256: "dddd".into(),
        unsupported_scopes: vec![UnsupportedExportScope::StemPackage],
    };

    ExportReceiptState::from_readiness_contract(
        ActionId(7),
        900,
        &contract,
        "exports/stem_package_manifest.json",
        "exports/stem_package_proof.json",
        Some("exports/stem_package_manifest.json".into()),
    )
}

#[test]
fn stem_package_readiness_blocks_missing_artifact_set_gate() {
    let receipt = stem_package_receipt();

    let report = validate_stem_package_receipt_readiness(&receipt);

    assert_eq!(report.status, StemPackageReceiptReadinessStatus::Blocked);
    assert!(
        report
            .blockers
            .contains(&StemPackageReceiptReadinessBlocker::UnsupportedScopeFlagPresent)
    );
    assert!(
        report
            .blockers
            .contains(&StemPackageReceiptReadinessBlocker::MissingArtifactSetQaGate)
    );
    assert!(
        report
            .blockers
            .contains(&StemPackageReceiptReadinessBlocker::MissingHashStabilityQaGate)
    );
    assert!(
        report
            .blockers
            .contains(&StemPackageReceiptReadinessBlocker::MissingNonSilenceQaGate)
    );
    assert!(
        report
            .blockers
            .contains(&StemPackageReceiptReadinessBlocker::MissingLineageQaGate)
    );
    assert!(
        report
            .blockers
            .contains(&StemPackageReceiptReadinessBlocker::MissingFallbackComparisonQaGate)
    );
}

#[test]
fn stem_package_readiness_blocks_deferred_artifact_set_gate() {
    let mut receipt = stem_package_receipt();
    receipt.qa_gates = all_required_stem_package_gates(ExportReceiptQaGateStatus::Passed);
    replace_gate_status(
        &mut receipt,
        STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID,
        ExportReceiptQaGateStatus::Deferred,
    );

    let report = receipt.stem_package_readiness_report();

    assert!(!report.ready());
    assert!(
        report
            .blockers
            .contains(&StemPackageReceiptReadinessBlocker::DeferredArtifactSetQaGate)
    );
}

#[test]
fn stem_package_readiness_blocks_failed_artifact_set_gate() {
    let mut receipt = stem_package_receipt();
    receipt.qa_gates = all_required_stem_package_gates(ExportReceiptQaGateStatus::Passed);
    replace_gate_status(
        &mut receipt,
        STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID,
        ExportReceiptQaGateStatus::Failed,
    );

    let report = receipt.stem_package_readiness_report();

    assert_eq!(report.status, StemPackageReceiptReadinessStatus::Blocked);
    assert!(
        report
            .blockers
            .contains(&StemPackageReceiptReadinessBlocker::FailedArtifactSetQaGate)
    );
}

#[test]
fn stem_package_readiness_blocks_passed_gate_when_scope_is_still_unsupported() {
    let mut receipt = stem_package_receipt();
    receipt.qa_gates = all_required_stem_package_gates(ExportReceiptQaGateStatus::Passed);

    let report = receipt.stem_package_readiness_report();

    assert!(!report.ready());
    assert_eq!(
        report.blockers,
        vec![StemPackageReceiptReadinessBlocker::UnsupportedScopeFlagPresent]
    );
}

#[test]
fn stem_package_readiness_blocks_missing_required_per_stem_gates() {
    let mut receipt = stem_package_receipt();
    receipt.unsupported_scopes.clear();
    receipt.qa_gates = all_required_stem_package_gates(ExportReceiptQaGateStatus::Passed);
    receipt
        .qa_gates
        .retain(|gate| gate.gate_id != STEM_PACKAGE_NON_SILENCE_QA_GATE_ID);

    let report = receipt.stem_package_readiness_report();

    assert!(!report.ready());
    assert_eq!(
        report.blockers,
        vec![StemPackageReceiptReadinessBlocker::MissingNonSilenceQaGate]
    );
}

#[test]
fn stem_package_readiness_blocks_deferred_required_per_stem_gates() {
    let mut receipt = stem_package_receipt();
    receipt.unsupported_scopes.clear();
    receipt.qa_gates = all_required_stem_package_gates(ExportReceiptQaGateStatus::Passed);

    for (gate_id, blocker) in [
        (
            STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID,
            StemPackageReceiptReadinessBlocker::DeferredHashStabilityQaGate,
        ),
        (
            STEM_PACKAGE_NON_SILENCE_QA_GATE_ID,
            StemPackageReceiptReadinessBlocker::DeferredNonSilenceQaGate,
        ),
        (
            STEM_PACKAGE_LINEAGE_QA_GATE_ID,
            StemPackageReceiptReadinessBlocker::DeferredLineageQaGate,
        ),
        (
            STEM_PACKAGE_FALLBACK_COMPARISON_QA_GATE_ID,
            StemPackageReceiptReadinessBlocker::DeferredFallbackComparisonQaGate,
        ),
    ] {
        let mut receipt = receipt.clone();
        replace_gate_status(&mut receipt, gate_id, ExportReceiptQaGateStatus::Deferred);

        let report = receipt.stem_package_readiness_report();

        assert!(!report.ready(), "{gate_id} should block readiness");
        assert_eq!(report.blockers, vec![blocker], "{gate_id}");
    }
}

#[test]
fn stem_package_readiness_blocks_failed_required_per_stem_gates() {
    let mut receipt = stem_package_receipt();
    receipt.unsupported_scopes.clear();
    receipt.qa_gates = all_required_stem_package_gates(ExportReceiptQaGateStatus::Passed);

    for (gate_id, blocker) in [
        (
            STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID,
            StemPackageReceiptReadinessBlocker::FailedHashStabilityQaGate,
        ),
        (
            STEM_PACKAGE_NON_SILENCE_QA_GATE_ID,
            StemPackageReceiptReadinessBlocker::FailedNonSilenceQaGate,
        ),
        (
            STEM_PACKAGE_LINEAGE_QA_GATE_ID,
            StemPackageReceiptReadinessBlocker::FailedLineageQaGate,
        ),
        (
            STEM_PACKAGE_FALLBACK_COMPARISON_QA_GATE_ID,
            StemPackageReceiptReadinessBlocker::FailedFallbackComparisonQaGate,
        ),
    ] {
        let mut receipt = receipt.clone();
        replace_gate_status(&mut receipt, gate_id, ExportReceiptQaGateStatus::Failed);

        let report = receipt.stem_package_readiness_report();

        assert!(!report.ready(), "{gate_id} should block readiness");
        assert_eq!(report.blockers, vec![blocker], "{gate_id}");
    }
}

#[test]
fn stem_package_readiness_allows_future_unblocked_passed_required_gates() {
    let mut receipt = stem_package_receipt();
    receipt.unsupported_scopes.clear();
    receipt.qa_gates = all_required_stem_package_gates(ExportReceiptQaGateStatus::Passed);

    let report = receipt.stem_package_readiness_report();

    assert_eq!(report.status, StemPackageReceiptReadinessStatus::Ready);
    assert!(report.blockers.is_empty());
}

#[test]
fn stem_package_ready_receipt_fixture_reports_ready_without_writer_side_effects() {
    let receipt = ready_stem_package_receipt_fixture();

    let report = receipt.stem_package_readiness_report();

    assert!(report.ready());
    assert!(report.blockers.is_empty());
    assert_eq!(receipt.export_scope, ExportScope::StemPackage);
    assert!(receipt.unsupported_scopes.is_empty());
    assert_eq!(receipt.qa_gates.len(), 5);
    assert!(
        receipt
            .qa_gates
            .iter()
            .all(|gate| gate.status == ExportReceiptQaGateStatus::Passed)
    );
    assert!(
        receipt
            .artifact_set
            .iter()
            .any(|artifact| artifact.role == ExportArtifactRole::ExportManifest)
    );
    assert!(
        receipt
            .artifact_set
            .iter()
            .any(|artifact| artifact.role == ExportArtifactRole::ProductExportProof)
    );

    for role in [ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass] {
        let artifact = receipt
            .artifact_set
            .iter()
            .find(|artifact| artifact.role == role)
            .expect("ready fixture stem artifact");
        assert_eq!(artifact.media_type, ExportArtifactMediaType::AudioWav);
        assert!(artifact.audio_metrics.is_some());
        assert!(artifact.source_graph_ref.is_some());
        assert!(artifact.fallback_comparison.is_some());
    }
}

#[test]
fn stem_package_ready_receipt_fixture_blocks_targeted_regressions() {
    let mut unsupported = ready_stem_package_receipt_fixture();
    unsupported
        .unsupported_scopes
        .push(UnsupportedExportScope::StemPackage);

    let report = unsupported.stem_package_readiness_report();

    assert_eq!(
        report.blockers,
        vec![StemPackageReceiptReadinessBlocker::UnsupportedScopeFlagPresent]
    );

    let mut missing_gate = ready_stem_package_receipt_fixture();
    missing_gate
        .qa_gates
        .retain(|gate| gate.gate_id != STEM_PACKAGE_NON_SILENCE_QA_GATE_ID);

    let report = missing_gate.stem_package_readiness_report();

    assert_eq!(
        report.blockers,
        vec![StemPackageReceiptReadinessBlocker::MissingNonSilenceQaGate]
    );
}

fn ready_stem_package_receipt_fixture() -> ExportReceiptState {
    let mut receipt = stem_package_receipt();
    receipt.unsupported_scopes.clear();
    receipt.artifact_set = vec![
        ready_stem_artifact(
            ExportArtifactRole::StemDrums,
            "exports/stems/drums.wav",
            "drums-sha",
        ),
        ready_stem_artifact(
            ExportArtifactRole::StemBass,
            "exports/stems/bass.wav",
            "bass-sha",
        ),
        ExportArtifactSetEntry::export_manifest(
            "exports/stem_package_manifest.json",
            "manifest-file-sha",
        ),
        ExportArtifactSetEntry::stem_package_proof(
            "exports/stem_package_proof.json",
            "proof-file-sha",
        ),
    ];
    let claimed_roles = vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass];
    let artifact_set_report = validate_stem_package_artifact_set_evidence_with_policy(
        &receipt.artifact_set,
        &claimed_roles,
        StemPackageArtifactSetQaPolicy {
            require_lineage_evidence: true,
            require_fallback_comparison_evidence: true,
        },
    );
    assert!(artifact_set_report.passed_structure_only());

    let non_silence_report =
        validate_stem_package_non_silence_evidence(&receipt.artifact_set, &claimed_roles);
    assert!(non_silence_report.passed());

    let lineage_report =
        validate_stem_package_lineage_evidence(&receipt.artifact_set, &claimed_roles);
    assert!(lineage_report.passed());

    let fallback_report =
        validate_stem_package_fallback_comparison_evidence(&receipt.artifact_set, &claimed_roles);
    assert!(fallback_report.passed());

    receipt.qa_gates = vec![
        passed_stem_package_gate(
            STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID,
            &claimed_roles,
            "stem package artifact-set accepted by fixture writer proof",
        ),
        passed_stem_package_gate(
            STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID,
            &claimed_roles,
            "stem package per-stem hash stability accepted by fixture writer proof",
        ),
        ExportReceiptQaGateResult::stem_package_non_silence(&non_silence_report),
        ExportReceiptQaGateResult::stem_package_lineage(&lineage_report),
        ExportReceiptQaGateResult::stem_package_fallback_comparison(&fallback_report),
    ];
    receipt
}

fn ready_stem_artifact(
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
        source_graph_ref: Some(ExportArtifactSourceGraphRef {
            source_id: SourceId::new("source-ready-fixture"),
            graph_version: SourceGraphVersion::V1,
            graph_hash: "ready-fixture-graph-sha".into(),
        }),
        timing_grid_ref: None,
        source_capture_refs: Vec::new(),
        lineage_capture_refs: Vec::new(),
        fallback_comparison: Some(ExportArtifactFallbackComparisonEvidence {
            comparison_kind: ExportArtifactFallbackComparisonKind::SourceVsFallback,
            reference_identity: "fallback://ready-fixture".into(),
            rms_difference_micros: Some(125_000),
            normalized_correlation_micros: Some(420_000),
        }),
        audio_metrics: Some(ExportArtifactAudioMetrics {
            peak_milli_dbfs: Some(-120),
            rms_milli_dbfs: Some(-6_000),
            peak_amplitude_micros: Some(986_000),
            rms_amplitude_micros: Some(125_000),
            silent_frame_count: Some(0),
            total_frame_count: Some(96_000),
        }),
        sample_rate_hz: Some(48_000),
        channel_count: Some(2),
        duration_ms: Some(2_000),
    }
}

fn passed_stem_package_gate(
    gate_id: &str,
    claimed_roles: &[ExportArtifactRole],
    summary: impl Into<String>,
) -> ExportReceiptQaGateResult {
    ExportReceiptQaGateResult {
        gate_id: gate_id.into(),
        status: ExportReceiptQaGateStatus::Passed,
        artifact_roles: claimed_roles.to_vec(),
        summary: Some(summary.into()),
    }
}

fn all_required_stem_package_gates(
    status: ExportReceiptQaGateStatus,
) -> Vec<ExportReceiptQaGateResult> {
    [
        STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID,
        STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID,
        STEM_PACKAGE_NON_SILENCE_QA_GATE_ID,
        STEM_PACKAGE_LINEAGE_QA_GATE_ID,
        STEM_PACKAGE_FALLBACK_COMPARISON_QA_GATE_ID,
    ]
    .into_iter()
    .map(|gate_id| stem_gate(gate_id, status))
    .collect()
}

fn replace_gate_status(
    receipt: &mut ExportReceiptState,
    gate_id: &str,
    status: ExportReceiptQaGateStatus,
) {
    let gate = receipt
        .qa_gates
        .iter_mut()
        .find(|gate| gate.gate_id == gate_id)
        .expect("fixture gate should exist");
    gate.status = status;
}

fn stem_gate(gate_id: &str, status: ExportReceiptQaGateStatus) -> ExportReceiptQaGateResult {
    ExportReceiptQaGateResult {
        gate_id: gate_id.into(),
        status,
        artifact_roles: vec![ExportArtifactRole::StemDrums],
        summary: Some(format!("fixture {gate_id} gate")),
    }
}
