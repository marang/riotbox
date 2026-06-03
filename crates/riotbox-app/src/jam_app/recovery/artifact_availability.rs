use riotbox_core::persistence::load_session_json;
use riotbox_core::persistence::{SessionRecoveryCandidate, SessionRecoveryCandidateStatus};

use crate::jam_app::{capture_artifacts, product_export};

pub(super) fn recovery_artifact_availability_label(candidate: &SessionRecoveryCandidate) -> String {
    if !matches!(
        candidate.status,
        SessionRecoveryCandidateStatus::ParseableSession
    ) {
        return "artifacts unchecked".into();
    }

    let Ok(session) = load_session_json(&candidate.path) else {
        return "artifacts unreadable".into();
    };

    let capture_count = session.captures.len();
    let export_receipt_count = session.export_receipts.len();
    if capture_count == 0 && export_receipt_count == 0 {
        return "artifacts n/a | no captures".into();
    }

    let base_dir = candidate.path.parent();
    let mut ready = 0usize;
    let mut missing = 0usize;
    let mut unreadable = 0usize;
    let mut missing_identity = 0usize;
    let mut missing_placement = 0usize;
    let mut missing_tempo_map = 0usize;

    for capture in &session.captures {
        match capture_artifacts::preflight_capture_artifact_hydration(capture, base_dir) {
            Ok(_) => ready += 1,
            Err(
                capture_artifacts::CaptureArtifactHydrationPreflightError::MissingStoragePath {
                    ..
                }
                | capture_artifacts::CaptureArtifactHydrationPreflightError::MissingSessionFileSet {
                    ..
                },
            ) => missing_identity += 1,
            Err(capture_artifacts::CaptureArtifactHydrationPreflightError::MissingArtifact {
                ..
            }) => missing += 1,
            Err(
                capture_artifacts::CaptureArtifactHydrationPreflightError::UnreadableArtifact {
                    ..
                }
                | capture_artifacts::CaptureArtifactHydrationPreflightError::NotFile { .. },
            ) => unreadable += 1,
        }
    }

    for receipt in &session.export_receipts {
        match product_export::preflight_export_receipt_artifacts(receipt, base_dir) {
            Ok(_) => ready += 1,
            Err(
                product_export::ExportReceiptArtifactPreflightError::MissingArtifactPath { .. }
                | product_export::ExportReceiptArtifactPreflightError::MissingProofPath { .. }
                | product_export::ExportReceiptArtifactPreflightError::MissingArtifactSetPath {
                    ..
                }
                | product_export::ExportReceiptArtifactPreflightError::MissingSessionFileSet {
                    ..
                },
            ) => missing_identity += 1,
            Err(
                product_export::ExportReceiptArtifactPreflightError::MissingExportArtifact {
                    ..
                }
                | product_export::ExportReceiptArtifactPreflightError::MissingProofArtifact {
                    ..
                }
                | product_export::ExportReceiptArtifactPreflightError::MissingArtifactSetArtifact {
                    ..
                },
            ) => missing += 1,
            Err(
                product_export::ExportReceiptArtifactPreflightError::UnreadableArtifact { .. }
                | product_export::ExportReceiptArtifactPreflightError::NotFile { .. }
                | product_export::ExportReceiptArtifactPreflightError::ArtifactSetNotFile { .. }
                | product_export::ExportReceiptArtifactPreflightError::UnreadableArtifactSetArtifact {
                    ..
                },
            ) => unreadable += 1,
            Err(product_export::ExportReceiptArtifactPreflightError::ArrangementPlacementBlocked {
                ..
            }) => missing_placement += 1,
            Err(product_export::ExportReceiptArtifactPreflightError::DawTempoMapBlocked {
                ..
            }) => missing_tempo_map += 1,
        }
    }

    let artifact_count = capture_count + export_receipt_count;
    if ready == artifact_count {
        return artifact_ready_label(capture_count, export_receipt_count);
    }

    let mut blockers = Vec::new();
    if missing_identity > 0 {
        blockers.push(format!("{missing_identity} missing identity"));
    }
    if missing_placement > 0 {
        blockers.push(format!("{missing_placement} missing placement"));
    }
    if missing_tempo_map > 0 {
        blockers.push(format!("{missing_tempo_map} missing tempo map"));
    }
    if missing > 0 {
        blockers.push(format!("{missing} missing"));
    }
    if unreadable > 0 {
        blockers.push(format!("{unreadable} unreadable"));
    }

    format!(
        "artifacts blocked: {} of {} | {}",
        artifact_count - ready,
        artifact_count,
        blockers.join(", ")
    )
}

fn artifact_ready_label(capture_count: usize, export_receipt_count: usize) -> String {
    match (capture_count, export_receipt_count) {
        (0, receipts) => format!("artifacts ready: {receipts} export receipt(s)"),
        (captures, 0) => format!("artifacts ready: {captures} capture(s)"),
        (captures, receipts) => {
            format!("artifacts ready: {captures} capture(s), {receipts} export receipt(s)")
        }
    }
}
