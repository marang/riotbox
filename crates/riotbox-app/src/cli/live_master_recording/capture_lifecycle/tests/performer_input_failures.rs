//! Source-free acceptance controls through the recorder's real input policy.
use std::{cell::Cell, collections::VecDeque, fs, io, rc::Rc, time::Duration};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use riotbox_core::{
    action::{
        ActionCommand, ActionDraft, ActionStatus, ActionTarget, ActorType, LiveRecordingDuration,
        Quantization,
    },
    ids::ActionId,
};

use super::super::execute_with_control_and_input;
use super::{Case, Event, GeneratedControl, fixture};
use crate::cli::{
    live_master_recording::terminal_input::RecordingInput,
    model::{AppLaunch, LaunchMode, LiveRecordingInputMode},
};

enum ScriptedEvent {
    Key(KeyEvent),
    ReadFailure,
}

struct ScriptedInput {
    elapsed: Rc<Cell<Duration>>,
    events: VecDeque<(Duration, ScriptedEvent)>,
    polls: usize,
}

impl ScriptedInput {
    fn new(
        control: &GeneratedControl,
        events: impl IntoIterator<Item = (u64, ScriptedEvent)>,
    ) -> Self {
        Self {
            elapsed: Rc::clone(&control.elapsed),
            events: events
                .into_iter()
                .map(|(ms, event)| (Duration::from_millis(ms), event))
                .collect(),
            polls: 0,
        }
    }
}

impl RecordingInput for ScriptedInput {
    fn poll_event(&mut self) -> io::Result<Option<KeyEvent>> {
        self.polls += 1;
        if self
            .events
            .front()
            .is_none_or(|(at, _)| *at > self.elapsed.get())
        {
            return Ok(None);
        }
        match self.events.pop_front().unwrap().1 {
            ScriptedEvent::Key(key) => Ok(Some(key)),
            ScriptedEvent::ReadFailure => Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "generated event read failure",
            )),
        }
    }
}

fn enable_input(launch: &mut AppLaunch) {
    let LaunchMode::LiveMasterRecordingExecute { input_mode, .. } = &mut launch.mode else {
        unreachable!();
    };
    *input_mode = LiveRecordingInputMode::TerminalFill;
}

#[test]
fn pending_cancellation_or_read_failure_stops_aborts_and_preserves_saved_session() {
    for event in [
        ScriptedEvent::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
        ScriptedEvent::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        ScriptedEvent::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        ScriptedEvent::ReadFailure,
    ] {
        let (_owner, mut launch, mut state, mut control) =
            fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
        enable_input(&mut launch);
        let path = state.files.as_ref().unwrap().session_path.clone();
        let saved = fs::read(&path).unwrap();
        let read_failure = matches!(event, ScriptedEvent::ReadFailure);
        let mut input = ScriptedInput::new(&control, [(500, event)]);
        let summary =
            execute_with_control_and_input(&launch, &mut state, &mut control, &mut input).unwrap();
        assert_eq!(summary["ready"], false);
        assert_eq!(summary["runtime_stopped"], true);
        let reason = summary["failure_reason"].as_str().unwrap();
        assert!(
            reason.contains(if read_failure {
                "event read failed"
            } else {
                "cancelled while capture was pending"
            }),
            "{reason}"
        );
        assert_eq!(
            *control.events.borrow(),
            [Event::Begin, Event::Stop, Event::Abort, Event::Stop]
        );
        assert_eq!(control.elapsed.get(), Duration::from_millis(500));
        assert!(control.stopped.get());
        assert!(input.events.is_empty());
        assert!(state.queue.pending_actions().is_empty());
        assert!(state.queue.history().iter().any(|action| action.command
            == ActionCommand::ExportLiveRecording
            && action.status == ActionStatus::Rejected));
        assert!(state.session.export_receipts.is_empty());
        // Existing Play/Pause cleanup records are legitimate; persisted Session
        // bytes, not every live transport observation, must remain unchanged.
        assert_eq!(fs::read(&path).unwrap(), saved);
        control.assert_unpublished();
    }
}

