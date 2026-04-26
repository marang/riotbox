use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use riotbox_audio::{
    mc202::Mc202RenderState,
    runtime::{AudioRuntimeHealth, AudioRuntimeTimingSnapshot},
    source_audio::SourceAudioCache,
    tr909::Tr909RenderState,
    w30::{W30PreviewRenderState, W30ResampleTapState},
};
use riotbox_core::{
    TimestampMs,
    action::{
        Action, ActionCommand, ActionDraft, ActionParams, ActionResult, ActionStatus, ActionTarget,
        ActorType, Quantization, TargetScope,
    },
    ids::SourceId,
    persistence::{
        PersistenceError, load_session_json, load_source_graph_json, save_session_json,
        save_source_graph_json,
    },
    queue::{ActionQueue, CommittedActionRef},
    session::{
        CaptureTarget, GraphStorageMode, SessionFile, SourceGraphRef, SourceRef,
        Tr909TakeoverProfileState,
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
    pub queue: ActionQueue,
    pub runtime: AppRuntimeState,
    pub jam_view: JamViewModel,
    pub runtime_view: JamRuntimeView,
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
        let runtime_view = JamRuntimeView::build(&AppRuntimeState::default(), &session);
        let mut state = Self {
            files: None,
            session,
            source_graph,
            source_audio_cache: None,
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
        );
        self.runtime.w30_resample_tap =
            build_w30_resample_tap_state(&self.session, &self.runtime.transport);
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

        let (bank_id, pad_id, capture_id) = self.next_w30_slice_pool_capture()?;
        if self
            .session
            .runtime_state
            .lane_state
            .w30
            .last_capture
            .as_ref()
            == Some(&capture_id)
        {
            return Some(QueueControlResult::AlreadyInState);
        }

        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::W30BrowseSlicePool,
            Quantization::NextBeat,
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
            "browse W-30 slice pool to {capture_id} on {bank_id}/{pad_id} on next beat"
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
            capture_ref_from_action(&self.session, self.source_graph.as_ref(), action, boundary)
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
        apply_scene_side_effects(&mut self.session, action, Some(boundary));
        if matches!(
            action.command,
            ActionCommand::SceneLaunch | ActionCommand::SceneRestore
        ) {
            self.runtime.transport.current_scene =
                self.session.runtime_state.transport.current_scene.clone();
        }
    }
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
mod tests {
    use crate::test_support::{scene_energy_for_label, scene_label_hint};
    use std::{f32::consts::PI, fs, io, path::Path, path::PathBuf};

    use serde::Deserialize;
    use tempfile::tempdir;

    use riotbox_audio::{
        mc202::{Mc202PhraseShape, Mc202RenderMode, Mc202RenderRouting},
        runtime::{AudioOutputInfo, AudioRuntimeHealth, AudioRuntimeLifecycle},
        source_audio::SourceAudioCache,
        tr909::{
            Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
            Tr909SourceSupportContext, Tr909SourceSupportProfile, Tr909TakeoverRenderProfile,
        },
        w30::{
            W30_PREVIEW_SAMPLE_WINDOW_LEN, W30PreviewRenderMode, W30PreviewRenderRouting,
            W30PreviewSourceProfile, W30ResampleTapMode, W30ResampleTapRouting,
            W30ResampleTapSourceProfile,
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
            CaptureRef, CaptureSourceWindow, CaptureTarget, CaptureType, GhostBudgetState,
            GhostState, GhostSuggestionRecord, GraphStorageMode, SessionFile, Snapshot,
            SourceGraphRef, SourceRef, Tr909ReinforcementModeState, Tr909TakeoverProfileState,
            W30PreviewModeState,
        },
        source_graph::{
            AnalysisSummary, AnalysisWarning, Asset, AssetType, Candidate, CandidateType,
            DecodeProfile, EnergyClass, GraphProvenance, QualityClass, Relationship,
            RelationshipType, Section, SectionLabelHint, SourceDescriptor, SourceGraph,
            SourceGraphVersion,
        },
        transport::TransportClockState,
        view::jam::{CaptureTargetKindView, SceneJumpAvailabilityView, W30PendingAuditionKind},
    };
    use riotbox_sidecar::client::ClientError as SidecarClientError;

    use super::*;

    #[derive(Debug, Deserialize)]
    struct RenderProjectionFixture {
        name: String,
        transport_position_beats: f64,
        #[serde(default)]
        scene_context: Option<String>,
        reinforcement_mode: Tr909ReinforcementModeState,
        takeover_enabled: bool,
        takeover_profile: Option<Tr909TakeoverProfileState>,
        pattern_ref: Option<String>,
        expected_mode: String,
        expected_routing: String,
        expected_pattern_adoption: Option<String>,
        expected_phrase_variation: Option<String>,
        expected_source_support_profile: Option<String>,
        expected_source_support_context: Option<String>,
        expected_support_accent: String,
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

    #[derive(Debug, Deserialize)]
    struct SceneRegressionFixture {
        name: String,
        section_labels: Vec<String>,
        action: SceneRegressionAction,
        #[serde(default)]
        initial_active_scene: Option<String>,
        #[serde(default)]
        initial_current_scene: Option<String>,
        #[serde(default)]
        initial_restore_scene: Option<String>,
        #[serde(default)]
        tr909_reinforcement_mode: Option<Tr909ReinforcementModeState>,
        #[serde(default)]
        tr909_pattern_ref: Option<String>,
        #[serde(default)]
        requested_at: Option<TimestampMs>,
        #[serde(default)]
        committed_at: Option<TimestampMs>,
        #[serde(default)]
        boundary: Option<SceneRegressionBoundary>,
        expected: SceneRegressionExpected,
    }

    #[derive(Clone, Copy, Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum SceneRegressionAction {
        ProjectCandidates,
        SelectNextScene,
        RestoreScene,
    }

    #[derive(Debug, Deserialize)]
    struct SceneRegressionBoundary {
        kind: SceneRegressionBoundaryKind,
        beat_index: u64,
        bar_index: u64,
        phrase_index: u64,
        scene_id: Option<String>,
    }

    #[derive(Clone, Copy, Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum SceneRegressionBoundaryKind {
        Immediate,
        Beat,
        HalfBar,
        Bar,
        Phrase,
        Scene,
    }

    #[derive(Debug, Deserialize)]
    struct SceneRegressionExpected {
        scenes: Vec<String>,
        active_scene: String,
        current_scene: String,
        active_scene_energy: String,
        #[serde(default)]
        restore_scene: Option<String>,
        #[serde(default)]
        restore_scene_energy: Option<String>,
        #[serde(default)]
        result_summary: Option<String>,
        #[serde(default)]
        tr909_render_profile: Option<String>,
        #[serde(default)]
        tr909_render_support_context: Option<String>,
        #[serde(default)]
        tr909_render_support_accent: Option<String>,
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
        #[serde(default)]
        source_window: Option<W30RegressionSourceWindow>,
        #[serde(default = "default_true")]
        capture_assigned: bool,
        #[serde(default)]
        extra_captures: Vec<W30RegressionCapture>,
        #[serde(default)]
        initial_active_bank: Option<String>,
        #[serde(default)]
        initial_focused_pad: Option<String>,
        #[serde(default)]
        initial_last_capture: Option<String>,
        #[serde(default)]
        initial_preview_mode: Option<String>,
        #[serde(default)]
        initial_w30_grit: Option<f32>,
        requested_at: TimestampMs,
        committed_at: TimestampMs,
        boundary: W30RegressionBoundary,
        expected: W30RegressionExpected,
    }

    #[derive(Debug, Deserialize)]
    struct W30RegressionSourceWindow {
        source_id: String,
        start_seconds: f32,
        end_seconds: f32,
        start_frame: u64,
        end_frame: u64,
    }

    #[derive(Debug, Deserialize)]
    struct W30RegressionCapture {
        capture_id: String,
        bank: String,
        pad: String,
        pinned: bool,
        #[serde(default)]
        notes: Option<String>,
    }

    #[derive(Clone, Copy, Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum W30RegressionAction {
        LiveRecall,
        RawCaptureAudition,
        PromotedAudition,
        TriggerPad,
        SwapBank,
        ApplyDamageProfile,
        LoopFreeze,
        BrowseSlicePool,
    }

    fn default_true() -> bool {
        true
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
        #[serde(default)]
        preview_mode: Option<String>,
        #[serde(default)]
        preview_routing: Option<String>,
        #[serde(default)]
        preview_profile: Option<String>,
        #[serde(default)]
        preview_capture: Option<String>,
        #[serde(default)]
        preview_music_bus_level: Option<f32>,
        #[serde(default)]
        preview_grit_level: Option<f32>,
        #[serde(default)]
        preview_transport_running: Option<bool>,
        result_summary: String,
    }

    fn w30_preview_mode_state(value: &str) -> W30PreviewModeState {
        match value {
            "live_recall" => W30PreviewModeState::LiveRecall,
            "raw_capture_audition" => W30PreviewModeState::RawCaptureAudition,
            "promoted_audition" => W30PreviewModeState::PromotedAudition,
            other => panic!("unsupported W-30 preview mode fixture value: {other}"),
        }
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

    impl SceneRegressionBoundary {
        fn into_commit_boundary_state(self) -> CommitBoundaryState {
            CommitBoundaryState {
                kind: match self.kind {
                    SceneRegressionBoundaryKind::Immediate => CommitBoundary::Immediate,
                    SceneRegressionBoundaryKind::Beat => CommitBoundary::Beat,
                    SceneRegressionBoundaryKind::HalfBar => CommitBoundary::HalfBar,
                    SceneRegressionBoundaryKind::Bar => CommitBoundary::Bar,
                    SceneRegressionBoundaryKind::Phrase => CommitBoundary::Phrase,
                    SceneRegressionBoundaryKind::Scene => CommitBoundary::Scene,
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

    fn expected_w30_command(action: W30RegressionAction) -> ActionCommand {
        match action {
            W30RegressionAction::LiveRecall => ActionCommand::W30LiveRecall,
            W30RegressionAction::RawCaptureAudition => ActionCommand::W30AuditionRawCapture,
            W30RegressionAction::PromotedAudition => ActionCommand::W30AuditionPromoted,
            W30RegressionAction::TriggerPad => ActionCommand::W30TriggerPad,
            W30RegressionAction::SwapBank => ActionCommand::W30SwapBank,
            W30RegressionAction::ApplyDamageProfile => ActionCommand::W30ApplyDamageProfile,
            W30RegressionAction::LoopFreeze => ActionCommand::W30LoopFreeze,
            W30RegressionAction::BrowseSlicePool => ActionCommand::W30BrowseSlicePool,
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

    fn scene_regression_graph(section_labels: &[String]) -> SourceGraph {
        let mut graph = sample_graph();
        graph.sections.clear();

        for (index, label) in section_labels.iter().enumerate() {
            let bar_start = (index as u32 * 8) + 1;
            graph.sections.push(Section {
                section_id: SectionId::from(format!("section-{index}")),
                label_hint: scene_label_hint(label),
                start_seconds: index as f32 * 16.0,
                end_seconds: (index + 1) as f32 * 16.0,
                bar_start,
                bar_end: bar_start + 7,
                energy_class: scene_energy_for_label(label),
                confidence: 0.9,
                tags: vec![label.clone()],
            });
        }

        graph
    }

    fn seed_scene_fixture_state(state: &mut JamAppState, fixture: &SceneRegressionFixture) {
        if let Some(current_scene) = fixture.initial_current_scene.as_deref() {
            state.session.runtime_state.transport.current_scene =
                Some(SceneId::from(current_scene));
        }
        if let Some(active_scene) = fixture.initial_active_scene.as_deref() {
            state.session.runtime_state.scene_state.active_scene =
                Some(SceneId::from(active_scene));
        }
        if let Some(restore_scene) = fixture.initial_restore_scene.as_deref() {
            state.session.runtime_state.scene_state.restore_scene =
                Some(SceneId::from(restore_scene));
        }
        if let Some(reinforcement_mode) = fixture.tr909_reinforcement_mode {
            state
                .session
                .runtime_state
                .lane_state
                .tr909
                .takeover_enabled = false;
            state
                .session
                .runtime_state
                .lane_state
                .tr909
                .takeover_profile = None;
            state
                .session
                .runtime_state
                .lane_state
                .tr909
                .reinforcement_mode = Some(reinforcement_mode);
        }
        if let Some(pattern_ref) = fixture.tr909_pattern_ref.as_deref() {
            state.session.runtime_state.lane_state.tr909.pattern_ref = Some(pattern_ref.into());
        }
        state.refresh_view();
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
        session.runtime_state.lane_state.w30.preview_mode = Some(W30PreviewModeState::LiveRecall);
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
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
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

    fn write_pcm24_wave(path: impl AsRef<Path>, sample_rate: u32, channel_count: u16) {
        let path = path.as_ref();
        let samples = [-8_388_608_i32, 0, 8_388_607, 4_194_304];
        assert_eq!(samples.len() % usize::from(channel_count), 0);
        let bits_per_sample = 24_u16;
        let bytes_per_sample = u32::from(bits_per_sample / 8);
        let byte_rate = sample_rate * u32::from(channel_count) * bytes_per_sample;
        let block_align = channel_count * (bits_per_sample / 8);
        let data_len = samples.len() as u32 * bytes_per_sample;

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

        for sample in samples {
            bytes.extend_from_slice(&sample.to_le_bytes()[..3]);
        }

        fs::write(path, bytes).expect("write PCM24 wave fixture");
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
    fn derives_scene_candidates_from_source_sections_when_session_is_empty() {
        let mut graph = sample_graph();
        graph.sections.push(Section {
            section_id: SectionId::from("section-b"),
            label_hint: SectionLabelHint::Break,
            start_seconds: 16.0,
            end_seconds: 24.0,
            bar_start: 9,
            bar_end: 12,
            energy_class: EnergyClass::Medium,
            confidence: 0.84,
            tags: vec!["contrast".into()],
        });

        let mut session = sample_session(&graph);
        session.runtime_state.transport.current_scene = None;
        session.runtime_state.scene_state.active_scene = None;
        session.runtime_state.scene_state.scenes.clear();

        let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(
            state
                .session
                .runtime_state
                .scene_state
                .scenes
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["scene-01-drop".to_string(), "scene-02-break".to_string()]
        );
        assert_eq!(
            state.session.runtime_state.scene_state.active_scene,
            Some(SceneId::from("scene-01-drop"))
        );
        assert_eq!(
            state.session.runtime_state.transport.current_scene,
            Some(SceneId::from("scene-01-drop"))
        );
        assert_eq!(state.jam_view.scene.scene_count, 2);
        assert_eq!(
            state.jam_view.scene.active_scene.as_deref(),
            Some("scene-01-drop")
        );
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
    fn loads_pcm24_source_audio_cache_from_app_files() {
        let dir = tempdir().expect("create temp dir");
        let session_path = dir.path().join("sessions").join("session.json");
        let graph_path = dir.path().join("graphs").join("source-graph.json");
        let source_path = dir.path().join("source24.wav");

        write_pcm24_wave(&source_path, 48_000, 2);

        let mut graph = sample_graph();
        graph.source.path = source_path.to_string_lossy().into_owned();
        graph.source.sample_rate = 48_000;
        graph.source.channel_count = 2;
        graph.source.duration_seconds = 2.0 / 48_000.0;
        let session = sample_session(&graph);
        save_session_json(&session_path, &session).expect("save session fixture");
        save_source_graph_json(&graph_path, &graph).expect("save graph fixture");

        let state =
            JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("load app state");
        let cache = state
            .source_audio_cache
            .as_ref()
            .expect("source audio cache");

        assert_eq!(cache.sample_rate, 48_000);
        assert_eq!(cache.channel_count, 2);
        assert_eq!(cache.frame_count(), 2);
        assert_eq!(cache.interleaved_samples()[0], -1.0);
        assert_eq!(cache.interleaved_samples()[1], 0.0);
        assert!((cache.interleaved_samples()[2] - 1.0).abs() < 0.000001);
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
            Some(Tr909TakeoverProfileState::ControlledPhraseTakeover);
        session.runtime_state.lane_state.tr909.pattern_ref = Some("scene-1-main".into());
        session.runtime_state.macro_state.tr909_slam = 0.91;
        session.runtime_state.mixer_state.drum_level = 0.0;
        let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(state.runtime_view.tr909_render_mode, "takeover");
        assert_eq!(state.runtime_view.tr909_render_routing, "drum_bus_takeover");
        assert_eq!(state.runtime_view.tr909_render_profile, "controlled_phrase");
        assert_eq!(state.runtime_view.tr909_render_support_context, "unset");
        assert_eq!(state.runtime_view.tr909_render_support_accent, "off");
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
    fn runtime_view_surfaces_mc202_render_diagnostics() {
        let graph = sample_graph();
        let mut session = sample_session(&graph);
        session.runtime_state.lane_state.mc202.role = Some("answer".into());
        session.runtime_state.lane_state.mc202.phrase_ref = Some("answer-scene-1".into());
        session.runtime_state.macro_state.mc202_touch = 0.82;
        session.runtime_state.mixer_state.music_level = 0.0;
        let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Answer);
        assert_eq!(
            state.runtime.mc202_render.routing,
            Mc202RenderRouting::MusicBusBass
        );
        assert_eq!(
            state.runtime.mc202_render.phrase_shape,
            Mc202PhraseShape::AnswerHook
        );
        assert_eq!(state.runtime_view.mc202_render_mode, "answer");
        assert_eq!(state.runtime_view.mc202_render_routing, "music_bus_bass");
        assert_eq!(state.runtime_view.mc202_render_phrase_shape, "answer_hook");
        assert_eq!(
            state.runtime_view.mc202_render_mix_summary,
            "music bus 0.00 | touch 0.82"
        );
        assert!(
            state
                .runtime_view
                .mc202_render_transport_summary
                .contains("running @ 32.0 beats")
        );
        assert!(state.runtime_view.runtime_warnings.iter().any(
            |warning| warning == "MC-202 render is routed to the music bus at zero music level"
        ));
    }

    #[test]
    fn runtime_view_surfaces_w30_resample_tap_diagnostics() {
        let graph = sample_graph();
        let mut session = sample_session(&graph);
        session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-b"),
            pad_id: PadId::from("pad-03"),
        });
        session.captures[0].is_pinned = true;
        session.captures[0].lineage_capture_refs =
            vec![CaptureId::from("cap-seed"), CaptureId::from("cap-bar-02")];
        session.captures[0].resample_generation_depth = 2;
        let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(
            state.runtime.w30_resample_tap.mode,
            W30ResampleTapMode::CaptureLineageReady
        );
        assert_eq!(
            state.runtime.w30_resample_tap.routing,
            W30ResampleTapRouting::InternalCaptureTap
        );
        assert_eq!(
            state.runtime.w30_resample_tap.source_profile,
            Some(W30ResampleTapSourceProfile::PinnedCapture)
        );
        assert_eq!(
            state.runtime.w30_resample_tap.source_capture_id.as_deref(),
            Some("cap-01")
        );
        assert_eq!(state.runtime.w30_resample_tap.lineage_capture_count, 2);
        assert_eq!(state.runtime.w30_resample_tap.generation_depth, 2);
        assert_eq!(
            state.runtime_view.w30_resample_tap_mode,
            "capture_lineage_ready"
        );
        assert_eq!(
            state.runtime_view.w30_resample_tap_routing,
            "internal_capture_tap"
        );
        assert_eq!(
            state.runtime_view.w30_resample_tap_profile,
            "pinned_capture"
        );
        assert_eq!(
            state.runtime_view.w30_resample_tap_source_summary,
            "cap-01 | gen 2 | lineage 2"
        );
        assert_eq!(
            state.runtime_view.w30_resample_tap_mix_summary,
            "music bus 0.64 | grit 0.40"
        );
    }

    #[test]
    fn adjusting_drum_bus_level_updates_session_and_runtime_view() {
        let graph = sample_graph();
        let mut session = sample_session(&graph);
        session.runtime_state.lane_state.tr909.takeover_enabled = true;
        session.runtime_state.lane_state.tr909.takeover_profile =
            Some(Tr909TakeoverProfileState::ControlledPhraseTakeover);
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
    fn adjusting_mc202_touch_updates_session_and_runtime_view() {
        let graph = sample_graph();
        let mut session = sample_session(&graph);
        session.runtime_state.lane_state.mc202.role = Some("follower".into());
        session.runtime_state.lane_state.mc202.phrase_ref = Some("follower-scene-1".into());
        session.runtime_state.macro_state.mc202_touch = 0.40;
        session.runtime_state.mixer_state.music_level = 0.64;
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        let raised = state.adjust_mc202_touch(0.24);
        assert!((raised - 0.64).abs() < f32::EPSILON);
        assert!((state.session.runtime_state.macro_state.mc202_touch - 0.64).abs() < f32::EPSILON);
        assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Follower);
        assert_eq!(
            state.runtime.mc202_render.phrase_shape,
            Mc202PhraseShape::FollowerDrive
        );
        assert_eq!(
            state.runtime.mc202_render.routing,
            Mc202RenderRouting::MusicBusBass
        );
        assert!((state.runtime.mc202_render.touch - 0.64).abs() < f32::EPSILON);
        assert_eq!(
            state.runtime_view.mc202_render_mix_summary,
            "music bus 0.64 | touch 0.64"
        );

        let lowered = state.adjust_mc202_touch(-1.5);
        assert_eq!(lowered, 0.0);
        assert_eq!(state.session.runtime_state.macro_state.mc202_touch, 0.0);
        assert_eq!(state.runtime.mc202_render.touch, 0.0);
        assert_eq!(
            state.runtime_view.mc202_render_mix_summary,
            "music bus 0.64 | touch 0.00"
        );
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
    fn setting_transport_playing_records_audio_transport_anchor() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.set_transport_playing(true);

        assert!(state.runtime.transport.is_playing);
        assert_eq!(
            state.runtime.transport_driver.last_audio_position_beats,
            Some(state.runtime.transport.beat_index)
        );

        state.set_transport_playing(false);

        assert!(!state.runtime.transport.is_playing);
        assert_eq!(
            state.runtime.transport_driver.last_audio_position_beats,
            None
        );
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
    fn queue_scene_select_enqueues_scene_launch_for_next_bar() {
        let mut graph = sample_graph();
        graph.sections.push(Section {
            section_id: SectionId::from("section-b"),
            label_hint: SectionLabelHint::Break,
            start_seconds: 16.0,
            end_seconds: 24.0,
            bar_start: 9,
            bar_end: 12,
            energy_class: EnergyClass::Medium,
            confidence: 0.84,
            tags: vec!["contrast".into()],
        });

        let mut session = sample_session(&graph);
        session.runtime_state.transport.current_scene = None;
        session.runtime_state.scene_state.active_scene = None;
        session.runtime_state.scene_state.scenes.clear();

        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
        assert_eq!(
            state.jam_view.scene.next_scene.as_deref(),
            Some("scene-02-break")
        );
        assert_eq!(
            state.jam_view.scene.next_scene_energy.as_deref(),
            Some("medium")
        );
        assert_eq!(state.queue_scene_select(300), QueueControlResult::Enqueued);
        assert_eq!(
            state.queue_scene_select(301),
            QueueControlResult::AlreadyPending
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::SceneLaunch);
        assert_eq!(pending[0].quantization, Quantization::NextBar);
        assert_eq!(
            pending[0].target.scene_id,
            Some(SceneId::from("scene-02-break"))
        );
        assert_eq!(
            pending[0].params,
            ActionParams::Scene {
                scene_id: Some(SceneId::from("scene-02-break"))
            }
        );
    }

    #[test]
    fn queue_scene_select_rejects_single_current_scene_without_pending_action() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(
            state.jam_view.scene.scene_jump_availability,
            SceneJumpAvailabilityView::WaitingForMoreScenes
        );
        assert_eq!(
            state.queue_scene_select(300),
            QueueControlResult::AlreadyInState
        );
        assert!(state.queue.pending_actions().is_empty());
    }

    #[test]
    fn queue_scene_select_prefers_energy_contrast_candidate() {
        let graph = scene_regression_graph(&[
            "drop".to_string(),
            "chorus".to_string(),
            "intro".to_string(),
        ]);
        let mut session = sample_session(&graph);
        session.runtime_state.scene_state.scenes = vec![
            SceneId::from("scene-01-drop"),
            SceneId::from("scene-02-chorus"),
            SceneId::from("scene-03-intro"),
        ];
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-drop"));
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-drop"));

        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(
            state.jam_view.scene.next_scene.as_deref(),
            Some("scene-03-intro")
        );
        assert_eq!(state.queue_scene_select(300), QueueControlResult::Enqueued);
        assert_eq!(
            state.queue.pending_actions()[0].target.scene_id,
            Some(SceneId::from("scene-03-intro"))
        );
        assert_eq!(
            state.queue.pending_actions()[0].explanation.as_deref(),
            Some("launch contrast scene scene-03-intro on next bar")
        );

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Bar,
                beat_index: 32,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-01-drop")),
            },
            350,
        );

        assert_eq!(committed.len(), 1);
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some("launched contrast scene scene-03-intro at bar 9 / phrase 2")
        );
    }

    #[test]
    fn committed_scene_select_updates_transport_and_scene_state() {
        let mut graph = sample_graph();
        graph.sections.push(Section {
            section_id: SectionId::from("section-b"),
            label_hint: SectionLabelHint::Break,
            start_seconds: 16.0,
            end_seconds: 24.0,
            bar_start: 9,
            bar_end: 12,
            energy_class: EnergyClass::Medium,
            confidence: 0.84,
            tags: vec!["contrast".into()],
        });

        let mut session = sample_session(&graph);
        session.runtime_state.transport.current_scene = None;
        session.runtime_state.scene_state.active_scene = None;
        session.runtime_state.scene_state.scenes.clear();

        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
        assert_eq!(state.queue_scene_select(300), QueueControlResult::Enqueued);

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Bar,
                beat_index: 32,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-01-drop")),
            },
            350,
        );

        assert_eq!(committed.len(), 1);
        assert_eq!(
            state.session.runtime_state.scene_state.active_scene,
            Some(SceneId::from("scene-02-break"))
        );
        assert_eq!(
            state.session.runtime_state.transport.current_scene,
            Some(SceneId::from("scene-02-break"))
        );
        assert_eq!(
            state.session.runtime_state.scene_state.restore_scene,
            Some(SceneId::from("scene-01-drop"))
        );
        assert_eq!(
            state.runtime.transport.current_scene,
            Some(SceneId::from("scene-02-break"))
        );
        assert_eq!(
            state.jam_view.scene.active_scene.as_deref(),
            Some("scene-02-break")
        );
        assert_eq!(
            state.jam_view.scene.restore_scene.as_deref(),
            Some("scene-01-drop")
        );
        assert_eq!(
            state.jam_view.scene.active_scene_energy.as_deref(),
            Some("medium")
        );
        assert_eq!(
            state.jam_view.scene.restore_scene_energy.as_deref(),
            Some("high")
        );
        assert_eq!(
            state.runtime.tr909_render.current_scene_id.as_deref(),
            Some("scene-02-break")
        );
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some("launched scene scene-02-break at bar 9 / phrase 2")
        );
    }

    #[test]
    fn committed_scene_select_projects_target_scene_into_tr909_source_support() {
        let mut graph = sample_graph();
        graph.sections.push(Section {
            section_id: SectionId::from("section-b"),
            label_hint: SectionLabelHint::Break,
            start_seconds: 16.0,
            end_seconds: 24.0,
            bar_start: 9,
            bar_end: 12,
            energy_class: EnergyClass::Medium,
            confidence: 0.84,
            tags: vec!["break".into()],
        });

        let mut session = sample_session(&graph);
        session.runtime_state.transport.position_beats = 32.0;
        session.runtime_state.transport.current_scene = None;
        session.runtime_state.scene_state.active_scene = None;
        session.runtime_state.scene_state.scenes.clear();
        session.runtime_state.lane_state.tr909.reinforcement_mode =
            Some(Tr909ReinforcementModeState::SourceSupport);
        session.runtime_state.lane_state.tr909.pattern_ref = Some("support-scene".into());

        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(
            state.runtime.tr909_render.source_support_profile,
            Some(Tr909SourceSupportProfile::DropDrive)
        );
        assert_eq!(state.queue_scene_select(300), QueueControlResult::Enqueued);

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Bar,
                beat_index: 32,
                bar_index: 8,
                phrase_index: 1,
                scene_id: Some(SceneId::from("scene-01-drop")),
            },
            350,
        );

        assert_eq!(committed.len(), 1);
        assert_eq!(
            state.session.runtime_state.scene_state.active_scene,
            Some(SceneId::from("scene-02-break"))
        );
        assert_eq!(
            state.runtime.tr909_render.current_scene_id.as_deref(),
            Some("scene-02-break")
        );
        assert_eq!(
            state.runtime.tr909_render.source_support_profile,
            Some(Tr909SourceSupportProfile::BreakLift)
        );
        assert_eq!(
            state.runtime.tr909_render.source_support_context,
            Some(Tr909SourceSupportContext::SceneTarget)
        );
        assert_eq!(
            state.runtime_view.tr909_render_support_context,
            "scene_target"
        );
        assert_eq!(state.runtime_view.tr909_render_support_accent, "scene");
        assert_eq!(
            state.runtime.tr909_render.pattern_adoption,
            Some(Tr909PatternAdoption::SupportPulse)
        );
    }

    #[test]
    fn queue_scene_restore_enqueues_scene_restore_for_next_bar() {
        let graph = sample_graph();
        let mut session = sample_session(&graph);
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-02-break"));
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-02-break"));
        session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));

        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
        assert_eq!(state.queue_scene_restore(300), QueueControlResult::Enqueued);
        assert_eq!(
            state.queue_scene_restore(301),
            QueueControlResult::AlreadyPending
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::SceneRestore);
        assert_eq!(pending[0].quantization, Quantization::NextBar);
        assert_eq!(
            pending[0].target.scene_id,
            Some(SceneId::from("scene-01-drop"))
        );
        assert_eq!(
            pending[0].params,
            ActionParams::Scene {
                scene_id: Some(SceneId::from("scene-01-drop"))
            }
        );
    }

    #[test]
    fn committed_scene_restore_updates_transport_scene_and_restore_pointer() {
        let graph = sample_graph();
        let mut session = sample_session(&graph);
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-02-break"));
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-02-break"));
        session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));

        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
        assert_eq!(state.queue_scene_restore(300), QueueControlResult::Enqueued);

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Bar,
                beat_index: 36,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-02-break")),
            },
            420,
        );

        assert_eq!(committed.len(), 1);
        assert_eq!(
            state.session.runtime_state.scene_state.active_scene,
            Some(SceneId::from("scene-01-drop"))
        );
        assert_eq!(
            state.session.runtime_state.transport.current_scene,
            Some(SceneId::from("scene-01-drop"))
        );
        assert_eq!(
            state.session.runtime_state.scene_state.restore_scene,
            Some(SceneId::from("scene-02-break"))
        );
        assert_eq!(
            state.runtime.transport.current_scene,
            Some(SceneId::from("scene-01-drop"))
        );
        assert_eq!(
            state.jam_view.scene.active_scene.as_deref(),
            Some("scene-01-drop")
        );
        assert_eq!(
            state.jam_view.scene.restore_scene.as_deref(),
            Some("scene-02-break")
        );
        assert_eq!(
            state.jam_view.scene.active_scene_energy.as_deref(),
            Some("high")
        );
        assert_eq!(state.jam_view.scene.restore_scene_energy.as_deref(), None);
        assert_eq!(
            state.runtime.tr909_render.current_scene_id.as_deref(),
            Some("scene-01-drop")
        );
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some("restored scene scene-01-drop at bar 9 / phrase 2")
        );
    }

    #[test]
    fn committed_scene_restore_projects_target_scene_into_tr909_source_support() {
        let graph = sample_graph();
        let mut session = sample_session(&graph);
        session.runtime_state.transport.position_beats = 32.0;
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-02-break"));
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-02-break"));
        session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));
        session.runtime_state.lane_state.tr909.reinforcement_mode =
            Some(Tr909ReinforcementModeState::SourceSupport);
        session.runtime_state.lane_state.tr909.pattern_ref = Some("restore-support".into());

        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
        assert_eq!(state.queue_scene_restore(300), QueueControlResult::Enqueued);

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Bar,
                beat_index: 36,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-02-break")),
            },
            420,
        );

        assert_eq!(committed.len(), 1);
        assert_eq!(
            state.session.runtime_state.scene_state.active_scene,
            Some(SceneId::from("scene-01-drop"))
        );
        assert_eq!(
            state.runtime.tr909_render.current_scene_id.as_deref(),
            Some("scene-01-drop")
        );
        assert_eq!(
            state.runtime.tr909_render.source_support_profile,
            Some(Tr909SourceSupportProfile::DropDrive)
        );
        assert_eq!(
            state.runtime.tr909_render.source_support_context,
            Some(Tr909SourceSupportContext::SceneTarget)
        );
        assert_eq!(
            state.runtime_view.tr909_render_support_context,
            "scene_target"
        );
        assert_eq!(state.runtime_view.tr909_render_support_accent, "scene");
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
            state.jam_view.lanes.tr909_takeover_pending_profile,
            Some(Tr909TakeoverProfileState::ControlledPhraseTakeover)
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
            state.jam_view.lanes.tr909_takeover_pending_profile,
            Some(Tr909TakeoverProfileState::SceneLockTakeover)
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
            .takeover_profile = Some(Tr909TakeoverProfileState::SceneLockTakeover);
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
            Some(Tr909TakeoverProfileState::ControlledPhraseTakeover);
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
    fn audio_timing_snapshot_commits_crossed_bar_boundary() {
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
        state.set_transport_playing(true);
        state.queue_tr909_fill(300);

        let committed = state.apply_audio_timing_snapshot(
            AudioRuntimeTimingSnapshot {
                is_transport_running: true,
                tempo_bpm: 124.0,
                position_beats: 32.0,
            },
            3_100,
        );

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
    fn audio_timing_snapshot_advances_transport_from_callback_position() {
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
        state.set_transport_playing(true);

        let committed = state.apply_audio_timing_snapshot(
            AudioRuntimeTimingSnapshot {
                is_transport_running: true,
                tempo_bpm: 124.0,
                position_beats: 33.0,
            },
            2_500,
        );

        assert!(committed.is_empty());
        assert!(state.runtime.transport.position_beats > 32.9);
        assert!(state.runtime.transport.position_beats < 33.1);
        assert_eq!(
            state.runtime.transport_driver.last_audio_position_beats,
            Some(33)
        );
    }

    #[test]
    fn stale_stopped_audio_timing_snapshot_does_not_cancel_transport_start() {
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
        state.set_transport_playing(true);

        let committed = state.apply_audio_timing_snapshot(
            AudioRuntimeTimingSnapshot {
                is_transport_running: false,
                tempo_bpm: 124.0,
                position_beats: 32.0,
            },
            2_500,
        );

        assert!(committed.is_empty());
        assert!(state.runtime.transport.is_playing);
        assert_eq!(state.runtime.transport.position_beats, 32.0);
        assert!(state.jam_view.transport.is_playing);
        assert_eq!(state.jam_view.transport.position_beats, 32.0);
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
        assert_eq!(state.jam_view.capture.last_capture_target_kind, None);
        assert_eq!(state.jam_view.capture.last_capture_origin_count, 2);
        assert_eq!(state.jam_view.capture.unassigned_capture_count, 2);
        assert_eq!(state.jam_view.capture.promoted_capture_count, 0);
        let source_window = state.session.captures[1]
            .source_window
            .as_ref()
            .expect("capture source window");
        assert_eq!(source_window.source_id, SourceId::from("src-1"));
        assert!((source_window.start_seconds - 15.238).abs() < 0.01);
        assert!((source_window.end_seconds - 22.857).abs() < 0.01);
        assert_eq!(source_window.start_frame, 731_428);
        assert_eq!(source_window.end_frame, 1_097_142);

        let tempdir = tempdir().expect("create capture window tempdir");
        let session_path = tempdir.path().join("capture-window.json");
        save_session_json(&session_path, &state.session).expect("save capture-window session");
        let reloaded = load_session_json(&session_path).expect("reload capture-window session");
        assert_eq!(
            reloaded.captures[1].source_window,
            Some(source_window.clone())
        );
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
            state.jam_view.capture.last_capture_target_kind,
            Some(CaptureTargetKindView::W30Pad)
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
    fn queue_w30_live_recall_targets_committed_lane_focus_before_latest_pinned_capture() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures.push(CaptureRef {
            capture_id: CaptureId::from("cap-02"),
            capture_type: CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
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
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-03.wav".into(),
            assigned_target: Some(CaptureTarget::W30Pad {
                bank_id: "bank-c".into(),
                pad_id: "pad-07".into(),
            }),
            is_pinned: true,
            notes: Some("keeper".into()),
        });
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-04"));
        state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-03"));
        state.refresh_view();

        assert_eq!(
            state.queue_w30_live_recall(600),
            Some(QueueControlResult::Enqueued)
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::W30LiveRecall);
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
            Some("recall cap-02 on W-30 pad bank-b/pad-04")
        );
        assert_eq!(
            state.jam_view.lanes.w30_pending_recall_target.as_deref(),
            Some("bank-b/pad-04")
        );
        assert_eq!(state.jam_view.lanes.w30_pending_audition, None);
        assert_eq!(state.jam_view.lanes.w30_pending_audition_target, None);
    }

    #[test]
    fn queue_w30_promoted_audition_targets_committed_lane_focus() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures.push(CaptureRef {
            capture_id: CaptureId::from("cap-02"),
            capture_type: CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-04".into(),
            }),
            is_pinned: false,
            notes: Some("secondary".into()),
        });
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-04"));
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
        let pending_audition = state
            .jam_view
            .lanes
            .w30_pending_audition
            .as_ref()
            .expect("pending promoted audition projects into Jam view");
        assert_eq!(pending_audition.kind, W30PendingAuditionKind::Promoted);
        assert_eq!(pending_audition.target, "bank-b/pad-04");
        assert_eq!(pending_audition.quantization, "next_bar");
        assert_eq!(state.jam_view.lanes.w30_pending_recall_target, None);
    }

    #[test]
    fn queue_w30_audition_targets_raw_capture_before_promotion() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(
            state.queue_w30_audition(620),
            Some(QueueControlResult::Enqueued)
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::W30AuditionRawCapture);
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
            Some("audition raw capture cap-01 on W-30 preview bank-a/pad-01")
        );
        assert_eq!(
            state.jam_view.lanes.w30_pending_audition_target.as_deref(),
            Some("bank-a/pad-01")
        );
        let pending_audition = state
            .jam_view
            .lanes
            .w30_pending_audition
            .as_ref()
            .expect("pending raw audition projects into Jam view");
        assert_eq!(pending_audition.kind, W30PendingAuditionKind::RawCapture);
        assert_eq!(pending_audition.target, "bank-a/pad-01");
        assert_eq!(pending_audition.quantization, "next_bar");
    }

    #[test]
    fn committed_w30_raw_capture_audition_updates_preview_state() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(
            state.queue_w30_audition(620),
            Some(QueueControlResult::Enqueued)
        );
        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Bar,
                beat_index: 40,
                bar_index: 10,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            700,
        );

        assert_eq!(committed.len(), 1);
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .map(|action| action.command),
            Some(ActionCommand::W30AuditionRawCapture)
        );
        assert_eq!(
            state.session.runtime_state.lane_state.w30.preview_mode,
            Some(W30PreviewModeState::RawCaptureAudition)
        );
        assert_eq!(
            state.runtime.w30_preview.mode,
            W30PreviewRenderMode::RawCaptureAudition
        );
        assert_eq!(
            state.runtime.w30_preview.source_profile,
            Some(W30PreviewSourceProfile::RawCaptureAudition)
        );
        assert_eq!(
            state.runtime.w30_preview.capture_id.as_deref(),
            Some("cap-01")
        );
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some("auditioned raw cap-01 on W-30 preview bank-a/pad-01")
        );
    }

    #[test]
    fn raw_capture_audition_projects_source_window_preview_samples() {
        let tempdir = tempdir().expect("create source audio tempdir");
        let source_path = tempdir.path().join("source.wav");
        write_pcm16_wave(&source_path, 48_000, 2, 1.0);

        let mut graph = sample_graph();
        graph.source.path = source_path.to_string_lossy().into_owned();
        graph.source.duration_seconds = 1.0;
        let mut session = sample_session(&graph);
        session.runtime_state.lane_state.w30.preview_mode =
            Some(W30PreviewModeState::RawCaptureAudition);
        session.captures[0].source_window = Some(CaptureSourceWindow {
            source_id: graph.source.source_id.clone(),
            start_seconds: 0.0,
            end_seconds: 1.0,
            start_frame: 0,
            end_frame: 48_000,
        });
        let source_audio_cache =
            SourceAudioCache::load_pcm_wav(&source_path).expect("load source audio cache");
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
        state.source_audio_cache = Some(source_audio_cache);

        state.refresh_view();

        let preview = state
            .runtime
            .w30_preview
            .source_window_preview
            .as_ref()
            .expect("source-window preview");
        assert_eq!(preview.source_start_frame, 0);
        assert_eq!(preview.source_end_frame, 48_000);
        assert_eq!(preview.sample_count, W30_PREVIEW_SAMPLE_WINDOW_LEN);
        assert!(preview.samples.iter().any(|sample| sample.abs() > 0.001));
    }

    #[test]
    fn captured_source_window_promotes_to_pad_and_auditions_source_preview() {
        let tempdir = tempdir().expect("create source audio tempdir");
        let source_path = tempdir.path().join("source.wav");
        write_pcm16_wave(&source_path, 48_000, 2, 8.0);

        let mut graph = sample_graph();
        graph.source.path = source_path.to_string_lossy().into_owned();
        graph.source.duration_seconds = 8.0;
        let mut session = sample_session(&graph);
        session.captures.clear();
        session.runtime_state.lane_state.w30.last_capture = None;
        let source_audio_cache =
            SourceAudioCache::load_pcm_wav(&source_path).expect("load source audio cache");
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
        state.source_audio_cache = Some(source_audio_cache);
        state.refresh_view();

        state.queue_capture_bar(300);
        let committed_capture = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Phrase,
                beat_index: 0,
                bar_index: 1,
                phrase_index: 0,
                scene_id: Some(SceneId::from("scene-1")),
            },
            400,
        );

        assert_eq!(committed_capture.len(), 1);
        assert_eq!(state.session.captures.len(), 1);
        assert_eq!(
            state.session.captures[0]
                .source_window
                .as_ref()
                .map(|source_window| source_window.start_frame),
            Some(0)
        );
        assert!(state.session.captures[0].source_window.is_some());

        assert!(state.queue_promote_last_capture(410));
        let committed_promotion = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Bar,
                beat_index: 4,
                bar_index: 2,
                phrase_index: 0,
                scene_id: Some(SceneId::from("scene-1")),
            },
            500,
        );

        assert_eq!(committed_promotion.len(), 1);
        assert_eq!(
            state.session.captures[0].assigned_target,
            Some(CaptureTarget::W30Pad {
                bank_id: BankId::from("bank-a"),
                pad_id: PadId::from("pad-01"),
            })
        );

        assert_eq!(
            state.queue_w30_audition(520),
            Some(QueueControlResult::Enqueued)
        );
        let committed_audition = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Bar,
                beat_index: 8,
                bar_index: 3,
                phrase_index: 0,
                scene_id: Some(SceneId::from("scene-1")),
            },
            600,
        );

        assert_eq!(committed_audition.len(), 1);
        assert_eq!(
            state.runtime.w30_preview.mode,
            W30PreviewRenderMode::PromotedAudition
        );
        assert_eq!(
            state.runtime.w30_preview.source_profile,
            Some(W30PreviewSourceProfile::PromotedAudition)
        );
        assert_eq!(
            state.runtime.w30_preview.capture_id.as_deref(),
            Some("cap-01")
        );
        let preview = state
            .runtime
            .w30_preview
            .source_window_preview
            .as_ref()
            .expect("source-backed promoted audition preview");
        assert_eq!(preview.source_start_frame, 0);
        assert!(preview.source_end_frame > preview.source_start_frame);
        assert_eq!(preview.sample_count, W30_PREVIEW_SAMPLE_WINDOW_LEN);
        assert!(preview.samples.iter().any(|sample| sample.abs() > 0.001));
    }

    #[test]
    fn promoted_and_recall_w30_previews_project_source_window_preview_samples() {
        for preview_mode in [
            W30PreviewModeState::PromotedAudition,
            W30PreviewModeState::LiveRecall,
        ] {
            let tempdir = tempdir().expect("create source audio tempdir");
            let source_path = tempdir.path().join("source.wav");
            write_pcm16_wave(&source_path, 48_000, 2, 1.0);

            let mut graph = sample_graph();
            graph.source.path = source_path.to_string_lossy().into_owned();
            graph.source.duration_seconds = 1.0;
            let mut session = sample_session(&graph);
            session.runtime_state.lane_state.w30.preview_mode = Some(preview_mode);
            session.captures[0].source_window = Some(CaptureSourceWindow {
                source_id: graph.source.source_id.clone(),
                start_seconds: 0.0,
                end_seconds: 1.0,
                start_frame: 0,
                end_frame: 48_000,
            });
            let source_audio_cache =
                SourceAudioCache::load_pcm_wav(&source_path).expect("load source audio cache");
            let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
            state.source_audio_cache = Some(source_audio_cache);

            state.refresh_view();

            let preview = state
                .runtime
                .w30_preview
                .source_window_preview
                .as_ref()
                .expect("source-window preview");
            assert_eq!(preview.source_start_frame, 0);
            assert_eq!(preview.source_end_frame, 48_000);
            assert_eq!(preview.sample_count, W30_PREVIEW_SAMPLE_WINDOW_LEN);
            assert!(preview.samples.iter().any(|sample| sample.abs() > 0.001));
        }
    }

    #[test]
    fn queue_w30_trigger_pad_targets_focused_lane_capture_on_next_beat() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        });
        state.session.captures.push(CaptureRef {
            capture_id: CaptureId::from("cap-02"),
            capture_type: CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-04".into(),
            }),
            is_pinned: false,
            notes: Some("secondary".into()),
        });
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-04"));
        state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
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
            Some("bank-b".into())
        );
        assert_eq!(
            pending[0].target.pad_id.as_ref().map(ToString::to_string),
            Some("pad-04".into())
        );
        assert_eq!(
            pending[0].explanation.as_deref(),
            Some("trigger W-30 pad bank-b/pad-04 from cap-02 on next beat")
        );
        assert_eq!(
            state.jam_view.lanes.w30_pending_trigger_target.as_deref(),
            Some("bank-b/pad-04")
        );
        assert_eq!(state.jam_view.lanes.w30_pending_recall_target, None);
        assert_eq!(state.jam_view.lanes.w30_pending_audition_target, None);
    }

    #[test]
    fn queue_w30_step_focus_targets_next_promoted_pad_on_next_beat() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        });
        state.session.captures.push(CaptureRef {
            capture_id: CaptureId::from("cap-02"),
            capture_type: CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(CaptureTarget::W30Pad {
                bank_id: BankId::from("bank-b"),
                pad_id: PadId::from("pad-04"),
            }),
            is_pinned: false,
            notes: Some("secondary".into()),
        });
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
        state.refresh_view();

        assert_eq!(
            state.queue_w30_step_focus(622),
            Some(QueueControlResult::Enqueued)
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::W30StepFocus);
        assert_eq!(pending[0].quantization, Quantization::NextBeat);
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
            Some("step W-30 focus to bank-b/pad-04 on next beat")
        );
        assert_eq!(
            state
                .jam_view
                .lanes
                .w30_pending_focus_step_target
                .as_deref(),
            Some("bank-b/pad-04")
        );
        assert_eq!(state.jam_view.lanes.w30_pending_recall_target, None);
        assert_eq!(state.jam_view.lanes.w30_pending_audition_target, None);
    }

    #[test]
    fn queue_w30_internal_resample_targets_focused_lane_capture_on_next_phrase() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        });
        state.session.captures.push(CaptureRef {
            capture_id: CaptureId::from("cap-02"),
            capture_type: CaptureType::Resample,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: vec![CaptureId::from("cap-01")],
            resample_generation_depth: 1,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-03".into(),
            }),
            is_pinned: false,
            notes: Some("resampled".into()),
        });
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
        state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
        state.refresh_view();

        assert_eq!(
            state.queue_w30_internal_resample(627),
            Some(QueueControlResult::Enqueued)
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::PromoteResample);
        assert_eq!(pending[0].quantization, Quantization::NextPhrase);
        assert_eq!(pending[0].target.scope, Some(TargetScope::LaneW30));
        assert!(matches!(
            &pending[0].params,
            ActionParams::Promotion {
                capture_id: Some(capture_id),
                ..
            } if capture_id == &CaptureId::from("cap-02")
        ));
        assert_eq!(
            pending[0].explanation.as_deref(),
            Some("resample cap-02 through W-30 on next phrase")
        );
    }

    #[test]
    fn queue_w30_swap_bank_targets_next_bank_on_next_bar() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        });
        state.session.captures.push(CaptureRef {
            capture_id: CaptureId::from("cap-02"),
            capture_type: CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(CaptureTarget::W30Pad {
                bank_id: BankId::from("bank-b"),
                pad_id: PadId::from("pad-01"),
            }),
            is_pinned: false,
            notes: Some("bank b".into()),
        });
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
        state.refresh_view();

        assert_eq!(
            state.queue_w30_swap_bank(628),
            Some(QueueControlResult::Enqueued)
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::W30SwapBank);
        assert_eq!(pending[0].quantization, Quantization::NextBar);
        assert_eq!(
            pending[0].target.bank_id.as_ref().map(ToString::to_string),
            Some("bank-b".into())
        );
        assert_eq!(
            pending[0].target.pad_id.as_ref().map(ToString::to_string),
            Some("pad-01".into())
        );
        assert!(matches!(
            &pending[0].params,
            ActionParams::Mutation {
                target_id: Some(target_id),
                ..
            } if target_id == "cap-02"
        ));
        assert_eq!(
            pending[0].explanation.as_deref(),
            Some("swap W-30 bank to bank-b/pad-01 with cap-02 on next bar")
        );
        assert_eq!(
            state.jam_view.lanes.w30_pending_bank_swap_target.as_deref(),
            Some("bank-b/pad-01")
        );
    }

    #[test]
    fn queue_w30_browse_slice_pool_targets_next_capture_on_current_pad() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        });
        state.session.captures.push(CaptureRef {
            capture_id: CaptureId::from("cap-02"),
            capture_type: CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: vec![CaptureId::from("cap-01")],
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(CaptureTarget::W30Pad {
                bank_id: BankId::from("bank-a"),
                pad_id: PadId::from("pad-01"),
            }),
            is_pinned: false,
            notes: Some("alt slice".into()),
        });
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
        state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
        state.refresh_view();

        assert_eq!(
            state.queue_w30_browse_slice_pool(629),
            Some(QueueControlResult::Enqueued)
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::W30BrowseSlicePool);
        assert_eq!(pending[0].quantization, Quantization::NextBeat);
        assert_eq!(
            pending[0].target.bank_id.as_ref().map(ToString::to_string),
            Some("bank-a".into())
        );
        assert_eq!(
            pending[0].target.pad_id.as_ref().map(ToString::to_string),
            Some("pad-01".into())
        );
        assert!(matches!(
            &pending[0].params,
            ActionParams::Mutation {
                target_id: Some(target_id),
                ..
            } if target_id == "cap-02"
        ));
        assert_eq!(
            pending[0].explanation.as_deref(),
            Some("browse W-30 slice pool to cap-02 on bank-a/pad-01 on next beat")
        );
        assert_eq!(
            state
                .jam_view
                .lanes
                .w30_pending_slice_pool_target
                .as_deref(),
            Some("bank-a/pad-01")
        );
    }

    #[test]
    fn queue_w30_apply_damage_profile_targets_focused_lane_capture_on_next_bar() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        });
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
        state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
        state.refresh_view();

        assert_eq!(
            state.queue_w30_apply_damage_profile(644),
            Some(QueueControlResult::Enqueued)
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command, ActionCommand::W30ApplyDamageProfile);
        assert_eq!(pending[0].quantization, Quantization::NextBar);
        assert_eq!(
            pending[0].target.bank_id.as_ref().map(ToString::to_string),
            Some("bank-a".into())
        );
        assert_eq!(
            pending[0].target.pad_id.as_ref().map(ToString::to_string),
            Some("pad-01".into())
        );
        assert!(matches!(
            &pending[0].params,
            ActionParams::Mutation {
                intensity,
                target_id: Some(target_id),
            } if (*intensity - JamAppState::W30_DAMAGE_PROFILE_GRIT).abs() < f32::EPSILON
                && target_id == "cap-01"
        ));
        assert_eq!(
            pending[0].explanation.as_deref(),
            Some("apply shred damage profile to cap-01 on W-30 pad bank-a/pad-01")
        );
        assert_eq!(
            state
                .jam_view
                .lanes
                .w30_pending_damage_profile_target
                .as_deref(),
            Some("bank-a/pad-01")
        );
    }

    #[test]
    fn queue_w30_live_recall_falls_back_to_latest_pinned_capture_without_explicit_focus() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures.push(CaptureRef {
            capture_id: CaptureId::from("cap-02"),
            capture_type: CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-04".into(),
            }),
            is_pinned: true,
            notes: Some("keeper".into()),
        });
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-z"));
        state.session.runtime_state.lane_state.w30.focused_pad = None;
        state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
        state.refresh_view();

        assert_eq!(
            state.queue_w30_live_recall(605),
            Some(QueueControlResult::Enqueued)
        );

        let pending = state.queue.pending_actions();
        assert_eq!(pending.len(), 1);
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
            Some("recall cap-02 on W-30 pad bank-b/pad-04")
        );
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
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
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
            state.queue_w30_step_focus(631),
            Some(QueueControlResult::AlreadyPending)
        );
        assert_eq!(
            state.queue_w30_swap_bank(631),
            Some(QueueControlResult::AlreadyPending)
        );
        assert_eq!(
            state.queue_w30_apply_damage_profile(631),
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
        other_state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-c"));
        other_state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-05"));
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
            other_state.queue_w30_step_focus(633),
            Some(QueueControlResult::AlreadyPending)
        );
        assert_eq!(
            other_state.queue_w30_swap_bank(633),
            Some(QueueControlResult::AlreadyPending)
        );
        assert_eq!(
            other_state.queue_w30_apply_damage_profile(633),
            Some(QueueControlResult::AlreadyPending)
        );
        assert_eq!(
            other_state.queue_w30_trigger_pad(634),
            Some(QueueControlResult::AlreadyPending)
        );
    }

    #[test]
    fn queueing_w30_internal_resample_blocks_duplicate_pending_actions() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-b"),
            pad_id: PadId::from("pad-03"),
        });
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
        state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
        state.refresh_view();

        assert_eq!(
            state.queue_w30_internal_resample(635),
            Some(QueueControlResult::Enqueued)
        );
        assert_eq!(
            state.queue_w30_internal_resample(636),
            Some(QueueControlResult::AlreadyPending)
        );
    }

    #[test]
    fn queueing_w30_internal_resample_blocks_pending_loop_freeze() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-b"),
            pad_id: PadId::from("pad-03"),
        });
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
        state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
        state.refresh_view();

        assert_eq!(
            state.queue_w30_loop_freeze(635),
            Some(QueueControlResult::Enqueued)
        );
        assert_eq!(
            state.queue_w30_internal_resample(636),
            Some(QueueControlResult::AlreadyPending)
        );
    }

    #[test]
    fn queueing_w30_loop_freeze_blocks_pending_internal_resample() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-b"),
            pad_id: PadId::from("pad-03"),
        });
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
        state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
        state.refresh_view();

        assert_eq!(
            state.queue_w30_internal_resample(635),
            Some(QueueControlResult::Enqueued)
        );
        assert_eq!(
            state.queue_w30_loop_freeze(636),
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
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
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
    fn committed_w30_bank_swap_updates_lane_focus_and_log_result() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        });
        state.session.captures.push(CaptureRef {
            capture_id: CaptureId::from("cap-02"),
            capture_type: CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(CaptureTarget::W30Pad {
                bank_id: BankId::from("bank-b"),
                pad_id: PadId::from("pad-01"),
            }),
            is_pinned: false,
            notes: Some("bank b".into()),
        });
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
        state.refresh_view();

        assert_eq!(
            state.queue_w30_swap_bank(612),
            Some(QueueControlResult::Enqueued)
        );

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Bar,
                beat_index: 41,
                bar_index: 11,
                phrase_index: 3,
                scene_id: Some(SceneId::from("scene-1")),
            },
            712,
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
            Some("pad-01".into())
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
            Some("cap-02".into())
        );
        assert_eq!(state.jam_view.lanes.w30_pending_bank_swap_target, None);
        assert_eq!(state.runtime_view.w30_preview_mode, "live_recall");
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some("swapped W-30 bank to bank-b/pad-01 with cap-02")
        );
    }

    #[test]
    fn committed_w30_slice_pool_browse_updates_last_capture_and_log_result() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        });
        state.session.captures.push(CaptureRef {
            capture_id: CaptureId::from("cap-02"),
            capture_type: CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: vec![CaptureId::from("cap-01")],
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(CaptureTarget::W30Pad {
                bank_id: BankId::from("bank-a"),
                pad_id: PadId::from("pad-01"),
            }),
            is_pinned: false,
            notes: Some("alt slice".into()),
        });
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
        state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
        state.refresh_view();

        assert_eq!(
            state.queue_w30_browse_slice_pool(713),
            Some(QueueControlResult::Enqueued)
        );

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Beat,
                beat_index: 42,
                bar_index: 11,
                phrase_index: 3,
                scene_id: Some(SceneId::from("scene-1")),
            },
            813,
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
            Some("bank-a".into())
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
            Some("pad-01".into())
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
            Some("cap-02".into())
        );
        assert_eq!(state.jam_view.lanes.w30_pending_slice_pool_target, None);
        assert_eq!(state.runtime_view.w30_preview_mode, "live_recall");
        assert_eq!(state.runtime_view.w30_preview_profile, "slice_pool_browse");
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some("browsed W-30 slice pool to cap-02 on bank-a/pad-01 at beat 42 / phrase 3")
        );
    }

    #[test]
    fn committed_w30_damage_profile_updates_grit_and_log_result() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        });
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
        state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
        state.session.runtime_state.lane_state.w30.preview_mode =
            Some(W30PreviewModeState::LiveRecall);
        state.session.runtime_state.macro_state.w30_grit = 0.4;
        state.refresh_view();

        assert_eq!(
            state.queue_w30_apply_damage_profile(620),
            Some(QueueControlResult::Enqueued)
        );

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Bar,
                beat_index: 45,
                bar_index: 12,
                phrase_index: 3,
                scene_id: Some(SceneId::from("scene-1")),
            },
            720,
        );

        assert_eq!(committed.len(), 1);
        assert_eq!(
            state.session.runtime_state.macro_state.w30_grit,
            JamAppState::W30_DAMAGE_PROFILE_GRIT
        );
        assert_eq!(state.jam_view.lanes.w30_pending_damage_profile_target, None);
        assert_eq!(state.runtime_view.w30_preview_mode, "live_recall");
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some("applied shred damage profile to cap-01 on W-30 pad bank-a/pad-01")
        );
    }

    #[test]
    fn committed_w30_step_focus_updates_lane_focus_and_preview() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        });
        state.session.captures.push(CaptureRef {
            capture_id: CaptureId::from("cap-02"),
            capture_type: CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(CaptureTarget::W30Pad {
                bank_id: BankId::from("bank-b"),
                pad_id: PadId::from("pad-04"),
            }),
            is_pinned: true,
            notes: Some("secondary".into()),
        });
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
        state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
        state.refresh_view();

        assert_eq!(
            state.queue_w30_step_focus(612),
            Some(QueueControlResult::Enqueued)
        );

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Beat,
                beat_index: 37,
                bar_index: 10,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            702,
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
            Some("pad-04".into())
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
            Some("pad-04")
        );
        assert_eq!(state.jam_view.lanes.w30_pending_focus_step_target, None);
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
            Some(W30PreviewSourceProfile::PromotedRecall)
        );
        assert_eq!(
            state.runtime.w30_preview.active_bank_id.as_deref(),
            Some("bank-b")
        );
        assert_eq!(
            state.runtime.w30_preview.focused_pad_id.as_deref(),
            Some("pad-04")
        );
        assert_eq!(
            state
                .session
                .action_log
                .actions
                .last()
                .and_then(|action| action.result.as_ref())
                .map(|result| result.summary.as_str()),
            Some("focused W-30 pad bank-b/pad-04 at beat 37 / phrase 2")
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
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
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
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
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
    fn committed_w30_trigger_preserves_source_window_preview_samples() {
        let tempdir = tempdir().expect("create source audio tempdir");
        let source_path = tempdir.path().join("source.wav");
        write_pcm16_wave(&source_path, 48_000, 2, 1.0);

        let mut graph = sample_graph();
        graph.source.path = source_path.to_string_lossy().into_owned();
        graph.source.duration_seconds = 1.0;
        let mut session = sample_session(&graph);
        session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-b"),
            pad_id: PadId::from("pad-03"),
        });
        session.captures[0].source_window = Some(CaptureSourceWindow {
            source_id: graph.source.source_id.clone(),
            start_seconds: 0.0,
            end_seconds: 1.0,
            start_frame: 0,
            end_frame: 48_000,
        });
        session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
        session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
        let source_audio_cache =
            SourceAudioCache::load_pcm_wav(&source_path).expect("load source audio cache");
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
        state.source_audio_cache = Some(source_audio_cache);
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
        assert_eq!(
            state.runtime.w30_preview.mode,
            W30PreviewRenderMode::LiveRecall
        );
        assert_eq!(
            state.runtime.w30_preview.source_profile,
            Some(W30PreviewSourceProfile::PromotedRecall)
        );
        assert_eq!(
            state.runtime.w30_preview.capture_id.as_deref(),
            Some("cap-01")
        );
        assert_eq!(state.runtime.w30_preview.trigger_revision, 2);
        let preview = state
            .runtime
            .w30_preview
            .source_window_preview
            .as_ref()
            .expect("source-window preview");
        assert_eq!(preview.source_start_frame, 0);
        assert_eq!(preview.source_end_frame, 48_000);
        assert_eq!(preview.sample_count, W30_PREVIEW_SAMPLE_WINDOW_LEN);
        assert!(preview.samples.iter().any(|sample| sample.abs() > 0.001));
    }

    #[test]
    fn committed_w30_internal_resample_materializes_lineage_safe_capture() {
        let graph = sample_graph();
        let session = sample_session(&graph);
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-b"),
            pad_id: PadId::from("pad-03"),
        });
        state.session.captures[0].is_pinned = true;
        state.session.captures[0].lineage_capture_refs = vec![CaptureId::from("cap-root")];
        state.session.captures[0].resample_generation_depth = 1;
        state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
        state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
        state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
        state.refresh_view();

        assert_eq!(
            state.queue_w30_internal_resample(650),
            Some(QueueControlResult::Enqueued)
        );

        let committed = state.commit_ready_actions(
            CommitBoundaryState {
                kind: CommitBoundary::Phrase,
                beat_index: 33,
                bar_index: 9,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            740,
        );

        assert_eq!(committed.len(), 1);
        let capture = state
            .session
            .captures
            .last()
            .expect("new resample capture should exist");
        assert_eq!(capture.capture_type, CaptureType::Resample);
        assert_eq!(capture.capture_id, CaptureId::from("cap-02"));
        assert_eq!(
            capture.lineage_capture_refs,
            vec![CaptureId::from("cap-root"), CaptureId::from("cap-01")]
        );
        assert_eq!(capture.resample_generation_depth, 2);
        assert_eq!(capture.assigned_target, None);
        assert_eq!(
            state.session.runtime_state.lane_state.w30.last_capture,
            Some(CaptureId::from("cap-02"))
        );
        assert_eq!(
            state.runtime.w30_resample_tap.source_capture_id.as_deref(),
            Some("cap-02")
        );
        assert_eq!(state.runtime.w30_resample_tap.lineage_capture_count, 2);
        assert_eq!(state.runtime.w30_resample_tap.generation_depth, 2);
    }

    #[test]
    fn legacy_w30_preview_mode_is_backfilled_from_committed_preview_history() {
        let graph = sample_graph();
        let mut session = sample_session(&graph);
        session.runtime_state.lane_state.w30.preview_mode = None;
        session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
        session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
        session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
        session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-b"),
            pad_id: PadId::from("pad-03"),
        });
        session.action_log.actions.push(Action {
            id: ActionId(2),
            actor: ActorType::User,
            command: ActionCommand::W30AuditionPromoted,
            params: ActionParams::Mutation {
                target_id: Some("cap-01".into()),
                intensity: 0.68,
            },
            target: ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(BankId::from("bank-b")),
                pad_id: Some(PadId::from("pad-03")),
                ..Default::default()
            },
            requested_at: 600,
            quantization: Quantization::NextBar,
            status: ActionStatus::Committed,
            committed_at: Some(700),
            undo_policy: UndoPolicy::Undoable,
            result: Some(ActionResult {
                accepted: true,
                summary: "auditioned cap-01 on W-30 pad bank-b/pad-03".into(),
            }),
            explanation: Some("audition promoted cap-01 on W-30 pad bank-b/pad-03".into()),
        });

        let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(
            state.session.runtime_state.lane_state.w30.preview_mode,
            Some(W30PreviewModeState::PromotedAudition)
        );
        assert_eq!(
            state.runtime.w30_preview.mode,
            W30PreviewRenderMode::PromotedAudition
        );
    }

    #[test]
    fn explicit_w30_preview_mode_overrides_stale_action_history() {
        let graph = sample_graph();
        let mut session = sample_session(&graph);
        session.runtime_state.lane_state.w30.preview_mode = Some(W30PreviewModeState::LiveRecall);
        session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
        session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
        session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
        session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-b"),
            pad_id: PadId::from("pad-03"),
        });
        session.action_log.actions.push(Action {
            id: ActionId(2),
            actor: ActorType::User,
            command: ActionCommand::W30AuditionPromoted,
            params: ActionParams::Mutation {
                target_id: Some("cap-01".into()),
                intensity: 0.68,
            },
            target: ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(BankId::from("bank-b")),
                pad_id: Some(PadId::from("pad-03")),
                ..Default::default()
            },
            requested_at: 600,
            quantization: Quantization::NextBar,
            status: ActionStatus::Committed,
            committed_at: Some(700),
            undo_policy: UndoPolicy::Undoable,
            result: Some(ActionResult {
                accepted: true,
                summary: "auditioned cap-01 on W-30 pad bank-b/pad-03".into(),
            }),
            explanation: Some("audition promoted cap-01 on W-30 pad bank-b/pad-03".into()),
        });

        let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

        assert_eq!(
            state.session.runtime_state.lane_state.w30.preview_mode,
            Some(W30PreviewModeState::LiveRecall)
        );
        assert_eq!(
            state.runtime.w30_preview.mode,
            W30PreviewRenderMode::LiveRecall
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
                .reinforcement_mode,
            Some(Tr909ReinforcementModeState::BreakReinforce)
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
            state.jam_view.lanes.tr909_reinforcement_mode,
            Some(Tr909ReinforcementModeState::BreakReinforce)
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
        assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Follower);
        assert_eq!(
            state.runtime.mc202_render.phrase_shape,
            Mc202PhraseShape::FollowerDrive
        );
        assert_eq!(
            state.runtime.mc202_render.routing,
            Mc202RenderRouting::MusicBusBass
        );
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
        assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Answer);
        assert_eq!(
            state.runtime.mc202_render.phrase_shape,
            Mc202PhraseShape::AnswerHook
        );
        assert_eq!(
            state.runtime.mc202_render.routing,
            Mc202RenderRouting::MusicBusBass
        );
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
            session.captures[0].assigned_target =
                fixture.capture_assigned.then(|| CaptureTarget::W30Pad {
                    bank_id: BankId::from(fixture.capture_bank.clone()),
                    pad_id: PadId::from(fixture.capture_pad.clone()),
                });
            session.captures[0].is_pinned = fixture.capture_pinned;
            session.captures[0].source_window =
                fixture
                    .source_window
                    .as_ref()
                    .map(|source_window| CaptureSourceWindow {
                        source_id: SourceId::from(source_window.source_id.clone()),
                        start_seconds: source_window.start_seconds,
                        end_seconds: source_window.end_seconds,
                        start_frame: source_window.start_frame,
                        end_frame: source_window.end_frame,
                    });
            for extra in &fixture.extra_captures {
                session.captures.push(CaptureRef {
                    capture_id: CaptureId::from(extra.capture_id.clone()),
                    capture_type: CaptureType::Pad,
                    source_origin_refs: vec!["fixture-extra".into()],
                    source_window: None,
                    lineage_capture_refs: Vec::new(),
                    resample_generation_depth: 0,
                    created_from_action: None,
                    storage_path: format!("captures/{}.wav", extra.capture_id),
                    assigned_target: Some(CaptureTarget::W30Pad {
                        bank_id: BankId::from(extra.bank.clone()),
                        pad_id: PadId::from(extra.pad.clone()),
                    }),
                    is_pinned: extra.pinned,
                    notes: extra.notes.clone(),
                });
            }
            session.runtime_state.lane_state.w30.active_bank = Some(BankId::from(
                fixture
                    .initial_active_bank
                    .clone()
                    .unwrap_or_else(|| fixture.capture_bank.clone()),
            ));
            session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from(
                fixture
                    .initial_focused_pad
                    .clone()
                    .unwrap_or_else(|| fixture.capture_pad.clone()),
            ));
            session.runtime_state.lane_state.w30.last_capture =
                fixture.initial_last_capture.clone().map(CaptureId::from);
            session.runtime_state.lane_state.w30.preview_mode = fixture
                .initial_preview_mode
                .as_deref()
                .map(w30_preview_mode_state);
            session.runtime_state.macro_state.w30_grit = fixture.initial_w30_grit.unwrap_or(0.0);
            let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

            let queue_result = match fixture.action {
                W30RegressionAction::LiveRecall => {
                    state.queue_w30_live_recall(fixture.requested_at)
                }
                W30RegressionAction::RawCaptureAudition => {
                    state.queue_w30_audition(fixture.requested_at)
                }
                W30RegressionAction::PromotedAudition => {
                    state.queue_w30_promoted_audition(fixture.requested_at)
                }
                W30RegressionAction::TriggerPad => {
                    state.queue_w30_trigger_pad(fixture.requested_at)
                }
                W30RegressionAction::SwapBank => state.queue_w30_swap_bank(fixture.requested_at),
                W30RegressionAction::ApplyDamageProfile => {
                    state.queue_w30_apply_damage_profile(fixture.requested_at)
                }
                W30RegressionAction::LoopFreeze => {
                    state.queue_w30_loop_freeze(fixture.requested_at)
                }
                W30RegressionAction::BrowseSlicePool => {
                    state.queue_w30_browse_slice_pool(fixture.requested_at)
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
                    .action_log
                    .actions
                    .last()
                    .map(|action| action.command),
                Some(expected_w30_command(fixture.action)),
                "{} command drifted",
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
            if let Some(expected) = fixture.expected.preview_mode.as_deref() {
                assert_eq!(
                    state.runtime.w30_preview.mode.label(),
                    expected,
                    "{} preview mode drifted",
                    fixture.name
                );
            }
            if let Some(expected) = fixture.expected.preview_routing.as_deref() {
                assert_eq!(
                    state.runtime.w30_preview.routing.label(),
                    expected,
                    "{} preview routing drifted",
                    fixture.name
                );
            }
            if let Some(expected) = fixture.expected.preview_profile.as_deref() {
                assert_eq!(
                    state
                        .runtime
                        .w30_preview
                        .source_profile
                        .map(|profile| profile.label()),
                    Some(expected),
                    "{} preview profile drifted",
                    fixture.name
                );
            }
            if let Some(expected) = fixture.expected.preview_capture.as_deref() {
                assert_eq!(
                    state.runtime.w30_preview.capture_id.as_deref(),
                    Some(expected),
                    "{} preview capture drifted",
                    fixture.name
                );
            }
            if let Some(expected) = fixture.expected.preview_music_bus_level {
                assert!(
                    (state.runtime.w30_preview.music_bus_level - expected).abs() < f32::EPSILON,
                    "{} preview music bus drifted",
                    fixture.name
                );
            }
            if let Some(expected) = fixture.expected.preview_grit_level {
                assert!(
                    (state.runtime.w30_preview.grit_level - expected).abs() < f32::EPSILON,
                    "{} preview grit drifted",
                    fixture.name
                );
            }
            if let Some(expected) = fixture.expected.preview_transport_running {
                assert_eq!(
                    state.runtime.w30_preview.is_transport_running, expected,
                    "{} preview transport-running drifted",
                    fixture.name
                );
            }
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
                .takeover_profile,
            Some(Tr909TakeoverProfileState::ControlledPhraseTakeover)
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .lane_state
                .tr909
                .reinforcement_mode,
            Some(Tr909ReinforcementModeState::Takeover)
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
            state.jam_view.lanes.tr909_takeover_profile,
            Some(Tr909TakeoverProfileState::ControlledPhraseTakeover)
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
                .takeover_profile,
            Some(Tr909TakeoverProfileState::SceneLockTakeover)
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
            state.jam_view.lanes.tr909_takeover_profile,
            Some(Tr909TakeoverProfileState::SceneLockTakeover)
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
                .reinforcement_mode,
            Some(Tr909ReinforcementModeState::SourceSupport)
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
        session.runtime_state.lane_state.tr909.reinforcement_mode =
            Some(Tr909ReinforcementModeState::SourceSupport);
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
        assert_eq!(
            state.runtime.tr909_render.source_support_context,
            Some(Tr909SourceSupportContext::TransportBar)
        );
        assert_eq!(
            state.runtime_view.tr909_render_support_context,
            "transport_bar"
        );
        assert_eq!(
            state.runtime_view.tr909_render_support_accent,
            "off fallback"
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
            state.runtime.tr909_render.source_support_context,
            Some(Tr909SourceSupportContext::TransportBar)
        );
        assert_eq!(
            state.runtime_view.tr909_render_support_context,
            "transport_bar"
        );
        assert_eq!(
            state.runtime_view.tr909_render_support_accent,
            "off fallback"
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
            .reinforcement_mode = Some(Tr909ReinforcementModeState::SourceSupport);
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
            .takeover_profile = Some(Tr909TakeoverProfileState::ControlledPhraseTakeover);
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
        session.runtime_state.lane_state.tr909.reinforcement_mode =
            Some(Tr909ReinforcementModeState::SourceSupport);
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

        let mut graph = sample_graph();
        graph.sections.push(Section {
            section_id: SectionId::from("section-b"),
            label_hint: SectionLabelHint::Break,
            start_seconds: 16.0,
            end_seconds: 32.0,
            bar_start: 9,
            bar_end: 16,
            energy_class: EnergyClass::Medium,
            confidence: 0.85,
            tags: vec!["break".into()],
        });
        for fixture in fixtures {
            let mut session = sample_session(&graph);
            session.runtime_state.transport.position_beats = fixture.transport_position_beats;
            if let Some(scene_context) = fixture.scene_context.as_deref() {
                let scene_id = SceneId::from(scene_context);
                session.runtime_state.scene_state.active_scene = Some(scene_id.clone());
                session.runtime_state.transport.current_scene = Some(scene_id);
            }
            session.runtime_state.lane_state.tr909.reinforcement_mode =
                Some(fixture.reinforcement_mode);
            session.runtime_state.lane_state.tr909.takeover_enabled = fixture.takeover_enabled;
            session.runtime_state.lane_state.tr909.takeover_profile = fixture.takeover_profile;
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
                    .source_support_context
                    .map(|context| context.label().to_string()),
                fixture.expected_source_support_context,
                "{} support context drifted",
                fixture.name
            );
            assert_eq!(
                state.runtime_view.tr909_render_support_accent, fixture.expected_support_accent,
                "{} support accent drifted",
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
        assert_eq!(state.jam_view.scene.scene_count, 2);
        assert_eq!(
            state.jam_view.scene.active_scene.as_deref(),
            Some("scene-01-intro")
        );
        assert_eq!(
            state.jam_view.scene.active_scene_energy.as_deref(),
            Some("medium")
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .scene_state
                .scenes
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["scene-01-intro".to_string(), "scene-02-drop".to_string()]
        );
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
        let persisted_session = load_session_json(&session_path).expect("reload session");
        assert_eq!(
            persisted_session
                .runtime_state
                .scene_state
                .scenes
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["scene-01-intro".to_string(), "scene-02-drop".to_string()]
        );
        assert_eq!(
            persisted_session.runtime_state.scene_state.active_scene,
            Some(SceneId::from("scene-01-intro"))
        );
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

    #[test]
    fn scene_fixture_backed_committed_state_regressions_hold() {
        let fixtures: Vec<SceneRegressionFixture> =
            serde_json::from_str(include_str!("../tests/fixtures/scene_regression.json"))
                .expect("parse Scene Brain regression fixtures");

        for fixture in fixtures {
            let graph = scene_regression_graph(&fixture.section_labels);
            let mut session = sample_session(&graph);
            session.runtime_state.transport.current_scene = None;
            session.runtime_state.scene_state.active_scene = None;
            session.runtime_state.scene_state.scenes.clear();

            let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
            seed_scene_fixture_state(&mut state, &fixture);

            match fixture.action {
                SceneRegressionAction::ProjectCandidates => {}
                SceneRegressionAction::SelectNextScene => {
                    assert_eq!(
                        state.queue_scene_select(
                            fixture.requested_at.expect("scene select requested_at")
                        ),
                        QueueControlResult::Enqueued,
                        "{} did not enqueue",
                        fixture.name
                    );

                    let committed = state.commit_ready_actions(
                        fixture
                            .boundary
                            .expect("scene select boundary")
                            .into_commit_boundary_state(),
                        fixture.committed_at.expect("scene select committed_at"),
                    );
                    assert_eq!(
                        committed.len(),
                        1,
                        "{} did not commit exactly one action",
                        fixture.name
                    );
                }
                SceneRegressionAction::RestoreScene => {
                    assert_eq!(
                        state.queue_scene_restore(
                            fixture.requested_at.expect("scene restore requested_at")
                        ),
                        QueueControlResult::Enqueued,
                        "{} did not enqueue",
                        fixture.name
                    );

                    let committed = state.commit_ready_actions(
                        fixture
                            .boundary
                            .expect("scene restore boundary")
                            .into_commit_boundary_state(),
                        fixture.committed_at.expect("scene restore committed_at"),
                    );
                    assert_eq!(
                        committed.len(),
                        1,
                        "{} did not commit exactly one action",
                        fixture.name
                    );
                }
            }

            let actual_scenes = state
                .session
                .runtime_state
                .scene_state
                .scenes
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            assert_eq!(
                actual_scenes, fixture.expected.scenes,
                "{} scenes drifted",
                fixture.name
            );
            assert_eq!(
                state
                    .session
                    .runtime_state
                    .scene_state
                    .active_scene
                    .as_ref()
                    .map(ToString::to_string)
                    .as_deref(),
                Some(fixture.expected.active_scene.as_str()),
                "{} active scene drifted",
                fixture.name
            );
            assert_eq!(
                state
                    .session
                    .runtime_state
                    .transport
                    .current_scene
                    .as_ref()
                    .map(ToString::to_string)
                    .as_deref(),
                Some(fixture.expected.current_scene.as_str()),
                "{} transport scene drifted",
                fixture.name
            );
            assert_eq!(
                state
                    .session
                    .runtime_state
                    .scene_state
                    .restore_scene
                    .as_ref()
                    .map(ToString::to_string)
                    .as_deref(),
                fixture.expected.restore_scene.as_deref(),
                "{} restore scene drifted",
                fixture.name
            );
            assert_eq!(
                state.jam_view.scene.active_scene.as_deref(),
                Some(fixture.expected.active_scene.as_str()),
                "{} jam view scene drifted",
                fixture.name
            );
            assert_eq!(
                state.jam_view.scene.active_scene_energy.as_deref(),
                Some(fixture.expected.active_scene_energy.as_str()),
                "{} jam view active energy drifted",
                fixture.name
            );
            assert_eq!(
                state.jam_view.scene.restore_scene_energy.as_deref(),
                fixture.expected.restore_scene_energy.as_deref(),
                "{} jam view restore energy drifted",
                fixture.name
            );
            assert_eq!(
                state
                    .runtime
                    .transport
                    .current_scene
                    .as_ref()
                    .map(ToString::to_string)
                    .as_deref(),
                Some(fixture.expected.current_scene.as_str()),
                "{} runtime transport scene drifted",
                fixture.name
            );

            if let Some(expected_summary) = &fixture.expected.result_summary {
                assert_eq!(
                    state
                        .session
                        .action_log
                        .actions
                        .last()
                        .and_then(|action| action.result.as_ref())
                        .map(|result| result.summary.as_str()),
                    Some(expected_summary.as_str()),
                    "{} result summary drifted",
                    fixture.name
                );
            }
            if let Some(expected_profile) = fixture.expected.tr909_render_profile.as_deref() {
                assert_eq!(
                    state.runtime_view.tr909_render_profile, expected_profile,
                    "{} TR-909 profile drifted",
                    fixture.name
                );
            }
            if let Some(expected_context) = fixture.expected.tr909_render_support_context.as_deref()
            {
                assert_eq!(
                    state.runtime_view.tr909_render_support_context, expected_context,
                    "{} TR-909 support context drifted",
                    fixture.name
                );
            }
            if let Some(expected_accent) = fixture.expected.tr909_render_support_accent.as_deref() {
                assert_eq!(
                    state.runtime_view.tr909_render_support_accent, expected_accent,
                    "{} TR-909 support accent drifted",
                    fixture.name
                );
            }

            let tempdir = tempdir().expect("create Scene Brain regression tempdir");
            let session_path = tempdir.path().join(format!("{}.json", fixture.name));
            save_session_json(&session_path, &state.session)
                .expect("save Scene Brain regression session");
            let loaded =
                load_session_json(&session_path).expect("reload Scene Brain regression session");

            let loaded_scenes = loaded
                .runtime_state
                .scene_state
                .scenes
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            assert_eq!(
                loaded_scenes, fixture.expected.scenes,
                "{} scenes did not survive replay roundtrip",
                fixture.name
            );
            assert_eq!(
                loaded
                    .runtime_state
                    .scene_state
                    .active_scene
                    .as_ref()
                    .map(ToString::to_string)
                    .as_deref(),
                Some(fixture.expected.active_scene.as_str()),
                "{} active scene did not survive replay roundtrip",
                fixture.name
            );
            assert_eq!(
                loaded
                    .runtime_state
                    .transport
                    .current_scene
                    .as_ref()
                    .map(ToString::to_string)
                    .as_deref(),
                Some(fixture.expected.current_scene.as_str()),
                "{} transport scene did not survive replay roundtrip",
                fixture.name
            );
            if let Some(expected_summary) = &fixture.expected.result_summary {
                assert_eq!(
                    loaded
                        .action_log
                        .actions
                        .last()
                        .and_then(|action| action.result.as_ref())
                        .map(|result| result.summary.as_str()),
                    Some(expected_summary.as_str()),
                    "{} result summary did not survive replay roundtrip",
                    fixture.name
                );
            }
        }
    }
}
