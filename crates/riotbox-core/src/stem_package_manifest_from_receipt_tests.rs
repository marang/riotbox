use super::*;
use crate::{
    export_qa::validate_stem_package_artifact_set_evidence,
    export_readiness::{
        ExportReadinessContract, ExportReadinessStatus, ExportScope, PRODUCT_EXPORT_PACK_ID,
        PRODUCT_EXPORT_PROOF_SCHEMA, ProductExportBoundary, ProductExportRole,
        UnsupportedExportScope,
    },
    ids::ActionId,
    session::{
        ExportArtifactAudioMetrics, ExportArtifactLocation, ExportArtifactMediaType,
        ExportArtifactRole, ExportArtifactSetEntry, ExportReceiptQaGateResult,
        ExportReceiptQaGateStatus, ExportReceiptState, STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID,
        StemPackageReceiptReadinessBlocker, StemPackageReceiptReadinessStatus,
        validate_stem_package_receipt_readiness,
    },
    stem_package_proof::StemPackageProof,
};

#[test]
fn stem_package_manifest_from_receipt_preserves_receipt_evidence() {
    let receipt = stem_package_receipt();

    let manifest = StemPackageManifest::from_receipt(&receipt).expect("build manifest");

    assert_eq!(manifest.package_id, PRODUCT_EXPORT_PACK_ID);
    assert_eq!(manifest.export_scope, ExportScope::StemPackage);
    assert_eq!(manifest.receipt_id, receipt.receipt_id);
    assert_eq!(manifest.created_by_action, receipt.created_by_action);
    assert_eq!(
        manifest.claimed_stem_roles,
        vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass]
    );
    assert_eq!(manifest.artifacts.len(), 2);
    assert_eq!(manifest.artifacts[0].role, ExportArtifactRole::StemDrums);
    assert_eq!(
        manifest.artifacts[0].location_identity(),
        "exports/drums.wav"
    );
    assert_eq!(manifest.artifacts[0].sha256, "drums-sha");
    assert_eq!(manifest.artifacts[0].sample_rate_hz, Some(48_000));
    assert_eq!(manifest.artifacts[0].channel_count, Some(2));
    assert_eq!(manifest.artifacts[0].duration_ms, Some(1_000));
    assert_eq!(
        manifest.artifacts[0].audio_metrics,
        Some(stem_audio_metrics())
    );
    assert_eq!(
        manifest.manifest_identity.location_identity(),
        "exports/stem_package_manifest.json"
    );
    assert_eq!(
        manifest.proof_identity.location_identity(),
        "exports/stem_package_proof.json"
    );
    assert_eq!(manifest.qa_gates, receipt.qa_gates);
}

#[test]
fn stem_package_manifest_fixture_roundtrips_json_and_keeps_readiness_blocked() {
    let mut receipt = stem_package_receipt();
    let claimed_roles = vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass];
    let qa_report =
        validate_stem_package_artifact_set_evidence(&receipt.artifact_set, &claimed_roles);
    let gate = ExportReceiptQaGateResult::stem_package_artifact_set_evidence(&qa_report);
    assert_eq!(gate.status, ExportReceiptQaGateStatus::Deferred);
    receipt.qa_gates = vec![gate];

    let manifest = StemPackageManifest::from_receipt(&receipt).expect("build fixture manifest");
    let manifest_json =
        serde_json::to_string_pretty(&manifest).expect("serialize fixture manifest");
    let manifest_roundtrip: StemPackageManifest =
        serde_json::from_str(&manifest_json).expect("deserialize fixture manifest");
    assert_eq!(manifest_roundtrip, manifest);
    assert_eq!(manifest_roundtrip.claimed_stem_roles, claimed_roles);
    assert_eq!(manifest_roundtrip.artifacts.len(), 2);

    let proof = StemPackageProof::from_manifest(&manifest).expect("build fixture proof");
    let proof_json = serde_json::to_string_pretty(&proof).expect("serialize fixture proof");
    let repeated_proof_json =
        serde_json::to_string_pretty(&proof).expect("serialize fixture proof again");
    let proof_roundtrip: StemPackageProof =
        serde_json::from_str(&proof_json).expect("deserialize fixture proof");
    assert_eq!(proof_json, repeated_proof_json);
    assert_eq!(proof_roundtrip, proof);
    assert_eq!(
        proof_roundtrip.manifest_sha256,
        manifest
            .normalized_json_sha256()
            .expect("hash fixture manifest")
    );

    let readiness = validate_stem_package_receipt_readiness(&receipt);
    assert_eq!(readiness.status, StemPackageReceiptReadinessStatus::Blocked);
    assert!(
        readiness
            .blockers
            .contains(&StemPackageReceiptReadinessBlocker::UnsupportedScopeFlagPresent)
    );
    assert!(
        readiness
            .blockers
            .contains(&StemPackageReceiptReadinessBlocker::DeferredArtifactSetQaGate)
    );
}

