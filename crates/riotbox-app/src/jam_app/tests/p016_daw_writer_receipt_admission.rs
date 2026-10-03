use std::{
    fs,
    path::{Path, PathBuf},
};

use riotbox_core::{
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionStatus, ActionTarget, ActorType,
        DawSessionExportBoundary, Quantization, TargetScope,
    },
    export_readiness::{ExportScope, ProductExportDestinationKind},
    ids::{ActionId, ExportReceiptId},
};

use crate::jam_app::{
    JamAppState, product_export::DawSessionExportQueueResult,
    tests::p016_daw_session_export_action::daw_session_writer_export_state,
};
use crate::{
    observer::observer_snapshot,
    ui::{JamShellState, ShellLaunchMode},
};

#[test]
fn local_writer_rejects_a_later_receipt_before_any_file_or_session_mutation() {
    let dir = tempfile::tempdir().unwrap();
    let destination = dir.path().join("writer");
    let mut state = daw_session_writer_export_state(dir.path(), &destination, true);
    let action_id = queue_writer(&mut state, dir.path(), &destination);
    let mut later = state.session.export_receipts[0].clone();
    later.receipt_id = ExportReceiptId::from("export-receipt-a-0099");
    later.created_by_action = ActionId(99);
    state.session.export_receipts.push(later);
    let original_receipt = state.session.export_receipts[0].clone();
    assert_rejected_without_mutation(&mut state, dir.path(), &destination, action_id);

    let retry_id = queue_writer(&mut state, dir.path(), &destination);
    let receipt = state
        .commit_daw_session_writer_export(Some(dir.path()), &destination, 1_000)
        .unwrap();
    assert_eq!(receipt.receipt_id.as_str(), "export-receipt-a-0099");
    assert_eq!(state.session.export_receipts[0], original_receipt);
    assert_written_identity(&state, &destination, retry_id, "export-receipt-a-0099");

    // Only consume metadata projections; this starts neither a TUI nor audio.
    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    let snapshot = observer_snapshot(&shell);
    let lifecycle = snapshot["export"]["lifecycle"].as_array().unwrap();
    let rejected = lifecycle
        .iter()
        .filter(|event| event["action_id"] == action_id.0)
        .collect::<Vec<_>>();
    assert_eq!(rejected.len(), 3);
    assert_eq!(rejected[2]["stage"], "failed");
    assert!(rejected.iter().all(|event| event["stage"] != "completed"));
    let completed = lifecycle
        .iter()
        .find(|event| event["action_id"] == retry_id.0 && event["stage"] == "completed")
        .unwrap();
    assert_eq!(completed["receipt"]["receipt_id"], "export-receipt-a-0099");
    assert!(!shell.app.daw_session_export_surface_gate().runnable());
}

#[test]
fn local_writer_rejects_missing_empty_unknown_or_wrong_scope_queued_identity() {
    for receipt_id in [
        None,
        Some(""),
        Some("unknown"),
        Some("export-receipt-a-0099"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let destination = dir.path().join("writer");
        let mut state = daw_session_writer_export_state(dir.path(), &destination, true);
        let mut unrelated = state.session.export_receipts[0].clone();
        unrelated.receipt_id = ExportReceiptId::from("export-receipt-a-0099");
        unrelated.created_by_action = ActionId(99);
        unrelated.export_scope = ExportScope::ProductMix;
        state.session.export_receipts.push(unrelated);
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::ExportDawSession,
            Quantization::Immediate,
            ActionTarget {
                scope: Some(TargetScope::Session),
                ..Default::default()
            },
        );
        draft.params = ActionParams::DawSessionExport {
            export_scope: ExportScope::DawSession,
            boundary: DawSessionExportBoundary::LocalProjectWriterV1,
            duration: None,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
            destination_path: Some(destination.to_string_lossy().into_owned()),
            receipt_id: receipt_id.map(str::to_owned),
        };
        let action_id = state.queue.enqueue(draft, 960);
        assert_rejected_without_mutation(&mut state, dir.path(), &destination, action_id);
    }
}