#[test]
fn cancellation_retires_only_owned_pending_fill_not_foreign_queue_work() {
    for foreign_fill in [false, true] {
        let (_owner, mut launch, mut state, mut control) =
            fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
        enable_input(&mut launch);
        let foreign = state
            .queue
            .enqueue(
                ActionDraft::new(
                    ActorType::User,
                    if foreign_fill {
                        ActionCommand::Tr909FillNext
                    } else {
                        ActionCommand::Tr909ReinforceBreak
                    },
                    if foreign_fill {
                        Quantization::NextBar
                    } else {
                        Quantization::NextPhrase
                    },
                    ActionTarget::default(),
                ),
                900,
            )
            .unwrap();
        let path = state.files.as_ref().unwrap().session_path.clone();
        let saved = fs::read(&path).unwrap();
        let mut input = ScriptedInput::new(
            &control,
            [
                (
                    500,
                    ScriptedEvent::Key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE)),
                ),
                (
                    750,
                    ScriptedEvent::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
                ),
            ],
        );
        let summary =
            execute_with_control_and_input(&launch, &mut state, &mut control, &mut input).unwrap();
        assert_eq!(summary["ready"], false);
        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, foreign);
        assert!(
            !state
                .queue
                .history()
                .iter()
                .any(|action| action.id == foreign)
        );
        let report = &summary["terminal_input"];
        assert_eq!(report["core_committed_action_ids"], serde_json::json!([]));
        assert_eq!(report["render_delivery"], "not_qualified");
        if foreign_fill {
            assert_eq!(report["queued_action_ids"], serde_json::json!([]));
            assert_eq!(report["not_committed_action_ids"], serde_json::json!([]));
            assert_eq!(report["ignored_event_count"], 1);
        } else {
            let ids = report["queued_action_ids"].as_array().unwrap();
            assert_eq!(ids.len(), 1);
            let owned = ActionId(ids[0].as_u64().unwrap());
            assert_ne!(owned, foreign);
            assert_eq!(
                report["not_committed_action_ids"],
                serde_json::json!([owned])
            );
            assert_eq!(report["ignored_event_count"], 0);
            assert_eq!(
                state.queue.history_action(owned).unwrap().status,
                ActionStatus::Rejected
            );
            assert!(
                !state
                    .session
                    .action_log
                    .actions
                    .iter()
                    .any(|action| action.id == owned)
            );
        }
        assert!(
            !state
                .session
                .action_log
                .actions
                .iter()
                .any(|action| action.id == foreign)
        );
        assert!(state.session.export_receipts.is_empty());
        assert_eq!(fs::read(&path).unwrap(), saved);
        control.assert_unpublished();
    }
}

#[test]
fn only_unmodified_fill_press_enqueues_and_pending_or_takeover_fill_is_ignored() {
    for takeover in [false, true] {
        let (_owner, mut launch, mut state, mut control) =
            fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
        enable_input(&mut launch);
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .takeover_enabled = takeover;
        let mut events = [
            KeyEvent {
                kind: KeyEventKind::Release,
                ..KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE)
            },
            KeyEvent {
                kind: KeyEventKind::Repeat,
                ..KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE)
            },
            KeyEvent::new(KeyCode::Char('F'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('f'), KeyModifiers::SHIFT),
            KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('f'), KeyModifiers::ALT),
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::SHIFT),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::ALT),
            KeyEvent::new(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            ),
        ]
        .into_iter()
        .map(|key| (0, ScriptedEvent::Key(key)))
        .collect::<Vec<_>>();
        events.extend([
            (
                1_000,
                ScriptedEvent::Key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE)),
            ),
            (
                1_010,
                ScriptedEvent::Key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE)),
            ),
            (
                1_500,
                ScriptedEvent::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
            ),
        ]);
        let mut input = ScriptedInput::new(&control, events);
        let summary =
            execute_with_control_and_input(&launch, &mut state, &mut control, &mut input).unwrap();
        assert_eq!(summary["ready"], false);
        assert!(
            summary["failure_reason"]
                .as_str()
                .unwrap()
                .contains("cancelled")
        );
        assert_eq!(control.elapsed.get(), Duration::from_millis(1_500));
        let report = &summary["terminal_input"];
        let queued = report["queued_action_ids"].as_array().unwrap();
        assert_eq!(queued.len(), usize::from(!takeover));
        assert_eq!(
            report["not_committed_action_ids"],
            report["queued_action_ids"]
        );
        assert_eq!(report["core_committed_action_ids"], serde_json::json!([]));
        assert_eq!(
            report["ignored_event_count"],
            if takeover { 11 } else { 10 }
        );
        assert!(input.events.is_empty());
        assert!(state.queue.pending_actions().is_empty());
        assert!(
            !state
                .session
                .action_log
                .actions
                .iter()
                .any(|action| action.command == ActionCommand::Tr909FillNext)
        );
        control.assert_unpublished();
    }
}

#[test]
fn terminal_completion_or_fault_wins_before_an_unread_input_event() {
    for (case, terminal_ms, ready) in [
        (Case::Clean, 6_000, true),
        (Case::CompleteAtDeadline, 8_000, true),
        (Case::FaultAndComplete, 0, false),
    ] {
        let (_owner, mut launch, mut state, mut control) =
            fixture(case, LiveRecordingDuration::TwoBars, false);
        enable_input(&mut launch);
        let mut input = ScriptedInput::new(
            &control,
            [(
                terminal_ms,
                ScriptedEvent::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
            )],
        );
        let summary =
            execute_with_control_and_input(&launch, &mut state, &mut control, &mut input).unwrap();
        assert_eq!(summary["ready"], ready);
        assert_eq!(summary["runtime_stopped"], true);
        assert_eq!(control.elapsed.get(), Duration::from_millis(terminal_ms));
        assert_eq!(input.polls, terminal_ms as usize / 10);
        assert_eq!(
            input.events.len(),
            1,
            "the terminal poll must not consume the ready cancellation"
        );
        assert_eq!(summary["terminal_input"]["ignored_event_count"], 0);
        assert_eq!(
            summary["terminal_input"]["queued_action_ids"],
            serde_json::json!([])
        );
        if ready {
            assert_eq!(
                *control.events.borrow(),
                [Event::Begin, Event::Stop, Event::Finish]
            );
            assert_eq!(state.session.export_receipts.len(), 1);
        } else {
            assert!(
                summary["failure_reason"]
                    .as_str()
                    .unwrap()
                    .contains("runtime faulted")
            );
            assert_eq!(
                *control.events.borrow(),
                [Event::Begin, Event::Stop, Event::Abort, Event::Stop]
            );
            assert!(state.session.export_receipts.is_empty());
            control.assert_unpublished();
        }
    }
}

