use std::{
    fs,
    path::{Path, PathBuf},
};

use riotbox_audio::source_audio::SourceAudioCache;
use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType, Quantization,
        StemPackageExportBoundary, StemPackageExportRole, StemPackageFallbackComparisonPolicy,
        StemPackageLineagePolicy, TargetScope, UndoPolicy,
    },
    export_readiness::{ExportScope, ProductExportDestinationKind},
    ids::ActionId,
    product_stem_handoff::{
        PRODUCT_STEM_DECLARED_METRIC_TOLERANCE, PRODUCT_STEM_PCM_MAX_ABS_ERROR,
        PRODUCT_STEM_PCM_MAX_RMS_ERROR, ProductStemHandoff, ProductStemHandoffArtifactRole,
    },
    queue::QueueEnqueueResult,
    session::{ExportArtifactSourceGraphRef, ExportArtifactTimingGridRef, ExportReceiptState},
    stem_package_writer::SOURCE_MATCHED_HANDOFF_STEM_ROLES,
};
use sha2::{Digest, Sha256};

use super::{
    JamAppError, JamAppState, QueueControlResult,
    persistence::source_graph_hash,
    product_export::sha256_file,
    stem_package_writer::{
        StemPackageSourceStem, StemPackageSourceWriterInput, write_source_matched_stem_package,
    },
};

struct SourceMatchedStemPackageActionParams {
    handoff_proof_path: PathBuf,
    destination_root: PathBuf,
}

struct ValidatedHandoffAudio {
    proof: ProductStemHandoff,
    proof_sha256: String,
    audio: Vec<(ProductStemHandoffArtifactRole, PathBuf, SourceAudioCache)>,
}

impl JamAppState {
    pub fn queue_stem_package_export_source_matched_handoff(
        &mut self,
        requested_at: TimestampMs,
        handoff_proof_path: Option<String>,
        destination_path: Option<String>,
    ) -> QueueControlResult {
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::ExportStemPackage,
            Quantization::Immediate,
            ActionTarget {
                scope: Some(TargetScope::Session),
                ..Default::default()
            },
        );
        draft.params = ActionParams::StemPackageExport {
            export_scope: ExportScope::StemPackage,
            export_role: StemPackageExportRole::PackageManifest,
            boundary: StemPackageExportBoundary::SourceMatchedHandoffV1,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
            destination_path,
            handoff_proof_path,
            claimed_stem_roles: SOURCE_MATCHED_HANDOFF_STEM_ROLES.to_vec(),
            lineage_policy: StemPackageLineagePolicy::RequireAnyCoreLineage,
            fallback_comparison_policy: StemPackageFallbackComparisonPolicy::Required,
        };
        draft.undo_policy = UndoPolicy::NotUndoable {
            reason: "stem package export writes files outside musical undo".into(),
        };
        draft.explanation = Some(
            "export active-source-matched product stems through the Session receipt path".into(),
        );

