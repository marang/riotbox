use super::*;
use crate::{
    export_readiness::{
        ARRANGEMENT_DAW_PLACEMENT_PACK_ID, EXPORT_READINESS_CONTRACT_SCHEMA,
        ExportReadinessContract, ExportReadinessStatus, PRODUCT_EXPORT_PROOF_SCHEMA,
    },
    ids::SourceId,
    session::{ExportArtifactSetEntry, ExportDawTempoMapRef},
};

#[test]
fn daw_session_manifest_roundtrips_stable_schema_scope_and_identities() {
    let receipt = ready_daw_receipt();
    let manifest = DawSessionManifest::from_receipt(&receipt, planned_artifacts())
        .expect("manifest from ready DAW receipt");

    let json = serde_json::to_value(&manifest).expect("serialize manifest");

    assert_eq!(json["schema_id"], DAW_SESSION_MANIFEST_SCHEMA_ID);
    assert_eq!(json["schema_version"], DAW_SESSION_MANIFEST_SCHEMA_VERSION);
    assert_eq!(json["package_id"], ARRANGEMENT_DAW_PLACEMENT_PACK_ID);
    assert_eq!(json["export_scope"], "daw_session");
    assert_eq!(json["export_role"], "arrangement_manifest");
    assert_eq!(
        json["export_boundary"],
        "arrangement_daw_placement_contract_v1"
    );
    assert_eq!(json["placement_refs"][0]["scene_id"], "scene-a");
    assert_eq!(json["tempo_map_ref"]["source_id"], "src-1");
    assert_eq!(json["source_artifacts"][0]["role"], "export_manifest");
    assert_eq!(json["source_artifacts"][0]["sha256"], "manifest-sha");
    assert_eq!(
        json["planned_artifacts"]
            .as_array()
            .expect("planned artifacts")
            .len(),
        3
    );
    assert_eq!(json["planned_artifacts"][1]["role"], "tempo_map");

    let roundtrip: DawSessionManifest = serde_json::from_value(json).expect("deserialize manifest");
    assert_eq!(roundtrip, manifest);
}

#[test]
fn daw_session_manifest_normalized_hash_is_stable_and_identity_sensitive() {
    let receipt = ready_daw_receipt();
    let manifest = DawSessionManifest::from_receipt(&receipt, planned_artifacts())
        .expect("manifest from ready DAW receipt");
    let mut changed_plan = planned_artifacts();
    changed_plan[1] = DawSessionPlannedJsonIdentity::local_path(
        DawSessionPlannedArtifactRole::TempoMap,
        "exports/daw_session/changed_tempo_map.json",
    );
    let changed = DawSessionManifest::from_receipt(&receipt, changed_plan)
        .expect("changed manifest from ready DAW receipt");

    assert_eq!(
        manifest.normalized_json_bytes().expect("manifest bytes"),
        manifest
            .normalized_json_bytes()
            .expect("manifest bytes again")
    );
    assert_eq!(
        manifest.normalized_json_sha256().expect("manifest hash"),
        manifest
            .normalized_json_sha256()
            .expect("manifest hash again")
    );
    assert_ne!(
        manifest.normalized_json_sha256().expect("manifest hash"),
        changed.normalized_json_sha256().expect("changed hash")
    );
}

#[test]
fn daw_session_manifest_from_receipt_rejects_non_daw_scope_boundary_and_unsupported_flag() {
    let mut non_daw = ready_daw_receipt();
    non_daw.export_scope = ExportScope::ProductMix;
    let err = DawSessionManifest::from_receipt(&non_daw, planned_artifacts())
        .expect_err("non-DAW scope should reject");
    assert_eq!(
        err,
        DawSessionManifestBuildError::NotDawSessionScope {
            export_scope: ExportScope::ProductMix
        }
    );

    let mut wrong_boundary = ready_daw_receipt();
    wrong_boundary.export_boundary = ProductExportBoundary::StemPackageLocalCiPackageV1;
    let err = DawSessionManifest::from_receipt(&wrong_boundary, planned_artifacts())
        .expect_err("wrong boundary should reject");
    assert_eq!(
        err,
        DawSessionManifestBuildError::NotDawSessionBoundary {
            export_boundary: ProductExportBoundary::StemPackageLocalCiPackageV1
        }
    );

    let mut unsupported = ready_daw_receipt();
    unsupported.unsupported_scopes = vec![UnsupportedExportScope::DawExport];
    let err = DawSessionManifest::from_receipt(&unsupported, planned_artifacts())
        .expect_err("unsupported DAW flag should reject");
    assert_eq!(
        err,
        DawSessionManifestBuildError::UnsupportedDawExportFlagPresent
    );
}

