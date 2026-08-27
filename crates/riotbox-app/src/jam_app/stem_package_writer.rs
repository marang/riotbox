// Package writing stays on the operator/control path; the musician-facing
// UI/Ghost surface has separate DAW-placement and listening gates.

use std::{
    fs,
    fs::OpenOptions,
    io::{self, Write},
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
        ExportScope, ProductExportBoundary, ProductExportDestinationKind, ProductExportRole,
        STEM_PACKAGE_LOCAL_CI_PACK_ID, STEM_PACKAGE_SOURCE_MATCHED_PACK_ID,
    },
    ids::ActionId,
    session::{
        ExportArtifactLocation, ExportArtifactMediaType, ExportArtifactRole,
        ExportArtifactSetEntry, ExportArtifactSourceGraphRef, ExportArtifactTimingGridRef,
        ExportReceiptQaGateResult, ExportReceiptQaGateStatus, ExportReceiptState,
        STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID, STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID,
    },
    stem_package_manifest::StemPackageManifest,
    stem_package_proof::{STEM_PACKAGE_PROOF_SCHEMA_ID, StemPackageProof},
    stem_package_writer::{
        STEM_PACKAGE_PACKAGE_DIR, StemPackageLocalWriterBoundary, StemPackageLocalWriterPlan,
        StemPackageLocalWriterRequest, plan_stem_package_local_ci_package,
        plan_stem_package_source_matched_handoff,
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
pub(crate) struct StemPackageSourceStem {
    pub(crate) role: ExportArtifactRole,
    pub(crate) source_path: PathBuf,
    pub(crate) expected_sha256: String,
    pub(crate) normalized_manifest_sha256: String,
    pub(crate) source_graph_ref: ExportArtifactSourceGraphRef,
    pub(crate) timing_grid_ref: Option<ExportArtifactTimingGridRef>,
    pub(crate) fallback_reference_identity: String,
}

#[derive(Clone, Debug)]
pub(crate) struct StemPackageSourceWriterInput {
    pub(crate) created_by_action: ActionId,
    pub(crate) created_at: TimestampMs,
    pub(crate) destination_root: PathBuf,
    pub(crate) source_sha256: String,
    pub(crate) stems: Vec<StemPackageSourceStem>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
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
    let identity = StemPackageWriterIdentity {
        writer_boundary: StemPackageLocalWriterBoundary::LocalCiPackageV1,
        receipt_boundary: ProductExportBoundary::StemPackageLocalCiPackageV1,
        pack_id: STEM_PACKAGE_LOCAL_CI_PACK_ID,
        source_sha256: "ci-safe-stem-package-fixture".into(),
        hash_stability_summary: "written stem package per-stem hash stability accepted by repeated CI fixture proof",
    };
    write_stem_package(
        input.created_by_action,
        input.created_at,
        input.destination_root,
        claimed_roles,
        identity,
        |plan| write_staged_fixture_stems(&input.stems, plan),
    )
}

pub(crate) fn write_source_matched_stem_package(
    input: StemPackageSourceWriterInput,
) -> Result<WrittenStemPackageFixture, JamAppError> {
    let claimed_roles = input.stems.iter().map(|stem| stem.role).collect::<Vec<_>>();
    let identity = StemPackageWriterIdentity {
        writer_boundary: StemPackageLocalWriterBoundary::SourceMatchedHandoffV1,
        receipt_boundary: ProductExportBoundary::StemPackageSourceMatchedHandoffV1,
        pack_id: STEM_PACKAGE_SOURCE_MATCHED_PACK_ID,
        source_sha256: input.source_sha256,
        hash_stability_summary: "per-stem hash stability accepted from the validated v2 double-render handoff",
    };
    write_stem_package(
        input.created_by_action,
        input.created_at,
        input.destination_root,
        claimed_roles,
        identity,
        |plan| write_staged_source_stems(&input.stems, plan),
    )
}

#[derive(Clone, Debug)]
struct StemPackageWriterIdentity {
    writer_boundary: StemPackageLocalWriterBoundary,
    receipt_boundary: ProductExportBoundary,
    pack_id: &'static str,
    source_sha256: String,
    hash_stability_summary: &'static str,
}

fn write_stem_package(
    created_by_action: ActionId,
    created_at: TimestampMs,
    destination_root: PathBuf,
    claimed_roles: Vec<ExportArtifactRole>,
    identity: StemPackageWriterIdentity,
    write_stems: impl FnOnce(
        &StemPackageLocalWriterPlan,
    ) -> Result<Vec<ExportArtifactSetEntry>, JamAppError>,
) -> Result<WrittenStemPackageFixture, JamAppError> {
    let final_plan = writer_plan(
        created_by_action,
        &destination_root,
        claimed_roles.clone(),
        identity.writer_boundary,
    )?;
    let staging_root = destination_root.join(format!(".stem_package_staging_{created_by_action}"));
    let staging_plan = writer_plan(
        created_by_action,
        &staging_root,
        claimed_roles,
        identity.writer_boundary,
    )?;

    if staging_root.exists() {
        return Err(JamAppError::InvalidSession(format!(
            "stem package staging destination already exists: {}",
            staging_root.display()
        )));
    }
    let final_package_dir = destination_root.join(STEM_PACKAGE_PACKAGE_DIR);
    if final_package_dir.exists() {
        return Err(JamAppError::InvalidSession(format!(
            "stem package destination already exists: {}",
            final_package_dir.display()
        )));
    }
    fs::create_dir_all(&staging_root)?;
    let staged_result = (|| {
        let staged_stems = write_stems(&staging_plan)?;
        let mut receipt = build_receipt(
            created_by_action,
            created_at,
            &final_plan,
            staged_stems,
            PENDING_JSON_SHA,
            PENDING_JSON_SHA,
            &identity,
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
        if !receipt.stem_package_readiness_report().ready() {
            return Err(JamAppError::InvalidSession(format!(
                "written stem package receipt is not ready: {:?}",
                receipt.stem_package_readiness_report().blockers
            )));
        }
        Ok((receipt, manifest, proof))
    })();
    let (receipt, manifest, proof) = match staged_result {
        Ok(value) => value,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging_root);
            return Err(error);
        }
    };

    let staged_package_dir = staging_root.join(STEM_PACKAGE_PACKAGE_DIR);
    if let Err(error) = fs::rename(&staged_package_dir, &final_package_dir) {
        let _ = fs::remove_dir_all(&staging_root);
        return Err(error.into());
    }
    let _ = fs::remove_dir(&staging_root);

    Ok(WrittenStemPackageFixture {
        package_dir: final_package_dir,
        receipt,
        manifest,
        proof,
    })
}