        match self
            .queue
            .enqueue_if_no_pending_command(draft, requested_at)
        {
            QueueEnqueueResult::AlreadyPending { .. } => QueueControlResult::AlreadyPending,
            QueueEnqueueResult::Enqueued(_) => {
                self.refresh_view();
                QueueControlResult::Enqueued
            }
        }
    }

    pub fn commit_stem_package_export_from_product_handoff(
        &mut self,
        handoff_proof_path: impl AsRef<Path>,
        destination_dir: impl AsRef<Path>,
        requested_at: TimestampMs,
    ) -> Result<ExportReceiptState, JamAppError> {
        let handoff_proof_path = handoff_proof_path.as_ref();
        let destination_dir = destination_dir.as_ref();
        let action_id = match self.pending_source_matched_stem_package_action_id() {
            Some(action_id) => action_id,
            None => {
                self.queue_stem_package_export_source_matched_handoff(
                    requested_at,
                    Some(handoff_proof_path.to_string_lossy().into_owned()),
                    Some(destination_dir.to_string_lossy().into_owned()),
                );
                self.pending_source_matched_stem_package_action_id()
                    .ok_or_else(|| {
                        JamAppError::InvalidSession(
                            "cannot queue source-matched stem package while another stem-package export is pending"
                                .into(),
                        )
                    })?
            }
        };
        let result = self.prepare_and_write_source_matched_stem_package(action_id, requested_at);
        let receipt = match result {
            Ok(receipt) => receipt,
            Err(error) => {
                self.queue.reject(action_id, error.to_string());
                self.refresh_view();
                return Err(error);
            }
        };
        let result_summary = format!(
            "exported source-matched stem_package receipt {} roles stem_drums,stem_music,stem_bass artifacts {}",
            receipt.receipt_id,
            receipt.artifact_set.len()
        );
        self.commit_export_receipt_after_side_effect(
            action_id,
            requested_at,
            receipt.clone(),
            result_summary,
        )?;
        Ok(receipt)
    }

    fn prepare_and_write_source_matched_stem_package(
        &self,
        action_id: ActionId,
        requested_at: TimestampMs,
    ) -> Result<ExportReceiptState, JamAppError> {
        let params = self.source_matched_stem_package_action_params(action_id)?;
        let graph = self.source_graph.as_ref().ok_or_else(|| {
            JamAppError::InvalidSession(
                "source-matched stem package export requires an active Source Graph identity"
                    .into(),
            )
        })?;
        let active_source_hash = normalized_sha256(&graph.source.content_hash);
        if active_source_hash.is_empty() {
            return Err(JamAppError::InvalidSession(
                "source-matched stem package active source identity is blank".into(),
            ));
        }
        let active_graph_hash = source_graph_hash(graph)?;
        let source_graph_ref = self
            .session
            .source_graph_refs
            .iter()
            .find(|graph_ref| {
                graph_ref.source_id == graph.source.source_id
                    && graph_ref.graph_version == graph.graph_version
                    && graph_ref.graph_hash == active_graph_hash
            })
            .map(|graph_ref| ExportArtifactSourceGraphRef {
                source_id: graph_ref.source_id.clone(),
                graph_version: graph_ref.graph_version,
                graph_hash: graph_ref.graph_hash.clone(),
            })
            .filter(|graph_ref| !graph_ref.graph_hash.trim().is_empty())
            .ok_or_else(|| {
                JamAppError::InvalidSession(
                    "source-matched stem package requires exact active Session Source Graph lineage"
                        .into(),
                )
            })?;
        let timing_grid_ref = self
            .session
            .runtime_state
            .source_timing
            .confirmed_grid
            .as_ref()
            .filter(|grid| grid.source_id == graph.source.source_id)
            .map(|grid| ExportArtifactTimingGridRef {
                source_id: grid.source_id.clone(),
                hypothesis_id: grid.hypothesis_id.clone(),
                confirmed_by_action: grid.confirmed_by_action,
                confirmed_at: grid.confirmed_at,
            });

        let validated =
            validate_product_stem_handoff_bundle(&params.handoff_proof_path, active_source_hash)?;
        let fallback_identity_prefix = format!(
            "product-stem-handoff-sha256:{}#fail-closed-silence-v1",
            validated.proof_sha256
        );
        let stems = [
            ProductStemHandoffArtifactRole::StemDrums,
            ProductStemHandoffArtifactRole::StemMusic,
            ProductStemHandoffArtifactRole::StemBass,
        ]
        .into_iter()
        .map(|role| {
            let artifact = validated
                .proof
                .artifact(role)
                .expect("validated handoff has every product stem role");
            let (_, source_path, _) = validated
                .audio
                .iter()
                .find(|(candidate, _, _)| *candidate == role)
                .expect("validated handoff audio has every product stem role");
            StemPackageSourceStem {
                role: role.export_artifact_role(),
                source_path: source_path.clone(),
                expected_sha256: artifact.sha256.clone(),
                normalized_manifest_sha256: validated.proof.normalized_manifest_sha256.clone(),
                source_graph_ref: source_graph_ref.clone(),
                timing_grid_ref: timing_grid_ref.clone(),
                fallback_reference_identity: format!(
                    "{fallback_identity_prefix}/{}",
                    product_stem_role_label(role)
                ),
            }
        })
        .collect();
        let written = write_source_matched_stem_package(StemPackageSourceWriterInput {
            created_by_action: action_id,
            created_at: requested_at,
            destination_root: params.destination_root,
            source_sha256: validated.proof.source_sha256,
            stems,
        })?;
        Ok(written.receipt)
    }

    fn pending_source_matched_stem_package_action_id(&self) -> Option<ActionId> {
        self.queue
            .pending_actions()
            .into_iter()
            .find(|action| {
                action.command == ActionCommand::ExportStemPackage
                    && matches!(
                        action.params,
                        ActionParams::StemPackageExport {
                            boundary: StemPackageExportBoundary::SourceMatchedHandoffV1,
                            ..
                        }
                    )
            })
            .map(|action| action.id)
    }

    fn source_matched_stem_package_action_params(
        &self,
        action_id: ActionId,
    ) -> Result<SourceMatchedStemPackageActionParams, JamAppError> {
        let action = self
            .queue
            .pending_actions()
            .into_iter()
            .find(|action| action.id == action_id)
            .ok_or_else(|| {
                JamAppError::InvalidSession(format!(
                    "queued source-matched stem package action {action_id} is not pending"
                ))
            })?;
        let ActionParams::StemPackageExport {
            export_scope,
            export_role,
            boundary,
            include_manifest,
            destination_kind,
            destination_path,
            handoff_proof_path,
            claimed_stem_roles,
            lineage_policy,
            fallback_comparison_policy,
        } = &action.params
        else {
            return Err(JamAppError::InvalidSession(format!(
                "queued action {action_id} is not a stem package export"
            )));
        };
        if *export_scope != ExportScope::StemPackage
            || *export_role != StemPackageExportRole::PackageManifest
            || *boundary != StemPackageExportBoundary::SourceMatchedHandoffV1
            || !*include_manifest
            || *destination_kind != ProductExportDestinationKind::LocalArtifactDirectory
            || claimed_stem_roles != SOURCE_MATCHED_HANDOFF_STEM_ROLES
            || *lineage_policy != StemPackageLineagePolicy::RequireAnyCoreLineage
            || *fallback_comparison_policy != StemPackageFallbackComparisonPolicy::Required
        {
            return Err(JamAppError::InvalidSession(format!(
                "source-matched stem package action {action_id} has unsupported parameters"
            )));
        }
        let handoff_proof_path = required_local_path(
            handoff_proof_path,
            "source-matched stem package handoff proof path",
        )?;
        let destination_root = required_local_path(
            destination_path,
            "source-matched stem package destination path",
        )?;
        Ok(SourceMatchedStemPackageActionParams {
            handoff_proof_path,
            destination_root,
        })
    }
}

