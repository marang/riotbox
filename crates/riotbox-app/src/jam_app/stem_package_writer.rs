// The CI writer is intentionally not attached to UI/Ghost/CLI until the
// reserved stem export command gets its commit/observer slice.
#![allow(dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use riotbox_audio::source_audio::write_interleaved_pcm16_wav;
use riotbox_core::{
    TimestampMs,
    export_qa::{
        StemPackageArtifactSetQaPolicy, validate_stem_package_artifact_set_evidence_with_policy,
        validate_stem_package_fallback_comparison_evidence, validate_stem_package_lineage_evidence,
        validate_stem_package_non_silence_evidence,
    },
    export_readiness::{
        EXPORT_READINESS_CONTRACT_SCHEMA, ExportReadinessContract, ExportReadinessStatus,
        ExportScope, PRODUCT_EXPORT_PACK_ID, PRODUCT_EXPORT_PROOF_SCHEMA, ProductExportBoundary,
        ProductExportDestinationKind, ProductExportRole,
    },
    ids::ActionId,
    session::{
        ExportArtifactLocation, ExportArtifactMediaType, ExportArtifactRole,
        ExportArtifactSetEntry, ExportArtifactSourceGraphRef, ExportReceiptQaGateResult,
        ExportReceiptQaGateStatus, ExportReceiptState, STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID,
        STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID,
    },
    stem_package_manifest::StemPackageManifest,
    stem_package_proof::StemPackageProof,
    stem_package_writer::{
        STEM_PACKAGE_PACKAGE_DIR, StemPackageLocalWriterBoundary, StemPackageLocalWriterPlan,
        StemPackageLocalWriterRequest, plan_stem_package_local_ci_package,
    },
};

use super::{
    JamAppError,
    product_export::sha256_file,
    product_export_receipt::{LocalWavAudioEvidence, local_wav_audio_evidence},
};

const CI_FIXTURE_SAMPLE_RATE: u32 = 48_000;
const CI_FIXTURE_CHANNELS: u16 = 2;
const PENDING_JSON_SHA: &str = "pending-written-json-sha256";

#[derive(Clone, Debug)]
pub(crate) struct StemPackageFixtureStem {
    pub(crate) role: ExportArtifactRole,
    pub(crate) samples: Vec<f32>,
    pub(crate) source_graph_ref: ExportArtifactSourceGraphRef,
    pub(crate) fallback_comparison: riotbox_core::session::ExportArtifactFallbackComparisonEvidence,
}

#[derive(Clone, Debug)]
pub(crate) struct StemPackageFixtureWriterInput {
    pub(crate) created_by_action: ActionId,
    pub(crate) created_at: TimestampMs,
    pub(crate) destination_root: PathBuf,
    pub(crate) stems: Vec<StemPackageFixtureStem>,
}

#[derive(Clone, Debug)]
pub(crate) struct WrittenStemPackageFixture {
    pub(crate) package_dir: PathBuf,
    pub(crate) receipt: ExportReceiptState,
    pub(crate) manifest: StemPackageManifest,
    pub(crate) proof: StemPackageProof,
}