#[test]
fn stem_package_manifest_and_proof_hashes_ignore_receipt_side_json_file_hashes() {
    let receipt = stem_package_receipt();
    let mut changed_json_hashes = stem_package_receipt();
    for entry in &mut changed_json_hashes.artifact_set {
        match entry.role {
            ExportArtifactRole::ExportManifest => {
                entry.sha256 = "changed-manifest-file-sha".into();
            }
            ExportArtifactRole::ProductExportProof => {
                entry.sha256 = "changed-proof-file-sha".into();
            }
            _ => {}
        }
    }

    let manifest = StemPackageManifest::from_receipt(&receipt).expect("build manifest");
    let changed_manifest =
        StemPackageManifest::from_receipt(&changed_json_hashes).expect("build changed manifest");
    let proof = StemPackageProof::from_manifest(&manifest).expect("build proof");
    let changed_proof =
        StemPackageProof::from_manifest(&changed_manifest).expect("build changed proof");

    assert_eq!(manifest, changed_manifest);
    assert_eq!(
        manifest.normalized_json_sha256().expect("hash manifest"),
        changed_manifest
            .normalized_json_sha256()
            .expect("hash changed manifest")
    );
    assert_eq!(proof.manifest_sha256, changed_proof.manifest_sha256);
    assert_eq!(
        serde_json::to_string_pretty(&proof).expect("serialize proof"),
        serde_json::to_string_pretty(&changed_proof).expect("serialize changed proof")
    );
}

#[test]
fn stem_package_manifest_from_receipt_rejects_non_stem_package_scope() {
    let mut receipt = stem_package_receipt();
    receipt.export_scope = ExportScope::ProductMix;

    let err = StemPackageManifest::from_receipt(&receipt).expect_err("scope should fail");

    assert_eq!(
        err,
        StemPackageManifestBuildError::NotStemPackageScope {
            export_scope: ExportScope::ProductMix
        }
    );
}

#[test]
fn stem_package_manifest_from_receipt_requires_artifact_set_gate_claims() {
    let mut receipt = stem_package_receipt();
    receipt.qa_gates.clear();

    let err = StemPackageManifest::from_receipt(&receipt).expect_err("missing gate should fail");

    assert_eq!(
        err,
        StemPackageManifestBuildError::MissingStemPackageArtifactSetQaGate
    );
}

#[test]
fn stem_package_manifest_from_receipt_requires_manifest_and_proof_entries() {
    let mut receipt = stem_package_receipt();
    receipt
        .artifact_set
        .retain(|entry| entry.role != ExportArtifactRole::ExportManifest);

    let err =
        StemPackageManifest::from_receipt(&receipt).expect_err("missing manifest should fail");

    assert_eq!(
        err,
        StemPackageManifestBuildError::MissingJsonIdentity {
            role: ExportArtifactRole::ExportManifest
        }
    );

    let mut receipt = stem_package_receipt();
    receipt
        .artifact_set
        .retain(|entry| entry.role != ExportArtifactRole::ProductExportProof);

    let err = StemPackageManifest::from_receipt(&receipt).expect_err("missing proof should fail");

    assert_eq!(
        err,
        StemPackageManifestBuildError::MissingJsonIdentity {
            role: ExportArtifactRole::ProductExportProof
        }
    );
}