fn validate_product_stem_handoff_bundle(
    proof_path: &Path,
    active_source_hash: &str,
) -> Result<ValidatedHandoffAudio, JamAppError> {
    let proof_metadata = fs::symlink_metadata(proof_path)?;
    if !proof_metadata.file_type().is_file() || proof_metadata.file_type().is_symlink() {
        return Err(JamAppError::InvalidSession(format!(
            "product-stem handoff proof must be a regular file: {}",
            proof_path.display()
        )));
    }
    let proof_bytes = fs::read(proof_path)?;
    let proof_sha256 = format!("{:x}", Sha256::digest(&proof_bytes));
    let proof: ProductStemHandoff = serde_json::from_slice(&proof_bytes)?;
    proof.validate().map_err(|error| {
        JamAppError::InvalidSession(format!("invalid product-stem handoff: {error:?}"))
    })?;
    if proof.source_sha256 != active_source_hash {
        return Err(JamAppError::InvalidSession(format!(
            "source-matched stem package source mismatch: proof {} active {}",
            proof.source_sha256, active_source_hash
        )));
    }
    let bundle_root = proof_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()?;
    let mut audio = Vec::new();
    for artifact in &proof.artifacts {
        let unresolved = bundle_root.join(&artifact.path);
        let metadata = fs::symlink_metadata(&unresolved)?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err(JamAppError::InvalidSession(format!(
                "product-stem artifact must be a regular file: {}",
                unresolved.display()
            )));
        }
        let path = unresolved.canonicalize()?;
        if !path.starts_with(&bundle_root) {
            return Err(JamAppError::InvalidSession(format!(
                "product-stem artifact escapes its handoff bundle: {}",
                path.display()
            )));
        }
        let actual_sha256 = sha256_file(&path)?;
        if actual_sha256 != artifact.sha256 {
            return Err(JamAppError::InvalidSession(format!(
                "product-stem artifact hash mismatch for {:?}: proof {} actual {}",
                artifact.role, artifact.sha256, actual_sha256
            )));
        }
        require_pcm16_wav(&path)?;
        let cache = SourceAudioCache::load_pcm_wav(&path)
            .map_err(|error| JamAppError::InvalidSession(error.to_string()))?;
        if cache.sample_rate != proof.grid.sample_rate_hz
            || cache.channel_count != proof.grid.channel_count
            || cache.frame_count() as u64 != proof.grid.frame_count
        {
            return Err(JamAppError::InvalidSession(format!(
                "product-stem format/grid mismatch for {:?}",
                artifact.role
            )));
        }
        audio.push((artifact.role, path, cache));
    }
    validate_reconstruction(&proof, &audio)?;
    Ok(ValidatedHandoffAudio {
        proof_sha256,
        proof,
        audio,
    })
}

