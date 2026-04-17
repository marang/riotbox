use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

use crate::{
    TimestampMs,
    ids::{ActionId, AssetId, BankId, CaptureId, PadId, SceneId},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorType {
    User,
    Ghost,
    System,
}

impl Display for ActorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::User => "user",
            Self::Ghost => "ghost",
            Self::System => "system",
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionStatus {
    Requested,
    Queued,
    PendingCommit,
    Committed,
    Rejected,
    Undone,
    Failed,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Quantization {
    Immediate,
    NextBeat,
    NextHalfBar,
    NextBar,
    NextPhrase,
    NextScene,
}

impl Quantization {
    #[must_use]
    pub fn is_ready_for(self, boundary: CommitBoundary) -> bool {
        self.rank() <= boundary.rank()
    }

    const fn rank(self) -> u8 {
        match self {
            Self::Immediate => 0,
            Self::NextBeat => 1,
            Self::NextHalfBar => 2,
            Self::NextBar => 3,
            Self::NextPhrase => 4,
            Self::NextScene => 5,
        }
    }
}

impl Display for Quantization {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Immediate => "immediate",
            Self::NextBeat => "next_beat",
            Self::NextHalfBar => "next_half_bar",
            Self::NextBar => "next_bar",
            Self::NextPhrase => "next_phrase",
            Self::NextScene => "next_scene",
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitBoundary {
    Immediate,
    Beat,
    HalfBar,
    Bar,
    Phrase,
    Scene,
}

impl CommitBoundary {
    const fn rank(self) -> u8 {
        match self {
            Self::Immediate => 0,
            Self::Beat => 1,
            Self::HalfBar => 2,
            Self::Bar => 3,
            Self::Phrase => 4,
            Self::Scene => 5,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetScope {
    Global,
    Scene,
    LaneMc202,
    LaneW30,
    LaneTr909,
    Mixer,
    Ghost,
    Session,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ActionTarget {
    pub scope: Option<TargetScope>,
    pub scene_id: Option<SceneId>,
    pub bank_id: Option<BankId>,
    pub pad_id: Option<PadId>,
    pub loop_id: Option<AssetId>,
    pub object_id: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GhostMode {
    Off,
    Watch,
    Assist,
    Perform,
}

impl Display for GhostMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Off => "off",
            Self::Watch => "watch",
            Self::Assist => "assist",
            Self::Perform => "perform",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ActionParams {
    Empty,
    Transport {
        position_beats: Option<u64>,
    },
    Mutation {
        intensity: f32,
        target_id: Option<String>,
    },
    Capture {
        bars: Option<u32>,
    },
    Promotion {
        capture_id: Option<CaptureId>,
        destination: Option<String>,
    },
    Scene {
        scene_id: Option<SceneId>,
    },
    Lock {
        object_id: String,
    },
    Snapshot {
        label: Option<String>,
    },
    Ghost {
        mode: Option<GhostMode>,
        proposal_id: Option<String>,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionCommand {
    TransportPlay,
    TransportPause,
    TransportStop,
    TransportSeek,
    MutateScene,
    MutateLane,
    MutateLoop,
    MutatePattern,
    MutateHook,
    CaptureNow,
    CaptureLoop,
    CaptureBarGroup,
    PromoteCaptureToPad,
    PromoteCaptureToScene,
    PromoteResample,
    SceneLaunch,
    SceneRestore,
    SceneRegenerate,
    SceneReinterpret,
    Mc202GenerateFollower,
    Mc202GenerateAnswer,
    Mc202SetRole,
    W30CaptureToPad,
    W30LiveRecall,
    W30TriggerPad,
    W30AuditionPromoted,
    W30SwapBank,
    W30BrowseSlicePool,
    W30StepFocus,
    W30ApplyDamageProfile,
    W30LoopFreeze,
    Tr909FillNext,
    Tr909SetSlam,
    Tr909ReinforceBreak,
    Tr909Takeover,
    Tr909SceneLock,
    Tr909Release,
    LockObject,
    UnlockObject,
    SnapshotSave,
    SnapshotLoad,
    UndoLast,
    RedoLast,
    RestoreSource,
    GhostSetMode,
    GhostAcceptSuggestion,
    GhostRejectSuggestion,
    GhostExecuteTool,
}

impl ActionCommand {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TransportPlay => "transport.play",
            Self::TransportPause => "transport.pause",
            Self::TransportStop => "transport.stop",
            Self::TransportSeek => "transport.seek",
            Self::MutateScene => "mutate.scene",
            Self::MutateLane => "mutate.lane",
            Self::MutateLoop => "mutate.loop",
            Self::MutatePattern => "mutate.pattern",
            Self::MutateHook => "mutate.hook",
            Self::CaptureNow => "capture.now",
            Self::CaptureLoop => "capture.loop",
            Self::CaptureBarGroup => "capture.bar_group",
            Self::PromoteCaptureToPad => "promote.capture_to_pad",
            Self::PromoteCaptureToScene => "promote.capture_to_scene",
            Self::PromoteResample => "promote.resample",
            Self::SceneLaunch => "scene.launch",
            Self::SceneRestore => "scene.restore",
            Self::SceneRegenerate => "scene.regenerate",
            Self::SceneReinterpret => "scene.reinterpret",
            Self::Mc202GenerateFollower => "mc202.generate_follower",
            Self::Mc202GenerateAnswer => "mc202.generate_answer",
            Self::Mc202SetRole => "mc202.set_role",
            Self::W30CaptureToPad => "w30.capture_to_pad",
            Self::W30LiveRecall => "w30.live_recall",
            Self::W30TriggerPad => "w30.trigger_pad",
            Self::W30AuditionPromoted => "w30.audition_promoted",
            Self::W30SwapBank => "w30.swap_bank",
            Self::W30BrowseSlicePool => "w30.browse_slice_pool",
            Self::W30StepFocus => "w30.step_focus",
            Self::W30ApplyDamageProfile => "w30.apply_damage_profile",
            Self::W30LoopFreeze => "w30.loop_freeze",
            Self::Tr909FillNext => "tr909.fill_next",
            Self::Tr909SetSlam => "tr909.set_slam",
            Self::Tr909ReinforceBreak => "tr909.reinforce_break",
            Self::Tr909Takeover => "tr909.takeover",
            Self::Tr909SceneLock => "tr909.scene_lock",
            Self::Tr909Release => "tr909.release",
            Self::LockObject => "lock.object",
            Self::UnlockObject => "unlock.object",
            Self::SnapshotSave => "snapshot.save",
            Self::SnapshotLoad => "snapshot.load",
            Self::UndoLast => "undo.last",
            Self::RedoLast => "redo.last",
            Self::RestoreSource => "restore.source",
            Self::GhostSetMode => "ghost.set_mode",
            Self::GhostAcceptSuggestion => "ghost.accept_suggestion",
            Self::GhostRejectSuggestion => "ghost.reject_suggestion",
            Self::GhostExecuteTool => "ghost.execute_tool",
        }
    }
}

impl Display for ActionCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionResult {
    pub accepted: bool,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UndoPolicy {
    Undoable,
    NotUndoable { reason: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub id: ActionId,
    pub actor: ActorType,
    pub command: ActionCommand,
    pub params: ActionParams,
    pub target: ActionTarget,
    pub requested_at: TimestampMs,
    pub quantization: Quantization,
    pub status: ActionStatus,
    pub committed_at: Option<TimestampMs>,
    pub result: Option<ActionResult>,
    pub undo_policy: UndoPolicy,
    pub explanation: Option<String>,
}

impl Action {
    #[must_use]
    pub fn short_label(&self) -> String {
        match &self.explanation {
            Some(explanation) if !explanation.is_empty() => {
                format!("{} ({explanation})", self.command)
            }
            _ => self.command.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionDraft {
    pub actor: ActorType,
    pub command: ActionCommand,
    pub params: ActionParams,
    pub target: ActionTarget,
    pub quantization: Quantization,
    pub undo_policy: UndoPolicy,
    pub explanation: Option<String>,
}

impl ActionDraft {
    #[must_use]
    pub fn new(
        actor: ActorType,
        command: ActionCommand,
        quantization: Quantization,
        target: ActionTarget,
    ) -> Self {
        Self {
            actor,
            command,
            params: ActionParams::Empty,
            target,
            quantization,
            undo_policy: UndoPolicy::Undoable,
            explanation: None,
        }
    }
}