#[test]
fn acquired_fill_allocation_failure_still_stops_when_cleanup_pause_cannot_allocate() {
    let (_owner, mut launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
    enable_input(&mut launch);
    // The final two IDs are consumed by Play and the recording request. Fill
    // then fails only after capture acquisition; the cleanup Pause fails too.
    state
        .queue
        .reserve_action_ids_after(Some(ActionId(u64::MAX - 2)));
    let path = state.files.as_ref().unwrap().session_path.clone();
    let saved = fs::read(&path).unwrap();
    let mut input = ScriptedInput::new(
        &control,
        [(
            0,
            ScriptedEvent::Key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE)),
        )],
    );
    let summary =
        execute_with_control_and_input(&launch, &mut state, &mut control, &mut input).unwrap();
    assert_eq!(summary["ready"], false);
    assert_eq!(summary["runtime_stopped"], true);
    let reason = summary["failure_reason"].as_str().unwrap();
    assert!(reason.contains("Fill could not be queued"), "{reason}");
    assert!(reason.contains("action ID space exhausted"), "{reason}");
    assert!(reason.contains("transport cleanup failed"), "{reason}");
    assert_eq!(
        *control.events.borrow(),
        [Event::Begin, Event::Stop, Event::Abort, Event::Stop]
    );
    assert!(control.request.borrow().is_some());
    assert!(control.stopped.get());
    assert!(
        state.runtime.transport.is_playing,
        "Pause must not acquire a fabricated commit"
    );
    assert_eq!(control.elapsed.get(), Duration::ZERO);
    assert_eq!(input.polls, 1);
    assert!(state.queue.pending_actions().is_empty());
    assert!(state.queue.clone().allocate_action_id().is_err());
    assert_eq!(state.session.action_log.actions.len(), 1);
    assert_eq!(
        state.session.action_log.actions[0].command,
        ActionCommand::TransportPlay
    );
    assert_eq!(
        state.session.action_log.actions[0].id,
        ActionId(u64::MAX - 1)
    );
    assert_eq!(
        summary["terminal_input"]["queued_action_ids"],
        serde_json::json!([])
    );
    assert!(state.session.export_receipts.is_empty());
    assert_eq!(fs::read(&path).unwrap(), saved);
    control.assert_unpublished();
}

#[test]
fn late_fill_at_capture_end_remains_core_committed_without_render_qualification() {
    let (_owner, mut launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
    enable_input(&mut launch);
    let mut input = ScriptedInput::new(
        &control,
        [(
            5_990,
            ScriptedEvent::Key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE)),
        )],
    );
    let summary =
        execute_with_control_and_input(&launch, &mut state, &mut control, &mut input).unwrap();
    assert_eq!(summary["ready"], true);
    let report = &summary["terminal_input"];
    let queued = report["queued_action_ids"].as_array().unwrap();
    assert_eq!(queued.len(), 1);
    let id = ActionId(queued[0].as_u64().unwrap());
    assert_eq!(report["core_committed_action_ids"], serde_json::json!([id]));
    assert_eq!(report["not_committed_action_ids"], serde_json::json!([]));
    assert_eq!(report["ignored_event_count"], 0);
    assert_eq!(report["render_delivery"], "not_qualified");
    let action = state
        .session
        .action_log
        .actions
        .iter()
        .find(|action| action.id == id)
        .unwrap();
    assert_eq!(action.status, ActionStatus::Committed);
    assert_eq!(action.committed_at, Some(7_000));
    let record = state
        .session
        .action_log
        .commit_records
        .iter()
        .find(|record| record.action_id == id)
        .unwrap();
    assert_eq!(record.boundary.beat_index, 12);
    // The generated capture is [4,12): a Core commit at its exclusive end is
    // real committed history, not evidence that this Fill reached captured PCM.
    let proof_path =
        crate::jam_app::live_master_recording_proof_path(&control.destination).unwrap();
    let proof: serde_json::Value = serde_json::from_slice(&fs::read(proof_path).unwrap()).unwrap();
    assert_eq!(proof["captured_start_position_microbeats"], 4_000_000);
    assert_eq!(proof["captured_end_position_microbeats"], 12_000_000);
    let saved =
        riotbox_core::persistence::load_session_json(&state.files.as_ref().unwrap().session_path)
            .unwrap();
    assert!(
        saved
            .action_log
            .actions
            .iter()
            .any(|action| action.id == id && action.status == ActionStatus::Committed)
    );
    assert!(state.queue.pending_actions().is_empty());
}