pub(crate) fn write_ci_safe_stem_package_fixture(
    input: StemPackageFixtureWriterInput,
) -> Result<WrittenStemPackageFixture, JamAppError> {
    let claimed_roles = input.stems.iter().map(|stem| stem.role).collect::<Vec<_>>();
    let final_plan = local_ci_plan(
        input.created_by_action,
        &input.destination_root,
        claimed_roles.clone(),
    )?;
    let staging_root = input
        .destination_root
        .join(format!(".stem_package_staging_{}", input.created_by_action));
    let staging_plan = local_ci_plan(input.created_by_action, &staging_root, claimed_roles)?;

    if staging_root.exists() {
        return Err(JamAppError::InvalidSession(format!(
            "stem package staging destination already exists: {}",
            staging_root.display()
        )));
    }
    let final_package_dir = input.destination_root.join(STEM_PACKAGE_PACKAGE_DIR);
    if final_package_dir.exists() {
        return Err(JamAppError::InvalidSession(format!(
            "stem package destination already exists: {}",
            final_package_dir.display()
        )));
    }
    fs::create_dir_all(&staging_root)?;

    let staged_stems = write_staged_stems(&input.stems, &staging_plan)?;
    let mut receipt = build_receipt(
        input.created_by_action,
        input.created_at,
        &final_plan,
        staged_stems,
        PENDING_JSON_SHA,
        PENDING_JSON_SHA,
    )?;
    let (manifest_artifact_path, proof_artifact_path) = json_artifact_paths(&staging_plan)?;
    let manifest = StemPackageManifest::from_receipt(&receipt)
        .map_err(|error| JamAppError::InvalidSession(format!("{error:?}")))?;
    fs::write(&manifest_artifact_path, manifest.normalized_json_bytes()?)?;
    let proof = StemPackageProof::from_manifest(&manifest)
        .map_err(|error| JamAppError::InvalidSession(format!("{error:?}")))?;
    fs::write(&proof_artifact_path, serde_json::to_vec_pretty(&proof)?)?;

    let manifest_sha = sha256_file(&manifest_artifact_path)?;
    let proof_sha = sha256_file(&proof_artifact_path)?;
    update_json_artifact_hashes(&mut receipt, &manifest_sha, &proof_sha);
    let final_manifest = StemPackageManifest::from_receipt(&receipt)
        .map_err(|error| JamAppError::InvalidSession(format!("{error:?}")))?;
    let final_proof = StemPackageProof::from_manifest(&final_manifest)
        .map_err(|error| JamAppError::InvalidSession(format!("{error:?}")))?;
    fs::write(
        &manifest_artifact_path,
        final_manifest.normalized_json_bytes()?,
    )?;
    fs::write(
        &proof_artifact_path,
        serde_json::to_vec_pretty(&final_proof)?,
    )?;

    let staged_package_dir = staging_root.join(STEM_PACKAGE_PACKAGE_DIR);
    fs::rename(&staged_package_dir, &final_package_dir)?;
    fs::remove_dir_all(&staging_root)?;

    rewrite_artifacts_to_final_paths(&mut receipt, &staging_root, &input.destination_root);
    refresh_final_hashes(&mut receipt)?;
    let final_manifest = StemPackageManifest::from_receipt(&receipt)
        .map_err(|error| JamAppError::InvalidSession(format!("{error:?}")))?;
    let final_proof = StemPackageProof::from_manifest(&final_manifest)
        .map_err(|error| JamAppError::InvalidSession(format!("{error:?}")))?;
    let (final_manifest_path, final_proof_path) = json_artifact_paths(&final_plan)?;
    fs::write(
        &final_manifest_path,
        final_manifest.normalized_json_bytes()?,
    )?;
    fs::write(&final_proof_path, serde_json::to_vec_pretty(&final_proof)?)?;
    refresh_final_hashes(&mut receipt)?;

    if !receipt.stem_package_readiness_report().ready() {
        return Err(JamAppError::InvalidSession(format!(
            "written stem package receipt is not ready: {:?}",
            receipt.stem_package_readiness_report().blockers
        )));
    }

    Ok(WrittenStemPackageFixture {
        package_dir: final_package_dir,
        receipt,
        manifest: final_manifest,
        proof: final_proof,
    })
}

fn local_ci_plan(
    created_by_action: ActionId,
    destination_root: &Path,
    claimed_stem_roles: Vec<ExportArtifactRole>,
) -> Result<StemPackageLocalWriterPlan, JamAppError> {
    plan_stem_package_local_ci_package(StemPackageLocalWriterRequest {
        created_by_action,
        boundary: StemPackageLocalWriterBoundary::LocalCiPackageV1,
        destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
        destination_root: destination_root.to_string_lossy().into_owned(),
        claimed_stem_roles,
    })
    .map_err(|error| JamAppError::InvalidSession(format!("{error:?}")))
}

fn write_staged_stems(
    stems: &[StemPackageFixtureStem],
    plan: &StemPackageLocalWriterPlan,
) -> Result<Vec<ExportArtifactSetEntry>, JamAppError> {
    let mut entries = Vec::new();
    for stem in stems {
        let path = path_for_role(plan, stem.role)?;
        write_interleaved_pcm16_wav(
            &path,
            CI_FIXTURE_SAMPLE_RATE,
            CI_FIXTURE_CHANNELS,
            &stem.samples,
        )
        .map_err(|error| JamAppError::InvalidSession(format!("{error}")))?;
        let evidence = local_wav_audio_evidence(&path).ok_or_else(|| {
            JamAppError::InvalidSession(format!("could not decode written stem {}", path.display()))
        })?;
        let mut entry = stem_artifact_from_written_path(stem, &path, evidence)?;
        entry.sha256 = sha256_file(&path)?;
        entries.push(entry);
    }
    Ok(entries)
}

