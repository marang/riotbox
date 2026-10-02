use crate::jam_app::StemPackageExportSurfaceBlocker;
use crate::jam_app::StemPackageExportSurfaceGate;
use crate::ui::lane_diagnostics::w30_target_compact;
use crate::ui::shell_state::JamShellState;
use crate::ui::stem_package_inspect::stem_package_export_receipt_lines;
use ratatui::text::Line;
use riotbox_core::action::Action;
use riotbox_core::action::ActionCommand;
use riotbox_core::action::ActionStatus;
use riotbox_core::export_readiness::ExportScope;
use riotbox_core::export_readiness::ProductExportBoundary;
use riotbox_core::export_readiness::ProductExportRole;
use riotbox_core::export_readiness::UnsupportedExportScope;
use riotbox_core::export_readiness::default_unsupported_export_scopes;
use riotbox_core::session::ExportReceiptState;

pub(in crate::ui) fn material_inspect_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let capture = &shell.app.jam_view.capture;
    let export_lines = export_readiness_lines(shell);
    let mut lines = vec![
        Line::from(format!(
            "captures {} | pending {}",
            capture.capture_count, capture.pending_capture_count
        )),
        Line::from(format!("w30 {}", w30_target_compact(shell))),
        Line::from(format!(
            "last {}",
            capture.last_capture_id.as_deref().unwrap_or("none")
        )),
        Line::from(format!(
            "target {}",
            capture
                .last_capture_target
                .as_deref()
                .unwrap_or("unassigned")
        )),
        Line::from(format!(
            "notes {}",
            capture
                .last_capture_notes
                .as_deref()
                .unwrap_or("no capture note yet")
        )),
    ];
    if export_lines.len() > 4 {
        lines.truncate(1);
    } else if export_lines.len() > 2 {
        lines.truncate(3);
    }
    lines.extend(export_lines);
    lines
}

pub(in crate::ui) fn export_readiness_lines(shell: &JamShellState) -> Vec<Line<'static>> {
    let latest_receipt = shell.app.session.export_receipts.last();
    let latest_failure = latest_export_failure(shell);
    if let Some(failure) = latest_failure
        && latest_receipt
            .map(|receipt| failure.requested_at > receipt.created_at)
            .unwrap_or(true)
    {
        return with_stem_package_surface_gate_lines(shell, export_failure_lines(failure));
    }

    let Some(receipt) = latest_receipt else {
        let role = ProductExportRole::FullGridMix.as_str();
        let boundary =
            export_boundary_short_label(ProductExportBoundary::FeralGridGeneratedSupport);
        let unsupported = default_unsupported_export_scopes()
            .into_iter()
            .map(unsupported_export_scope_short_label)
            .collect::<Vec<_>>()
            .join("/");

        return with_stem_package_surface_gate_lines(
            shell,
            vec![
                Line::from(format!("export {role} | {boundary}")),
                Line::from(format!("reproducible | no {unsupported}")),
            ],
        );
    };

    with_stem_package_surface_gate_lines(shell, export_receipt_lines(receipt))
}

pub(in crate::ui) fn with_stem_package_surface_gate_lines(
    shell: &JamShellState,
    mut lines: Vec<Line<'static>>,
) -> Vec<Line<'static>> {
    lines.extend(stem_package_surface_gate_lines(
        &shell.app.stem_package_export_surface_gate(),
    ));
    lines
}

pub(in crate::ui) fn stem_package_surface_gate_lines(
    gate: &StemPackageExportSurfaceGate,
) -> Vec<Line<'static>> {
    vec![
        Line::from(format!("stem_package surface | {}", gate.status.as_str())),
        Line::from(format!(
            "needs {}",
            compact_stem_package_surface_blockers(&gate.blockers)
        )),
    ]
}

pub(in crate::ui) fn compact_stem_package_surface_blockers(
    blockers: &[StemPackageExportSurfaceBlocker],
) -> String {
    if blockers.is_empty() {
        return "none".into();
    }

    blockers
        .iter()
        .map(|blocker| blocker.compact_label())
        .collect::<Vec<_>>()
        .join("/")
}

pub(in crate::ui) fn export_receipt_lines(receipt: &ExportReceiptState) -> Vec<Line<'static>> {
    if receipt.export_scope == ExportScope::StemPackage {
        return stem_package_export_receipt_lines(receipt);
    }

    let role = receipt.export_role.as_str();
    let boundary = export_boundary_short_label(receipt.export_boundary);
    let unsupported = receipt
        .unsupported_scopes
        .iter()
        .copied()
        .map(unsupported_export_scope_short_label)
        .collect::<Vec<_>>()
        .join("/");
    let path_status = if receipt.artifact_path.is_empty() || receipt.proof_path.is_empty() {
        "path missing"
    } else {
        "wav+proof"
    };

    vec![
        Line::from(format!("export {role} | {boundary}")),
        Line::from(format!(
            "{} {} | {path_status} | no {unsupported}",
            compact_export_receipt_id(receipt),
            export_readiness_status_label(receipt.readiness_status)
        )),
    ]
}

