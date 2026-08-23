use riotbox_app::jam_app::JamAppState;
use riotbox_audio::runtime::{
    RuntimeMixRenderOutput, RuntimeMixRenderPlan, SourceMonitorAudioRoute,
};
use riotbox_core::{
    action::{ActionCommand, CommitBoundary},
    ids::SourceId,
    live_performance_policy::LivePerformancePolicy,
    style::PerformancePresetId,
};

pub const SAMPLE_RATE: u32 = 48_000;
pub const CHANNEL_COUNT: u16 = 2;
pub const CALLBACK_FRAME_COUNT: usize = 128;
pub const MONITOR_REVIEW_BARS: usize = 4;
pub const MIN_MIX_RMS: f32 = 0.01;
pub const MIN_MONITOR_DELTA_RMS: f32 = 0.005;
pub const MIN_ISOLATED_TR909_REGRESSION_RMS: f32 = 0.005;
pub const MAX_SOURCE_MONITOR_SILENCE_RATIO: f64 = 0.05;
pub const MAX_EXACT_MIX_LIMITED_SAMPLE_COUNT: usize = 0;
pub const TONAL_PITCH_DIVE_ACTIVE_BEATS: u32 = 12;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExactLiveReviewMode {
    Standard,
    Tonal,
    Sparse,
}

#[derive(Clone)]
pub struct MonitorProof {
    pub case_id: &'static str,
    pub artifact_path: &'static str,
    pub expected_route: SourceMonitorAudioRoute,
    pub action_id: Option<u64>,
    pub plan: RuntimeMixRenderPlan,
}

#[derive(Clone)]
pub struct RenderStage {
    pub case_id: &'static str,
    pub artifact_path: &'static str,
    pub duration_beats: u32,
    pub key: Option<&'static str>,
    pub command: Option<ActionCommand>,
    pub boundary: Option<CommitBoundary>,
    pub action_id: Option<u64>,
    pub scene_id: String,
    pub source_anchor_seconds: Option<f64>,
    pub plan: Box<RuntimeMixRenderPlan>,
}

#[derive(Clone)]
pub struct GestureTransition {
    pub case_id: &'static str,
    pub key: &'static str,
    pub command: ActionCommand,
    pub boundary: CommitBoundary,
    pub action_id: u64,
    pub companion_actions: Vec<GestureCompanionAction>,
    pub commit_boundary: riotbox_core::transport::CommitBoundaryState,
    pub prefix: Vec<(RuntimeMixRenderPlan, u32)>,
    pub before: Box<RuntimeMixRenderPlan>,
    pub after: Box<RuntimeMixRenderPlan>,
}

#[derive(Clone)]
pub struct GestureCompanionAction {
    pub command: ActionCommand,
    pub action_id: u64,
}

pub struct PreparedLivePath {
    pub state: JamAppState,
    pub source_timing: ConfirmedSourceTiming,
    pub live_policy: LivePerformancePolicy,
    pub preset_id: PerformancePresetId,
    pub preset_action_id: u64,
    pub alpha_arc_stages: Vec<RenderStage>,
    pub alpha_arc_proof: Option<AlphaArcProof>,
    pub restart_recall_plan: Option<Box<RuntimeMixRenderPlan>>,
    pub restart_recall_proof: Option<RestartRecallProof>,
    pub capture_journey_proof: CaptureJourneyProof,
    pub monitor_proofs: Vec<MonitorProof>,
    pub stages: Vec<RenderStage>,
    pub transitions: Vec<GestureTransition>,
    pub scene_transition_proof: Option<SceneTransitionProof>,
    pub monitor_action_ids: [u64; 4],
    pub legacy_riotbox_action_id: u64,
    pub normal_plan: RuntimeMixRenderPlan,
    pub damaged_plan: RuntimeMixRenderPlan,
    pub sparse_journey: Option<Box<SparseJourney>>,
    pub tonal_journey: Option<Box<TonalJourney>>,
}

pub struct SparseJourney {
    pub held_plan: Box<RuntimeMixRenderPlan>,
    pub damage_plan: Box<RuntimeMixRenderPlan>,
    pub reentry_plan: Box<RuntimeMixRenderPlan>,
    pub restart_recall_plan: Box<RuntimeMixRenderPlan>,
    pub proof: SparseJourneyProof,
    pub restart_recall_proof: RestartRecallProof,
}

