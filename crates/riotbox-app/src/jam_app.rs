use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    io,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use riotbox_audio::{
    runtime::{AudioRuntimeHealth, AudioRuntimeLifecycle},
    tr909::{
        Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
        Tr909RenderState, Tr909SourceSupportProfile, Tr909TakeoverRenderProfile,
    },
    w30::{
        W30PreviewRenderMode, W30PreviewRenderRouting, W30PreviewRenderState,
        W30PreviewSourceProfile,
    },
};
use riotbox_core::{
    TimestampMs,
    action::{
        Action, ActionCommand, ActionDraft, ActionParams, ActionResult, ActionStatus, ActionTarget,
        ActorType, Quantization, TargetScope,
    },
    ids::{CaptureId, SourceId},
    persistence::{
        PersistenceError, load_session_json, load_source_graph_json, save_session_json,
        save_source_graph_json,
    },
    queue::{ActionQueue, CommittedActionRef},
    session::{
        CaptureRef, CaptureTarget, CaptureType, GraphStorageMode, SessionFile, SourceGraphRef,
        SourceRef,
    },
    source_graph::{DecodeProfile, SourceGraph},
    transport::{CommitBoundaryState, TransportClockState},
    view::jam::JamViewModel,
};
use riotbox_sidecar::client::{ClientError as SidecarClientError, StdioSidecarClient};
use sha2::{Digest, Sha256};

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
    pub w30_preview: W30PreviewRenderState,
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
            w30_preview: W30PreviewRenderState::default(),
            last_commit_boundary: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TransportDriverState {
    pub last_pulse_at_ms: Option<TimestampMs>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JamRuntimeView {
    pub audio_status: String,
    pub audio_callback_count: u64,
    pub audio_last_error: Option<String>,
    pub sidecar_status: String,
    pub sidecar_version: Option<String>,
    pub tr909_render_mode: String,
    pub tr909_render_routing: String,
    pub tr909_render_profile: String,
    pub tr909_render_pattern_ref: Option<String>,
    pub tr909_render_pattern_adoption: String,
    pub tr909_render_phrase_variation: String,
    pub tr909_render_mix_summary: String,
    pub tr909_render_alignment: String,
    pub tr909_render_transport_summary: String,
    pub w30_preview_mode: String,
    pub w30_preview_routing: String,
    pub w30_preview_profile: String,
    pub w30_preview_target_summary: String,
    pub w30_preview_mix_summary: String,
    pub w30_preview_transport_summary: String,
    pub w30_preview_trigger_summary: String,
    pub runtime_warnings: Vec<String>,
}

impl JamRuntimeView {
    #[must_use]
    pub fn build(runtime: &AppRuntimeState, session: &SessionFile) -> Self {
        let (audio_status, audio_callback_count, audio_last_error) = match &runtime.audio {
            Some(health) => (
                match health.lifecycle {
                    AudioRuntimeLifecycle::Idle => "idle".into(),
                    AudioRuntimeLifecycle::Running => "running".into(),
                    AudioRuntimeLifecycle::Stopped => "stopped".into(),
                    AudioRuntimeLifecycle::Faulted => "faulted".into(),
                },
                health.callback_count,
                health.last_stream_error.clone(),
            ),
            None => ("unknown".into(), 0, None),
        };

        let (sidecar_status, sidecar_version) = match &runtime.sidecar {
            SidecarState::Unknown => ("unknown".into(), None),
            SidecarState::Ready { version, .. } => ("ready".into(), version.clone()),
            SidecarState::Unavailable { .. } => ("unavailable".into(), None),
            SidecarState::Degraded { .. } => ("degraded".into(), None),
        };

        let mut runtime_warnings = Vec::new();
        if matches!(
            runtime.audio.as_ref().map(|health| health.lifecycle),
            Some(AudioRuntimeLifecycle::Faulted)
        ) {
            runtime_warnings.push("audio runtime faulted".into());
        }
        match &runtime.sidecar {
            SidecarState::Unavailable { reason } => {
                runtime_warnings.push(format!("sidecar unavailable: {reason}"));
            }
            SidecarState::Degraded { reason } => {
                runtime_warnings.push(format!("sidecar degraded: {reason}"));
            }
            SidecarState::Unknown | SidecarState::Ready { .. } => {}
        }

        runtime_warnings.extend(derive_tr909_render_warnings(&runtime.tr909_render, session));
        runtime_warnings.extend(derive_w30_preview_warnings(&runtime.w30_preview, session));

        Self {
            audio_status,
            audio_callback_count,
            audio_last_error,
            sidecar_status,
            sidecar_version,
            tr909_render_mode: runtime.tr909_render.mode.label().into(),
            tr909_render_routing: runtime.tr909_render.routing.label().into(),
            tr909_render_profile: tr909_render_profile_label(&runtime.tr909_render).into(),
            tr909_render_pattern_ref: runtime.tr909_render.pattern_ref.clone(),
            tr909_render_pattern_adoption: runtime
                .tr909_render
                .pattern_adoption
                .map_or_else(|| "unset".into(), |pattern| pattern.label().into()),
            tr909_render_phrase_variation: runtime
                .tr909_render
                .phrase_variation
                .map_or_else(|| "unset".into(), |variation| variation.label().into()),
            tr909_render_mix_summary: format!(
                "drum bus {:.2} | slam {:.2}",
                runtime.tr909_render.drum_bus_level, runtime.tr909_render.slam_intensity
            ),
            tr909_render_alignment: tr909_render_alignment_label(&runtime.tr909_render).into(),
            tr909_render_transport_summary: tr909_render_transport_summary(&runtime.tr909_render),
            w30_preview_mode: runtime.w30_preview.mode.label().into(),
            w30_preview_routing: runtime.w30_preview.routing.label().into(),
            w30_preview_profile: w30_preview_profile_label(&runtime.w30_preview).into(),
            w30_preview_target_summary: w30_preview_target_summary(&runtime.w30_preview),
            w30_preview_mix_summary: format!(
                "music bus {:.2} | grit {:.2}",
                runtime.w30_preview.music_bus_level, runtime.w30_preview.grit_level
            ),
            w30_preview_transport_summary: w30_preview_transport_summary(&runtime.w30_preview),
            w30_preview_trigger_summary: w30_preview_trigger_summary(&runtime.w30_preview),
            runtime_warnings,
        }
    }
}

fn w30_preview_profile_label(render: &W30PreviewRenderState) -> &'static str {
    render
        .source_profile
        .map_or("unset", W30PreviewSourceProfile::label)
}

fn w30_preview_target_summary(render: &W30PreviewRenderState) -> String {
    if matches!(render.mode, W30PreviewRenderMode::Idle) {
        return "target unset".into();
    }

    format!(
        "{} / {} | {}",
        render.active_bank_id.as_deref().unwrap_or("bank unset"),
        render.focused_pad_id.as_deref().unwrap_or("pad unset"),
        render.capture_id.as_deref().unwrap_or("capture unset")
    )
}

fn w30_preview_transport_summary(render: &W30PreviewRenderState) -> String {
    if matches!(render.mode, W30PreviewRenderMode::Idle) {
        return "preview idle".into();
    }

    format!(
        "{} @ {:.1} | {:.1} BPM",
        if render.is_transport_running {
            "transport running"
        } else {
            "transport stopped"
        },
        render.position_beats,
        render.tempo_bpm
    )
}

fn w30_preview_trigger_summary(render: &W30PreviewRenderState) -> String {
    if render.trigger_revision == 0 {
        if matches!(render.mode, W30PreviewRenderMode::Idle) {
            return "trigger unset".into();
        }

        return "trigger pending from committed seam".into();
    }

    format!(
        "trigger r{} @ {:.2}",
        render.trigger_revision, render.trigger_velocity
    )
}

fn tr909_render_profile_label(render: &Tr909RenderState) -> &'static str {
    match (render.takeover_profile, render.source_support_profile) {
        (Some(profile), _) => profile.label(),
        (None, Some(profile)) => profile.label(),
        (None, None) => "unset",
    }
}

fn derive_tr909_pattern_adoption(
    mode: Tr909RenderMode,
    pattern_ref: Option<&str>,
    source_support_profile: Option<Tr909SourceSupportProfile>,
    takeover_profile: Option<Tr909TakeoverRenderProfile>,
) -> Option<Tr909PatternAdoption> {
    if matches!(mode, Tr909RenderMode::Idle) {
        return None;
    }

    if matches!(mode, Tr909RenderMode::Takeover)
        || matches!(
            takeover_profile,
            Some(Tr909TakeoverRenderProfile::ControlledPhrase)
        )
    {
        return Some(Tr909PatternAdoption::TakeoverGrid);
    }

    let pattern_ref = pattern_ref.map(str::to_ascii_lowercase);
    if pattern_ref
        .as_deref()
        .is_some_and(|pattern| pattern.contains("takeover"))
    {
        return Some(Tr909PatternAdoption::TakeoverGrid);
    }

    if pattern_ref
        .as_deref()
        .is_some_and(|pattern| pattern.contains("main") || pattern.contains("drop"))
        || matches!(
            source_support_profile,
            Some(Tr909SourceSupportProfile::DropDrive)
        )
        || matches!(
            mode,
            Tr909RenderMode::Fill | Tr909RenderMode::BreakReinforce
        )
    {
        return Some(Tr909PatternAdoption::MainlineDrive);
    }

    Some(Tr909PatternAdoption::SupportPulse)
}

fn derive_tr909_phrase_variation(
    mode: Tr909RenderMode,
    transport: &TransportClockState,
    pattern_ref: Option<&str>,
    source_support_profile: Option<Tr909SourceSupportProfile>,
    takeover_profile: Option<Tr909TakeoverRenderProfile>,
) -> Option<Tr909PhraseVariation> {
    if matches!(mode, Tr909RenderMode::Idle) {
        return None;
    }

    let pattern_ref = pattern_ref.map(str::to_ascii_lowercase);
    if pattern_ref
        .as_deref()
        .is_some_and(|pattern| pattern.contains("release"))
    {
        return Some(Tr909PhraseVariation::PhraseRelease);
    }

    let phrase_cycle = transport.phrase_index % 4;
    let variation = match mode {
        Tr909RenderMode::Takeover => match takeover_profile {
            Some(Tr909TakeoverRenderProfile::ControlledPhrase) | None => match phrase_cycle {
                0 => Tr909PhraseVariation::PhraseAnchor,
                1 => Tr909PhraseVariation::PhraseLift,
                2 => Tr909PhraseVariation::PhraseDrive,
                _ => Tr909PhraseVariation::PhraseRelease,
            },
            Some(Tr909TakeoverRenderProfile::SceneLock) => match phrase_cycle % 2 {
                0 => Tr909PhraseVariation::PhraseDrive,
                _ => Tr909PhraseVariation::PhraseAnchor,
            },
        },
        Tr909RenderMode::Fill | Tr909RenderMode::BreakReinforce => match phrase_cycle % 2 {
            0 => Tr909PhraseVariation::PhraseDrive,
            _ => Tr909PhraseVariation::PhraseLift,
        },
        Tr909RenderMode::SourceSupport => match source_support_profile {
            Some(Tr909SourceSupportProfile::SteadyPulse) | None => match phrase_cycle % 2 {
                0 => Tr909PhraseVariation::PhraseAnchor,
                _ => Tr909PhraseVariation::PhraseLift,
            },
            Some(Tr909SourceSupportProfile::BreakLift) => match phrase_cycle % 2 {
                0 => Tr909PhraseVariation::PhraseLift,
                _ => Tr909PhraseVariation::PhraseDrive,
            },
            Some(Tr909SourceSupportProfile::DropDrive) => match phrase_cycle % 2 {
                0 => Tr909PhraseVariation::PhraseDrive,
                _ => Tr909PhraseVariation::PhraseLift,
            },
        },
        Tr909RenderMode::Idle => Tr909PhraseVariation::PhraseAnchor,
    };

    Some(variation)
}

fn tr909_render_alignment_label(render: &Tr909RenderState) -> &'static str {
    match render.mode {
        Tr909RenderMode::Idle => "source-only idle",
        Tr909RenderMode::SourceSupport => "support aligned",
        Tr909RenderMode::Fill => "fill aligned",
        Tr909RenderMode::BreakReinforce => "break reinforce aligned",
        Tr909RenderMode::Takeover => "takeover aligned",
    }
}

fn tr909_render_transport_summary(render: &Tr909RenderState) -> String {
    let transport = if render.is_transport_running {
        "running"
    } else {
        "stopped"
    };
    let scene = render.current_scene_id.as_deref().unwrap_or("none");
    format!(
        "{transport} @ {:.1} beats | {:.1} BPM | scene {scene}",
        render.position_beats, render.tempo_bpm
    )
}

fn derive_tr909_render_warnings(render: &Tr909RenderState, session: &SessionFile) -> Vec<String> {
    let mut warnings = Vec::new();
    let lane = &session.runtime_state.lane_state.tr909;

    if matches!(render.mode, Tr909RenderMode::Idle)
        && !matches!(render.routing, Tr909RenderRouting::SourceOnly)
    {
        warnings.push("909 render idle but routing is not source_only".into());
    }

    if matches!(render.mode, Tr909RenderMode::Takeover)
        && !matches!(render.routing, Tr909RenderRouting::DrumBusTakeover)
    {
        warnings.push("909 takeover render is not routed to drum_bus_takeover".into());
    }

    if !matches!(render.mode, Tr909RenderMode::Takeover) && render.takeover_profile.is_some() {
        warnings.push("909 render carries a takeover profile outside takeover mode".into());
    }

    if matches!(render.mode, Tr909RenderMode::SourceSupport)
        && render.source_support_profile.is_none()
    {
        warnings.push("909 source-support render is missing a support profile".into());
    }

    if !matches!(render.mode, Tr909RenderMode::SourceSupport)
        && render.source_support_profile.is_some()
    {
        warnings.push("909 render carries a support profile outside source-support mode".into());
    }

    if matches!(
        render.routing,
        Tr909RenderRouting::DrumBusSupport | Tr909RenderRouting::DrumBusTakeover
    ) && render.drum_bus_level <= 0.0
    {
        warnings.push("909 render is routed to the drum bus at zero drum level".into());
    }

    if lane.takeover_enabled && !matches!(render.mode, Tr909RenderMode::Takeover) {
        warnings.push("909 lane takeover is committed but render mode is not takeover".into());
    }

    if render.pattern_ref.is_none()
        && (lane.takeover_enabled
            || lane.reinforcement_mode.is_some()
            || lane.slam_enabled
            || render.takeover_profile.is_some())
    {
        warnings.push("909 render has no pattern_ref while musical support is active".into());
    }

    warnings
}

fn derive_w30_preview_warnings(
    render: &W30PreviewRenderState,
    session: &SessionFile,
) -> Vec<String> {
    if matches!(render.mode, W30PreviewRenderMode::Idle) {
        return Vec::new();
    }

    let mut warnings = Vec::new();

    if matches!(render.routing, W30PreviewRenderRouting::MusicBusPreview)
        && render.music_bus_level <= 0.0
    {
        warnings.push("W-30 preview is routed to the music bus at zero music level".into());
    }

    let has_capture = render.capture_id.as_ref().is_some_and(|capture_id| {
        session
            .captures
            .iter()
            .any(|capture| capture.capture_id.to_string() == *capture_id)
    });
    if !has_capture {
        warnings
            .push("W-30 preview has no committed capture backing the current lane focus".into());
    }

    warnings
}

#[derive(Clone, Debug)]
pub struct JamAppState {
    pub files: Option<JamFileSet>,
    pub session: SessionFile,
    pub source_graph: Option<SourceGraph>,
    pub queue: ActionQueue,
    pub runtime: AppRuntimeState,
    pub jam_view: JamViewModel,
    pub runtime_view: JamRuntimeView,
}

