use std::{
    fs,
    fs::OpenOptions,
    io::{self, Read, Seek, Write},
    path::{Path, PathBuf},
};

use riotbox_core::{
    TimestampMs,
    action::ActionCommand,
    export_readiness::{ExportReadinessContract, ProductExportReproducibilityProof},
    ids::ActionId,
    session::{ExportArtifactSetEntry, ExportReceiptState},
    transport::CommitBoundaryState,
};
use sha2::{Digest, Sha256};

use crate::jam_app::{
    JamAppError, JamAppState,
    helpers::update_logged_action_result,
    product_export_receipt::{
        attach_product_export_artifact_audio_metrics, attach_product_export_artifact_lineage,
    },
};

use super::sha256_file;

#[cfg(test)]
mod copy_tests;
#[cfg(test)]
pub(super) mod test_support;

impl JamAppState {
    pub fn commit_product_mix_export_from_proof(
        &mut self,
        proof_path: impl AsRef<Path>,
        destination_dir: impl AsRef<Path>,
        requested_at: TimestampMs,
    ) -> Result<ExportReceiptState, JamAppError> {
        let expected_source_hash = self
            .source_graph
            .as_ref()
            .map(|graph| graph.source.content_hash.clone());
        self.commit_product_mix_export_from_proof_with_source_identity(
            proof_path,
            destination_dir,
            requested_at,
            expected_source_hash.as_deref(),
        )
    }

    pub fn commit_product_mix_export_from_active_source_proof(
        &mut self,
        proof_path: impl AsRef<Path>,
        destination_dir: impl AsRef<Path>,
        requested_at: TimestampMs,
    ) -> Result<ExportReceiptState, JamAppError> {
        let expected_source_hash = self
            .source_graph
            .as_ref()
            .map(|graph| graph.source.content_hash.clone())
            .filter(|hash| !hash.trim().is_empty());
        let Some(expected_source_hash) = expected_source_hash else {
            let error = JamAppError::InvalidSession(
                "product mix export requires an active Source Graph identity".into(),
            );
            self.reject_product_mix_export_request(requested_at, error.to_string());
            return Err(error);
        };
        self.commit_product_mix_export_from_proof_with_source_identity(
            proof_path,
            destination_dir,
            requested_at,
            Some(&expected_source_hash),
        )
    }

    pub fn reject_product_mix_export_request(
        &mut self,
        requested_at: TimestampMs,
        reason: impl Into<String>,
    ) -> ActionId {
        let action_id = self.pending_export_action_id().unwrap_or_else(|| {
            self.queue_product_mix_export(requested_at, None);
            self.pending_export_action_id()
                .expect("queued export action should be pending")
        });
        self.queue.reject(action_id, reason);
        self.refresh_view();
        action_id
    }

    fn commit_product_mix_export_from_proof_with_source_identity(
        &mut self,
        proof_path: impl AsRef<Path>,
        destination_dir: impl AsRef<Path>,
        requested_at: TimestampMs,
        expected_source_hash: Option<&str>,
    ) -> Result<ExportReceiptState, JamAppError> {
        let destination_dir = destination_dir.as_ref();
        let action_id = self.pending_export_action_id().unwrap_or_else(|| {
            self.queue_product_mix_export(
                requested_at,
                Some(destination_dir.to_string_lossy().into_owned()),
            );
            self.pending_export_action_id()
                .expect("queued export action should be pending")
        });

        let export_result =
            prepare_product_mix_export(proof_path.as_ref(), destination_dir, expected_source_hash);
        let written = match export_result {
            Ok(written) => written,
            Err(error) => {
                self.queue.reject(action_id, error.to_string());
                self.refresh_view();
                return Err(error);
            }
        };

        let boundary = CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Immediate,
            beat_index: self.runtime.transport.beat_index,
            bar_index: self.runtime.transport.bar_index,
            phrase_index: self.runtime.transport.phrase_index,
            scene_id: self.runtime.transport.current_scene.clone(),
        };
        let mut receipt = ExportReceiptState::from_readiness_contract(
            action_id,
            requested_at,
            &written.contract,
            written.artifact_path.to_string_lossy().into_owned(),
            written.proof_path.to_string_lossy().into_owned(),
            None,
        );
        receipt
            .artifact_set
            .push(ExportArtifactSetEntry::product_export_proof(
                written.proof_path.to_string_lossy().into_owned(),
                written.proof_hash.clone(),
            ));
        if expected_source_hash.is_some() {
            attach_product_export_artifact_lineage(&mut receipt, &self.session);
        }
        attach_product_export_artifact_audio_metrics(&mut receipt);
        let result_summary = format!(
            "exported {} receipt {} hash {}",
            written.contract.export_role.as_str(),
            receipt.receipt_id,
            written.contract.export_sha256
        );

