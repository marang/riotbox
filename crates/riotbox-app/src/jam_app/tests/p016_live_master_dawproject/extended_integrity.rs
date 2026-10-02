//! Generated admission and metadata-restore integrity for extended DAW handoffs.

use super::canonical_recording::{
    GeneratedRecording, append_generated_recording, embedded_proof, generated_recording,
};
use crate::jam_app::{
    JamAppState, LiveMasterRecordingProof, product_export::DawSessionExportQueueResult,
};
use riotbox_core::{
    action::{ActionParams, ActionStatus, DawSessionExportBoundary, LiveRecordingDuration},
    export_readiness::{ExportScope, ProductExportBoundary, ProductExportRole},
    persistence::save_session_json,
    session::ExportArtifactRole,
};
use sha2::{Digest, Sha256};
use std::fs;
use tempfile::tempdir;

#[test]
fn extended_handoff_restore_rejects_tampered_duration_pin_and_geometry_without_artifact_io() {
    let temp = tempdir().unwrap();
    let (state, recording) = generated_recording(
        temp.path(),
        GeneratedRecording::ExtendedV3(LiveRecordingDuration::EightBars),
        137.25,
        1_000,
        100,
    );
    let session_path = temp.path().join("session.json");
    save_session_json(&session_path, &state.session).unwrap();
    let mut state = JamAppState::from_json_files_for_export_metadata(&session_path).unwrap();
    let destination = temp.path().join("extended.dawproject");
    state
        .commit_and_save_live_master_dawproject_export(&destination, 3_000)
        .unwrap();
    fs::rename(&recording, temp.path().join("withheld.wav")).unwrap();
    fs::rename(
        &state.session.export_receipts[0].proof_path,
        temp.path().join("withheld-proof.json"),
    )
    .unwrap();
    fs::rename(&destination, temp.path().join("withheld.dawproject")).unwrap();
    assert!(JamAppState::from_json_files_for_export_metadata(&session_path).is_ok());
    for mutation in [
        "action-duration",
        "receipt-duration",
        "source-pin",
        "placement",
        "frames",
        "tempo",
    ] {
        let mut session = state.session.clone();
        match mutation {
            "action-duration" | "source-pin" => {
                let ActionParams::DawSessionExport {
                    duration,
                    receipt_id,
                    ..
                } = &mut session.action_log.actions.last_mut().unwrap().params
                else {
                    panic!("DAW action")
                };
                if mutation == "action-duration" {
                    *duration = None;
                } else {
                    *receipt_id = Some("missing-recording".into());
                }
            }
            _ => {
                let receipt = session.export_receipts.last_mut().unwrap();
                match mutation {
                    "receipt-duration" => {
                        receipt.live_recording_duration = Some(LiveRecordingDuration::SixteenBars)
                    }
                    "placement" => receipt.arrangement_placement_refs[0].end_bar = 2,
                    "frames" => {
                        let audio = receipt
                            .artifact_set
                            .iter_mut()
                            .find(|entry| entry.role == ExportArtifactRole::LiveRecordingCapture)
                            .unwrap();
                        *audio
                            .audio_metrics
                            .as_mut()
                            .unwrap()
                            .total_frame_count
                            .as_mut()
                            .unwrap() += 1;
                    }
                    "tempo" => receipt.daw_tempo_map_ref.as_mut().unwrap().bpm_micros = 120_000_000,
                    _ => unreachable!(),
                }
            }
        }
        save_session_json(&session_path, &session).unwrap();
        assert!(
            JamAppState::from_json_files_for_export_metadata(&session_path).is_err(),
            "{mutation}"
        );
        assert!(!recording.exists());
        assert!(!destination.exists());
    }
}

#[test]
fn queued_extended_handoff_revalidates_duration_and_proof_identity_before_publication() {
    let temp = tempdir().unwrap();
    let (ready, recording) = generated_recording(
        temp.path(),
        GeneratedRecording::ExtendedV3(LiveRecordingDuration::EightBars),
        137.25,
        1_000,
        100,
    );
    let source_bytes = fs::read(&recording).unwrap();
    let proof_path = &ready.session.export_receipts[0].proof_path;
    let original_proof_bytes = fs::read(proof_path).unwrap();
    for mutation in [
        "receipt-duration",
        "receipt-boundary",
        "recording-action",
        "proof-duration-missing",
        "proof-duration-other",
        "proof-schema",
        "proof-beats",
    ] {
        fs::write(proof_path, &original_proof_bytes).unwrap();
        let mut state = ready.clone();
        let destination = temp.path().join(format!("{mutation}.dawproject"));
        assert!(matches!(
            state.queue_live_master_dawproject_export(
                3_000,
                Some(destination.to_string_lossy().into_owned()),
            ),
            DawSessionExportQueueResult::Enqueued { .. }
        ));
        match mutation {
            "receipt-duration" => {
                state.session.export_receipts[0].live_recording_duration =
                    Some(LiveRecordingDuration::SixteenBars)
            }
            "receipt-boundary" => {
                state.session.export_receipts[0].export_boundary =
                    ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV4
            }
            "recording-action" => {
                let ActionParams::LiveRecordingExport { duration, .. } =
                    &mut state.session.action_log.actions[0].params
                else {
                    panic!("recording action")
                };
                *duration = Some(LiveRecordingDuration::SixteenBars);
            }
            _ => {
                let mut proof: LiveMasterRecordingProof =
                    serde_json::from_slice(&original_proof_bytes).unwrap();
                match mutation {
                    "proof-duration-missing" => proof.duration = None,
                    "proof-duration-other" => {
                        proof.duration = Some(LiveRecordingDuration::SixteenBars)
                    }
                    "proof-schema" => {
                        proof.schema = "riotbox.live_recording_runtime_master_bar_window.v2".into()
                    }
                    "proof-beats" => proof.duration_beats = 64,
                    _ => unreachable!(),
                }
                let bytes = serde_json::to_vec_pretty(&proof).unwrap();
                let hash = format!("{:x}", Sha256::digest(&bytes));
                fs::write(proof_path, bytes).unwrap();
                let source = &mut state.session.export_receipts[0];
                source.normalized_manifest_hash = hash.clone();
                source
                    .artifact_set
                    .iter_mut()
                    .find(|entry| entry.role == ExportArtifactRole::ProductExportProof)
                    .unwrap()
                    .sha256 = hash;
                assert!(source.live_recording_runtime_master_ready());
            }
        }
        assert!(
            state
                .commit_live_master_dawproject_export(&destination, 3_100)
                .is_err(),
            "{mutation}"
        );
        assert!(!destination.exists(), "{mutation}");
        assert_eq!(state.session.export_receipts.len(), 1);
        assert_eq!(
            state.queue.history().last().unwrap().status,
            ActionStatus::Rejected
        );
        assert_eq!(fs::read(&recording).unwrap(), source_bytes);
    }
}

