use std::{fs, path::PathBuf};

use riotbox_core::{
    export_readiness::{
        ExportScope, ProductExportBoundary, ProductExportRole, STEM_PACKAGE_LOCAL_CI_PACK_ID,
    },
    ids::{ActionId, SourceId},
    session::{
        ExportArtifactFallbackComparisonEvidence, ExportArtifactFallbackComparisonKind,
        ExportArtifactMediaType, ExportArtifactRole, ExportArtifactSetEntry,
        ExportArtifactSourceGraphRef, ExportReceiptState, STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID,
        StemPackageReceiptReadinessBlocker,
    },
    source_graph::SourceGraphVersion,
};

use super::{
    product_export::preflight_export_receipt_artifacts,
    stem_package_writer::{
        StemPackageFixtureStem, StemPackageFixtureWriterInput, write_ci_safe_stem_package_fixture,
    },
};

const FIXTURE_SAMPLE_RATE: u32 = 48_000;
const FIXTURE_CHANNELS: u16 = 2;

#[test]
fn ci_safe_stem_package_writer_promotes_final_files_and_ready_receipt() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let written = write_ci_safe_stem_package_fixture(fixture_input(
        ActionId(1128),
        tempdir.path().join("run-a"),
    ))
    .expect("write CI-safe stem package");

    assert!(written.package_dir.is_dir());
    assert_eq!(written.receipt.stem_package_readiness_report().blockers, []);
    assert!(written.receipt.stem_package_readiness_report().ready());
    assert_eq!(written.receipt.export_scope, ExportScope::StemPackage);
    assert_eq!(written.receipt.pack_id, STEM_PACKAGE_LOCAL_CI_PACK_ID);
    assert_eq!(
        written.receipt.export_role,
        ProductExportRole::PackageManifest
    );
    assert_eq!(
        written.receipt.export_boundary,
        ProductExportBoundary::StemPackageLocalCiPackageV1
    );
    assert_eq!(
        written.receipt.artifact_path,
        written
            .package_dir
            .join("stem_package_manifest.json")
            .to_string_lossy()
            .into_owned()
    );
    assert_eq!(written.manifest.package_id, STEM_PACKAGE_LOCAL_CI_PACK_ID);
    assert_eq!(
        written.manifest.export_role,
        ProductExportRole::PackageManifest
    );
    assert_eq!(
        written.manifest.export_boundary,
        ProductExportBoundary::StemPackageLocalCiPackageV1
    );
    assert_eq!(written.proof.package_id, STEM_PACKAGE_LOCAL_CI_PACK_ID);
    assert_eq!(
        written.proof.export_role,
        ProductExportRole::PackageManifest
    );
    assert_eq!(
        written.proof.export_boundary,
        ProductExportBoundary::StemPackageLocalCiPackageV1
    );
    assert!(written.receipt.unsupported_scopes.is_empty());
    assert_eq!(written.receipt.qa_gates.len(), 5);
    assert!(written.package_dir.join("stems/stem_drums.wav").is_file());
    assert!(written.package_dir.join("stems/stem_bass.wav").is_file());
    assert!(
        written
            .package_dir
            .join("stem_package_manifest.json")
            .is_file()
    );
    assert!(
        written
            .package_dir
            .join("stem_package_proof.json")
            .is_file()
    );
    let (primary_artifact, proof_artifact) =
        preflight_export_receipt_artifacts(&written.receipt, None)
            .expect("written receipt artifacts should preflight");
    let primary_entry = written
        .receipt
        .artifact_set
        .iter()
        .find(|artifact| artifact.location_identity() == written.receipt.artifact_path)
        .expect("primary artifact set entry");
    assert_eq!(written.receipt.export_hash, primary_entry.sha256);
    assert_eq!(
        primary_artifact,
        written.package_dir.join("stem_package_manifest.json")
    );
    assert_eq!(
        proof_artifact,
        written.package_dir.join("stem_package_proof.json")
    );
    assert_eq!(
        written.proof.manifest_sha256,
        written
            .manifest
            .normalized_json_sha256()
            .expect("manifest sha")
    );

    for role in [ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass] {
        let artifact = written
            .receipt
            .artifact_set
            .iter()
            .find(|artifact| artifact.role == role)
            .expect("stem artifact");
        assert_stem_artifact_evidence(artifact);
    }
}

#[test]
fn ci_safe_stem_package_writer_is_hash_stable_for_identical_inputs() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let destination = tempdir.path().join("run-a");
    let first =
        write_ci_safe_stem_package_fixture(fixture_input(ActionId(1128), destination.clone()))
            .expect("write first package");
    fs::remove_dir_all(&first.package_dir).expect("remove first package for repeat proof");
    let second = write_ci_safe_stem_package_fixture(fixture_input(ActionId(1128), destination))
        .expect("write repeated package");

    assert_eq!(
        artifact_hashes(&first.receipt),
        artifact_hashes(&second.receipt),
        "same fixture input should write byte-stable package artifacts"
    );
    assert_eq!(
        first.proof.manifest_sha256,
        first
            .manifest
            .normalized_json_sha256()
            .expect("first manifest sha")
    );
    assert_eq!(
        second.proof.manifest_sha256,
        second
            .manifest
            .normalized_json_sha256()
            .expect("second manifest sha")
    );
}

