//! Tests the real controller's optional post-stop observation, not another lifecycle.
use std::{cell::RefCell, fs, io, path::PathBuf, rc::Rc};

use serde_json::Value;

use super::{Case, Event, fixture};
use crate::cli::live_master_recording::{
    apply_live_master_observer_status,
    capture_lifecycle::execute_with_control_observed,
    observation::{Observation, WindowStopped, WindowStoppedSink},
};
use riotbox_core::{action::LiveRecordingDuration, persistence::load_session_json};

struct GeneratedSink {
    events: Rc<RefCell<Vec<Event>>>,
    destination: PathBuf,
    session: PathBuf,
    baseline: Vec<u8>,
    records: Vec<Value>,
    fail: bool,
}

impl WindowStoppedSink for GeneratedSink {
    fn record_window_stopped(&mut self, milestone: &WindowStopped) -> io::Result<()> {
        assert!(self.events.borrow().contains(&Event::Stop));
        assert!(!self.events.borrow().contains(&Event::Finish));
        assert!(
            !self.destination.exists(),
            "observer must precede artifact I/O"
        );
        assert!(
            !crate::jam_app::live_master_recording_proof_path(&self.destination)
                .unwrap()
                .exists()
        );
        assert_eq!(
            fs::read(&self.session).unwrap(),
            self.baseline,
            "observer must precede Session save"
        );
        self.events.borrow_mut().push(Event::Milestone);
        self.records.push(serde_json::to_value(milestone).unwrap());
        if self.fail {
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "generated milestone write failure",
            ))
        } else {
            Ok(())
        }
    }
}

