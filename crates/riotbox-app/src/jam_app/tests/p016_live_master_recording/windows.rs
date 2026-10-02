//! Generated-PCM integration evidence, never host-device or human listening proof.
use super::{
    live_master_recording_state, live_master_test_health, live_master_test_outcome,
    live_master_test_output,
};
use crate::jam_app::{
    JamAppState, JamFileSet, LiveMasterRecordingProof, LiveMasterRecordingQueueResult,
};
use riotbox_core::action::{
    ActionParams, ActionStatus, LiveRecordingDuration, LiveRecordingExportBoundary,
};
use riotbox_core::session::LIVE_RECORDING_BAR_WINDOW_ALIGNMENT_QA_GATE_ID;
use std::fs;
use tempfile::tempdir;

#[test]
fn legacy_two_bar_publication_preserves_legacy_rounding_and_readiness() {
    for (bpm, expected_span_nanobeats) in [(121.5, 42_187), (166.5, 57_812)] {
        let temp = tempdir().unwrap();
        let destination = temp.path().join("legacy-two-bar.wav");
        let session_path = temp.path().join("session.json");
        let mut output = live_master_test_output();
        output.sample_rate = 48_000;
        let mut state = live_master_recording_state();
        state.session.runtime_state.source_timing.confirmed_bpm = Some(bpm);
        state.files = Some(JamFileSet {
            session_path: session_path.clone(),
            source_graph_path: None,
        });
        let LiveMasterRecordingQueueResult::Enqueued(plan) =
            state.queue_legacy_two_bar_recording_fixture(1_000, &output, &destination)
        else {
            panic!("queue default two-bar window")
        };
        let receipt = state
            .commit_and_save_live_master_recording(
                &plan,
                &live_master_test_outcome(&plan),
                &live_master_test_health(&output),
                40_000,
            )
            .unwrap();
        let proof: LiveMasterRecordingProof =
            serde_json::from_slice(&fs::read(&plan.proof_path).unwrap()).unwrap();
        assert!(destination.is_file());
        assert_eq!(plan.duration, LiveRecordingDuration::TwoBars);
        assert_eq!(proof.duration, None);
        assert_eq!(receipt.live_recording_duration, None);
        assert_eq!(
            receipt
                .qa_gates
                .iter()
                .find(|gate| gate.gate_id == LIVE_RECORDING_BAR_WINDOW_ALIGNMENT_QA_GATE_ID)
                .unwrap()
                .summary
                .as_deref(),
            Some(
                "real callback capture began on the requested 4/4 bar boundary within one output frame and completed the exact two-bar window"
            )
        );
        assert!(receipt.is_live_recording_runtime_master_bar_window_v2());
        assert_eq!(proof.beat_span_per_frame_nanobeats, expected_span_nanobeats);
        assert!(receipt.live_recording_host_audio_readiness_report().ready());
        // Historical publication did not imply full receipt/DAW readiness.
        // Keep that distinction without retroactively rewriting V2 evidence.
        assert!(!receipt.live_recording_runtime_master_ready());
        let restored =
            JamAppState::from_json_files(&session_path, None::<&std::path::Path>).unwrap();
        assert_eq!(restored.session.export_receipts, vec![receipt]);
    }
}

