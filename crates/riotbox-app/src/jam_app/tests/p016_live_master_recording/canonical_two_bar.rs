//! Generated V4 controls; not host-device or human-listening qualification.
use super::{
    live_master_recording_state, live_master_test_health, live_master_test_outcome,
    live_master_test_output,
};
use crate::jam_app::{
    JamAppState, JamFileSet, LiveMasterRecordingProof, LiveMasterRecordingQueueResult,
    live_master_recording::LIVE_MASTER_RECORDING_PROOF_SCHEMA_V4,
};
use riotbox_core::action::{ActionParams, LiveRecordingDuration, LiveRecordingExportBoundary};
use riotbox_core::session::LIVE_RECORDING_BAR_WINDOW_ALIGNMENT_QA_GATE_ID;
use std::fs;
use tempfile::tempdir;

#[test]
fn canonical_two_bar_rounding_ties_publish_fully_ready_identity_and_restore() {
    for (rate, bpm, explicit) in [
        (48_000, 121.5, false),
        (48_000, 166.5, true),
        (44_100, 80.6632_f32, false),
        (44_100, 80.6838_f32, true),
        (44_100, 120.1_f32, false),
        (96_000, 137.25, true),
    ] {
        let temp = tempdir().unwrap();
        let destination = temp.path().join("canonical.wav");
        let session_path = temp.path().join("session.json");
        let mut output = live_master_test_output();
        output.sample_rate = rate;
        let mut state = live_master_recording_state();
        state.session.runtime_state.source_timing.confirmed_bpm = Some(bpm);
        state.files = Some(JamFileSet {
            session_path: session_path.clone(),
            source_graph_path: None,
        });
        let queued = if explicit {
            state.queue_live_master_recording_with_duration(
                1_000,
                &output,
                &destination,
                LiveRecordingDuration::TwoBars,
            )
        } else {
            state.queue_live_master_recording(1_000, &output, &destination)
        };
        let LiveMasterRecordingQueueResult::Enqueued(plan) = queued else {
            panic!("queue canonical two-bar take");
        };
        assert_eq!(
            plan.boundary,
            LiveRecordingExportBoundary::RuntimeMasterBarWindowV4
        );
        let receipt = state
            .commit_and_save_live_master_recording(
                &plan,
                &live_master_test_outcome(&plan),
                &live_master_test_health(&output),
                40_000,
            )
            .unwrap();
        assert!(receipt.is_live_recording_runtime_master_bar_window_v4());
        assert!(receipt.live_recording_runtime_master_ready());
        assert_eq!(
            receipt
                .qa_gates
                .iter()
                .find(|gate| gate.gate_id == LIVE_RECORDING_BAR_WINDOW_ALIGNMENT_QA_GATE_ID)
                .unwrap()
                .summary
                .as_deref(),
            Some(
                "real callback capture began on the requested 4/4 bar boundary within one output frame and completed the exact 2-bar window"
            )
        );
        assert_eq!(
            receipt.live_recording_duration,
            Some(LiveRecordingDuration::TwoBars)
        );
        let proof: LiveMasterRecordingProof =
            serde_json::from_slice(&fs::read(&plan.proof_path).unwrap()).unwrap();
        assert_eq!(proof.schema, LIVE_MASTER_RECORDING_PROOF_SCHEMA_V4);
        assert_eq!(proof.duration, Some(LiveRecordingDuration::TwoBars));
        assert_eq!(proof.duration_beats, 8);
        assert_eq!(
            proof.frame_count,
            LiveRecordingDuration::TwoBars
                .target_frame_count(rate, proof.confirmed_bpm_micros,)
                .unwrap()
        );
        if bpm == 121.5 {
            assert_eq!(proof.beat_span_per_frame_nanobeats, 42_187);
        }
        if bpm == 166.5 {
            assert_eq!(proof.beat_span_per_frame_nanobeats, 57_812);
        }
        let restored =
            JamAppState::from_json_files(&session_path, None::<&std::path::Path>).unwrap();
        assert_eq!(restored.session.export_receipts, vec![receipt.clone()]);
        let action = restored
            .session
            .action_log
            .actions
            .iter()
            .find(|action| action.id == plan.action_id)
            .unwrap();
        assert!(matches!(
            &action.params,
            ActionParams::LiveRecordingExport {
                boundary: LiveRecordingExportBoundary::RuntimeMasterBarWindowV4,
                duration: Some(LiveRecordingDuration::TwoBars),
                ..
            }
        ));
        let mut changed = restored.session;
        changed.export_receipts[0].live_recording_duration = None;
        fs::write(&session_path, serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(JamAppState::from_json_files(&session_path, None::<&std::path::Path>).is_err());
    }
}

#[test]
fn canonical_two_bar_endpoint_or_version_drift_does_not_publish() {
    for version_drift in [false, true] {
        let temp = tempdir().unwrap();
        let destination = temp.path().join("refused.wav");
        let mut output = live_master_test_output();
        output.sample_rate = 48_000;
        let mut state = live_master_recording_state();
        state.session.runtime_state.source_timing.confirmed_bpm = Some(121.5);
        let LiveMasterRecordingQueueResult::Enqueued(mut plan) =
            state.queue_live_master_recording(1_000, &output, &destination)
        else {
            panic!("queue V4");
        };
        let mut outcome = live_master_test_outcome(&plan);
        if version_drift {
            plan.boundary = LiveRecordingExportBoundary::RuntimeMasterBarWindowV2;
        } else {
            *outcome.captured_end_position_beats.as_mut().unwrap() +=
                (f64::from(plan.confirmed_bpm) / 60.0 / f64::from(output.sample_rate)) * 0.25;
        }
        assert!(
            state
                .commit_live_master_recording(
                    &plan,
                    &outcome,
                    &live_master_test_health(&output),
                    40_000,
                )
                .is_err()
        );
        assert!(!destination.exists());
        assert!(!plan.proof_path.exists());
        assert!(state.session.export_receipts.is_empty());
    }
}

#[test]
fn canonical_two_bar_unrepresentable_tempo_fails_before_publication() {
    let temp = tempdir().unwrap();
    let destination = temp.path().join("refused.wav");
    let output = live_master_test_output();
    let mut state = live_master_recording_state();
    state.session.runtime_state.source_timing.confirmed_bpm = Some(4.123_456_5_f32);
    let LiveMasterRecordingQueueResult::Rejected { reason } =
        state.queue_live_master_recording(1_000, &output, &destination)
    else {
        panic!("reject noncanonical tempo");
    };
    assert!(reason.contains("exact runtime BPM"));
    assert!(!destination.exists());
    assert!(state.queue.pending_actions().is_empty());
}