fn validate_reconstruction(
    proof: &ProductStemHandoff,
    audio: &[(ProductStemHandoffArtifactRole, PathBuf, SourceAudioCache)],
) -> Result<(), JamAppError> {
    let samples = |role| {
        audio
            .iter()
            .find(|(candidate, _, _)| *candidate == role)
            .map(|(_, _, cache)| cache.interleaved_samples())
            .expect("validated handoff has exact artifact roles")
    };
    let drums = samples(ProductStemHandoffArtifactRole::StemDrums);
    let music = samples(ProductStemHandoffArtifactRole::StemMusic);
    let bass = samples(ProductStemHandoffArtifactRole::StemBass);
    let full_mix = samples(ProductStemHandoffArtifactRole::FullGridMix);
    let mut max_abs_error = 0.0_f64;
    let mut squared_error_sum = 0.0_f64;
    for index in 0..full_mix.len() {
        let error = f64::from(drums[index]) + f64::from(music[index]) + f64::from(bass[index])
            - f64::from(full_mix[index]);
        max_abs_error = max_abs_error.max(error.abs());
        squared_error_sum += error * error;
    }
    let rms_error = (squared_error_sum / full_mix.len().max(1) as f64).sqrt();
    if max_abs_error > PRODUCT_STEM_PCM_MAX_ABS_ERROR
        || rms_error > PRODUCT_STEM_PCM_MAX_RMS_ERROR
        || (max_abs_error - proof.reconstruction.max_abs_error).abs()
            > PRODUCT_STEM_DECLARED_METRIC_TOLERANCE
        || (rms_error - proof.reconstruction.rms_error).abs()
            > PRODUCT_STEM_DECLARED_METRIC_TOLERANCE
    {
        return Err(JamAppError::InvalidSession(format!(
            "product-stem reconstruction mismatch: measured max {max_abs_error:.9} RMS {rms_error:.9}"
        )));
    }
    Ok(())
}

fn require_pcm16_wav(path: &Path) -> Result<(), JamAppError> {
    let bytes = fs::read(path)?;
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(JamAppError::InvalidSession(format!(
            "product-stem artifact is not RIFF/WAVE: {}",
            path.display()
        )));
    }
    let mut cursor = 12_usize;
    while cursor.checked_add(8).is_some_and(|end| end <= bytes.len()) {
        let chunk_len = u32::from_le_bytes([
            bytes[cursor + 4],
            bytes[cursor + 5],
            bytes[cursor + 6],
            bytes[cursor + 7],
        ]) as usize;
        let chunk_start = cursor + 8;
        let chunk_end = chunk_start.checked_add(chunk_len).ok_or_else(|| {
            JamAppError::InvalidSession("product-stem WAV chunk length overflow".into())
        })?;
        if chunk_end > bytes.len() {
            return Err(JamAppError::InvalidSession(
                "product-stem WAV chunk extends past file end".into(),
            ));
        }
        if &bytes[cursor..cursor + 4] == b"fmt " {
            if chunk_len < 16 {
                return Err(JamAppError::InvalidSession(
                    "product-stem WAV fmt chunk is too short".into(),
                ));
            }
            let audio_format = u16::from_le_bytes([bytes[chunk_start], bytes[chunk_start + 1]]);
            let bits_per_sample =
                u16::from_le_bytes([bytes[chunk_start + 14], bytes[chunk_start + 15]]);
            if audio_format == 1 && bits_per_sample == 16 {
                return Ok(());
            }
            return Err(JamAppError::InvalidSession(format!(
                "product-stem WAV must be uncompressed PCM16: {}",
                path.display()
            )));
        }
        cursor = chunk_end.saturating_add(chunk_len % 2);
    }
    Err(JamAppError::InvalidSession(format!(
        "product-stem WAV is missing its fmt chunk: {}",
        path.display()
    )))
}

fn required_local_path(value: &Option<String>, label: &str) -> Result<PathBuf, JamAppError> {
    value
        .as_ref()
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| JamAppError::InvalidSession(format!("{label} is missing")))
}

fn normalized_sha256(value: &str) -> &str {
    value.trim().strip_prefix("sha256:").unwrap_or(value.trim())
}

fn product_stem_role_label(role: ProductStemHandoffArtifactRole) -> &'static str {
    match role {
        ProductStemHandoffArtifactRole::StemDrums => "stem_drums",
        ProductStemHandoffArtifactRole::StemMusic => "stem_music",
        ProductStemHandoffArtifactRole::StemBass => "stem_bass",
        ProductStemHandoffArtifactRole::FullGridMix => "full_grid_mix",
    }
}