#[test]
fn bounded_windows_bind_action_proof_receipt_and_restore_without_reopening_audio() {
    for (duration, rate, bpm) in [
        (LiveRecordingDuration::EightBars, 44_100, 137.25),
        (LiveRecordingDuration::SixteenBars, 48_000, 129.75),
        (LiveRecordingDuration::EightBars, 44_100, 80.0082_f32),
        (LiveRecordingDuration::SixteenBars, 48_000, 90.049_f32),
        (LiveRecordingDuration::EightBars, 44_100, 120.1_f32),
        (LiveRecordingDuration::SixteenBars, 48_000, 121.5),
    ] {
        let temp = tempdir().unwrap();
        let destination = temp.path().join("generated.wav");
        let session_path = temp.path().join("session.json");
        let mut output = live_master_test_output();
        output.sample_rate = rate;
        let mut state = live_master_recording_state();
        state.session.runtime_state.source_timing.confirmed_bpm = Some(bpm);
        state.files = Some(JamFileSet {
            session_path: session_path.clone(),
            source_graph_path: None,
        });
        let LiveMasterRecordingQueueResult::Enqueued(plan) =
            state.queue_live_master_recording_with_duration(1_000, &output, &destination, duration)
        else {
            panic!("queue window")
        };
        let outcome = live_master_test_outcome(&plan);
        let receipt = state
            .commit_and_save_live_master_recording(
                &plan,
                &outcome,
                &live_master_test_health(&output),
                40_000,
            )
            .unwrap();
        assert!(receipt.is_live_recording_runtime_master_bar_window_v3());
        assert!(receipt.live_recording_runtime_master_ready());
        assert_eq!(receipt.live_recording_duration, Some(duration));
        assert_eq!(
            receipt
                .qa_gates
                .iter()
                .find(|gate| gate.gate_id == LIVE_RECORDING_BAR_WINDOW_ALIGNMENT_QA_GATE_ID)
                .unwrap()
                .summary,
            Some(format!(
                "real callback capture began on the requested 4/4 bar boundary within one output frame and completed the exact {}-bar window",
                duration.bars()
            ))
        );
        let proof: LiveMasterRecordingProof =
            serde_json::from_slice(&fs::read(&plan.proof_path).unwrap()).unwrap();
        assert_eq!(
            proof.schema,
            "riotbox.live_recording_runtime_master_bar_window.v3"
        );
        assert_eq!(proof.duration, Some(duration));
        assert_eq!(proof.duration_beats, duration.duration_beats());
        assert_eq!(proof.frame_count as usize, plan.request.target_frame_count);
        let action = state
            .session
            .action_log
            .actions
            .iter()
            .find(|action| action.id == plan.action_id)
            .unwrap();
        assert!(matches!(action.params, ActionParams::LiveRecordingExport {
            boundary: LiveRecordingExportBoundary::RuntimeMasterBarWindowV3,
            duration: Some(actual), ..
        } if actual == duration));
        let restored =
            JamAppState::from_json_files(&session_path, None::<&std::path::Path>).unwrap();
        assert_eq!(
            restored.session.export_receipts,
            state.session.export_receipts
        );
        assert_eq!(restored.session.action_log, state.session.action_log);

        // Old V3 receipts used the legacy description. Restore must retain it,
        // not normalize hash-bound historical evidence to the new producer text.
        let mut historical = state.session.clone();
        historical.export_receipts[0]
            .qa_gates
            .iter_mut()
            .find(|gate| gate.gate_id == LIVE_RECORDING_BAR_WINDOW_ALIGNMENT_QA_GATE_ID)
            .unwrap()
            .summary = Some("real callback capture began on the requested 4/4 bar boundary within one output frame and completed the exact two-bar window".into());
        let stored = serde_json::to_vec(&historical).unwrap();
        fs::write(&session_path, &stored).unwrap();
        let historical_restored =
            JamAppState::from_json_files(&session_path, None::<&std::path::Path>).unwrap();
        assert_eq!(
            historical_restored.session.export_receipts,
            historical.export_receipts
        );
        assert!(
            historical_restored.session.export_receipts[0].live_recording_runtime_master_ready()
        );
        assert_eq!(fs::read(&session_path).unwrap(), stored);

        // A V3 recording must not silently expand the existing two-bar DAW contract.
        assert!(!receipt.is_live_recording_runtime_master_bar_window_v2());
        let daw_destination = temp.path().join("unsupported-v3.dawproject");
        assert!(
            state
                .commit_live_master_dawproject_export(&daw_destination, 41_000)
                .is_err()
        );
        assert!(!daw_destination.exists());

        let mut changed = state.session.clone();
        changed.export_receipts[0].live_recording_duration = Some(LiveRecordingDuration::TwoBars);
        fs::write(&session_path, serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(JamAppState::from_json_files(&session_path, None::<&std::path::Path>).is_err());
    }
}

#[test]
fn changed_plan_duration_cannot_replace_the_queued_window() {
    let temp = tempdir().unwrap();
    let destination = temp.path().join("changed.wav");
    let output = live_master_test_output();
    let mut state = live_master_recording_state();
    let LiveMasterRecordingQueueResult::Enqueued(mut plan) = state
        .queue_live_master_recording_with_duration(
            1_000,
            &output,
            &destination,
            LiveRecordingDuration::EightBars,
        )
    else {
        panic!("queue window")
    };
    plan.duration = LiveRecordingDuration::SixteenBars;
    plan.request.target_frame_count *= 2;
    let outcome = live_master_test_outcome(&plan);
    let error = state
        .commit_live_master_recording(&plan, &outcome, &live_master_test_health(&output), 40_000)
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("pending action identity changed")
    );
    assert!(state.session.export_receipts.is_empty());
    assert!(!destination.exists());
    assert!(!plan.proof_path.exists());
}

