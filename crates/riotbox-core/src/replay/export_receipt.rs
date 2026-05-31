use crate::{
    action::ActionCommand,
    ids::{ActionId, ExportReceiptId},
    replay::ReplayPlanEntry,
    session::{ExportReceiptState, SessionFile},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportReceiptReplayValidationPlan {
    pub action_id: ActionId,
    pub receipt_id: ExportReceiptId,
    pub artifact_path: String,
    pub proof_path: String,
    pub export_hash: String,
    pub normalized_manifest_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportReceiptReplayValidationError {
    NotProductExportAction {
        action_id: ActionId,
        command: ActionCommand,
    },
    MissingExportReceipt {
        action_id: ActionId,
    },
    AmbiguousExportReceipt {
        action_id: ActionId,
        receipt_count: usize,
    },
    MissingArtifactPath {
        receipt_id: ExportReceiptId,
    },
    MissingProofPath {
        receipt_id: ExportReceiptId,
    },
}

pub fn plan_export_receipt_replay_validation(
    session: &SessionFile,
    entry: &ReplayPlanEntry<'_>,
) -> Result<ExportReceiptReplayValidationPlan, ExportReceiptReplayValidationError> {
    let action = entry.action;
    if action.command != ActionCommand::ExportProductMix {
        return Err(ExportReceiptReplayValidationError::NotProductExportAction {
            action_id: action.id,
            command: action.command,
        });
    }

    let receipt = export_receipt_for_action(session, action.id)?;
    if receipt.artifact_path.trim().is_empty() {
        return Err(ExportReceiptReplayValidationError::MissingArtifactPath {
            receipt_id: receipt.receipt_id.clone(),
        });
    }
    if receipt.proof_path.trim().is_empty() {
        return Err(ExportReceiptReplayValidationError::MissingProofPath {
            receipt_id: receipt.receipt_id.clone(),
        });
    }

    Ok(ExportReceiptReplayValidationPlan {
        action_id: action.id,
        receipt_id: receipt.receipt_id.clone(),
        artifact_path: receipt.artifact_path.clone(),
        proof_path: receipt.proof_path.clone(),
        export_hash: receipt.export_hash.clone(),
        normalized_manifest_hash: receipt.normalized_manifest_hash.clone(),
    })
}

fn export_receipt_for_action(
    session: &SessionFile,
    action_id: ActionId,
) -> Result<&ExportReceiptState, ExportReceiptReplayValidationError> {
    let mut matches = session
        .export_receipts
        .iter()
        .filter(|receipt| receipt.created_by_action == action_id);
    let Some(receipt) = matches.next() else {
        return Err(ExportReceiptReplayValidationError::MissingExportReceipt { action_id });
    };
    if matches.next().is_some() {
        let receipt_count = session
            .export_receipts
            .iter()
            .filter(|receipt| receipt.created_by_action == action_id)
            .count();
        return Err(ExportReceiptReplayValidationError::AmbiguousExportReceipt {
            action_id,
            receipt_count,
        });
    }

    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use crate::{
        action::{
            Action, ActionParams, ActionResult, ActionStatus, ActionTarget, ActorType,
            CommitBoundary, Quantization, TargetScope, UndoPolicy,
        },
        export_readiness::{
            ExportReadinessStatus, ProductExportBoundary, ProductExportDestinationKind,
            ProductExportRole, UnsupportedExportScope,
        },
        ids::ExportReceiptId,
        replay::build_committed_replay_plan,
        session::{ActionCommitRecord, ActionLog, ExportReceiptState, ReplayPolicy},
        transport::CommitBoundaryState,
    };

    use super::*;

    #[test]
    fn export_receipt_replay_validation_reads_receipt_metadata_without_side_effects() {
        let session =
            session_with_export_receipt("exports/full_grid_mix.wav", "exports/proof.json");
        let plan = build_committed_replay_plan(&session.action_log).expect("replay plan");

        let validation =
            plan_export_receipt_replay_validation(&session, &plan[0]).expect("receipt validation");

        assert_eq!(validation.action_id, ActionId(4));
        assert_eq!(
            validation.receipt_id,
            ExportReceiptId::from("export-receipt-a-0004")
        );
        assert_eq!(validation.artifact_path, "exports/full_grid_mix.wav");
        assert_eq!(validation.proof_path, "exports/proof.json");
        assert_eq!(
            validation.export_hash,
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
    }

    #[test]
    fn export_receipt_replay_validation_reports_missing_receipt() {
        let mut session =
            session_with_export_receipt("exports/full_grid_mix.wav", "exports/proof.json");
        session.export_receipts.clear();
        let plan = build_committed_replay_plan(&session.action_log).expect("replay plan");

        let error = plan_export_receipt_replay_validation(&session, &plan[0])
            .expect_err("missing receipt should reject");

        assert_eq!(
            error,
            ExportReceiptReplayValidationError::MissingExportReceipt {
                action_id: ActionId(4)
            }
        );
    }

    #[test]
    fn export_receipt_replay_validation_reports_missing_artifact_identity() {
        let session = session_with_export_receipt(" ", "exports/proof.json");
        let plan = build_committed_replay_plan(&session.action_log).expect("replay plan");

        let error = plan_export_receipt_replay_validation(&session, &plan[0])
            .expect_err("missing artifact path should reject");

        assert_eq!(
            error,
            ExportReceiptReplayValidationError::MissingArtifactPath {
                receipt_id: ExportReceiptId::from("export-receipt-a-0004")
            }
        );
    }

    fn session_with_export_receipt(artifact_path: &str, proof_path: &str) -> SessionFile {
        let mut session =
            SessionFile::new("session-export-replay", "0.1.0", "2026-05-31T00:00:00Z");
        session.action_log = ActionLog {
            actions: vec![export_action()],
            commit_records: vec![ActionCommitRecord {
                action_id: ActionId(4),
                boundary: CommitBoundaryState {
                    kind: CommitBoundary::Immediate,
                    beat_index: 0,
                    bar_index: 0,
                    phrase_index: 0,
                    scene_id: None,
                },
                commit_sequence: 1,
                committed_at: 900,
            }],
            replay_policy: ReplayPolicy::DeterministicPreferred,
        };
        session.export_receipts.push(ExportReceiptState {
            receipt_id: ExportReceiptId::from("export-receipt-a-0004"),
            created_by_action: ActionId(4),
            created_at: 900,
            export_role: ProductExportRole::FullGridMix,
            export_boundary: ProductExportBoundary::FeralGridGeneratedSupport,
            artifact_path: artifact_path.into(),
            proof_path: proof_path.into(),
            manifest_path: None,
            export_hash: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            normalized_manifest_hash:
                "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".into(),
            readiness_status: ExportReadinessStatus::Reproducible,
            unsupported_scopes: vec![
                UnsupportedExportScope::StemPackage,
                UnsupportedExportScope::LiveRecording,
                UnsupportedExportScope::DawExport,
                UnsupportedExportScope::HostAudioSoak,
            ],
        });
        session
    }

    fn export_action() -> Action {
        Action {
            id: ActionId(4),
            actor: ActorType::User,
            command: ActionCommand::ExportProductMix,
            params: ActionParams::ProductExport {
                export_role: ProductExportRole::FullGridMix,
                boundary: ProductExportBoundary::FeralGridGeneratedSupport,
                include_manifest: true,
                destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
                destination_path: Some("exports".into()),
            },
            target: ActionTarget {
                scope: Some(TargetScope::Session),
                ..Default::default()
            },
            requested_at: 800,
            quantization: Quantization::Immediate,
            status: ActionStatus::Committed,
            committed_at: Some(900),
            result: Some(ActionResult {
                accepted: true,
                summary: "exported full_grid_mix".into(),
            }),
            undo_policy: UndoPolicy::NotUndoable {
                reason: "export writes files".into(),
            },
            explanation: Some("export full_grid_mix product proof".into()),
        }
    }
}
