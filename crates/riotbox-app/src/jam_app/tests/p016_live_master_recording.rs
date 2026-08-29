use std::fs;

use riotbox_audio::{
    runtime::{AudioOutputInfo, AudioRuntimeHealth, AudioRuntimeLifecycle},
    w30::{W30PreviewRenderMode, W30PreviewRenderRouting},
};
use riotbox_core::{
    action::{ActionCommand, ActionParams, ActionStatus},
    ids::{CaptureId, SceneId},
    persistence::load_session_json,
    queue::ActionQueue,
    session::{CaptureRef, CaptureType, ExportArtifactRole, SessionFile},
};
use tempfile::tempdir;

use super::super::{
    JamAppState, JamFileSet, LIVE_MASTER_RECORDING_DURATION_BEATS,
    LIVE_MASTER_RECORDING_PROOF_SCHEMA, LiveMasterRecordingPlan, LiveMasterRecordingProof,
    LiveMasterRecordingQueueResult, product_export::sha256_file,
};

fn live_master_recording_state() -> JamAppState {
    let mut session = SessionFile::new(
        "live-master-session",
        "riotbox-test",
        "2026-08-28T00:00:00Z",
    );
    session.runtime_state.source_timing.confirmed_bpm = Some(120.0);
    session.runtime_state.scene_state.scenes = vec![SceneId::from("scene-live")];
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-live"));
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-live"));
    JamAppState::from_parts(session, None, ActionQueue::new())
}

fn live_master_capture_ref(capture_id: &str, lineage_capture_refs: &[&str]) -> CaptureRef {
    CaptureRef {
        capture_id: CaptureId::from(capture_id),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec![format!("source-{capture_id}")],
        source_window: None,
        lineage_capture_refs: lineage_capture_refs
            .iter()
            .map(|capture_id| CaptureId::from(*capture_id))
            .collect(),
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: format!("captures/{capture_id}.wav"),
        assigned_target: None,
        is_pinned: false,
        notes: None,
    }
}

fn live_master_test_output() -> AudioOutputInfo {
    AudioOutputInfo {
        host_name: "Alsa".into(),
        device_name: "pipewire-default".into(),
        sample_format: "F32".into(),
        sample_rate: 1_000,
        channel_count: 2,
        buffer_size: "Fixed(64)".into(),
        supported_output_config_count: Some(1),
    }
}

fn live_master_test_health(output: &AudioOutputInfo) -> AudioRuntimeHealth {
    AudioRuntimeHealth {
        lifecycle: AudioRuntimeLifecycle::Running,
        output: Some(output.clone()),
        callback_count: 80,
        max_callback_gap_micros: Some(12_000),
        callback_scratch_overflow_count: 0,
        stream_error_count: 0,
        last_stream_error: None,
    }
}

fn live_master_test_outcome(
    plan: &LiveMasterRecordingPlan,
) -> riotbox_audio::runtime::LiveMasterCaptureOutcome {
    let sample_count = plan.request.target_frame_count * usize::from(plan.output.channel_count);
    let samples = (0..sample_count)
        .map(|index| if index.is_multiple_of(2) { 0.25 } else { -0.25 })
        .collect::<Vec<_>>();
    let beats_per_frame = f64::from(plan.confirmed_bpm) / 60.0 / f64::from(plan.output.sample_rate);
    riotbox_audio::runtime::LiveMasterCaptureOutcome {
        samples,
        progress: riotbox_audio::runtime::LiveMasterCaptureProgress {
            target_sample_count: sample_count,
            written_sample_count: sample_count,
            callback_count: 80,
            max_callback_gap_micros: Some(12_000),
            callback_gap_over_threshold_count: 0,
            callback_scratch_overflow_count: 0,
            stream_error_count: 0,
            transport_mismatch_count: 0,
            tempo_mismatch_count: 0,
            timing_window_mismatch_count: 0,
            armed_callback_count: 1,
            capture_started: true,
            complete: true,
        },
        captured_start_position_beats: Some(plan.requested_start_position_beats),
        captured_end_position_beats: Some(
            plan.requested_start_position_beats
                + beats_per_frame * plan.request.target_frame_count as f64,
        ),
    }
}

