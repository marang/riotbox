//! Direct cooperative supervision controls; no runtime construction or audio.
use std::{ops::ControlFlow, time::Duration};

use super::super::{
    RuntimeControl, capture_wait::CaptureWait, execute_with_control, sync_live_master_runtime,
};
use super::{Case, Event, GeneratedControl, fixture};
use crate::jam_app::{JamAppState, LiveMasterRecordingPlan, LiveMasterRecordingQueueResult};
use riotbox_audio::runtime::{AudioOutputInfo, AudioRuntimeLifecycle};
use riotbox_core::action::LiveRecordingDuration;

#[test]
fn completed_poll_returns_only_after_verified_stop_without_wait_or_publication() {
    let (_temp, launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
    let (plan, output) = arm(&mut state, &control);
    let deadline = control.origin + Duration::from_secs(8);
    let wait = CaptureWait::new(&launch, &plan, &output, true, deadline);
    // Explicit test-clock advancement, never a production wait/sleep.
    control.elapsed.set(Duration::from_secs(6));
    let ControlFlow::Break(Ok(stopped)) = wait.poll(&mut state, &mut control, false) else {
        panic!("a complete healthy capture must yield a verified-stop fact");
    };
    assert!(
        control.stopped.get(),
        "terminal return must not expose a running device"
    );
    assert_eq!(control.elapsed.get(), Duration::from_secs(6));
    assert_eq!(*control.events.borrow(), [Event::Begin, Event::Stop]);
    control.assert_unpublished();
    assert!(state.session.export_receipts.is_empty());
    assert_eq!(state.queue.pending_actions().len(), 1);
    let (pre_stop_health, milestone) = stopped.into_parts();
    assert_eq!(pre_stop_health.lifecycle, AudioRuntimeLifecycle::Running);
    assert!(
        milestone.is_none(),
        "disabled observation performs no clock/evidence work"
    );
}

pub(super) fn arm(
    state: &mut JamAppState,
    control: &GeneratedControl,
) -> (Box<LiveMasterRecordingPlan>, AudioOutputInfo) {
    state
        .commit_transport_toggle(control.timestamp_now())
        .unwrap();
    sync_live_master_runtime(state, control).unwrap();
    let output = control.output();
    let plan = match state
        .queue_live_master_recording(control.timestamp_now(), &output, &control.destination)
        .unwrap()
    {
        LiveMasterRecordingQueueResult::Enqueued(plan) => plan,
        _ => panic!("generated capture plan must be admitted"),
    };
    control
        .begin_live_master_capture(plan.request.clone())
        .unwrap();
    (plan, output)
}

#[test]
fn pending_polls_neither_wait_nor_restart_the_fixed_deadline() {
    for case in [Case::MissingProgress, Case::IncompleteProgress] {
        let (_temp, launch, mut state, mut control) =
            fixture(case, LiveRecordingDuration::TwoBars, false);
        let (plan, output) = arm(&mut state, &control);
        let deadline = control.origin + Duration::from_secs(8);
        let mut wait = CaptureWait::new(&launch, &plan, &output, true, deadline);
        for millis in [0, 0, 7_990] {
            control.elapsed.set(Duration::from_millis(millis));
            let ControlFlow::Continue(next) = wait.poll(&mut state, &mut control, true) else {
                panic!("an incomplete healthy capture before deadline must yield control");
            };
            wait = next;
            assert_eq!(control.elapsed.get(), Duration::from_millis(millis));
            assert!(!control.stopped.get());
            assert_eq!(*control.events.borrow(), [Event::Begin]);
            control.assert_unpublished();
            assert!(state.session.export_receipts.is_empty());
            assert_eq!(state.queue.pending_actions().len(), 1);
        }
        control.elapsed.set(Duration::from_secs(8));
        let ControlFlow::Break(Err(summary)) = wait.poll(&mut state, &mut control, true) else {
            panic!("the original fixed deadline must terminate incomplete capture");
        };
        assert_eq!(summary["ready"], false);
        assert_eq!(summary["runtime_stopped"], true);
        assert!(
            summary["failure_reason"]
                .as_str()
                .unwrap()
                .contains("stopped early")
        );
        assert_eq!(control.elapsed.get(), Duration::from_secs(8));
        assert_eq!(
            *control.events.borrow(),
            [Event::Begin, Event::Stop, Event::Abort, Event::Stop]
        );
        control.assert_unpublished();
        assert!(state.queue.pending_actions().is_empty());
        assert!(state.session.export_receipts.is_empty());
    }
}

#[test]
fn runtime_and_progress_faults_beat_simultaneous_completion() {
    for case in [
        Case::FaultAndComplete,
        Case::RuntimeFault,
        Case::ScratchFault,
        Case::ProgressFault,
    ] {
        let (_temp, launch, mut state, mut control) =
            fixture(case, LiveRecordingDuration::TwoBars, false);
        let (plan, output) = arm(&mut state, &control);
        let wait = CaptureWait::new(
            &launch,
            &plan,
            &output,
            true,
            control.origin + Duration::from_secs(8),
        );
        control.elapsed.set(Duration::from_secs(6));
        assert!(control.live_master_capture_progress().unwrap().complete);
        let ControlFlow::Break(Err(summary)) = wait.poll(&mut state, &mut control, false) else {
            panic!("a simultaneous fault must beat completion: {case:?}");
        };
        assert_eq!(summary["ready"], false);
        assert_eq!(summary["runtime_stopped"], true);
        let expected = if case == Case::ProgressFault {
            "callback capture faulted"
        } else {
            "runtime faulted"
        };
        assert!(
            summary["failure_reason"]
                .as_str()
                .unwrap()
                .contains(expected)
        );
        assert_eq!(control.elapsed.get(), Duration::from_secs(6));
        assert_eq!(
            *control.events.borrow(),
            [Event::Begin, Event::Stop, Event::Abort, Event::Stop]
        );
        control.assert_unpublished();
        assert!(state.queue.pending_actions().is_empty());
        assert!(state.session.export_receipts.is_empty());
    }
}

#[test]
fn stopped_fact_is_not_finish_or_publication_success() {
    for case in [
        Case::CompleteAtDeadline,
        Case::StopUnconfirmed,
        Case::FinishFailure,
        Case::LateFault,
    ] {
        // Separate borrowed-fixture phases avoid keeping two large App facades
        // on one ordinary test stack. Neither helper returns an App or Session.
        let stopped = direct_stop_boundary(case);
        let (wrapper_stopped, ready) = wrapper_terminal(case);
        assert_eq!(stopped, wrapper_stopped);
        assert_eq!(ready, case == Case::CompleteAtDeadline);
    }
}

fn direct_stop_boundary(case: Case) -> bool {
    let (_temp, launch, mut state, mut control) =
        fixture(case, LiveRecordingDuration::TwoBars, false);
    let (plan, output) = arm(&mut state, &control);
    let wait = CaptureWait::new(
        &launch,
        &plan,
        &output,
        true,
        control.origin + Duration::from_secs(8),
    );
    let completion = if case == Case::CompleteAtDeadline {
        8
    } else {
        6
    };
    control.elapsed.set(Duration::from_secs(completion));
    match wait.poll(&mut state, &mut control, false) {
        ControlFlow::Break(Ok(stopped)) => {
            assert_ne!(case, Case::StopUnconfirmed);
            let (pre_stop, milestone) = stopped.into_parts();
            assert_eq!(pre_stop.lifecycle, AudioRuntimeLifecycle::Running);
            assert!(milestone.is_none());
        }
        ControlFlow::Break(Err(summary)) => {
            assert_eq!(case, Case::StopUnconfirmed);
            assert_eq!(summary["runtime_stopped"], false);
        }
        ControlFlow::Continue(_) => panic!("complete capture must terminate: {case:?}"),
    }
    assert_eq!(control.elapsed.get(), Duration::from_secs(completion));
    assert!(!control.events.borrow().contains(&Event::Finish));
    assert!(!control.events.borrow().contains(&Event::Milestone));
    control.assert_unpublished();
    assert!(state.session.export_receipts.is_empty());
    control.stopped.get()
}

fn wrapper_terminal(case: Case) -> (bool, bool) {
    let (_temp, launch, mut state, mut control) =
        fixture(case, LiveRecordingDuration::TwoBars, false);
    let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
    let ready = summary["ready"].as_bool().unwrap();
    if !ready {
        control.assert_unpublished();
        assert!(state.session.export_receipts.is_empty());
    }
    (summary["runtime_stopped"].as_bool().unwrap(), ready)
}
