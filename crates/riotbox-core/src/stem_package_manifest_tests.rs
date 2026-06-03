use serde_json::json;

use super::*;
use crate::session::{
    ExportArtifactAudioMetrics, ExportArtifactLocation, ExportArtifactMediaType,
    ExportArtifactRole, ExportArtifactSetEntry,
};

#[test]
fn stem_package_manifest_roundtrips_stable_schema_scope_roles_and_identity() {
    let manifest = fixture_manifest();

    let json = serde_json::to_value(&manifest).expect("serialize manifest");

    assert_eq!(json["schema_id"], STEM_PACKAGE_MANIFEST_SCHEMA_ID);
    assert_eq!(json["schema_version"], STEM_PACKAGE_MANIFEST_SCHEMA_VERSION);
    assert_eq!(json["export_scope"], "stem_package");
    assert_eq!(
        json["claimed_stem_roles"],
        json!(["stem_drums", "stem_bass"])
    );
    assert_eq!(json["artifacts"][0]["role"], "stem_drums");
    assert_eq!(json["manifest_identity"]["role"], "export_manifest");
    assert_eq!(json["proof_identity"]["role"], "product_export_proof");

    let roundtrip: StemPackageManifest =
        serde_json::from_value(json).expect("deserialize manifest");
    assert_eq!(roundtrip, manifest);
}

#[test]
fn stem_package_manifest_normalized_json_bytes_are_stable_roundtrippable_and_identity_sensitive() {
    let manifest = fixture_manifest();
    let bytes = manifest
        .normalized_json_bytes()
        .expect("serialize normalized manifest");
    let repeated_bytes = manifest
        .normalized_json_bytes()
        .expect("serialize normalized manifest again");
    let roundtrip: StemPackageManifest =
        serde_json::from_slice(&bytes).expect("deserialize normalized manifest");

    assert_eq!(bytes, repeated_bytes);
    assert_eq!(roundtrip, manifest);
    assert!(bytes.starts_with(b"{\n  \"schema_id\": \"riotbox.stem_package_manifest\""));

    let mut changed = manifest.clone();
    changed.artifacts[0].sha256 = "changed-drums-sha".into();
    let changed_bytes = changed
        .normalized_json_bytes()
        .expect("serialize changed normalized manifest");

    assert_ne!(bytes, changed_bytes);
}

#[test]
fn stem_package_manifest_normalized_json_sha256_uses_stable_proof_bytes() {
    let manifest = fixture_manifest();
    let hash = manifest
        .normalized_json_sha256()
        .expect("hash normalized manifest");
    let repeated_hash = manifest
        .normalized_json_sha256()
        .expect("hash normalized manifest again");
    let expected_hash = sha256_hex(
        &manifest
            .normalized_json_bytes()
            .expect("serialize normalized manifest"),
    );

    assert_eq!(hash, repeated_hash);
    assert_eq!(hash, expected_hash);
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|ch| ch.is_ascii_hexdigit()));

    let mut changed = manifest.clone();
    changed.artifacts[0].sha256 = "changed-drums-sha".into();
    let changed_hash = changed
        .normalized_json_sha256()
        .expect("hash changed normalized manifest");

    assert_ne!(hash, changed_hash);
}

#[test]
fn stem_package_manifest_rejects_blank_package_id() {
    let err = StemPackageManifest::new(manifest_input(
        " ",
        vec![ExportArtifactRole::StemDrums],
        vec![stem_artifact(
            ExportArtifactRole::StemDrums,
            "drums.wav",
            "drums",
        )],
        manifest_identity("manifest.json", "manifest"),
        proof_identity("proof.json", "proof"),
    ))
    .expect_err("blank package id should fail");

    assert_eq!(err, StemPackageManifestError::BlankPackageId);
}

#[test]
fn stem_package_manifest_rejects_empty_and_non_stem_claimed_roles() {
    let err = StemPackageManifest::new(manifest_input(
        "pkg",
        Vec::new(),
        vec![stem_artifact(
            ExportArtifactRole::StemDrums,
            "drums.wav",
            "drums",
        )],
        manifest_identity("manifest.json", "manifest"),
        proof_identity("proof.json", "proof"),
    ))
    .expect_err("empty claims should fail");
    assert_eq!(err, StemPackageManifestError::EmptyClaimedStemRoles);

    let err = StemPackageManifest::new(manifest_input(
        "pkg",
        vec![ExportArtifactRole::FullGridMix],
        vec![stem_artifact(
            ExportArtifactRole::StemDrums,
            "drums.wav",
            "drums",
        )],
        manifest_identity("manifest.json", "manifest"),
        proof_identity("proof.json", "proof"),
    ))
    .expect_err("non-stem claim should fail");
    assert_eq!(
        err,
        StemPackageManifestError::NonStemClaimedRole {
            role: ExportArtifactRole::FullGridMix
        }
    );
}

