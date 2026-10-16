//! Generated terminal events through the real recorder policy and App/Core queue.
use std::{cell::Cell, io, rc::Rc, time::Duration};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use riotbox_core::action::{ActionCommand, LiveRecordingDuration};

use super::super::{
    execute_with_control_and_input, execute_with_control_observed,
    execute_with_control_observed_and_input,
};
use super::{Case, fixture};
use crate::cli::{
    live_master_recording::observation::{Observation, WindowStopped, WindowStoppedSink},
    live_master_recording::terminal_input::RecordingInput,
    model::{LaunchMode, LiveRecordingInputMode},
};

struct FillInput {
    elapsed: Rc<Cell<Duration>>,
    sent: bool,
    polls: usize,
}

#[derive(Default)]
struct MemorySink(Vec<serde_json::Value>);

impl WindowStoppedSink for MemorySink {
    fn record_window_stopped(&mut self, milestone: &WindowStopped) -> io::Result<()> {
        self.0.push(serde_json::to_value(milestone).unwrap());
        Ok(())
    }
}

#[test]
fn default_recording_keeps_v1_milestone_and_has_no_input_report() {
    let (_temp, launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
    let mut sink = MemorySink::default();
    let mut observation = Observation::new(Some(&mut sink));
    let summary =
        execute_with_control_observed(&launch, &mut state, &mut control, &mut observation).unwrap();
    observation.into_result().unwrap();
    assert_eq!(summary["ready"], true);
    assert!(summary.get("terminal_input").is_none());
    assert_eq!(sink.0.len(), 1);
    let milestone = &sink.0[0];
    assert_eq!(
        milestone["observation_schema"],
        "riotbox.live_master_recording_window_stopped.v1"
    );
    assert_eq!(
        milestone["capture_context"],
        "non_interactive_real_audio_cli"
    );
    assert!(milestone.get("input_mode").is_none());
    assert_eq!(milestone["window_complete"], true);
    assert_eq!(milestone["runtime_stopped"], true);
    assert_eq!(milestone["frame_count"], 4000);
    assert_eq!(milestone["window_complete_monotonic_ns"], 7_000_000_000_u64);
    assert_eq!(milestone["runtime_stopped_monotonic_ns"], 7_000_000_000_u64);
}

#[test]
fn opt_in_milestone_is_not_mislabeled_as_noninteractive_v1() {
    let (_temp, mut launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
    let LaunchMode::LiveMasterRecordingExecute { input_mode, .. } = &mut launch.mode else {
        unreachable!();
    };
    *input_mode = LiveRecordingInputMode::TerminalFill;
    let mut input = FillInput {
        elapsed: control.elapsed.clone(),
        sent: true,
        polls: 0,
    };
    let mut sink = MemorySink::default();
    let mut observation = Observation::new(Some(&mut sink));
    let summary = execute_with_control_observed_and_input(
        &launch,
        &mut state,
        &mut control,
        &mut observation,
        Some(&mut input),
    )
    .unwrap();
    observation.into_result().unwrap();
    assert_eq!(summary["ready"], true);
    assert_eq!(sink.0.len(), 1);
    let milestone = &sink.0[0];
    assert_eq!(
        milestone["observation_schema"],
        "riotbox.live_master_recording_window_stopped.v2"
    );
    assert_eq!(
        milestone["capture_context"],
        "terminal_input_real_audio_cli"
    );
    assert_eq!(milestone["input_mode"], "terminal_fill");
    assert_eq!(milestone["window_complete"], true);
    assert_eq!(milestone["runtime_stopped"], true);
    assert_eq!(
        summary["terminal_input"]["render_delivery"],
        "not_qualified"
    );
}

impl RecordingInput for FillInput {
    fn poll_event(&mut self) -> io::Result<Option<KeyEvent>> {
        self.polls += 1;
        if !self.sent && self.elapsed.get() >= Duration::from_millis(500) {
            self.sent = true;
            return Ok(Some(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE)));
        }
        Ok(None)
    }
}

#[test]
fn pending_recording_accepts_existing_fill_and_persists_its_next_bar_commit() {
    let (_temp, mut launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
    let LaunchMode::LiveMasterRecordingExecute { input_mode, .. } = &mut launch.mode else {
        unreachable!();
    };
    *input_mode = LiveRecordingInputMode::TerminalFill;
    let mut input = FillInput {
        elapsed: control.elapsed.clone(),
        sent: false,
        polls: 0,
    };
    let summary =
        execute_with_control_and_input(&launch, &mut state, &mut control, &mut input).unwrap();
    assert_eq!(summary["ready"], true);
    let fills = state
        .session
        .action_log
        .actions
        .iter()
        .filter(|action| action.command == ActionCommand::Tr909FillNext)
        .collect::<Vec<_>>();
    assert_eq!(
        fills.len(),
        1,
        "the input must reach the real App/Core queue"
    );
    assert!(input.sent);
    assert_eq!(
        input.polls, 600,
        "exactly one input attempt per pending poll"
    );
    let record = state
        .session
        .action_log
        .commit_records
        .iter()
        .find(|record| record.action_id == fills[0].id)
        .unwrap();
    assert_eq!(record.boundary.beat_index, 4);
    assert_eq!(fills[0].committed_at, Some(3_000));
    assert!(state.queue.pending_actions().is_empty());
    let saved =
        riotbox_core::persistence::load_session_json(&state.files.as_ref().unwrap().session_path)
            .unwrap();
    assert!(
        saved
            .action_log
            .actions
            .iter()
            .any(|action| action.id == fills[0].id)
    );
    assert_eq!(saved.export_receipts.len(), 1);
}
