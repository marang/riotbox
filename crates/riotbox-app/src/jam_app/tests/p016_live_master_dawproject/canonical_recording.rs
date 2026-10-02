//! Source-free canonical two-bar handoff proof; no DAW-host or listening claim.

use super::{attach_generated_recording_lineage, prepared_live_master_state};
use crate::jam_app::{
    JamAppState, JamFileSet, LiveMasterDawprojectProof, LiveMasterRecordingProof,
    LiveMasterRecordingQueueResult, daw_export_operator_readiness_report,
    daw_export_operator_report::{DawExportOperatorReadinessStatus, DawExportReleaseBlocker},
    product_export::DawSessionExportQueueResult,
    tests::{
        dawproject_xml_schema::assert_dawproject_xml_documents_conform,
        p016_live_master_recording::{
            live_master_recording_state, live_master_test_health, live_master_test_outcome,
            live_master_test_output,
        },
    },
};
use dawproject::{DawprojectReader, prelude::project::LanesTypeContent};
use riotbox_core::{
    action::{
        ActionCommand, ActionParams, ActionStatus, DawSessionExportBoundary, LiveRecordingDuration,
    },
    export_readiness::ProductExportBoundary,
    ids::ActionId,
    persistence::save_session_json,
    session::{ExportArtifactRole, SessionFile},
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};
use tempfile::tempdir;

#[derive(Clone, Copy)]
pub(super) enum GeneratedRecording {
    LegacyV2,
    CanonicalV4,
    ExtendedV3(LiveRecordingDuration),
}

pub(super) fn generated_recording(
    root: &Path,
    kind: GeneratedRecording,
    bpm: f32,
    rate: u32,
    after_action: u64,
) -> (JamAppState, PathBuf) {
    let recording = root.join("generated-master.wav");
    let mut output = live_master_test_output();
    output.sample_rate = rate;
    let mut state = live_master_recording_state();
    state.session.runtime_state.source_timing.confirmed_bpm = Some(bpm);
    state
        .queue
        .reserve_action_ids_after(Some(ActionId(after_action)));
    let queued = match kind {
        GeneratedRecording::LegacyV2 => {
            state.queue_legacy_two_bar_recording_fixture(1_000, &output, &recording)
        }
        GeneratedRecording::CanonicalV4 => {
            state.queue_live_master_recording(1_000, &output, &recording)
        }
        GeneratedRecording::ExtendedV3(duration) => {
            state.queue_live_master_recording_with_duration(1_000, &output, &recording, duration)
        }
    };
    let LiveMasterRecordingQueueResult::Enqueued(plan) = queued else {
        panic!("queue generated recording")
    };
    state
        .commit_live_master_recording(
            &plan,
            &live_master_test_outcome(&plan),
            &live_master_test_health(&output),
            2_000,
        )
        .expect("commit generated recording");
    attach_generated_recording_lineage(&mut state);
    (state, recording)
}

pub(super) fn append_generated_recording(state: &mut JamAppState, other: JamAppState) {
    assert_eq!(state.session.session_id, other.session.session_id);
    assert_eq!(
        state.session.source_graph_refs,
        other.session.source_graph_refs
    );
    state.queue.reserve_action_ids_after(
        other
            .session
            .action_log
            .actions
            .iter()
            .map(|action| action.id)
            .max(),
    );
    state
        .session
        .action_log
        .actions
        .extend(other.session.action_log.actions);
    state
        .session
        .action_log
        .commit_records
        .extend(other.session.action_log.commit_records);
    state
        .session
        .export_receipts
        .extend(other.session.export_receipts);
}

pub(super) fn embedded_proof(destination: &Path) -> LiveMasterDawprojectProof {
    let mut reader = DawprojectReader::open(destination).expect("open generated archive");
    let mut bytes = Vec::new();
    reader
        .by_name("riotbox-proof.json")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    serde_json::from_slice(&bytes).expect("embedded typed proof")
}

