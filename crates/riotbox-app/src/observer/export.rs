use std::collections::BTreeSet;

use riotbox_core::{
    action::{Action, ActionCommand, ActionStatus},
    export_readiness::ExportScope,
    session::{ExportReceiptState, StemPackageReceiptReadinessBlocker},
};
use serde_json::{Value, json};

use crate::jam_app::StemPackageExportSurfaceGate;
use crate::ui::JamShellState;

pub(super) fn export_observer_snapshot(shell: &JamShellState) -> Value {
    let actions = export_actions(shell);
    let lifecycle = actions
        .iter()
        .flat_map(|action| export_lifecycle_records(shell, action))
        .collect::<Vec<_>>();

    json!({
        "present": !lifecycle.is_empty(),
        "receipt_count": shell.app.session.export_receipts.len(),
        "lifecycle": lifecycle,
        "stem_package_surface_gate": stem_package_surface_gate_observer_snapshot(
            &shell.app.stem_package_export_surface_gate()
        ),
    })
}

fn export_actions(shell: &JamShellState) -> Vec<&Action> {
    let mut seen = BTreeSet::new();
    let mut actions = Vec::new();

    for action in &shell.app.session.action_log.actions {
        push_export_action(&mut actions, &mut seen, action);
    }
    for action in shell.app.queue.history() {
        push_export_action(&mut actions, &mut seen, action);
    }
    for action in shell.app.queue.pending_actions() {
        push_export_action(&mut actions, &mut seen, action);
    }

    actions
}

fn push_export_action<'a>(
    actions: &mut Vec<&'a Action>,
    seen: &mut BTreeSet<u64>,
    action: &'a Action,
) {
    if is_export_action(action.command) && seen.insert(action.id.0) {
        actions.push(action);
    }
}

fn is_export_action(command: ActionCommand) -> bool {
    matches!(
        command,
        ActionCommand::ExportProductMix | ActionCommand::ExportStemPackage
    )
}

fn export_lifecycle_records(shell: &JamShellState, action: &Action) -> Vec<Value> {
    let receipt = shell
        .app
        .session
        .export_receipts
        .iter()
        .find(|receipt| receipt.created_by_action == action.id);
    let mut records = vec![export_lifecycle_record(
        "requested",
        action.requested_at,
        action,
        receipt,
        None,
    )];

    match action.status {
        ActionStatus::Committed => {
            let timestamp = action.committed_at.unwrap_or(action.requested_at);
            records.push(export_lifecycle_record(
                "started", timestamp, action, receipt, None,
            ));
            records.push(export_lifecycle_record(
                "completed",
                timestamp,
                action,
                receipt,
                None,
            ));
        }
        ActionStatus::Rejected | ActionStatus::Failed => {
            let reason = action.result.as_ref().map(|result| result.summary.as_str());
            records.push(export_lifecycle_record(
                "started",
                action.requested_at,
                action,
                receipt,
                None,
            ));
            records.push(export_lifecycle_record(
                "failed",
                action.requested_at,
                action,
                receipt,
                reason,
            ));
        }
        ActionStatus::Requested | ActionStatus::Queued | ActionStatus::PendingCommit => {}
        ActionStatus::Undone => {}
    }

    records
}

fn export_lifecycle_record(
    stage: &str,
    timestamp_ms: u64,
    action: &Action,
    receipt: Option<&ExportReceiptState>,
    failure_reason: Option<&str>,
) -> Value {
    json!({
        "stage": stage,
        "timestamp_ms": timestamp_ms,
        "action_id": action.id.0,
        "command": action.command.as_str(),
        "status": format!("{:?}", action.status),
        "result": action.result.as_ref().map(|result| result.summary.as_str()),
        "failure_reason": failure_reason,
        "receipt": receipt.map(export_receipt_observer_snapshot),
    })
}

