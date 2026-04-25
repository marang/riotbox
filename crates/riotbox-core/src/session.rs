use serde::{Deserialize, Serialize};

use crate::{
    action::{Action, GhostMode},
    ids::{ActionId, BankId, CaptureId, PadId, SceneId, SnapshotId, SourceId},
    source_graph::{GraphProvenance, SourceGraph, SourceGraphVersion},
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
    pub replay_policy: ReplayPolicy,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ReplayPolicy {
    #[default]
    DeterministicPreferred,
    SnapshotOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub snapshot_id: SnapshotId,
    pub created_at: String,
    pub label: String,
    pub action_cursor: usize,
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

impl Default for GhostState {
    fn default() -> Self {
        Self {
            mode: GhostMode::Watch,
            budgets: GhostBudgetState::default(),
            suggestion_history: Vec::new(),
            lock_awareness_enabled: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GhostBudgetState {
    pub max_actions_per_phrase: u8,
    pub max_destructive_actions_per_scene: u8,
    pub max_pending_actions: u8,
}

impl Default for GhostBudgetState {
    fn default() -> Self {
        Self {
            max_actions_per_phrase: 2,
            max_destructive_actions_per_scene: 1,
            max_pending_actions: 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GhostSuggestionRecord {
    pub proposal_id: String,
    pub summary: String,
    pub accepted: bool,
}

#[cfg(test)]
mod tests {
    use crate::{
        action::{
            Action, ActionCommand, ActionParams, ActionResult, ActionStatus, ActionTarget,
            ActorType, GhostMode, Quantization, TargetScope, UndoPolicy,
        },
        ids::{ActionId, BankId, CaptureId, PadId, SceneId, SnapshotId, SourceId},
        source_graph::{
            DecodeProfile, GraphProvenance, SourceDescriptor, SourceGraph, SourceGraphVersion,
        },
    };

    use super::*;

    #[test]
    fn session_file_roundtrips_via_json() {
        let graph = SourceGraph::new(
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
                run_notes: Some("roundtrip".into()),
            },
        );

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
        session.runtime_state.lane_state.mc202.role = Some("follower".into());
        session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
        session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
        session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
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
            assigned_target: Some(CaptureTarget::W30Pad {
                bank_id: BankId::from("bank-a"),
                pad_id: PadId::from("pad-01"),
            }),
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

        let json = serde_json::to_string_pretty(&session).expect("serialize session");
        let decoded: SessionFile = serde_json::from_str(&json).expect("deserialize session");

        assert_eq!(decoded, session);
    }

    #[test]
    fn legacy_capture_refs_without_source_window_still_load() {
        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-12T18:00:00Z");
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
            notes: None,
        });

        let mut value = serde_json::to_value(&session).expect("serialize session");
        value["captures"][0]
            .as_object_mut()
            .expect("capture object")
            .remove("source_window");

        let decoded: SessionFile =
            serde_json::from_value(value).expect("deserialize legacy session");

        assert_eq!(decoded.captures[0].source_window, None);
    }
}
