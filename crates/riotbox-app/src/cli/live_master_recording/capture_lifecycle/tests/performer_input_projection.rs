//! Real recorder/App projection with generated trusted Graph metadata only.
//! Composed with the separate Audio callback/CaptureTap control, not an E2E test.
use std::{cell::Cell, cell::RefCell, io, path::Path, rc::Rc, time::Duration, time::Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use riotbox_audio::{
    runtime::{
        AudioRuntimeHealth, AudioRuntimeTimingSnapshot, AudioTransportControlError,
        AudioTransportPositionUpdate, LiveMasterCaptureError, LiveMasterCaptureOutcome,
        LiveMasterCaptureProgress, LiveMasterCaptureRequest,
    },
    tr909::{Tr909FillRecipeId, Tr909RenderMode, Tr909RenderState},
};
use riotbox_core::{
    TimestampMs,
    action::{ActionCommand, ActionStatus, CommitBoundary, LiveRecordingDuration},
    persistence::load_session_json,
    session::{GraphStorageMode, SourceGraphRef, SourceRef},
    source_graph::{ManualSourceTimingGrid, install_manual_source_timing_grid},
};

use super::super::{RuntimeControl, execute_with_control_and_input};
use super::{Case, GeneratedControl, fixture};
use crate::{
    cli::{
        live_master_recording::terminal_input::RecordingInput,
        model::{LaunchMode, LiveRecordingInputMode},
        transport_sync::TransportControl,
    },
    jam_app::{JamAppState, QueueControlResult, tests::sample_graph},
};

struct FillInput {
    elapsed: Rc<Cell<Duration>>,
    sent: bool,
}

impl RecordingInput for FillInput {
    fn poll_event(&mut self) -> io::Result<Option<KeyEvent>> {
        if !self.sent && self.elapsed.get() >= Duration::from_millis(500) {
            self.sent = true;
            return Ok(Some(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE)));
        }
        Ok(None)
    }
}

struct Projection {
    position_beats: f64,
    render: Tr909RenderState,
}

struct ProjectionControl<'a> {
    inner: &'a mut GeneratedControl,
    projections: RefCell<Vec<Projection>>,
}

impl TransportControl for ProjectionControl<'_> {
    fn timing_snapshot(&self) -> Option<AudioRuntimeTimingSnapshot> {
        self.inner.timing_snapshot()
    }

    fn update_transport_state(
        &self,
        playing: bool,
        bpm: f32,
        position: AudioTransportPositionUpdate,
    ) -> Result<(), AudioTransportControlError> {
        self.inner.update_transport_state(playing, bpm, position)
    }
}

impl RuntimeControl for ProjectionControl<'_> {
    fn health_snapshot(&self) -> AudioRuntimeHealth {
        self.inner.health_snapshot()
    }

    fn update_render_states(&self, state: &JamAppState) {
        self.projections.borrow_mut().push(Projection {
            position_beats: state.runtime.transport.position_beats,
            render: state.runtime.tr909_render.clone(),
        });
        self.inner.update_render_states(state);
    }

    fn begin_live_master_capture(
        &self,
        request: LiveMasterCaptureRequest,
    ) -> Result<(), LiveMasterCaptureError> {
        self.inner.begin_live_master_capture(request)
    }

    fn live_master_capture_progress(&self) -> Option<LiveMasterCaptureProgress> {
        self.inner.live_master_capture_progress()
    }

    fn abort_live_master_capture(&self) -> Option<LiveMasterCaptureProgress> {
        self.inner.abort_live_master_capture()
    }

    fn finish_live_master_capture(
        &self,
    ) -> Result<LiveMasterCaptureOutcome, LiveMasterCaptureError> {
        self.inner.finish_live_master_capture()
    }

    fn stop(&mut self) {
        self.inner.stop();
    }

    fn now(&self) -> Instant {
        self.inner.now()
    }

    fn timestamp_now(&self) -> TimestampMs {
        self.inner.timestamp_now()
    }

    fn wait(&self, duration: Duration) {
        self.inner.wait(duration);
    }

    fn monotonic_ns(&self) -> Option<u64> {
        self.inner.monotonic_ns()
    }
}