        let mut committed_ref = self
            .queue
            .commit_pending_after_side_effect(
                action_id,
                boundary.clone(),
                requested_at,
                result_summary.clone(),
            )
            .ok_or_else(|| {
                JamAppError::InvalidSession(format!(
                    "queued export action {action_id} was not ready to commit"
                ))
            })?;
        let action = self
            .queue
            .history_action(committed_ref.action_id)
            .cloned()
            .ok_or_else(|| {
                JamAppError::InvalidSession(format!(
                    "committed export action {} missing from queue history",
                    committed_ref.action_id
                ))
            })?;

        self.record_committed_action(action, &mut committed_ref, requested_at);
        self.session.export_receipts.push(receipt.clone());
        update_logged_action_result(&mut self.session, action_id, result_summary);
        self.runtime.last_commit_boundary = Some(boundary);
        self.refresh_view();

        Ok(receipt)
    }

    fn pending_export_action_id(&self) -> Option<ActionId> {
        self.queue
            .pending_actions()
            .into_iter()
            .find(|action| action.command == ActionCommand::ExportProductMix)
            .map(|action| action.id)
    }
}

struct WrittenProductMixExport {
    contract: ExportReadinessContract,
    artifact_path: PathBuf,
    proof_path: PathBuf,
    proof_hash: String,
}