#[test]
fn post_stop_milestone_has_exact_plan_identity_and_precedes_publication() {
    for duration in [
        LiveRecordingDuration::TwoBars,
        LiveRecordingDuration::EightBars,
        LiveRecordingDuration::SixteenBars,
    ] {
        let (_owner, launch, mut state, mut control) = fixture(Case::Clean, duration, false);
        let session = state.files.as_ref().unwrap().session_path.clone();
        let mut sink = GeneratedSink {
            events: Rc::clone(&control.events),
            destination: control.destination.clone(),
            baseline: fs::read(&session).unwrap(),
            session: session.clone(),
            records: Vec::new(),
            fail: false,
        };
        let mut observation = Observation::new(Some(&mut sink));
        let summary =
            execute_with_control_observed(&launch, &mut state, &mut control, &mut observation)
                .unwrap();
        observation.into_result().unwrap();
        assert_eq!(summary["ready"], true);
        assert_eq!(
            *control.events.borrow(),
            [
                Event::Begin,
                Event::CompletionClock,
                Event::Stop,
                Event::StoppedClock,
                Event::Milestone,
                Event::Finish
            ]
        );
        assert_eq!(sink.records.len(), 1);
        let marker = &sink.records[0];
        assert_eq!(marker["event"], "live_master_recording_window_stopped");
        assert_eq!(marker["schema"], "riotbox.user_session_observer.v1");
        assert_eq!(
            marker["observation_schema"],
            "riotbox.live_master_recording_window_stopped.v1"
        );
        assert_eq!(marker["pid"], std::process::id());
        assert_eq!(marker["clock_domain"], "linux_clock_monotonic");
        assert_eq!(marker["window_complete"], true);
        assert_eq!(marker["runtime_stopped"], true);
        assert_eq!(marker["fault_count"], 0);
        assert!(
            marker["window_complete_monotonic_ns"].as_u64().unwrap()
                <= marker["runtime_stopped_monotonic_ns"].as_u64().unwrap()
        );
        assert_eq!(marker["opt_in"], true);
        assert_eq!(marker["capture_context"], "non_interactive_real_audio_cli");
        assert_eq!(marker["realtime_callback_io"], false);
        assert_eq!(marker["raw_audio_recording"], false);
        assert_eq!(marker["live_master_recording"], true);
        assert!(marker.get("ready").is_none() && marker.get("status").is_none());
        let proof: Value = serde_json::from_slice(
            &fs::read(
                crate::jam_app::live_master_recording_proof_path(&control.destination).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        for key in [
            "action_id",
            "session_id",
            "session_pre_capture_sha256",
            "duration_beats",
            "frame_count",
            "sample_rate_hz",
            "channel_count",
            "source_graph_ref",
            "timing_grid_ref",
            "source_capture_refs",
            "lineage_capture_refs",
        ] {
            assert_eq!(marker[key], proof[key], "milestone/proof mismatch: {key}");
        }
        assert_eq!(marker["duration_bars"], duration.bars());
        assert_eq!(
            marker["requested_start_position_beats"],
            summary["requested_start_position_beats"]
        );
        assert_eq!(
            load_session_json(&session).unwrap().export_receipts,
            state.session.export_receipts
        );
    }
}

#[test]
fn unavailable_or_reversed_clock_cannot_qualify_but_does_not_reject_product_capture() {
    for case in [
        Case::MissingCompletionClock,
        Case::MissingStoppedClock,
        Case::MissingBothClocks,
        Case::ReversedClock,
    ] {
        let (_owner, launch, mut state, mut control) =
            fixture(case, LiveRecordingDuration::TwoBars, false);
        let session = state.files.as_ref().unwrap().session_path.clone();
        let mut sink = GeneratedSink {
            events: Rc::clone(&control.events),
            destination: control.destination.clone(),
            baseline: fs::read(&session).unwrap(),
            session,
            records: Vec::new(),
            fail: false,
        };
        let mut observation = Observation::new(Some(&mut sink));
        let summary =
            execute_with_control_observed(&launch, &mut state, &mut control, &mut observation)
                .unwrap();
        observation.into_result().unwrap();
        assert_eq!(summary["ready"], true);
        let marker = &sink.records[0];
        assert!(
            marker["window_complete_monotonic_ns"].is_null()
                || marker["runtime_stopped_monotonic_ns"].is_null()
        );
        assert_eq!(
            marker["window_complete_monotonic_ns"].is_null(),
            case != Case::MissingStoppedClock
        );
        assert_eq!(
            marker["runtime_stopped_monotonic_ns"].is_null(),
            case != Case::MissingCompletionClock
        );
    }
}

#[test]
fn failed_stop_never_emits_a_stopped_milestone_or_finalizes() {
    let (_owner, launch, mut state, mut control) =
        fixture(Case::StopUnconfirmed, LiveRecordingDuration::TwoBars, false);
    let session = state.files.as_ref().unwrap().session_path.clone();
    let mut sink = GeneratedSink {
        events: Rc::clone(&control.events),
        destination: control.destination.clone(),
        baseline: fs::read(&session).unwrap(),
        session,
        records: Vec::new(),
        fail: false,
    };
    let mut observation = Observation::new(Some(&mut sink));
    let summary =
        execute_with_control_observed(&launch, &mut state, &mut control, &mut observation).unwrap();
    observation.into_result().unwrap();
    assert_eq!(summary["ready"], false);
    assert_eq!(summary["runtime_stopped"], false);
    assert!(sink.records.is_empty());
    assert!(!control.events.borrow().contains(&Event::Finish));
}

#[test]
fn milestone_write_error_is_latched_even_when_final_observer_write_succeeds() {
    let (_owner, launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
    let session = state.files.as_ref().unwrap().session_path.clone();
    let mut sink = GeneratedSink {
        events: Rc::clone(&control.events),
        destination: control.destination.clone(),
        baseline: fs::read(&session).unwrap(),
        session,
        records: Vec::new(),
        fail: true,
    };
    let mut observation = Observation::new(Some(&mut sink));
    let mut summary =
        execute_with_control_observed(&launch, &mut state, &mut control, &mut observation).unwrap();
    let result = observation.into_result().and(Ok(()));
    assert!(result.is_err());
    apply_live_master_observer_status(&mut summary, true, &result);
    assert_eq!(summary["ready"], true);
    assert_eq!(summary["observer_events"], false);
    assert_eq!(summary["observer_status"], "write_failed");
    assert_eq!(
        summary["observer_failure_reason"],
        "generated milestone write failure"
    );
    assert_eq!(state.session.export_receipts.len(), 1);
    assert!(control.destination.is_file());
}

#[test]
fn no_observer_does_not_read_clocks_or_change_the_existing_event_sequence() {
    let (_owner, launch, mut state, mut control) = fixture(
        Case::MissingBothClocks,
        LiveRecordingDuration::TwoBars,
        false,
    );
    let mut observation = Observation::new(None);
    let summary =
        execute_with_control_observed(&launch, &mut state, &mut control, &mut observation).unwrap();
    observation.into_result().unwrap();
    assert_eq!(summary["ready"], true);
    assert_eq!(
        *control.events.borrow(),
        [Event::Begin, Event::Stop, Event::Finish]
    );
}

#[test]
fn stopped_milestone_is_not_a_finalization_or_receipt_success_verdict() {
    for case in [Case::FinishFailure, Case::LateFault] {
        let (_owner, launch, mut state, mut control) =
            fixture(case, LiveRecordingDuration::TwoBars, false);
        let session = state.files.as_ref().unwrap().session_path.clone();
        let mut sink = GeneratedSink {
            events: Rc::clone(&control.events),
            destination: control.destination.clone(),
            baseline: fs::read(&session).unwrap(),
            session,
            records: Vec::new(),
            fail: false,
        };
        let mut observation = Observation::new(Some(&mut sink));
        let summary =
            execute_with_control_observed(&launch, &mut state, &mut control, &mut observation)
                .unwrap();
        observation.into_result().unwrap();
        assert_eq!(sink.records.len(), 1);
        assert_eq!(sink.records[0]["window_complete"], true);
        assert_eq!(sink.records[0]["runtime_stopped"], true);
        assert_eq!(summary["ready"], false);
        assert!(state.session.export_receipts.is_empty());
        assert!(!control.destination.exists());
    }
}