#[test]
fn stem_package_manifest_rejects_missing_duplicate_and_unclaimed_artifacts() {
    let err = StemPackageManifest::new(manifest_input(
        "pkg",
        vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        vec![stem_artifact(
            ExportArtifactRole::StemDrums,
            "drums.wav",
            "drums",
        )],
        manifest_identity("manifest.json", "manifest"),
        proof_identity("proof.json", "proof"),
    ))
    .expect_err("missing claimed bass should fail");
    assert_eq!(
        err,
        StemPackageManifestError::MissingClaimedStemArtifact {
            role: ExportArtifactRole::StemBass
        }
    );

    let err = StemPackageManifest::new(manifest_input(
        "pkg",
        vec![ExportArtifactRole::StemDrums],
        vec![
            stem_artifact(ExportArtifactRole::StemDrums, "drums-a.wav", "a"),
            stem_artifact(ExportArtifactRole::StemDrums, "drums-b.wav", "b"),
        ],
        manifest_identity("manifest.json", "manifest"),
        proof_identity("proof.json", "proof"),
    ))
    .expect_err("duplicate claimed drums should fail");
    assert_eq!(
        err,
        StemPackageManifestError::DuplicateStemArtifact {
            role: ExportArtifactRole::StemDrums
        }
    );

    let err = StemPackageManifest::new(manifest_input(
        "pkg",
        vec![ExportArtifactRole::StemDrums],
        vec![
            stem_artifact(ExportArtifactRole::StemDrums, "drums.wav", "drums"),
            stem_artifact(ExportArtifactRole::StemBass, "bass.wav", "bass"),
        ],
        manifest_identity("manifest.json", "manifest"),
        proof_identity("proof.json", "proof"),
    ))
    .expect_err("unclaimed bass should fail");
    assert_eq!(
        err,
        StemPackageManifestError::UnclaimedStemArtifact {
            role: ExportArtifactRole::StemBass
        }
    );
}

#[test]
fn stem_package_manifest_rejects_blank_artifact_identity_and_non_wav_stems() {
    let mut artifact = stem_artifact(ExportArtifactRole::StemDrums, " ", "drums");
    let err = StemPackageManifest::new(manifest_input(
        "pkg",
        vec![ExportArtifactRole::StemDrums],
        vec![artifact.clone()],
        manifest_identity("manifest.json", "manifest"),
        proof_identity("proof.json", "proof"),
    ))
    .expect_err("blank location should fail");
    assert_eq!(
        err,
        StemPackageManifestError::BlankArtifactLocation {
            role: ExportArtifactRole::StemDrums
        }
    );

    artifact.location = ExportArtifactLocation::LocalPath {
        path: "drums.wav".into(),
    };
    artifact.sha256.clear();
    let err = StemPackageManifest::new(manifest_input(
        "pkg",
        vec![ExportArtifactRole::StemDrums],
        vec![artifact.clone()],
        manifest_identity("manifest.json", "manifest"),
        proof_identity("proof.json", "proof"),
    ))
    .expect_err("blank sha should fail");
    assert_eq!(
        err,
        StemPackageManifestError::BlankArtifactSha256 {
            role: ExportArtifactRole::StemDrums
        }
    );

    artifact.sha256 = "drums".into();
    artifact.media_type = ExportArtifactMediaType::Json;
    let err = StemPackageManifest::new(manifest_input(
        "pkg",
        vec![ExportArtifactRole::StemDrums],
        vec![artifact],
        manifest_identity("manifest.json", "manifest"),
        proof_identity("proof.json", "proof"),
    ))
    .expect_err("json stem should fail");
    assert_eq!(
        err,
        StemPackageManifestError::NonWavStemArtifact {
            role: ExportArtifactRole::StemDrums,
            media_type: ExportArtifactMediaType::Json
        }
    );
}

#[test]
fn stem_package_manifest_rejects_wrong_manifest_or_proof_identity() {
    let err = StemPackageManifest::new(manifest_input(
        "pkg",
        vec![ExportArtifactRole::StemDrums],
        vec![stem_artifact(
            ExportArtifactRole::StemDrums,
            "drums.wav",
            "drums",
        )],
        proof_identity("proof.json", "proof"),
        proof_identity("proof.json", "proof"),
    ))
    .expect_err("wrong manifest role should fail");
    assert_eq!(
        err,
        StemPackageManifestError::UnexpectedJsonIdentityRole {
            expected: ExportArtifactRole::ExportManifest,
            actual: ExportArtifactRole::ProductExportProof
        }
    );

    let mut proof = proof_identity("proof.json", "proof");
    proof.media_type = ExportArtifactMediaType::AudioWav;
    let err = StemPackageManifest::new(manifest_input(
        "pkg",
        vec![ExportArtifactRole::StemDrums],
        vec![stem_artifact(
            ExportArtifactRole::StemDrums,
            "drums.wav",
            "drums",
        )],
        manifest_identity("manifest.json", "manifest"),
        proof,
    ))
    .expect_err("non-json proof should fail");
    assert_eq!(
        err,
        StemPackageManifestError::NonJsonIdentity {
            role: ExportArtifactRole::ProductExportProof,
            media_type: ExportArtifactMediaType::AudioWav
        }
    );
}