impl JamAppState {
    #[must_use]
    pub fn from_parts(
        session: SessionFile,
        source_graph: Option<SourceGraph>,
        mut queue: ActionQueue,
    ) -> Self {
        queue.reserve_action_ids_after(max_action_id(&session));
        let transport = transport_clock_from_state(&session, source_graph.as_ref());
        let jam_view = JamViewModel::build(&session, &queue, source_graph.as_ref());
        let runtime_view = JamRuntimeView::build(&AppRuntimeState::default(), &session);
        let mut state = Self {
            files: None,
            session,
            source_graph,
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

    pub fn from_json_files(
        session_path: impl AsRef<Path>,
        source_graph_path: Option<impl AsRef<Path>>,
    ) -> Result<Self, JamAppError> {
        let session_path = session_path.as_ref().to_path_buf();
        let session = load_session_json(&session_path)?;
        validate_mvp_single_source_session(&session)?;
        let explicit_source_graph_path = source_graph_path.map(|path| path.as_ref().to_path_buf());
        let source_graph = resolve_source_graph(&session, explicit_source_graph_path.as_deref())?;
        let mut queue = ActionQueue::new();
        queue.reserve_action_ids_after(max_action_id(&session));
        let transport = transport_clock_from_state(&session, source_graph.as_ref());
        let jam_view = JamViewModel::build(&session, &queue, source_graph.as_ref());
        let runtime_view = JamRuntimeView::build(&AppRuntimeState::default(), &session);
        let mut state = Self {
            files: Some(JamFileSet {
                session_path,
                source_graph_path: explicit_source_graph_path,
            }),
            session,
            source_graph,
            queue,
            runtime: AppRuntimeState {
                transport,
                ..AppRuntimeState::default()
            },
            jam_view,
            runtime_view,
        };
        state.refresh_view();
        Ok(state)
    }

    pub fn analyze_source_file_to_json(
        source_path: impl AsRef<Path>,
        session_path: impl AsRef<Path>,
        source_graph_path: Option<PathBuf>,
        sidecar_script_path: impl AsRef<Path>,
        analysis_seed: u64,
    ) -> Result<Self, JamAppError> {
        let source_path = source_path.as_ref().canonicalize()?;
        let session_path = session_path.as_ref().to_path_buf();

        let mut client = StdioSidecarClient::spawn_python(sidecar_script_path)?;
        let pong = client.ping()?;
        let graph = client.analyze_source_file(&source_path, analysis_seed)?;

        let session =
            session_from_ingested_graph(&graph, &source_path, source_graph_path.as_deref())?;
        if let Some(source_graph_path) = source_graph_path.as_deref() {
            save_source_graph_json(source_graph_path, &graph)?;
        }
        save_session_json(&session_path, &session)?;

        let mut state = Self::from_json_files(&session_path, source_graph_path.as_deref())?;
        state.set_sidecar_state(SidecarState::Ready {
            version: Some(pong.sidecar_version),
            transport: "stdio-ndjson".into(),
        });
        Ok(state)
    }

    pub fn refresh_view(&mut self) {
        self.runtime.tr909_render = build_tr909_render_state(
            &self.session,
            &self.runtime.transport,
            self.source_graph.as_ref(),
        );
        self.runtime.w30_preview = build_w30_preview_render_state(
            &self.session,
            &self.runtime.transport,
            self.source_graph.as_ref(),
        );
        self.jam_view = JamViewModel::build(&self.session, &self.queue, self.source_graph.as_ref());
        self.runtime_view = JamRuntimeView::build(&self.runtime, &self.session);
    }

    pub fn set_audio_health(&mut self, health: AudioRuntimeHealth) {
        self.runtime.audio = Some(health);
        self.runtime_view = JamRuntimeView::build(&self.runtime, &self.session);
    }

    pub fn set_sidecar_state(&mut self, state: SidecarState) {
        self.runtime.sidecar = state;
        self.runtime_view = JamRuntimeView::build(&self.runtime, &self.session);
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
        self.runtime.transport_driver.last_pulse_at_ms = None;
    }

    pub fn set_transport_playing_at(&mut self, is_playing: bool, now_ms: TimestampMs) {
        let next_clock = transport_clock_for_state(
            self.runtime.transport.position_beats,
            is_playing,
            self.runtime.transport.current_scene.clone(),
            self.source_graph.as_ref(),
        );
        self.update_transport_clock(next_clock);
        self.runtime.transport_driver.last_pulse_at_ms = is_playing.then_some(now_ms);
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

    pub fn apply_runtime_pulse(&mut self, now_ms: TimestampMs) -> Vec<CommittedActionRef> {
        if !self.runtime.transport.is_playing {
            self.runtime.transport_driver.last_pulse_at_ms = None;
            return Vec::new();
        }

        let Some(previous_pulse_at) = self.runtime.transport_driver.last_pulse_at_ms else {
            self.runtime.transport_driver.last_pulse_at_ms = Some(now_ms);
            return Vec::new();
        };

        if now_ms <= previous_pulse_at {
            self.runtime.transport_driver.last_pulse_at_ms = Some(now_ms);
            return Vec::new();
        }

        self.runtime.transport_driver.last_pulse_at_ms = Some(now_ms);
        let delta_beats = self.beats_for_elapsed_ms(now_ms - previous_pulse_at);
        self.advance_transport_by(delta_beats, now_ms)
    }

    pub fn queue_scene_mutation(&mut self, requested_at: TimestampMs) {
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::MutateScene,
            Quantization::NextBar,
            riotbox_core::action::ActionTarget {
                scope: Some(TargetScope::Scene),
                scene_id: self.session.runtime_state.scene_state.active_scene.clone(),
                ..Default::default()
            },
        );
        draft.params = ActionParams::Mutation {
            intensity: self.session.runtime_state.macro_state.chaos,
            target_id: self
                .session
                .runtime_state
                .scene_state
                .active_scene
                .as_ref()
                .map(ToString::to_string),
        };
        draft.explanation = Some("mutate current scene on next bar".into());
        self.queue.enqueue(draft, requested_at);
        self.refresh_view();
    }

    fn mc202_phrase_control_pending(&self) -> bool {
        self.queue.pending_actions().iter().any(|action| {
            matches!(
                action.command,
                ActionCommand::Mc202SetRole
                    | ActionCommand::Mc202GenerateFollower
                    | ActionCommand::Mc202GenerateAnswer
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
            && self
                .session
                .runtime_state
                .lane_state
                .tr909
                .takeover_profile
                .as_deref()
                == Some("scene_lock_takeover")
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

    fn recallable_w30_capture(&self) -> Option<&CaptureRef> {
        self.session
            .captures
            .iter()
            .rev()
            .find(|capture| capture.is_pinned && capture_targets_w30_pad(capture))
            .or_else(|| {
                self.session
                    .captures
                    .iter()
                    .rev()
                    .find(|capture| capture_targets_w30_pad(capture))
            })
    }

    fn auditionable_w30_capture(&self) -> Option<&CaptureRef> {
        self.session
            .captures
            .iter()
            .rev()
            .find(|capture| capture_targets_w30_pad(capture))
    }

    fn triggerable_w30_capture(&self) -> Option<&CaptureRef> {
        self.session
            .runtime_state
            .lane_state
            .w30
            .last_capture
            .as_ref()
            .and_then(|capture_id| {
                self.session
                    .captures
                    .iter()
                    .find(|capture| capture.capture_id == *capture_id)
            })
            .or_else(|| self.recallable_w30_capture())
    }

    fn w30_pad_cue_pending(&self) -> bool {
        self.queue.pending_actions().into_iter().any(|action| {
            matches!(
                action.command,
                ActionCommand::W30SwapBank
                    | ActionCommand::W30AuditionPromoted
                    | ActionCommand::W30TriggerPad
            )
        })
    }

    pub fn undo_last_action(&mut self, requested_at: TimestampMs) -> Option<Action> {
        let next_undo_action_id = next_action_id_from_session(&self.session);

        let undone = self
            .session
            .action_log
            .actions
            .iter_mut()
            .rev()
            .find(|action| {
                action.status == ActionStatus::Committed
                    && matches!(
                        action.undo_policy,
                        riotbox_core::action::UndoPolicy::Undoable
                    )
            })?;

        undone.status = ActionStatus::Undone;
        undone.result = Some(ActionResult {
            accepted: true,
            summary: format!("undone by user at {requested_at}"),
        });

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
        if let Some(capture) =
            capture_ref_from_action(&self.session, self.source_graph.as_ref(), action)
        {
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
    }

    pub fn save(&self) -> Result<(), JamAppError> {
        if let Some(files) = &self.files {
            let mut session_to_save = self.session.clone();
            sync_graph_refs_with_state(
                &mut session_to_save,
                self.source_graph.as_ref(),
                files.source_graph_path.as_deref(),
            );
            save_session_json(&files.session_path, &session_to_save)?;

            if let Some(source_graph) = &self.source_graph
                && let Some(source_graph_path) = resolve_external_graph_path(
                    &session_to_save,
                    files.source_graph_path.as_deref(),
                )
            {
                save_source_graph_json(source_graph_path, source_graph)?;
            }
        }

        Ok(())
    }

    fn beats_for_elapsed_ms(&self, elapsed_ms: TimestampMs) -> f64 {
        let bpm = self
            .jam_view
            .source
            .bpm_estimate
            .map(f64::from)
            .filter(|bpm| *bpm > 0.0)
            .unwrap_or(120.0);

        bpm * Duration::from_millis(elapsed_ms).as_secs_f64() / 60.0
    }
}

fn session_from_ingested_graph(
    graph: &SourceGraph,
    source_path: &Path,
    source_graph_path: Option<&Path>,
) -> Result<SessionFile, JamAppError> {
    let timestamp = timestamp_now();
    let source_id = SourceId::from(graph.source.source_id.as_str());
    let graph_hash = source_graph_hash(graph)?;

    let mut session = SessionFile::new(
        format!("session-{}", graph.source.source_id.as_str()),
        env!("CARGO_PKG_VERSION"),
        timestamp.clone(),
    );
    session.updated_at = timestamp;
    session.source_refs.push(SourceRef {
        source_id: source_id.clone(),
        path_hint: source_path.to_string_lossy().into_owned(),
        content_hash: graph.source.content_hash.clone(),
        duration_seconds: graph.source.duration_seconds,
        decode_profile: decode_profile_label(&graph.source.decode_profile),
    });
    session.source_graph_refs.push(SourceGraphRef {
        source_id,
        graph_version: graph.graph_version,
        graph_hash,
        storage_mode: if source_graph_path.is_some() {
            GraphStorageMode::External
        } else {
            GraphStorageMode::Embedded
        },
        embedded_graph: source_graph_path.is_none().then(|| graph.clone()),
        external_path: source_graph_path.map(|path| path.to_string_lossy().into_owned()),
        provenance: graph.provenance.clone(),
    });
    // Keep the music bus open enough that W-30 preview work is audible in fresh ingest sessions.
    session.runtime_state.mixer_state.music_level = 0.64;
    session.notes = Some("session created from analysis ingest slice".into());

    Ok(session)
}

fn source_graph_hash(graph: &SourceGraph) -> Result<String, JamAppError> {
    let encoded = serde_json::to_vec(graph)?;
    Ok(format!("sha256:{:x}", Sha256::digest(encoded)))
}

fn decode_profile_label(profile: &DecodeProfile) -> String {
    match profile {
        DecodeProfile::Native => "native".into(),
        DecodeProfile::NormalizedStereo => "normalized_stereo".into(),
        DecodeProfile::NormalizedMono => "normalized_mono".into(),
        DecodeProfile::Custom(value) => value.clone(),
    }
}

fn timestamp_now() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("unix_ms:{millis}")
}

fn transport_clock_from_state(
    session: &SessionFile,
    source_graph: Option<&SourceGraph>,
) -> TransportClockState {
    transport_clock_for_state(
        session.runtime_state.transport.position_beats,
        session.runtime_state.transport.is_playing,
        session.runtime_state.transport.current_scene.clone(),
        source_graph,
    )
}

fn transport_clock_for_state(
    position_beats: f64,
    is_playing: bool,
    current_scene: Option<riotbox_core::ids::SceneId>,
    source_graph: Option<&SourceGraph>,
) -> TransportClockState {
    let beat_index = position_beats.floor() as u64;
    let beats_per_bar = source_graph
        .and_then(|graph| {
            graph
                .timing
                .meter_hint
                .as_ref()
                .map(|meter| u64::from(meter.beats_per_bar))
        })
        .filter(|beats| *beats > 0)
        .unwrap_or(4);
    let bar_index = ((beat_index.saturating_sub(1)) / beats_per_bar).saturating_add(1);
    let phrase_index = source_graph
        .and_then(|graph| {
            graph
                .timing
                .phrase_grid
                .iter()
                .find(|phrase| {
                    let start_beat = (u64::from(phrase.start_bar).saturating_sub(1)
                        * beats_per_bar)
                        .saturating_add(1);
                    let end_beat = u64::from(phrase.end_bar) * beats_per_bar;
                    beat_index >= start_beat && beat_index <= end_beat
                })
                .map(|phrase| u64::from(phrase.phrase_index))
        })
        .unwrap_or_else(|| ((bar_index.saturating_sub(1)) / 8).saturating_add(1));

    TransportClockState {
        is_playing,
        position_beats,
        beat_index,
        bar_index,
        phrase_index,
        current_scene,
    }
}

fn crossed_commit_boundary(
    previous: &TransportClockState,
    next: &TransportClockState,
) -> Option<CommitBoundaryState> {
    if next.phrase_index > previous.phrase_index {
        return Some(next.boundary_state(riotbox_core::action::CommitBoundary::Phrase));
    }

    if next.bar_index > previous.bar_index {
        return Some(next.boundary_state(riotbox_core::action::CommitBoundary::Bar));
    }

    if next.beat_index > previous.beat_index {
        return Some(next.boundary_state(riotbox_core::action::CommitBoundary::Beat));
    }

    None
}

fn next_action_id_from_session(session: &SessionFile) -> riotbox_core::ids::ActionId {
    riotbox_core::ids::ActionId(
        max_action_id(session)
            .map(|id| id.0.saturating_add(1))
            .unwrap_or(1),
    )
}

fn next_capture_id(session: &SessionFile) -> CaptureId {
    CaptureId::from(format!(
        "cap-{:02}",
        session.captures.len().saturating_add(1)
    ))
}

fn capture_ref_from_action(
    session: &SessionFile,
    source_graph: Option<&SourceGraph>,
    action: &Action,
) -> Option<CaptureRef> {
    let capture_type = match action.command {
        ActionCommand::CaptureNow | ActionCommand::CaptureLoop => CaptureType::Loop,
        ActionCommand::CaptureBarGroup | ActionCommand::W30CaptureToPad => CaptureType::Pad,
        _ => return None,
    };

    let assigned_target = match action.command {
        ActionCommand::W30CaptureToPad => session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .clone()
            .zip(session.runtime_state.lane_state.w30.focused_pad.clone())
            .map(|(bank_id, pad_id)| CaptureTarget::W30Pad { bank_id, pad_id }),
        _ => None,
    };

    let capture_id = next_capture_id(session);
    let source_origin_refs = source_graph
        .map(capture_origin_refs)
        .unwrap_or_else(|| vec!["source-graph-unavailable".into()]);

    Some(CaptureRef {
        storage_path: format!("captures/{capture_id}.wav"),
        notes: Some(capture_note(action)),
        capture_id,
        capture_type,
        source_origin_refs,
        created_from_action: Some(action.id),
        assigned_target,
        is_pinned: false,
    })
}

fn apply_capture_promotion_side_effects(session: &mut SessionFile, action: &Action) -> bool {
    if !matches!(
        action.command,
        ActionCommand::PromoteCaptureToPad | ActionCommand::PromoteCaptureToScene
    ) {
        return false;
    }

    let target = match promotion_target_from_action(session, action) {
        Some(target) => target,
        None => return false,
    };
    let capture_id = match promotion_capture_id(session, action) {
        Some(capture_id) => capture_id,
        None => return false,
    };

    let Some(capture) = session
        .captures
        .iter_mut()
        .find(|capture| capture.capture_id == capture_id)
    else {
        return false;
    };

    capture.assigned_target = Some(target.clone());
    capture.notes = Some(updated_capture_note(capture.notes.as_deref(), &target));

    session.runtime_state.lane_state.w30.last_capture = Some(capture.capture_id.clone());
    if let CaptureTarget::W30Pad { bank_id, pad_id } = target {
        session.runtime_state.lane_state.w30.active_bank = Some(bank_id);
        session.runtime_state.lane_state.w30.focused_pad = Some(pad_id);
    }

    true
}

fn capture_origin_refs(graph: &SourceGraph) -> Vec<String> {
    let mut refs = Vec::new();
    refs.push(graph.source.source_id.to_string());
    refs.extend(
        graph
            .candidates
            .iter()
            .take(2)
            .map(|candidate| candidate.asset_ref.to_string()),
    );
    refs.dedup();
    refs
}

fn capture_note(action: &Action) -> String {
    match &action.explanation {
        Some(explanation) if !explanation.is_empty() => explanation.clone(),
        _ => format!("capture committed from {}", action.command),
    }
}

fn promotion_capture_id(session: &SessionFile, action: &Action) -> Option<CaptureId> {
    match &action.params {
        ActionParams::Promotion {
            capture_id: Some(capture_id),
            ..
        } => Some(capture_id.clone()),
        _ => session
            .captures
            .last()
            .map(|capture| capture.capture_id.clone()),
    }
}

fn promotion_target_from_action(session: &SessionFile, action: &Action) -> Option<CaptureTarget> {
    match action.command {
        ActionCommand::PromoteCaptureToPad => action
            .target
            .bank_id
            .clone()
            .or_else(|| session.runtime_state.lane_state.w30.active_bank.clone())
            .zip(
                action
                    .target
                    .pad_id
                    .clone()
                    .or_else(|| session.runtime_state.lane_state.w30.focused_pad.clone()),
            )
            .map(|(bank_id, pad_id)| CaptureTarget::W30Pad { bank_id, pad_id }),
        ActionCommand::PromoteCaptureToScene => {
            action.target.scene_id.clone().map(CaptureTarget::Scene)
        }
        _ => None,
    }
}

fn promotion_note(target: &CaptureTarget) -> String {
    match target {
        CaptureTarget::W30Pad { bank_id, pad_id } => {
            format!("promoted to pad {bank_id}/{pad_id}")
        }
        CaptureTarget::Scene(scene_id) => format!("promoted to scene {scene_id}"),
    }
}

fn capture_promotion_summary(session: &SessionFile, action: &Action) -> Option<String> {
    let capture_id = promotion_capture_id(session, action)?;
    let capture = session
        .captures
        .iter()
        .find(|capture| capture.capture_id == capture_id)?;
    capture.notes.clone()
}

fn updated_capture_note(existing_notes: Option<&str>, target: &CaptureTarget) -> String {
    let promotion = promotion_note(target);
    match existing_notes {
        Some(existing_notes) => {
            let base = existing_notes
                .split(" | promoted to ")
                .next()
                .unwrap_or(existing_notes);
            format!("{base} | {promotion}")
        }
        None => promotion,
    }
}

fn capture_targets_w30_pad(capture: &CaptureRef) -> bool {
    matches!(capture.assigned_target, Some(CaptureTarget::W30Pad { .. }))
}

fn apply_w30_side_effects(
    session: &mut SessionFile,
    action: &Action,
    boundary: Option<&CommitBoundaryState>,
) {
    if !matches!(
        action.command,
        ActionCommand::W30SwapBank
            | ActionCommand::W30AuditionPromoted
            | ActionCommand::W30TriggerPad
    ) {
        return;
    }

    let Some(bank_id) = action.target.bank_id.clone() else {
        return;
    };
    let Some(pad_id) = action.target.pad_id.clone() else {
        return;
    };
    let capture_id = match &action.params {
        ActionParams::Mutation {
            target_id: Some(target_id),
            ..
        } => Some(CaptureId::from(target_id.clone())),
        _ => None,
    };

    session.runtime_state.lane_state.w30.active_bank = Some(bank_id.clone());
    session.runtime_state.lane_state.w30.focused_pad = Some(pad_id.clone());
    if let Some(capture_id) = capture_id.clone() {
        session.runtime_state.lane_state.w30.last_capture = Some(capture_id);
    }
    if matches!(
        action.command,
        ActionCommand::W30AuditionPromoted | ActionCommand::W30TriggerPad
    ) {
        let grit = match &action.params {
            ActionParams::Mutation { intensity, .. } => match action.command {
                ActionCommand::W30AuditionPromoted => intensity.clamp(0.0, 1.0),
                ActionCommand::W30TriggerPad => (intensity * 0.82).clamp(0.0, 1.0),
                _ => unreachable!("checked above"),
            },
            _ => 0.68,
        };
        session.runtime_state.macro_state.w30_grit =
            session.runtime_state.macro_state.w30_grit.max(grit);
    }

    if let Some(logged_action) = session
        .action_log
        .actions
        .iter_mut()
        .rev()
        .find(|logged_action| logged_action.id == action.id)
    {
        let summary = match action.command {
            ActionCommand::W30SwapBank => capture_id.as_ref().map_or_else(
                || format!("recalled W-30 pad {bank_id}/{pad_id}"),
                |capture_id| format!("recalled {capture_id} on W-30 pad {bank_id}/{pad_id}"),
            ),
            ActionCommand::W30AuditionPromoted => capture_id.as_ref().map_or_else(
                || format!("auditioned W-30 pad {bank_id}/{pad_id}"),
                |capture_id| format!("auditioned {capture_id} on W-30 pad {bank_id}/{pad_id}"),
            ),
            ActionCommand::W30TriggerPad => {
                let position = boundary.map_or_else(
                    || "beat pending".to_string(),
                    |boundary| {
                        format!(
                            "beat {} / phrase {}",
                            boundary.beat_index, boundary.phrase_index
                        )
                    },
                );
                capture_id.as_ref().map_or_else(
                    || format!("triggered W-30 pad {bank_id}/{pad_id} at {position}"),
                    |capture_id| {
                        format!(
                            "triggered {capture_id} on W-30 pad {bank_id}/{pad_id} at {position}"
                        )
                    },
                )
            }
            _ => unreachable!("checked above"),
        };
        logged_action.result = Some(ActionResult {
            accepted: true,
            summary,
        });
    }
}

fn apply_tr909_side_effects(
    session: &mut SessionFile,
    action: &Action,
    boundary: Option<&CommitBoundaryState>,
) {
    match action.command {
        ActionCommand::Tr909SetSlam => {
            let intensity = match &action.params {
                ActionParams::Mutation { intensity, .. } => intensity.clamp(0.0, 1.0),
                _ => session.runtime_state.macro_state.tr909_slam,
            };
            session.runtime_state.macro_state.tr909_slam = intensity;
            session.runtime_state.lane_state.tr909.slam_enabled = intensity > 0.0;

            if let Some(logged_action) = session
                .action_log
                .actions
                .iter_mut()
                .rev()
                .find(|logged_action| logged_action.id == action.id)
            {
                let summary = if intensity > 0.0 {
                    format!("enabled TR-909 slam at {:.2}", intensity)
                } else {
                    "disabled TR-909 slam".into()
                };
                logged_action.result = Some(ActionResult {
                    accepted: true,
                    summary,
                });
            }
        }
        ActionCommand::Tr909FillNext => {
            session.runtime_state.lane_state.tr909.fill_armed_next_bar = false;
            session.runtime_state.lane_state.tr909.last_fill_bar =
                boundary.map(|boundary| boundary.bar_index);
            session.runtime_state.lane_state.tr909.pattern_ref =
                boundary.map(|boundary| format!("fill-bar-{}", boundary.bar_index));
            session.runtime_state.lane_state.tr909.reinforcement_mode = Some("fills".into());
        }
        ActionCommand::Tr909ReinforceBreak => {
            session.runtime_state.lane_state.tr909.reinforcement_mode =
                Some("break_reinforce".into());
            session.runtime_state.lane_state.tr909.pattern_ref = boundary.map(|boundary| {
                boundary.scene_id.as_ref().map_or_else(
                    || format!("reinforce-phrase-{}", boundary.phrase_index),
                    |scene_id| format!("reinforce-{scene_id}"),
                )
            });
        }
        ActionCommand::Tr909Takeover => {
            session.runtime_state.lane_state.tr909.takeover_enabled = true;
            session.runtime_state.lane_state.tr909.takeover_profile =
                Some("controlled_phrase_takeover".into());
            session.runtime_state.lane_state.tr909.pattern_ref = boundary.map(|boundary| {
                boundary.scene_id.as_ref().map_or_else(
                    || format!("takeover-phrase-{}", boundary.phrase_index),
                    |scene_id| format!("takeover-{scene_id}"),
                )
            });
            session.runtime_state.lane_state.tr909.reinforcement_mode = Some("takeover".into());
        }
        ActionCommand::Tr909SceneLock => {
            session.runtime_state.lane_state.tr909.takeover_enabled = true;
            session.runtime_state.lane_state.tr909.takeover_profile =
                Some("scene_lock_takeover".into());
            session.runtime_state.lane_state.tr909.pattern_ref = boundary.map(|boundary| {
                boundary.scene_id.as_ref().map_or_else(
                    || format!("lock-phrase-{}", boundary.phrase_index),
                    |scene_id| format!("lock-{scene_id}"),
                )
            });
            session.runtime_state.lane_state.tr909.reinforcement_mode = Some("takeover".into());
        }
        ActionCommand::Tr909Release => {
            session.runtime_state.lane_state.tr909.takeover_enabled = false;
            session.runtime_state.lane_state.tr909.takeover_profile = None;
            session.runtime_state.lane_state.tr909.pattern_ref = boundary.map(|boundary| {
                boundary.scene_id.as_ref().map_or_else(
                    || format!("release-phrase-{}", boundary.phrase_index),
                    |scene_id| format!("release-{scene_id}"),
                )
            });
            if session
                .runtime_state
                .lane_state
                .tr909
                .reinforcement_mode
                .as_deref()
                == Some("takeover")
            {
                session.runtime_state.lane_state.tr909.reinforcement_mode =
                    Some("source_support".into());
            }
        }
        _ => {}
    }
}

fn apply_mc202_side_effects(
    session: &mut SessionFile,
    action: &Action,
    boundary: Option<&CommitBoundaryState>,
) {
    match action.command {
        ActionCommand::Mc202SetRole => {
            let Some(role) = action
                .target
                .object_id
                .clone()
                .or_else(|| match &action.params {
                    ActionParams::Mutation { target_id, .. } => target_id.clone(),
                    _ => None,
                })
            else {
                return;
            };

            session.runtime_state.lane_state.mc202.role = Some(role.clone());
            session.runtime_state.lane_state.mc202.phrase_ref =
                Some(boundary_phrase_ref(boundary, &role));

            let touch = match &action.params {
                ActionParams::Mutation { intensity, .. } => intensity.clamp(0.0, 1.0),
                _ if role == "leader" => 0.85,
                _ => 0.65,
            };
            session.runtime_state.macro_state.mc202_touch = touch;

            if let Some(logged_action) = session
                .action_log
                .actions
                .iter_mut()
                .rev()
                .find(|logged_action| logged_action.id == action.id)
            {
                logged_action.result = Some(ActionResult {
                    accepted: true,
                    summary: format!("set MC-202 role to {role} at {touch:.2}"),
                });
            }
        }
        ActionCommand::Mc202GenerateFollower => {
            let role = "follower";
            let phrase_ref = boundary_phrase_ref(boundary, role);
            let touch = match &action.params {
                ActionParams::Mutation { intensity, .. } => intensity.clamp(0.0, 1.0),
                _ => 0.78,
            };

            session.runtime_state.lane_state.mc202.role = Some(role.into());
            session.runtime_state.lane_state.mc202.phrase_ref = Some(phrase_ref.clone());
            session.runtime_state.macro_state.mc202_touch =
                session.runtime_state.macro_state.mc202_touch.max(touch);

            if let Some(logged_action) = session
                .action_log
                .actions
                .iter_mut()
                .rev()
                .find(|logged_action| logged_action.id == action.id)
            {
                logged_action.result = Some(ActionResult {
                    accepted: true,
                    summary: format!(
                        "generated MC-202 follower phrase {phrase_ref} at {:.2}",
                        session.runtime_state.macro_state.mc202_touch
                    ),
                });
            }
        }
        ActionCommand::Mc202GenerateAnswer => {
            let role = "answer";
            let phrase_ref = boundary_phrase_ref(boundary, role);
            let touch = match &action.params {
                ActionParams::Mutation { intensity, .. } => intensity.clamp(0.0, 1.0),
                _ => 0.82,
            };

            session.runtime_state.lane_state.mc202.role = Some(role.into());
            session.runtime_state.lane_state.mc202.phrase_ref = Some(phrase_ref.clone());
            session.runtime_state.macro_state.mc202_touch =
                session.runtime_state.macro_state.mc202_touch.max(touch);

            if let Some(logged_action) = session
                .action_log
                .actions
                .iter_mut()
                .rev()
                .find(|logged_action| logged_action.id == action.id)
            {
                logged_action.result = Some(ActionResult {
                    accepted: true,
                    summary: format!(
                        "generated MC-202 answer phrase {phrase_ref} at {:.2}",
                        session.runtime_state.macro_state.mc202_touch
                    ),
                });
            }
        }
        _ => {}
    }
}

fn boundary_phrase_ref(boundary: Option<&CommitBoundaryState>, role: &str) -> String {
    boundary.map_or_else(
        || format!("{role}-phrase"),
        |boundary| {
            boundary.scene_id.as_ref().map_or_else(
                || format!("{role}-phrase-{}", boundary.phrase_index),
                |scene_id| format!("{role}-{scene_id}"),
            )
        },
    )
}

fn build_tr909_render_state(
    session: &SessionFile,
    transport: &TransportClockState,
    source_graph: Option<&SourceGraph>,
) -> Tr909RenderState {
    let tr909 = &session.runtime_state.lane_state.tr909;
    let mixer = &session.runtime_state.mixer_state;
    let tempo_bpm = source_graph
        .and_then(|graph| graph.timing.bpm_estimate)
        .unwrap_or(0.0);

    let mode = if tr909.takeover_enabled {
        Tr909RenderMode::Takeover
    } else {
        match tr909.reinforcement_mode.as_deref() {
            Some("fills") => Tr909RenderMode::Fill,
            Some("break_reinforce") => Tr909RenderMode::BreakReinforce,
            Some("takeover") => Tr909RenderMode::Takeover,
            Some("source_support") => Tr909RenderMode::SourceSupport,
            Some(_) => Tr909RenderMode::SourceSupport,
            None if tr909.pattern_ref.is_some() || tr909.slam_enabled => {
                Tr909RenderMode::SourceSupport
            }
            None => Tr909RenderMode::Idle,
        }
    };

    let routing = match mode {
        Tr909RenderMode::Idle => Tr909RenderRouting::SourceOnly,
        Tr909RenderMode::SourceSupport
        | Tr909RenderMode::Fill
        | Tr909RenderMode::BreakReinforce => Tr909RenderRouting::DrumBusSupport,
        Tr909RenderMode::Takeover => Tr909RenderRouting::DrumBusTakeover,
    };

    let source_support_profile = matches!(mode, Tr909RenderMode::SourceSupport)
        .then(|| derive_tr909_source_support_profile(source_graph, transport));
    let source_support_profile = source_support_profile.flatten();
    let takeover_profile = derive_tr909_takeover_render_profile(tr909);
    let pattern_adoption = derive_tr909_pattern_adoption(
        mode,
        tr909.pattern_ref.as_deref(),
        source_support_profile,
        takeover_profile,
    );
    let phrase_variation = derive_tr909_phrase_variation(
        mode,
        transport,
        tr909.pattern_ref.as_deref(),
        source_support_profile,
        takeover_profile,
    );

    Tr909RenderState {
        mode,
        routing,
        source_support_profile,
        pattern_ref: tr909.pattern_ref.clone(),
        pattern_adoption,
        phrase_variation,
        takeover_profile,
        drum_bus_level: mixer.drum_level.clamp(0.0, 1.0),
        slam_intensity: session.runtime_state.macro_state.tr909_slam.clamp(0.0, 1.0),
        is_transport_running: transport.is_playing,
        tempo_bpm,
        position_beats: transport.position_beats,
        current_scene_id: transport.current_scene.as_ref().map(ToString::to_string),
    }
}

fn derive_tr909_source_support_profile(
    source_graph: Option<&SourceGraph>,
    transport: &TransportClockState,
) -> Option<Tr909SourceSupportProfile> {
    let graph = source_graph?;
    let current_section = graph.sections.iter().find(|section| {
        let bar_index = transport.bar_index as u32;
        bar_index >= section.bar_start && bar_index <= section.bar_end
    });

    let profile = match current_section.map(|section| (section.label_hint, section.energy_class)) {
        Some((
            riotbox_core::source_graph::SectionLabelHint::Break
            | riotbox_core::source_graph::SectionLabelHint::Build,
            _,
        )) => Tr909SourceSupportProfile::BreakLift,
        Some((
            riotbox_core::source_graph::SectionLabelHint::Drop
            | riotbox_core::source_graph::SectionLabelHint::Chorus,
            riotbox_core::source_graph::EnergyClass::High
            | riotbox_core::source_graph::EnergyClass::Peak,
        )) => Tr909SourceSupportProfile::DropDrive,
        _ => Tr909SourceSupportProfile::SteadyPulse,
    };

    Some(profile)
}

fn build_w30_preview_render_state(
    session: &SessionFile,
    transport: &TransportClockState,
    source_graph: Option<&SourceGraph>,
) -> W30PreviewRenderState {
    let w30 = &session.runtime_state.lane_state.w30;
    let has_lane_focus =
        w30.active_bank.is_some() || w30.focused_pad.is_some() || w30.last_capture.is_some();
    if !has_lane_focus {
        return W30PreviewRenderState::default();
    }

    let mode = match last_committed_w30_preview_action(session).map(|action| action.command) {
        Some(ActionCommand::W30AuditionPromoted) => W30PreviewRenderMode::PromotedAudition,
        Some(ActionCommand::W30SwapBank | ActionCommand::W30TriggerPad) | None => {
            W30PreviewRenderMode::LiveRecall
        }
        Some(_) => unreachable!("filtered by helper"),
    };
    let last_trigger = last_committed_w30_trigger_action(session);

    let capture = w30.last_capture.as_ref().and_then(|capture_id| {
        session
            .captures
            .iter()
            .find(|capture| capture.capture_id == *capture_id)
    });
    let source_profile = match mode {
        W30PreviewRenderMode::Idle => None,
        W30PreviewRenderMode::PromotedAudition => Some(W30PreviewSourceProfile::PromotedAudition),
        W30PreviewRenderMode::LiveRecall => capture.map(|capture| {
            if capture.is_pinned {
                W30PreviewSourceProfile::PinnedRecall
            } else {
                W30PreviewSourceProfile::PromotedRecall
            }
        }),
    };
    let tempo_bpm = source_graph
        .and_then(|graph| graph.timing.bpm_estimate)
        .unwrap_or(0.0);

    W30PreviewRenderState {
        mode,
        routing: W30PreviewRenderRouting::MusicBusPreview,
        source_profile,
        active_bank_id: w30.active_bank.as_ref().map(ToString::to_string),
        focused_pad_id: w30.focused_pad.as_ref().map(ToString::to_string),
        capture_id: w30.last_capture.as_ref().map(ToString::to_string),
        trigger_revision: last_trigger.map_or(0, |action| action.id.0),
        trigger_velocity: last_trigger
            .and_then(|action| match &action.params {
                ActionParams::Mutation { intensity, .. } => Some(intensity.clamp(0.0, 1.0)),
                _ => None,
            })
            .unwrap_or(0.0),
        music_bus_level: session
            .runtime_state
            .mixer_state
            .music_level
            .clamp(0.0, 1.0),
        grit_level: session.runtime_state.macro_state.w30_grit.clamp(0.0, 1.0),
        is_transport_running: transport.is_playing,
        tempo_bpm,
        position_beats: transport.position_beats,
    }
}

fn last_committed_w30_preview_action(session: &SessionFile) -> Option<&Action> {
    session.action_log.actions.iter().rev().find(|action| {
        action.status == ActionStatus::Committed
            && matches!(
                action.command,
                ActionCommand::W30SwapBank
                    | ActionCommand::W30AuditionPromoted
                    | ActionCommand::W30TriggerPad
            )
    })
}

fn last_committed_w30_trigger_action(session: &SessionFile) -> Option<&Action> {
    session.action_log.actions.iter().rev().find(|action| {
        action.status == ActionStatus::Committed
            && matches!(action.command, ActionCommand::W30TriggerPad)
    })
}

fn derive_tr909_takeover_render_profile(
    tr909: &riotbox_core::session::Tr909LaneState,
) -> Option<Tr909TakeoverRenderProfile> {
    if !tr909.takeover_enabled {
        return None;
    }

    match tr909.takeover_profile.as_deref() {
        Some("controlled_phrase_takeover") => Some(Tr909TakeoverRenderProfile::ControlledPhrase),
        Some(_) | None => Some(Tr909TakeoverRenderProfile::SceneLock),
    }
}

fn resolve_source_graph(
    session: &SessionFile,
    explicit_source_graph_path: Option<&Path>,
) -> Result<Option<SourceGraph>, JamAppError> {
    if let Some(path) = explicit_source_graph_path {
        return Ok(Some(load_source_graph_json(path)?));
    }

    let Some(graph_ref) = session.source_graph_refs.first() else {
        return Ok(None);
    };

    match graph_ref.storage_mode {
        GraphStorageMode::Embedded => graph_ref.embedded_graph.clone().map(Some).ok_or_else(|| {
            JamAppError::InvalidSession(
                "source graph ref is embedded but embedded_graph is missing".into(),
            )
        }),
        GraphStorageMode::External => match graph_ref.external_path.as_deref() {
            Some(path) => Ok(Some(load_source_graph_json(path)?)),
            None => Err(JamAppError::InvalidSession(
                "source graph ref is external but external_path is missing".into(),
            )),
        },
    }
}

fn validate_mvp_single_source_session(session: &SessionFile) -> Result<(), JamAppError> {
    if session.source_refs.len() > 1 {
        return Err(JamAppError::InvalidSession(
            "Riotbox MVP currently supports exactly one source reference per session".into(),
        ));
    }

    if session.source_graph_refs.len() > 1 {
        return Err(JamAppError::InvalidSession(
            "Riotbox MVP currently supports exactly one source graph reference per session".into(),
        ));
    }

    if let (Some(source_ref), Some(graph_ref)) = (
        session.source_refs.first(),
        session.source_graph_refs.first(),
    ) && source_ref.source_id != graph_ref.source_id
    {
        return Err(JamAppError::InvalidSession(format!(
            "source ref {} does not match source graph ref {}",
            source_ref.source_id, graph_ref.source_id
        )));
    }

    Ok(())
}

fn sync_graph_refs_with_state(
    session: &mut SessionFile,
    source_graph: Option<&SourceGraph>,
    explicit_source_graph_path: Option<&Path>,
) {
    for graph_ref in &mut session.source_graph_refs {
        match graph_ref.storage_mode {
            GraphStorageMode::Embedded => {
                graph_ref.embedded_graph = source_graph.cloned();
            }
            GraphStorageMode::External => {
                if let Some(path) = explicit_source_graph_path {
                    graph_ref.external_path = Some(path.to_string_lossy().into_owned());
                }
            }
        }
    }
}

fn resolve_external_graph_path<'a>(
    session: &'a SessionFile,
    explicit_source_graph_path: Option<&'a Path>,
) -> Option<&'a Path> {
    if let Some(path) = explicit_source_graph_path {
        return Some(path);
    }

    session
        .source_graph_refs
        .iter()
        .find(|graph_ref| graph_ref.storage_mode == GraphStorageMode::External)
        .and_then(|graph_ref| graph_ref.external_path.as_deref())
        .map(Path::new)
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
mod tests {
    use std::{f32::consts::PI, fs, io, path::Path, path::PathBuf};

    use serde::Deserialize;
    use tempfile::tempdir;

    use riotbox_audio::{
        runtime::{AudioOutputInfo, AudioRuntimeHealth, AudioRuntimeLifecycle},
        tr909::{
            Tr909RenderMode, Tr909RenderRouting, Tr909SourceSupportProfile,
            Tr909TakeoverRenderProfile,
        },
    };
    use riotbox_core::{
        action::{
            Action, ActionCommand, ActionDraft, ActionParams, ActionResult, ActionStatus,
            ActionTarget, ActorType, CommitBoundary, GhostMode, Quantization, TargetScope,
            UndoPolicy,
        },
        ids::{
            ActionId, AssetId, BankId, CaptureId, PadId, SceneId, SectionId, SnapshotId, SourceId,
        },
        persistence::{
            load_session_json, load_source_graph_json, save_session_json, save_source_graph_json,
        },
        session::{
            CaptureRef, CaptureTarget, CaptureType, GhostBudgetState, GhostState,
            GhostSuggestionRecord, GraphStorageMode, SessionFile, Snapshot, SourceGraphRef,
            SourceRef,
        },
        source_graph::{
            AnalysisSummary, AnalysisWarning, Asset, AssetType, Candidate, CandidateType,
            DecodeProfile, EnergyClass, GraphProvenance, QualityClass, Relationship,
            RelationshipType, Section, SectionLabelHint, SourceDescriptor, SourceGraph,
            SourceGraphVersion,
        },
        transport::TransportClockState,
    };
    use riotbox_sidecar::client::ClientError as SidecarClientError;

    use super::*;

    #[derive(Debug, Deserialize)]
    struct RenderProjectionFixture {
        name: String,
        transport_position_beats: f64,
        reinforcement_mode: String,
        takeover_enabled: bool,
        takeover_profile: Option<String>,
        pattern_ref: Option<String>,
        expected_mode: String,
        expected_routing: String,
        expected_pattern_adoption: Option<String>,
        expected_phrase_variation: Option<String>,
        expected_source_support_profile: Option<String>,
        expected_takeover_profile: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct Mc202RegressionFixture {
        name: String,
        initial_role: String,
        action: Mc202RegressionAction,
        requested_at: TimestampMs,
        committed_at: TimestampMs,
        boundary: Mc202RegressionBoundary,
        expected: Mc202RegressionExpected,
    }

    #[derive(Clone, Copy, Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum Mc202RegressionAction {
        SetRole,
        GenerateFollower,
        GenerateAnswer,
    }

    #[derive(Debug, Deserialize)]
    struct Mc202RegressionBoundary {
        kind: Mc202RegressionBoundaryKind,
        beat_index: u64,
        bar_index: u64,
        phrase_index: u64,
        scene_id: Option<String>,
    }

    #[derive(Clone, Copy, Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum Mc202RegressionBoundaryKind {
        Immediate,
        Beat,
        HalfBar,
        Bar,
        Phrase,
        Scene,
    }

    #[derive(Debug, Deserialize)]
    struct Mc202RegressionExpected {
        role: String,
        phrase_ref: String,
        touch: f32,
        result_summary: String,
    }

    #[derive(Debug, Deserialize)]
    struct W30RegressionFixture {
        name: String,
        action: W30RegressionAction,
        capture_bank: String,
        capture_pad: String,
        capture_pinned: bool,
        requested_at: TimestampMs,
        committed_at: TimestampMs,
        boundary: W30RegressionBoundary,
        expected: W30RegressionExpected,
    }

    #[derive(Clone, Copy, Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum W30RegressionAction {
        LiveRecall,
        PromotedAudition,
    }

    #[derive(Debug, Deserialize)]
    struct W30RegressionBoundary {
        kind: W30RegressionBoundaryKind,
        beat_index: u64,
        bar_index: u64,
        phrase_index: u64,
        scene_id: Option<String>,
    }

    #[derive(Clone, Copy, Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum W30RegressionBoundaryKind {
        Immediate,
        Beat,
        HalfBar,
        Bar,
        Phrase,
        Scene,
    }

    #[derive(Debug, Deserialize)]
    struct W30RegressionExpected {
        active_bank: String,
        focused_pad: String,
        last_capture: String,
        w30_grit: f32,
        result_summary: String,
    }

    impl Mc202RegressionBoundary {
        fn into_commit_boundary_state(self) -> CommitBoundaryState {
            CommitBoundaryState {
                kind: match self.kind {
                    Mc202RegressionBoundaryKind::Immediate => CommitBoundary::Immediate,
                    Mc202RegressionBoundaryKind::Beat => CommitBoundary::Beat,
                    Mc202RegressionBoundaryKind::HalfBar => CommitBoundary::HalfBar,
                    Mc202RegressionBoundaryKind::Bar => CommitBoundary::Bar,
                    Mc202RegressionBoundaryKind::Phrase => CommitBoundary::Phrase,
                    Mc202RegressionBoundaryKind::Scene => CommitBoundary::Scene,
                },
                beat_index: self.beat_index,
                bar_index: self.bar_index,
                phrase_index: self.phrase_index,
                scene_id: self.scene_id.map(SceneId::from),
            }
        }
    }

    impl W30RegressionBoundary {
        fn into_commit_boundary_state(self) -> CommitBoundaryState {
            CommitBoundaryState {
                kind: match self.kind {
                    W30RegressionBoundaryKind::Immediate => CommitBoundary::Immediate,
                    W30RegressionBoundaryKind::Beat => CommitBoundary::Beat,
                    W30RegressionBoundaryKind::HalfBar => CommitBoundary::HalfBar,
                    W30RegressionBoundaryKind::Bar => CommitBoundary::Bar,
                    W30RegressionBoundaryKind::Phrase => CommitBoundary::Phrase,
                    W30RegressionBoundaryKind::Scene => CommitBoundary::Scene,
                },
                beat_index: self.beat_index,
                bar_index: self.bar_index,
                phrase_index: self.phrase_index,
                scene_id: self.scene_id.map(SceneId::from),
            }
        }
    }

    fn sample_audio_health(lifecycle: AudioRuntimeLifecycle) -> AudioRuntimeHealth {
        AudioRuntimeHealth {
            lifecycle,
            output: Some(AudioOutputInfo {
                host_name: "Alsa".into(),
                device_name: "default".into(),
                sample_format: "F32".into(),
                sample_rate: 44_100,
                channel_count: 2,
                buffer_size: "Default".into(),
                supported_output_config_count: Some(160),
            }),
            callback_count: 18,
            max_callback_gap_micros: Some(21_330),
            stream_error_count: u64::from(matches!(lifecycle, AudioRuntimeLifecycle::Faulted)),
            last_stream_error: matches!(lifecycle, AudioRuntimeLifecycle::Faulted)
                .then(|| "stream stalled".into()),
        }
    }

    fn sample_graph() -> SourceGraph {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: SourceId::from("src-1"),
                path: "input.wav".into(),
                content_hash: "hash-1".into(),
                duration_seconds: 120.0,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "0.1.0".into(),
                provider_set: vec!["beat".into(), "section".into()],
                generated_at: "2026-04-12T18:00:00Z".into(),
                source_hash: "hash-1".into(),
                analysis_seed: 7,
                run_notes: Some("app-test".into()),
            },
        );
        graph.sections.push(Section {
            section_id: SectionId::from("section-a"),
            label_hint: SectionLabelHint::Drop,
            start_seconds: 0.0,
            end_seconds: 16.0,
            bar_start: 1,
            bar_end: 8,
            energy_class: EnergyClass::High,
            confidence: 0.9,
            tags: vec!["main".into()],
        });
        graph.assets.push(Asset {
            asset_id: AssetId::from("asset-a"),
            asset_type: AssetType::LoopWindow,
            start_seconds: 0.0,
            end_seconds: 4.0,
            start_bar: 1,
            end_bar: 2,
            confidence: 0.8,
            tags: vec!["loop".into()],
            source_refs: vec!["src-1".into()],
        });
        graph.candidates.push(Candidate {
            candidate_id: "candidate-a".into(),
            candidate_type: CandidateType::LoopCandidate,
            asset_ref: "asset-a".into(),
            score: 0.88,
            confidence: 0.91,
            tags: vec!["useful".into()],
            constraints: vec!["bar_aligned".into()],
            provenance_refs: vec!["provider:beats".into()],
        });
        graph.relationships.push(Relationship {
            relation_type: RelationshipType::BelongsToSection,
            from_id: "asset-a".into(),
            to_id: "section-a".into(),
            weight: 1.0,
            notes: Some("primary loop".into()),
        });
        graph.timing.bpm_estimate = Some(126.0);
        graph.timing.bpm_confidence = 0.81;
        graph.analysis_summary = AnalysisSummary {
            overall_confidence: 0.87,
            timing_quality: QualityClass::High,
            section_quality: QualityClass::Medium,
            loop_candidate_count: 1,
            hook_candidate_count: 0,
            break_rebuild_potential: QualityClass::High,
            warnings: vec![AnalysisWarning {
                code: "low_hook_density".into(),
                message: "few hook fragments".into(),
            }],
        };
        graph
    }