pub struct SparseJourneyProof {
    pub damage_action_id: u64,
    pub bypass_action_id: u64,
    pub damage_intensity: f32,
    pub bypass_intensity: f32,
    pub damage_gate_step_fraction: f32,
    pub reentry_gate_step_fraction: f32,
}

pub struct TonalJourney {
    pub held_plan: Box<RuntimeMixRenderPlan>,
    pub contrast_plan: Box<RuntimeMixRenderPlan>,
    pub reentry_plan: Box<RuntimeMixRenderPlan>,
    pub restart_recall_plan: Box<RuntimeMixRenderPlan>,
    pub proof: TonalJourneyProof,
    pub restart_recall_proof: RestartRecallProof,
}

pub struct TonalJourneyProof {
    pub contrast_action_id: u64,
    pub reentry_action_id: u64,
    pub ordinary_reentry_cleared_articulation: bool,
}

pub struct CaptureJourneyProof {
    pub capture_action_id: u64,
    pub raw_audition_action_id: u64,
    pub promotion_action_id: u64,
}

pub struct AlphaArcProof {
    pub hook_action_id: u64,
    pub pressure_action_id: u64,
    pub destructive_fill_action_id: u64,
    pub role_swap_action_id: u64,
    pub return_action_id: u64,
    pub return_damage_action_id: u64,
    pub original_scene: String,
    pub contrast_scene: String,
    pub returned_scene: String,
}

pub struct RestartRecallProof {
    pub preset_survived_restart: bool,
    pub capture_id: String,
    pub recall_action_id: u64,
    pub trigger_action_id: u64,
}

#[derive(Clone, Debug)]
pub struct ConfirmedSourceTiming {
    pub source_id: SourceId,
    pub hypothesis_id: String,
    pub cli_bpm_hint: f32,
    pub bpm: f32,
    pub beats_per_bar: u64,
    pub primary_bar_anchor_beat_index: u32,
    pub primary_bar_anchor_beat_cursor: u64,
    pub primary_bar_anchor_bar_index: u64,
}

impl ConfirmedSourceTiming {
    pub fn bar_start_beat_cursor(&self, bar_index: u64) -> Option<u64> {
        if bar_index >= self.primary_bar_anchor_bar_index {
            Some(self.primary_bar_anchor_beat_cursor.saturating_add(
                (bar_index - self.primary_bar_anchor_bar_index).saturating_mul(self.beats_per_bar),
            ))
        } else {
            self.primary_bar_anchor_beat_cursor.checked_sub(
                (self.primary_bar_anchor_bar_index - bar_index).saturating_mul(self.beats_per_bar),
            )
        }
    }
}

pub struct SceneTransitionProof {
    pub launch_action_id: u64,
    pub restore_action_id: u64,
    pub return_damage_action_id: u64,
    pub pre_jump_scene: String,
    pub launched_scene: String,
    pub restored_scene: String,
    pub pre_jump_render_anchor_seconds: Option<f64>,
    pub expected_launch_anchor_seconds: Option<f64>,
    pub expected_restore_anchor_seconds: Option<f64>,
    pub launched_anchor_seconds: Option<f64>,
    pub restored_anchor_seconds: Option<f64>,
    pub mc202_plan_source_section: Option<String>,
    pub launched_source_section: Option<String>,
    pub launch_mc202_stayed_out_for_section_mismatch: bool,
    pub restore_audio_projection_matches_pre_jump: bool,
    pub restore_only_lane_projection_matches_pre_jump: bool,
    pub changed_return_w30_differs_from_restore_only: bool,
    pub changed_return_non_w30_projection_matches_restore_only: bool,
}

pub struct RenderedGestureTransition {
    pub before: RuntimeMixRenderOutput,
    pub after: RuntimeMixRenderOutput,
}

pub struct RenderedLivePath {
    pub monitor_outputs: Vec<RuntimeMixRenderOutput>,
    pub stage_outputs: Vec<RuntimeMixRenderOutput>,
    pub alpha_arc_outputs: Vec<RuntimeMixRenderOutput>,
    pub alpha_source_reference: RuntimeMixRenderOutput,
    pub restart_recall_output: RuntimeMixRenderOutput,
    pub transition_outputs: Vec<RenderedGestureTransition>,
    pub normal: RuntimeMixRenderOutput,
    pub damaged: RuntimeMixRenderOutput,
    pub w30: RuntimeMixRenderOutput,
    pub tr909: RuntimeMixRenderOutput,
    pub mc202_selected_role: RuntimeMixRenderOutput,
    pub direct_mc202: Vec<f32>,
}