#[test]
fn stem_package_manifest_builds_artifact_and_identity_from_receipt_entries() {
    let mut entry = ExportArtifactSetEntry {
        role: ExportArtifactRole::StemVocals,
        location: ExportArtifactLocation::Uri {
            uri: "ipfs://vocals".into(),
        },
        media_type: ExportArtifactMediaType::AudioWav,
        sha256: "vocals".into(),
        normalized_manifest_hash: None,
        source_graph_ref: None,
        timing_grid_ref: None,
        source_capture_refs: Vec::new(),
        lineage_capture_refs: Vec::new(),
        fallback_comparison: None,
        audio_metrics: Some(ExportArtifactAudioMetrics {
            peak_milli_dbfs: Some(-3000),
            rms_milli_dbfs: Some(-12000),
            peak_amplitude_micros: Some(800_000),
            rms_amplitude_micros: Some(180_000),
            silent_frame_count: Some(0),
            total_frame_count: Some(48_000),
        }),
        sample_rate_hz: Some(48_000),
        channel_count: Some(2),
        duration_ms: Some(1000),
    };

    let artifact =
        StemPackageManifestArtifact::from_artifact_set_entry(&entry).expect("build stem artifact");
    assert_eq!(artifact.role, ExportArtifactRole::StemVocals);
    assert_eq!(artifact.location_identity(), "ipfs://vocals");
    assert_eq!(artifact.audio_metrics, entry.audio_metrics);

    entry = ExportArtifactSetEntry::export_manifest("manifest.json", "manifest");
    let identity = StemPackageManifestJsonIdentity::from_artifact_set_entry(&entry)
        .expect("build manifest identity");
    assert_eq!(identity.role, ExportArtifactRole::ExportManifest);
    assert_eq!(identity.location_identity(), "manifest.json");
}

fn fixture_manifest() -> StemPackageManifest {
    StemPackageManifest::new(manifest_input(
        "pkg-1",
        vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        vec![
            stem_artifact(ExportArtifactRole::StemDrums, "drums.wav", "drums"),
            stem_artifact(ExportArtifactRole::StemBass, "bass.wav", "bass"),
        ],
        manifest_identity("manifest.json", "manifest"),
        proof_identity("proof.json", "proof"),
    ))
    .expect("fixture manifest")
}

fn manifest_input(
    package_id: impl Into<String>,
    claimed_stem_roles: Vec<ExportArtifactRole>,
    artifacts: Vec<StemPackageManifestArtifact>,
    manifest_identity: StemPackageManifestJsonIdentity,
    proof_identity: StemPackageManifestJsonIdentity,
) -> StemPackageManifestInput {
    StemPackageManifestInput {
        package_id: package_id.into(),
        receipt_id: ExportReceiptId::new("receipt-1"),
        created_by_action: ActionId(7),
        claimed_stem_roles,
        artifacts,
        manifest_identity,
        proof_identity,
        qa_gates: Vec::new(),
    }
}

fn stem_artifact(
    role: ExportArtifactRole,
    path: impl Into<String>,
    sha256: impl Into<String>,
) -> StemPackageManifestArtifact {
    StemPackageManifestArtifact {
        role,
        location: ExportArtifactLocation::LocalPath { path: path.into() },
        media_type: ExportArtifactMediaType::AudioWav,
        sha256: sha256.into(),
        source_graph_ref: None,
        timing_grid_ref: None,
        source_capture_refs: Vec::new(),
        lineage_capture_refs: Vec::new(),
        fallback_comparison: None,
        audio_metrics: None,
        sample_rate_hz: Some(48_000),
        channel_count: Some(2),
        duration_ms: Some(1000),
    }
}

fn manifest_identity(
    path: impl Into<String>,
    sha256: impl Into<String>,
) -> StemPackageManifestJsonIdentity {
    json_identity(ExportArtifactRole::ExportManifest, path, sha256)
}

fn proof_identity(
    path: impl Into<String>,
    sha256: impl Into<String>,
) -> StemPackageManifestJsonIdentity {
    json_identity(ExportArtifactRole::ProductExportProof, path, sha256)
}

fn json_identity(
    role: ExportArtifactRole,
    path: impl Into<String>,
    sha256: impl Into<String>,
) -> StemPackageManifestJsonIdentity {
    StemPackageManifestJsonIdentity {
        role,
        location: ExportArtifactLocation::LocalPath { path: path.into() },
        media_type: ExportArtifactMediaType::Json,
        sha256: sha256.into(),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    let mut digest = Sha256::new();
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}