    fn sample_session(graph: &SourceGraph) -> SessionFile {
        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-12T18:00:00Z");
        session.source_refs.push(SourceRef {
            source_id: SourceId::from("src-1"),
            path_hint: "input.wav".into(),
            content_hash: "hash-1".into(),
            duration_seconds: 120.0,
            decode_profile: "normalized_stereo".into(),
        });
        session.source_graph_refs.push(SourceGraphRef {
            source_id: SourceId::from("src-1"),
            graph_version: SourceGraphVersion::V1,
            graph_hash: "graph-hash-1".into(),
            storage_mode: GraphStorageMode::Embedded,
            embedded_graph: Some(graph.clone()),
            external_path: None,
            provenance: graph.provenance.clone(),
        });
        session.runtime_state.transport.is_playing = true;
        session.runtime_state.transport.position_beats = 32.0;
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-1"));
        session.runtime_state.macro_state.scene_aggression = 0.75;
        session.runtime_state.macro_state.tr909_slam = 0.55;
        session.runtime_state.lane_state.mc202.role = Some("follower".into());
        session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
        session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
        session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
        session.runtime_state.mixer_state.drum_level = 0.72;
        session.runtime_state.mixer_state.music_level = 0.64;
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-1"));
        session.runtime_state.scene_state.scenes = vec![SceneId::from("scene-1")];
        session.runtime_state.lock_state.locked_object_ids = vec!["ghost.main".into()];
        session.action_log.actions.push(Action {
            id: ActionId(1),
            actor: ActorType::User,
            command: ActionCommand::CaptureNow,
            params: ActionParams::Capture { bars: Some(2) },
            target: ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(BankId::from("bank-a")),
                pad_id: Some(PadId::from("pad-01")),
                ..Default::default()
            },
            requested_at: 100,
            quantization: Quantization::NextBar,
            status: ActionStatus::Committed,
            committed_at: Some(200),
            result: Some(ActionResult {
                accepted: true,
                summary: "captured".into(),
            }),
            undo_policy: UndoPolicy::Undoable,
            explanation: Some("capture current break".into()),
        });
        session.snapshots.push(Snapshot {
            snapshot_id: SnapshotId::from("snap-1"),
            created_at: "2026-04-12T18:05:00Z".into(),
            label: "first jam".into(),
            action_cursor: 1,
        });
        session.captures.push(CaptureRef {
            capture_id: CaptureId::from("cap-01"),
            capture_type: CaptureType::Pad,
            source_origin_refs: vec!["asset-a".into()],
            created_from_action: Some(ActionId(1)),
            storage_path: "captures/cap-01.wav".into(),
            assigned_target: None,
            is_pinned: false,
            notes: Some("keeper".into()),
        });
        session.ghost_state = GhostState {
            mode: GhostMode::Assist,
            budgets: GhostBudgetState {
                max_actions_per_phrase: 2,
                max_destructive_actions_per_scene: 1,
                max_pending_actions: 2,
            },
            suggestion_history: vec![GhostSuggestionRecord {
                proposal_id: "gp-1".into(),
                summary: "capture next bar".into(),
                accepted: false,
            }],
            lock_awareness_enabled: true,
        };
        session.notes = Some("keeper session".into());
        session
    }

    fn sidecar_script_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../python/sidecar/json_stdio_sidecar.py")
            .canonicalize()
            .expect("resolve sidecar script path")
    }

    fn write_pcm16_wave(
        path: impl AsRef<Path>,
        sample_rate: u32,
        channel_count: u16,
        duration_seconds: f32,
    ) {
        let path = path.as_ref();
        let frame_count = (sample_rate as f32 * duration_seconds) as u32;
        let bits_per_sample = 16_u16;
        let bytes_per_sample = (bits_per_sample / 8) as u32;
        let byte_rate = sample_rate * channel_count as u32 * bytes_per_sample;
        let block_align = channel_count * (bits_per_sample / 8);
        let data_len = frame_count * channel_count as u32 * bytes_per_sample;

        let mut bytes = Vec::with_capacity((44 + data_len) as usize);
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&channel_count.to_le_bytes());
        bytes.extend_from_slice(&sample_rate.to_le_bytes());
        bytes.extend_from_slice(&byte_rate.to_le_bytes());
        bytes.extend_from_slice(&block_align.to_le_bytes());
        bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());

        for frame_index in 0..frame_count {
            let phase = (frame_index as f32 / sample_rate as f32) * 220.0 * 2.0 * PI;
            let sample = (phase.sin() * i16::MAX as f32 * 0.25) as i16;
            for _ in 0..channel_count {
                bytes.extend_from_slice(&sample.to_le_bytes());
            }
        }

        fs::write(path, bytes).expect("write PCM wave fixture");
    }

    #[test]
    fn builds_jam_app_state_from_parts() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert!(state.jam_view.transport.is_playing);
        assert_eq!(state.jam_view.scene.scene_count, 1);
        assert_eq!(state.jam_view.lanes.mc202_role.as_deref(), Some("follower"));
        assert_eq!(state.runtime_view.audio_status, "unknown");
        assert_eq!(state.runtime_view.sidecar_status, "unknown");
    }

    #[test]
    fn loads_and_saves_jam_app_state_from_files() {
        let dir = tempdir().expect("create temp dir");
        let session_path = dir.path().join("sessions").join("session.json");
        let graph_path = dir.path().join("graphs").join("source-graph.json");

        let graph = sample_graph();
        let session = sample_session(&graph);
        save_session_json(&session_path, &session).expect("save session fixture");
        save_source_graph_json(&graph_path, &graph).expect("save graph fixture");

        let mut state =
            JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("load app state");
        assert!(state.jam_view.transport.is_playing);
        assert_eq!(state.jam_view.source.section_count, 1);

        state.session.notes = Some("updated".into());
        state.refresh_view();
        state.save().expect("save app state");

        let persisted_session = load_session_json(&session_path).expect("reload session");
        let persisted_graph = load_source_graph_json(&graph_path).expect("reload graph");

        assert_eq!(persisted_session.notes.as_deref(), Some("updated"));
        assert_eq!(persisted_graph, graph);
    }

    #[test]
    fn loads_embedded_graph_session_without_separate_graph_file() {
        let dir = tempdir().expect("create temp dir");
        let session_path = dir.path().join("sessions").join("session.json");

        let graph = sample_graph();
        let session = sample_session(&graph);
        save_session_json(&session_path, &session).expect("save embedded session fixture");

        let state =
            JamAppState::from_json_files(&session_path, None::<&Path>).expect("load app state");

        assert_eq!(state.source_graph, Some(graph));
        assert_eq!(state.jam_view.source.section_count, 1);
    }

    #[test]
    fn save_persists_embedded_graph_into_session_file() {
        let dir = tempdir().expect("create temp dir");
        let session_path = dir.path().join("sessions").join("session.json");

        let graph = sample_graph();
        let session = sample_session(&graph);
        save_session_json(&session_path, &session).expect("save embedded session fixture");

        let mut state =
            JamAppState::from_json_files(&session_path, None::<&Path>).expect("load app state");
        state.session.notes = Some("updated embedded session".into());
        state.save().expect("save app state");

        let persisted_session = load_session_json(&session_path).expect("reload session");
        let persisted_graph = persisted_session.source_graph_refs[0]
            .embedded_graph
            .clone()
            .expect("embedded graph should persist");

        assert_eq!(
            persisted_session.notes.as_deref(),
            Some("updated embedded session")
        );
        assert_eq!(persisted_graph, graph);
    }

    #[test]
    fn rejects_session_with_multiple_source_refs_in_mvp_mode() {
        let dir = tempdir().expect("create temp dir");
        let session_path = dir.path().join("jam-session.json");
        let graph = sample_graph();
        let mut session = sample_session(&graph);
        session.source_refs.push(SourceRef {
            source_id: SourceId::from("src-2"),
            path_hint: "other.wav".into(),
            content_hash: "hash-2".into(),
            duration_seconds: 64.0,
            decode_profile: "normalized_stereo".into(),
        });
        save_session_json(&session_path, &session).expect("save multi-source session fixture");

        let error = JamAppState::from_json_files(&session_path, None::<&Path>)
            .expect_err("load should fail");

        match error {
            JamAppError::InvalidSession(message) => {
                assert!(message.contains("exactly one source reference"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn rejects_session_with_mismatched_single_source_and_graph_refs() {
        let dir = tempdir().expect("create temp dir");
        let session_path = dir.path().join("jam-session.json");
        let graph = sample_graph();
        let mut session = sample_session(&graph);
        session.source_graph_refs[0].source_id = SourceId::from("src-other");
        save_session_json(&session_path, &session).expect("save mismatched session fixture");

        let error = JamAppState::from_json_files(&session_path, None::<&Path>)
            .expect_err("load should fail");

        match error {
            JamAppError::InvalidSession(message) => {
                assert!(message.contains("does not match source graph ref"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn runtime_view_updates_from_audio_and_sidecar_state() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.set_audio_health(sample_audio_health(AudioRuntimeLifecycle::Running));
        state.set_sidecar_state(SidecarState::Ready {
            version: Some("0.1.0".into()),
            transport: "stdio-ndjson".into(),
        });

        assert_eq!(state.runtime_view.audio_status, "running");
        assert_eq!(state.runtime_view.audio_callback_count, 18);
        assert_eq!(state.runtime_view.sidecar_status, "ready");
        assert_eq!(state.runtime_view.sidecar_version.as_deref(), Some("0.1.0"));
        assert!(state.runtime_view.runtime_warnings.is_empty());
    }

    #[test]
    fn runtime_view_surfaces_faulted_and_degraded_states() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.set_audio_health(sample_audio_health(AudioRuntimeLifecycle::Faulted));
        state.set_sidecar_state(SidecarState::Degraded {
            reason: "worker restart pending".into(),
        });

        assert_eq!(state.runtime_view.audio_status, "faulted");
        assert_eq!(
            state.runtime_view.audio_last_error.as_deref(),
            Some("stream stalled")
        );
        assert_eq!(state.runtime_view.sidecar_status, "degraded");
        assert!(
            state
                .runtime_view
                .runtime_warnings
                .iter()
                .any(|warning| warning == "audio runtime faulted")
        );
        assert!(
            state
                .runtime_view
                .runtime_warnings
                .iter()
                .any(|warning| warning.contains("sidecar degraded"))
        );
    }

    #[test]
    fn runtime_view_surfaces_tr909_render_diagnostics() {
        let graph = sample_graph();
        let mut session = sample_session(&graph);
        session.runtime_state.lane_state.tr909.takeover_enabled = true;
        session.runtime_state.lane_state.tr909.takeover_profile =
            Some("controlled_phrase_takeover".into());
        session.runtime_state.lane_state.tr909.pattern_ref = Some("scene-1-main".into());
        session.runtime_state.macro_state.tr909_slam = 0.91;
        session.runtime_state.mixer_state.drum_level = 0.0;
        let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(state.runtime_view.tr909_render_mode, "takeover");
        assert_eq!(state.runtime_view.tr909_render_routing, "drum_bus_takeover");
        assert_eq!(state.runtime_view.tr909_render_profile, "controlled_phrase");
        assert_eq!(
            state.runtime_view.tr909_render_pattern_ref.as_deref(),
            Some("scene-1-main")
        );
        assert_eq!(
            state.runtime_view.tr909_render_pattern_adoption,
            "takeover_grid"
        );
        assert_eq!(
            state.runtime_view.tr909_render_phrase_variation,
            "phrase_lift"
        );
        assert_eq!(
            state.runtime_view.tr909_render_mix_summary,
            "drum bus 0.00 | slam 0.91"
        );
        assert_eq!(
            state.runtime_view.tr909_render_alignment,
            "takeover aligned"
        );
        assert!(
            state
                .runtime_view
                .tr909_render_transport_summary
                .contains("running @ 32.0 beats")
        );
        assert!(
            state
                .runtime_view
                .runtime_warnings
                .iter()
                .any(|warning| warning == "909 render is routed to the drum bus at zero drum level")
        );
    }

    #[test]
    fn adjusting_drum_bus_level_updates_session_and_runtime_view() {
        let graph = sample_graph();
        let mut session = sample_session(&graph);
        session.runtime_state.lane_state.tr909.takeover_enabled = true;
        session.runtime_state.lane_state.tr909.takeover_profile =
            Some("controlled_phrase_takeover".into());
        session.runtime_state.lane_state.tr909.pattern_ref = Some("scene-1-main".into());
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        let raised = state.adjust_drum_bus_level(0.18);
        assert!((raised - 0.90).abs() < f32::EPSILON);
        assert!((state.session.runtime_state.mixer_state.drum_level - 0.90).abs() < f32::EPSILON);
        assert_eq!(
            state.runtime_view.tr909_render_mix_summary,
            "drum bus 0.90 | slam 0.55"
        );

        let lowered = state.adjust_drum_bus_level(-1.5);
        assert_eq!(lowered, 0.0);
        assert_eq!(state.session.runtime_state.mixer_state.drum_level, 0.0);
        assert_eq!(
            state.runtime_view.tr909_render_mix_summary,
            "drum bus 0.00 | slam 0.55"
        );
        assert!(state
            .runtime_view
            .runtime_warnings
            .iter()
            .any(|warning| warning == "909 render is routed to the drum bus at zero drum level"));
    }

    #[test]
    fn updates_transport_clock_and_refreshes_jam_state() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        let clock = TransportClockState {
            is_playing: false,
            position_beats: 48.5,
            beat_index: 48,
            bar_index: 13,
            phrase_index: 4,
            current_scene: Some(SceneId::from("scene-2")),
        };

        state.update_transport_clock(clock.clone());

        assert_eq!(state.runtime.transport, clock);
        assert!(!state.session.runtime_state.transport.is_playing);
        assert_eq!(state.session.runtime_state.transport.position_beats, 48.5);
        assert_eq!(
            state
                .session
                .runtime_state
                .transport
                .current_scene
                .as_ref()
                .map(ToString::to_string),
            Some("scene-2".into())
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .scene_state
                .active_scene
                .as_ref()
                .map(ToString::to_string),
            Some("scene-2".into())
        );
        assert!(!state.jam_view.transport.is_playing);
        assert_eq!(state.jam_view.transport.position_beats, 48.5);
        assert_eq!(
            state.jam_view.scene.active_scene.as_deref(),
            Some("scene-2")
        );
    }

    #[test]
    fn setting_transport_playing_at_records_runtime_driver_anchor() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.set_transport_playing_at(true, 1_000);

        assert!(state.runtime.transport.is_playing);
        assert_eq!(state.runtime.transport_driver.last_pulse_at_ms, Some(1_000));

        state.set_transport_playing_at(false, 1_250);

        assert!(!state.runtime.transport.is_playing);
        assert_eq!(state.runtime.transport_driver.last_pulse_at_ms, None);
    }

    #[test]
    fn reconstructs_bar_and_phrase_indices_from_loaded_state() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(state.runtime.transport.beat_index, 32);
        assert_eq!(state.runtime.transport.bar_index, 8);
        assert_eq!(state.runtime.transport.phrase_index, 1);
    }

    #[test]
    fn default_tr909_render_state_stays_idle_until_lane_state_requests_support() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(state.runtime.tr909_render.mode, Tr909RenderMode::Idle);
        assert_eq!(
            state.runtime.tr909_render.routing,
            Tr909RenderRouting::SourceOnly
        );
        assert_eq!(state.runtime.tr909_render.pattern_ref, None);
        assert_eq!(state.runtime.tr909_render.drum_bus_level, 0.72);
        assert!(state.runtime.tr909_render.is_transport_running);
        assert_eq!(state.runtime.tr909_render.tempo_bpm, 126.0);
        assert_eq!(state.runtime.tr909_render.position_beats, 32.0);
        assert_eq!(
            state.runtime.tr909_render.current_scene_id.as_deref(),
            Some("scene-1")
        );
    }

    #[test]
    fn commits_ready_actions_into_session_log_in_stable_order() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        let first = state.queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::CaptureNow,
                Quantization::NextBar,
                ActionTarget {
                    scope: Some(TargetScope::LaneW30),
                    ..Default::default()
                },
            ),
            300,
        );
        let second = state.queue.enqueue(
            ActionDraft::new(
                ActorType::Ghost,
                ActionCommand::MutateScene,
                Quantization::NextBar,
                ActionTarget {
                    scope: Some(TargetScope::Scene),
                    ..Default::default()
                },
            ),
            301,
        );

        let boundary = CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 64,
            bar_index: 17,
            phrase_index: 4,
            scene_id: Some(SceneId::from("scene-1")),
        };

        let committed = state.commit_ready_actions(boundary.clone(), 400);

        assert_eq!(committed.len(), 2);
        assert_eq!(committed[0].action_id, first);
        assert_eq!(committed[0].commit_sequence, 1);
        assert_eq!(committed[1].action_id, second);
        assert_eq!(committed[1].commit_sequence, 2);
        assert_eq!(state.runtime.last_commit_boundary, Some(boundary));
        assert_eq!(state.queue.pending_actions().len(), 0);
        assert_eq!(state.session.action_log.actions.len(), 3);
        assert_eq!(state.session.action_log.actions[1].id, first);
        assert_eq!(state.session.action_log.actions[2].id, second);
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>(),
            vec![ActionId(1), first, second]
        );
        assert_eq!(state.jam_view.pending_actions.len(), 0);
        assert_eq!(state.jam_view.recent_actions[0].id, second.to_string());
        assert_eq!(state.jam_view.recent_actions[1].id, first.to_string());
    }

    #[test]
    fn queues_first_live_safe_jam_actions() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.queue_scene_mutation(300);
        assert_eq!(
            state.queue_mc202_role_toggle(301),
            QueueControlResult::Enqueued
        );
        state.queue_tr909_fill(302);
        state.queue_tr909_reinforce(303);
        assert!(state.queue_tr909_slam_toggle(304));
        state.queue_capture_bar(305);
        assert!(state.queue_promote_last_capture(306));

        let pending = state.queue.pending_actions();

        assert_eq!(pending.len(), 7);
        assert_eq!(pending[0].command, ActionCommand::MutateScene);
        assert_eq!(pending[0].quantization, Quantization::NextBar);
        assert_eq!(pending[1].command, ActionCommand::Mc202SetRole);
        assert_eq!(pending[1].quantization, Quantization::NextPhrase);
        assert_eq!(pending[2].command, ActionCommand::Tr909FillNext);
        assert_eq!(pending[2].quantization, Quantization::NextBar);
        assert_eq!(pending[3].command, ActionCommand::Tr909ReinforceBreak);
        assert_eq!(pending[3].quantization, Quantization::NextPhrase);
        assert_eq!(pending[4].command, ActionCommand::Tr909SetSlam);
        assert_eq!(pending[4].quantization, Quantization::NextBeat);
        assert_eq!(pending[5].command, ActionCommand::CaptureBarGroup);
        assert_eq!(pending[5].quantization, Quantization::NextPhrase);
        assert_eq!(pending[6].command, ActionCommand::PromoteCaptureToPad);
        assert_eq!(pending[6].quantization, Quantization::NextBar);
        assert_eq!(
            state.jam_view.lanes.mc202_pending_role.as_deref(),
            Some("leader")
        );
        assert!(!state.jam_view.lanes.mc202_pending_follower_generation);
        assert!(!state.jam_view.lanes.mc202_pending_answer_generation);
        assert!(
            !state
                .session
                .runtime_state
                .lane_state
                .tr909
                .fill_armed_next_bar
        );
        assert!(state.jam_view.lanes.tr909_fill_armed_next_bar);
        assert_eq!(state.jam_view.pending_actions.len(), 7);
    }

    #[test]
    fn queueing_mc202_role_change_blocks_duplicate_pending_actions() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(
            state.queue_mc202_role_toggle(300),
            QueueControlResult::Enqueued
        );
        assert_eq!(
            state.queue_mc202_role_toggle(301),
            QueueControlResult::AlreadyPending
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::Mc202SetRole);
        assert_eq!(
            state.jam_view.lanes.mc202_pending_role.as_deref(),
            Some("leader")
        );
    }

    #[test]
    fn queueing_mc202_follower_generation_blocks_duplicate_pending_actions() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(
            state.queue_mc202_generate_follower(300),
            QueueControlResult::Enqueued
        );
        assert_eq!(
            state.queue_mc202_generate_follower(301),
            QueueControlResult::AlreadyPending
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::Mc202GenerateFollower);
        assert!(state.jam_view.lanes.mc202_pending_follower_generation);
        assert!(!state.jam_view.lanes.mc202_pending_answer_generation);
    }

    #[test]
    fn queueing_mc202_answer_generation_blocks_duplicate_pending_actions() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(
            state.queue_mc202_generate_answer(300),
            QueueControlResult::Enqueued
        );
        assert_eq!(
            state.queue_mc202_generate_answer(301),
            QueueControlResult::AlreadyPending
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::Mc202GenerateAnswer);
        assert!(!state.jam_view.lanes.mc202_pending_follower_generation);
        assert!(state.jam_view.lanes.mc202_pending_answer_generation);
    }

    #[test]
    fn queueing_mc202_role_and_generation_blocks_conflicting_phrase_controls() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph.clone()), ActionQueue::new());

        assert_eq!(
            state.queue_mc202_role_toggle(300),
            QueueControlResult::Enqueued
        );
        assert_eq!(
            state.queue_mc202_generate_follower(301),
            QueueControlResult::AlreadyPending
        );

        let mut other_state = JamAppState::from_parts(
            sample_session(&graph),
            Some(graph.clone()),
            ActionQueue::new(),
        );
        assert_eq!(
            other_state.queue_mc202_generate_follower(302),
            QueueControlResult::Enqueued
        );
        assert_eq!(
            other_state.queue_mc202_role_toggle(303),
            QueueControlResult::AlreadyPending
        );

        let mut answer_state =
            JamAppState::from_parts(sample_session(&graph), Some(graph), ActionQueue::new());
        assert_eq!(
            answer_state.queue_mc202_generate_answer(304),
            QueueControlResult::Enqueued
        );
        assert_eq!(
            answer_state.queue_mc202_role_toggle(305),
            QueueControlResult::AlreadyPending
        );
    }

    #[test]
    fn queueing_tr909_slam_blocks_duplicate_pending_slam_actions() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert!(state.queue_tr909_slam_toggle(300));
        assert!(!state.queue_tr909_slam_toggle(301));

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::Tr909SetSlam);
    }

    #[test]
    fn queueing_tr909_takeover_requires_clear_pending_and_inactive_state() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(
            state.queue_tr909_takeover(300),
            QueueControlResult::Enqueued
        );
        assert_eq!(
            state.queue_tr909_takeover(301),
            QueueControlResult::AlreadyPending
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::Tr909Takeover);
        assert_eq!(
            state.jam_view.lanes.tr909_takeover_pending_target,
            Some(true)
        );
        assert_eq!(
            state
                .jam_view
                .lanes
                .tr909_takeover_pending_profile
                .as_deref(),
            Some("controlled_phrase_takeover")
        );
        assert!(!state.jam_view.lanes.tr909_takeover_enabled);
    }

    #[test]
    fn queueing_tr909_scene_lock_requires_clear_pending_and_non_scene_lock_state() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph.clone()), ActionQueue::new());

        assert_eq!(
            state.queue_tr909_scene_lock(300),
            QueueControlResult::Enqueued
        );
        assert_eq!(
            state.queue_tr909_scene_lock(301),
            QueueControlResult::AlreadyPending
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::Tr909SceneLock);
        assert_eq!(
            state.jam_view.lanes.tr909_takeover_pending_target,
            Some(true)
        );
        assert_eq!(
            state
                .jam_view
                .lanes
                .tr909_takeover_pending_profile
                .as_deref(),
            Some("scene_lock_takeover")
        );
        assert!(!state.jam_view.lanes.tr909_takeover_enabled);

        let mut already_locked =
            JamAppState::from_parts(sample_session(&graph), Some(graph), ActionQueue::new());
        already_locked
            .session
            .runtime_state
            .lane_state
            .tr909
            .takeover_enabled = true;
        already_locked
            .session
            .runtime_state
            .lane_state
            .tr909
            .takeover_profile = Some("scene_lock_takeover".into());
        already_locked.refresh_view();

        assert_eq!(
            already_locked.queue_tr909_scene_lock(302),
            QueueControlResult::AlreadyInState
        );
    }

    #[test]
    fn queueing_tr909_release_requires_takeover_to_be_active() {
        let graph = sample_graph();
        let mut session = sample_session(&graph);
        session.runtime_state.lane_state.tr909.takeover_enabled = true;
        session.runtime_state.lane_state.tr909.takeover_profile =
            Some("controlled_phrase_takeover".into());
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(state.queue_tr909_release(300), QueueControlResult::Enqueued);
        assert_eq!(
            state.queue_tr909_release(301),
            QueueControlResult::AlreadyPending
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::Tr909Release);
        assert_eq!(
            state.jam_view.lanes.tr909_takeover_pending_target,
            Some(false)
        );
        assert_eq!(state.jam_view.lanes.tr909_takeover_pending_profile, None);
        assert!(state.jam_view.lanes.tr909_takeover_enabled);
    }

    #[test]
    fn advancing_transport_commits_crossed_bar_boundary() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.update_transport_clock(TransportClockState {
            is_playing: false,
            position_beats: 28.0,
            beat_index: 28,
            bar_index: 7,
            phrase_index: 1,
            current_scene: Some(SceneId::from("scene-1")),
        });
        state.set_transport_playing_at(true, 1_000);
        state.queue_tr909_fill(300);

        let committed = state.apply_runtime_pulse(3_100);

        assert_eq!(committed.len(), 1);
        assert_eq!(committed[0].boundary.kind, CommitBoundary::Bar);
        assert_eq!(state.queue.pending_actions().len(), 0);
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .map(|action| action.command),
            Some(ActionCommand::Tr909FillNext)
        );
    }

    #[test]
    fn runtime_pulse_advances_transport_from_elapsed_time() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.update_transport_clock(TransportClockState {
            is_playing: false,
            position_beats: 32.0,
            beat_index: 32,
            bar_index: 8,
            phrase_index: 1,
            current_scene: Some(SceneId::from("scene-1")),
        });
        state.set_transport_playing_at(true, 2_000);

        let committed = state.apply_runtime_pulse(2_500);

        assert!(committed.is_empty());
        assert!(state.runtime.transport.position_beats > 32.9);
        assert!(state.runtime.transport.position_beats < 33.1);
        assert_eq!(state.runtime.transport_driver.last_pulse_at_ms, Some(2_500));
    }

    #[test]
    fn committed_capture_actions_materialize_capture_refs() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.queue_capture_bar(300);

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Phrase,
                beat_index: 32,
                bar_index: 8,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            400,
        );

        assert_eq!(committed.len(), 1);
        assert_eq!(state.session.captures.len(), 2);
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .w30
                .last_capture
                .as_ref()
                .map(ToString::to_string),
            Some("cap-02".into())
        );
        assert_eq!(state.jam_view.capture.capture_count, 2);
        assert_eq!(
            state.jam_view.capture.last_capture_id.as_deref(),
            Some("cap-02")
        );
        assert_eq!(state.jam_view.capture.last_capture_target.as_deref(), None);
        assert_eq!(state.jam_view.capture.last_capture_origin_count, 2);
        assert_eq!(state.jam_view.capture.unassigned_capture_count, 2);
        assert_eq!(state.jam_view.capture.promoted_capture_count, 0);
    }

    #[test]
    fn committed_promotion_actions_assign_target_to_existing_capture() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.queue_promote_last_capture(300);

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Bar,
                beat_index: 33,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            400,
        );

        assert_eq!(committed.len(), 1);
        assert_eq!(state.session.captures.len(), 1);
        assert_eq!(
            state.session.captures[0].assigned_target,
            Some(CaptureTarget::W30Pad {
                bank_id: BankId::from("bank-a"),
                pad_id: PadId::from("pad-01"),
            })
        );
        assert_eq!(
            state.jam_view.capture.last_capture_target.as_deref(),
            Some("pad bank-a/pad-01")
        );
        assert_eq!(
            state.jam_view.capture.last_promotion_result.as_deref(),
            Some("promoted to pad bank-a/pad-01")
        );
    }

    #[test]
    fn second_promotion_updates_existing_capture_note_and_target() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.queue_promote_last_capture(300);
        state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Bar,
                beat_index: 33,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            400,
        );

        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-02"));
        assert!(state.queue_promote_last_capture(401));
        state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Bar,
                beat_index: 37,
                bar_index: 10,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            500,
        );

        assert_eq!(
            state.session.captures[0].assigned_target,
            Some(CaptureTarget::W30Pad {
                bank_id: BankId::from("bank-a"),
                pad_id: PadId::from("pad-02"),
            })
        );
        assert_eq!(
            state.session.captures[0].notes.as_deref(),
            Some("keeper | promoted to pad bank-a/pad-02")
        );
    }

    #[test]
    fn toggling_pin_latest_capture_updates_session_and_view() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(state.toggle_pin_latest_capture(), Some(true));
        assert!(state.session.captures[0].is_pinned);
        assert_eq!(state.jam_view.capture.pinned_capture_count, 1);
        assert_eq!(
            state.jam_view.capture.pinned_capture_ids,
            vec!["cap-01".to_string()]
        );

        assert_eq!(state.toggle_pin_latest_capture(), Some(false));
        assert!(!state.session.captures[0].is_pinned);
        assert_eq!(state.jam_view.capture.pinned_capture_count, 0);
        assert!(state.jam_view.capture.pinned_capture_ids.is_empty());
    }

    #[test]
    fn queue_w30_live_recall_prefers_latest_pinned_promoted_capture() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures.push(CaptureRef {
            capture_id: CaptureId::from("cap-02"),
            capture_type: CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-04".into(),
            }),
            is_pinned: false,
            notes: Some("secondary".into()),
        });
        state.session.captures.push(CaptureRef {
            capture_id: CaptureId::from("cap-03"),
            capture_type: CaptureType::Pad,
            source_origin_refs: vec!["asset-c".into()],
            created_from_action: None,
            storage_path: "captures/cap-03.wav".into(),
            assigned_target: Some(CaptureTarget::W30Pad {
                bank_id: "bank-c".into(),
                pad_id: "pad-07".into(),
            }),
            is_pinned: true,
            notes: Some("keeper".into()),
        });
        state.refresh_view();

        assert_eq!(
            state.queue_w30_live_recall(600),
            Some(QueueControlResult::Enqueued)
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::W30SwapBank);
        assert_eq!(
            pending[0].target.bank_id.as_ref().map(ToString::to_string),
            Some("bank-c".into())
        );
        assert_eq!(
            pending[0].target.pad_id.as_ref().map(ToString::to_string),
            Some("pad-07".into())
        );
        assert_eq!(
            pending[0].explanation.as_deref(),
            Some("recall cap-03 on W-30 pad bank-c/pad-07")
        );
        assert_eq!(
            state.jam_view.lanes.w30_pending_recall_target.as_deref(),
            Some("bank-c/pad-07")
        );
        assert_eq!(state.jam_view.lanes.w30_pending_audition_target, None);
    }

    #[test]
    fn queue_w30_promoted_audition_targets_latest_promoted_capture() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures.push(CaptureRef {
            capture_id: CaptureId::from("cap-02"),
            capture_type: CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-04".into(),
            }),
            is_pinned: false,
            notes: Some("secondary".into()),
        });
        state.refresh_view();

        assert_eq!(
            state.queue_w30_promoted_audition(620),
            Some(QueueControlResult::Enqueued)
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::W30AuditionPromoted);
        assert_eq!(
            pending[0].target.bank_id.as_ref().map(ToString::to_string),
            Some("bank-b".into())
        );
        assert_eq!(
            pending[0].target.pad_id.as_ref().map(ToString::to_string),
            Some("pad-04".into())
        );
        assert_eq!(
            pending[0].explanation.as_deref(),
            Some("audition promoted cap-02 on W-30 pad bank-b/pad-04")
        );
        assert_eq!(
            state.jam_view.lanes.w30_pending_audition_target.as_deref(),
            Some("bank-b/pad-04")
        );
        assert_eq!(state.jam_view.lanes.w30_pending_recall_target, None);
    }

    #[test]
    fn queue_w30_trigger_pad_targets_current_lane_capture_on_next_beat() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        });
        state.refresh_view();

        assert_eq!(
            state.queue_w30_trigger_pad(625),
            Some(QueueControlResult::Enqueued)
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::W30TriggerPad);
        assert_eq!(pending[0].quantization, Quantization::NextBeat);
        assert_eq!(
            pending[0].target.bank_id.as_ref().map(ToString::to_string),
            Some("bank-a".into())
        );
        assert_eq!(
            pending[0].target.pad_id.as_ref().map(ToString::to_string),
            Some("pad-01".into())
        );
        assert_eq!(
            pending[0].explanation.as_deref(),
            Some("trigger W-30 pad bank-a/pad-01 from cap-01 on next beat")
        );
        assert_eq!(
            state.jam_view.lanes.w30_pending_trigger_target.as_deref(),
            Some("bank-a/pad-01")
        );
        assert_eq!(state.jam_view.lanes.w30_pending_recall_target, None);
        assert_eq!(state.jam_view.lanes.w30_pending_audition_target, None);
    }

    #[test]
    fn queueing_w30_pad_cues_blocks_conflicting_pending_actions() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-b"),
            pad_id: PadId::from("pad-03"),
        });
        state.refresh_view();

        assert_eq!(
            state.queue_w30_live_recall(630),
            Some(QueueControlResult::Enqueued)
        );
        assert_eq!(
            state.queue_w30_promoted_audition(631),
            Some(QueueControlResult::AlreadyPending)
        );
        assert_eq!(
            state.queue_w30_trigger_pad(632),
            Some(QueueControlResult::AlreadyPending)
        );

        let other_graph = sample_graph();
        let mut other_state = JamAppState::from_parts(
            sample_session(&other_graph),
            Some(other_graph),
            ActionQueue::new(),
        );
        other_state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-c"),
            pad_id: PadId::from("pad-05"),
        });
        other_state.refresh_view();

        assert_eq!(
            other_state.queue_w30_promoted_audition(632),
            Some(QueueControlResult::Enqueued)
        );
        assert_eq!(
            other_state.queue_w30_live_recall(633),
            Some(QueueControlResult::AlreadyPending)
        );
        assert_eq!(
            other_state.queue_w30_trigger_pad(634),
            Some(QueueControlResult::AlreadyPending)
        );
    }

    #[test]
    fn committed_w30_live_recall_updates_lane_focus_and_log_result() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-b"),
            pad_id: PadId::from("pad-03"),
        });
        state.session.captures[0].is_pinned = true;
        state.refresh_view();

        assert_eq!(
            state.queue_w30_live_recall(610),
            Some(QueueControlResult::Enqueued)
        );

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Bar,
                beat_index: 33,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            700,
        );

        assert_eq!(committed.len(), 1);
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .w30
                .active_bank
                .as_ref()
                .map(ToString::to_string),
            Some("bank-b".into())
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .w30
                .focused_pad
                .as_ref()
                .map(ToString::to_string),
            Some("pad-03".into())
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .w30
                .last_capture
                .as_ref()
                .map(ToString::to_string),
            Some("cap-01".into())
        );
        assert_eq!(
            state.jam_view.lanes.w30_active_bank.as_deref(),
            Some("bank-b")
        );
        assert_eq!(
            state.jam_view.lanes.w30_focused_pad.as_deref(),
            Some("pad-03")
        );
        assert_eq!(state.jam_view.lanes.w30_pending_recall_target, None);
        assert_eq!(state.jam_view.lanes.w30_pending_audition_target, None);
        assert_eq!(
            state.runtime.w30_preview.mode,
            W30PreviewRenderMode::LiveRecall
        );
        assert_eq!(
            state.runtime.w30_preview.routing,
            W30PreviewRenderRouting::MusicBusPreview
        );
        assert_eq!(
            state.runtime.w30_preview.source_profile,
            Some(W30PreviewSourceProfile::PinnedRecall)
        );
        assert_eq!(
            state.runtime.w30_preview.capture_id.as_deref(),
            Some("cap-01")
        );
        assert_eq!(state.runtime_view.w30_preview_mode, "live_recall");
        assert_eq!(state.runtime_view.w30_preview_profile, "pinned_recall");
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some("recalled cap-01 on W-30 pad bank-b/pad-03")
        );
    }

    #[test]
    fn committed_w30_promoted_audition_updates_lane_focus_grit_and_log_result() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-b"),
            pad_id: PadId::from("pad-03"),
        });
        state.refresh_view();

        assert_eq!(
            state.queue_w30_promoted_audition(640),
            Some(QueueControlResult::Enqueued)
        );

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Bar,
                beat_index: 33,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            700,
        );

        assert_eq!(committed.len(), 1);
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .w30
                .active_bank
                .as_ref()
                .map(ToString::to_string),
            Some("bank-b".into())
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .w30
                .focused_pad
                .as_ref()
                .map(ToString::to_string),
            Some("pad-03".into())
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .w30
                .last_capture
                .as_ref()
                .map(ToString::to_string),
            Some("cap-01".into())
        );
        assert_eq!(state.session.runtime_state.macro_state.w30_grit, 0.68);
        assert_eq!(
            state.jam_view.lanes.w30_active_bank.as_deref(),
            Some("bank-b")
        );
        assert_eq!(
            state.jam_view.lanes.w30_focused_pad.as_deref(),
            Some("pad-03")
        );
        assert_eq!(state.jam_view.lanes.w30_pending_recall_target, None);
        assert_eq!(state.jam_view.lanes.w30_pending_audition_target, None);
        assert_eq!(
            state.runtime.w30_preview.mode,
            W30PreviewRenderMode::PromotedAudition
        );
        assert_eq!(
            state.runtime.w30_preview.routing,
            W30PreviewRenderRouting::MusicBusPreview
        );
        assert_eq!(
            state.runtime.w30_preview.source_profile,
            Some(W30PreviewSourceProfile::PromotedAudition)
        );
        assert_eq!(
            state.runtime.w30_preview.capture_id.as_deref(),
            Some("cap-01")
        );
        assert_eq!(state.runtime_view.w30_preview_mode, "promoted_audition");
        assert_eq!(state.runtime_view.w30_preview_profile, "promoted_audition");
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some("auditioned cap-01 on W-30 pad bank-b/pad-03")
        );
    }

    #[test]
    fn committed_w30_trigger_updates_preview_trigger_revision_and_log_result() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-b"),
            pad_id: PadId::from("pad-03"),
        });
        state.refresh_view();

        assert_eq!(
            state.queue_w30_trigger_pad(645),
            Some(QueueControlResult::Enqueued)
        );

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Beat,
                beat_index: 33,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            740,
        );

        assert_eq!(committed.len(), 1);
        assert_eq!(state.jam_view.lanes.w30_pending_trigger_target, None);
        assert_eq!(
            state.runtime.w30_preview.mode,
            W30PreviewRenderMode::LiveRecall
        );
        assert_eq!(
            state.runtime.w30_preview.capture_id.as_deref(),
            Some("cap-01")
        );
        assert_eq!(state.runtime.w30_preview.trigger_revision, 2);
        assert!((state.runtime.w30_preview.trigger_velocity - 0.84).abs() < f32::EPSILON);
        assert_eq!(state.runtime_view.w30_preview_mode, "live_recall");
        assert_eq!(state.runtime_view.w30_preview_profile, "promoted_recall");
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some("triggered cap-01 on W-30 pad bank-b/pad-03 at beat 33 / phrase 2")
        );
    }

    #[test]
    fn committed_tr909_actions_update_lane_state() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.queue_tr909_fill(300);
        state.queue_tr909_reinforce(301);

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Phrase,
                beat_index: 36,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            400,
        );

        assert_eq!(committed.len(), 2);
        assert_eq!(
            state.session.runtime_state.lane_state.tr909.last_fill_bar,
            Some(9)
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .tr909
                .pattern_ref
                .as_deref(),
            Some("reinforce-scene-1")
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .tr909
                .reinforcement_mode
                .as_deref(),
            Some("break_reinforce")
        );
        assert!(
            !state
                .session
                .runtime_state
                .lane_state
                .tr909
                .fill_armed_next_bar
        );
        assert_eq!(
            state.jam_view.lanes.tr909_reinforcement_mode.as_deref(),
            Some("break_reinforce")
        );
        assert_eq!(
            state.runtime.tr909_render.mode,
            Tr909RenderMode::BreakReinforce
        );
        assert_eq!(
            state.runtime.tr909_render.routing,
            Tr909RenderRouting::DrumBusSupport
        );
        assert_eq!(
            state.runtime.tr909_render.pattern_ref.as_deref(),
            Some("reinforce-scene-1")
        );
    }

    #[test]
    fn committed_mc202_role_change_updates_lane_state_and_macro_touch() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(
            state.queue_mc202_role_toggle(300),
            QueueControlResult::Enqueued
        );

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Phrase,
                beat_index: 36,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            400,
        );

        assert_eq!(committed.len(), 1);
        assert_eq!(
            state.session.runtime_state.lane_state.mc202.role.as_deref(),
            Some("leader")
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .mc202
                .phrase_ref
                .as_deref(),
            Some("leader-scene-1")
        );
        assert_eq!(state.session.runtime_state.macro_state.mc202_touch, 0.85);
        assert_eq!(state.jam_view.lanes.mc202_role.as_deref(), Some("leader"));
        assert_eq!(state.jam_view.lanes.mc202_pending_role, None);
        assert_eq!(
            state.jam_view.lanes.mc202_phrase_ref.as_deref(),
            Some("leader-scene-1")
        );
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some("set MC-202 role to leader at 0.85")
        );
    }

    #[test]
    fn committed_mc202_follower_generation_updates_phrase_ref_and_touch() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(
            state.queue_mc202_generate_follower(300),
            QueueControlResult::Enqueued
        );

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Phrase,
                beat_index: 36,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            400,
        );

        assert_eq!(committed.len(), 1);
        assert_eq!(
            state.session.runtime_state.lane_state.mc202.role.as_deref(),
            Some("follower")
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .mc202
                .phrase_ref
                .as_deref(),
            Some("follower-scene-1")
        );
        assert_eq!(state.session.runtime_state.macro_state.mc202_touch, 0.78);
        assert_eq!(state.jam_view.lanes.mc202_role.as_deref(), Some("follower"));
        assert!(!state.jam_view.lanes.mc202_pending_follower_generation);
        assert!(!state.jam_view.lanes.mc202_pending_answer_generation);
        assert_eq!(
            state.jam_view.lanes.mc202_phrase_ref.as_deref(),
            Some("follower-scene-1")
        );
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some("generated MC-202 follower phrase follower-scene-1 at 0.78")
        );
    }

    #[test]
    fn committed_mc202_answer_generation_updates_phrase_ref_and_touch() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(
            state.queue_mc202_generate_answer(300),
            QueueControlResult::Enqueued
        );

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Phrase,
                beat_index: 36,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            400,
        );

        assert_eq!(committed.len(), 1);
        assert_eq!(
            state.session.runtime_state.lane_state.mc202.role.as_deref(),
            Some("answer")
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .mc202
                .phrase_ref
                .as_deref(),
            Some("answer-scene-1")
        );
        assert_eq!(state.session.runtime_state.macro_state.mc202_touch, 0.82);
        assert_eq!(state.jam_view.lanes.mc202_role.as_deref(), Some("answer"));
        assert!(!state.jam_view.lanes.mc202_pending_follower_generation);
        assert!(!state.jam_view.lanes.mc202_pending_answer_generation);
        assert_eq!(
            state.jam_view.lanes.mc202_phrase_ref.as_deref(),
            Some("answer-scene-1")
        );
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some("generated MC-202 answer phrase answer-scene-1 at 0.82")
        );
    }

    #[test]
    fn mc202_fixture_backed_committed_state_regressions_hold() {
        let fixtures: Vec<Mc202RegressionFixture> =
            serde_json::from_str(include_str!("../tests/fixtures/mc202_regression.json"))
                .expect("parse MC-202 regression fixtures");

        for fixture in fixtures {
            let graph = sample_graph();
            let mut session = sample_session(&graph);
            session.runtime_state.lane_state.mc202.role = Some(fixture.initial_role.clone());
            let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

            let queue_result = match fixture.action {
                Mc202RegressionAction::SetRole => {
                    state.queue_mc202_role_toggle(fixture.requested_at)
                }
                Mc202RegressionAction::GenerateFollower => {
                    state.queue_mc202_generate_follower(fixture.requested_at)
                }
                Mc202RegressionAction::GenerateAnswer => {
                    state.queue_mc202_generate_answer(fixture.requested_at)
                }
            };
            assert_eq!(
                queue_result,
                QueueControlResult::Enqueued,
                "{} did not enqueue",
                fixture.name
            );

            let committed = state.commit_ready_actions(
                fixture.boundary.into_commit_boundary_state(),
                fixture.committed_at,
            );
            assert_eq!(
                committed.len(),
                1,
                "{} did not commit exactly one action",
                fixture.name
            );

            assert_eq!(
                state.session.runtime_state.lane_state.mc202.role.as_deref(),
                Some(fixture.expected.role.as_str()),
                "{} role drifted",
                fixture.name
            );
            assert_eq!(
                state
                    .session
                    .runtime_state
                    .lane_state
                    .mc202
                    .phrase_ref
                    .as_deref(),
                Some(fixture.expected.phrase_ref.as_str()),
                "{} phrase ref drifted",
                fixture.name
            );
            assert_eq!(
                state.session.runtime_state.macro_state.mc202_touch, fixture.expected.touch,
                "{} touch drifted",
                fixture.name
            );
            assert_eq!(
                state
                    .session
                    .action_log
                    .actions
                    .last()
                    .and_then(|action| action.result.as_ref())
                    .map(|result| result.summary.as_str()),
                Some(fixture.expected.result_summary.as_str()),
                "{} result summary drifted",
                fixture.name
            );
            assert!(
                state.jam_view.lanes.mc202_pending_role.is_none(),
                "{} left a pending role behind",
                fixture.name
            );
            assert!(
                !state.jam_view.lanes.mc202_pending_follower_generation,
                "{} left a pending follower-generation behind",
                fixture.name
            );
            assert!(
                !state.jam_view.lanes.mc202_pending_answer_generation,
                "{} left a pending answer-generation behind",
                fixture.name
            );

            let tempdir = tempdir().expect("create MC-202 regression tempdir");
            let session_path = tempdir.path().join(format!("{}.json", fixture.name));
            save_session_json(&session_path, &state.session)
                .expect("save MC-202 regression session");
            let loaded =
                load_session_json(&session_path).expect("reload MC-202 regression session");

            assert_eq!(
                loaded.runtime_state.lane_state.mc202.role.as_deref(),
                Some(fixture.expected.role.as_str()),
                "{} role did not survive replay roundtrip",
                fixture.name
            );
            assert_eq!(
                loaded.runtime_state.lane_state.mc202.phrase_ref.as_deref(),
                Some(fixture.expected.phrase_ref.as_str()),
                "{} phrase ref did not survive replay roundtrip",
                fixture.name
            );
            assert_eq!(
                loaded.runtime_state.macro_state.mc202_touch, fixture.expected.touch,
                "{} touch did not survive replay roundtrip",
                fixture.name
            );
            assert_eq!(
                loaded
                    .action_log
                    .actions
                    .last()
                    .and_then(|action| action.result.as_ref())
                    .map(|result| result.summary.as_str()),
                Some(fixture.expected.result_summary.as_str()),
                "{} result summary did not survive replay roundtrip",
                fixture.name
            );
        }
    }

    #[test]
    fn w30_fixture_backed_committed_state_regressions_hold() {
        let fixtures: Vec<W30RegressionFixture> =
            serde_json::from_str(include_str!("../tests/fixtures/w30_regression.json"))
                .expect("parse W-30 regression fixtures");

        for fixture in fixtures {
            let graph = sample_graph();
            let mut session = sample_session(&graph);
            session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
                bank_id: BankId::from(fixture.capture_bank.clone()),
                pad_id: PadId::from(fixture.capture_pad.clone()),
            });
            session.captures[0].is_pinned = fixture.capture_pinned;
            session.runtime_state.macro_state.w30_grit = 0.0;
            let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

            let queue_result = match fixture.action {
                W30RegressionAction::LiveRecall => {
                    state.queue_w30_live_recall(fixture.requested_at)
                }
                W30RegressionAction::PromotedAudition => {
                    state.queue_w30_promoted_audition(fixture.requested_at)
                }
            };
            assert_eq!(
                queue_result,
                Some(QueueControlResult::Enqueued),
                "{} did not enqueue",
                fixture.name
            );

            let committed = state.commit_ready_actions(
                fixture.boundary.into_commit_boundary_state(),
                fixture.committed_at,
            );
            assert_eq!(
                committed.len(),
                1,
                "{} did not commit exactly one action",
                fixture.name
            );

            assert_eq!(
                state
                    .session
                    .runtime_state
                    .lane_state
                    .w30
                    .active_bank
                    .as_ref()
                    .map(ToString::to_string),
                Some(fixture.expected.active_bank.clone()),
                "{} bank drifted",
                fixture.name
            );
            assert_eq!(
                state
                    .session
                    .runtime_state
                    .lane_state
                    .w30
                    .focused_pad
                    .as_ref()
                    .map(ToString::to_string),
                Some(fixture.expected.focused_pad.clone()),
                "{} pad drifted",
                fixture.name
            );
            assert_eq!(
                state
                    .session
                    .runtime_state
                    .lane_state
                    .w30
                    .last_capture
                    .as_ref()
                    .map(ToString::to_string),
                Some(fixture.expected.last_capture.clone()),
                "{} last capture drifted",
                fixture.name
            );
            assert_eq!(
                state.session.runtime_state.macro_state.w30_grit, fixture.expected.w30_grit,
                "{} grit drifted",
                fixture.name
            );
            assert_eq!(
                state
                    .session
                    .action_log
                    .actions
                    .last()
                    .and_then(|action| action.result.as_ref())
                    .map(|result| result.summary.as_str()),
                Some(fixture.expected.result_summary.as_str()),
                "{} result summary drifted",
                fixture.name
            );
        }
    }

    #[test]
    fn committed_tr909_slam_action_updates_lane_state_and_macro_intensity() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert!(state.queue_tr909_slam_toggle(300));

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Beat,
                beat_index: 33,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            400,
        );

        assert_eq!(committed.len(), 1);
        assert!(state.session.runtime_state.lane_state.tr909.slam_enabled);
        assert!(state.session.runtime_state.macro_state.tr909_slam >= 0.85);
        assert!(state.jam_view.lanes.tr909_slam_enabled);
        assert!(state.jam_view.macros.tr909_slam >= 0.85);
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some("enabled TR-909 slam at 0.85")
        );
    }

    #[test]
    fn committed_tr909_takeover_and_release_update_lane_state() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(
            state.queue_tr909_takeover(300),
            QueueControlResult::Enqueued
        );
        let committed_takeover = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Phrase,
                beat_index: 36,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            400,
        );

        assert_eq!(committed_takeover.len(), 1);
        assert!(
            state
                .session
                .runtime_state
                .lane_state
                .tr909
                .takeover_enabled
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .tr909
                .takeover_profile
                .as_deref(),
            Some("controlled_phrase_takeover")
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .tr909
                .reinforcement_mode
                .as_deref(),
            Some("takeover")
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .tr909
                .pattern_ref
                .as_deref(),
            Some("takeover-scene-1")
        );
        assert!(state.jam_view.lanes.tr909_takeover_enabled);
        assert_eq!(
            state.jam_view.lanes.tr909_takeover_profile.as_deref(),
            Some("controlled_phrase_takeover")
        );
        assert_eq!(state.jam_view.lanes.tr909_takeover_pending_profile, None);
        assert_eq!(state.jam_view.lanes.tr909_takeover_pending_target, None);
        assert_eq!(state.runtime.tr909_render.mode, Tr909RenderMode::Takeover);
        assert_eq!(
            state.runtime.tr909_render.routing,
            Tr909RenderRouting::DrumBusTakeover
        );
        assert_eq!(
            state.runtime.tr909_render.takeover_profile,
            Some(Tr909TakeoverRenderProfile::ControlledPhrase)
        );
        assert_eq!(
            state.runtime.tr909_render.phrase_variation,
            Some(Tr909PhraseVariation::PhraseLift)
        );

        state.update_transport_clock(TransportClockState {
            is_playing: true,
            position_beats: 64.0,
            beat_index: 64,
            bar_index: 16,
            phrase_index: 2,
            current_scene: Some(SceneId::from("scene-1")),
        });

        assert_eq!(
            state.queue_tr909_scene_lock(450),
            QueueControlResult::Enqueued
        );
        let committed_scene_lock = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Phrase,
                beat_index: 44,
                bar_index: 11,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            500,
        );

        assert_eq!(committed_scene_lock.len(), 1);
        assert!(
            state
                .session
                .runtime_state
                .lane_state
                .tr909
                .takeover_enabled
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .tr909
                .takeover_profile
                .as_deref(),
            Some("scene_lock_takeover")
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .tr909
                .pattern_ref
                .as_deref(),
            Some("lock-scene-1")
        );
        assert_eq!(
            state.jam_view.lanes.tr909_takeover_profile.as_deref(),
            Some("scene_lock_takeover")
        );
        assert_eq!(
            state.runtime.tr909_render.takeover_profile,
            Some(Tr909TakeoverRenderProfile::SceneLock)
        );
        assert_eq!(
            state.runtime.tr909_render.phrase_variation,
            Some(Tr909PhraseVariation::PhraseDrive)
        );

        state.update_transport_clock(TransportClockState {
            is_playing: true,
            position_beats: 32.0,
            beat_index: 32,
            bar_index: 8,
            phrase_index: 1,
            current_scene: Some(SceneId::from("scene-1")),
        });

        assert_eq!(state.queue_tr909_release(500), QueueControlResult::Enqueued);
        let committed_release = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Phrase,
                beat_index: 52,
                bar_index: 13,
                phrase_index: 3,
                scene_id: Some(SceneId::from("scene-1")),
            },
            600,
        );

        assert_eq!(committed_release.len(), 1);
        assert!(
            !state
                .session
                .runtime_state
                .lane_state
                .tr909
                .takeover_enabled
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .tr909
                .takeover_profile,
            None
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .tr909
                .reinforcement_mode
                .as_deref(),
            Some("source_support")
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .tr909
                .pattern_ref
                .as_deref(),
            Some("release-scene-1")
        );
        assert!(!state.jam_view.lanes.tr909_takeover_enabled);
        assert_eq!(state.jam_view.lanes.tr909_takeover_profile, None);
        assert_eq!(state.jam_view.lanes.tr909_takeover_pending_target, None);
        assert_eq!(
            state.runtime.tr909_render.mode,
            Tr909RenderMode::SourceSupport
        );
        assert_eq!(
            state.runtime.tr909_render.routing,
            Tr909RenderRouting::DrumBusSupport
        );
        assert_eq!(
            state.runtime.tr909_render.pattern_ref.as_deref(),
            Some("release-scene-1")
        );
        assert_eq!(
            state.runtime.tr909_render.source_support_profile,
            Some(Tr909SourceSupportProfile::DropDrive)
        );
        assert_eq!(
            state.runtime.tr909_render.pattern_adoption,
            Some(Tr909PatternAdoption::MainlineDrive)
        );
        assert_eq!(
            state.runtime.tr909_render.phrase_variation,
            Some(Tr909PhraseVariation::PhraseRelease)
        );
    }

    #[test]
    fn source_support_render_profile_tracks_current_source_section() {
        let mut graph = sample_graph();
        graph.sections.push(Section {
            section_id: SectionId::from("section-b"),
            label_hint: SectionLabelHint::Break,
            start_seconds: 16.0,
            end_seconds: 32.0,
            bar_start: 9,
            bar_end: 16,
            energy_class: EnergyClass::Medium,
            confidence: 0.86,
            tags: vec!["break".into()],
        });

        let mut session = sample_session(&graph);
        session.runtime_state.lane_state.tr909.reinforcement_mode = Some("source_support".into());
        session.runtime_state.lane_state.tr909.pattern_ref = Some("support-scene-1".into());
        session.runtime_state.transport.position_beats = 16.0;

        let state =
            JamAppState::from_parts(session.clone(), Some(graph.clone()), ActionQueue::new());

        assert_eq!(
            state.runtime.tr909_render.mode,
            Tr909RenderMode::SourceSupport
        );
        assert_eq!(
            state.runtime.tr909_render.source_support_profile,
            Some(Tr909SourceSupportProfile::DropDrive)
        );

        session.runtime_state.transport.position_beats = 36.0;
        let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(
            state.runtime.tr909_render.mode,
            Tr909RenderMode::SourceSupport
        );
        assert_eq!(
            state.runtime.tr909_render.source_support_profile,
            Some(Tr909SourceSupportProfile::BreakLift)
        );
        assert_eq!(
            state.runtime.tr909_render.pattern_adoption,
            Some(Tr909PatternAdoption::SupportPulse)
        );
    }

    #[test]
    fn pattern_adoption_can_be_derived_without_pattern_ref() {
        let graph = sample_graph();

        let mut support_session = sample_session(&graph);
        support_session
            .runtime_state
            .lane_state
            .tr909
            .reinforcement_mode = Some("source_support".into());
        support_session.runtime_state.lane_state.tr909.pattern_ref = None;
        let support_state =
            JamAppState::from_parts(support_session, Some(graph.clone()), ActionQueue::new());
        assert_eq!(
            support_state.runtime.tr909_render.pattern_adoption,
            Some(Tr909PatternAdoption::MainlineDrive)
        );

        let mut takeover_session = sample_session(&graph);
        takeover_session
            .runtime_state
            .lane_state
            .tr909
            .takeover_enabled = true;
        takeover_session
            .runtime_state
            .lane_state
            .tr909
            .takeover_profile = Some("controlled_phrase_takeover".into());
        takeover_session.runtime_state.lane_state.tr909.pattern_ref = None;
        let takeover_state =
            JamAppState::from_parts(takeover_session, Some(graph), ActionQueue::new());
        assert_eq!(
            takeover_state.runtime.tr909_render.pattern_adoption,
            Some(Tr909PatternAdoption::TakeoverGrid)
        );
        assert_eq!(
            takeover_state.runtime.tr909_render.phrase_variation,
            Some(Tr909PhraseVariation::PhraseLift)
        );
    }

    #[test]
    fn phrase_variation_tracks_phrase_context_and_release_patterns() {
        let graph = sample_graph();
        let mut session = sample_session(&graph);
        session.runtime_state.lane_state.tr909.reinforcement_mode = Some("source_support".into());
        session.runtime_state.lane_state.tr909.pattern_ref = Some("release-scene-1".into());
        let release_state =
            JamAppState::from_parts(session.clone(), Some(graph.clone()), ActionQueue::new());
        assert_eq!(
            release_state.runtime.tr909_render.phrase_variation,
            Some(Tr909PhraseVariation::PhraseRelease)
        );

        session.runtime_state.lane_state.tr909.pattern_ref = Some("scene-1-main".into());
        session.runtime_state.transport.position_beats = 64.0;
        let drive_state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
        assert_eq!(
            drive_state.runtime.tr909_render.phrase_variation,
            Some(Tr909PhraseVariation::PhraseAnchor)
        );
    }

    #[test]
    fn committed_state_fixture_backed_render_projections_hold() {
        let fixtures: Vec<RenderProjectionFixture> = serde_json::from_str(include_str!(
            "../tests/fixtures/tr909_committed_render_projection.json"
        ))
        .expect("parse committed render projection fixture");

        let graph = sample_graph();
        for fixture in fixtures {
            let mut session = sample_session(&graph);
            session.runtime_state.transport.position_beats = fixture.transport_position_beats;
            session.runtime_state.lane_state.tr909.reinforcement_mode =
                Some(fixture.reinforcement_mode.clone());
            session.runtime_state.lane_state.tr909.takeover_enabled = fixture.takeover_enabled;
            session.runtime_state.lane_state.tr909.takeover_profile =
                fixture.takeover_profile.clone();
            session.runtime_state.lane_state.tr909.pattern_ref = fixture.pattern_ref.clone();

            let state = JamAppState::from_parts(session, Some(graph.clone()), ActionQueue::new());

            assert_eq!(
                state.runtime.tr909_render.mode.label(),
                fixture.expected_mode,
                "{} render mode drifted",
                fixture.name
            );
            assert_eq!(
                state.runtime.tr909_render.routing.label(),
                fixture.expected_routing,
                "{} render routing drifted",
                fixture.name
            );
            assert_eq!(
                state
                    .runtime
                    .tr909_render
                    .pattern_adoption
                    .map(|pattern| pattern.label().to_string()),
                fixture.expected_pattern_adoption,
                "{} pattern adoption drifted",
                fixture.name
            );
            assert_eq!(
                state
                    .runtime
                    .tr909_render
                    .phrase_variation
                    .map(|variation| variation.label().to_string()),
                fixture.expected_phrase_variation,
                "{} phrase variation drifted",
                fixture.name
            );
            assert_eq!(
                state
                    .runtime
                    .tr909_render
                    .source_support_profile
                    .map(|profile| profile.label().to_string()),
                fixture.expected_source_support_profile,
                "{} support profile drifted",
                fixture.name
            );
            assert_eq!(
                state
                    .runtime
                    .tr909_render
                    .takeover_profile
                    .map(|profile| profile.label().to_string()),
                fixture.expected_takeover_profile,
                "{} takeover profile drifted",
                fixture.name
            );
        }
    }

    #[test]
    fn undo_marks_last_undoable_action_and_appends_undo_marker() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        let undo_action = state.undo_last_action(500).expect("undo latest action");

        assert_eq!(undo_action.command, ActionCommand::UndoLast);
        assert_eq!(state.session.action_log.actions.len(), 2);
        assert_eq!(
            state.session.action_log.actions[0].status,
            ActionStatus::Undone
        );
        assert_eq!(
            state.session.action_log.actions[1].command,
            ActionCommand::UndoLast
        );
        assert_eq!(state.jam_view.recent_actions[0].status, "committed");
        assert_eq!(state.jam_view.recent_actions[1].status, "undone");
    }

    #[test]
    fn saving_with_pending_tr909_fill_does_not_persist_committed_lane_state() {
        let dir = tempdir().expect("create temp dir");
        let session_path = dir.path().join("jam-session.json");
        let graph_path = dir.path().join("source-graph.json");
        let graph = sample_graph();
        let session = sample_session(&graph);

        save_session_json(&session_path, &session).expect("save session fixture");
        save_source_graph_json(&graph_path, &graph).expect("save graph fixture");

        let mut state =
            JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("load app state");
        state.queue_tr909_fill(700);

        assert!(state.jam_view.lanes.tr909_fill_armed_next_bar);
        assert!(
            !state
                .session
                .runtime_state
                .lane_state
                .tr909
                .fill_armed_next_bar
        );

        state.save().expect("save app state");

        let persisted_session = load_session_json(&session_path).expect("reload session");
        let reloaded =
            JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("reload app");

        assert!(
            !persisted_session
                .runtime_state
                .lane_state
                .tr909
                .fill_armed_next_bar
        );
        assert!(
            !reloaded
                .session
                .runtime_state
                .lane_state
                .tr909
                .fill_armed_next_bar
        );
        assert!(!reloaded.jam_view.lanes.tr909_fill_armed_next_bar);
        assert_eq!(reloaded.queue.pending_actions().len(), 0);
    }

    #[test]
    fn ingests_source_file_through_sidecar_and_persists_state() {
        let dir = tempdir().expect("create temp dir");
        let source_path = dir.path().join("input.wav");
        let session_path = dir.path().join("sessions").join("session.json");
        let graph_path = dir.path().join("graphs").join("source-graph.json");

        write_pcm16_wave(&source_path, 44_100, 2, 2.0);

        let state = JamAppState::analyze_source_file_to_json(
            &source_path,
            &session_path,
            Some(graph_path.clone()),
            sidecar_script_path(),
            29,
        )
        .expect("ingest source file");

        assert_eq!(state.runtime_view.sidecar_status, "ready");
        assert_eq!(state.runtime_view.sidecar_version.as_deref(), Some("0.1.0"));
        assert_eq!(
            state
                .source_graph
                .as_ref()
                .map(|graph| graph.source.path.clone()),
            Some(source_path.to_string_lossy().into_owned())
        );
        assert_eq!(state.session.source_refs.len(), 1);
        assert_eq!(state.session.source_graph_refs.len(), 1);
        assert_eq!(state.session.runtime_state.mixer_state.music_level, 0.64);
        assert_eq!(
            state.session.source_graph_refs[0].storage_mode,
            GraphStorageMode::External
        );
        assert_eq!(
            state.session.source_graph_refs[0].external_path.as_deref(),
            Some(graph_path.to_string_lossy().as_ref())
        );
        assert!(session_path.exists());
        assert!(graph_path.exists());

        let persisted_graph = load_source_graph_json(&graph_path).expect("reload graph");
        assert_eq!(
            persisted_graph.provenance.provider_set,
            vec!["decoded.wav_baseline"]
        );
        assert_eq!(persisted_graph.provenance.analysis_seed, 29);
        assert_eq!(persisted_graph.source.sample_rate, 44_100);
        assert_eq!(persisted_graph.source.channel_count, 2);
        assert!(persisted_graph.source.duration_seconds >= 1.9);
        assert!(persisted_graph.timing.bpm_estimate.is_some());
    }

    #[test]
    fn ingest_defaults_to_embedded_graph_storage_when_no_external_path_is_requested() {
        let dir = tempdir().expect("create temp dir");
        let source_path = dir.path().join("input.wav");
        let session_path = dir.path().join("sessions").join("session.json");

        write_pcm16_wave(&source_path, 44_100, 2, 2.0);

        let state = JamAppState::analyze_source_file_to_json(
            &source_path,
            &session_path,
            None,
            sidecar_script_path(),
            31,
        )
        .expect("ingest source file");

        assert_eq!(state.session.source_graph_refs.len(), 1);
        assert_eq!(state.session.runtime_state.mixer_state.music_level, 0.64);
        assert_eq!(
            state.session.source_graph_refs[0].storage_mode,
            GraphStorageMode::Embedded
        );
        assert!(state.session.source_graph_refs[0].external_path.is_none());
        assert!(state.session.source_graph_refs[0].embedded_graph.is_some());
        assert!(session_path.exists());
    }

    #[test]
    fn ingest_surfaces_missing_source_file_as_sidecar_error() {
        let dir = tempdir().expect("create temp dir");
        let source_path = dir.path().join("missing.wav");
        let session_path = dir.path().join("sessions").join("session.json");
        let graph_path = dir.path().join("graphs").join("source-graph.json");

        let error = JamAppState::analyze_source_file_to_json(
            &source_path,
            &session_path,
            Some(graph_path.clone()),
            sidecar_script_path(),
            29,
        )
        .expect_err("missing source should fail");

        match error {
            JamAppError::Io(io_error) => {
                assert_eq!(io_error.kind(), io::ErrorKind::NotFound);
            }
            JamAppError::Sidecar(SidecarClientError::Sidecar(payload)) => {
                assert_eq!(payload.code, "source_missing");
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}
