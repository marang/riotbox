use std::{fs, io::Read as _, os::unix::fs::symlink, path::PathBuf};

use dawproject::{DawprojectReader, prelude::project::LanesTypeContent};
use riotbox_core::{
    action::{ActionCommand, ActionParams, ActionStatus},
    export_readiness::{ExportScope, ProductExportBoundary, ProductExportRole},
    ids::{CaptureId, SourceId},
    persistence::{load_session_json, save_session_json},
    session::{
        ExportArtifactRole, ExportArtifactSourceGraphRef, ExportArtifactTimingGridRef,
        GraphStorageMode, SourceGraphRef, SourceTimingGridConfirmationState,
    },
    source_graph::{GraphProvenance, SourceGraphVersion},
};
use sha2::{Digest, Sha256};
use tempfile::tempdir;

use super::{
    JamAppState, JamFileSet, LiveMasterRecordingProof, LiveMasterRecordingQueueResult,
    SessionHydrationPolicy,
    p016_live_master_recording::{
        live_master_recording_state, live_master_test_health, live_master_test_outcome,
        live_master_test_output,
    },
};
use crate::jam_app::{
    daw_export_operator_readiness_report,
    daw_export_operator_report::{DawExportOperatorReadinessStatus, DawExportReleaseBlocker},
    daw_export_proof_gates::DawExportProofGateStatus,
    live_master_dawproject::LiveMasterDawprojectProof,
};

fn prepared_live_master_state(root: &std::path::Path) -> (JamAppState, PathBuf) {
    let recording = root.join("recorded-live-master.wav");
    let output = live_master_test_output();
    let mut state = live_master_recording_state();
    let plan = match state.queue_live_master_recording(1_000, &output, &recording) {
        LiveMasterRecordingQueueResult::Enqueued(plan) => plan,
        other => panic!("expected live master record plan, got {other:?}"),
    };
    let health = live_master_test_health(&output);
    state.set_audio_health(health.clone());
    state
        .commit_live_master_recording(&plan, &live_master_test_outcome(&plan), &health, 2_000)
        .expect("commit synthetic live master");

    let source_id = SourceId::from("synthetic-source");
    let graph_ref = ExportArtifactSourceGraphRef {
        source_id: source_id.clone(),
        graph_version: SourceGraphVersion::V1,
        graph_hash: format!("sha256:{}", "a".repeat(64)),
    };
    let timing_ref = ExportArtifactTimingGridRef {
        source_id: source_id.clone(),
        hypothesis_id: Some("synthetic-grid".into()),
        confirmed_by_action: riotbox_core::ids::ActionId(0),
        confirmed_at: 900,
    };
    state.session.source_graph_refs = vec![SourceGraphRef {
        source_id,
        graph_version: SourceGraphVersion::V1,
        graph_hash: graph_ref.graph_hash.clone(),
        storage_mode: GraphStorageMode::External,
        embedded_graph: None,
        external_path: Some("intentionally-missing-graph.json".into()),
        provenance: GraphProvenance {
            sidecar_version: "synthetic".into(),
            provider_set: Vec::new(),
            generated_at: "2026-09-08T00:00:00Z".into(),
            source_hash: "synthetic".into(),
            analysis_seed: 0,
            run_notes: None,
        },
    }];
    state.session.runtime_state.source_timing.confirmed_grid =
        Some(SourceTimingGridConfirmationState {
            source_id: graph_ref.source_id.clone(),
            hypothesis_id: timing_ref.hypothesis_id.clone(),
            confirmed_by_action: timing_ref.confirmed_by_action,
            confirmed_at: timing_ref.confirmed_at,
        });

    let receipt = state
        .session
        .export_receipts
        .last_mut()
        .expect("recording receipt");
    let proof_path = PathBuf::from(&receipt.proof_path);
    let mut proof: LiveMasterRecordingProof =
        serde_json::from_slice(&fs::read(&proof_path).expect("synthetic recording proof"))
            .expect("parse recording proof");
    proof.source_graph_ref = Some(graph_ref.clone());
    proof.timing_grid_ref = Some(timing_ref.clone());
    proof.source_capture_refs = Vec::<CaptureId>::new();
    proof.lineage_capture_refs = Vec::<CaptureId>::new();
    let proof_bytes = serde_json::to_vec_pretty(&proof).expect("serialize amended proof");
    let proof_sha = format!("{:x}", Sha256::digest(&proof_bytes));
    fs::write(&proof_path, proof_bytes).expect("write synthetic amended proof");
    receipt.normalized_manifest_hash = proof_sha.clone();
    for artifact in &mut receipt.artifact_set {
        artifact.source_graph_ref = Some(graph_ref.clone());
        artifact.timing_grid_ref = Some(timing_ref.clone());
        if artifact.role == ExportArtifactRole::ProductExportProof {
            artifact.sha256 = proof_sha.clone();
        }
    }
    (state, recording)
}

