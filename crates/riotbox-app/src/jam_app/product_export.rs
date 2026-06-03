use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType, Quantization,
        StemPackageExportBoundary, StemPackageExportRole, StemPackageFallbackComparisonPolicy,
        StemPackageLineagePolicy, TargetScope, UndoPolicy,
    },
    export_readiness::{
        ExportReadinessContract, ExportScope, ProductExportBoundary, ProductExportDestinationKind,
        ProductExportReproducibilityProof, ProductExportRole, STEM_PACKAGE_LOCAL_CI_PACK_ID,
    },
    ids::ActionId,
    queue::QueueEnqueueResult,
    session::{ActionCommitRecord, ExportArtifactRole, ExportArtifactSetEntry, ExportReceiptState},
    transport::CommitBoundaryState,
};
use sha2::{Digest, Sha256};

use super::{
    JamAppError, JamAppState, QueueControlResult,
    helpers::update_logged_action_result,
    product_export_receipt::{
        attach_product_export_artifact_audio_metrics, attach_product_export_artifact_lineage,
    },
};

pub(in crate::jam_app) use super::product_export_artifact_preflight::{
    ExportReceiptArtifactPreflightError, preflight_export_receipt_artifacts,
};

mod daw_session_export_commit;
mod daw_session_export_queue;
mod daw_session_surface_gate;
mod live_recording_export_queue;
pub use daw_session_surface_gate::{
    DawSessionExportSurfaceBlocker, DawSessionExportSurfaceGate, DawSessionExportSurfaceStatus,
    daw_session_export_surface_gate_for_session,
};
#[allow(unused_imports)]
pub use live_recording_export_queue::{
    LIVE_RECORDING_EXPORT_RESERVED_REASON, LiveRecordingExportQueueResult,
};

pub const STEM_PACKAGE_EXPORT_RESERVED_REASON: &str = "stem package export is disabled for musicians; local CI packages are developer proof only until DAW placement and listening review are ready";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StemPackageExportSurfaceGate {
    pub status: StemPackageExportSurfaceStatus,
    pub blockers: Vec<StemPackageExportSurfaceBlocker>,
}