#[test]
fn real_runtime_master_take_commits_wav_proof_action_session_and_receipt() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("live-master.wav");
    let output = live_master_test_output();
    let mut state = live_master_recording_state();
    let plan = match state.queue_live_master_recording(1_000, &output, &destination) {
        LiveMasterRecordingQueueResult::Enqueued(plan) => plan,
        other => panic!("expected queued live master recording, got {other:?}"),
    };
    assert_eq!(plan.request.target_frame_count, 4_000);
    assert_eq!(plan.request.expected_tempo_bpm, 120.0);
    assert_eq!(plan.request.start_position_beats, Some(4.0));
    assert_eq!(plan.bar_grid_anchor_beat_cursor, 0);
    let outcome = live_master_test_outcome(&plan);
    let health = live_master_test_health(&output);
    state.set_audio_health(health.clone());

    let receipt = state
        .commit_live_master_recording(&plan, &outcome, &health, 5_000)
        .expect("commit live master recording");

    assert!(destination.is_file());
    assert!(plan.proof_path.is_file());
    assert!(receipt.is_live_recording_runtime_master_bar_window_v2());
    assert!(receipt.live_recording_runtime_master_ready());
    assert!(receipt.live_recording_host_audio_readiness_report().ready());
    assert_eq!(receipt.export_hash, sha256_file(&destination).unwrap());
    assert_eq!(receipt.artifact_set.len(), 2);
    let audio = receipt
        .artifact_set
        .iter()
        .find(|artifact| artifact.role == ExportArtifactRole::LiveRecordingCapture)
        .expect("live recording artifact");
    assert_eq!(audio.sample_rate_hz, Some(1_000));
    assert_eq!(audio.channel_count, Some(2));
    assert_eq!(audio.duration_ms, Some(4_000));
    assert_eq!(
        audio
            .audio_metrics
            .as_ref()
            .and_then(|metrics| metrics.total_frame_count),
        Some(4_000)
    );
    assert_eq!(
        receipt
            .qa_gates
            .iter()
            .map(|gate| gate.gate_id.as_str())
            .collect::<Vec<_>>(),
        vec![
            riotbox_core::session::LIVE_RECORDING_RUNTIME_CAPTURE_QA_GATE_ID,
            riotbox_core::session::LIVE_RECORDING_WAV_READBACK_QA_GATE_ID,
            riotbox_core::session::LIVE_RECORDING_BAR_WINDOW_ALIGNMENT_QA_GATE_ID,
        ]
    );
    let proof: LiveMasterRecordingProof =
        serde_json::from_slice(&fs::read(&plan.proof_path).expect("read proof"))
            .expect("parse proof");
    assert_eq!(proof.schema, LIVE_MASTER_RECORDING_PROOF_SCHEMA);
    assert_eq!(proof.frame_count, 4_000);
    assert_eq!(proof.scene_id, SceneId::from("scene-live"));
    assert_eq!(proof.duration_beats, LIVE_MASTER_RECORDING_DURATION_BEATS);
    assert_eq!(proof.beats_per_bar, 4);
    assert_eq!(proof.bar_grid_anchor_position_microbeats, 0);
    assert_eq!(proof.beat_span_per_frame_nanobeats, 2_000_000);
    assert_eq!(proof.requested_start_position_microbeats, 4_000_000);
    assert_eq!(proof.captured_start_position_microbeats, 4_000_000);
    assert_eq!(proof.captured_end_position_microbeats, 12_000_000);
    assert_eq!(proof.start_alignment_error_frame_micros, 0);
    assert_eq!(proof.duration_error_frame_micros, 0);
    assert_eq!(proof.wav_sha256, receipt.export_hash);
    assert_eq!(proof.clip_count, 0);
    assert_eq!(state.session.export_receipts, vec![receipt.clone()]);
    let committed = state
        .session
        .action_log
        .actions
        .iter()
        .find(|action| action.id == plan.action_id)
        .expect("committed live recording action");
    assert_eq!(committed.status, ActionStatus::Committed);
    assert_eq!(committed.target.scene_id, Some(SceneId::from("scene-live")));
    assert!(matches!(
        committed.params,
        ActionParams::LiveRecordingExport {
            boundary: riotbox_core::action::LiveRecordingExportBoundary::RuntimeMasterBarWindowV2,
            ..
        }
    ));
    let restored: SessionFile =
        serde_json::from_slice(&serde_json::to_vec(&state.session).expect("serialize Session"))
            .expect("restore Session");
    assert_eq!(restored.export_receipts, state.session.export_receipts);
    assert_eq!(restored.action_log, state.session.action_log);
}

