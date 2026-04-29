use crate::test_support::{scene_energy_for_label, scene_label_hint};
use std::{f32::consts::PI, fs, io, path::Path, path::PathBuf};

use serde::Deserialize;
use tempfile::tempdir;

use riotbox_audio::{
    mc202::{
        Mc202ContourHint, Mc202HookResponse, Mc202PhraseShape, Mc202RenderMode, Mc202RenderRouting,
        Mc202RenderState, render_mc202_buffer,
    },
    runtime::{
        AudioOutputInfo, AudioRuntimeHealth, AudioRuntimeLifecycle, render_mc202_offline,
        render_tr909_offline, render_w30_preview_offline, render_w30_resample_tap_offline,
        signal_delta_metrics, signal_metrics,
    },
    source_audio::SourceAudioCache,
    tr909::{
        Tr909PatternAdoption, Tr909PhraseVariation, Tr909RenderMode, Tr909RenderRouting,
        Tr909SourceSupportContext, Tr909SourceSupportProfile, Tr909TakeoverRenderProfile,
    },
    w30::{
        W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN, W30_PREVIEW_SAMPLE_WINDOW_LEN, W30PreviewRenderMode,
        W30PreviewRenderRouting, W30PreviewSourceProfile, W30ResampleTapMode,
        W30ResampleTapRouting, W30ResampleTapSourceProfile,
    },
};
use riotbox_core::{
    action::{
        Action, ActionCommand, ActionDraft, ActionParams, ActionResult, ActionStatus, ActionTarget,
        ActorType, CommitBoundary, GhostMode, Quantization, TargetScope, UndoPolicy,
    },
    ghost::{
        GhostSuggestedAction, GhostSuggestionBlocker, GhostSuggestionBlockerKind,
        GhostSuggestionConfidence, GhostSuggestionSafety, GhostWatchSuggestion, GhostWatchTool,
    },
    ids::{ActionId, AssetId, BankId, CaptureId, PadId, SceneId, SectionId, SnapshotId, SourceId},
    persistence::{
        load_session_json, load_source_graph_json, save_session_json, save_source_graph_json,
    },
    session::{
        CaptureRef, CaptureSourceWindow, CaptureTarget, CaptureType, GhostBudgetState, GhostState,
        GhostSuggestionRecord, GraphStorageMode, Mc202PhraseVariantState,
        SceneMovementDirectionState, SceneMovementLaneIntentState, SessionFile, Snapshot,
        SourceGraphRef, SourceRef, Tr909ReinforcementModeState, Tr909TakeoverProfileState,
        W30PreviewModeState,
    },
    source_graph::{
        AnalysisSummary, AnalysisWarning, Asset, AssetType, Candidate, CandidateType,
        DecodeProfile, EnergyClass, GraphProvenance, QualityClass, Relationship, RelationshipType,
        Section, SectionLabelHint, SourceDescriptor, SourceGraph, SourceGraphVersion,
    },
    transport::TransportClockState,
    view::jam::{
        CaptureTargetKindView, SceneJumpAvailabilityView, SceneTransitionDirectionView,
        SceneTransitionKindView, SceneTransitionLaneIntentView, SceneTransitionPolicyView,
        W30PendingAuditionKind,
    },
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
    GeneratePressure,
    GenerateInstigator,
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
