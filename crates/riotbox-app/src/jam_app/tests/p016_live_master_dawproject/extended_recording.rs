//! Generated long-window handoff evidence, not DAW-host or listening qualification.

use super::canonical_recording::{GeneratedRecording, embedded_proof, generated_recording};
use crate::jam_app::{
    JamAppState, JamFileSet, LiveMasterRecordingProof, daw_export_operator_readiness_report,
    daw_export_operator_report::{
        DawExportOperatorReadinessStatus, DawExportReadinessBlocker, DawExportReleaseBlocker,
    },
    daw_export_proof_gates::DawExportProofGateStatus,
    product_export::DawSessionExportSurfaceBlocker,
    tests::dawproject_xml_schema::assert_dawproject_xml_documents_conform,
};
use dawproject::{DawprojectReader, prelude::project::LanesTypeContent};
use riotbox_core::{
    action::{ActionParams, ActionStatus, DawSessionExportBoundary, LiveRecordingDuration},
    persistence::save_session_json,
    session::ExportArtifactRole,
};
use sha2::{Digest, Sha256};
use std::{fs, io::Read};
use tempfile::tempdir;

#[test]
fn extended_recordings_export_distinct_v3_handoffs_for_both_durations() {
    for (duration, rate, bpm, beats, bars) in [
        (LiveRecordingDuration::EightBars, 44_100, 120.1_f32, 32, 8),
        (
            LiveRecordingDuration::SixteenBars,
            48_000,
            90.049_f32,
            64,
            16,
        ),
    ] {
        let temp = tempdir().unwrap();
        let (state, recording) = generated_recording(
            temp.path(),
            GeneratedRecording::ExtendedV3(duration),
            bpm,
            rate,
            100,
        );
        let source = state.session.export_receipts[0].clone();
        assert!(source.live_recording_runtime_master_ready());
        let source_proof: LiveMasterRecordingProof =
            serde_json::from_slice(&fs::read(&source.proof_path).unwrap()).unwrap();
        let source_bytes = fs::read(&recording).unwrap();
        let session_path = temp.path().join("session.json");
        save_session_json(&session_path, &state.session).unwrap();
        let mut restored = JamAppState::from_json_files_for_export_metadata(&session_path).unwrap();
        assert!(restored.source_graph.is_none());
        assert!(restored.source_audio_cache.is_none());
        assert!(restored.capture_audio_cache.is_empty());
        restored.session.runtime_state.source_timing.confirmed_bpm = Some(173.0);
        let destination = temp.path().join("extended.dawproject");
        let receipt = restored
            .commit_and_save_live_master_dawproject_export(&destination, 3_000)
            .expect("fully ready long recording has its own V3 DAW handoff");
        assert_eq!(
            receipt.export_boundary.as_proof_str(),
            "daw_session.live_master_dawproject_v3"
        );
        assert_eq!(receipt.pack_id, "live-master-dawproject-v3");
        assert_eq!(receipt.live_recording_duration, Some(duration));
        assert!(receipt.live_master_dawproject_archive_ready());
        assert!(matches!(
            &restored.session.action_log.actions.last().unwrap().params,
            ActionParams::DawSessionExport {
                boundary: DawSessionExportBoundary::LiveMasterDawprojectV3,
                duration: Some(selected), receipt_id: Some(id), ..
            } if *selected == duration && id == source.receipt_id.as_str()
        ));
        assert_eq!(receipt.artifact_set.len(), 4);
        let [placement] = receipt.arrangement_placement_refs.as_slice() else {
            panic!("one recorded scene placement")
        };
        assert_eq!((placement.start_bar, placement.end_bar), (1, bars));
        assert_eq!(
            (placement.start_beat, placement.end_beat),
            (0, u64::from(beats))
        );
        let tempo = receipt.daw_tempo_map_ref.as_ref().unwrap();
        assert_eq!((tempo.start_beat, tempo.end_beat), (0, u64::from(beats)));
        assert_eq!(
            u64::from(tempo.bpm_micros),
            source_proof.confirmed_bpm_micros
        );
        let proof = embedded_proof(&destination);
        assert_eq!(proof.schema, "riotbox.live_master_dawproject.v3");
        assert_eq!(
            proof.source_boundary,
            "live_recording.runtime_master_bar_window_v3"
        );
        assert_eq!(proof.source_receipt_id, source.receipt_id);
        assert_eq!(proof.source_action_id, source.created_by_action);
        assert_eq!(proof.duration, Some(duration));
        assert_eq!(proof.duration_beats, beats);
        assert_eq!(proof.start_beat, 0);
        assert_eq!(proof.beats_per_bar, 4);
        assert_eq!(proof.frame_count, source_proof.frame_count);
        assert_eq!(proof.source_wav_sha256, source.export_hash);
        assert_eq!(proof.source_proof_sha256, source.normalized_manifest_hash);
        assert_eq!(
            proof.sample_payload_sha256,
            source_proof.sample_payload_sha256
        );
        assert_dawproject_xml_documents_conform(&destination);
        let mut reader = DawprojectReader::open(&destination).unwrap();
        let mut members = reader.file_names().collect::<Vec<_>>();
        members.sort_unstable();
        assert_eq!(
            members,
            [
                "audio/live_master.wav",
                "metadata.xml",
                "project.xml",
                "riotbox-proof.json"
            ]
        );
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
        assert_eq!(clips.clip.len(), 1);
        assert_eq!(clips.clip[0].time, 0.0);
        assert_eq!(clips.clip[0].duration, Some(f64::from(beats)));
        assert_eq!(
            clips.clip[0].name,
            Some(format!("Live Master — {bars} bars"))
        );
        let mut embedded = Vec::new();
        reader
            .by_name("audio/live_master.wav")
            .unwrap()
            .read_to_end(&mut embedded)
            .unwrap();
        assert_eq!(embedded, source_bytes);
        assert_eq!(
            format!("{:x}", Sha256::digest(&embedded)),
            proof.embedded_audio_sha256
        );
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

        // Metadata-only restore remains valid even when every generated audio/proof
        // artifact is withheld; it must not recreate the external side effect.
        drop(reader);
        let archive_bytes = fs::read(&destination).unwrap();
        let withheld_archive = temp.path().join("withheld.dawproject");
        fs::rename(&recording, temp.path().join("withheld.wav")).unwrap();
        fs::rename(&source.proof_path, temp.path().join("withheld-proof.json")).unwrap();
        fs::rename(&destination, &withheld_archive).unwrap();
        let reopened = JamAppState::from_json_files_for_export_metadata(&session_path).unwrap();
        assert_eq!(reopened.session.export_receipts.last(), Some(&receipt));
        assert_eq!(reopened.session.action_log, restored.session.action_log);
        assert!(!recording.exists());
        assert!(!destination.exists());
        assert_eq!(fs::read(withheld_archive).unwrap(), archive_bytes);
    }
}

