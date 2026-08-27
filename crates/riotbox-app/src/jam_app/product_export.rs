use std::{fs, io::Read, path::Path};

use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType, Quantization,
        StemPackageExportBoundary, StemPackageExportRole, StemPackageFallbackComparisonPolicy,
        StemPackageLineagePolicy, TargetScope, UndoPolicy,
    },
    export_readiness::{
        ExportScope, ProductExportBoundary, ProductExportDestinationKind, ProductExportRole,
        STEM_PACKAGE_LOCAL_CI_PACK_ID, STEM_PACKAGE_SOURCE_MATCHED_PACK_ID,
    },
    ids::ActionId,
    queue::QueueEnqueueResult,
    session::ExportArtifactRole,
};
use sha2::{Digest, Sha256};

use super::{JamAppError, JamAppState, QueueControlResult};

pub(in crate::jam_app) use super::product_export_artifact_preflight::{
    ExportReceiptArtifactPreflightError, preflight_export_receipt_artifacts,
};

mod daw_session_export_commit;
mod daw_session_export_queue;
mod daw_session_surface_gate;
mod live_recording_export_queue;
mod product_mix_export_commit;
pub use daw_session_surface_gate::{
    DawSessionExportSurfaceBlocker, DawSessionExportSurfaceGate, DawSessionExportSurfaceStatus,
    daw_session_export_surface_gate_for_session,
};
#[allow(unused_imports)]
pub use live_recording_export_queue::{
    LIVE_RECORDING_EXPORT_RESERVED_REASON, LiveRecordingExportQueueResult,
};

pub const STEM_PACKAGE_EXPORT_RESERVED_REASON: &str = "stem package export is disabled for musicians; current packages are operator proof only until DAW placement and listening review are ready";

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
            Self::CiWriterProofMissing => "stem writer proof is missing",
            Self::StemPackageReceiptReadinessBlocked => "stem receipt QA is still blocked",
            Self::StemPackageReceiptIdentityMissing => {
                "stem receipt identity is not an accepted operator package boundary"
            }
            Self::DeveloperProofOnly => "current stem package is operator proof only",
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
            handoff_proof_path: None,
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
        let accepted_identity = matches!(
            (receipt.pack_id.as_str(), receipt.export_boundary),
            (
                STEM_PACKAGE_LOCAL_CI_PACK_ID,
                ProductExportBoundary::StemPackageLocalCiPackageV1
            ) | (
                STEM_PACKAGE_SOURCE_MATCHED_PACK_ID,
                ProductExportBoundary::StemPackageSourceMatchedHandoffV1
            )
        ) && receipt.export_role == ProductExportRole::PackageManifest;
        if !accepted_identity {
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