#[test]
fn live_transport_progress_cannot_commit_capture_before_the_writer_succeeds() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("live-master.wav");
    let output = live_master_test_output();
    let mut state = live_master_recording_state();
    state.runtime.transport.is_playing = true;
    state.session.runtime_state.transport.is_playing = true;
    let plan = match state.queue_live_master_recording(1_000, &output, &destination) {
        LiveMasterRecordingQueueResult::Enqueued(plan) => plan,
        other => panic!("expected queued live master recording, got {other:?}"),
    };

    for position_beats in [1.0, 4.0, 8.0] {
        state.apply_audio_timing_snapshot(
            riotbox_audio::runtime::AudioRuntimeTimingSnapshot {
                is_transport_running: true,
                tempo_bpm: 120.0,
                position_beats,
            },
            1_000 + (position_beats * 500.0) as u64,
        );
    }

    assert_eq!(
        state
            .queue
            .pending_actions()
            .iter()
            .map(|action| action.id)
            .collect::<Vec<_>>(),
        vec![plan.action_id]
    );
    assert!(state.queue.history_action(plan.action_id).is_none());

    let outcome = live_master_test_outcome(&plan);
    let health = live_master_test_health(&output);
    state.set_audio_health(health.clone());
    state
        .commit_live_master_recording(&plan, &outcome, &health, 6_000)
        .expect("writer success explicitly commits live master action");

    assert!(state.queue.pending_actions().is_empty());
    assert_eq!(
        state
            .queue
            .history_action(plan.action_id)
            .expect("committed action")
            .status,
        ActionStatus::Committed
    );
}

#[test]
fn live_master_recording_fails_closed_on_callback_fault_without_final_files_or_receipt() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("faulted-live-master.wav");
    let output = live_master_test_output();
    let mut state = live_master_recording_state();
    let plan = match state.queue_live_master_recording(1_000, &output, &destination) {
        LiveMasterRecordingQueueResult::Enqueued(plan) => plan,
        other => panic!("expected queued live master recording, got {other:?}"),
    };
    let mut outcome = live_master_test_outcome(&plan);
    outcome.progress.callback_gap_over_threshold_count = 1;
    let health = live_master_test_health(&output);

    let error = state
        .commit_live_master_recording(&plan, &outcome, &health, 5_000)
        .expect_err("faulted capture rejected");

    assert!(error.to_string().contains("faulted"));
    assert!(!destination.exists());
    assert!(!plan.proof_path.exists());
    assert!(state.session.export_receipts.is_empty());
    assert!(state.queue.pending_actions().is_empty());
    assert_eq!(
        state
            .queue
            .history_action(plan.action_id)
            .expect("rejected action")
            .status,
        ActionStatus::Rejected
    );
}

