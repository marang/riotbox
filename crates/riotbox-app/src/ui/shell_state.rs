use crate::jam_app::JamAppState;
use crate::jam_app::SessionRecoverySurface;
use crate::ui::first_run_capture::first_run_onramp_stage;
use crate::ui::gestures::GESTURE_ANSWER;
use crate::ui::gestures::GESTURE_AUDITION;
use crate::ui::gestures::GESTURE_BANK;
use crate::ui::gestures::GESTURE_BROWSE;
use crate::ui::gestures::GESTURE_CAPTURE;
use crate::ui::gestures::GESTURE_DAMAGE;
use crate::ui::gestures::GESTURE_FILL;
use crate::ui::gestures::GESTURE_FILTER_SLAM;
use crate::ui::gestures::GESTURE_FOLLOW;
use crate::ui::gestures::GESTURE_FREEZE;
use crate::ui::gestures::GESTURE_HIT;
use crate::ui::gestures::GESTURE_INSTIGATE;
use crate::ui::gestures::GESTURE_LOCK;
use crate::ui::gestures::GESTURE_MUTATE;
use crate::ui::gestures::GESTURE_NEXT_PAD;
use crate::ui::gestures::GESTURE_PHRASE;
use crate::ui::gestures::GESTURE_PITCH_DIVE;
use crate::ui::gestures::GESTURE_PRESSURE;
use crate::ui::gestures::GESTURE_PROMOTE;
use crate::ui::gestures::GESTURE_PUSH;
use crate::ui::gestures::GESTURE_RECALL;
use crate::ui::gestures::GESTURE_RELEASE;
use crate::ui::gestures::GESTURE_RESAMPLE;
use crate::ui::gestures::GESTURE_RESTORE;
use crate::ui::gestures::GESTURE_SCENE_JUMP;
use crate::ui::gestures::GESTURE_SLAM;
use crate::ui::gestures::GESTURE_TAKEOVER;
use crate::ui::gestures::GESTURE_TURNAROUND;
use crate::ui::gestures::GESTURE_VOICE;
use crate::ui::gestures::queued_status_message;
use crossterm::event::KeyCode;
use riotbox_core::action::SourceMonitorMode;
use riotbox_core::style::PerformancePresetId;

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
    QueuePerformancePreset(PerformancePresetId),
    QueueSourceMonitorMode(SourceMonitorMode),
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
    QueueW30HookTurnaround,
    QueueW30PitchDive,
    QueueW30FilterSlam,
    QueueW30LoopFreeze,
    QueueW30LiveRecall,
    QueueW30Audition,
    QueueW30Resample,
    QueueProductMixExport,
    ConfirmSourceTimingGrid,
    RevertSourceTimingGrid,
    NavigateSourceMapPreviousBar,
    NavigateSourceMapNextBar,
    NavigateSourceMapPreviousPhrase,
    NavigateSourceMapNextPhrase,
    PreviousCaptureLength,
    NextCaptureLength,
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

impl JamShellState {
    #[must_use]
    pub fn new(app: JamAppState, launch_mode: ShellLaunchMode) -> Self {
        let first_run_onramp = matches!(launch_mode, ShellLaunchMode::Ingest)
            && app.session.action_log.actions.is_empty()
            && app.session.captures.is_empty();
        let status_message = match launch_mode {
            ShellLaunchMode::Load => "loaded session from disk".into(),
            ShellLaunchMode::Ingest => "ingested source into Jam shell".into(),
        };

        Self {
            app,
            launch_mode,
            active_screen: ShellScreen::Jam,
            jam_mode: JamViewMode::Perform,
            recovery_surface: None,
            first_run_onramp,
            show_help: false,
            status_message,
        }
    }

