use super::*;
use crate::{
    daw_session_manifest::{
        DawSessionManifestError, DawSessionManifestInput, DawSessionPlannedArtifactRole,
        DawSessionPlannedJsonIdentity, DawSessionSourceArtifactIdentity,
    },
    export_readiness::ARRANGEMENT_DAW_PLACEMENT_PACK_ID,
    ids::SourceId,
    session::{
        ExportArrangementPlacementRef, ExportArtifactLocation, ExportArtifactMediaType,
        ExportArtifactRole, ExportDawTempoMapRef,
    },
};

#[test]
fn daw_session_proof_roundtrips_stable_schema_scope_and_manifest_hash() {
    let proof = DawSessionProof::from_manifest(&fixture_manifest()).expect("proof from manifest");

    let json = serde_json::to_value(&proof).expect("serialize proof");

    assert_eq!(json["schema_id"], DAW_SESSION_PROOF_SCHEMA_ID);
    assert_eq!(json["schema_version"], DAW_SESSION_PROOF_SCHEMA_VERSION);
    assert_eq!(json["package_id"], ARRANGEMENT_DAW_PLACEMENT_PACK_ID);
    assert_eq!(json["export_scope"], "daw_session");
    assert_eq!(json["export_role"], "arrangement_manifest");
    assert_eq!(
        json["export_boundary"],
        "arrangement_daw_placement_contract_v1"
    );
    assert!(
        json["manifest_sha256"]
            .as_str()
            .expect("manifest hash")
            .len()
            >= 32
    );
    assert_eq!(json["placement_refs"][0]["scene_id"], "scene-a");
    assert_eq!(json["tempo_map_ref"]["source_id"], "src-1");
    assert_eq!(json["source_artifacts"][0]["role"], "export_manifest");
    assert_eq!(json["planned_artifacts"][2]["role"], "daw_session_proof");

    let roundtrip: DawSessionProof = serde_json::from_value(json).expect("deserialize proof");
    assert_eq!(roundtrip, proof);
}

#[test]
fn daw_session_proof_from_manifest_uses_normalized_manifest_hash() {
    let manifest = fixture_manifest();
    let proof = DawSessionProof::from_manifest(&manifest).expect("proof from manifest");
    let expected_manifest_hash = manifest
        .normalized_json_sha256()
        .expect("hash normalized manifest");

    assert_eq!(proof.manifest_sha256, expected_manifest_hash);
    assert_eq!(proof.package_id, manifest.package_id);
    assert_eq!(proof.receipt_id, manifest.receipt_id);
    assert_eq!(proof.created_by_action, manifest.created_by_action);
    assert_eq!(proof.placement_refs, manifest.placement_refs);
    assert_eq!(proof.tempo_map_ref, manifest.tempo_map_ref);
}

#[test]
fn daw_session_proof_serialized_value_changes_with_manifest_identity() {
    let manifest = fixture_manifest();
    let mut changed = fixture_manifest();
    changed.planned_artifacts[1] = DawSessionPlannedJsonIdentity::local_path(
        DawSessionPlannedArtifactRole::TempoMap,
        "exports/daw_session/changed_tempo_map.json",
    );
    let proof = DawSessionProof::from_manifest(&manifest).expect("proof from manifest");
    let changed_proof =
        DawSessionProof::from_manifest(&changed).expect("proof from changed manifest");

    assert_ne!(proof.manifest_sha256, changed_proof.manifest_sha256);
    assert_ne!(
        serde_json::to_vec_pretty(&proof).expect("proof json"),
        serde_json::to_vec_pretty(&changed_proof).expect("changed proof json")
    );
}

#[test]
fn daw_session_proof_rejects_blank_hash_and_missing_evidence() {
    let manifest = fixture_manifest();
    let err = DawSessionProof::new(DawSessionProofInput {
        package_id: manifest.package_id.clone(),
        export_role: manifest.export_role,
        export_boundary: manifest.export_boundary,
        receipt_id: manifest.receipt_id.clone(),
        created_by_action: manifest.created_by_action,
        manifest_sha256: " ".into(),
        placement_refs: manifest.placement_refs.clone(),
        tempo_map_ref: manifest.tempo_map_ref.clone(),
        source_artifacts: manifest.source_artifacts.clone(),
        planned_artifacts: manifest.planned_artifacts.clone(),
    })
    .expect_err("blank manifest hash should reject");
    assert_eq!(err, DawSessionProofError::BlankManifestSha256);

    let err = DawSessionProof::new(DawSessionProofInput {
        package_id: manifest.package_id.clone(),
        export_role: manifest.export_role,
        export_boundary: manifest.export_boundary,
        receipt_id: manifest.receipt_id.clone(),
        created_by_action: manifest.created_by_action,
        manifest_sha256: "manifest-sha".into(),
        placement_refs: Vec::new(),
        tempo_map_ref: manifest.tempo_map_ref.clone(),
        source_artifacts: manifest.source_artifacts.clone(),
        planned_artifacts: manifest.planned_artifacts.clone(),
    })
    .expect_err("missing placement refs should reject");
    assert_eq!(err, DawSessionProofError::MissingPlacementRefs);
}