#[test]
fn stem_package_manifest_from_receipt_requires_every_claimed_stem_entry() {
    let mut receipt = stem_package_receipt();
    receipt
        .artifact_set
        .retain(|entry| entry.role != ExportArtifactRole::StemBass);

    let err = StemPackageManifest::from_receipt(&receipt).expect_err("missing bass should fail");

    assert_eq!(
        err,
        StemPackageManifestBuildError::Manifest(
            StemPackageManifestError::MissingClaimedStemArtifact {
                role: ExportArtifactRole::StemBass
            }
        )
    );
}

#[test]
fn stem_package_manifest_from_receipt_rejects_duplicate_manifest_or_proof_entries() {
    let mut receipt = stem_package_receipt();
    receipt
        .artifact_set
        .push(ExportArtifactSetEntry::export_manifest(
            "exports/other_manifest.json",
            "other-manifest-sha",
        ));

    let err =
        StemPackageManifest::from_receipt(&receipt).expect_err("duplicate manifest should fail");

    assert_eq!(
        err,
        StemPackageManifestBuildError::MultipleJsonIdentities {
            role: ExportArtifactRole::ExportManifest
        }
    );

    let mut receipt = stem_package_receipt();
    receipt
        .artifact_set
        .push(ExportArtifactSetEntry::stem_package_proof(
            "exports/other_proof.json",
            "other-proof-sha",
        ));

    let err = StemPackageManifest::from_receipt(&receipt).expect_err("duplicate proof should fail");

    assert_eq!(
        err,
        StemPackageManifestBuildError::MultipleJsonIdentities {
            role: ExportArtifactRole::ProductExportProof
        }
    );
}

fn stem_package_receipt() -> ExportReceiptState {
    let contract = ExportReadinessContract {
        schema: "riotbox.export_readiness_contract.v1".into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: PRODUCT_EXPORT_PROOF_SCHEMA.into(),
        export_scope: ExportScope::StemPackage,
        boundary: ProductExportBoundary::FeralGridGeneratedSupport,
        pack_id: PRODUCT_EXPORT_PACK_ID.into(),
        export_role: ProductExportRole::FullGridMix,
        export_artifact: "exports/stem_package.zip".into(),
        source_sha256: "source-sha".into(),
        export_sha256: "package-sha".into(),
        normalized_manifest_sha256: "normalized-manifest-sha".into(),
        unsupported_scopes: Vec::new(),
    };
    let mut receipt = ExportReceiptState::from_readiness_contract(
        ActionId(7),
        900,
        &contract,
        "exports/stem_package.zip",
        "exports/stem_package_proof.json",
        Some("exports/stem_package_manifest.json".into()),
    );
    receipt.unsupported_scopes = vec![UnsupportedExportScope::StemPackage];
    receipt.artifact_set = vec![
        stem_entry(
            ExportArtifactRole::StemDrums,
            "exports/drums.wav",
            "drums-sha",
        ),
        stem_entry(ExportArtifactRole::StemBass, "exports/bass.wav", "bass-sha"),
        ExportArtifactSetEntry::export_manifest(
            "exports/stem_package_manifest.json",
            "manifest-sha",
        ),
        ExportArtifactSetEntry::stem_package_proof("exports/stem_package_proof.json", "proof-sha"),
    ];
    receipt.qa_gates = vec![ExportReceiptQaGateResult {
        gate_id: STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID.into(),
        status: ExportReceiptQaGateStatus::Passed,
        artifact_roles: vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        summary: Some("fixture stem-package artifact-set gate".into()),
    }];
    receipt
}

fn stem_entry(
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
        audio_metrics: Some(stem_audio_metrics()),
        sample_rate_hz: Some(48_000),
        channel_count: Some(2),
        duration_ms: Some(1_000),
    }
}

fn stem_audio_metrics() -> ExportArtifactAudioMetrics {
    ExportArtifactAudioMetrics {
        peak_milli_dbfs: Some(-3_000),
        rms_milli_dbfs: Some(-12_000),
        peak_amplitude_micros: Some(800_000),
        rms_amplitude_micros: Some(180_000),
        silent_frame_count: Some(0),
        total_frame_count: Some(48_000),
    }
}