pub(in crate::ui) fn latest_export_failure(shell: &JamShellState) -> Option<&Action> {
    shell.app.queue.history().iter().rev().find(|action| {
        action.command == ActionCommand::ExportProductMix
            && matches!(action.status, ActionStatus::Rejected | ActionStatus::Failed)
    })
}

pub(in crate::ui) fn export_failure_lines(action: &Action) -> Vec<Line<'static>> {
    let reason = action
        .result
        .as_ref()
        .map(|result| compact_export_failure_reason(&result.summary))
        .unwrap_or_else(|| "unknown reason".into());

    vec![
        Line::from("export full_grid_mix | failed"),
        Line::from(format!("a-{:04} | {reason}", action.id.0)),
    ]
}

pub(in crate::ui) fn compact_export_failure_reason(reason: &str) -> String {
    const MAX_LEN: usize = 52;
    let reason = reason.trim();
    if reason.len() <= MAX_LEN {
        return reason.into();
    }
    format!("{}...", reason.chars().take(MAX_LEN).collect::<String>())
}

pub(in crate::ui) fn compact_export_receipt_id(receipt: &ExportReceiptState) -> &str {
    receipt
        .receipt_id
        .as_str()
        .strip_prefix("export-receipt-")
        .unwrap_or_else(|| receipt.receipt_id.as_str())
}

pub(in crate::ui) fn export_readiness_status_label(
    status: riotbox_core::export_readiness::ExportReadinessStatus,
) -> &'static str {
    match status {
        riotbox_core::export_readiness::ExportReadinessStatus::Reproducible => "ok",
    }
}

pub(in crate::ui) fn export_boundary_short_label(boundary: ProductExportBoundary) -> &'static str {
    match boundary {
        ProductExportBoundary::FeralGridGeneratedSupport => "feral-grid",
        ProductExportBoundary::StemPackageLocalCiPackageV1 => "stem-pkg",
        ProductExportBoundary::StemPackageSourceMatchedHandoffV1 => "stem-src",
        ProductExportBoundary::StemPackageW30HookLoopV1
        | ProductExportBoundary::StemPackageW30HookLoopV2
        | ProductExportBoundary::StemPackageW30HookLoopV3
        | ProductExportBoundary::StemPackageW30HookLoopV4 => "w30-hook",
        ProductExportBoundary::ArrangementDawPlacementContractV1 => "arrange-daw",
        ProductExportBoundary::DawSessionW30HookDawprojectV1 => "w30-daw",
        ProductExportBoundary::DawSessionLiveMasterDawprojectV1 => "live-master-daw",
        ProductExportBoundary::DawSessionLiveMasterDawprojectV2 => "live-master-daw-v2",
        ProductExportBoundary::DawSessionLiveMasterDawprojectV3 => "live-master-daw-v3",
        ProductExportBoundary::LiveRecordingReceiptContractV1 => "live-rec",
        ProductExportBoundary::LiveRecordingRuntimeMasterCaptureV1 => "live-master",
        ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV2 => "live-bar",
        ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV3 => "live-bar-v3",
        ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV4 => "live-bar-v4",
    }
}

pub(in crate::ui) fn unsupported_export_scope_short_label(
    scope: UnsupportedExportScope,
) -> &'static str {
    match scope {
        UnsupportedExportScope::StemPackage => "stem",
        UnsupportedExportScope::LiveRecording => "live",
        UnsupportedExportScope::DawExport => "DAW",
        UnsupportedExportScope::HostAudioSoak => "host",
    }
}

#[cfg(test)]
mod tests {
    use super::export_boundary_short_label;
    use riotbox_core::export_readiness::ProductExportBoundary;

    #[test]
    fn live_export_labels_keep_historical_and_successor_versions_distinct() {
        for (boundary, label) in [
            (
                ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV2,
                "live-bar",
            ),
            (
                ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV3,
                "live-bar-v3",
            ),
            (
                ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV4,
                "live-bar-v4",
            ),
            (
                ProductExportBoundary::DawSessionLiveMasterDawprojectV1,
                "live-master-daw",
            ),
            (
                ProductExportBoundary::DawSessionLiveMasterDawprojectV2,
                "live-master-daw-v2",
            ),
        ] {
            assert_eq!(export_boundary_short_label(boundary), label);
        }
    }
}