#[test]
fn pending_fill_projects_one_trusted_bar_and_persists_exact_graph_grid_lineage() {
    let (owner, mut launch, mut state, mut inner) =
        fixture(Case::Clean, LiveRecordingDuration::TwoBars, false);
    install_trusted_metadata(&mut state, owner.path());
    let LaunchMode::LiveMasterRecordingExecute { input_mode, .. } = &mut launch.mode else {
        unreachable!();
    };
    *input_mode = LiveRecordingInputMode::TerminalFill;
    let graph_ref = state.session.source_graph_refs[0].clone();
    let confirmed_grid = state
        .session
        .runtime_state
        .source_timing
        .confirmed_grid
        .clone()
        .unwrap();
    let mut input = FillInput {
        elapsed: inner.elapsed.clone(),
        sent: false,
    };
    let mut control = ProjectionControl {
        inner: &mut inner,
        projections: RefCell::new(Vec::new()),
    };
    let summary =
        execute_with_control_and_input(&launch, &mut state, &mut control, &mut input).unwrap();
    assert_eq!(summary["ready"], true, "{summary}");
    assert!(input.sent);
    let mut observed = [0_usize; 3];
    for projection in control.projections.borrow().iter() {
        if !projection.render.is_transport_running {
            continue;
        }
        assert_eq!(projection.render.tempo_bpm, 120.0);
        if projection.position_beats < 4.0 {
            observed[0] += 1;
            assert_ne!(projection.render.mode, Tr909RenderMode::Fill);
        } else if projection.position_beats < 8.0 {
            observed[1] += 1;
            assert_eq!(projection.render.mode, Tr909RenderMode::Fill);
            assert_eq!(
                projection.render.fill_recipe_id(),
                Some(Tr909FillRecipeId::GenericFillV1)
            );
        } else {
            observed[2] += 1;
            assert_ne!(projection.render.mode, Tr909RenderMode::Fill);
        }
    }
    assert!(observed.iter().all(|count| *count > 0));

    let fills = state
        .session
        .action_log
        .actions
        .iter()
        .filter(|action| action.command == ActionCommand::Tr909FillNext)
        .collect::<Vec<_>>();
    assert_eq!(fills.len(), 1);
    let fill = fills[0];
    assert_eq!(fill.status, ActionStatus::Committed);
    assert!(fill.result.as_ref().unwrap().accepted);
    let record = state
        .session
        .action_log
        .commit_records
        .iter()
        .find(|record| record.action_id == fill.id)
        .unwrap();
    assert_eq!(record.boundary.beat_index, 4);
    assert_eq!(record.boundary.kind, CommitBoundary::Bar);
    assert!(state.queue.pending_actions().is_empty());

    // Only the owned generated Session JSON is read back. Do not use App load,
    // which would attempt source/capture hydration through the full load path.
    let saved = load_session_json(&state.files.as_ref().unwrap().session_path).unwrap();
    assert!(saved.action_log.actions.iter().any(|action| action == fill));
    assert!(
        saved
            .action_log
            .commit_records
            .iter()
            .any(|entry| entry == record)
    );
    assert_eq!(saved.source_graph_refs, vec![graph_ref.clone()]);
    assert_eq!(
        saved.runtime_state.source_timing.confirmed_grid,
        Some(confirmed_grid.clone())
    );
    assert_eq!(saved.export_receipts.len(), 1);
    assert!(
        saved.export_receipts[0]
            .artifact_set
            .iter()
            .any(|artifact| {
                artifact.source_graph_ref.as_ref().is_some_and(|source| {
                    source.source_id == graph_ref.source_id
                        && source.graph_hash == graph_ref.graph_hash
                }) && artifact.timing_grid_ref.as_ref().is_some_and(|timing| {
                    timing.source_id == confirmed_grid.source_id
                        && timing.hypothesis_id == confirmed_grid.hypothesis_id
                        && timing.confirmed_by_action == confirmed_grid.confirmed_by_action
                })
            })
    );
    assert!(state.source_audio_cache.is_none());
    assert!(state.capture_audio_cache.is_empty());
}

fn install_trusted_metadata(state: &mut JamAppState, directory: &Path) {
    let mut graph = sample_graph();
    // The generated descriptor names an absent owned-temp path, never an
    // actual source. No source file is created, opened, decoded or hashed.
    graph.source.path = directory
        .join("never-created-source.wav")
        .to_string_lossy()
        .into_owned();
    install_manual_source_timing_grid(
        &mut graph,
        ManualSourceTimingGrid {
            bpm: 120.0,
            downbeat_seconds: 0.0,
        },
    )
    .unwrap();
    let hypothesis = graph.timing.primary_hypothesis().unwrap();
    assert_eq!(hypothesis.bpm, 120.0);
    assert_eq!(hypothesis.meter.beats_per_bar, 4);
    assert_eq!(hypothesis.meter.beat_unit, 4);
    assert!(graph.timing.primary_hypothesis_id.is_some());
    state.session.source_refs.push(SourceRef {
        source_id: graph.source.source_id.clone(),
        path_hint: graph.source.path.clone(),
        content_hash: graph.source.content_hash.clone(),
        duration_seconds: graph.source.duration_seconds,
        decode_profile: graph.source.decode_profile.clone(),
    });
    state.session.source_graph_refs.push(SourceGraphRef {
        source_id: graph.source.source_id.clone(),
        graph_version: graph.graph_version,
        // Normal App save below owns the actual Graph hash calculation.
        graph_hash: String::new(),
        storage_mode: GraphStorageMode::Embedded,
        embedded_graph: Some(graph.clone()),
        external_path: None,
        provenance: graph.provenance.clone(),
    });
    state.source_graph = Some(graph);
    assert_eq!(
        state.queue_source_timing_grid_confirmation(100).unwrap(),
        QueueControlResult::Enqueued
    );
    let boundary = state
        .runtime
        .transport
        .boundary_state(CommitBoundary::Immediate);
    let committed = state.commit_ready_actions(boundary, 101).unwrap();
    assert_eq!(committed.len(), 1);
    state.save().unwrap();
    // save(&self) prepares a copy. Bring back that normally computed embedded
    // Graph identity through metadata-only Core loading, without hydration.
    state.session = load_session_json(&state.files.as_ref().unwrap().session_path).unwrap();
    assert!(!state.session.source_graph_refs[0].graph_hash.is_empty());
    assert_eq!(
        state.session.source_graph_refs[0].embedded_graph.as_ref(),
        state.source_graph.as_ref()
    );
    let confirmed = state
        .session
        .runtime_state
        .source_timing
        .confirmed_grid
        .as_ref()
        .unwrap();
    assert_eq!(
        confirmed.hypothesis_id,
        state
            .source_graph
            .as_ref()
            .unwrap()
            .timing
            .primary_hypothesis_id
    );
    state.refresh_view();
    assert!(state.source_audio_cache.is_none());
    assert!(state.capture_audio_cache.is_empty());
}