#[test]
fn canonical_two_bar_recordings_restore_and_export_exact_v2_archives_at_fractional_tempos() {
    for bpm in [121.5_f32, 166.5, 90.016] {
        let temp = tempdir().unwrap();
        let (state, recording) = generated_recording(
            temp.path(),
            GeneratedRecording::CanonicalV4,
            bpm,
            48_000,
            100,
        );
        let source = state.session.export_receipts[0].clone();
        assert!(source.is_live_recording_runtime_master_bar_window_v4());
        assert!(source.live_recording_runtime_master_ready());
        assert_eq!(
            source.live_recording_duration,
            Some(LiveRecordingDuration::TwoBars)
        );
        let source_proof: LiveMasterRecordingProof =
            serde_json::from_slice(&fs::read(&source.proof_path).unwrap()).unwrap();
        assert_eq!(
            source_proof.schema,
            "riotbox.live_recording_runtime_master_bar_window.v4"
        );
        assert_eq!(source_proof.duration, Some(LiveRecordingDuration::TwoBars));
        if bpm == 121.5 {
            assert_eq!(source_proof.beat_span_per_frame_nanobeats, 42_187);
        } else if bpm == 166.5 {
            assert_eq!(source_proof.beat_span_per_frame_nanobeats, 57_812);
        } else {
            assert_eq!(source_proof.frame_count, 255_955);
        }
        let session_path = temp.path().join("generated-session.json");
        save_session_json(&session_path, &state.session).unwrap();
        let mut restored = JamAppState::from_json_files_for_export_metadata(&session_path).unwrap();
        assert!(restored.source_graph.is_none());
        assert!(restored.source_audio_cache.is_none());
        assert!(restored.capture_audio_cache.is_empty());
        restored.session.runtime_state.source_timing.confirmed_bpm = Some(173.0);
        let destination = temp.path().join("canonical.dawproject");
        let receipt = restored
            .commit_and_save_live_master_dawproject_export(&destination, 3_000)
            .unwrap();
        assert!(receipt.is_live_master_dawproject_v2());
        assert!(!receipt.is_live_master_dawproject_v1());
        assert!(
            serde_json::to_value(&receipt)
                .unwrap()
                .get("live_recording_duration")
                .is_none()
        );
        assert!(receipt.live_master_dawproject_archive_ready());
        assert_eq!(receipt.pack_id, "live-master-dawproject-v2");
        let action = restored.session.action_log.actions.last().unwrap();
        assert!(
            serde_json::to_value(action).unwrap()["params"]["DawSessionExport"]
                .get("duration")
                .is_none()
        );
        assert!(matches!(
            action.params,
            ActionParams::DawSessionExport {
                boundary: DawSessionExportBoundary::LiveMasterDawprojectV2,
                ..
            }
        ));
        let proof = embedded_proof(&destination);
        assert_eq!(proof.duration, None);
        assert_eq!(proof.schema, "riotbox.live_master_dawproject.v2");
        assert_eq!(
            proof.source_boundary,
            "live_recording.runtime_master_bar_window_v4"
        );
        assert_eq!(proof.source_receipt_id, source.receipt_id);
        assert_eq!(proof.source_wav_sha256, source.export_hash);
        assert_eq!(proof.source_proof_sha256, source.normalized_manifest_hash);
        assert_eq!(proof.frame_count, source_proof.frame_count);
        assert_eq!(proof.duration_beats, 8);
        assert_eq!(proof.start_beat, 0);
        assert_dawproject_xml_documents_conform(&destination);
        let mut reader = DawprojectReader::open(&destination).unwrap();
        let mut proof_bytes = Vec::new();
        reader
            .by_name("riotbox-proof.json")
            .unwrap()
            .read_to_end(&mut proof_bytes)
            .unwrap();
        let proof_json: serde_json::Value = serde_json::from_slice(&proof_bytes).unwrap();
        assert!(proof_json.get("duration").is_none());
        reader.read_dawproject().unwrap();
        let project = reader.build_dawproject().unwrap();
        assert_eq!(
            project
                .project
                .transport
                .as_ref()
                .unwrap()
                .tempo
                .as_ref()
                .unwrap()
                .value,
            Some(format!(
                "{:.6}",
                source_proof.confirmed_bpm_micros as f64 / 1_000_000.0
            ))
        );
        let lanes = project
            .project
            .arrangement
            .as_ref()
            .unwrap()
            .lanes
            .as_ref()
            .unwrap();
        let LanesTypeContent::Lanes(master) = &lanes.content[0] else {
            panic!("master lane")
        };
        let LanesTypeContent::Clips(clips) = &master.content[0] else {
            panic!("master clip")
        };
        assert_eq!(clips.clip[0].duration, Some(8.0));
        let mut embedded = Vec::new();
        reader
            .by_name("audio/live_master.wav")
            .unwrap()
            .read_to_end(&mut embedded)
            .unwrap();
        assert_eq!(embedded, fs::read(&recording).unwrap());
        let readiness = daw_export_operator_readiness_report(&restored.session, None);
        assert_eq!(
            readiness.status,
            DawExportOperatorReadinessStatus::DawprojectReady
        );
        assert!(
            readiness
                .release_blockers
                .contains(&DawExportReleaseBlocker::DawHostImportProofMissing)
        );
        assert!(
            readiness
                .release_blockers
                .contains(&DawExportReleaseBlocker::AudibleOutputProofMissing)
        );
        let archive_before = fs::read(&destination).unwrap();
        let reopened = JamAppState::from_json_files_for_export_metadata(&session_path).unwrap();
        assert_eq!(reopened.session.export_receipts.last(), Some(&receipt));
        let replay =
            riotbox_core::replay::build_committed_replay_plan(&reopened.session.action_log)
                .unwrap();
        assert!(
            replay
                .iter()
                .any(|entry| entry.action.id == receipt.created_by_action)
        );
        assert_eq!(fs::read(&destination).unwrap(), archive_before);
    }
}

