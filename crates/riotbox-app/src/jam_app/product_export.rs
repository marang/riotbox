use std::{
    fs, io,
    io::Read,
    path::{Path, PathBuf},
};

use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType, Quantization,
        TargetScope, UndoPolicy,
    },
    export_readiness::{
        ExportReadinessContract, ExportScope, ProductExportBoundary, ProductExportDestinationKind,
        ProductExportReproducibilityProof, ProductExportRole,
    },
    ids::ActionId,
    session::{
        ActionCommitRecord, ExportArtifactLocation, ExportArtifactRole,
        ExportArtifactSourceGraphRef, ExportReceiptState,
    },
    transport::CommitBoundaryState,
};
use sha2::{Digest, Sha256};

use super::{JamAppError, JamAppState, QueueControlResult, helpers::update_logged_action_result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::jam_app) enum ExportReceiptArtifactPreflightError {
    MissingArtifactPath {
        receipt_id: riotbox_core::ids::ExportReceiptId,
    },
    MissingProofPath {
        receipt_id: riotbox_core::ids::ExportReceiptId,
    },
    MissingArtifactSetPath {
        receipt_id: riotbox_core::ids::ExportReceiptId,
        role: ExportArtifactRole,
    },
    MissingSessionFileSet {
        receipt_id: riotbox_core::ids::ExportReceiptId,
    },
    MissingExportArtifact {
        receipt_id: riotbox_core::ids::ExportReceiptId,
        path: PathBuf,
    },
    MissingProofArtifact {
        receipt_id: riotbox_core::ids::ExportReceiptId,
        path: PathBuf,
    },
    MissingArtifactSetArtifact {
        receipt_id: riotbox_core::ids::ExportReceiptId,
        role: ExportArtifactRole,
        path: PathBuf,
    },
    NotFile {
        receipt_id: riotbox_core::ids::ExportReceiptId,
        path: PathBuf,
    },
    ArtifactSetNotFile {
        receipt_id: riotbox_core::ids::ExportReceiptId,
        role: ExportArtifactRole,
        path: PathBuf,
    },
    UnreadableArtifact {
        receipt_id: riotbox_core::ids::ExportReceiptId,
        path: PathBuf,
        reason: String,
    },
    UnreadableArtifactSetArtifact {
        receipt_id: riotbox_core::ids::ExportReceiptId,
        role: ExportArtifactRole,
        path: PathBuf,
        reason: String,
    },
}