impl StemPackageExportSurfaceGate {
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            status: StemPackageExportSurfaceStatus::Disabled,
            blockers: vec![
                StemPackageExportSurfaceBlocker::CiWriterProofMissing,
                StemPackageExportSurfaceBlocker::DeveloperProofOnly,
                StemPackageExportSurfaceBlocker::DawPlacementWorkflowMissing,
                StemPackageExportSurfaceBlocker::StructuredListeningReviewMissing,
            ],
        }
    }

    #[must_use]
    pub fn runnable(&self) -> bool {
        self.status == StemPackageExportSurfaceStatus::Runnable && self.blockers.is_empty()
    }

    #[must_use]
    pub fn musician_summary(&self) -> String {
        if self.runnable() {
            return "stem package export is ready for musicians".into();
        }

        format!(
            "{STEM_PACKAGE_EXPORT_RESERVED_REASON}; blockers: {}",
            self.blockers
                .iter()
                .map(|blocker| blocker.musician_label())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StemPackageExportSurfaceStatus {
    Disabled,
    Runnable,
}

impl StemPackageExportSurfaceStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Runnable => "runnable",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StemPackageExportSurfaceBlocker {
    CiWriterProofMissing,
    StemPackageReceiptReadinessBlocked,
    StemPackageReceiptIdentityMissing,
    DeveloperProofOnly,
    DawPlacementWorkflowMissing,
    StructuredListeningReviewMissing,
}

impl StemPackageExportSurfaceBlocker {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CiWriterProofMissing => "ci_writer_proof_missing",
            Self::StemPackageReceiptReadinessBlocked => "receipt_readiness_blocked",
            Self::StemPackageReceiptIdentityMissing => "receipt_identity_missing",
            Self::DeveloperProofOnly => "developer_proof_only",
            Self::DawPlacementWorkflowMissing => "daw_placement_workflow_missing",
            Self::StructuredListeningReviewMissing => "structured_listening_review_missing",
        }
    }

    #[must_use]
    pub const fn musician_label(self) -> &'static str {
        match self {
            Self::CiWriterProofMissing => "CI writer proof is missing",
            Self::StemPackageReceiptReadinessBlocked => "stem receipt QA is still blocked",
            Self::StemPackageReceiptIdentityMissing => {
                "stem receipt identity is not the local CI package boundary"
            }
            Self::DeveloperProofOnly => "local CI package is developer proof only",
            Self::DawPlacementWorkflowMissing => "DAW placement workflow is not ready",
            Self::StructuredListeningReviewMissing => "structured listening review is not verified",
        }
    }

    #[must_use]
    pub const fn compact_label(self) -> &'static str {
        match self {
            Self::CiWriterProofMissing => "ci-proof",
            Self::StemPackageReceiptReadinessBlocked => "qa",
            Self::StemPackageReceiptIdentityMissing => "identity",
            Self::DeveloperProofOnly => "dev-only",
            Self::DawPlacementWorkflowMissing => "DAW",
            Self::StructuredListeningReviewMissing => "listening",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StemPackageExportQueueResult {
    Rejected { reason: String },
    AlreadyPending,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DawSessionExportQueueResult {
    Enqueued { action_id: ActionId },
    Rejected { reason: String },
    AlreadyPending,
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

    pub fn queue_stem_package_export_reserved(
        &mut self,
        requested_at: TimestampMs,
        destination_path: Option<String>,
        claimed_stem_roles: Vec<ExportArtifactRole>,
    ) -> StemPackageExportQueueResult {
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
            boundary: StemPackageExportBoundary::ReservedContractOnly,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
            destination_path,
            claimed_stem_roles,
            lineage_policy: StemPackageLineagePolicy::RequireAnyCoreLineage,
            fallback_comparison_policy: StemPackageFallbackComparisonPolicy::Required,
        };
        draft.undo_policy = UndoPolicy::NotUndoable {
            reason: "stem package export writes files outside musical undo".into(),
        };
        draft.explanation = Some("reserved stem package export contract; not runnable yet".into());

        match self
            .queue
            .enqueue_if_no_pending_command(draft, requested_at)
        {
            QueueEnqueueResult::AlreadyPending { .. } => {
                StemPackageExportQueueResult::AlreadyPending
            }
            QueueEnqueueResult::Enqueued(action_id) => {
                let reason = self.stem_package_export_surface_gate().musician_summary();
                self.queue.reject(action_id, reason.clone());
                self.refresh_view();
                StemPackageExportQueueResult::Rejected { reason }
            }
        }
    }

    pub fn stem_package_export_surface_gate(&self) -> StemPackageExportSurfaceGate {
        let Some(receipt) = self
            .session
            .export_receipts
            .iter()
            .rev()
            .find(|receipt| receipt.export_scope == ExportScope::StemPackage)
        else {
            return StemPackageExportSurfaceGate::disabled();
        };

        let mut blockers = Vec::new();
        if !receipt.stem_package_readiness_report().ready() {
            blockers.push(StemPackageExportSurfaceBlocker::StemPackageReceiptReadinessBlocked);
        }
        if receipt.pack_id != STEM_PACKAGE_LOCAL_CI_PACK_ID
            || receipt.export_role != ProductExportRole::PackageManifest
            || receipt.export_boundary != ProductExportBoundary::StemPackageLocalCiPackageV1
        {
            blockers.push(StemPackageExportSurfaceBlocker::StemPackageReceiptIdentityMissing);
        }

        blockers.extend([
            StemPackageExportSurfaceBlocker::DeveloperProofOnly,
            StemPackageExportSurfaceBlocker::DawPlacementWorkflowMissing,
            StemPackageExportSurfaceBlocker::StructuredListeningReviewMissing,
        ]);

        StemPackageExportSurfaceGate {
            status: StemPackageExportSurfaceStatus::Disabled,
            blockers,
        }
    }

    pub fn daw_session_export_surface_gate(&self) -> DawSessionExportSurfaceGate {
        daw_session_export_surface_gate_for_session(&self.session)
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
        receipt
            .artifact_set
            .push(ExportArtifactSetEntry::product_export_proof(
                written.proof_path.to_string_lossy().into_owned(),
                written.proof_hash.clone(),
            ));
        attach_product_export_artifact_lineage(&mut receipt, &self.session);
        attach_product_export_artifact_audio_metrics(&mut receipt);
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
    let proof_hash = sha256_file(&destination_proof)?;

    Ok(WrittenProductMixExport {
        contract,
        artifact_path: destination_artifact,
        proof_path: destination_proof,
        proof_hash,
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

pub(super) fn sha256_file(path: &Path) -> Result<String, JamAppError> {
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
