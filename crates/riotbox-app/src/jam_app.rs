use std::{
    collections::BTreeMap,
    error::Error,
    fmt::{self, Display, Formatter},
    io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use riotbox_audio::{
    mc202::Mc202RenderState,
    runtime::{AudioRuntimeHealth, AudioRuntimeTimingSnapshot, render_w30_resample_tap_offline},
    source_audio::{SourceAudioCache, SourceAudioWindow, write_interleaved_pcm16_wav},
    tr909::Tr909RenderState,
    w30::{
        W30PreviewRenderState, W30ResampleTapMode, W30ResampleTapRouting,
        W30ResampleTapSourceProfile, W30ResampleTapState,
    },
};
use riotbox_core::{
    TimestampMs,
    action::{
        Action, ActionCommand, ActionDraft, ActionParams, ActionResult, ActionStatus, ActionTarget,
        ActorType, Quantization, TargetScope,
    },
    ids::{ActionId, CaptureId, SourceId},
    persistence::{
        PersistenceError, load_session_json, load_source_graph_json, save_session_json,
        save_source_graph_json,
    },
    queue::{ActionQueue, CommittedActionRef},
    session::{
        CaptureRef, CaptureTarget, GraphStorageMode, Mc202UndoSnapshotState, SessionFile,
        SourceGraphRef, SourceRef, Tr909TakeoverProfileState,
    },
    source_graph::{DecodeProfile, SourceGraph},
    transport::{CommitBoundaryState, TransportClockState},
    view::jam::JamViewModel,
};
use riotbox_sidecar::client::{ClientError as SidecarClientError, StdioSidecarClient};
use sha2::{Digest, Sha256};

mod capture_helpers;
mod persistence;
mod projection;
mod runtime_view;
mod scene_ops;
mod side_effects;
mod transport_helpers;
mod w30_targets;

use capture_helpers::{
    apply_capture_promotion_side_effects, capture_promotion_summary, capture_ref_from_action,
    capture_targets_specific_w30_pad, capture_targets_w30_pad,
};
use projection::{
    build_mc202_render_state, build_tr909_render_state, build_w30_preview_render_state,
    build_w30_resample_tap_state, normalize_w30_preview_mode,
};
pub use runtime_view::JamRuntimeView;
use side_effects::{
    apply_mc202_side_effects, apply_scene_side_effects, apply_tr909_side_effects,
    apply_w30_side_effects,
};
use transport_helpers::{
    crossed_commit_boundary, normalize_scene_candidates, transport_clock_for_state,
    transport_clock_from_state,
};

#[derive(Debug)]
pub enum JamAppError {
    Io(io::Error),
    Persistence(PersistenceError),
    Serialization(serde_json::Error),
    Sidecar(SidecarClientError),
    InvalidSession(String),
}

impl Display for JamAppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Persistence(error) => write!(f, "persistence error: {error}"),
            Self::Serialization(error) => write!(f, "serialization error: {error}"),
            Self::Sidecar(error) => write!(f, "sidecar error: {error}"),
            Self::InvalidSession(message) => write!(f, "invalid session: {message}"),
        }
    }
}

impl Error for JamAppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Persistence(error) => Some(error),
            Self::Serialization(error) => Some(error),
            Self::Sidecar(error) => Some(error),
            Self::InvalidSession(_) => None,
        }
    }
}

impl From<io::Error> for JamAppError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<PersistenceError> for JamAppError {
    fn from(value: PersistenceError) -> Self {
        Self::Persistence(value)
    }
}

impl From<serde_json::Error> for JamAppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value)
    }
}