fn prepare_product_mix_export(
    proof_path: &Path,
    destination_dir: &Path,
    expected_source_hash: Option<&str>,
) -> Result<WrittenProductMixExport, JamAppError> {
    let proof_bytes = fs::read(proof_path)?;
    let proof: ProductExportReproducibilityProof = serde_json::from_slice(&proof_bytes)?;
    let contract = ExportReadinessContract::from_product_export_proof(&proof)
        .map_err(|error| JamAppError::InvalidSession(format!("{error:?}")))?;
    if let Some(expected_source_hash) = expected_source_hash {
        validate_product_export_source_identity(&contract.source_sha256, expected_source_hash)?;
    }
    let source_artifact = resolve_proof_artifact_path(proof_path, &contract.export_artifact);

    let artifact_file_name = source_artifact.file_name().ok_or_else(|| {
        JamAppError::InvalidSession("export artifact path has no file name".into())
    })?;
    let destination_artifact = destination_dir.join(artifact_file_name);
    let destination_proof = destination_dir.join("product_export_proof.json");
    if destination_artifact == destination_proof {
        return Err(JamAppError::InvalidSession(
            "product mix artifact name conflicts with product_export_proof.json".into(),
        ));
    }
    #[cfg(test)]
    test_support::run(test_support::ProductMixCheckpoint::BeforeArtifactSnapshot);
    // Only this private descriptor is used after admission; mutable handoff
    // paths are never reopened for publication. Memory use is buffer-bounded.
    let mut staged_artifact = tempfile::tempfile()?;
    let mut source = fs::File::open(&source_artifact)?;
    let source_hash = copy_hashed(&mut source, &mut staged_artifact)?;
    if source_hash != contract.export_sha256 {
        return Err(JamAppError::InvalidSession(format!(
            "export artifact hash mismatch for {}: proof {} actual {}",
            contract.export_role.as_str(),
            contract.export_sha256,
            source_hash
        )));
    }
    drop(source);
    staged_artifact.rewind()?;
    let proof_hash = format!("{:x}", Sha256::digest(&proof_bytes));
    #[cfg(test)]
    test_support::run(test_support::ProductMixCheckpoint::AfterArtifactSnapshot);

    match (
        destination_artifact.try_exists()?,
        destination_proof.try_exists()?,
    ) {
        (true, true) => {
            let existing_artifact_hash = sha256_file(&destination_artifact)?;
            let existing_proof_hash = sha256_file(&destination_proof)?;
            if existing_artifact_hash != contract.export_sha256 || existing_proof_hash != proof_hash
            {
                return Err(JamAppError::InvalidSession(
                    "product mix export destination contains a different existing bundle".into(),
                ));
            }
        }
        (true, false) | (false, true) => {
            return Err(JamAppError::InvalidSession(
                "product mix export destination contains an incomplete existing bundle".into(),
            ));
        }
        (false, false) => {
            fs::create_dir_all(destination_dir)?;
            copy_file_new_verified(&mut proof_bytes.as_slice(), &destination_proof, &proof_hash)?;
            #[cfg(test)]
            test_support::run(test_support::ProductMixCheckpoint::BeforeArtifactPublication);
            if let Err(error) = copy_file_new_verified(
                &mut staged_artifact,
                &destination_artifact,
                &contract.export_sha256,
            ) {
                let _ = fs::remove_file(&destination_proof);
                return Err(error);
            }
        }
    }

    Ok(WrittenProductMixExport {
        contract,
        artifact_path: destination_artifact,
        proof_path: destination_proof,
        proof_hash,
    })
}

fn validate_product_export_source_identity(
    proof_source_hash: &str,
    active_source_hash: &str,
) -> Result<(), JamAppError> {
    let proof_source_hash = normalized_sha256(proof_source_hash);
    let active_source_hash = normalized_sha256(active_source_hash);
    if proof_source_hash.is_empty() || active_source_hash.is_empty() {
        return Err(JamAppError::InvalidSession(
            "product mix export source identity is blank".into(),
        ));
    }
    if proof_source_hash != active_source_hash {
        return Err(JamAppError::InvalidSession(format!(
            "product mix export source mismatch: proof {proof_source_hash} active {active_source_hash}"
        )));
    }
    Ok(())
}

fn normalized_sha256(value: &str) -> &str {
    value.trim().strip_prefix("sha256:").unwrap_or(value.trim())
}

fn resolve_proof_artifact_path(proof_path: &Path, artifact_path: &str) -> PathBuf {
    let artifact_path = PathBuf::from(artifact_path);
    if artifact_path.is_absolute() {
        artifact_path
    } else {
        proof_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(artifact_path)
    }
}

fn copy_file_new_verified(
    source: &mut impl Read,
    to: &Path,
    expected_hash: &str,
) -> Result<(), JamAppError> {
    let mut destination = OpenOptions::new().write(true).create_new(true).open(to)?;
    let result = (|| {
        let actual_hash = copy_hashed(source, &mut destination)?;
        destination.flush()?;
        if actual_hash != expected_hash {
            return Err(JamAppError::InvalidSession(format!(
                "published product mix file hash mismatch: expected {expected_hash} actual {actual_hash}"
            )));
        }
        Ok(())
    })();
    if result.is_err() {
        drop(destination);
        let _ = fs::remove_file(to);
    }
    result
}

fn copy_hashed(source: &mut impl Read, destination: &mut impl Write) -> io::Result<String> {
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = match source.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        destination.write_all(&buffer[..read])?;
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}