    pub fn handle_key_code(&mut self, code: KeyCode) -> ShellKeyOutcome {
        if self.show_help {
            return match code {
                KeyCode::Char('q') => ShellKeyOutcome::Quit,
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('h') => {
                    self.show_help = false;
                    self.status_message = "help overlay closed".into();
                    ShellKeyOutcome::Continue
                }
                _ => {
                    self.status_message = "help open | Esc, ? or h closes it".into();
                    ShellKeyOutcome::Continue
                }
            };
        }

        match code {
            KeyCode::Esc | KeyCode::Char('q') => ShellKeyOutcome::Quit,
            KeyCode::Tab | KeyCode::BackTab => {
                self.active_screen = self.active_screen.next();
                self.status_message = format!("switched to {} screen", self.active_screen.label());
                ShellKeyOutcome::Continue
            }
            KeyCode::Char('1') => {
                self.active_screen = ShellScreen::Jam;
                self.status_message = "switched to jam screen".into();
                ShellKeyOutcome::Continue
            }
            KeyCode::Char('2') => {
                self.active_screen = ShellScreen::Log;
                self.status_message = "switched to log screen".into();
                ShellKeyOutcome::Continue
            }
            KeyCode::Char('3') => {
                self.active_screen = ShellScreen::Source;
                self.status_message = "switched to source screen".into();
                ShellKeyOutcome::Continue
            }
            KeyCode::Char('4') => {
                self.active_screen = ShellScreen::Capture;
                self.status_message = "switched to capture screen".into();
                ShellKeyOutcome::Continue
            }
            KeyCode::Char(' ') => {
                self.status_message = "transport toggle requested".into();
                ShellKeyOutcome::ToggleTransport
            }
            KeyCode::Char('M') => {
                let mode = self.app.session.runtime_state.source_monitor.mode.next();
                self.status_message = format!("queue monitor {mode} for immediate commit");
                ShellKeyOutcome::QueueSourceMonitorMode(mode)
            }
            KeyCode::Char('F') => {
                let preset_id = riotbox_core::style::PerformancePresetId::FeralBreakAlphaV2;
                self.status_message = format!("queue {} for immediate commit", preset_id.label());
                ShellKeyOutcome::QueuePerformancePreset(preset_id)
            }
            KeyCode::Char('?') | KeyCode::Char('h') => {
                self.show_help = true;
                self.status_message = "help overlay opened".into();
                ShellKeyOutcome::Continue
            }
            KeyCode::Char('i') => {
                if self.active_screen != ShellScreen::Jam {
                    self.status_message = "open Jam first if you want to use inspect".into();
                } else if first_run_onramp_stage(self).is_some() {
                    self.status_message =
                        "finish the first guided move before opening inspect".into();
                } else {
                    self.jam_mode = match self.jam_mode {
                        JamViewMode::Perform => JamViewMode::Inspect,
                        JamViewMode::Inspect => JamViewMode::Perform,
                    };
                    self.status_message = match self.jam_mode {
                        JamViewMode::Perform => "returned Jam to perform mode".into(),
                        JamViewMode::Inspect => {
                            "opened Jam inspect | press i to return to perform".into()
                        }
                    };
                }
                ShellKeyOutcome::Continue
            }
            KeyCode::Char('r') => {
                self.status_message = format!("{} requested", self.launch_mode.refresh_verb());
                ShellKeyOutcome::RequestRefresh
            }
            KeyCode::Char('m') => {
                self.status_message = queued_status_message(GESTURE_MUTATE, "next bar");
                ShellKeyOutcome::QueueSceneMutation
            }
            KeyCode::Char('y') => {
                self.status_message = queued_status_message(GESTURE_SCENE_JUMP, "next bar");
                ShellKeyOutcome::QueueSceneSelect
            }
            KeyCode::Char('Y') => {
                self.status_message = queued_status_message(GESTURE_RESTORE, "next bar");
                ShellKeyOutcome::QueueSceneRestore
            }
            KeyCode::Char('b') => {
                self.status_message = queued_status_message(GESTURE_VOICE, "next phrase");
                ShellKeyOutcome::QueueMc202RoleToggle
            }
            KeyCode::Char('g') => {
                self.status_message = queued_status_message(GESTURE_FOLLOW, "next phrase");
                ShellKeyOutcome::QueueMc202GenerateFollower
            }
            KeyCode::Char('a') => {
                self.status_message = queued_status_message(GESTURE_ANSWER, "next phrase");
                ShellKeyOutcome::QueueMc202GenerateAnswer
            }
            KeyCode::Char('P') => {
                self.status_message = queued_status_message(GESTURE_PRESSURE, "next phrase");
                ShellKeyOutcome::QueueMc202GeneratePressure
            }
            KeyCode::Char('I') => {
                self.status_message = queued_status_message(GESTURE_INSTIGATE, "next phrase");
                ShellKeyOutcome::QueueMc202GenerateInstigator
            }
            KeyCode::Char('G') => {
                self.status_message = queued_status_message(GESTURE_PHRASE, "next phrase");
                ShellKeyOutcome::QueueMc202MutatePhrase
            }
            KeyCode::Char('f') => {
                self.status_message = queued_status_message(GESTURE_FILL, "next bar");
                ShellKeyOutcome::QueueTr909Fill
            }
            KeyCode::Char('d') => {
                self.status_message = queued_status_message(GESTURE_PUSH, "next phrase");
                ShellKeyOutcome::QueueTr909Reinforce
            }
            KeyCode::Char('s') => {
                self.status_message = queued_status_message(GESTURE_SLAM, "next beat");
                ShellKeyOutcome::QueueTr909Slam
            }
            KeyCode::Char('t') => {
                self.status_message = queued_status_message(GESTURE_TAKEOVER, "next phrase");
                ShellKeyOutcome::QueueTr909Takeover
            }
            KeyCode::Char('k') => {
                self.status_message = queued_status_message(GESTURE_LOCK, "next phrase");
                ShellKeyOutcome::QueueTr909SceneLock
            }
            KeyCode::Char('x') => {
                self.status_message = queued_status_message(GESTURE_RELEASE, "next phrase");
                ShellKeyOutcome::QueueTr909Release
            }
            KeyCode::Char('c') => {
                self.status_message = queued_status_message(GESTURE_CAPTURE, "next phrase");
                ShellKeyOutcome::QueueCaptureBar
            }
            KeyCode::Char('p') => {
                self.status_message = format!("queue {GESTURE_PROMOTE} for latest capture");
                ShellKeyOutcome::PromoteLastCapture
            }
            KeyCode::Char('w') => {
                self.status_message = queued_status_message(GESTURE_HIT, "next beat");
                ShellKeyOutcome::QueueW30TriggerPad
            }
            KeyCode::Char('n') => {
                self.status_message = queued_status_message(GESTURE_NEXT_PAD, "next beat");
                ShellKeyOutcome::QueueW30StepFocus
            }
            KeyCode::Char('B') => {
                self.status_message = queued_status_message(GESTURE_BANK, "next bar");
                ShellKeyOutcome::QueueW30SwapBank
            }
            KeyCode::Char('j') => {
                self.status_message = queued_status_message(GESTURE_BROWSE, "next beat");
                ShellKeyOutcome::QueueW30BrowseSlicePool
            }
            KeyCode::Char('D') => {
                self.status_message = queued_status_message(GESTURE_DAMAGE, "next bar");
                ShellKeyOutcome::QueueW30ApplyDamageProfile
            }
            KeyCode::Char('H') => {
                self.status_message = queued_status_message(GESTURE_TURNAROUND, "next bar");
                ShellKeyOutcome::QueueW30HookTurnaround
            }
            KeyCode::Char('V') => {
                self.status_message = queued_status_message(GESTURE_PITCH_DIVE, "next bar");
                ShellKeyOutcome::QueueW30PitchDive
            }
            KeyCode::Char('L') => {
                self.status_message = queued_status_message(GESTURE_FILTER_SLAM, "next bar");
                ShellKeyOutcome::QueueW30FilterSlam
            }
            KeyCode::Char('z') => {
                self.status_message = queued_status_message(GESTURE_FREEZE, "next phrase");
                ShellKeyOutcome::QueueW30LoopFreeze
            }
            KeyCode::Char('l') => {
                self.status_message = queued_status_message(GESTURE_RECALL, "next bar");
                ShellKeyOutcome::QueueW30LiveRecall
            }
            KeyCode::Char('o') => {
                self.status_message = queued_status_message(GESTURE_AUDITION, "next bar");
                ShellKeyOutcome::QueueW30Audition
            }
            KeyCode::Char('e') => {
                self.status_message = queued_status_message(GESTURE_RESAMPLE, "next phrase");
                ShellKeyOutcome::QueueW30Resample
            }
            KeyCode::Char('E') => {
                self.status_message = "export full_grid_mix requested".into();
                ShellKeyOutcome::QueueProductMixExport
            }
            KeyCode::Char('C') => {
                self.status_message = "confirm source timing grid requested".into();
                ShellKeyOutcome::ConfirmSourceTimingGrid
            }
            KeyCode::Char('R') => {
                self.status_message = "revert source timing grid confirmation requested".into();
                ShellKeyOutcome::RevertSourceTimingGrid
            }
            KeyCode::Left => {
                self.status_message = "source map previous bar requested".into();
                ShellKeyOutcome::NavigateSourceMapPreviousBar
            }
            KeyCode::Right => {
                self.status_message = "source map next bar requested".into();
                ShellKeyOutcome::NavigateSourceMapNextBar
            }
            KeyCode::Up => {
                self.status_message = "source map previous phrase requested".into();
                ShellKeyOutcome::NavigateSourceMapPreviousPhrase
            }
            KeyCode::Down => {
                self.status_message = "source map next phrase requested".into();
                ShellKeyOutcome::NavigateSourceMapNextPhrase
            }
            KeyCode::Char('-') => {
                self.status_message = "previous capture length requested".into();
                ShellKeyOutcome::PreviousCaptureLength
            }
            KeyCode::Char('=') => {
                self.status_message = "next capture length requested".into();
                ShellKeyOutcome::NextCaptureLength
            }
            KeyCode::Char('v') => {
                self.status_message = "toggle pin for latest capture".into();
                ShellKeyOutcome::TogglePinLatestCapture
            }
            KeyCode::Char('[') => {
                self.status_message = "lower drum bus level".into();
                ShellKeyOutcome::LowerDrumBusLevel
            }
            KeyCode::Char(']') => {
                self.status_message = "raise drum bus level".into();
                ShellKeyOutcome::RaiseDrumBusLevel
            }
            KeyCode::Char('<') => {
                self.status_message = "lower MC-202 touch".into();
                ShellKeyOutcome::LowerMc202Touch
            }
            KeyCode::Char('>') => {
                self.status_message = "raise MC-202 touch".into();
                ShellKeyOutcome::RaiseMc202Touch
            }
            KeyCode::Enter => {
                self.status_message = "accept ghost suggestion requested".into();
                ShellKeyOutcome::AcceptCurrentGhostSuggestion
            }
            KeyCode::Char('N') => {
                self.status_message = "reject ghost suggestion requested".into();
                ShellKeyOutcome::RejectCurrentGhostSuggestion
            }
            KeyCode::Char('u') => {
                self.status_message = "undo most recent action requested".into();
                ShellKeyOutcome::UndoLast
            }
            _ => ShellKeyOutcome::Continue,
        }
    }

    pub fn replace_app_state(&mut self, app: JamAppState) {
        self.first_run_onramp = matches!(self.launch_mode, ShellLaunchMode::Ingest)
            && app.session.action_log.actions.is_empty()
            && app.session.captures.is_empty();
        self.app = app;
        self.status_message = match self.launch_mode {
            ShellLaunchMode::Load => "reloaded session from disk".into(),
            ShellLaunchMode::Ingest => "re-ingested source into Jam shell".into(),
        };
    }

    pub fn set_error_status(&mut self, message: impl Into<String>) {
        self.status_message = message.into();
    }

    pub fn set_recovery_surface(&mut self, surface: SessionRecoverySurface) {
        self.recovery_surface = Some(surface);
    }

    pub fn clear_recovery_surface(&mut self) {
        self.recovery_surface = None;
    }
}