fn writer_plan(
    created_by_action: ActionId,
    destination_root: &Path,
    claimed_stem_roles: Vec<ExportArtifactRole>,
    boundary: StemPackageLocalWriterBoundary,
) -> Result<StemPackageLocalWriterPlan, JamAppError> {
    let request = StemPackageLocalWriterRequest {
        created_by_action,
        boundary,
        destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
        destination_root: destination_root.to_string_lossy().into_owned(),
        claimed_stem_roles,
    };
    match boundary {
        StemPackageLocalWriterBoundary::LocalCiPackageV1 => {
            plan_stem_package_local_ci_package(request)
        }
        StemPackageLocalWriterBoundary::SourceMatchedHandoffV1 => {
            plan_stem_package_source_matched_handoff(request)
        }
    }
    .map_err(|error| JamAppError::InvalidSession(format!("{error:?}")))
}

fn write_staged_fixture_stems(
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

fn write_staged_source_stems(
    stems: &[StemPackageSourceStem],
    plan: &StemPackageLocalWriterPlan,
) -> Result<Vec<ExportArtifactSetEntry>, JamAppError> {
    let mut entries = Vec::new();
    for stem in stems {
        if !stem.role.is_stem_role() {
            return Err(JamAppError::InvalidSession(format!(
                "non-stem role claimed for source-matched package: {:?}",
                stem.role
            )));
        }
        let path = path_for_role(plan, stem.role)?;
        copy_file_new(&stem.source_path, &path)?;
        let actual_sha256 = sha256_file(&path)?;
        if actual_sha256 != stem.expected_sha256 {
            return Err(JamAppError::InvalidSession(format!(
                "source-matched stem hash drift for {:?}: expected {} actual {}",
                stem.role, stem.expected_sha256, actual_sha256
            )));
        }
        let evidence = local_wav_audio_evidence(&path).ok_or_else(|| {
            JamAppError::InvalidSession(format!(
                "could not decode written source-matched stem {}",
                path.display()
            ))
        })?;
        let fallback_comparison = riotbox_core::session::ExportArtifactFallbackComparisonEvidence {
            comparison_kind:
                riotbox_core::session::ExportArtifactFallbackComparisonKind::SourceVsFallback,
            reference_identity: stem.fallback_reference_identity.clone(),
            rms_difference_micros: evidence.audio_metrics.rms_amplitude_micros,
            normalized_correlation_micros: None,
        };
        entries.push(ExportArtifactSetEntry {
            role: stem.role,
            location: ExportArtifactLocation::LocalPath {
                path: path.to_string_lossy().into_owned(),
            },
            media_type: ExportArtifactMediaType::AudioWav,
            sha256: actual_sha256,
            normalized_manifest_hash: Some(stem.normalized_manifest_sha256.clone()),
            source_graph_ref: Some(stem.source_graph_ref.clone()),
            timing_grid_ref: stem.timing_grid_ref.clone(),
            source_capture_refs: Vec::new(),
            lineage_capture_refs: Vec::new(),
            fallback_comparison: Some(fallback_comparison),
            audio_metrics: Some(evidence.audio_metrics),
            sample_rate_hz: Some(evidence.sample_rate_hz),
            channel_count: Some(evidence.channel_count),
            duration_ms: Some(evidence.duration_ms),
        });
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
    identity: &StemPackageWriterIdentity,
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
    if !stem_artifacts
        .iter()
        .any(|artifact| artifact.role.is_stem_role())
    {
        return Err(JamAppError::InvalidSession(
            "stem package has no stem artifact".into(),
        ));
    }
    let manifest_artifact_path = manifest_path.to_string_lossy().into_owned();
    let contract = ExportReadinessContract {
        schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: STEM_PACKAGE_PROOF_SCHEMA_ID.into(),
        export_scope: ExportScope::StemPackage,
        boundary: identity.receipt_boundary,
        pack_id: identity.pack_id.into(),
        export_role: ProductExportRole::PackageManifest,
        export_artifact: manifest_artifact_path.clone(),
        source_sha256: identity.source_sha256.clone(),
        export_sha256: manifest_sha.to_owned(),
        normalized_manifest_sha256: manifest_sha.to_owned(),
        unsupported_scopes: Vec::new(),
    };
    let mut receipt = ExportReceiptState::from_readiness_contract(
        created_by_action,
        created_at,
        &contract,
        manifest_artifact_path,
        proof_path.to_string_lossy().into_owned(),
        Some(manifest_path.to_string_lossy().into_owned()),
    );
    receipt.artifact_set = stem_artifacts;
    receipt.qa_gates = stem_package_qa_gates(
        &receipt,
        &final_plan.claimed_stem_roles,
        identity.hash_stability_summary,
    )?;
    Ok(receipt)
}

fn stem_package_qa_gates(
    receipt: &ExportReceiptState,
    claimed_roles: &[ExportArtifactRole],
    hash_stability_summary: &str,
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
            hash_stability_summary,
        ),
        ExportReceiptQaGateResult::stem_package_non_silence(&non_silence_report),
        ExportReceiptQaGateResult::stem_package_lineage(&lineage_report),
        ExportReceiptQaGateResult::stem_package_fallback_comparison(&fallback_report),
    ])
}

fn copy_file_new(from: &Path, to: &Path) -> Result<(), JamAppError> {
    let parent = to.parent().ok_or_else(|| {
        JamAppError::InvalidSession(format!(
            "source-matched stem destination has no parent: {}",
            to.display()
        ))
    })?;
    fs::create_dir_all(parent)?;
    let mut source = fs::File::open(from)?;
    let mut destination = OpenOptions::new().write(true).create_new(true).open(to)?;
    let result = io::copy(&mut source, &mut destination)
        .and_then(|_| destination.flush())
        .map(|_| ());
    if result.is_err() {
        drop(destination);
        let _ = fs::remove_file(to);
    }
    result.map_err(Into::into)
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
    if receipt
        .manifest_path
        .as_ref()
        .is_some_and(|path| path == &receipt.artifact_path)
    {
        receipt.export_hash = manifest_sha.to_owned();
    }
}
