use super::*;

use crate::{
    export_readiness::{
        ARRANGEMENT_DAW_PLACEMENT_PACK_ID, EXPORT_READINESS_CONTRACT_SCHEMA,
        ExportReadinessContract, ExportReadinessStatus, ExportScope, PRODUCT_EXPORT_PACK_ID,
        PRODUCT_EXPORT_PROOF_SCHEMA, ProductExportBoundary, ProductExportRole,
        UnsupportedExportScope,
    },
    ids::ActionId,
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

fn arrangement_receipt() -> ExportReceiptState {
    let mut receipt = ExportReceiptState::from_readiness_contract(
        ActionId(7),
        900,
        &fixture_contract(),
        "exports/full_grid_mix.wav",
        "exports/product_export_proof.json",
        Some("exports/manifest.json".into()),
    );
    receipt.export_scope = ExportScope::DawSession;
    receipt.pack_id = ARRANGEMENT_DAW_PLACEMENT_PACK_ID.into();
    receipt.export_role = ProductExportRole::ArrangementManifest;
    receipt.export_boundary = ProductExportBoundary::ArrangementDawPlacementContractV1;
    receipt.unsupported_scopes.clear();
    receipt
}

#[test]
fn daw_tempo_map_readiness_blocks_missing_tempo_map_contract() {
    let mut receipt = arrangement_receipt();
    receipt
        .unsupported_scopes
        .push(UnsupportedExportScope::DawExport);

    let report = receipt.daw_tempo_map_report();

    assert_eq!(report.status, DawTempoMapReadinessStatus::Blocked);
    assert!(!report.ready());
    assert_eq!(
        report.blockers,
        vec![
            DawTempoMapReadinessBlocker::UnsupportedDawExportFlagPresent,
            DawTempoMapReadinessBlocker::MissingTempoMapRef,
        ]
    );
}

#[test]
fn daw_tempo_map_readiness_accepts_valid_confirmed_grid_contract() {
    let mut receipt = arrangement_receipt();
    receipt.daw_tempo_map_ref = Some(ExportDawTempoMapRef::confirmed_grid(
        "src-1",
        Some("primary-grid".into()),
        ActionId(8),
        880,
        0,
        16,
        128_000_000,
    ));

    let report = receipt.daw_tempo_map_report();

    assert_eq!(report.status, DawTempoMapReadinessStatus::Ready);
    assert!(report.ready());
    assert!(report.blockers.is_empty());
}

#[test]
fn daw_tempo_map_readiness_reports_invalid_source_range_and_tempo() {
    let mut receipt = arrangement_receipt();
    receipt.daw_tempo_map_ref = Some(ExportDawTempoMapRef::confirmed_grid(
        " ",
        None,
        ActionId(8),
        880,
        16,
        16,
        0,
    ));

    let report = receipt.daw_tempo_map_report();

    assert_eq!(report.status, DawTempoMapReadinessStatus::Blocked);
    assert_eq!(
        report.blockers,
        vec![
            DawTempoMapReadinessBlocker::BlankSourceRef,
            DawTempoMapReadinessBlocker::InvalidBeatRange,
            DawTempoMapReadinessBlocker::InvalidTempo,
        ]
    );
}
