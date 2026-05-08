use crossterm::event::KeyCode;
use ratatui::{
    Frame, Terminal,
    backend::TestBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use riotbox_audio::w30::W30PreviewRenderMode;
use riotbox_core::source_graph::{
    DecodeProfile, EnergyClass, QualityClass, Section, SectionLabelHint, TimingDegradedPolicy,
    TimingWarningCode,
};
use riotbox_core::{
    action::{ActionCommand, ActionStatus, GhostMode},
    view::jam::{
        CaptureHandoffReadinessView, CaptureTargetKindView, SceneJumpAvailabilityView,
        SceneTransitionKindView, SceneTransitionPolicyView, W30PendingAuditionKind,
    },
};

use crate::jam_app::{JamAppState, SessionRecoverySurface};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellScreen {
    Jam,
    Log,
    Source,
    Capture,
}

impl ShellScreen {
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Jam => "jam",
            Self::Log => "log",
            Self::Source => "source",
            Self::Capture => "capture",
        }
    }

    #[must_use]
    pub const fn next(&self) -> Self {
        match self {
            Self::Jam => Self::Log,
            Self::Log => Self::Source,
            Self::Source => Self::Capture,
            Self::Capture => Self::Jam,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShellLaunchMode {
    Load,
    Ingest,
}

impl ShellLaunchMode {
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Load => "load",
            Self::Ingest => "ingest",
        }
    }

    #[must_use]
    pub const fn refresh_verb(&self) -> &'static str {
        match self {
            Self::Load => "reload session",
            Self::Ingest => "re-ingest source",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JamViewMode {
    Perform,
    Inspect,
}

impl JamViewMode {
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Perform => "perform",
            Self::Inspect => "inspect",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellKeyOutcome {
    Continue,
    RequestRefresh,
    ToggleTransport,
    QueueSceneMutation,
    QueueSceneSelect,
    QueueSceneRestore,
    QueueMc202RoleToggle,
    QueueMc202GenerateFollower,
    QueueMc202GenerateAnswer,
    QueueMc202GeneratePressure,
    QueueMc202GenerateInstigator,
    QueueMc202MutatePhrase,
    QueueTr909Fill,
    QueueTr909Reinforce,
    QueueTr909Slam,
    QueueTr909Takeover,
    QueueTr909SceneLock,
    QueueTr909Release,
    QueueCaptureBar,
    PromoteLastCapture,
    QueueW30TriggerPad,
    QueueW30StepFocus,
    QueueW30SwapBank,
    QueueW30BrowseSlicePool,
    QueueW30ApplyDamageProfile,
    QueueW30LoopFreeze,
    QueueW30LiveRecall,
    QueueW30Audition,
    QueueW30Resample,
    TogglePinLatestCapture,
    LowerDrumBusLevel,
    RaiseDrumBusLevel,
    LowerMc202Touch,
    RaiseMc202Touch,
    AcceptCurrentGhostSuggestion,
    RejectCurrentGhostSuggestion,
    UndoLast,
    Quit,
}

const GESTURE_MUTATE: &str = "mutate";
const GESTURE_SCENE_JUMP: &str = "scene jump";
const GESTURE_RESTORE: &str = "restore";
const GESTURE_VOICE: &str = "voice";
const GESTURE_FOLLOW: &str = "follow";
const GESTURE_ANSWER: &str = "answer";
const GESTURE_PRESSURE: &str = "pressure";
const GESTURE_INSTIGATE: &str = "instigate";
const GESTURE_PHRASE: &str = "phrase";
const GESTURE_FILL: &str = "fill";
const GESTURE_PUSH: &str = "push";
const GESTURE_SLAM: &str = "slam";
const GESTURE_TAKEOVER: &str = "takeover";
const GESTURE_LOCK: &str = "lock";
const GESTURE_RELEASE: &str = "release";
const GESTURE_CAPTURE: &str = "capture";
const GESTURE_PROMOTE: &str = "promote";
const GESTURE_HIT: &str = "hit";
const GESTURE_NEXT_PAD: &str = "next pad";
const GESTURE_BANK: &str = "bank";
const GESTURE_BROWSE: &str = "browse";
const GESTURE_DAMAGE: &str = "damage";
const GESTURE_FREEZE: &str = "freeze";
const GESTURE_RECALL: &str = "recall";
const GESTURE_AUDITION: &str = "audition";
const GESTURE_RESAMPLE: &str = "resample";
const GESTURE_TOUCH: &str = "touch";
const GESTURE_UNDO: &str = "undo";

const ADVANCED_GESTURES: &[(&str, &str)] = &[
    ("Y", GESTURE_RESTORE),
    ("a", GESTURE_ANSWER),
    ("b", GESTURE_VOICE),
    ("P", GESTURE_PRESSURE),
    ("I", GESTURE_INSTIGATE),
    ("G", GESTURE_PHRASE),
    ("d", GESTURE_PUSH),
    ("t", GESTURE_TAKEOVER),
    ("k", GESTURE_LOCK),
];

const LANE_GESTURES: &[(&str, &str)] = &[
    ("< >", GESTURE_TOUCH),
    ("l", GESTURE_RECALL),
    ("o", GESTURE_AUDITION),
    ("z", GESTURE_FREEZE),
    ("e", GESTURE_RESAMPLE),
    ("B", GESTURE_BANK),
    ("j", GESTURE_BROWSE),
];

const HELP_PRIMARY_CONFIRM_GESTURES: &[(&str, &str)] = &[
    ("c", GESTURE_CAPTURE),
    ("w", GESTURE_HIT),
    ("u", GESTURE_UNDO),
];

const HELP_ADVANCED_GESTURES_A: &[(&str, &str)] = &[
    ("Y", GESTURE_RESTORE),
    ("a", GESTURE_ANSWER),
    ("m", GESTURE_MUTATE),
    ("b", GESTURE_VOICE),
    ("P", GESTURE_PRESSURE),
    ("I", GESTURE_INSTIGATE),
    ("G", GESTURE_PHRASE),
    ("d", GESTURE_PUSH),
];

const HELP_ADVANCED_GESTURES_B: &[(&str, &str)] = &[
    ("s", GESTURE_SLAM),
    ("t", GESTURE_TAKEOVER),
    ("k", GESTURE_LOCK),
    ("x", GESTURE_RELEASE),
];

const HELP_ADVANCED_GESTURES_C: &[(&str, &str)] = &[
    ("p", GESTURE_PROMOTE),
    ("n", GESTURE_NEXT_PAD),
    ("B", GESTURE_BANK),
    ("j", GESTURE_BROWSE),
];

const HELP_ADVANCED_GESTURES_D: &[(&str, &str)] = &[
    ("D", GESTURE_DAMAGE),
    ("z", GESTURE_FREEZE),
    ("l", GESTURE_RECALL),
    ("o", GESTURE_AUDITION),
    ("e", GESTURE_RESAMPLE),
];

fn render_gesture_items(items: &[(&str, &str)], separator: &str) -> String {
    items
        .iter()
        .map(|(key, label)| format!("{key}{separator}{label}"))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn queued_status_message(label: &str, boundary: &str) -> String {
    format!("queue {label} on {boundary}")
}

#[derive(Clone, Debug)]
pub struct JamShellState {
    pub app: JamAppState,
    pub launch_mode: ShellLaunchMode,
    pub active_screen: ShellScreen,
    pub jam_mode: JamViewMode,
    pub recovery_surface: Option<SessionRecoverySurface>,
    pub first_run_onramp: bool,
    pub show_help: bool,
    pub status_message: String,
}