#[test]
fn ci_safe_stem_package_writer_rejects_existing_final_package() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let destination = tempdir.path().join("run-a");
    write_ci_safe_stem_package_fixture(fixture_input(ActionId(1128), destination.clone()))
        .expect("write package");

    let err = write_ci_safe_stem_package_fixture(fixture_input(ActionId(1128), destination))
        .expect_err("second write should not overwrite final package");

    assert!(
        err.to_string()
            .contains("stem package destination already exists")
    );
}

#[test]
fn ci_safe_stem_package_writer_rejects_existing_staging_package() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let destination = tempdir.path().join("run-a");
    fs::create_dir_all(destination.join(format!(".stem_package_staging_{}", ActionId(1128))))
        .expect("create stale staging dir");

    let err = write_ci_safe_stem_package_fixture(fixture_input(ActionId(1128), destination))
        .expect_err("existing staging directory should not be deleted");

    assert!(
        err.to_string()
            .contains("stem package staging destination already exists")
    );
}

#[test]
fn ci_safe_stem_package_writer_rejects_unsupported_claim_before_side_effects() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let mut input = fixture_input(ActionId(1128), tempdir.path().join("run-a"));
    input.stems.push(StemPackageFixtureStem {
        role: ExportArtifactRole::StemMusic,
        samples: fixture_bass_samples(),
        source_graph_ref: fixture_source_graph_ref(),
        fallback_comparison: fixture_fallback_comparison("music"),
    });

    let err =
        write_ci_safe_stem_package_fixture(input).expect_err("unsupported stem role should fail");

    assert!(err.to_string().contains("UnsupportedStemRole"));
    assert!(!tempdir.path().join("run-a/stem_package").exists());
}

#[test]
fn ci_safe_stem_package_writer_would_block_without_required_gate() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let mut written = write_ci_safe_stem_package_fixture(fixture_input(
        ActionId(1128),
        tempdir.path().join("run-a"),
    ))
    .expect("write package");
    written
        .receipt
        .qa_gates
        .retain(|gate| gate.gate_id != STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID);

    assert_eq!(
        written.receipt.stem_package_readiness_report().blockers,
        vec![StemPackageReceiptReadinessBlocker::MissingHashStabilityQaGate]
    );
}

fn assert_stem_artifact_evidence(artifact: &ExportArtifactSetEntry) {
    assert_eq!(artifact.media_type, ExportArtifactMediaType::AudioWav);
    assert!(!artifact.sha256.trim().is_empty());
    assert!(artifact.audio_metrics.is_some());
    assert!(artifact.source_graph_ref.is_some());
    assert!(artifact.fallback_comparison.is_some());
    assert!(
        artifact
            .location_identity()
            .contains("/stem_package/stems/")
    );
}

fn artifact_hashes(receipt: &ExportReceiptState) -> Vec<(ExportArtifactRole, String)> {
    receipt
        .artifact_set
        .iter()
        .map(|artifact| (artifact.role, artifact.sha256.clone()))
        .collect()
}

fn fixture_input(
    created_by_action: ActionId,
    destination_root: PathBuf,
) -> StemPackageFixtureWriterInput {
    StemPackageFixtureWriterInput {
        created_by_action,
        created_at: 1_128,
        destination_root,
        stems: vec![
            StemPackageFixtureStem {
                role: ExportArtifactRole::StemDrums,
                samples: fixture_drums_samples(),
                source_graph_ref: fixture_source_graph_ref(),
                fallback_comparison: fixture_fallback_comparison("drums"),
            },
            StemPackageFixtureStem {
                role: ExportArtifactRole::StemBass,
                samples: fixture_bass_samples(),
                source_graph_ref: fixture_source_graph_ref(),
                fallback_comparison: fixture_fallback_comparison("bass"),
            },
        ],
    }
}

fn fixture_source_graph_ref() -> ExportArtifactSourceGraphRef {
    ExportArtifactSourceGraphRef {
        source_id: SourceId::new("source-stem-package-ci-fixture"),
        graph_version: SourceGraphVersion::V1,
        graph_hash: "stem-package-ci-fixture-graph-sha".into(),
    }
}

fn fixture_fallback_comparison(role: &str) -> ExportArtifactFallbackComparisonEvidence {
    ExportArtifactFallbackComparisonEvidence {
        comparison_kind: ExportArtifactFallbackComparisonKind::SourceVsFallback,
        reference_identity: format!("fallback://stem-package-ci-fixture/{role}"),
        rms_difference_micros: Some(180_000),
        normalized_correlation_micros: Some(260_000),
    }
}

fn fixture_drums_samples() -> Vec<f32> {
    let frames = 48_000;
    let mut samples = vec![0.0; frames * usize::from(FIXTURE_CHANNELS)];
    for frame in (0..frames).step_by(6_000) {
        for transient in 0..96 {
            let amp = 0.92 * (1.0 - transient as f32 / 96.0);
            let index = (frame + transient) * usize::from(FIXTURE_CHANNELS);
            samples[index] = amp;
            samples[index + 1] = -amp * 0.75;
        }
    }
    samples
}

fn fixture_bass_samples() -> Vec<f32> {
    let frames = 48_000;
    let mut samples = Vec::with_capacity(frames * usize::from(FIXTURE_CHANNELS));
    for frame in 0..frames {
        let phase = frame as f32 / FIXTURE_SAMPLE_RATE as f32;
        let amp = (phase * 55.0 * std::f32::consts::TAU).sin() * 0.44;
        samples.push(amp);
        samples.push(amp * 0.96);
    }
    samples
}
