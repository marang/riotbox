use super::*;
use crate::{
    export_readiness::{
        ARRANGEMENT_DAW_PLACEMENT_PACK_ID, EXPORT_READINESS_CONTRACT_SCHEMA,
        ExportReadinessContract, ExportReadinessStatus, PRODUCT_EXPORT_PROOF_SCHEMA,
    },
    ids::ActionId,
    session::{ExportDawTempoMapRef, ExportReceiptState},
};

#[test]
fn daw_session_tempo_map_roundtrips_stable_schema_scope_and_timing_identity() {
    let tempo_map =
        DawSessionTempoMap::from_receipt(&ready_daw_receipt()).expect("tempo map from receipt");

    let json = serde_json::to_value(&tempo_map).expect("serialize tempo map");

    assert_eq!(json["schema_id"], DAW_SESSION_TEMPO_MAP_SCHEMA_ID);
    assert_eq!(json["schema_version"], DAW_SESSION_TEMPO_MAP_SCHEMA_VERSION);
    assert_eq!(json["package_id"], ARRANGEMENT_DAW_PLACEMENT_PACK_ID);
    assert_eq!(json["export_scope"], "daw_session");
    assert_eq!(json["export_role"], "arrangement_manifest");
    assert_eq!(
        json["export_boundary"],
        "arrangement_daw_placement_contract_v1"
    );
    assert_eq!(json["source_id"], "src-1");
    assert_eq!(json["hypothesis_id"], "primary-grid");
    assert_eq!(json["confirmed_by_action"], 8);
    assert_eq!(json["confirmed_at"], 880);
    assert_eq!(json["start_beat"], 0);
    assert_eq!(json["end_beat"], 16);
    assert_eq!(json["bpm_micros"], 128_000_000);

    let roundtrip: DawSessionTempoMap =
        serde_json::from_value(json).expect("deserialize tempo map");
    assert_eq!(roundtrip, tempo_map);
}

#[test]
fn daw_session_tempo_map_normalized_hash_is_stable_and_tempo_sensitive() {
    let receipt = ready_daw_receipt();
    let tempo_map = DawSessionTempoMap::from_receipt(&receipt).expect("tempo map from receipt");
    let mut changed_receipt = receipt;
    changed_receipt
        .daw_tempo_map_ref
        .as_mut()
        .expect("tempo map ref")
        .bpm_micros = 132_000_000;
    let changed = DawSessionTempoMap::from_receipt(&changed_receipt).expect("changed tempo map");

    assert_eq!(
        tempo_map.normalized_json_bytes().expect("tempo bytes"),
        tempo_map
            .normalized_json_bytes()
            .expect("tempo bytes again")
    );
    assert_eq!(
        tempo_map.normalized_json_sha256().expect("tempo hash"),
        tempo_map
            .normalized_json_sha256()
            .expect("tempo hash again")
    );
    assert_ne!(
        tempo_map.normalized_json_sha256().expect("tempo hash"),
        changed.normalized_json_sha256().expect("changed hash")
    );
}

#[test]
fn daw_session_tempo_map_from_receipt_rejects_non_daw_scope_boundary_role_and_flag() {
    let mut non_daw = ready_daw_receipt();
    non_daw.export_scope = ExportScope::ProductMix;
    let err = DawSessionTempoMap::from_receipt(&non_daw).expect_err("non-DAW scope rejects");
    assert_eq!(
        err,
        DawSessionTempoMapBuildError::NotDawSessionScope {
            export_scope: ExportScope::ProductMix
        }
    );

    let mut wrong_boundary = ready_daw_receipt();
    wrong_boundary.export_boundary = ProductExportBoundary::StemPackageLocalCiPackageV1;
    let err =
        DawSessionTempoMap::from_receipt(&wrong_boundary).expect_err("wrong boundary rejects");
    assert_eq!(
        err,
        DawSessionTempoMapBuildError::NotDawSessionBoundary {
            export_boundary: ProductExportBoundary::StemPackageLocalCiPackageV1
        }
    );

    let mut wrong_role = ready_daw_receipt();
    wrong_role.export_role = ProductExportRole::FullGridMix;
    let err = DawSessionTempoMap::from_receipt(&wrong_role).expect_err("wrong role rejects");
    assert_eq!(
        err,
        DawSessionTempoMapBuildError::UnexpectedExportRole {
            export_role: ProductExportRole::FullGridMix
        }
    );

    let mut unsupported = ready_daw_receipt();
    unsupported
        .unsupported_scopes
        .push(UnsupportedExportScope::DawExport);
    let err = DawSessionTempoMap::from_receipt(&unsupported).expect_err("unsupported flag rejects");
    assert_eq!(
        err,
        DawSessionTempoMapBuildError::UnsupportedDawExportFlagPresent
    );
}