#[test]
fn mixed_supported_versions_pin_source_id_and_handoff_version_before_later_takes() {
    for newer_canonical in [true, false] {
        let temp = tempdir().unwrap();
        let older_root = temp.path().join("older");
        let selected_root = temp.path().join("selected");
        let later_root = temp.path().join("later");
        for root in [&older_root, &selected_root, &later_root] {
            fs::create_dir(root).unwrap();
        }
        let (older_kind, selected_kind, expected_boundary) = if newer_canonical {
            (
                GeneratedRecording::LegacyV2,
                GeneratedRecording::CanonicalV4,
                DawSessionExportBoundary::LiveMasterDawprojectV2,
            )
        } else {
            (
                GeneratedRecording::CanonicalV4,
                GeneratedRecording::LegacyV2,
                DawSessionExportBoundary::LiveMasterDawprojectV1,
            )
        };
        let (mut state, _) = generated_recording(&older_root, older_kind, 120.0, 1_000, 0);
        let (selected, _) = generated_recording(&selected_root, selected_kind, 120.0, 1_000, 100);
        let selected_id = selected.session.export_receipts[0].receipt_id.clone();
        append_generated_recording(&mut state, selected);
        let destination = temp.path().join("pinned.dawproject");
        assert!(matches!(
            state.queue_live_master_dawproject_export(
                3_000,
                Some(destination.to_string_lossy().into_owned())
            ),
            DawSessionExportQueueResult::Enqueued { .. }
        ));
        assert!(
            matches!(&state.queue.pending_actions()[0].params, ActionParams::DawSessionExport {
            boundary, receipt_id: Some(id), ..
        } if *boundary == expected_boundary && id == selected_id.as_str())
        );
        let (later, _) = generated_recording(&later_root, older_kind, 120.0, 1_000, 200);
        append_generated_recording(&mut state, later);
        let receipt = state
            .commit_live_master_dawproject_export(&destination, 3_100)
            .unwrap();
        assert_eq!(receipt.is_live_master_dawproject_v2(), newer_canonical);
        assert_eq!(receipt.is_live_master_dawproject_v1(), !newer_canonical);
        assert_eq!(embedded_proof(&destination).source_receipt_id, selected_id);
    }
}

#[test]
fn invalid_latest_v4_does_not_fall_back_and_valid_v3_uses_only_its_new_handoff() {
    for invalid_newer in [true, false] {
        let temp = tempdir().unwrap();
        let (mut state, _) = prepared_live_master_state(temp.path());
        let newer_root = temp.path().join("newer");
        fs::create_dir(&newer_root).unwrap();
        let kind = if invalid_newer {
            GeneratedRecording::CanonicalV4
        } else {
            GeneratedRecording::ExtendedV3(LiveRecordingDuration::EightBars)
        };
        let (mut newer, _) = generated_recording(&newer_root, kind, 120.0, 1_000, 100);
        let newer_id = newer.session.export_receipts[0].receipt_id.clone();
        if invalid_newer {
            newer.session.export_receipts[0].export_hash = "invalid".into();
        }
        append_generated_recording(&mut state, newer);
        let destination = temp.path().join("selected.dawproject");
        let result = state.commit_live_master_dawproject_export(&destination, 3_000);
        if invalid_newer {
            assert!(result.is_err());
            assert!(!destination.exists());
        } else {
            let receipt = result.unwrap();
            assert!(receipt.is_live_master_dawproject_v3());
            assert!(!receipt.is_live_master_dawproject_v1());
            assert!(!receipt.is_live_master_dawproject_v2());
            assert_eq!(embedded_proof(&destination).source_receipt_id, newer_id);
        }
    }
}