#[test]
fn live_master_dawproject_commits_typed_four_member_archive_and_preserves_recorded_tempo() {
    let temp = tempdir().expect("tempdir");
    let (mut state, recording) = prepared_live_master_state(temp.path());
    let destination = temp.path().join("live-master.dawproject");
    state.session.runtime_state.source_timing.confirmed_bpm = Some(173.0);

    let receipt = state
        .commit_live_master_dawproject_export(&destination, 3_000)
        .expect("export synthetic live master");

    super::dawproject_xml_schema::assert_dawproject_xml_documents_conform(&destination);

    assert!(receipt.is_live_master_dawproject_v1());
    assert_eq!(receipt.export_scope, ExportScope::DawSession);
    assert_eq!(
        receipt.export_boundary,
        ProductExportBoundary::DawSessionLiveMasterDawprojectV1
    );
    assert_eq!(receipt.export_role, ProductExportRole::ArrangementManifest);
    assert_eq!(receipt.artifact_set.len(), 4);
    assert_eq!(
        receipt
            .artifact_set
            .iter()
            .map(|entry| entry.role)
            .collect::<Vec<_>>(),
        vec![
            ExportArtifactRole::DawProjectFile,
            ExportArtifactRole::ExportManifest,
            ExportArtifactRole::LiveRecordingCapture,
            ExportArtifactRole::DawProjectProof,
        ]
    );
    let action = state
        .session
        .action_log
        .actions
        .last()
        .expect("export action");
    assert_eq!(action.command, ActionCommand::ExportDawSession);
    assert_eq!(action.status, ActionStatus::Committed);
    assert!(
        matches!(&action.params, ActionParams::DawSessionExport { receipt_id: Some(id), .. }
        if id == state.session.export_receipts[0].receipt_id.as_str())
    );

    let mut reader = DawprojectReader::open(&destination).expect("open archive");
    reader.read_dawproject().expect("parse archive");
    let project = reader.build_dawproject().expect("typed project");
    assert_eq!(
        project
            .project
            .transport
            .as_ref()
            .and_then(|transport| transport.tempo.as_ref())
            .and_then(|tempo| tempo.value.as_deref()),
        Some("120.000000"),
        "DAW placement uses the recording tempo, not the later Session tempo"
    );
    let mut embedded = Vec::new();
    reader
        .by_name("audio/live_master.wav")
        .expect("embedded master")
        .read_to_end(&mut embedded)
        .expect("read embedded master");
    assert_eq!(embedded, fs::read(recording).expect("source recording"));
    let readiness = daw_export_operator_readiness_report(&state.session, None);
    assert_eq!(
        readiness.status,
        DawExportOperatorReadinessStatus::DawprojectReady
    );
    assert_eq!(
        readiness.proof_gates.writer_proof.status,
        DawExportProofGateStatus::Passed
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
}

#[test]
fn live_master_dawproject_rejects_ambiguous_receipt_ids_before_and_after_queue() {
    for duplicate_after_queue in [false, true] {
        let temp = tempdir().expect("tempdir");
        let (mut state, _) = prepared_live_master_state(temp.path());
        let destination = temp.path().join("ambiguous.dawproject");
        if duplicate_after_queue {
            assert!(matches!(
                state.queue_live_master_dawproject_export(
                    3_000,
                    Some(destination.to_string_lossy().into_owned())
                ),
                super::DawSessionExportQueueResult::Enqueued { .. }
            ));
        }
        let mut duplicate = state.session.export_receipts[0].clone();
        duplicate.created_at += 1;
        state.session.export_receipts.push(duplicate);
        assert!(
            state
                .commit_live_master_dawproject_export(&destination, 3_100)
                .is_err()
        );
        assert!(!destination.exists());
        assert_eq!(state.session.export_receipts.len(), 2);
        assert_eq!(
            state.queue.history().last().map(|action| action.status),
            Some(ActionStatus::Rejected)
        );
    }
}

#[test]
fn live_master_dawproject_readiness_rejects_incomplete_or_contradictory_archive_receipts() {
    let temp = tempdir().expect("tempdir");
    let (mut state, _) = prepared_live_master_state(temp.path());
    let destination = temp.path().join("validated.dawproject");
    state
        .commit_live_master_dawproject_export(&destination, 3_000)
        .expect("archive");
    let original = state.session.clone();
    assert!(
        original
            .export_receipts
            .last()
            .unwrap()
            .live_master_dawproject_archive_ready()
    );
    for mutation in 0..12 {
        let mut session = original.clone();
        let receipt = session.export_receipts.last_mut().unwrap();
        match mutation {
            0..=3 => {
                receipt.artifact_set.remove(mutation);
            }
            4 => receipt.artifact_set.clear(),
            5 => receipt.artifact_set[1] = receipt.artifact_set[0].clone(),
            6 => receipt.artifact_set[0].sha256 = "0".repeat(64),
            7 => {
                receipt.artifact_set[2].location =
                    riotbox_core::session::ExportArtifactLocation::Uri {
                        uri: format!("{}#wrong.wav", destination.display()),
                    }
            }
            8 => {
                receipt.artifact_set[2].media_type =
                    riotbox_core::session::ExportArtifactMediaType::Json
            }
            9 => receipt.qa_gates.push(receipt.qa_gates[0].clone()),
            10 => receipt.qa_gates.retain(|gate| {
                gate.gate_id != riotbox_core::session::DAWPROJECT_XML_DOCUMENT_QA_GATE_ID
            }),
            11 => receipt.qa_gates.push(receipt.qa_gates[1].clone()),
            _ => unreachable!(),
        }
        assert!(
            !receipt.live_master_dawproject_archive_ready(),
            "case {mutation}"
        );
        let report = daw_export_operator_readiness_report(&session, Some(temp.path()));
        assert_eq!(
            report.status,
            DawExportOperatorReadinessStatus::Blocked,
            "case {mutation}"
        );
        assert_eq!(
            report.proof_gates.writer_proof.status,
            DawExportProofGateStatus::Failed
        );
        assert!(
            report
                .release_blockers
                .contains(&DawExportReleaseBlocker::DawWriterMissing)
        );
    }
}

#[test]
fn live_master_dawproject_pins_the_selected_receipt_and_never_falls_back_from_newer_invalid_v2() {
    let temp = tempdir().expect("tempdir");
    let (mut state, _) = prepared_live_master_state(temp.path());
    let destination = temp.path().join("pinned.dawproject");
    let queued = state.queue_live_master_dawproject_export(
        3_000,
        Some(destination.to_string_lossy().into_owned()),
    );
    assert!(matches!(
        queued,
        super::DawSessionExportQueueResult::Enqueued { .. }
    ));
    let pending = state.queue.pending_actions();
    let queued_action = pending.last().expect("queued action");
    assert!(
        matches!(&queued_action.params, ActionParams::DawSessionExport { receipt_id: Some(id), .. }
        if id == state.session.export_receipts[0].receipt_id.as_str())
    );

    let mut newer = state.session.export_receipts[0].clone();
    newer.receipt_id = "export-receipt-newer-invalid".into();
    newer.export_hash = "sha256:invalid".into();
    state.session.export_receipts.push(newer);
    state
        .commit_live_master_dawproject_export(&destination, 3_100)
        .expect("pinned earlier valid receipt remains the source");

    let no_fallback_root = temp.path().join("no-fallback");
    fs::create_dir(&no_fallback_root).expect("no-fallback root");
    let (mut no_fallback, _) = prepared_live_master_state(&no_fallback_root);
    let mut newer_invalid = no_fallback.session.export_receipts[0].clone();
    newer_invalid.receipt_id = "export-receipt-newer-invalid".into();
    newer_invalid.export_hash = "sha256:invalid".into();
    no_fallback.session.export_receipts.push(newer_invalid);
    let blocked = temp.path().join("newest-invalid.dawproject");
    let result = no_fallback
        .queue_live_master_dawproject_export(3_200, Some(blocked.to_string_lossy().into_owned()));
    assert!(matches!(
        result,
        super::DawSessionExportQueueResult::Rejected { .. }
    ));
    assert!(!blocked.exists());
}

#[test]
fn live_master_dawproject_never_skips_newer_v2_boundary_with_corrupt_scope_or_role() {
    let temp = tempdir().expect("tempdir");
    for corruption in ["scope", "role"] {
        let root = temp.path().join(corruption);
        fs::create_dir(&root).expect("case root");
        let (mut state, _) = prepared_live_master_state(&root);
        let mut newer = state.session.export_receipts[0].clone();
        newer.receipt_id = format!("export-receipt-newer-{corruption}").into();
        match corruption {
            "scope" => newer.export_scope = ExportScope::ProductMix,
            "role" => newer.export_role = ProductExportRole::ArrangementManifest,
            _ => unreachable!(),
        }
        state.session.export_receipts.push(newer);
        let destination = root.join("must-not-fallback.dawproject");
        assert!(
            matches!(
                state.queue_live_master_dawproject_export(
                    3_300,
                    Some(destination.to_string_lossy().into_owned())
                ),
                super::DawSessionExportQueueResult::Rejected { .. }
            ),
            "{corruption}"
        );
        assert!(!destination.exists(), "{corruption}");
    }
}

#[test]
fn live_master_dawproject_rejects_hash_and_metadata_proof_drift_without_archive_or_receipt() {
    let temp = tempdir().expect("tempdir");
    for mutation in [
        "hash",
        "requested-start",
        "window",
        "anchor",
        "beat-span",
        "start-error",
        "duration-error",
        "health",
        "callback-gap",
        "unarmed",
        "clip-count",
        "scene",
        "lineage",
        "graph-source",
        "timing-source",
        "wav-symlink",
        "proof-symlink",
    ] {
        let case = temp.path().join(mutation);
        fs::create_dir(&case).expect("case dir");
        let (mut state, recording) = prepared_live_master_state(&case);
        let receipt = state
            .session
            .export_receipts
            .last_mut()
            .expect("source receipt");
        match mutation {
            "hash" => fs::write(&recording, b"not the recorded WAV").expect("drift source bytes"),
            "wav-symlink" => {
                let target = case.join("real-recorded-live-master.wav");
                fs::rename(&recording, &target).expect("move source WAV");
                symlink(&target, &recording).expect("synthetic WAV symlink");
            }
            "proof-symlink" => {
                let proof_path = PathBuf::from(&receipt.proof_path);
                let target = case.join("real-recording-proof.json");
                fs::rename(&proof_path, &target).expect("move source proof");
                symlink(&target, &proof_path).expect("synthetic proof symlink");
            }
            _ => {
                let mut proof: LiveMasterRecordingProof =
                    serde_json::from_slice(&fs::read(&receipt.proof_path).expect("proof"))
                        .expect("parse proof");
                match mutation {
                    "requested-start" => proof.requested_start_position_microbeats += 1,
                    "window" => proof.captured_end_position_microbeats += 1,
                    "anchor" => proof.bar_grid_anchor_position_microbeats += 1,
                    "beat-span" => proof.beat_span_per_frame_nanobeats += 1,
                    "start-error" => proof.start_alignment_error_frame_micros += 1,
                    "duration-error" => proof.duration_error_frame_micros += 1,
                    "health" => proof.stream_error_count = 1,
                    "callback-gap" => proof.callback_gap_over_threshold_count = 1,
                    "unarmed" => proof.armed_callback_count = 0,
                    "clip-count" => proof.clip_count = 1,
                    "scene" => proof.scene_id = "other-scene".into(),
                    "lineage" => proof.source_capture_refs = vec![CaptureId::from("missing")],
                    "graph-source" => {
                        proof
                            .source_graph_ref
                            .as_mut()
                            .expect("graph proof")
                            .source_id = SourceId::from("other-source")
                    }
                    "timing-source" => {
                        proof
                            .timing_grid_ref
                            .as_mut()
                            .expect("timing proof")
                            .source_id = SourceId::from("other-source")
                    }
                    _ => unreachable!(),
                }
                let bytes = serde_json::to_vec_pretty(&proof).expect("serialize mutated proof");
                let hash = format!("{:x}", Sha256::digest(&bytes));
                fs::write(&receipt.proof_path, bytes).expect("rewrite proof");
                receipt.normalized_manifest_hash = hash.clone();
                receipt
                    .artifact_set
                    .iter_mut()
                    .filter(|artifact| artifact.role == ExportArtifactRole::ProductExportProof)
                    .for_each(|artifact| artifact.sha256 = hash.clone());
            }
        }
        let destination = case.join("blocked.dawproject");
        assert!(
            state
                .commit_live_master_dawproject_export(&destination, 4_000)
                .is_err(),
            "{mutation}"
        );
        assert!(!destination.exists(), "{mutation}");
        assert_eq!(state.session.export_receipts.len(), 1, "{mutation}");
        assert_eq!(
            state.queue.history().last().map(|action| action.status),
            Some(ActionStatus::Rejected)
        );
    }
}

#[test]
fn live_master_dawproject_metadata_only_hydration_exports_without_graph_source_or_capture_reads() {
    let temp = tempdir().expect("tempdir");
    let (state, _) = prepared_live_master_state(temp.path());
    let session_path = temp.path().join("metadata-only-session.json");
    save_session_json(&session_path, &state.session).expect("save synthetic Session");
    let graph_refs_before = state.session.source_graph_refs.clone();
    let mut restored = JamAppState::from_json_files_for_export_metadata(&session_path)
        .expect("metadata-only restore");
    assert!(restored.source_graph.is_none());
    assert!(restored.source_audio_cache.is_none());
    assert!(restored.capture_audio_cache.is_empty());
    assert_eq!(restored.session.source_graph_refs, graph_refs_before);

    let destination = temp.path().join("metadata-only.dawproject");
    let receipt = restored
        .commit_live_master_dawproject_export(&destination, 6_000)
        .expect("metadata-only DAWproject export");
    assert!(receipt.is_live_master_dawproject_v1());
    assert_eq!(restored.session.source_graph_refs, graph_refs_before);
}

#[test]
fn live_master_dawproject_commit_and_save_survives_metadata_only_reload_without_rewriting_archive()
{
    let temp = tempdir().expect("tempdir");
    let (mut state, _) = prepared_live_master_state(temp.path());
    let session_path = temp.path().join("saved-session.json");
    save_session_json(&session_path, &state.session).expect("seed saved Session");
    state.files = Some(JamFileSet {
        session_path: session_path.clone(),
        source_graph_path: None,
    });
    state.session_hydration_policy = SessionHydrationPolicy::ExportMetadataOnly;
    let destination = temp.path().join("saved-live-master.dawproject");

    let committed = state
        .commit_and_save_live_master_dawproject_export(&destination, 7_000)
        .expect("commit and save live-master export");
    let archive_bytes = fs::read(&destination).expect("snapshot archive bytes");
    let archive_modified = fs::metadata(&destination)
        .expect("archive metadata")
        .modified()
        .expect("archive modification time");
    let saved = load_session_json(&session_path).expect("saved Session");
    assert_eq!(saved.export_receipts.last(), Some(&committed));
    assert!(
        saved
            .action_log
            .commit_records
            .iter()
            .any(|record| record.action_id == committed.created_by_action)
    );

    let restored = JamAppState::from_json_files_for_export_metadata(&session_path)
        .expect("metadata-only reload");
    assert_eq!(restored.session.export_receipts.last(), Some(&committed));
    let mut reader = DawprojectReader::open(&destination).expect("saved archive");
    reader.read_dawproject().expect("saved archive XML");
    let project = reader.build_dawproject().expect("saved typed project");
    assert_eq!(
        project
            .project
            .transport
            .as_ref()
            .and_then(|transport| transport.tempo.as_ref())
            .and_then(|tempo| tempo.value.as_deref()),
        Some("120.000000")
    );
    let arrangement = project.project.arrangement.as_ref().expect("arrangement");
    let lanes = arrangement.lanes.as_ref().expect("arrangement lanes");
    let LanesTypeContent::Lanes(live_master_lanes) = &lanes.content[0] else {
        panic!("first arrangement lane owns live master")
    };
    let LanesTypeContent::Clips(clips) = &live_master_lanes.content[0] else {
        panic!("live-master lane owns clips")
    };
    assert_eq!(clips.clip[0].time, 0.0);
    assert_eq!(clips.clip[0].duration, Some(8.0));
    let mut proof_bytes = Vec::new();
    reader
        .by_name("riotbox-proof.json")
        .expect("embedded proof")
        .read_to_end(&mut proof_bytes)
        .expect("read embedded proof");
    let proof: LiveMasterDawprojectProof =
        serde_json::from_slice(&proof_bytes).expect("typed embedded proof");
    assert_eq!(proof.source_receipt_id, saved.export_receipts[0].receipt_id);
    assert_eq!(proof.confirmed_bpm_micros, 120_000_000);
    assert_eq!(proof.start_beat, 0);
    assert_eq!(proof.duration_beats, 8);
    let plan = riotbox_core::replay::build_committed_replay_plan(&restored.session.action_log)
        .expect("committed replay plan");
    let export_entries = plan
        .iter()
        .filter(|entry| entry.action.command == ActionCommand::ExportDawSession)
        .collect::<Vec<_>>();
    assert_eq!(export_entries.len(), 1);
    assert_eq!(export_entries[0].action.id, committed.created_by_action);
    assert_eq!(
        export_entries[0].commit_record.action_id,
        committed.created_by_action
    );

    // Building/restoring the replay plan is metadata-only. External export
    // actions are intentionally not replay-executor-applied, so this test
    // asserts no archive rewrite rather than simulating one.
    assert_eq!(
        fs::read(&destination).expect("archive after reload"),
        archive_bytes
    );
    assert_eq!(
        fs::metadata(&destination)
            .expect("archive metadata after reload")
            .modified()
            .expect("archive modification time after reload"),
        archive_modified
    );
}

#[test]
fn live_master_dawproject_refuses_no_clobber_and_rolls_back_both_save_entry_points() {
    let temp = tempdir().expect("tempdir");
    let (mut collision, _) = prepared_live_master_state(temp.path());
    let existing = temp.path().join("owned.dawproject");
    fs::write(&existing, b"user-owned").expect("existing destination");
    assert!(
        collision
            .commit_live_master_dawproject_export(&existing, 5_000)
            .is_err()
    );
    assert_eq!(
        fs::read(&existing).expect("preserved destination"),
        b"user-owned"
    );
    assert_eq!(collision.session.export_receipts.len(), 1);
    assert_eq!(
        collision.queue.history().last().map(|action| action.status),
        Some(ActionStatus::Rejected)
    );

    for queued_first in [false, true] {
        let case = temp
            .path()
            .join(if queued_first { "pending" } else { "implicit" });
        fs::create_dir(&case).expect("case dir");
        let (mut state, _) = prepared_live_master_state(&case);
        let destination = case.join("rolled-back.dawproject");
        if queued_first {
            assert!(matches!(
                state.queue_live_master_dawproject_export(
                    5_100,
                    Some(destination.to_string_lossy().into_owned())
                ),
                super::DawSessionExportQueueResult::Enqueued { .. }
            ));
        }
        let invalid_session_destination = case.join("session-is-a-directory");
        fs::create_dir(&invalid_session_destination).expect("invalid Session destination");
        state.files = Some(JamFileSet {
            session_path: invalid_session_destination,
            source_graph_path: None,
        });
        assert!(
            state
                .commit_and_save_live_master_dawproject_export(&destination, 5_200)
                .is_err(),
            "queued_first={queued_first}"
        );
        assert!(!destination.exists(), "queued_first={queued_first}");
        assert_eq!(
            state.session.export_receipts.len(),
            1,
            "queued_first={queued_first}"
        );
        assert_eq!(
            state.queue.history().last().map(|action| action.status),
            Some(ActionStatus::Rejected)
        );
    }
}