impl From<SidecarClientError> for JamAppError {
    fn from(value: SidecarClientError) -> Self {
        Self::Sidecar(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JamFileSet {
    pub session_path: PathBuf,
    pub source_graph_path: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppRuntimeState {
    pub audio: Option<AudioRuntimeHealth>,
    pub sidecar: SidecarState,
    pub transport: TransportClockState,
    pub transport_driver: TransportDriverState,
    pub tr909_render: Tr909RenderState,
    pub mc202_render: Mc202RenderState,
    pub w30_preview: W30PreviewRenderState,
    pub w30_resample_tap: W30ResampleTapState,
    pub last_commit_boundary: Option<CommitBoundaryState>,
}

impl Default for AppRuntimeState {
    fn default() -> Self {
        Self {
            audio: None,
            sidecar: SidecarState::Unknown,
            transport: TransportClockState::default(),
            transport_driver: TransportDriverState::default(),
            tr909_render: Tr909RenderState::default(),
            mc202_render: Mc202RenderState::default(),
            w30_preview: W30PreviewRenderState::default(),
            w30_resample_tap: W30ResampleTapState::default(),
            last_commit_boundary: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TransportDriverState {
    pub last_audio_position_beats: Option<u64>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum QueueControlResult {
    Enqueued,
    AlreadyPending,
    AlreadyInState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SidecarState {
    Unknown,
    Ready {
        version: Option<String>,
        transport: String,
    },
    Unavailable {
        reason: String,
    },
    Degraded {
        reason: String,
    },
}

#[derive(Clone, Debug)]
pub struct JamAppState {
    pub files: Option<JamFileSet>,
    pub session: SessionFile,
    pub source_graph: Option<SourceGraph>,
    pub source_audio_cache: Option<SourceAudioCache>,
    pub capture_audio_cache: BTreeMap<CaptureId, SourceAudioCache>,
    pub queue: ActionQueue,
    pub runtime: AppRuntimeState,
    pub jam_view: JamViewModel,
    pub runtime_view: JamRuntimeView,
}

struct W30BusPrintInput {
    sample_rate: u32,
    channel_count: u16,
    samples: Vec<f32>,
}

impl JamAppState {
    const W30_DAMAGE_PROFILE_LABEL: &str = "shred";
    const W30_DAMAGE_PROFILE_GRIT: f32 = 0.82;
    const W30_LOOP_FREEZE_LABEL: &str = "freeze";

    #[must_use]
    pub fn from_parts(
        mut session: SessionFile,
        source_graph: Option<SourceGraph>,
        mut queue: ActionQueue,
    ) -> Self {
        normalize_w30_preview_mode(&mut session);
        normalize_scene_candidates(&mut session, source_graph.as_ref());
        queue.reserve_action_ids_after(max_action_id(&session));
        let transport = transport_clock_from_state(&session, source_graph.as_ref());
        let jam_view = JamViewModel::build(&session, &queue, source_graph.as_ref());
        let runtime_view =
            JamRuntimeView::build(&AppRuntimeState::default(), &session, source_graph.as_ref());
        let mut state = Self {
            files: None,
            session,
            source_graph,
            source_audio_cache: None,
            capture_audio_cache: BTreeMap::new(),
            queue,
            runtime: AppRuntimeState {
                transport,
                ..AppRuntimeState::default()
            },
            jam_view,
            runtime_view,
        };
        state.refresh_view();
        state
    }

    pub fn refresh_view(&mut self) {
        self.runtime.tr909_render = build_tr909_render_state(
            &self.session,
            &self.runtime.transport,
            self.source_graph.as_ref(),
        );
        self.runtime.mc202_render = build_mc202_render_state(
            &self.session,
            &self.runtime.transport,
            self.source_graph.as_ref(),
        );
        self.runtime.w30_preview = build_w30_preview_render_state(
            &self.session,
            &self.runtime.transport,
            self.source_graph.as_ref(),
            self.source_audio_cache.as_ref(),
            Some(&self.capture_audio_cache),
        );
        self.runtime.w30_resample_tap =
            build_w30_resample_tap_state(&self.session, &self.runtime.transport);
        self.jam_view = JamViewModel::build(&self.session, &self.queue, self.source_graph.as_ref());
        self.runtime_view =
            JamRuntimeView::build(&self.runtime, &self.session, self.source_graph.as_ref());
    }

    pub fn set_audio_health(&mut self, health: AudioRuntimeHealth) {
        self.runtime.audio = Some(health);
        self.runtime_view =
            JamRuntimeView::build(&self.runtime, &self.session, self.source_graph.as_ref());
    }

    pub fn set_sidecar_state(&mut self, state: SidecarState) {
        self.runtime.sidecar = state;
        self.runtime_view =
            JamRuntimeView::build(&self.runtime, &self.session, self.source_graph.as_ref());
    }

    fn persist_capture_audio_artifact(&mut self, capture: &mut CaptureRef) {
        match self.write_capture_audio_artifact(capture) {
            Ok(Some(path)) => {
                if let Ok(cache) = SourceAudioCache::load_pcm_wav(&path) {
                    self.capture_audio_cache
                        .insert(capture.capture_id.clone(), cache);
                }
                append_capture_note(
                    capture,
                    &format!("audio artifact written {}", capture.storage_path),
                );
            }
            Ok(None) => {}
            Err(reason) => {
                append_capture_note(capture, &format!("audio artifact pending: {reason}"))
            }
        }
    }

    fn persist_w30_bus_print_artifact(&mut self, capture: &mut CaptureRef) {
        match self.write_w30_bus_print_artifact(capture) {
            Ok(Some(path)) => {
                if let Ok(cache) = SourceAudioCache::load_pcm_wav(&path) {
                    self.capture_audio_cache
                        .insert(capture.capture_id.clone(), cache);
                }
                append_capture_note(
                    capture,
                    &format!("bus print artifact written {}", capture.storage_path),
                );
            }
            Ok(None) => {}
            Err(reason) => append_capture_note(capture, &format!("bus print pending: {reason}")),
        }
    }

    fn write_w30_bus_print_artifact(
        &self,
        capture: &CaptureRef,
    ) -> Result<Option<PathBuf>, String> {
        if capture.capture_type != riotbox_core::session::CaptureType::Resample {
            return Ok(None);
        }

        let Some(source_capture_id) = capture.lineage_capture_refs.last() else {
            return Err("resample capture has no source capture lineage".into());
        };
        let source_capture = self
            .session
            .captures
            .iter()
            .find(|candidate| candidate.capture_id == *source_capture_id)
            .ok_or_else(|| format!("source capture {source_capture_id} not found"))?;
        let Some(path) = self.capture_audio_artifact_path(capture) else {
            return Ok(None);
        };
        let input = self
            .w30_bus_print_input(source_capture)?
            .ok_or_else(|| format!("source capture {source_capture_id} has no printable audio"))?;
        let channel_count = usize::from(input.channel_count);
        let input_frames = input.samples.len() / channel_count.max(1);
        if input_frames == 0 {
            return Err("source capture audio is empty".into());
        }
        let max_frames = usize::try_from(input.sample_rate)
            .unwrap_or(usize::MAX)
            .saturating_mul(8);
        let frame_count = input_frames.min(max_frames).max(1);
        let sample_count = frame_count.saturating_mul(channel_count);
        let dry = &input.samples[..sample_count.min(input.samples.len())];
        let render_state = self.w30_bus_print_render_state(capture, source_capture);
        let wet = render_w30_resample_tap_offline(
            &render_state,
            input.sample_rate,
            input.channel_count,
            frame_count,
        );
        let printed: Vec<f32> = dry
            .iter()
            .zip(wet.iter())
            .map(|(dry, wet)| (dry * 0.68 + wet * 1.45).clamp(-1.0, 1.0))
            .collect();

        write_interleaved_pcm16_wav(&path, input.sample_rate, input.channel_count, &printed)
            .map_err(|error| error.to_string())?;
        Ok(Some(path))
    }

    fn w30_bus_print_input(
        &self,
        capture: &CaptureRef,
    ) -> Result<Option<W30BusPrintInput>, String> {
        if let Some(cache) = self.capture_audio_cache.get(&capture.capture_id) {
            return Ok(Some(W30BusPrintInput {
                sample_rate: cache.sample_rate,
                channel_count: cache.channel_count,
                samples: cache.interleaved_samples().to_vec(),
            }));
        }

        let Some(source_window) = capture.source_window.as_ref() else {
            return Ok(None);
        };
        let Some(source_audio_cache) = self.source_audio_cache.as_ref() else {
            return Ok(None);
        };
        let frame_count = source_window
            .end_frame
            .saturating_sub(source_window.start_frame);
        if frame_count == 0 {
            return Ok(None);
        }
        let samples = source_audio_cache
            .window_samples(SourceAudioWindow {
                start_frame: usize::try_from(source_window.start_frame)
                    .map_err(|_| "source window start frame exceeds usize".to_string())?,
                frame_count: usize::try_from(frame_count)
                    .map_err(|_| "source window frame count exceeds usize".to_string())?,
            })
            .to_vec();

        Ok(Some(W30BusPrintInput {
            sample_rate: source_audio_cache.sample_rate,
            channel_count: source_audio_cache.channel_count,
            samples,
        }))
    }

    fn w30_bus_print_render_state(
        &self,
        capture: &CaptureRef,
        source_capture: &CaptureRef,
    ) -> W30ResampleTapState {
        let source_profile = if source_capture.is_pinned {
            Some(W30ResampleTapSourceProfile::PinnedCapture)
        } else if source_capture.assigned_target.is_some() {
            Some(W30ResampleTapSourceProfile::PromotedCapture)
        } else {
            Some(W30ResampleTapSourceProfile::RawCapture)
        };

        W30ResampleTapState {
            mode: W30ResampleTapMode::CaptureLineageReady,
            routing: W30ResampleTapRouting::InternalCaptureTap,
            source_profile,
            source_capture_id: Some(source_capture.capture_id.to_string()),
            lineage_capture_count: capture
                .lineage_capture_refs
                .len()
                .try_into()
                .unwrap_or(u8::MAX),
            generation_depth: capture.resample_generation_depth,
            music_bus_level: self
                .session
                .runtime_state
                .mixer_state
                .music_level
                .clamp(0.0, 1.0),
            grit_level: self
                .session
                .runtime_state
                .macro_state
                .w30_grit
                .clamp(0.0, 1.0),
            is_transport_running: self.runtime.transport.is_playing,
        }
    }

    fn refresh_capture_audio_cache(&mut self) {
        self.capture_audio_cache.clear();
        for capture in &self.session.captures {
            let Some(path) = self.capture_audio_artifact_path(capture) else {
                continue;
            };
            let Ok(cache) = SourceAudioCache::load_pcm_wav(path) else {
                continue;
            };
            self.capture_audio_cache
                .insert(capture.capture_id.clone(), cache);
        }
    }

    fn write_capture_audio_artifact(
        &self,
        capture: &CaptureRef,
    ) -> Result<Option<PathBuf>, String> {
        let Some(source_window) = capture.source_window.as_ref() else {
            return Ok(None);
        };
        let Some(source_audio_cache) = self.source_audio_cache.as_ref() else {
            return Ok(None);
        };
        if let Some(source_graph) = self.source_graph.as_ref()
            && source_graph.source.source_id != source_window.source_id
        {
            return Err(format!(
                "capture source {} does not match loaded source {}",
                source_window.source_id, source_graph.source.source_id
            ));
        }
        if let Some(source_graph) = self.source_graph.as_ref()
            && (source_graph.source.sample_rate != source_audio_cache.sample_rate
                || source_graph.source.channel_count != source_audio_cache.channel_count)
        {
            return Err(format!(
                "source graph audio shape {} Hz/{} ch does not match decoded source {} Hz/{} ch",
                source_graph.source.sample_rate,
                source_graph.source.channel_count,
                source_audio_cache.sample_rate,
                source_audio_cache.channel_count
            ));
        }
        let Some(path) = self.capture_audio_artifact_path(capture) else {
            return Ok(None);
        };

        let frame_count = source_window
            .end_frame
            .saturating_sub(source_window.start_frame);
        if frame_count == 0 {
            return Err("source window is empty".into());
        }

        source_audio_cache
            .write_window_pcm16_wav(
                &path,
                SourceAudioWindow {
                    start_frame: usize::try_from(source_window.start_frame)
                        .map_err(|_| "source window start frame exceeds usize".to_string())?,
                    frame_count: usize::try_from(frame_count)
                        .map_err(|_| "source window frame count exceeds usize".to_string())?,
                },
            )
            .map_err(|error| error.to_string())?;

        Ok(Some(path))
    }

    fn capture_audio_artifact_path(&self, capture: &CaptureRef) -> Option<PathBuf> {
        let storage_path = Path::new(&capture.storage_path);
        if storage_path.is_absolute() {
            return Some(storage_path.to_path_buf());
        }

        let files = self.files.as_ref()?;
        let session_dir = files
            .session_path
            .parent()
            .unwrap_or_else(|| Path::new("."));
        Some(session_dir.join(storage_path))
    }

    pub fn update_transport_clock(&mut self, clock: TransportClockState) {
        self.runtime.transport = clock.clone();
        self.session.runtime_state.transport.is_playing = clock.is_playing;
        self.session.runtime_state.transport.position_beats = clock.position_beats;
        self.session.runtime_state.transport.current_scene = clock.current_scene.clone();
        self.session.runtime_state.scene_state.active_scene = clock.current_scene;
        self.refresh_view();
    }

    pub fn set_transport_playing(&mut self, is_playing: bool) {
        let next_clock = transport_clock_for_state(
            self.runtime.transport.position_beats,
            is_playing,
            self.runtime.transport.current_scene.clone(),
            self.source_graph.as_ref(),
        );
        self.update_transport_clock(next_clock);
        self.runtime.transport_driver.last_audio_position_beats =
            is_playing.then_some(self.runtime.transport.beat_index);
    }

    pub fn advance_transport_by(
        &mut self,
        delta_beats: f64,
        committed_at: TimestampMs,
    ) -> Vec<CommittedActionRef> {
        if !self.runtime.transport.is_playing || delta_beats <= 0.0 {
            return Vec::new();
        }

        let previous = self.runtime.transport.clone();
        let next_position = (previous.position_beats + delta_beats).max(0.0);
        let next_clock = transport_clock_for_state(
            next_position,
            true,
            previous.current_scene.clone(),
            self.source_graph.as_ref(),
        );
        self.update_transport_clock(next_clock.clone());

        if let Some(boundary) = crossed_commit_boundary(&previous, &next_clock) {
            self.commit_ready_actions(boundary, committed_at)
        } else {
            Vec::new()
        }
    }

    pub fn apply_audio_timing_snapshot(
        &mut self,
        timing: AudioRuntimeTimingSnapshot,
        committed_at: TimestampMs,
    ) -> Vec<CommittedActionRef> {
        if self.runtime.transport.is_playing && !timing.is_transport_running {
            return Vec::new();
        }

        let previous = self.runtime.transport.clone();
        let next_clock = transport_clock_for_state(
            timing.position_beats,
            timing.is_transport_running,
            previous.current_scene.clone(),
            self.source_graph.as_ref(),
        );
        self.update_transport_clock(next_clock.clone());
        self.runtime.transport_driver.last_audio_position_beats =
            timing.is_transport_running.then_some(next_clock.beat_index);

        if timing.is_transport_running
            && let Some(boundary) = crossed_commit_boundary(&previous, &next_clock)
        {
            return self.commit_ready_actions(boundary, committed_at);
        }

        Vec::new()
    }

    fn mc202_phrase_control_pending(&self) -> bool {
        self.queue.pending_actions().iter().any(|action| {
            matches!(
                action.command,
                ActionCommand::Mc202SetRole
                    | ActionCommand::Mc202GenerateFollower
                    | ActionCommand::Mc202GenerateAnswer
                    | ActionCommand::Mc202GeneratePressure
                    | ActionCommand::Mc202GenerateInstigator
                    | ActionCommand::Mc202MutatePhrase
            )
        })
    }

    pub fn queue_mc202_role_toggle(&mut self, requested_at: TimestampMs) -> QueueControlResult {
        if self.mc202_phrase_control_pending() {
            return QueueControlResult::AlreadyPending;
        }

        let next_role = match self.session.runtime_state.lane_state.mc202.role.as_deref() {
            Some("follower") => "leader",
            Some("leader") => "follower",
            Some(_) | None => "follower",
        };
        let target_touch = if next_role == "leader" { 0.85 } else { 0.65 };

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Mc202SetRole,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneMc202),
                object_id: Some(next_role.into()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: target_touch,
            target_id: Some(next_role.into()),
        };
        draft.explanation = Some(format!("set MC-202 role to {next_role} on next phrase"));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_mc202_mutate_phrase(&mut self, requested_at: TimestampMs) -> QueueControlResult {
        if self.mc202_phrase_control_pending() {
            return QueueControlResult::AlreadyPending;
        }
        if self.session.runtime_state.lane_state.mc202.role.is_none() {
            return QueueControlResult::AlreadyInState;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Mc202MutatePhrase,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneMc202),
                object_id: Some("mutated_drive".into()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 0.88,
            target_id: Some("mutated_drive".into()),
        };
        draft.explanation = Some("mutate MC-202 phrase on next phrase boundary".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_mc202_generate_follower(
        &mut self,
        requested_at: TimestampMs,
    ) -> QueueControlResult {
        if self.mc202_phrase_control_pending() {
            return QueueControlResult::AlreadyPending;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Mc202GenerateFollower,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneMc202),
                object_id: Some("follower".into()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 0.78,
            target_id: Some("follower".into()),
        };
        draft.explanation = Some("generate MC-202 follower phrase on next phrase boundary".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_mc202_generate_answer(&mut self, requested_at: TimestampMs) -> QueueControlResult {
        if self.mc202_phrase_control_pending() {
            return QueueControlResult::AlreadyPending;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Mc202GenerateAnswer,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneMc202),
                object_id: Some("answer".into()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 0.82,
            target_id: Some("answer".into()),
        };
        draft.explanation = Some("generate MC-202 answer phrase on next phrase boundary".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_mc202_generate_pressure(
        &mut self,
        requested_at: TimestampMs,
    ) -> QueueControlResult {
        if self.mc202_phrase_control_pending() {
            return QueueControlResult::AlreadyPending;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Mc202GeneratePressure,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneMc202),
                object_id: Some("pressure".into()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 0.84,
            target_id: Some("pressure".into()),
        };
        draft.explanation = Some("generate MC-202 pressure phrase on next phrase boundary".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_mc202_generate_instigator(
        &mut self,
        requested_at: TimestampMs,
    ) -> QueueControlResult {
        if self.mc202_phrase_control_pending() {
            return QueueControlResult::AlreadyPending;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Mc202GenerateInstigator,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneMc202),
                object_id: Some("instigator".into()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 0.90,
            target_id: Some("instigator".into()),
        };
        draft.explanation =
            Some("generate MC-202 instigator phrase on next phrase boundary".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_tr909_fill(&mut self, requested_at: TimestampMs) {
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909FillNext,
            Quantization::NextBar,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        );
        draft.explanation = Some("trigger TR-909 fill on next bar".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
    }

    pub fn queue_tr909_reinforce(&mut self, requested_at: TimestampMs) {
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909ReinforceBreak,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        );
        draft.explanation = Some("reinforce next phrase with TR-909 drum layer".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
    }

    fn tr909_takeover_change_pending(&self) -> bool {
        self.queue.pending_actions().iter().any(|action| {
            matches!(
                action.command,
                ActionCommand::Tr909Takeover
                    | ActionCommand::Tr909SceneLock
                    | ActionCommand::Tr909Release
            )
        })
    }

    pub fn queue_tr909_slam_toggle(&mut self, requested_at: TimestampMs) -> bool {
        if self
            .queue
            .pending_actions()
            .iter()
            .any(|action| action.command == ActionCommand::Tr909SetSlam)
        {
            return false;
        }

        let enabling = !self.session.runtime_state.lane_state.tr909.slam_enabled;
        let intensity = if enabling {
            self.session.runtime_state.macro_state.tr909_slam.max(0.85)
        } else {
            0.0
        };

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909SetSlam,
            Quantization::NextBeat,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity,
            target_id: Some(if enabling { "enabled" } else { "disabled" }.into()),
        };
        draft.explanation = Some(if enabling {
            format!("enable TR-909 slam at {:.2}", intensity)
        } else {
            "disable TR-909 slam".into()
        });
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        true
    }

    pub fn queue_tr909_takeover(&mut self, requested_at: TimestampMs) -> QueueControlResult {
        if self.tr909_takeover_change_pending() {
            return QueueControlResult::AlreadyPending;
        }
        if self.session.runtime_state.lane_state.tr909.takeover_enabled {
            return QueueControlResult::AlreadyInState;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909Takeover,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 1.0,
            target_id: Some("takeover".into()),
        };
        draft.explanation = Some("engage controlled TR-909 takeover on next phrase".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_tr909_scene_lock(&mut self, requested_at: TimestampMs) -> QueueControlResult {
        if self.tr909_takeover_change_pending() {
            return QueueControlResult::AlreadyPending;
        }

        if self.session.runtime_state.lane_state.tr909.takeover_enabled
            && self.session.runtime_state.lane_state.tr909.takeover_profile
                == Some(Tr909TakeoverProfileState::SceneLockTakeover)
        {
            return QueueControlResult::AlreadyInState;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909SceneLock,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 1.0,
            target_id: Some("scene_lock".into()),
        };
        draft.explanation = Some("engage scene-lock TR-909 variation on next phrase".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_tr909_release(&mut self, requested_at: TimestampMs) -> QueueControlResult {
        if self.tr909_takeover_change_pending() {
            return QueueControlResult::AlreadyPending;
        }
        if !self.session.runtime_state.lane_state.tr909.takeover_enabled {
            return QueueControlResult::AlreadyInState;
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909Release,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 0.0,
            target_id: Some("release".into()),
        };
        draft.explanation = Some("release controlled TR-909 takeover on next phrase".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        QueueControlResult::Enqueued
    }

    pub fn queue_capture_bar(&mut self, requested_at: TimestampMs) {
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::CaptureBarGroup,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Capture { bars: Some(4) };
        draft.explanation = Some("capture next phrase into W-30 path".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
    }

    pub fn queue_promote_last_capture(&mut self, requested_at: TimestampMs) -> bool {
        let Some(capture) = self
            .session
            .captures
            .iter()
            .rev()
            .find(|capture| capture.assigned_target.is_none())
            .or_else(|| self.session.captures.last())
        else {
            return false;
        };

        let Some(bank_id) = self
            .session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .clone()
        else {
            return false;
        };
        let Some(pad_id) = self
            .session
            .runtime_state
            .lane_state
            .w30
            .focused_pad
            .clone()
        else {
            return false;
        };

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::PromoteCaptureToPad,
            Quantization::NextBar,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Promotion {
            capture_id: Some(capture.capture_id.clone()),
            destination: Some(format!("w30:{bank_id}/{pad_id}")),
        };
        draft.explanation = Some(format!(
            "promote {} into W-30 pad {bank_id}/{pad_id}",
            capture.capture_id
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        true
    }

    pub fn queue_w30_live_recall(
        &mut self,
        requested_at: TimestampMs,
    ) -> Option<QueueControlResult> {
        if self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let capture = self.recallable_w30_capture()?.clone();
        let CaptureTarget::W30Pad { bank_id, pad_id } = capture.assigned_target.clone()? else {
            return None;
        };

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30LiveRecall,
            Quantization::NextBar,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 1.0,
            target_id: Some(capture.capture_id.to_string()),
        };
        draft.explanation = Some(format!(
            "recall {} on W-30 pad {bank_id}/{pad_id}",
            capture.capture_id
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn queue_w30_step_focus(
        &mut self,
        requested_at: TimestampMs,
    ) -> Option<QueueControlResult> {
        if self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let (bank_id, pad_id) = self.next_w30_focus_target()?;
        let current_focus = self
            .session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .clone()
            .zip(
                self.session
                    .runtime_state
                    .lane_state
                    .w30
                    .focused_pad
                    .clone(),
            );
        if current_focus == Some((bank_id.clone(), pad_id.clone())) {
            return Some(QueueControlResult::AlreadyInState);
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30StepFocus,
            Quantization::NextBeat,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
        );
        draft.explanation = Some(format!(
            "step W-30 focus to {bank_id}/{pad_id} on next beat"
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn queue_w30_swap_bank(&mut self, requested_at: TimestampMs) -> Option<QueueControlResult> {
        if self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let (bank_id, pad_id, capture_id) = self.next_w30_bank_target()?;
        let current_focus = self
            .session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .clone()
            .zip(
                self.session
                    .runtime_state
                    .lane_state
                    .w30
                    .focused_pad
                    .clone(),
            );
        if current_focus == Some((bank_id.clone(), pad_id.clone())) {
            return Some(QueueControlResult::AlreadyInState);
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30SwapBank,
            Quantization::NextBar,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 1.0,
            target_id: Some(capture_id.to_string()),
        };
        draft.explanation = Some(format!(
            "swap W-30 bank to {bank_id}/{pad_id} with {capture_id} on next bar"
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn queue_w30_browse_slice_pool(
        &mut self,
        requested_at: TimestampMs,
    ) -> Option<QueueControlResult> {
        if self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let target = self.next_w30_slice_pool_capture()?;
        if self
            .session
            .runtime_state
            .lane_state
            .w30
            .last_capture
            .as_ref()
            == Some(&target.capture_id)
        {
            return Some(QueueControlResult::AlreadyInState);
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30BrowseSlicePool,
            Quantization::NextBeat,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(target.bank_id.clone()),
                pad_id: Some(target.pad_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 1.0,
            target_id: Some(target.capture_id.to_string()),
        };
        let reason = match target.selection_reason {
            w30_targets::W30SlicePoolSelectionReason::Cycle => "slice pool",
            w30_targets::W30SlicePoolSelectionReason::FeralScorecard => "feral slice pool",
        };
        draft.explanation = Some(format!(
            "browse W-30 {reason} to {} on {}/{} on next beat",
            target.capture_id, target.bank_id, target.pad_id
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn queue_w30_apply_damage_profile(
        &mut self,
        requested_at: TimestampMs,
    ) -> Option<QueueControlResult> {
        if self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let capture = self.damage_profile_ready_w30_capture()?.clone();
        let CaptureTarget::W30Pad { bank_id, pad_id } = capture.assigned_target.clone()? else {
            return None;
        };

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30ApplyDamageProfile,
            Quantization::NextBar,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: Self::W30_DAMAGE_PROFILE_GRIT,
            target_id: Some(capture.capture_id.to_string()),
        };
        draft.explanation = Some(format!(
            "apply {} damage profile to {} on W-30 pad {bank_id}/{pad_id}",
            Self::W30_DAMAGE_PROFILE_LABEL,
            capture.capture_id
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn queue_w30_loop_freeze(
        &mut self,
        requested_at: TimestampMs,
    ) -> Option<QueueControlResult> {
        if self.w30_phrase_capture_cue_pending() || self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let capture = self.loop_freeze_ready_w30_capture()?.clone();
        let CaptureTarget::W30Pad { bank_id, pad_id } = capture.assigned_target.clone()? else {
            return None;
        };

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30LoopFreeze,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Promotion {
            capture_id: Some(capture.capture_id.clone()),
            destination: Some("w30:loop_freeze".into()),
        };
        draft.explanation = Some(format!(
            "{} {} for W-30 reuse on {bank_id}/{pad_id} on next phrase",
            Self::W30_LOOP_FREEZE_LABEL,
            capture.capture_id
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn queue_w30_promoted_audition(
        &mut self,
        requested_at: TimestampMs,
    ) -> Option<QueueControlResult> {
        if self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let capture = self.auditionable_w30_capture()?.clone();
        let CaptureTarget::W30Pad { bank_id, pad_id } = capture.assigned_target.clone()? else {
            return None;
        };

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30AuditionPromoted,
            Quantization::NextBar,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 0.68,
            target_id: Some(capture.capture_id.to_string()),
        };
        draft.explanation = Some(format!(
            "audition promoted {} on W-30 pad {bank_id}/{pad_id}",
            capture.capture_id
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn queue_w30_audition(&mut self, requested_at: TimestampMs) -> Option<QueueControlResult> {
        if self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        if self.auditionable_w30_capture().is_some() {
            return self.queue_w30_promoted_audition(requested_at);
        }

        self.queue_w30_raw_capture_audition(requested_at)
    }

    pub fn queue_w30_raw_capture_audition(
        &mut self,
        requested_at: TimestampMs,
    ) -> Option<QueueControlResult> {
        if self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let capture = self.raw_auditionable_capture()?.clone();
        let w30 = &self.session.runtime_state.lane_state.w30;
        let bank_id = w30.active_bank.clone()?;
        let pad_id = w30.focused_pad.clone()?;

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30AuditionRawCapture,
            Quantization::NextBar,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: 0.58,
            target_id: Some(capture.capture_id.to_string()),
        };
        draft.explanation = Some(format!(
            "audition raw capture {} on W-30 preview {bank_id}/{pad_id}",
            capture.capture_id
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn queue_w30_trigger_pad(
        &mut self,
        requested_at: TimestampMs,
    ) -> Option<QueueControlResult> {
        if self.w30_pad_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let capture = self.triggerable_w30_capture()?.clone();
        let CaptureTarget::W30Pad { bank_id, pad_id } = capture.assigned_target.clone()? else {
            return None;
        };

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30TriggerPad,
            Quantization::NextBeat,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(bank_id.clone()),
                pad_id: Some(pad_id.clone()),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: if capture.is_pinned { 0.72 } else { 0.84 },
            target_id: Some(capture.capture_id.to_string()),
        };
        draft.explanation = Some(format!(
            "trigger W-30 pad {bank_id}/{pad_id} from {} on next beat",
            capture.capture_id
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn queue_w30_internal_resample(
        &mut self,
        requested_at: TimestampMs,
    ) -> Option<QueueControlResult> {
        if self.w30_phrase_capture_cue_pending() {
            return Some(QueueControlResult::AlreadyPending);
        }

        let capture = self.resample_ready_w30_capture()?.clone();
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::PromoteResample,
            Quantization::NextPhrase,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::LaneW30),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Promotion {
            capture_id: Some(capture.capture_id.clone()),
            destination: Some("w30:resample".into()),
        };
        draft.explanation = Some(format!(
            "resample {} through W-30 on next phrase",
            capture.capture_id
        ));
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
        Some(QueueControlResult::Enqueued)
    }

    pub fn toggle_pin_latest_capture(&mut self) -> Option<bool> {
        let new_state = {
            let capture = self.session.captures.last_mut()?;
            capture.is_pinned = !capture.is_pinned;
            capture.is_pinned
        };
        self.refresh_view();
        Some(new_state)
    }

    pub fn adjust_drum_bus_level(&mut self, delta: f32) -> f32 {
        let next_level =
            (self.session.runtime_state.mixer_state.drum_level + delta).clamp(0.0, 1.0);
        self.session.runtime_state.mixer_state.drum_level = next_level;
        self.refresh_view();
        next_level
    }

    pub fn adjust_mc202_touch(&mut self, delta: f32) -> f32 {
        let next_touch =
            (self.session.runtime_state.macro_state.mc202_touch + delta).clamp(0.0, 1.0);
        self.session.runtime_state.macro_state.mc202_touch = next_touch;
        self.refresh_view();
        next_touch
    }

    pub fn undo_last_action(&mut self, requested_at: TimestampMs) -> Option<Action> {
        let next_undo_action_id = next_action_id_from_session(&self.session);

        let undone_index = self.session.action_log.actions.iter().rposition(|action| {
            action.status == ActionStatus::Committed
                && matches!(
                    action.undo_policy,
                    riotbox_core::action::UndoPolicy::Undoable
                )
        })?;

        let undone_action_id = self.session.action_log.actions[undone_index].id;
        let undone_command = self.session.action_log.actions[undone_index].command;
        let is_mc202_undo = is_mc202_phrase_action(undone_command);
        let mc202_restored = if is_mc202_undo {
            self.restore_mc202_undo_snapshot(undone_action_id)
        } else {
            false
        };
        if is_mc202_undo && !mc202_restored {
            return None;
        }

        let undo_summary = if mc202_restored {
            format!("undone by user at {requested_at}; restored MC-202 lane state")
        } else {
            format!("undone by user at {requested_at}")
        };

        {
            let undone = &mut self.session.action_log.actions[undone_index];
            undone.status = ActionStatus::Undone;
            undone.result = Some(ActionResult {
                accepted: true,
                summary: undo_summary,
            });
        }

        let undo_action = Action {
            id: next_undo_action_id,
            actor: ActorType::User,
            command: ActionCommand::UndoLast,
            params: ActionParams::Empty,
            target: ActionTarget {
                scope: Some(TargetScope::Session),
                ..Default::default()
            },
            requested_at,
            quantization: Quantization::Immediate,
            status: ActionStatus::Committed,
            committed_at: Some(requested_at),
            result: Some(ActionResult {
                accepted: true,
                summary: "undid most recent undoable action".into(),
            }),
            undo_policy: riotbox_core::action::UndoPolicy::NotUndoable {
                reason: "undo marker actions are not themselves undoable".into(),
            },
            explanation: Some("undo most recent committed action".into()),
        };

        self.session.action_log.actions.push(undo_action.clone());
        self.queue
            .reserve_action_ids_after(max_action_id(&self.session));
        self.refresh_view();
        Some(undo_action)
    }

    fn restore_mc202_undo_snapshot(&mut self, action_id: ActionId) -> bool {
        let Some(snapshot_index) = self
            .session
            .runtime_state
            .undo_state
            .mc202_snapshots
            .iter()
            .rposition(|snapshot| snapshot.action_id == action_id)
        else {
            return false;
        };
        let snapshot = self
            .session
            .runtime_state
            .undo_state
            .mc202_snapshots
            .remove(snapshot_index);
        snapshot.apply_to_session(&mut self.session);
        true
    }

    pub fn commit_ready_actions(
        &mut self,
        boundary: CommitBoundaryState,
        committed_at: TimestampMs,
    ) -> Vec<CommittedActionRef> {
        let committed = self
            .queue
            .commit_ready_for_transport(boundary.clone(), committed_at);

        for committed_ref in &committed {
            if let Some(action) = self.queue.history_action(committed_ref.action_id) {
                let action = action.clone();
                self.session.action_log.actions.push(action.clone());
                self.apply_committed_action_side_effects(&action, &boundary);
            }
        }

        self.runtime.last_commit_boundary = Some(boundary);
        self.refresh_view();
        committed
    }

    fn apply_committed_action_side_effects(
        &mut self,
        action: &Action,
        boundary: &CommitBoundaryState,
    ) {
        if is_mc202_phrase_action(action.command) {
            self.session.runtime_state.undo_state.mc202_snapshots.push(
                Mc202UndoSnapshotState::from_session(action.id, &self.session),
            );
        }

        if let Some(mut capture) =
            capture_ref_from_action(&self.session, self.source_graph.as_ref(), action, boundary)
        {
            if matches!(action.command, ActionCommand::PromoteResample) {
                self.persist_w30_bus_print_artifact(&mut capture);
            } else {
                self.persist_capture_audio_artifact(&mut capture);
            }
            self.session.runtime_state.lane_state.w30.last_capture =
                Some(capture.capture_id.clone());
            self.session.captures.push(capture);
        } else if apply_capture_promotion_side_effects(&mut self.session, action) {
            let result_summary = capture_promotion_summary(&self.session, action)
                .unwrap_or_else(|| "promotion committed".into());
            if let Some(logged_action) = self
                .session
                .action_log
                .actions
                .iter_mut()
                .rev()
                .find(|logged_action| logged_action.id == action.id)
            {
                logged_action.result = Some(ActionResult {
                    accepted: true,
                    summary: result_summary,
                });
            }
        }

        apply_w30_side_effects(&mut self.session, action, Some(boundary));
        apply_mc202_side_effects(&mut self.session, action, Some(boundary));
        apply_tr909_side_effects(&mut self.session, action, Some(boundary));
        apply_scene_side_effects(
            &mut self.session,
            action,
            Some(boundary),
            self.source_graph.as_ref(),
        );
        if matches!(
            action.command,
            ActionCommand::SceneLaunch | ActionCommand::SceneRestore
        ) {
            self.runtime.transport.current_scene =
                self.session.runtime_state.transport.current_scene.clone();
        }
    }
}

fn is_mc202_phrase_action(command: ActionCommand) -> bool {
    matches!(
        command,
        ActionCommand::Mc202SetRole
            | ActionCommand::Mc202GenerateFollower
            | ActionCommand::Mc202GenerateAnswer
            | ActionCommand::Mc202GeneratePressure
            | ActionCommand::Mc202GenerateInstigator
            | ActionCommand::Mc202MutatePhrase
    )
}

fn append_capture_note(capture: &mut CaptureRef, detail: &str) {
    capture.notes = Some(match capture.notes.as_deref() {
        Some(existing) if !existing.is_empty() => format!("{existing} | {detail}"),
        _ => detail.into(),
    });
}

fn next_action_id_from_session(session: &SessionFile) -> riotbox_core::ids::ActionId {
    riotbox_core::ids::ActionId(
        max_action_id(session)
            .map(|id| id.0.saturating_add(1))
            .unwrap_or(1),
    )
}

fn max_action_id(session: &SessionFile) -> Option<riotbox_core::ids::ActionId> {
    session
        .action_log
        .actions
        .iter()
        .map(|action| action.id)
        .max()
}

#[cfg(test)]
mod tests;