impl JamAppState {
    pub fn queue_product_mix_export(
        &mut self,
        requested_at: TimestampMs,
        destination_path: Option<String>,
    ) -> QueueControlResult {
        if self
            .queue
            .pending_actions()
            .iter()
            .any(|action| action.command == ActionCommand::ExportProductMix)
        {
            return QueueControlResult::AlreadyPending;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::ExportProductMix,
            Quantization::Immediate,
            ActionTarget {
                scope: Some(TargetScope::Session),
                ..Default::default()
            },
        );
        draft.params = ActionParams::ProductExport {
            export_scope: ExportScope::ProductMix,
            export_role: ProductExportRole::FullGridMix,
            boundary: ProductExportBoundary::FeralGridGeneratedSupport,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
            destination_path,
        };
        draft.undo_policy = UndoPolicy::NotUndoable {
            reason: "export writes files; deleting them is outside musical undo".into(),
        };
        draft.explanation = Some("export full_grid_mix product proof".into());

        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn commit_product_mix_export_from_proof(
        &mut self,
        proof_path: impl AsRef<Path>,
        destination_dir: impl AsRef<Path>,
        requested_at: TimestampMs,
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

        let export_result = prepare_product_mix_export(proof_path.as_ref(), destination_dir);
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
        if let Some(source_graph_ref) = self.export_artifact_source_graph_ref() {
            receipt.attach_artifact_source_graph_ref(
                ExportArtifactRole::FullGridMix,
                source_graph_ref,
            );
        }
        let result_summary = format!(
            "exported {} receipt {} hash {}",
            written.contract.export_role.as_str(),
            receipt.receipt_id,
            written.contract.export_sha256
        );

        let committed_ref = self
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

        self.session.action_log.actions.push(action);
        self.session
            .action_log
            .commit_records
            .push(ActionCommitRecord {
                action_id,
                boundary: committed_ref.boundary,
                commit_sequence: committed_ref.commit_sequence,
                committed_at: requested_at,
            });
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

    fn export_artifact_source_graph_ref(&self) -> Option<ExportArtifactSourceGraphRef> {
        self.session
            .source_graph_refs
            .first()
            .map(|graph_ref| ExportArtifactSourceGraphRef {
                source_id: graph_ref.source_id.clone(),
                graph_version: graph_ref.graph_version,
                graph_hash: graph_ref.graph_hash.clone(),
            })
    }
}

struct WrittenProductMixExport {
    contract: ExportReadinessContract,
    artifact_path: PathBuf,
    proof_path: PathBuf,
}

fn prepare_product_mix_export(
    proof_path: &Path,
    destination_dir: &Path,
) -> Result<WrittenProductMixExport, JamAppError> {
    let proof: ProductExportReproducibilityProof =
        serde_json::from_str(&fs::read_to_string(proof_path)?)?;
    let contract = ExportReadinessContract::from_product_export_proof(&proof)
        .map_err(|error| JamAppError::InvalidSession(format!("{error:?}")))?;
    let source_artifact = resolve_proof_artifact_path(proof_path, &contract.export_artifact);
    let source_hash = sha256_file(&source_artifact)?;
    if source_hash != contract.export_sha256 {
        return Err(JamAppError::InvalidSession(format!(
            "export artifact hash mismatch for {}: proof {} actual {}",
            contract.export_role.as_str(),
            contract.export_sha256,
            source_hash
        )));
    }

    fs::create_dir_all(destination_dir)?;
    let artifact_file_name = source_artifact.file_name().ok_or_else(|| {
        JamAppError::InvalidSession("export artifact path has no file name".into())
    })?;
    let destination_artifact = destination_dir.join(artifact_file_name);
    copy_file_if_distinct(&source_artifact, &destination_artifact)?;

    let destination_proof = destination_dir.join("product_export_proof.json");
    copy_file_if_distinct(proof_path, &destination_proof)?;

    Ok(WrittenProductMixExport {
        contract,
        artifact_path: destination_artifact,
        proof_path: destination_proof,
    })
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

fn sha256_file(path: &Path) -> Result<String, JamAppError> {
    let mut digest = Sha256::new();
    let mut file = fs::File::open(path)?;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn copy_file_if_distinct(from: &Path, to: &Path) -> Result<(), JamAppError> {
    if from != to {
        fs::copy(from, to)?;
    }
    Ok(())
}

pub(in crate::jam_app) fn preflight_export_receipt_artifacts(
    receipt: &ExportReceiptState,
    base_dir: Option<&Path>,
) -> Result<(PathBuf, PathBuf), ExportReceiptArtifactPreflightError> {
    let artifact_path = resolve_receipt_path(
        receipt,
        &receipt.artifact_path,
        base_dir,
        ReceiptPathKind::Artifact,
    )?;
    let proof_path = resolve_receipt_path(
        receipt,
        &receipt.proof_path,
        base_dir,
        ReceiptPathKind::Proof,
    )?;

    let artifact = require_receipt_file(receipt, artifact_path, ReceiptPathKind::Artifact)?;
    let proof = require_receipt_file(receipt, proof_path, ReceiptPathKind::Proof)?;
    preflight_artifact_set_entries(receipt, base_dir)?;

    Ok((artifact, proof))
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ReceiptPathKind {
    Artifact,
    Proof,
    ArtifactSet(ExportArtifactRole),
}

fn preflight_artifact_set_entries(
    receipt: &ExportReceiptState,
    base_dir: Option<&Path>,
) -> Result<(), ExportReceiptArtifactPreflightError> {
    for artifact in receipt.artifact_set_or_legacy() {
        match artifact.location {
            ExportArtifactLocation::LocalPath { path } => {
                let artifact_path = resolve_receipt_path(
                    receipt,
                    &path,
                    base_dir,
                    ReceiptPathKind::ArtifactSet(artifact.role),
                )?;
                require_receipt_file(
                    receipt,
                    artifact_path,
                    ReceiptPathKind::ArtifactSet(artifact.role),
                )?;
            }
            ExportArtifactLocation::Uri { uri } if uri.trim().is_empty() => {
                return Err(
                    ExportReceiptArtifactPreflightError::MissingArtifactSetPath {
                        receipt_id: receipt.receipt_id.clone(),
                        role: artifact.role,
                    },
                );
            }
            ExportArtifactLocation::Uri { .. } => {}
        }
    }

    Ok(())
}

fn resolve_receipt_path(
    receipt: &ExportReceiptState,
    path: &str,
    base_dir: Option<&Path>,
    kind: ReceiptPathKind,
) -> Result<PathBuf, ExportReceiptArtifactPreflightError> {
    let path = path.trim();
    if path.is_empty() {
        return Err(match kind {
            ReceiptPathKind::Artifact => ExportReceiptArtifactPreflightError::MissingArtifactPath {
                receipt_id: receipt.receipt_id.clone(),
            },
            ReceiptPathKind::Proof => ExportReceiptArtifactPreflightError::MissingProofPath {
                receipt_id: receipt.receipt_id.clone(),
            },
            ReceiptPathKind::ArtifactSet(role) => {
                ExportReceiptArtifactPreflightError::MissingArtifactSetPath {
                    receipt_id: receipt.receipt_id.clone(),
                    role,
                }
            }
        });
    }

    let path = Path::new(path);
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    let base_dir =
        base_dir.ok_or_else(
            || ExportReceiptArtifactPreflightError::MissingSessionFileSet {
                receipt_id: receipt.receipt_id.clone(),
            },
        )?;
    Ok(base_dir.join(path))
}

fn require_receipt_file(
    receipt: &ExportReceiptState,
    path: PathBuf,
    kind: ReceiptPathKind,
) -> Result<PathBuf, ExportReceiptArtifactPreflightError> {
    match std::fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() => Ok(path),
        Ok(_) => Err(match kind {
            ReceiptPathKind::ArtifactSet(role) => {
                ExportReceiptArtifactPreflightError::ArtifactSetNotFile {
                    receipt_id: receipt.receipt_id.clone(),
                    role,
                    path,
                }
            }
            ReceiptPathKind::Artifact | ReceiptPathKind::Proof => {
                ExportReceiptArtifactPreflightError::NotFile {
                    receipt_id: receipt.receipt_id.clone(),
                    path,
                }
            }
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Err(match kind {
            ReceiptPathKind::Artifact => {
                ExportReceiptArtifactPreflightError::MissingExportArtifact {
                    receipt_id: receipt.receipt_id.clone(),
                    path,
                }
            }
            ReceiptPathKind::Proof => ExportReceiptArtifactPreflightError::MissingProofArtifact {
                receipt_id: receipt.receipt_id.clone(),
                path,
            },
            ReceiptPathKind::ArtifactSet(role) => {
                ExportReceiptArtifactPreflightError::MissingArtifactSetArtifact {
                    receipt_id: receipt.receipt_id.clone(),
                    role,
                    path,
                }
            }
        }),
        Err(error) => Err(match kind {
            ReceiptPathKind::ArtifactSet(role) => {
                ExportReceiptArtifactPreflightError::UnreadableArtifactSetArtifact {
                    receipt_id: receipt.receipt_id.clone(),
                    role,
                    path,
                    reason: error.to_string(),
                }
            }
            ReceiptPathKind::Artifact | ReceiptPathKind::Proof => {
                ExportReceiptArtifactPreflightError::UnreadableArtifact {
                    receipt_id: receipt.receipt_id.clone(),
                    path,
                    reason: error.to_string(),
                }
            }
        }),
    }
}