#[test]
fn daw_session_tempo_map_from_receipt_rejects_missing_and_invalid_tempo_ref() {
    let mut missing = ready_daw_receipt();
    missing.daw_tempo_map_ref = None;
    let err = DawSessionTempoMap::from_receipt(&missing).expect_err("missing tempo rejects");
    assert_eq!(
        err,
        DawSessionTempoMapBuildError::DawTempoMapBlocked {
            blockers: vec![DawTempoMapReadinessBlocker::MissingTempoMapRef]
        }
    );

    let mut invalid = ready_daw_receipt();
    invalid.daw_tempo_map_ref = Some(ExportDawTempoMapRef::confirmed_grid(
        " ",
        None,
        ActionId(8),
        880,
        16,
        16,
        0,
    ));
    let err = DawSessionTempoMap::from_receipt(&invalid).expect_err("invalid tempo rejects");
    assert_eq!(
        err,
        DawSessionTempoMapBuildError::DawTempoMapBlocked {
            blockers: vec![
                DawTempoMapReadinessBlocker::BlankSourceRef,
                DawTempoMapReadinessBlocker::InvalidBeatRange,
                DawTempoMapReadinessBlocker::InvalidTempo
            ]
        }
    );
}

#[test]
fn daw_session_tempo_map_new_rejects_blank_package_id() {
    let receipt = ready_daw_receipt();
    let tempo_map = receipt.daw_tempo_map_ref.expect("tempo map ref");

    let err = DawSessionTempoMap::new(DawSessionTempoMapInput {
        package_id: " ".into(),
        export_role: receipt.export_role,
        export_boundary: receipt.export_boundary,
        receipt_id: receipt.receipt_id,
        created_by_action: receipt.created_by_action,
        source_id: tempo_map.source_id,
        hypothesis_id: tempo_map.hypothesis_id,
        confirmed_by_action: tempo_map.confirmed_by_action,
        confirmed_at: tempo_map.confirmed_at,
        start_beat: tempo_map.start_beat,
        end_beat: tempo_map.end_beat,
        bpm_micros: tempo_map.bpm_micros,
    })
    .expect_err("blank package id rejects");

    assert_eq!(err, DawSessionTempoMapError::BlankPackageId);
}

fn ready_daw_receipt() -> ExportReceiptState {
    let contract = ExportReadinessContract {
        schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: PRODUCT_EXPORT_PROOF_SCHEMA.into(),
        export_scope: ExportScope::DawSession,
        boundary: ProductExportBoundary::ArrangementDawPlacementContractV1,
        pack_id: ARRANGEMENT_DAW_PLACEMENT_PACK_ID.into(),
        export_role: ProductExportRole::ArrangementManifest,
        export_artifact: "exports/arrangement_manifest.json".into(),
        source_sha256: "source-sha".into(),
        export_sha256: "manifest-sha".into(),
        normalized_manifest_sha256: "normalized-manifest-sha".into(),
        unsupported_scopes: Vec::new(),
    };
    let mut receipt = ExportReceiptState::from_readiness_contract(
        ActionId(42),
        91_000,
        &contract,
        "exports/arrangement_manifest.json",
        "exports/proof.json",
        Some("exports/arrangement_manifest.json".into()),
    );
    receipt.daw_tempo_map_ref = Some(ExportDawTempoMapRef::confirmed_grid(
        "src-1",
        Some("primary-grid".into()),
        ActionId(8),
        880,
        0,
        16,
        128_000_000,
    ));
    receipt
}
