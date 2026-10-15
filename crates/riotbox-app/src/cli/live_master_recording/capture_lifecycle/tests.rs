//! Generated control observations only. Never constructs an audio runtime.
use std::{
    cell::{Cell, RefCell},
    fs,
    rc::Rc,
    time::{Duration, Instant},
};

use super::{RuntimeControl, execute_with_control};
use crate::{
    cli::model::{AppLaunch, LaunchMode},
    jam_app::{JamAppState, JamFileSet},
};
use riotbox_audio::runtime::{
    AudioOutputInfo, AudioRuntimeHealth, AudioRuntimeLifecycle, AudioRuntimeTimingSnapshot,
    LiveMasterCaptureError, LiveMasterCaptureOutcome, LiveMasterCaptureProgress,
    LiveMasterCaptureRequest,
};
use riotbox_core::{
    TimestampMs,
    action::{ActionCommand, LiveRecordingDuration},
    ids::SceneId,
    persistence::{load_session_json, save_session_json},
    queue::ActionQueue,
    session::SessionFile,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Case {
    DestinationOccupied,
    Clean,
    ConfirmTimeout,
    ConfirmDelayed,
    ConfirmFault,
    WrongBpm,
    ArmFailure,
    RuntimeFault,
    ScratchFault,
    ProgressFault,
    MissingProgress,
    IncompleteProgress,
    CompleteAtDeadline,
    FaultAndComplete,
    StopUnconfirmed,
    FinishFailure,
    LateFault,
    MissingCompletionClock,
    MissingStoppedClock,
    MissingBothClocks,
    ReversedClock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Event {
    Begin,
    Stop,
    Abort,
    Finish,
    CompletionClock,
    StoppedClock,
    Milestone,
}

struct GeneratedControl {
    case: Case,
    origin: Instant,
    elapsed: Cell<Duration>,
    playing: Cell<bool>,
    stopped: Cell<bool>,
    request: RefCell<Option<LiveMasterCaptureRequest>>,
    events: Rc<RefCell<Vec<Event>>>,
    destination: std::path::PathBuf,
}

impl GeneratedControl {
    fn output(&self) -> AudioOutputInfo {
        AudioOutputInfo {
            host_name: "generated-control".into(),
            device_name: "not-a-device".into(),
            sample_format: "F32".into(),
            sample_rate: 1000,
            channel_count: 2,
            buffer_size: "Fixed(64)".into(),
            supported_output_config_count: Some(1),
        }
    }

    fn complete(&self) -> bool {
        let request = self.request.borrow();
        let Some(request) = request.as_ref() else {
            return false;
        };
        match self.case {
            Case::MissingProgress | Case::IncompleteProgress => false,
            Case::FaultAndComplete => true,
            Case::CompleteAtDeadline => self.elapsed.get() >= Duration::from_secs(8),
            _ => {
                let end_beats = request.start_position_beats.unwrap()
                    + request.target_frame_count as f64 * 120.0 / 60.0 / 1000.0;
                self.elapsed.get().as_secs_f64() >= end_beats / 2.0
            }
        }
    }

    fn assert_unpublished(&self) {
        if self.case == Case::DestinationOccupied {
            assert_eq!(
                fs::read(&self.destination).unwrap(),
                b"foreign generated control"
            );
        } else {
            assert!(
                !self.destination.exists(),
                "device operation must precede publication"
            );
        }
        assert!(
            !crate::jam_app::live_master_recording_proof_path(&self.destination)
                .unwrap()
                .exists()
        );
    }
}

impl RuntimeControl for GeneratedControl {
    fn health_snapshot(&self) -> AudioRuntimeHealth {
        let capturing = self.request.borrow().is_some();
        let stream_fault = self.case == Case::ConfirmFault
            || (capturing && matches!(self.case, Case::RuntimeFault | Case::FaultAndComplete))
            || (self.case == Case::LateFault && self.stopped.get());
        let scratch_fault = capturing && self.case == Case::ScratchFault;
        AudioRuntimeHealth {
            lifecycle: if self.stopped.get() {
                AudioRuntimeLifecycle::Stopped
            } else if stream_fault {
                AudioRuntimeLifecycle::Faulted
            } else {
                AudioRuntimeLifecycle::Running
            },
            output: Some(self.output()),
            callback_count: 80,
            max_callback_gap_micros: Some(12_000),
            callback_scratch_overflow_count: u64::from(scratch_fault),
            stream_error_count: u64::from(stream_fault),
            last_stream_error: stream_fault.then(|| "generated stream fault".into()),
        }
    }
    fn timing_snapshot(&self) -> AudioRuntimeTimingSnapshot {
        AudioRuntimeTimingSnapshot {
            is_transport_running: self.playing.get()
                && !matches!(self.case, Case::ConfirmTimeout | Case::ConfirmFault)
                && (self.case != Case::ConfirmDelayed
                    || self.elapsed.get() >= Duration::from_millis(100)),
            tempo_bpm: if self.case == Case::WrongBpm {
                f32::from_bits(120.0_f32.to_bits() + 1)
            } else {
                120.0
            },
            position_beats: self.elapsed.get().as_secs_f64() * 2.0,
        }
    }
    fn update_transport_state(&self, playing: bool, _bpm: f32, _position: f64) {
        self.playing.set(playing);
    }
    fn update_render_states(&self, _state: &JamAppState) {}
    fn begin_live_master_capture(
        &self,
        request: LiveMasterCaptureRequest,
    ) -> Result<(), LiveMasterCaptureError> {
        self.events.borrow_mut().push(Event::Begin);
        self.assert_unpublished();
        if self.case == Case::ArmFailure {
            return Err(LiveMasterCaptureError::AllocationFailed);
        }
        *self.request.borrow_mut() = Some(request);
        Ok(())
    }
    fn live_master_capture_progress(&self) -> Option<LiveMasterCaptureProgress> {
        if self.case == Case::MissingProgress {
            return None;
        }
        let request = self.request.borrow();
        let request = request.as_ref()?;
        let target = request.target_frame_count * usize::from(request.channel_count);
        let complete = self.complete();
        Some(LiveMasterCaptureProgress {
            target_sample_count: target,
            written_sample_count: if complete { target } else { 0 },
            callback_count: 80,
            max_callback_gap_micros: Some(12_000),
            callback_gap_over_threshold_count: 0,
            callback_scratch_overflow_count: 0,
            stream_error_count: u64::from(
                self.case == Case::ProgressFault
                    || (self.case == Case::LateFault && self.stopped.get()),
            ),
            transport_mismatch_count: 0,
            tempo_mismatch_count: 0,
            timing_window_mismatch_count: 0,
            armed_callback_count: 1,
            capture_started: true,
            complete,
        })
    }
    fn abort_live_master_capture(&self) -> Option<LiveMasterCaptureProgress> {
        self.events.borrow_mut().push(Event::Abort);
        self.assert_unpublished();
        self.live_master_capture_progress()
    }
    fn finish_live_master_capture(
        &self,
    ) -> Result<LiveMasterCaptureOutcome, LiveMasterCaptureError> {
        self.events.borrow_mut().push(Event::Finish);
        assert!(self.stopped.get(), "finalization must follow stream stop");
        self.assert_unpublished();
        let progress = self.live_master_capture_progress().unwrap();
        if self.case == Case::FinishFailure {
            return Err(LiveMasterCaptureError::CallbackStillActive(progress));
        }
        let request = self.request.borrow();
        let request = request.as_ref().unwrap();
        let start = request.start_position_beats.unwrap();
        Ok(LiveMasterCaptureOutcome {
            samples: (0..progress.target_sample_count)
                .map(|i| if i % 2 == 0 { 0.25 } else { -0.25 })
                .collect(),
            progress,
            captured_start_position_beats: Some(start),
            captured_end_position_beats: Some(
                start
                    + request.target_frame_count as f64 * f64::from(request.expected_tempo_bpm)
                        / 60.0
                        / 1000.0,
            ),
        })
    }
    fn stop(&mut self) {
        self.events.borrow_mut().push(Event::Stop);
        self.assert_unpublished();
        if self.case != Case::StopUnconfirmed {
            self.stopped.set(true);
        }
    }
    fn now(&self) -> Instant {
        self.origin + self.elapsed.get()
    }
    fn timestamp_now(&self) -> TimestampMs {
        1000 + self.elapsed.get().as_millis() as u64
    }
    fn wait(&self, duration: Duration) {
        assert_eq!(duration, Duration::from_millis(10));
        self.elapsed.set(self.elapsed.get() + duration);
        assert!(
            self.elapsed.get() <= Duration::from_secs(40),
            "generated loop must be bounded"
        );
    }
    fn monotonic_ns(&self) -> Option<u64> {
        let stopped = self.stopped.get();
        self.events.borrow_mut().push(if stopped {
            Event::StoppedClock
        } else {
            Event::CompletionClock
        });
        if self.case == Case::MissingBothClocks
            || (self.case == Case::MissingCompletionClock && !stopped)
            || (self.case == Case::MissingStoppedClock && stopped)
        {
            return None;
        }
        let time = 1_000_000_000 + u64::try_from(self.elapsed.get().as_nanos()).unwrap();
        Some(if self.case == Case::ReversedClock && stopped {
            time - 1
        } else {
            time
        })
    }
}

fn fixture(
    case: Case,
    duration: LiveRecordingDuration,
    already_playing: bool,
) -> (tempfile::TempDir, AppLaunch, JamAppState, GeneratedControl) {
    let temp = tempfile::tempdir().unwrap();
    let mut session = SessionFile::new("generated-lifecycle", "test", "2026-10-13T00:00:00Z");
    session.runtime_state.source_timing.confirmed_bpm = Some(120.0);
    session.runtime_state.scene_state.scenes = vec![SceneId::from("scene-generated")];
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-generated"));
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-generated"));
    session.runtime_state.transport.is_playing = already_playing;
    let session_path = temp.path().join("session.json");
    save_session_json(&session_path, &session).unwrap();
    let mut state = JamAppState::from_parts(session, None, ActionQueue::new());
    state.files = Some(JamFileSet {
        session_path: session_path.clone(),
        source_graph_path: None,
    });
    let destination = temp.path().join("generated.wav");
    if case == Case::DestinationOccupied {
        fs::write(&destination, b"foreign generated control").unwrap();
    }
    let launch = AppLaunch {
        mode: LaunchMode::LiveMasterRecordingExecute {
            session_path,
            source_graph_path: None,
            destination_path: destination.clone(),
            duration,
        },
        observer_path: None,
    };
    let control = GeneratedControl {
        case,
        origin: Instant::now(),
        elapsed: Cell::new(Duration::ZERO),
        playing: Cell::new(already_playing),
        stopped: Cell::new(false),
        request: RefCell::new(None),
        events: Rc::new(RefCell::new(Vec::new())),
        destination,
    };
    (temp, launch, state, control)
}

mod observation;

fn assert_blocked(summary: &serde_json::Value, state: &JamAppState, control: &GeneratedControl) {
    assert_eq!(summary["ready"], false);
    assert_eq!(summary["status"], "blocked");
    assert!(state.session.export_receipts.is_empty());
    control.assert_unpublished();
    assert!(control.events.borrow().contains(&Event::Stop));
}

#[test]
fn generated_2_8_16_bar_runs_use_actual_queue_publish_save_and_restore() {
    for duration in [
        LiveRecordingDuration::TwoBars,
        LiveRecordingDuration::EightBars,
        LiveRecordingDuration::SixteenBars,
    ] {
        let (_temp, launch, mut state, mut control) = fixture(Case::Clean, duration, false);
        let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
        assert_eq!(summary["ready"], true);
        assert_eq!(summary["duration_bars"], duration.bars());
        assert_eq!(summary["runtime_stopped"], true);
        assert_eq!(
            *control.events.borrow(),
            [Event::Begin, Event::Stop, Event::Finish]
        );
        assert!(!state.runtime.transport.is_playing);
        assert!(control.destination.is_file());
        let restored = load_session_json(&state.files.as_ref().unwrap().session_path).unwrap();
        assert_eq!(restored.export_receipts, state.session.export_receipts);
        assert_eq!(restored.export_receipts.len(), 1);
        assert!(restored.export_receipts[0].live_recording_runtime_master_ready());
    }
}

#[test]
fn already_playing_transport_is_not_given_an_invented_pause() {
    let (_temp, launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, true);
    let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
    assert_eq!(summary["ready"], true);
    assert!(state.runtime.transport.is_playing);
    assert!(control.playing.get());
    assert_eq!(state.session.action_log.actions.len(), 1);
    assert_eq!(
        state.session.action_log.actions[0].command,
        ActionCommand::ExportLiveRecording
    );
}

#[test]
fn delayed_confirmation_and_timing_updates_still_use_actual_core_action_commits() {
    use riotbox_core::action::{
        ActionDraft, ActionParams, ActionStatus, ActionTarget, ActorType, CaptureLengthIntent,
        Quantization, TargetScope,
    };
    let (_temp, launch, mut state, mut control) =
        fixture(Case::ConfirmDelayed, LiveRecordingDuration::TwoBars, true);
    let mut draft = ActionDraft::new(
        ActorType::User,
        ActionCommand::CaptureSetLength,
        Quantization::Immediate,
        ActionTarget {
            scope: Some(TargetScope::Session),
            ..Default::default()
        },
    );
    draft.params = ActionParams::CaptureLength {
        intent: Some(CaptureLengthIntent::OneBar),
    };
    let queued = state.queue.enqueue(draft, 900);
    let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
    assert_eq!(summary["ready"], true);
    assert_eq!(
        state.session.runtime_state.capture.length_intent,
        CaptureLengthIntent::OneBar
    );
    let committed = state
        .session
        .action_log
        .actions
        .iter()
        .find(|action| action.id == queued)
        .unwrap();
    assert_eq!(committed.status, ActionStatus::Committed);
    assert!(control.elapsed.get() >= Duration::from_millis(100));
    assert!(state.queue.pending_actions().is_empty());
}

#[test]
fn confirmation_requires_exact_bpm_and_bounds_fault_or_missing_confirmation() {
    for case in [Case::ConfirmTimeout, Case::WrongBpm, Case::ConfirmFault] {
        let (_temp, launch, mut state, mut control) =
            fixture(case, LiveRecordingDuration::TwoBars, false);
        let error = execute_with_control(&launch, &mut state, &mut control).unwrap_err();
        assert!(error.to_string().contains(if case == Case::ConfirmFault {
            "faulted before"
        } else {
            "did not confirm"
        }));
        assert_eq!(
            control.elapsed.get(),
            if case == Case::ConfirmFault {
                Duration::ZERO
            } else {
                Duration::from_secs(2)
            }
        );
        assert_eq!(*control.events.borrow(), [Event::Stop]);
        assert!(!control.playing.get());
        control.assert_unpublished();
    }
}

#[test]
fn arm_failure_stops_without_aborting_a_capture_it_did_not_acquire() {
    let (_temp, launch, mut state, mut control) =
        fixture(Case::ArmFailure, LiveRecordingDuration::TwoBars, false);
    let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
    assert_blocked(&summary, &state, &control);
    assert!(
        summary["failure_reason"]
            .as_str()
            .unwrap()
            .contains("could not start")
    );
    assert_eq!(*control.events.borrow(), [Event::Begin, Event::Stop]);
    assert!(state.queue.pending_actions().is_empty());
}

#[test]
fn runtime_and_capture_faults_stop_abort_and_reject_before_any_publication() {
    for case in [
        Case::RuntimeFault,
        Case::ScratchFault,
        Case::ProgressFault,
        Case::FaultAndComplete,
    ] {
        let (_temp, launch, mut state, mut control) =
            fixture(case, LiveRecordingDuration::TwoBars, false);
        let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
        assert_blocked(&summary, &state, &control);
        let events = control.events.borrow();
        assert!(events.contains(&Event::Abort));
        assert!(!events.contains(&Event::Finish));
        assert_eq!(events[1], Event::Stop);
        assert_eq!(events[2], Event::Abort);
        assert!(state.queue.pending_actions().is_empty());
        if case == Case::FaultAndComplete {
            assert!(
                summary["failure_reason"]
                    .as_str()
                    .unwrap()
                    .contains("runtime faulted")
            );
        }
    }
}

#[test]
fn missing_or_incomplete_progress_expires_at_existing_arm_window_deadline() {
    for case in [Case::MissingProgress, Case::IncompleteProgress] {
        let (_temp, launch, mut state, mut control) =
            fixture(case, LiveRecordingDuration::TwoBars, false);
        let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
        assert_blocked(&summary, &state, &control);
        assert_eq!(control.elapsed.get(), Duration::from_secs(8));
        assert!(
            summary["failure_reason"]
                .as_str()
                .unwrap()
                .contains("stopped early")
        );
        assert!(!control.events.borrow().contains(&Event::Finish));
    }
}

#[test]
fn completion_at_the_deadline_still_wins_over_timeout() {
    let (_temp, launch, mut state, mut control) = fixture(
        Case::CompleteAtDeadline,
        LiveRecordingDuration::TwoBars,
        false,
    );
    let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
    assert_eq!(summary["ready"], true);
    assert_eq!(control.elapsed.get(), Duration::from_secs(8));
    assert_eq!(
        *control.events.borrow(),
        [Event::Begin, Event::Stop, Event::Finish]
    );
}

#[test]
fn unconfirmed_stop_or_finalize_failure_never_admits_files() {
    for case in [Case::StopUnconfirmed, Case::FinishFailure] {
        let (_temp, launch, mut state, mut control) =
            fixture(case, LiveRecordingDuration::TwoBars, false);
        let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
        assert_blocked(&summary, &state, &control);
        if case == Case::StopUnconfirmed {
            assert_eq!(
                summary["runtime_stopped"], false,
                "unconfirmed cleanup must not be reported as verified"
            );
            assert!(!control.events.borrow().contains(&Event::Finish));
            assert!(
                summary["failure_reason"]
                    .as_str()
                    .unwrap()
                    .contains("did not stop")
            );
        } else {
            assert!(
                summary["failure_reason"]
                    .as_str()
                    .unwrap()
                    .contains("did not finalize")
            );
        }
    }
}

#[test]
fn late_capture_fault_is_not_erased_by_the_pre_stop_health_snapshot() {
    let (_temp, launch, mut state, mut control) =
        fixture(Case::LateFault, LiveRecordingDuration::TwoBars, false);
    let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
    assert_blocked(&summary, &state, &control);
    assert!(control.events.borrow().contains(&Event::Finish));
    assert!(
        summary["failure_reason"]
            .as_str()
            .unwrap()
            .contains("incomplete or faulted")
    );
}

#[test]
fn queue_rejection_and_existing_pending_action_stop_without_capture() {
    for already_pending in [false, true] {
        let case = if already_pending {
            Case::Clean
        } else {
            Case::DestinationOccupied
        };
        let (_temp, launch, mut state, mut control) =
            fixture(case, LiveRecordingDuration::TwoBars, false);
        if already_pending {
            let previous = control.destination.with_file_name("previous.wav");
            assert!(matches!(
                state.queue_live_master_recording(100, &control.output(), previous),
                crate::jam_app::LiveMasterRecordingQueueResult::Enqueued(_)
            ));
        }
        let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
        assert_blocked(&summary, &state, &control);
        assert_eq!(*control.events.borrow(), [Event::Stop]);
        assert_eq!(
            state.queue.pending_actions().len(),
            usize::from(already_pending)
        );
    }
}

#[test]
fn session_save_failure_uses_existing_rollback_and_preserves_foreign_targets() {
    let (temp, launch, mut state, mut control) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
    let original = fs::read(&state.files.as_ref().unwrap().session_path).unwrap();
    state.files.as_mut().unwrap().session_path = temp.path().to_path_buf();
    let summary = execute_with_control(&launch, &mut state, &mut control).unwrap();
    assert_blocked(&summary, &state, &control);
    assert!(
        summary["failure_reason"]
            .as_str()
            .unwrap()
            .contains("Session save failed")
    );
    assert!(temp.path().is_dir());
    assert_eq!(
        fs::read(temp.path().join("session.json")).unwrap(),
        original
    );
}