fn stem_artifact_from_written_path(
    stem: &StemPackageFixtureStem,
    path: &Path,
    evidence: LocalWavAudioEvidence,
) -> Result<ExportArtifactSetEntry, JamAppError> {
    if !stem.role.is_stem_role() {
        return Err(JamAppError::InvalidSession(format!(
            "non-stem role claimed for CI package fixture: {:?}",
            stem.role
        )));
    }

    Ok(ExportArtifactSetEntry {
        role: stem.role,
        location: ExportArtifactLocation::LocalPath {
            path: path.to_string_lossy().into_owned(),
        },
        media_type: ExportArtifactMediaType::AudioWav,
        sha256: String::new(),
        normalized_manifest_hash: None,
        source_graph_ref: Some(stem.source_graph_ref.clone()),
        timing_grid_ref: None,
        source_capture_refs: Vec::new(),
        lineage_capture_refs: Vec::new(),
        fallback_comparison: Some(stem.fallback_comparison.clone()),
        audio_metrics: Some(evidence.audio_metrics),
        sample_rate_hz: Some(evidence.sample_rate_hz),
        channel_count: Some(evidence.channel_count),
        duration_ms: Some(evidence.duration_ms),
    })
}

fn build_receipt(
    created_by_action: ActionId,
    created_at: TimestampMs,
    final_plan: &StemPackageLocalWriterPlan,
    mut stem_artifacts: Vec<ExportArtifactSetEntry>,
    manifest_sha: &str,
    proof_sha: &str,
) -> Result<ExportReceiptState, JamAppError> {
    rewrite_artifacts_to_final_plan(&mut stem_artifacts, final_plan)?;
    let manifest_path = path_for_role(final_plan, ExportArtifactRole::ExportManifest)?;
    let proof_path = path_for_role(final_plan, ExportArtifactRole::ProductExportProof)?;
    stem_artifacts.push(ExportArtifactSetEntry::export_manifest(
        manifest_path.to_string_lossy().into_owned(),
        manifest_sha.to_owned(),
    ));
    stem_artifacts.push(ExportArtifactSetEntry::stem_package_proof(
        proof_path.to_string_lossy().into_owned(),
        proof_sha.to_owned(),
    ));
    let primary_stem_path = stem_artifacts
        .iter()
        .find(|artifact| artifact.role.is_stem_role())
        .map(|artifact| artifact.location_identity().to_owned())
        .ok_or_else(|| JamAppError::InvalidSession("stem package has no stem artifact".into()))?;
    let contract = ExportReadinessContract {
        schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: PRODUCT_EXPORT_PROOF_SCHEMA.into(),
        export_scope: ExportScope::StemPackage,
        boundary: ProductExportBoundary::FeralGridGeneratedSupport,
        pack_id: PRODUCT_EXPORT_PACK_ID.into(),
        export_role: ProductExportRole::FullGridMix,
        export_artifact: primary_stem_path.clone(),
        source_sha256: "ci-safe-stem-package-fixture".into(),
        export_sha256: stem_artifacts
            .iter()
            .find(|artifact| artifact.role.is_stem_role())
            .map(|artifact| artifact.sha256.clone())
            .unwrap_or_default(),
        normalized_manifest_sha256: manifest_sha.to_owned(),
        unsupported_scopes: Vec::new(),
    };
    let mut receipt = ExportReceiptState::from_readiness_contract(
        created_by_action,
        created_at,
        &contract,
        primary_stem_path,
        proof_path.to_string_lossy().into_owned(),
        Some(manifest_path.to_string_lossy().into_owned()),
    );
    receipt.artifact_set = stem_artifacts;
    receipt.qa_gates = stem_package_qa_gates(&receipt, &final_plan.claimed_stem_roles)?;
    Ok(receipt)
}

fn stem_package_qa_gates(
    receipt: &ExportReceiptState,
    claimed_roles: &[ExportArtifactRole],
) -> Result<Vec<ExportReceiptQaGateResult>, JamAppError> {
    let artifact_set_report = validate_stem_package_artifact_set_evidence_with_policy(
        &receipt.artifact_set,
        claimed_roles,
        StemPackageArtifactSetQaPolicy {
            require_lineage_evidence: true,
            require_fallback_comparison_evidence: true,
        },
    );
    if !artifact_set_report.passed_structure_only() {
        return Err(JamAppError::InvalidSession(format!(
            "stem package artifact-set evidence failed: {:?}",
            artifact_set_report.failures
        )));
    }
    let non_silence_report =
        validate_stem_package_non_silence_evidence(&receipt.artifact_set, claimed_roles);
    let lineage_report =
        validate_stem_package_lineage_evidence(&receipt.artifact_set, claimed_roles);
    let fallback_report =
        validate_stem_package_fallback_comparison_evidence(&receipt.artifact_set, claimed_roles);

    Ok(vec![
        passed_stem_package_gate(
            STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID,
            claimed_roles,
            "written stem package artifact-set accepted from final local WAV/JSON files",
        ),
        passed_stem_package_gate(
            STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID,
            claimed_roles,
            "written stem package per-stem hash stability accepted by repeated CI fixture proof",
        ),
        ExportReceiptQaGateResult::stem_package_non_silence(&non_silence_report),
        ExportReceiptQaGateResult::stem_package_lineage(&lineage_report),
        ExportReceiptQaGateResult::stem_package_fallback_comparison(&fallback_report),
    ])
}