#[test]
fn daw_session_manifest_from_receipt_rejects_missing_placement_tempo_and_source_identities() {
    let mut missing_placement = ready_daw_receipt();
    missing_placement.arrangement_placement_refs.clear();
    let err = DawSessionManifest::from_receipt(&missing_placement, planned_artifacts())
        .expect_err("missing placement should reject");
    assert!(matches!(
        err,
        DawSessionManifestBuildError::ArrangementPlacementBlocked { .. }
    ));

    let mut missing_tempo = ready_daw_receipt();
    missing_tempo.daw_tempo_map_ref = None;
    let err = DawSessionManifest::from_receipt(&missing_tempo, planned_artifacts())
        .expect_err("missing tempo map should reject");
    assert!(matches!(
        err,
        DawSessionManifestBuildError::DawTempoMapBlocked { .. }
    ));

    let mut missing_proof = ready_daw_receipt();
    missing_proof
        .artifact_set
        .retain(|entry| entry.role != ExportArtifactRole::ProductExportProof);
    let err = DawSessionManifest::from_receipt(&missing_proof, planned_artifacts())
        .expect_err("missing proof identity should reject");
    assert_eq!(
        err,
        DawSessionManifestBuildError::MissingSourceArtifact {
            role: ExportArtifactRole::ProductExportProof
        }
    );
}

#[test]
fn daw_session_manifest_rejects_invalid_planned_artifact_identities() {
    let receipt = ready_daw_receipt();
    let mut missing = planned_artifacts();
    missing.retain(|artifact| artifact.role != DawSessionPlannedArtifactRole::DawSessionProof);
    let err = DawSessionManifest::from_receipt(&receipt, missing)
        .expect_err("missing proof plan should reject");
    assert_eq!(
        err,
        DawSessionManifestBuildError::Manifest(DawSessionManifestError::MissingPlannedArtifact {
            role: DawSessionPlannedArtifactRole::DawSessionProof
        })
    );

    let mut non_json = planned_artifacts();
    non_json[1].media_type = ExportArtifactMediaType::AudioWav;
    let err = DawSessionManifest::from_receipt(&receipt, non_json)
        .expect_err("non-json planned tempo map should reject");
    assert_eq!(
        err,
        DawSessionManifestBuildError::Manifest(DawSessionManifestError::NonJsonPlannedArtifact {
            role: DawSessionPlannedArtifactRole::TempoMap,
            media_type: ExportArtifactMediaType::AudioWav
        })
    );
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
    receipt.artifact_set = vec![
        ExportArtifactSetEntry::export_manifest(
            "exports/arrangement_manifest.json",
            "manifest-sha",
        ),
        ExportArtifactSetEntry::product_export_proof("exports/proof.json", "proof-sha"),
    ];
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

fn planned_artifacts() -> Vec<DawSessionPlannedJsonIdentity> {
    vec![
        DawSessionPlannedJsonIdentity::local_path(
            DawSessionPlannedArtifactRole::ArrangementManifest,
            "exports/daw_session/arrangement_manifest.json",
        ),
        DawSessionPlannedJsonIdentity::local_path(
            DawSessionPlannedArtifactRole::TempoMap,
            "exports/daw_session/tempo_map.json",
        ),
        DawSessionPlannedJsonIdentity::local_path(
            DawSessionPlannedArtifactRole::DawSessionProof,
            "exports/daw_session/daw_session_proof.json",
        ),
    ]
}