#[test]
fn invalid_latest_extended_recording_never_falls_back_to_an_older_ready_take() {
    let temp = tempdir().unwrap();
    let newer_root = temp.path().join("newer");
    fs::create_dir(&newer_root).unwrap();
    let (mut ready, _) = generated_recording(
        temp.path(),
        GeneratedRecording::CanonicalV4,
        120.0,
        1_000,
        0,
    );
    let (newer, _) = generated_recording(
        &newer_root,
        GeneratedRecording::ExtendedV3(LiveRecordingDuration::EightBars),
        137.25,
        1_000,
        100,
    );
    append_generated_recording(&mut ready, newer);
    for mutation in ["missing-duration", "two-bars", "scope", "role", "hash"] {
        let mut state = ready.clone();
        let newest = state.session.export_receipts.last_mut().unwrap();
        match mutation {
            "missing-duration" => newest.live_recording_duration = None,
            "two-bars" => newest.live_recording_duration = Some(LiveRecordingDuration::TwoBars),
            "scope" => newest.export_scope = ExportScope::StemPackage,
            "role" => newest.export_role = ProductExportRole::FullGridMix,
            "hash" => newest.export_hash = "b".repeat(64),
            _ => unreachable!(),
        }
        assert!(state.session.export_receipts[0].live_recording_runtime_master_ready());
        let destination = temp.path().join(format!("{mutation}.dawproject"));
        let error = state
            .commit_live_master_dawproject_export(&destination, 3_000)
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("no older take may be substituted"),
            "{mutation}: {error}"
        );
        assert!(!destination.exists(), "{mutation}");
        assert_eq!(state.session.export_receipts.len(), 2);
        assert!(state.queue.pending_actions().is_empty());
        assert_eq!(
            state.queue.history().last().unwrap().status,
            ActionStatus::Rejected
        );
    }
}

#[test]
fn queued_extended_handoff_pins_source_version_and_duration_across_newer_takes() {
    for (selected_kind, selected_duration, boundary, later_kind) in [
        (
            GeneratedRecording::ExtendedV3(LiveRecordingDuration::EightBars),
            Some(LiveRecordingDuration::EightBars),
            DawSessionExportBoundary::LiveMasterDawprojectV3,
            GeneratedRecording::CanonicalV4,
        ),
        (
            GeneratedRecording::ExtendedV3(LiveRecordingDuration::SixteenBars),
            Some(LiveRecordingDuration::SixteenBars),
            DawSessionExportBoundary::LiveMasterDawprojectV3,
            GeneratedRecording::ExtendedV3(LiveRecordingDuration::EightBars),
        ),
        (
            GeneratedRecording::CanonicalV4,
            None,
            DawSessionExportBoundary::LiveMasterDawprojectV2,
            GeneratedRecording::ExtendedV3(LiveRecordingDuration::SixteenBars),
        ),
    ] {
        let temp = tempdir().unwrap();
        let selected_root = temp.path().join("selected");
        let later_root = temp.path().join("later");
        fs::create_dir(&selected_root).unwrap();
        fs::create_dir(&later_root).unwrap();
        let (mut state, _) =
            generated_recording(temp.path(), GeneratedRecording::LegacyV2, 120.0, 1_000, 0);
        let (selected, _) = generated_recording(&selected_root, selected_kind, 137.25, 1_000, 100);
        let source = selected.session.export_receipts[0].clone();
        append_generated_recording(&mut state, selected);
        let destination = temp.path().join("pinned.dawproject");
        assert!(matches!(
            state.queue_live_master_dawproject_export(
                3_000,
                Some(destination.to_string_lossy().into_owned()),
            ),
            DawSessionExportQueueResult::Enqueued { .. }
        ));
        assert!(matches!(
            &state.queue.pending_actions()[0].params,
            ActionParams::DawSessionExport {
                boundary: selected_boundary, duration, receipt_id: Some(id), ..
            } if *selected_boundary == boundary && *duration == selected_duration
                && id == source.receipt_id.as_str()
        ));
        let (later, _) = generated_recording(&later_root, later_kind, 129.75, 1_000, 200);
        append_generated_recording(&mut state, later);
        let receipt = state
            .commit_live_master_dawproject_export(&destination, 3_100)
            .unwrap();
        assert!(
            receipt.live_master_dawproject_action_contract_matches(boundary, selected_duration)
        );
        let proof = embedded_proof(&destination);
        assert_eq!(proof.source_receipt_id, source.receipt_id);
        assert_eq!(proof.source_wav_sha256, source.export_hash);
        assert_eq!(proof.duration, selected_duration);
    }
}