fn passed_stem_package_gate(
    gate_id: &str,
    claimed_roles: &[ExportArtifactRole],
    summary: &str,
) -> ExportReceiptQaGateResult {
    ExportReceiptQaGateResult {
        gate_id: gate_id.into(),
        status: ExportReceiptQaGateStatus::Passed,
        artifact_roles: claimed_roles.to_vec(),
        summary: Some(summary.into()),
    }
}

fn path_for_role(
    plan: &StemPackageLocalWriterPlan,
    role: ExportArtifactRole,
) -> Result<PathBuf, JamAppError> {
    let artifact = plan
        .artifacts
        .iter()
        .find(|artifact| artifact.role == role)
        .ok_or_else(|| JamAppError::InvalidSession(format!("missing planned artifact {role:?}")))?;
    let ExportArtifactLocation::LocalPath { path } = &artifact.location else {
        return Err(JamAppError::InvalidSession(format!(
            "planned artifact {role:?} is not a local path"
        )));
    };
    Ok(PathBuf::from(path))
}

fn json_artifact_paths(
    plan: &StemPackageLocalWriterPlan,
) -> Result<(PathBuf, PathBuf), JamAppError> {
    Ok((
        path_for_role(plan, ExportArtifactRole::ExportManifest)?,
        path_for_role(plan, ExportArtifactRole::ProductExportProof)?,
    ))
}

fn rewrite_artifacts_to_final_plan(
    artifacts: &mut [ExportArtifactSetEntry],
    final_plan: &StemPackageLocalWriterPlan,
) -> Result<(), JamAppError> {
    for artifact in artifacts {
        if artifact.role.is_stem_role() {
            artifact.location = ExportArtifactLocation::LocalPath {
                path: path_for_role(final_plan, artifact.role)?
                    .to_string_lossy()
                    .into_owned(),
            };
        }
    }
    Ok(())
}

fn rewrite_artifacts_to_final_paths(
    receipt: &mut ExportReceiptState,
    staging_root: &Path,
    destination_root: &Path,
) {
    let staging_prefix = staging_root.to_string_lossy();
    let destination_prefix = destination_root.to_string_lossy();
    for artifact in &mut receipt.artifact_set {
        if let ExportArtifactLocation::LocalPath { path } = &mut artifact.location
            && let Some(relative) = path.strip_prefix(staging_prefix.as_ref())
        {
            *path = format!("{destination_prefix}{relative}");
        }
    }
    if let Some(relative) = receipt.proof_path.strip_prefix(staging_prefix.as_ref()) {
        receipt.proof_path = format!("{destination_prefix}{relative}");
    }
    if let Some(manifest_path) = &mut receipt.manifest_path
        && let Some(relative) = manifest_path.strip_prefix(staging_prefix.as_ref())
    {
        *manifest_path = format!("{destination_prefix}{relative}");
    }
}

fn update_json_artifact_hashes(
    receipt: &mut ExportReceiptState,
    manifest_sha: &str,
    proof_sha: &str,
) {
    for artifact in &mut receipt.artifact_set {
        match artifact.role {
            ExportArtifactRole::ExportManifest => artifact.sha256 = manifest_sha.to_owned(),
            ExportArtifactRole::ProductExportProof => artifact.sha256 = proof_sha.to_owned(),
            _ => {}
        }
    }
    receipt.normalized_manifest_hash = manifest_sha.to_owned();
}

fn refresh_final_hashes(receipt: &mut ExportReceiptState) -> Result<(), JamAppError> {
    let mut manifest_sha = None;
    let mut primary_artifact_sha = None;
    for artifact in &mut receipt.artifact_set {
        let ExportArtifactLocation::LocalPath { path } = &artifact.location else {
            continue;
        };
        let sha = sha256_file(Path::new(path))?;
        artifact.sha256 = sha.clone();
        if path == &receipt.artifact_path {
            primary_artifact_sha = Some(sha.clone());
        }
        if artifact.role == ExportArtifactRole::ExportManifest {
            manifest_sha = Some(sha);
        }
    }
    if let Some(sha) = manifest_sha {
        receipt.normalized_manifest_hash = sha;
    }
    if let Some(sha) = primary_artifact_sha {
        receipt.export_hash = sha;
    }
    Ok(())
}