#[test]
fn live_master_recording_rederives_and_enforces_the_frozen_eight_beat_window() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("mutated-window-live-master.wav");
    let output = live_master_test_output();
    let mut state = live_master_recording_state();
    let mut plan = match state.queue_live_master_recording(1_000, &output, &destination) {
        LiveMasterRecordingQueueResult::Enqueued(plan) => plan,
        other => panic!("expected queued live master recording, got {other:?}"),
    };
    plan.request.target_frame_count -= 1;
    let outcome = live_master_test_outcome(&plan);
    let health = live_master_test_health(&output);

    let error = state
        .commit_live_master_recording(&plan, &outcome, &health, 5_000)
        .expect_err("mutated capture window must fail closed");

    assert!(error.to_string().contains("identity changed"));
    assert!(!destination.exists());
    assert!(!plan.proof_path.exists());
    assert!(state.session.export_receipts.is_empty());
    assert_eq!(
        state
            .queue
            .history_action(plan.action_id)
            .expect("mutated plan action")
            .status,
        ActionStatus::Rejected
    );
}

#[test]
fn live_master_recording_rejects_a_capture_start_later_than_one_output_frame() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("misaligned-live-master.wav");
    let output = live_master_test_output();
    let mut state = live_master_recording_state();
    let plan = match state.queue_live_master_recording(1_000, &output, &destination) {
        LiveMasterRecordingQueueResult::Enqueued(plan) => plan,
        other => panic!("expected queued live master recording, got {other:?}"),
    };
    let mut outcome = live_master_test_outcome(&plan);
    let beats_per_frame = f64::from(plan.confirmed_bpm) / 60.0 / f64::from(output.sample_rate);
    outcome.captured_start_position_beats =
        Some(plan.requested_start_position_beats + beats_per_frame * 2.0);
    outcome.captured_end_position_beats = Some(
        outcome
            .captured_start_position_beats
            .expect("captured start")
            + beats_per_frame * plan.request.target_frame_count as f64,
    );

    let error = state
        .commit_live_master_recording(&plan, &outcome, &live_master_test_health(&output), 5_000)
        .expect_err("late capture start must fail closed");

    assert!(error.to_string().contains("timing window is not exact"));
    assert!(!destination.exists());
    assert!(!plan.proof_path.exists());
    assert!(state.session.export_receipts.is_empty());
}

#[test]
fn live_master_recording_records_destination_collision_as_rejected_action() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("existing-live-master.wav");
    fs::write(&destination, b"owned-by-user").expect("write collision");
    let output = live_master_test_output();
    let mut state = live_master_recording_state();

    let result = state.queue_live_master_recording(1_000, &output, &destination);

    assert!(matches!(
        result,
        LiveMasterRecordingQueueResult::Rejected { .. }
    ));
    assert!(state.queue.pending_actions().is_empty());
    let rejected = state
        .queue
        .history()
        .iter()
        .find(|action| action.command == ActionCommand::ExportLiveRecording)
        .expect("rejected live recording action");
    assert_eq!(rejected.status, ActionStatus::Rejected);
    assert_eq!(fs::read(&destination).unwrap(), b"owned-by-user");
}

#[test]
fn live_master_recording_records_only_active_render_capture_lineage() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("active-lineage-live-master.wav");
    let output = live_master_test_output();
    let mut state = live_master_recording_state();
    state.session.captures = vec![
        live_master_capture_ref("cap-active", &["cap-active-source"]),
        live_master_capture_ref("cap-active-source", &[]),
        live_master_capture_ref("cap-inactive", &["cap-inactive-source"]),
        live_master_capture_ref("cap-inactive-source", &[]),
    ];
    state.runtime.w30_preview.mode = W30PreviewRenderMode::LiveRecall;
    state.runtime.w30_preview.routing = W30PreviewRenderRouting::MusicBusPreview;
    state.runtime.w30_preview.capture_id = Some("cap-active".into());
    state.runtime.w30_preview.music_bus_level = 0.8;

    let plan = match state.queue_live_master_recording(1_000, &output, &destination) {
        LiveMasterRecordingQueueResult::Enqueued(plan) => plan,
        other => panic!("expected queued live master recording, got {other:?}"),
    };

    assert_eq!(
        plan.source_capture_refs,
        vec![CaptureId::from("cap-active")]
    );
    assert_eq!(
        plan.lineage_capture_refs,
        vec![CaptureId::from("cap-active-source")]
    );
}