#[test]
fn daw_session_proof_rejects_invalid_source_and_planned_identities() {
    let manifest = fixture_manifest();
    let mut missing_source = manifest.source_artifacts.clone();
    missing_source.retain(|artifact| artifact.role != ExportArtifactRole::ProductExportProof);
    let err = DawSessionProof::new(DawSessionProofInput {
        package_id: manifest.package_id.clone(),
        export_role: manifest.export_role,
        export_boundary: manifest.export_boundary,
        receipt_id: manifest.receipt_id.clone(),
        created_by_action: manifest.created_by_action,
        manifest_sha256: "manifest-sha".into(),
        placement_refs: manifest.placement_refs.clone(),
        tempo_map_ref: manifest.tempo_map_ref.clone(),
        source_artifacts: missing_source,
        planned_artifacts: manifest.planned_artifacts.clone(),
    })
    .expect_err("missing source proof identity should reject");
    assert_eq!(
        err,
        DawSessionProofError::InvalidSourceArtifacts(
            DawSessionManifestError::MissingSourceArtifact {
                role: ExportArtifactRole::ProductExportProof
            }
        )
    );

    let mut bad_plan = manifest.planned_artifacts.clone();
    bad_plan[2].media_type = ExportArtifactMediaType::AudioWav;
    let err = DawSessionProof::new(DawSessionProofInput {
        package_id: manifest.package_id.clone(),
        export_role: manifest.export_role,
        export_boundary: manifest.export_boundary,
        receipt_id: manifest.receipt_id.clone(),
        created_by_action: manifest.created_by_action,
        manifest_sha256: "manifest-sha".into(),
        placement_refs: manifest.placement_refs.clone(),
        tempo_map_ref: manifest.tempo_map_ref.clone(),
        source_artifacts: manifest.source_artifacts.clone(),
        planned_artifacts: bad_plan,
    })
    .expect_err("non-json planned proof identity should reject");
    assert_eq!(
        err,
        DawSessionProofError::InvalidPlannedArtifacts(
            DawSessionManifestError::NonJsonPlannedArtifact {
                role: DawSessionPlannedArtifactRole::DawSessionProof,
                media_type: ExportArtifactMediaType::AudioWav
            }
        )
    );
}

fn fixture_manifest() -> DawSessionManifest {
    DawSessionManifest::new(DawSessionManifestInput {
        package_id: ARRANGEMENT_DAW_PLACEMENT_PACK_ID.into(),
        export_role: ProductExportRole::ArrangementManifest,
        export_boundary: ProductExportBoundary::ArrangementDawPlacementContractV1,
        receipt_id: ExportReceiptId::new("export-receipt-42"),
        created_by_action: ActionId(42),
        placement_refs: vec![ExportArrangementPlacementRef::scene_range(
            "scene-a",
            Some(SourceId::from("src-1")),
            1,
            4,
            0,
            16,
        )],
        tempo_map_ref: ExportDawTempoMapRef::confirmed_grid(
            "src-1",
            Some("primary-grid".into()),
            ActionId(8),
            880,
            0,
            16,
            128_000_000,
        ),
        source_artifacts: vec![
            source_artifact(
                ExportArtifactRole::ExportManifest,
                "exports/arrangement_manifest.json",
            ),
            source_artifact(ExportArtifactRole::ProductExportProof, "exports/proof.json"),
        ],
        planned_artifacts: planned_artifacts(),
    })
    .expect("fixture DAW manifest")
}

fn source_artifact(role: ExportArtifactRole, path: &str) -> DawSessionSourceArtifactIdentity {
    DawSessionSourceArtifactIdentity {
        role,
        location: ExportArtifactLocation::LocalPath { path: path.into() },
        media_type: ExportArtifactMediaType::Json,
        sha256: format!("{role:?}-sha"),
    }
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
