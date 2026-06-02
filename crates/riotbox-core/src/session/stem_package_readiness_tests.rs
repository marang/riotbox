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
}

#[test]
fn stem_package_readiness_blocks_deferred_artifact_set_gate() {
    let mut receipt = stem_package_receipt();
    receipt.qa_gates = vec![stem_gate(ExportReceiptQaGateStatus::Deferred)];

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
    receipt.qa_gates = vec![stem_gate(ExportReceiptQaGateStatus::Failed)];

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
    receipt.qa_gates = vec![stem_gate(ExportReceiptQaGateStatus::Passed)];

    let report = receipt.stem_package_readiness_report();

    assert!(!report.ready());
    assert_eq!(
        report.blockers,
        vec![StemPackageReceiptReadinessBlocker::UnsupportedScopeFlagPresent]
    );
}

#[test]
fn stem_package_readiness_allows_future_unblocked_passed_gate() {
    let mut receipt = stem_package_receipt();
    receipt.unsupported_scopes.clear();
    receipt.qa_gates = vec![stem_gate(ExportReceiptQaGateStatus::Passed)];

    let report = receipt.stem_package_readiness_report();

    assert_eq!(report.status, StemPackageReceiptReadinessStatus::Ready);
    assert!(report.blockers.is_empty());
}

fn stem_gate(status: ExportReceiptQaGateStatus) -> ExportReceiptQaGateResult {
    ExportReceiptQaGateResult {
        gate_id: STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID.into(),
        status,
        artifact_roles: vec![ExportArtifactRole::StemDrums],
        summary: Some("fixture stem-package artifact-set gate".into()),
    }
}