#[test]
fn live_master_recording_rejects_an_active_render_capture_missing_from_session() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("missing-lineage-live-master.wav");
    let output = live_master_test_output();
    let mut state = live_master_recording_state();
    state.runtime.w30_preview.mode = W30PreviewRenderMode::LiveRecall;
    state.runtime.w30_preview.routing = W30PreviewRenderRouting::MusicBusPreview;
    state.runtime.w30_preview.capture_id = Some("cap-missing".into());
    state.runtime.w30_preview.music_bus_level = 0.8;

    let result = state.queue_live_master_recording(1_000, &output, &destination);

    let LiveMasterRecordingQueueResult::Rejected { reason } = result else {
        panic!("missing active capture must reject");
    };
    assert!(reason.contains("cap-missing"));
    assert!(reason.contains("missing from Session"));
}

#[test]
fn live_master_recording_rejects_pathological_positive_bpm_before_runtime_allocation() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("oversized-live-master.wav");
    let output = AudioOutputInfo {
        sample_rate: 44_100,
        channel_count: 2,
        ..live_master_test_output()
    };
    let mut state = live_master_recording_state();
    state.session.runtime_state.source_timing.confirmed_bpm = Some(0.1);

    let result = state.queue_live_master_recording(1_000, &output, &destination);

    let LiveMasterRecordingQueueResult::Rejected { reason } = result else {
        panic!("oversized positive-BPM capture must reject");
    };
    assert!(reason.contains("allocation limit"));
    assert!(!destination.exists());
}

#[test]
fn live_master_recording_persists_session_after_owned_artifacts_are_validated() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("persisted-live-master.wav");
    let session_path = temp.path().join("session.json");
    let output = live_master_test_output();
    let mut state = live_master_recording_state();
    state.files = Some(JamFileSet {
        session_path: session_path.clone(),
        source_graph_path: None,
    });
    let plan = match state.queue_live_master_recording(1_000, &output, &destination) {
        LiveMasterRecordingQueueResult::Enqueued(plan) => plan,
        other => panic!("expected queued live master recording, got {other:?}"),
    };
    let outcome = live_master_test_outcome(&plan);
    let health = live_master_test_health(&output);

    let receipt = state
        .commit_and_save_live_master_recording(&plan, &outcome, &health, 5_000)
        .expect("commit and persist live master recording");

    let restored = load_session_json(&session_path).expect("load persisted Session");
    assert_eq!(restored.export_receipts, vec![receipt]);
    assert!(destination.is_file());
    assert!(plan.proof_path.is_file());
}

#[test]
fn live_master_recording_rolls_back_owned_files_and_action_when_session_save_fails() {
    let temp = tempdir().expect("tempdir");
    let destination = temp.path().join("rolled-back-live-master.wav");
    let output = live_master_test_output();
    let mut state = live_master_recording_state();
    state.files = Some(JamFileSet {
        session_path: temp.path().to_path_buf(),
        source_graph_path: None,
    });
    let session_before_commit = state.session.clone();
    let plan = match state.queue_live_master_recording(1_000, &output, &destination) {
        LiveMasterRecordingQueueResult::Enqueued(plan) => plan,
        other => panic!("expected queued live master recording, got {other:?}"),
    };
    let outcome = live_master_test_outcome(&plan);
    let health = live_master_test_health(&output);

    let error = state
        .commit_and_save_live_master_recording(&plan, &outcome, &health, 5_000)
        .expect_err("Session save failure must reject the recording transaction");

    assert!(error.to_string().contains("Session save failed"));
    assert_eq!(state.session, session_before_commit);
    assert!(state.session.export_receipts.is_empty());
    assert!(!destination.exists());
    assert!(!plan.proof_path.exists());
    assert_eq!(
        state
            .queue
            .history_action(plan.action_id)
            .expect("failed transaction action")
            .status,
        ActionStatus::Rejected
    );
}