fn export_receipt_observer_snapshot(receipt: &ExportReceiptState) -> Value {
    let artifact_set = receipt.artifact_set_or_legacy();
    json!({
        "receipt_id": receipt.receipt_id.to_string(),
        "created_by_action": receipt.created_by_action.0,
        "export_scope": receipt.export_scope,
        "pack_id": receipt.pack_id,
        "export_role": receipt.export_role,
        "export_boundary": receipt.export_boundary,
        "artifact_path": receipt.artifact_path,
        "proof_path": receipt.proof_path,
        "manifest_path": receipt.manifest_path,
        "export_hash": receipt.export_hash,
        "normalized_manifest_hash": receipt.normalized_manifest_hash,
        "artifact_set": artifact_set,
        "qa_gates": receipt.qa_gates,
        "readiness_status": receipt.readiness_status,
        "stem_package_readiness": stem_package_readiness_observer_snapshot(receipt),
        "unsupported_scopes": receipt
            .unsupported_scopes
            .iter()
            .map(|scope| json!(scope))
            .collect::<Vec<_>>(),
        "unsupported_scope_labels": receipt
            .unsupported_scopes
            .iter()
            .map(|scope| scope.musician_label())
            .collect::<Vec<_>>(),
    })
}

fn stem_package_readiness_observer_snapshot(receipt: &ExportReceiptState) -> Option<Value> {
    if receipt.export_scope != ExportScope::StemPackage {
        return None;
    }

    let report = receipt.stem_package_readiness_report();
    let blockers = report.blockers.clone();
    Some(json!({
        "status": report.status,
        "ready": report.ready(),
        "blockers": blockers,
        "blocker_labels": blockers
            .iter()
            .map(stem_package_readiness_blocker_label)
            .collect::<Vec<_>>(),
    }))
}

fn stem_package_readiness_blocker_label(
    blocker: &StemPackageReceiptReadinessBlocker,
) -> &'static str {
    match blocker {
        StemPackageReceiptReadinessBlocker::NotStemPackageScope => {
            "receipt is not a stem package export"
        }
        StemPackageReceiptReadinessBlocker::UnsupportedScopeFlagPresent => {
            "stem package export is still marked unsupported"
        }
        StemPackageReceiptReadinessBlocker::MissingArtifactSetQaGate => {
            "stem package artifact-set QA gate is missing"
        }
        StemPackageReceiptReadinessBlocker::DeferredArtifactSetQaGate => {
            "stem package artifact-set QA gate is deferred"
        }
        StemPackageReceiptReadinessBlocker::FailedArtifactSetQaGate => {
            "stem package artifact-set QA gate failed"
        }
        StemPackageReceiptReadinessBlocker::MissingHashStabilityQaGate => {
            "stem package hash-stability QA gate is missing"
        }
        StemPackageReceiptReadinessBlocker::DeferredHashStabilityQaGate => {
            "stem package hash-stability QA gate is deferred"
        }
        StemPackageReceiptReadinessBlocker::FailedHashStabilityQaGate => {
            "stem package hash-stability QA gate failed"
        }
        StemPackageReceiptReadinessBlocker::MissingNonSilenceQaGate => {
            "stem package non-silence QA gate is missing"
        }
        StemPackageReceiptReadinessBlocker::DeferredNonSilenceQaGate => {
            "stem package non-silence QA gate is deferred"
        }
        StemPackageReceiptReadinessBlocker::FailedNonSilenceQaGate => {
            "stem package non-silence QA gate failed"
        }
        StemPackageReceiptReadinessBlocker::MissingLineageQaGate => {
            "stem package lineage QA gate is missing"
        }
        StemPackageReceiptReadinessBlocker::DeferredLineageQaGate => {
            "stem package lineage QA gate is deferred"
        }
        StemPackageReceiptReadinessBlocker::FailedLineageQaGate => {
            "stem package lineage QA gate failed"
        }
        StemPackageReceiptReadinessBlocker::MissingFallbackComparisonQaGate => {
            "stem package fallback-comparison QA gate is missing"
        }
        StemPackageReceiptReadinessBlocker::DeferredFallbackComparisonQaGate => {
            "stem package fallback-comparison QA gate is deferred"
        }
        StemPackageReceiptReadinessBlocker::FailedFallbackComparisonQaGate => {
            "stem package fallback-comparison QA gate failed"
        }
    }
}

fn stem_package_surface_gate_observer_snapshot(gate: &StemPackageExportSurfaceGate) -> Value {
    json!({
        "status": gate.status.as_str(),
        "runnable": gate.runnable(),
        "blockers": gate
            .blockers
            .iter()
            .map(|blocker| blocker.as_str())
            .collect::<Vec<_>>(),
        "blocker_labels": gate
            .blockers
            .iter()
            .map(|blocker| blocker.musician_label())
            .collect::<Vec<_>>(),
    })
}
