use super::*;

use crate::{
    export_readiness::{
        EXPORT_READINESS_CONTRACT_SCHEMA, PRODUCT_EXPORT_PACK_ID, PRODUCT_EXPORT_PROOF_SCHEMA,
    },
    ids::ActionId,
};

fn stem_package_receipt() -> ExportReceiptState {
    let contract = ExportReadinessContract {
        schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: PRODUCT_EXPORT_PROOF_SCHEMA.into(),
        export_scope: ExportScope::StemPackage,
        boundary: ProductExportBoundary::FeralGridGeneratedSupport,
        pack_id: PRODUCT_EXPORT_PACK_ID.into(),
        export_role: ProductExportRole::FullGridMix,
        export_artifact: "run-a/stem_package.zip".into(),
        source_sha256: "eeee".into(),
        export_sha256: "aaaa".into(),
        normalized_manifest_sha256: "dddd".into(),
        unsupported_scopes: vec![UnsupportedExportScope::StemPackage],
    };

    ExportReceiptState::from_readiness_contract(
        ActionId(7),
        900,
        &contract,
        "exports/stem_package.zip",
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