#[test]
fn queued_v4_handoff_rejects_changed_source_boundary_before_archive_publication() {
    let temp = tempdir().unwrap();
    let (mut state, _) = generated_recording(
        temp.path(),
        GeneratedRecording::CanonicalV4,
        120.0,
        1_000,
        100,
    );
    let destination = temp.path().join("changed.dawproject");
    assert!(matches!(
        state.queue_live_master_dawproject_export(
            3_000,
            Some(destination.to_string_lossy().into_owned())
        ),
        DawSessionExportQueueResult::Enqueued { .. }
    ));
    state.session.export_receipts[0].export_boundary =
        ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV2;
    let error = state
        .commit_live_master_dawproject_export(&destination, 3_100)
        .unwrap_err();
    assert!(error.to_string().contains("queued handoff version"));
    assert!(!destination.exists());
    assert_eq!(
        state.queue.history().last().unwrap().status,
        ActionStatus::Rejected
    );
}

#[test]
fn canonical_source_proof_schema_and_duration_cannot_be_relabelled_even_with_updated_hashes() {
    for mutation in ["legacy-schema", "missing-duration", "long-duration"] {
        let temp = tempdir().unwrap();
        let (mut state, _) = generated_recording(
            temp.path(),
            GeneratedRecording::CanonicalV4,
            120.0,
            1_000,
            100,
        );
        let source = &mut state.session.export_receipts[0];
        let mut proof: LiveMasterRecordingProof =
            serde_json::from_slice(&fs::read(&source.proof_path).unwrap()).unwrap();
        match mutation {
            "legacy-schema" => {
                proof.schema = "riotbox.live_recording_runtime_master_bar_window.v2".into()
            }
            "missing-duration" => proof.duration = None,
            "long-duration" => proof.duration = Some(LiveRecordingDuration::EightBars),
            _ => unreachable!(),
        }
        let bytes = serde_json::to_vec_pretty(&proof).unwrap();
        let sha = format!("{:x}", Sha256::digest(&bytes));
        fs::write(&source.proof_path, bytes).unwrap();
        source.normalized_manifest_hash = sha.clone();
        source
            .artifact_set
            .iter_mut()
            .filter(|artifact| artifact.role == ExportArtifactRole::ProductExportProof)
            .for_each(|artifact| artifact.sha256 = sha.clone());
        let destination = temp.path().join("tampered.dawproject");
        assert!(
            state
                .commit_live_master_dawproject_export(&destination, 3_000)
                .is_err(),
            "{mutation}"
        );
        assert!(!destination.exists());
        assert_eq!(state.session.export_receipts.len(), 1);
    }
}

#[test]
fn canonical_daw_export_preserves_no_clobber_and_rolls_back_failed_session_save() {
    let temp = tempdir().unwrap();
    let (mut state, _) = generated_recording(
        temp.path(),
        GeneratedRecording::CanonicalV4,
        120.0,
        1_000,
        100,
    );
    let collision = temp.path().join("owned.dawproject");
    fs::write(&collision, b"user-owned").unwrap();
    assert!(
        state
            .commit_live_master_dawproject_export(&collision, 3_000)
            .is_err()
    );
    assert_eq!(fs::read(&collision).unwrap(), b"user-owned");
    let before: SessionFile = state.session.clone();
    state.files = Some(JamFileSet {
        session_path: temp.path().to_owned(),
        source_graph_path: None,
    });
    let destination = temp.path().join("rollback.dawproject");
    assert!(
        state
            .commit_and_save_live_master_dawproject_export(&destination, 3_100)
            .is_err()
    );
    assert!(!destination.exists());
    assert_eq!(state.session, before);
    assert_eq!(
        state.queue.history().last().unwrap().command,
        ActionCommand::ExportDawSession
    );
    assert_eq!(
        state.queue.history().last().unwrap().status,
        ActionStatus::Rejected
    );
}
