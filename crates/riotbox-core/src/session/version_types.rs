use serde::{Deserialize, Serialize};

use crate::{
    action::{Action, GhostMode},
    ids::{ActionId, BankId, CaptureId, PadId, SceneId, SnapshotId, SourceId},
    source_graph::{GraphProvenance, SourceGraph, SourceGraphVersion},
    transport::CommitBoundaryState,
    TimestampMs,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionVersion {
    V1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionFile {
    pub session_version: SessionVersion,
    pub session_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub app_version: String,
    pub source_refs: Vec<SourceRef>,
    pub source_graph_refs: Vec<SourceGraphRef>,
    pub runtime_state: RuntimeState,
    pub action_log: ActionLog,
    pub snapshots: Vec<Snapshot>,
    pub captures: Vec<CaptureRef>,
    pub ghost_state: GhostState,
    pub notes: Option<String>,
}

impl SessionFile {
    #[must_use]
    pub fn new(
        session_id: impl Into<String>,
        app_version: impl Into<String>,
        created_at: impl Into<String>,
    ) -> Self {
        let created_at = created_at.into();

        Self {
            session_version: SessionVersion::V1,
            session_id: session_id.into(),
            created_at: created_at.clone(),
            updated_at: created_at,
            app_version: app_version.into(),
            source_refs: Vec::new(),
            source_graph_refs: Vec::new(),
            runtime_state: RuntimeState::default(),
            action_log: ActionLog::default(),
            snapshots: Vec::new(),
            captures: Vec::new(),
            ghost_state: GhostState::default(),
            notes: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceRef {
    pub source_id: SourceId,
    pub path_hint: String,
    pub content_hash: String,
    pub duration_seconds: f32,
    pub decode_profile: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphStorageMode {
    Embedded,
    External,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceGraphRef {
    pub source_id: SourceId,
    pub graph_version: SourceGraphVersion,
    pub graph_hash: String,
    pub storage_mode: GraphStorageMode,
    pub embedded_graph: Option<SourceGraph>,
    pub external_path: Option<String>,
    pub provenance: GraphProvenance,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct RuntimeState {
    pub transport: TransportRuntimeState,
    pub macro_state: MacroState,
    pub lane_state: LaneState,
    pub mixer_state: MixerState,
    pub scene_state: SceneState,
    pub lock_state: LockState,
    pub pending_policy: PendingPolicy,
    #[serde(default)]
    pub undo_state: UndoRuntimeState,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct UndoRuntimeState {
    #[serde(default)]
    pub mc202_snapshots: Vec<Mc202UndoSnapshotState>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mc202UndoSnapshotState {
    pub action_id: ActionId,
    pub role: Option<String>,
    pub phrase_ref: Option<String>,
    #[serde(default)]
    pub phrase_variant: Option<Mc202PhraseVariantState>,
    pub touch: f32,
}

impl Mc202UndoSnapshotState {
    #[must_use]
    pub fn from_session(action_id: ActionId, session: &SessionFile) -> Self {
        Self {
            action_id,
            role: session.runtime_state.lane_state.mc202.role.clone(),
            phrase_ref: session.runtime_state.lane_state.mc202.phrase_ref.clone(),
            phrase_variant: session.runtime_state.lane_state.mc202.phrase_variant,
            touch: session.runtime_state.macro_state.mc202_touch,
        }
    }

    pub fn apply_to_session(&self, session: &mut SessionFile) {
        session.runtime_state.lane_state.mc202.role = self.role.clone();
        session.runtime_state.lane_state.mc202.phrase_ref = self.phrase_ref.clone();
        session.runtime_state.lane_state.mc202.phrase_variant = self.phrase_variant;
        session.runtime_state.macro_state.mc202_touch = self.touch;
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransportRuntimeState {
    pub is_playing: bool,
    pub position_beats: f64,
    pub current_scene: Option<SceneId>,
}

impl Default for TransportRuntimeState {
    fn default() -> Self {
        Self {
            is_playing: false,
            position_beats: 0.0,
            current_scene: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MacroState {
    pub source_retain: f32,
    pub chaos: f32,
    pub mc202_touch: f32,
    pub w30_grit: f32,
    pub tr909_slam: f32,
    pub scene_aggression: f32,
    pub capture_eagerness: f32,
    pub dirt_room_intensity: f32,
}

impl Default for MacroState {
    fn default() -> Self {
        Self {
            source_retain: 0.5,
            chaos: 0.25,
            mc202_touch: 0.4,
            w30_grit: 0.4,
            tr909_slam: 0.4,
            scene_aggression: 0.4,
            capture_eagerness: 0.3,
            dirt_room_intensity: 0.3,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct LaneState {
    pub mc202: Mc202LaneState,
    pub w30: W30LaneState,
    pub tr909: Tr909LaneState,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Mc202LaneState {
    pub role: Option<String>,
    pub phrase_ref: Option<String>,
    #[serde(default)]
    pub phrase_variant: Option<Mc202PhraseVariantState>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mc202PhraseVariantState {
    MutatedDrive,
}

impl Mc202PhraseVariantState {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::MutatedDrive => "mutated_drive",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum W30PreviewModeState {
    LiveRecall,
    RawCaptureAudition,
    PromotedAudition,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tr909TakeoverProfileState {
    ControlledPhraseTakeover,
    SceneLockTakeover,
}

impl Tr909TakeoverProfileState {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ControlledPhraseTakeover => "controlled_phrase_takeover",
            Self::SceneLockTakeover => "scene_lock_takeover",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tr909ReinforcementModeState {
    SourceSupport,
    Fills,
    BreakReinforce,
    Takeover,
}

impl Tr909ReinforcementModeState {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SourceSupport => "source_support",
            Self::Fills => "fills",
            Self::BreakReinforce => "break_reinforce",
            Self::Takeover => "takeover",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct W30LaneState {
    pub preview_mode: Option<W30PreviewModeState>,
    pub active_bank: Option<BankId>,
    pub focused_pad: Option<PadId>,
    pub last_capture: Option<CaptureId>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Tr909LaneState {
    pub pattern_ref: Option<String>,
    pub takeover_enabled: bool,
    pub takeover_profile: Option<Tr909TakeoverProfileState>,
    pub slam_enabled: bool,
    pub fill_armed_next_bar: bool,
    pub last_fill_bar: Option<u64>,
    pub reinforcement_mode: Option<Tr909ReinforcementModeState>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct MixerState {
    pub source_level: f32,
    pub drum_level: f32,
    pub music_level: f32,
    pub fx_send_level: f32,
    pub master_level: f32,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SceneState {
    pub active_scene: Option<SceneId>,
    pub scenes: Vec<SceneId>,
    pub restore_scene: Option<SceneId>,
    #[serde(default)]
    pub last_movement: Option<SceneMovementState>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneMovementState {
    pub action_id: ActionId,
    pub from_scene: Option<SceneId>,
    pub to_scene: SceneId,
    pub kind: SceneMovementKindState,
    pub direction: SceneMovementDirectionState,
    pub tr909_intent: SceneMovementLaneIntentState,
    pub mc202_intent: SceneMovementLaneIntentState,
    pub intensity: f32,
    pub committed_bar_index: u64,
    pub committed_phrase_index: u64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SceneMovementKindState {
    Launch,
    Restore,
}

impl SceneMovementKindState {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Launch => "launch",
            Self::Restore => "restore",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SceneMovementDirectionState {
    Rise,
    Drop,
    Hold,
}

impl SceneMovementDirectionState {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Rise => "rise",
            Self::Drop => "drop",
            Self::Hold => "hold",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SceneMovementLaneIntentState {
    Drive,
    Lift,
    Release,
    Anchor,
}

impl SceneMovementLaneIntentState {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Drive => "drive",
            Self::Lift => "lift",
            Self::Release => "release",
            Self::Anchor => "anchor",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct LockState {
    pub locked_object_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendingPolicy {
    pub max_pending_actions: usize,
    pub require_explicit_accept_for_ghost: bool,
}

impl Default for PendingPolicy {
    fn default() -> Self {
        Self {
            max_pending_actions: 8,
            require_explicit_accept_for_ghost: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ActionLog {
    pub actions: Vec<Action>,
    #[serde(default)]
    pub commit_records: Vec<ActionCommitRecord>,
    pub replay_policy: ReplayPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionCommitRecord {
    pub action_id: ActionId,
    pub boundary: CommitBoundaryState,
    pub commit_sequence: u32,
    pub committed_at: TimestampMs,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ReplayPolicy {
    #[default]
    DeterministicPreferred,
    SnapshotOnly,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotPayloadVersion {
    V1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SnapshotPayload {
    pub payload_version: SnapshotPayloadVersion,
    pub snapshot_id: SnapshotId,
    pub action_cursor: usize,
    pub runtime_state: RuntimeState,
}

impl SnapshotPayload {
    #[must_use]
    pub fn from_runtime_state(
        snapshot_id: &SnapshotId,
        action_cursor: usize,
        runtime_state: &RuntimeState,
    ) -> Self {
        Self {
            payload_version: SnapshotPayloadVersion::V1,
            snapshot_id: snapshot_id.clone(),
            action_cursor,
            runtime_state: runtime_state.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub snapshot_id: SnapshotId,
    pub created_at: String,
    pub label: String,
    pub action_cursor: usize,
    #[serde(default)]
    pub payload: Option<SnapshotPayload>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CaptureRef {
    pub capture_id: CaptureId,
    pub capture_type: CaptureType,
    pub source_origin_refs: Vec<String>,
    #[serde(default)]
    pub source_window: Option<CaptureSourceWindow>,
    #[serde(default)]
    pub lineage_capture_refs: Vec<CaptureId>,
    #[serde(default)]
    pub resample_generation_depth: u8,
    pub created_from_action: Option<ActionId>,
    pub storage_path: String,
    pub assigned_target: Option<CaptureTarget>,
    pub is_pinned: bool,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CaptureSourceWindow {
    pub source_id: SourceId,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub start_frame: u64,
    pub end_frame: u64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureType {
    Loop,
    Pad,
    Resample,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureTarget {
    W30Pad { bank_id: BankId, pad_id: PadId },
    Scene(SceneId),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GhostState {
    pub mode: GhostMode,
    pub budgets: GhostBudgetState,
    pub suggestion_history: Vec<GhostSuggestionRecord>,
    pub lock_awareness_enabled: bool,
}
