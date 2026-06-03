use super::*;

use crate::{
    export_readiness::{
        ARRANGEMENT_DAW_PLACEMENT_PACK_ID, EXPORT_READINESS_CONTRACT_SCHEMA,
        ExportReadinessContract, ExportReadinessStatus, ExportScope, PRODUCT_EXPORT_PACK_ID,
        PRODUCT_EXPORT_PROOF_SCHEMA, ProductExportBoundary, ProductExportRole,
        UnsupportedExportScope,
    },
    ids::{ActionId, SourceId},
    session::ExportReceiptState,
};

fn fixture_contract() -> ExportReadinessContract {
    ExportReadinessContract {
        schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: PRODUCT_EXPORT_PROOF_SCHEMA.into(),
        export_scope: ExportScope::ProductMix,
        boundary: ProductExportBoundary::FeralGridGeneratedSupport,
        pack_id: PRODUCT_EXPORT_PACK_ID.into(),
        export_role: ProductExportRole::FullGridMix,
        export_artifact: "run-a/full_grid_mix.wav".into(),
        source_sha256: "eeee".into(),
        export_sha256: "aaaa".into(),
        normalized_manifest_sha256: "dddd".into(),
        unsupported_scopes: vec![UnsupportedExportScope::StemPackage],
    }
}

fn fixture_receipt() -> ExportReceiptState {
    ExportReceiptState::from_readiness_contract(
        ActionId(7),
        900,
        &fixture_contract(),
        "exports/full_grid_mix.wav",
        "exports/product_export_proof.json",
        Some("exports/manifest.json".into()),
    )
}

fn arrangement_receipt() -> ExportReceiptState {
    let mut receipt = fixture_receipt();
    receipt.export_scope = ExportScope::DawSession;
    receipt.pack_id = ARRANGEMENT_DAW_PLACEMENT_PACK_ID.into();
    receipt.export_role = ProductExportRole::ArrangementManifest;
    receipt.export_boundary = ProductExportBoundary::ArrangementDawPlacementContractV1;
    receipt.unsupported_scopes.clear();
    receipt.arrangement_placement_refs.clear();
    receipt
}

#[test]
fn arrangement_export_placement_readiness_blocks_missing_placement_contract() {
    let mut receipt = arrangement_receipt();
    receipt
        .unsupported_scopes
        .push(UnsupportedExportScope::DawExport);

    let report = receipt.arrangement_export_placement_report();

    assert_eq!(
        report.status,
        ArrangementExportPlacementReadinessStatus::Blocked
    );
    assert!(!report.ready());
    assert_eq!(
        report.blockers,
        vec![
            ArrangementExportPlacementReadinessBlocker::UnsupportedDawExportFlagPresent,
            ArrangementExportPlacementReadinessBlocker::MissingPlacementRefs,
        ]
    );
}

#[test]
fn arrangement_export_placement_readiness_accepts_valid_scene_range_contract() {
    let mut receipt = arrangement_receipt();
    receipt
        .arrangement_placement_refs
        .push(ExportArrangementPlacementRef::scene_range(
            "scene-a",
            Some(SourceId::from("src-1")),
            1,
            4,
            0,
            16,
        ));

    let report = receipt.arrangement_export_placement_report();

    assert_eq!(
        report.status,
        ArrangementExportPlacementReadinessStatus::Ready
    );
    assert!(report.ready());
    assert!(report.blockers.is_empty());
}

#[test]
fn arrangement_export_placement_readiness_reports_invalid_scene_and_ranges() {
    let mut receipt = arrangement_receipt();
    receipt
        .arrangement_placement_refs
        .push(ExportArrangementPlacementRef::scene_range(
            " ",
            Some(SourceId::from("src-1")),
            4,
            2,
            16,
            16,
        ));

    let report = receipt.arrangement_export_placement_report();

    assert_eq!(
        report.status,
        ArrangementExportPlacementReadinessStatus::Blocked
    );
    assert_eq!(
        report.blockers,
        vec![
            ArrangementExportPlacementReadinessBlocker::BlankSceneRef,
            ArrangementExportPlacementReadinessBlocker::InvalidBarRange,
            ArrangementExportPlacementReadinessBlocker::InvalidBeatRange,
        ]
    );
}
