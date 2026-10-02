//! Restore/replay validation of explicitly selected, versioned recording windows.
use std::collections::BTreeMap;

use super::{ExportReceiptState, SessionFile};
use crate::{
    action::{Action, ActionCommand, ActionParams, ActionStatus, LiveRecordingExportBoundary},
    export_readiness::{ExportScope, ProductExportBoundary, ProductExportDestinationKind},
    ids::{ActionId, ExportReceiptId},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiveRecordingDurationContractError {
    InvalidAction { action_id: ActionId },
    MissingReceipt { action_id: ActionId },
    AmbiguousReceipt { action_id: ActionId },
    DuplicateReceiptId { receipt_id: ExportReceiptId },
    MissingAction { receipt_id: ExportReceiptId },
    AmbiguousAction { action_id: ActionId },
    InvalidReceipt { receipt_id: ExportReceiptId },
    ActionReceiptMismatch { action_id: ActionId },
}

impl SessionFile {
    /// Validate V3/V4 or explicit-duration state without loading or reproducing audio.
    /// Legacy omitted-duration V1/V2 receipts retain their existing restore policy.
    pub fn validate_live_recording_duration_contracts(
        &self,
    ) -> Result<(), LiveRecordingDurationContractError> {
        let mut receipts_by_action = BTreeMap::new();
        let mut receipt_id_counts = BTreeMap::new();
        for receipt in &self.export_receipts {
            receipts_by_action
                .entry(receipt.created_by_action)
                .and_modify(|(_, count)| *count += 1)
                .or_insert((receipt, 1_usize));
            *receipt_id_counts
                .entry(&receipt.receipt_id)
                .or_insert(0_usize) += 1;
        }
        let mut actions_by_id = BTreeMap::new();
        for action in &self.action_log.actions {
            actions_by_id
                .entry(action.id)
                .and_modify(|(_, count)| *count += 1)
                .or_insert((action, 1_usize));
        }
        for action in &self.action_log.actions {
            let ActionParams::LiveRecordingExport {
                boundary, duration, ..
            } = &action.params
            else {
                continue;
            };
            if action.status != ActionStatus::Committed
                || (!matches!(
                    boundary,
                    LiveRecordingExportBoundary::RuntimeMasterBarWindowV3
                        | LiveRecordingExportBoundary::RuntimeMasterBarWindowV4
                ) && duration.is_none())
            {
                continue;
            }
            let (receipt, count) = receipts_by_action.get(&action.id).ok_or(
                LiveRecordingDurationContractError::MissingReceipt {
                    action_id: action.id,
                },
            )?;
            if *count != 1 {
                return Err(LiveRecordingDurationContractError::AmbiguousReceipt {
                    action_id: action.id,
                });
            }
            validate_action_receipt(action, receipt)?;
        }
        for receipt in &self.export_receipts {
            if !matches!(
                receipt.export_boundary,
                ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV3
                    | ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV4
            ) && receipt.live_recording_duration.is_none()
            {
                continue;
            }
            if receipt_id_counts[&receipt.receipt_id] != 1 {
                return Err(LiveRecordingDurationContractError::DuplicateReceiptId {
                    receipt_id: receipt.receipt_id.clone(),
                });
            }
            let (action, count) =
                actions_by_id
                    .get(&receipt.created_by_action)
                    .ok_or_else(|| LiveRecordingDurationContractError::MissingAction {
                        receipt_id: receipt.receipt_id.clone(),
                    })?;
            if *count != 1 {
                return Err(LiveRecordingDurationContractError::AmbiguousAction {
                    action_id: action.id,
                });
            }
            validate_action_receipt(action, receipt)?;
        }
        Ok(())
    }
}

fn validate_action_receipt(
    action: &Action,
    receipt: &ExportReceiptState,
) -> Result<(), LiveRecordingDurationContractError> {
    let ActionParams::LiveRecordingExport {
        export_scope,
        boundary,
        duration,
        include_manifest,
        destination_kind,
        destination_path,
        receipt_id,
        ..
    } = &action.params
    else {
        return Err(LiveRecordingDurationContractError::InvalidAction {
            action_id: action.id,
        });
    };
    if action.command != ActionCommand::ExportLiveRecording
        || action.status != ActionStatus::Committed
        || *export_scope != ExportScope::LiveRecording
        || !boundary.valid_duration(*duration)
        || !include_manifest
        || *destination_kind != ProductExportDestinationKind::LocalFilePath
    {
        return Err(LiveRecordingDurationContractError::InvalidAction {
            action_id: action.id,
        });
    }
    if !receipt.live_recording_runtime_master_ready() {
        return Err(LiveRecordingDurationContractError::InvalidReceipt {
            receipt_id: receipt.receipt_id.clone(),
        });
    }
    if !receipt.live_recording_action_contract_matches(*boundary, *duration)
        || destination_path.as_deref() != Some(receipt.artifact_path.as_str())
        || receipt_id
            .as_ref()
            .is_some_and(|id| id != receipt.receipt_id.as_str())
    {
        return Err(LiveRecordingDurationContractError::ActionReceiptMismatch {
            action_id: action.id,
        });
    }
    Ok(())
}