#[test]
fn local_writer_rejects_removed_or_no_longer_daw_receipt() {
    for remove in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let destination = dir.path().join("writer");
        let mut state = daw_session_writer_export_state(dir.path(), &destination, true);
        let action_id = queue_writer(&mut state, dir.path(), &destination);
        if remove {
            state.session.export_receipts.clear();
        } else {
            state.session.export_receipts[0].export_scope = ExportScope::ProductMix;
        }
        assert_rejected_without_mutation(&mut state, dir.path(), &destination, action_id);
    }
}

#[test]
fn later_non_daw_receipt_does_not_displace_selected_writer_receipt() {
    let dir = tempfile::tempdir().unwrap();
    let destination = dir.path().join("writer");
    let mut state = daw_session_writer_export_state(dir.path(), &destination, true);
    let action_id = queue_writer(&mut state, dir.path(), &destination);
    let mut unrelated = state.session.export_receipts[0].clone();
    unrelated.receipt_id = ExportReceiptId::from("export-receipt-a-0099");
    unrelated.created_by_action = ActionId(99);
    unrelated.export_scope = ExportScope::ProductMix;
    state.session.export_receipts.push(unrelated.clone());
    let receipt = state
        .commit_daw_session_writer_export(Some(dir.path()), &destination, 980)
        .unwrap();
    assert_eq!(receipt.receipt_id.as_str(), "export-receipt-a-0042");
    assert_eq!(state.session.export_receipts[1], unrelated);
    assert_written_identity(&state, &destination, action_id, "export-receipt-a-0042");
}

fn assert_rejected_without_mutation(
    state: &mut JamAppState,
    base: &Path,
    destination: &Path,
    action_id: ActionId,
) {
    let before = state.session.clone();
    let before_files = fixture_snapshot(base);
    let result = state.commit_daw_session_writer_export(Some(base), destination, 980);

    assert!(
        result.is_err(),
        "must not write the newer receipt under the queued identity"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("queue the export again")
    );
    assert_eq!(state.session, before);
    assert_eq!(fixture_snapshot(base), before_files);
    assert!(!destination.join("daw_session_writer").exists());
    assert!(state.queue.pending_actions().is_empty());
    assert_eq!(
        state.queue.history_action(action_id).unwrap().status,
        ActionStatus::Rejected
    );
}

fn assert_written_identity(
    state: &JamAppState,
    destination: &Path,
    action_id: ActionId,
    receipt_id: &str,
) {
    assert_eq!(state.session.action_log.actions.len(), 1);
    let action = &state.session.action_log.actions[0];
    assert_eq!(action.id, action_id);
    assert_eq!(action.status, ActionStatus::Committed);
    assert!(
        matches!(&action.params, ActionParams::DawSessionExport { receipt_id: Some(id), .. } if id == receipt_id)
    );
    assert_eq!(state.session.action_log.commit_records.len(), 1);
    assert_eq!(
        state.session.action_log.commit_records[0].action_id,
        action_id
    );
    for name in ["local_project_skeleton.json", "writer_proof.json"] {
        let proof: serde_json::Value = serde_json::from_slice(
            &fs::read(destination.join("daw_session_writer").join(name)).unwrap(),
        )
        .unwrap();
        assert_eq!(proof["receipt_id"], receipt_id);
    }
}

fn queue_writer(state: &mut JamAppState, base: &Path, destination: &Path) -> ActionId {
    match state.queue_daw_session_writer_export(
        960,
        Some(base),
        Some(destination.to_string_lossy().into_owned()),
    ) {
        DawSessionExportQueueResult::Enqueued { action_id } => action_id,
        other => panic!("expected writer queue admission, got {other:?}"),
    }
}

fn fixture_snapshot(root: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    // Bounded generated metadata fixture only; record directories and file bytes
    // so a rejected operation cannot hide staging debris or in-place changes.
    let mut pending = vec![root.to_path_buf()];
    let mut snapshot = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let bytes = if entry.file_type().unwrap().is_dir() {
                pending.push(path.clone());
                None
            } else {
                Some(fs::read(&path).unwrap())
            };
            snapshot.push((path.strip_prefix(root).unwrap().to_path_buf(), bytes));
        }
    }
    snapshot.sort();
    snapshot
}
