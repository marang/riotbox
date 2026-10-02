//! Restore/replay validation of versioned recording windows and their DAW handoffs.
use std::collections::BTreeMap;

use super::{ExportReceiptState, SessionFile};
use crate::{
    action::{
        Action, ActionCommand, ActionParams, ActionStatus, DawSessionExportBoundary,
        LiveRecordingExportBoundary,
    },
    export_readiness::{ExportScope, ProductExportBoundary, ProductExportDestinationKind},
    ids::{ActionId, ExportReceiptId},
};

type ReceiptIndex<'a> = BTreeMap<&'a ExportReceiptId, (&'a ExportReceiptState, usize)>;

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
    /// Validate recording V3/V4 and DAW V3 duration/source identity without file I/O.
    /// Legacy omitted-duration receipts retain their existing restore policy.
    pub fn validate_live_recording_duration_contracts(
        &self,
    ) -> Result<(), LiveRecordingDurationContractError> {
        let mut receipts_by_action = BTreeMap::new();
        let mut receipts_by_id = ReceiptIndex::new();
        for receipt in &self.export_receipts {
            receipts_by_action
                .entry(receipt.created_by_action)
                .and_modify(|(_, count)| *count += 1)
                .or_insert((receipt, 1_usize));
            receipts_by_id
                .entry(&receipt.receipt_id)
                .and_modify(|(_, count)| *count += 1)
                .or_insert((receipt, 1));
        }
        let mut actions_by_id = BTreeMap::new();
        for action in &self.action_log.actions {
            actions_by_id
                .entry(action.id)
                .and_modify(|(_, count)| *count += 1)
                .or_insert((action, 1_usize));
        }
        for action in &self.action_log.actions {
            let explicit_contract = match &action.params {
                ActionParams::LiveRecordingExport {
                    boundary, duration, ..
                } => {
                    matches!(
                        boundary,
                        LiveRecordingExportBoundary::RuntimeMasterBarWindowV3
                            | LiveRecordingExportBoundary::RuntimeMasterBarWindowV4
                    ) || duration.is_some()
                }
                ActionParams::DawSessionExport {
                    boundary, duration, ..
                } => {
                    *boundary == DawSessionExportBoundary::LiveMasterDawprojectV3
                        || duration.is_some()
                }
                _ => false,
            };
            if action.status != ActionStatus::Committed || !explicit_contract {
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
            validate_action_receipt(action, receipt, &receipts_by_id)?;
        }
        for receipt in &self.export_receipts {
            if !matches!(
                receipt.export_boundary,
                ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV3
                    | ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV4
                    | ProductExportBoundary::DawSessionLiveMasterDawprojectV3
            ) && receipt.live_recording_duration.is_none()
            {
                continue;
            }
            if receipts_by_id[&receipt.receipt_id].1 != 1 {
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
            validate_action_receipt(action, receipt, &receipts_by_id)?;
        }
        Ok(())
    }
}

fn validate_action_receipt(
    action: &Action,
    receipt: &ExportReceiptState,
    receipts_by_id: &ReceiptIndex<'_>,
) -> Result<(), LiveRecordingDurationContractError> {
    if matches!(action.params, ActionParams::DawSessionExport { .. }) {
        return validate_daw_action_receipt(action, receipt, receipts_by_id);
    }
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

fn validate_daw_action_receipt(
    action: &Action,
    receipt: &ExportReceiptState,
    receipts_by_id: &ReceiptIndex<'_>,
) -> Result<(), LiveRecordingDurationContractError> {
    let ActionParams::DawSessionExport {
        export_scope,
        boundary,
        duration,
        include_manifest,
        destination_kind,
        destination_path,
        receipt_id,
    } = &action.params
    else {
        unreachable!("caller selected DAW action params")
    };
    if action.command != ActionCommand::ExportDawSession
        || action.status != ActionStatus::Committed
        || *export_scope != ExportScope::DawSession
        || *boundary != DawSessionExportBoundary::LiveMasterDawprojectV3
        || !boundary.valid_duration(*duration)
        || !include_manifest
        || *destination_kind != ProductExportDestinationKind::LocalFilePath
    {
        return Err(LiveRecordingDurationContractError::InvalidAction {
            action_id: action.id,
        });
    }
    if !receipt.live_master_dawproject_archive_ready() {
        return Err(LiveRecordingDurationContractError::InvalidReceipt {
            receipt_id: receipt.receipt_id.clone(),
        });
    }
    let source = receipt_id
        .as_ref()
        .map(ExportReceiptId::new)
        .and_then(|id| receipts_by_id.get(&id))
        .filter(|(_, count)| *count == 1)
        .map(|(source, _)| *source);
    if !receipt.live_master_dawproject_action_contract_matches(*boundary, *duration)
        || destination_path.as_deref() != Some(receipt.artifact_path.as_str())
        || !source
            .is_some_and(|source| receipt.live_master_dawproject_source_contract_matches(source))
    {
        return Err(LiveRecordingDurationContractError::ActionReceiptMismatch {
            action_id: action.id,
        });
    }
    Ok(())
}
