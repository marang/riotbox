//! Direct-poll acceptance controls, not new pre-fix regression evidence.
use std::{cell::Cell, fs, io, ops::ControlFlow, rc::Rc, time::Duration};

use super::super::{RuntimeControl, capture_wait::CaptureWait};
use super::{Case, Event, capture_wait::arm, fixture};
use crate::cli::live_master_recording::observation::{
    Observation, WindowStopped, WindowStoppedSink,
};
use riotbox_audio::runtime::AudioRuntimeLifecycle;
use riotbox_core::{
    action::{
        ActionCommand, ActionDraft, ActionTarget, ActorType, LiveRecordingDuration, Quantization,
    },
    persistence::save_session_json,
    session::ReplayPolicy,
};

#[test]
fn acquired_sync_refusal_beats_completed_capture_and_aborts_before_return() {
    let (_owner, launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
    state.session.action_log.replay_policy = ReplayPolicy::DeterministicPreferred;
    let session_path = state.files.as_ref().unwrap().session_path.clone();
    save_session_json(&session_path, &state.session).unwrap();
    let saved = fs::read(&session_path).unwrap();
    let (plan, output) = arm(&mut state, &control);
    let id = state
        .queue
        .enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::TransportStop,
                Quantization::NextBar,
                ActionTarget::default(),
            ),
            1_001,
        )
        .unwrap();
    let wait = CaptureWait::new(
        &launch,
        &plan,
        &output,
        true,
        control.origin + Duration::from_secs(8),
    );
    control.elapsed.set(Duration::from_secs(6));
    assert_eq!(
        control.health_snapshot().lifecycle,
        AudioRuntimeLifecycle::Running
    );
    let progress = control.live_master_capture_progress().unwrap();
    assert!(progress.complete);
    assert_eq!(progress.fault_count(), 0);
    let ControlFlow::Break(Err(summary)) = wait.poll(&mut state, &mut control, true) else {
        panic!("an acquired synchronization refusal must beat otherwise healthy completion");
    };
    assert_eq!(summary["ready"], false);
    assert_eq!(summary["runtime_stopped"], true);
    let reason = summary["failure_reason"].as_str().unwrap();
    assert!(reason.contains("timing commit failed"), "{reason}");
    assert!(reason.contains("DeterministicPreferred"), "{reason}");
    assert_eq!(
        *control.events.borrow(),
        [Event::Begin, Event::Stop, Event::Abort, Event::Stop]
    );
    assert!(control.stopped.get());
    assert_eq!(control.elapsed.get(), Duration::from_secs(6));
    assert!(
        state
            .queue
            .pending_actions()
            .iter()
            .any(|action| action.id == id)
    );
    assert!(
        !state
            .session
            .action_log
            .actions
            .iter()
            .any(|action| action.id == id)
    );
    assert!(
        !state
            .session
            .action_log
            .commit_records
            .iter()
            .any(|record| record.action_id == id)
    );
    assert!(
        !state
            .session
            .action_log
            .actions
            .iter()
            .any(|action| action.id == plan.action_id)
    );
    assert!(state.session.export_receipts.is_empty());
    assert_eq!(fs::read(&session_path).unwrap(), saved);
    control.assert_unpublished();
}

struct CountingSink {
    writes: Rc<Cell<usize>>,
}

impl WindowStoppedSink for CountingSink {
    fn record_window_stopped(&mut self, _milestone: &WindowStopped) -> io::Result<()> {
        self.writes.set(self.writes.get() + 1);
        Ok(())
    }
}

#[test]
fn observed_poll_returns_plan_bound_clocks_after_stop_without_emitting_milestone() {
    let (_owner, launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
    let session_path = state.files.as_ref().unwrap().session_path.clone();
    let saved = fs::read(&session_path).unwrap();
    let (plan, output) = arm(&mut state, &control);
    let wait = CaptureWait::new(
        &launch,
        &plan,
        &output,
        true,
        control.origin + Duration::from_secs(8),
    );
    let writes = Rc::new(Cell::new(0));
    let mut sink = CountingSink {
        writes: writes.clone(),
    };
    let mut observation = Observation::new(Some(&mut sink));
    control.elapsed.set(Duration::from_secs(6));
    let ControlFlow::Break(Ok(stopped)) =
        wait.poll(&mut state, &mut control, observation.enabled())
    else {
        panic!("an observed healthy completion must stop before returning");
    };
    assert!(control.stopped.get());
    assert_eq!(control.elapsed.get(), Duration::from_secs(6));
    assert_eq!(
        *control.events.borrow(),
        [
            Event::Begin,
            Event::CompletionClock,
            Event::Stop,
            Event::StoppedClock
        ]
    );
    assert_eq!(
        writes.get(),
        0,
        "poll returns evidence but must not emit it"
    );
    assert!(state.session.export_receipts.is_empty());
    assert_eq!(state.queue.pending_actions().len(), 1);
    control.assert_unpublished();
    assert_eq!(fs::read(&session_path).unwrap(), saved);

    let (pre_stop_health, milestone) = stopped.into_parts();
    assert_eq!(pre_stop_health.lifecycle, AudioRuntimeLifecycle::Running);
    let milestone = milestone.expect("enabled observation must bind a stopped milestone");
    // Serialization is deliberately performed by the caller after poll returns.
    let value = serde_json::to_value(&milestone).unwrap();
    assert_eq!(value["event"], "live_master_recording_window_stopped");
    assert_eq!(value["window_complete"], true);
    assert_eq!(value["runtime_stopped"], true);
    assert_eq!(value["fault_count"], 0);
    assert_eq!(value["window_complete_monotonic_ns"], 7_000_000_000_u64);
    assert_eq!(value["runtime_stopped_monotonic_ns"], 7_000_000_000_u64);
    assert_eq!(value["action_id"], plan.action_id.0);
    assert_eq!(value["session_id"], plan.session_id);
    assert_eq!(
        value["session_pre_capture_sha256"],
        plan.session_pre_capture_sha256
    );
    assert_eq!(value["duration_bars"], plan.duration.bars());
    assert_eq!(value["duration_beats"], plan.duration.duration_beats());
    assert_eq!(
        value["requested_start_position_beats"],
        plan.requested_start_position_beats
    );
    assert_eq!(value["frame_count"], plan.request.target_frame_count);
    assert_eq!(value["sample_rate_hz"], plan.output.sample_rate);
    assert_eq!(value["channel_count"], plan.output.channel_count);
    assert_eq!(
        value["source_graph_ref"],
        serde_json::to_value(&plan.source_graph_ref).unwrap()
    );
    assert_eq!(
        value["timing_grid_ref"],
        serde_json::to_value(&plan.timing_grid_ref).unwrap()
    );
    assert_eq!(
        value["source_capture_refs"],
        serde_json::to_value(&plan.source_capture_refs).unwrap()
    );
    assert_eq!(
        value["lineage_capture_refs"],
        serde_json::to_value(&plan.lineage_capture_refs).unwrap()
    );
    assert!(value.get("ready").is_none());

    // Only this explicit caller operation may contact the observation sink.
    observation.record(&milestone);
    observation.into_result().unwrap();
    assert_eq!(writes.get(), 1);
    assert!(!control.events.borrow().contains(&Event::Finish));
    control.assert_unpublished();
    assert_eq!(fs::read(&session_path).unwrap(), saved);
}