#[test]
fn long_window_rejects_allocation_overflow_before_callback_buffer_or_publication() {
    let temp = tempdir().unwrap();
    let destination = temp.path().join("oversized.wav");
    let mut output = live_master_test_output();
    output.sample_rate = 192_000;
    let mut state = live_master_recording_state();
    state.session.runtime_state.source_timing.confirmed_bpm = Some(60.0);
    let LiveMasterRecordingQueueResult::Rejected { reason } = state
        .queue_live_master_recording_with_duration(
            1_000,
            &output,
            &destination,
            LiveRecordingDuration::SixteenBars,
        )
    else {
        panic!("must reject oversized capture")
    };
    assert!(reason.contains("allocation limit"));
    assert!(state.queue.pending_actions().is_empty());
    assert!(!destination.exists());
}

#[test]
fn long_window_failed_session_save_rolls_back_only_owned_artifacts() {
    for duration in [
        LiveRecordingDuration::EightBars,
        LiveRecordingDuration::SixteenBars,
    ] {
        let temp = tempdir().unwrap();
        let destination = temp.path().join("rolled-back.wav");
        let output = live_master_test_output();
        let mut state = live_master_recording_state();
        state.files = Some(JamFileSet {
            session_path: temp.path().to_owned(),
            source_graph_path: None,
        });
        let before = state.session.clone();
        let LiveMasterRecordingQueueResult::Enqueued(plan) =
            state.queue_live_master_recording_with_duration(1_000, &output, &destination, duration)
        else {
            panic!("queue window")
        };
        let error = state
            .commit_and_save_live_master_recording(
                &plan,
                &live_master_test_outcome(&plan),
                &live_master_test_health(&output),
                40_000,
            )
            .unwrap_err();
        assert!(error.to_string().contains("Session save failed"));
        assert_eq!(state.session, before);
        assert_eq!(
            state.queue.history_action(plan.action_id).unwrap().status,
            ActionStatus::Rejected
        );
        assert!(!destination.exists());
        assert!(!plan.proof_path.exists());
    }
}

#[test]
fn unrepresentable_v3_runtime_tempo_rejects_before_allocating_or_writing() {
    let temp = tempdir().unwrap();
    let destination = temp.path().join("unrepresentable.wav");
    let output = live_master_test_output();
    let mut state = live_master_recording_state();
    state.session.runtime_state.source_timing.confirmed_bpm = Some(4.123_456_5_f32);
    let result = state.queue_live_master_recording_with_duration(
        1_000,
        &output,
        &destination,
        LiveRecordingDuration::EightBars,
    );
    let LiveMasterRecordingQueueResult::Rejected { reason } = result else {
        panic!("runtime tempo must roundtrip exactly");
    };
    assert!(reason.contains("exact runtime BPM"));
    assert!(state.queue.pending_actions().is_empty());
    assert!(!destination.exists());
}

#[test]
fn actual_clock_roundoff_is_accepted_but_not_a_displaced_endpoint() {
    // Endpoint values reproduced with production advance_transport_timing and
    // SharedLiveMasterCapture (16 bars, 96 kHz, 137.25 BPM, 128-frame callbacks).
    // The Audio regression exercises that actual clock; this tests publication.
    for displace in [false, true] {
        let temp = tempdir().unwrap();
        let destination = temp.path().join("clock.wav");
        let mut output = live_master_test_output();
        output.sample_rate = 96_000;
        let mut state = live_master_recording_state();
        state.session.runtime_state.source_timing.confirmed_bpm = Some(137.25);
        let LiveMasterRecordingQueueResult::Enqueued(plan) = state
            .queue_live_master_recording_with_duration(
                1_000,
                &output,
                &destination,
                LiveRecordingDuration::SixteenBars,
            )
        else {
            panic!("queue window")
        };
        assert_eq!(plan.request.target_frame_count, 2_685_902);
        let mut outcome = live_master_test_outcome(&plan);
        let span = 137.25 / 60.0 / 96_000.0;
        outcome.progress.callback_count = 20_985;
        outcome.captured_start_position_beats = Some(4.000_003_515_624_997);
        outcome.captured_end_position_beats =
            Some(68.000_012_109_405_22 + if displace { span / 4.0 } else { 0.0 });
        let result = state.commit_live_master_recording(
            &plan,
            &outcome,
            &live_master_test_health(&output),
            40_000,
        );
        if displace {
            assert!(result.is_err());
            assert!(!destination.exists());
            assert!(!plan.proof_path.exists());
            assert!(state.session.export_receipts.is_empty());
        } else {
            assert!(result.unwrap().live_recording_runtime_master_ready());
            let proof: LiveMasterRecordingProof =
                serde_json::from_slice(&fs::read(&plan.proof_path).unwrap()).unwrap();
            assert_eq!(proof.captured_end_position_microbeats, 68_000_012);
            assert!(proof.duration_error_frame_micros <= 500_001);
        }
    }
}