#[test]
fn extended_handoffs_preserve_no_clobber_and_rollback_on_failed_session_save() {
    for duration in [
        LiveRecordingDuration::EightBars,
        LiveRecordingDuration::SixteenBars,
    ] {
        let temp = tempdir().unwrap();
        let (mut state, recording) = generated_recording(
            temp.path(),
            GeneratedRecording::ExtendedV3(duration),
            137.25,
            1_000,
            100,
        );
        let source_bytes = fs::read(&recording).unwrap();
        let source_proof_path = state.session.export_receipts[0].proof_path.clone();
        let source_proof_bytes = fs::read(&source_proof_path).unwrap();
        let collision = temp.path().join("collision.dawproject");
        fs::write(&collision, b"generated collision control").unwrap();
        assert!(
            state
                .commit_live_master_dawproject_export(&collision, 3_000)
                .is_err()
        );
        assert_eq!(
            fs::read(&collision).unwrap(),
            b"generated collision control"
        );
        let session_before = state.session.clone();
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
        assert_eq!(state.session, session_before);
        assert!(state.queue.pending_actions().is_empty());
        let rejected = state.queue.history().last().unwrap();
        assert_eq!(rejected.status, ActionStatus::Rejected);
        assert!(matches!(
            rejected.params,
            ActionParams::DawSessionExport {
                boundary: DawSessionExportBoundary::LiveMasterDawprojectV3,
                duration: Some(selected), ..
            } if selected == duration
        ));
        assert_eq!(fs::read(&recording).unwrap(), source_bytes);
        assert_eq!(fs::read(&source_proof_path).unwrap(), source_proof_bytes);
    }
}

#[test]
fn extended_archive_readiness_projections_reject_missing_duration_and_wrong_frame_count() {
    let temp = tempdir().unwrap();
    let (mut ready, _) = generated_recording(
        temp.path(),
        GeneratedRecording::ExtendedV3(LiveRecordingDuration::EightBars),
        137.25,
        1_000,
        100,
    );
    let destination = temp.path().join("extended.dawproject");
    ready
        .commit_live_master_dawproject_export(&destination, 3_000)
        .unwrap();
    for missing_duration in [true, false] {
        let mut state = ready.clone();
        let receipt = state.session.export_receipts.last_mut().unwrap();
        if missing_duration {
            receipt.live_recording_duration = None;
        } else {
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
        assert!(!receipt.live_master_dawproject_archive_ready());
        let report = daw_export_operator_readiness_report(&state.session, Some(temp.path()));
        assert_eq!(
            report.proof_gates.writer_proof.status,
            DawExportProofGateStatus::Failed
        );
        assert_eq!(report.status, DawExportOperatorReadinessStatus::Blocked);
        assert!(
            report
                .release_blockers
                .contains(&DawExportReleaseBlocker::DawWriterMissing)
        );
        assert!(
            report
                .readiness_blockers
                .contains(&DawExportReadinessBlocker::MissingArtifactIdentity)
        );
        assert!(
            state
                .daw_session_export_surface_gate()
                .blockers
                .contains(&DawSessionExportSurfaceBlocker::DawWriterMissing)
        );
    }
}
