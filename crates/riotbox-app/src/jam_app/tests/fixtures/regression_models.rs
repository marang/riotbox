use riotbox_audio::runtime::AudioOutputInfo;
use riotbox_audio::runtime::AudioRuntimeHealth;
use riotbox_audio::runtime::AudioRuntimeLifecycle;
use riotbox_core::TimestampMs;
use riotbox_core::action::ActionCommand;
use riotbox_core::action::CommitBoundary;
use riotbox_core::ids::SceneId;
use riotbox_core::session::Tr909ReinforcementModeState;
use riotbox_core::session::Tr909TakeoverProfileState;
use riotbox_core::session::W30PreviewModeState;
use riotbox_core::transport::CommitBoundaryState;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(in crate::jam_app::tests) struct RenderProjectionFixture {
    pub(in crate::jam_app::tests) name: String,
    pub(in crate::jam_app::tests) transport_position_beats: f64,
    #[serde(default)]
    pub(in crate::jam_app::tests) scene_context: Option<String>,
    pub(in crate::jam_app::tests) reinforcement_mode: Tr909ReinforcementModeState,
    pub(in crate::jam_app::tests) takeover_enabled: bool,
    pub(in crate::jam_app::tests) takeover_profile: Option<Tr909TakeoverProfileState>,
    pub(in crate::jam_app::tests) pattern_ref: Option<String>,
    pub(in crate::jam_app::tests) expected_mode: String,
    pub(in crate::jam_app::tests) expected_routing: String,
    pub(in crate::jam_app::tests) expected_pattern_adoption: Option<String>,
    pub(in crate::jam_app::tests) expected_phrase_variation: Option<String>,
    pub(in crate::jam_app::tests) expected_source_support_profile: Option<String>,
    pub(in crate::jam_app::tests) expected_source_support_context: Option<String>,
    pub(in crate::jam_app::tests) expected_support_accent: String,
    pub(in crate::jam_app::tests) expected_takeover_profile: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(in crate::jam_app::tests) struct Mc202RegressionFixture {
    pub(in crate::jam_app::tests) name: String,
    pub(in crate::jam_app::tests) initial_role: String,
    pub(in crate::jam_app::tests) action: Mc202RegressionAction,
    pub(in crate::jam_app::tests) requested_at: TimestampMs,
    pub(in crate::jam_app::tests) committed_at: TimestampMs,
    pub(in crate::jam_app::tests) boundary: Mc202RegressionBoundary,
    pub(in crate::jam_app::tests) expected: Mc202RegressionExpected,
}

#[derive(Debug, Deserialize)]
pub(in crate::jam_app::tests) struct SceneRegressionFixture {
    pub(in crate::jam_app::tests) name: String,
    pub(in crate::jam_app::tests) section_labels: Vec<String>,
    pub(in crate::jam_app::tests) action: SceneRegressionAction,
    #[serde(default)]
    pub(in crate::jam_app::tests) initial_active_scene: Option<String>,
    #[serde(default)]
    pub(in crate::jam_app::tests) initial_current_scene: Option<String>,
    #[serde(default)]
    pub(in crate::jam_app::tests) initial_restore_scene: Option<String>,
    #[serde(default)]
    pub(in crate::jam_app::tests) tr909_reinforcement_mode: Option<Tr909ReinforcementModeState>,
    #[serde(default)]
    pub(in crate::jam_app::tests) tr909_pattern_ref: Option<String>,
    #[serde(default)]
    pub(in crate::jam_app::tests) requested_at: Option<TimestampMs>,
    #[serde(default)]
    pub(in crate::jam_app::tests) committed_at: Option<TimestampMs>,
    #[serde(default)]
    pub(in crate::jam_app::tests) boundary: Option<SceneRegressionBoundary>,
    pub(in crate::jam_app::tests) expected: SceneRegressionExpected,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::jam_app::tests) enum SceneRegressionAction {
    ProjectCandidates,
    SelectNextScene,
    RestoreScene,
}

#[derive(Debug, Deserialize)]
pub(in crate::jam_app::tests) struct SceneRegressionBoundary {
    pub(in crate::jam_app::tests) kind: SceneRegressionBoundaryKind,
    pub(in crate::jam_app::tests) beat_index: u64,
    pub(in crate::jam_app::tests) bar_index: u64,
    pub(in crate::jam_app::tests) phrase_index: u64,
    pub(in crate::jam_app::tests) scene_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::jam_app::tests) enum SceneRegressionBoundaryKind {
    Immediate,
    Beat,
    HalfBar,
    Bar,
    Phrase,
    Scene,
}

#[derive(Debug, Deserialize)]
pub(in crate::jam_app::tests) struct SceneRegressionExpected {
    pub(in crate::jam_app::tests) scenes: Vec<String>,
    pub(in crate::jam_app::tests) active_scene: String,
    pub(in crate::jam_app::tests) current_scene: String,
    pub(in crate::jam_app::tests) active_scene_energy: String,
    #[serde(default)]
    pub(in crate::jam_app::tests) restore_scene: Option<String>,
    #[serde(default)]
    pub(in crate::jam_app::tests) restore_scene_energy: Option<String>,
    #[serde(default)]
    pub(in crate::jam_app::tests) result_summary: Option<String>,
    #[serde(default)]
    pub(in crate::jam_app::tests) tr909_render_profile: Option<String>,
    #[serde(default)]
    pub(in crate::jam_app::tests) tr909_render_support_context: Option<String>,
    #[serde(default)]
    pub(in crate::jam_app::tests) tr909_render_support_accent: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::jam_app::tests) enum Mc202RegressionAction {
    SetRole,
    GenerateFollower,
    GenerateAnswer,
    GeneratePressure,
    GenerateInstigator,
}

#[derive(Debug, Deserialize)]
pub(in crate::jam_app::tests) struct Mc202RegressionBoundary {
    pub(in crate::jam_app::tests) kind: Mc202RegressionBoundaryKind,
    pub(in crate::jam_app::tests) beat_index: u64,
    pub(in crate::jam_app::tests) bar_index: u64,
    pub(in crate::jam_app::tests) phrase_index: u64,
    pub(in crate::jam_app::tests) scene_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::jam_app::tests) enum Mc202RegressionBoundaryKind {
    Immediate,
    Beat,
    HalfBar,
    Bar,
    Phrase,
    Scene,
}

#[derive(Debug, Deserialize)]
pub(in crate::jam_app::tests) struct Mc202RegressionExpected {
    pub(in crate::jam_app::tests) role: String,
    pub(in crate::jam_app::tests) phrase_ref: String,
    pub(in crate::jam_app::tests) touch: f32,
    pub(in crate::jam_app::tests) result_summary: String,
}

#[derive(Debug, Deserialize)]
pub(in crate::jam_app::tests) struct W30RegressionFixture {
    pub(in crate::jam_app::tests) name: String,
    pub(in crate::jam_app::tests) action: W30RegressionAction,
    pub(in crate::jam_app::tests) capture_bank: String,
    pub(in crate::jam_app::tests) capture_pad: String,
    pub(in crate::jam_app::tests) capture_pinned: bool,
    #[serde(default)]
    pub(in crate::jam_app::tests) source_window: Option<W30RegressionSourceWindow>,
    #[serde(default = "default_true")]
    pub(in crate::jam_app::tests) capture_assigned: bool,
    #[serde(default)]
    pub(in crate::jam_app::tests) extra_captures: Vec<W30RegressionCapture>,
    #[serde(default)]
    pub(in crate::jam_app::tests) initial_active_bank: Option<String>,
    #[serde(default)]
    pub(in crate::jam_app::tests) initial_focused_pad: Option<String>,
    #[serde(default)]
    pub(in crate::jam_app::tests) initial_last_capture: Option<String>,
    #[serde(default)]
    pub(in crate::jam_app::tests) initial_preview_mode: Option<String>,
    #[serde(default)]
    pub(in crate::jam_app::tests) initial_w30_grit: Option<f32>,
    pub(in crate::jam_app::tests) requested_at: TimestampMs,
    pub(in crate::jam_app::tests) committed_at: TimestampMs,
    pub(in crate::jam_app::tests) boundary: W30RegressionBoundary,
    pub(in crate::jam_app::tests) expected: W30RegressionExpected,
}

#[derive(Debug, Deserialize)]
pub(in crate::jam_app::tests) struct W30RegressionSourceWindow {
    pub(in crate::jam_app::tests) source_id: String,
    pub(in crate::jam_app::tests) start_seconds: f32,
    pub(in crate::jam_app::tests) end_seconds: f32,
    pub(in crate::jam_app::tests) start_frame: u64,
    pub(in crate::jam_app::tests) end_frame: u64,
}

#[derive(Debug, Deserialize)]
pub(in crate::jam_app::tests) struct W30RegressionCapture {
    pub(in crate::jam_app::tests) capture_id: String,
    pub(in crate::jam_app::tests) bank: String,
    pub(in crate::jam_app::tests) pad: String,
    pub(in crate::jam_app::tests) pinned: bool,
    #[serde(default)]
    pub(in crate::jam_app::tests) notes: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::jam_app::tests) enum W30RegressionAction {
    LiveRecall,
    RawCaptureAudition,
    PromotedAudition,
    TriggerPad,
    SwapBank,
    ApplyDamageProfile,
    LoopFreeze,
    BrowseSlicePool,
}

pub(in crate::jam_app::tests) fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub(in crate::jam_app::tests) struct W30RegressionBoundary {
    pub(in crate::jam_app::tests) kind: W30RegressionBoundaryKind,
    pub(in crate::jam_app::tests) beat_index: u64,
    pub(in crate::jam_app::tests) bar_index: u64,
    pub(in crate::jam_app::tests) phrase_index: u64,
    pub(in crate::jam_app::tests) scene_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::jam_app::tests) enum W30RegressionBoundaryKind {
    Immediate,
    Beat,
    HalfBar,
    Bar,
    Phrase,
    Scene,
}

#[derive(Debug, Deserialize)]
pub(in crate::jam_app::tests) struct W30RegressionExpected {
    pub(in crate::jam_app::tests) active_bank: String,
    pub(in crate::jam_app::tests) focused_pad: String,
    pub(in crate::jam_app::tests) last_capture: String,
    pub(in crate::jam_app::tests) w30_grit: f32,
    #[serde(default)]
    pub(in crate::jam_app::tests) preview_mode: Option<String>,
    #[serde(default)]
    pub(in crate::jam_app::tests) preview_routing: Option<String>,
    #[serde(default)]
    pub(in crate::jam_app::tests) preview_profile: Option<String>,
    #[serde(default)]
    pub(in crate::jam_app::tests) preview_capture: Option<String>,
    #[serde(default)]
    pub(in crate::jam_app::tests) preview_music_bus_level: Option<f32>,
    #[serde(default)]
    pub(in crate::jam_app::tests) preview_grit_level: Option<f32>,
    #[serde(default)]
    pub(in crate::jam_app::tests) preview_transport_running: Option<bool>,
    pub(in crate::jam_app::tests) result_summary: String,
}

pub(in crate::jam_app::tests) fn w30_preview_mode_state(value: &str) -> W30PreviewModeState {
    match value {
        "live_recall" => W30PreviewModeState::LiveRecall,
        "raw_capture_audition" => W30PreviewModeState::RawCaptureAudition,
        "promoted_audition" => W30PreviewModeState::PromotedAudition,
        other => panic!("unsupported W-30 preview mode fixture value: {other}"),
    }
}

impl Mc202RegressionBoundary {
    pub(in crate::jam_app::tests) fn into_commit_boundary_state(self) -> CommitBoundaryState {
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
    pub(in crate::jam_app::tests) fn into_commit_boundary_state(self) -> CommitBoundaryState {
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
    pub(in crate::jam_app::tests) fn into_commit_boundary_state(self) -> CommitBoundaryState {
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

pub(in crate::jam_app::tests) fn expected_w30_command(
    action: W30RegressionAction,
) -> ActionCommand {
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

pub(in crate::jam_app::tests) fn sample_audio_health(
    lifecycle: AudioRuntimeLifecycle,
) -> AudioRuntimeHealth {
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
        callback_scratch_overflow_count: 0,
        stream_error_count: u64::from(matches!(lifecycle, AudioRuntimeLifecycle::Faulted)),
        last_stream_error: matches!(lifecycle, AudioRuntimeLifecycle::Faulted)
            .then(|| "stream stalled".into()),
    }
}
