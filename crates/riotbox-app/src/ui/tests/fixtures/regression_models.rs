use crate::test_support::scene_energy_for_label;
use crate::test_support::scene_label_hint;
use crate::ui::shell_state::JamShellState;
use crate::ui::tests::fixtures::shells::sample_shell_state;
use riotbox_core::TimestampMs;
use riotbox_core::ids::SceneId;
use riotbox_core::session::Tr909ReinforcementModeState;
use riotbox_core::source_graph::SourceGraph;
use riotbox_core::transport::CommitBoundaryState;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(in crate::ui::tests) struct Mc202RegressionFixture {
    pub(in crate::ui::tests) name: String,
    pub(in crate::ui::tests) initial_role: String,
    pub(in crate::ui::tests) action: Mc202RegressionAction,
    pub(in crate::ui::tests) requested_at: TimestampMs,
    pub(in crate::ui::tests) committed_at: TimestampMs,
    pub(in crate::ui::tests) boundary: Mc202RegressionBoundary,
    pub(in crate::ui::tests) expected: Mc202RegressionExpected,
}

#[derive(Debug, Deserialize)]
pub(in crate::ui::tests) struct SceneRegressionFixture {
    pub(in crate::ui::tests) name: String,
    pub(in crate::ui::tests) section_labels: Vec<String>,
    pub(in crate::ui::tests) action: SceneRegressionAction,
    #[serde(default)]
    pub(in crate::ui::tests) initial_active_scene: Option<String>,
    #[serde(default)]
    pub(in crate::ui::tests) initial_current_scene: Option<String>,
    #[serde(default)]
    pub(in crate::ui::tests) initial_restore_scene: Option<String>,
    #[serde(default)]
    pub(in crate::ui::tests) tr909_reinforcement_mode: Option<Tr909ReinforcementModeState>,
    #[serde(default)]
    pub(in crate::ui::tests) tr909_pattern_ref: Option<String>,
    pub(in crate::ui::tests) requested_at: Option<TimestampMs>,
    #[serde(default)]
    pub(in crate::ui::tests) committed_at: Option<TimestampMs>,
    #[serde(default)]
    pub(in crate::ui::tests) boundary: Option<SceneRegressionBoundary>,
    pub(in crate::ui::tests) expected: SceneRegressionExpected,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::ui::tests) enum SceneRegressionAction {
    ProjectCandidates,
    SelectNextScene,
    RestoreScene,
}

#[derive(Debug, Deserialize)]
pub(in crate::ui::tests) struct SceneRegressionBoundary {
    pub(in crate::ui::tests) kind: SceneRegressionBoundaryKind,
    pub(in crate::ui::tests) beat_index: u64,
    pub(in crate::ui::tests) bar_index: u64,
    pub(in crate::ui::tests) phrase_index: u64,
    pub(in crate::ui::tests) scene_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::ui::tests) enum SceneRegressionBoundaryKind {
    Immediate,
    Beat,
    HalfBar,
    Bar,
    Phrase,
    Scene,
}

#[derive(Debug, Deserialize)]
pub(in crate::ui::tests) struct SceneRegressionExpected {
    pub(in crate::ui::tests) active_scene: String,
    #[allow(dead_code)]
    pub(in crate::ui::tests) current_scene: String,
    #[allow(dead_code)]
    pub(in crate::ui::tests) scenes: Vec<String>,
    #[serde(default)]
    pub(in crate::ui::tests) result_summary: Option<String>,
    pub(in crate::ui::tests) jam_contains: Vec<String>,
    pub(in crate::ui::tests) log_contains: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::ui::tests) enum Mc202RegressionAction {
    SetRole,
    GenerateFollower,
    GenerateAnswer,
    GeneratePressure,
    GenerateInstigator,
}

#[derive(Debug, Deserialize)]
pub(in crate::ui::tests) struct Mc202RegressionBoundary {
    pub(in crate::ui::tests) kind: Mc202RegressionBoundaryKind,
    pub(in crate::ui::tests) beat_index: u64,
    pub(in crate::ui::tests) bar_index: u64,
    pub(in crate::ui::tests) phrase_index: u64,
    pub(in crate::ui::tests) scene_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::ui::tests) enum Mc202RegressionBoundaryKind {
    Immediate,
    Beat,
    HalfBar,
    Bar,
    Phrase,
    Scene,
}

#[derive(Debug, Deserialize)]
pub(in crate::ui::tests) struct Mc202RegressionExpected {
    pub(in crate::ui::tests) role: String,
    pub(in crate::ui::tests) phrase_ref: String,
    pub(in crate::ui::tests) touch: f32,
    pub(in crate::ui::tests) result_summary: String,
    pub(in crate::ui::tests) jam_contains: Vec<String>,
    pub(in crate::ui::tests) log_contains: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(in crate::ui::tests) struct W30RegressionFixture {
    pub(in crate::ui::tests) name: String,
    pub(in crate::ui::tests) action: W30RegressionAction,
    pub(in crate::ui::tests) capture_bank: String,
    pub(in crate::ui::tests) capture_pad: String,
    pub(in crate::ui::tests) capture_pinned: bool,
    #[serde(default)]
    pub(in crate::ui::tests) source_window: Option<W30RegressionSourceWindow>,
    #[serde(default = "default_true")]
    pub(in crate::ui::tests) capture_assigned: bool,
    #[serde(default)]
    pub(in crate::ui::tests) extra_captures: Vec<W30RegressionCapture>,
    #[serde(default)]
    pub(in crate::ui::tests) initial_active_bank: Option<String>,
    #[serde(default)]
    pub(in crate::ui::tests) initial_focused_pad: Option<String>,
    #[serde(default)]
    pub(in crate::ui::tests) initial_last_capture: Option<String>,
    #[serde(default)]
    pub(in crate::ui::tests) initial_preview_mode: Option<String>,
    #[serde(default)]
    pub(in crate::ui::tests) initial_w30_grit: Option<f32>,
    pub(in crate::ui::tests) requested_at: TimestampMs,
    pub(in crate::ui::tests) committed_at: TimestampMs,
    pub(in crate::ui::tests) boundary: W30RegressionBoundary,
    pub(in crate::ui::tests) expected: W30RegressionExpected,
}

#[derive(Debug, Deserialize)]
pub(in crate::ui::tests) struct W30RegressionSourceWindow {
    pub(in crate::ui::tests) source_id: String,
    pub(in crate::ui::tests) start_seconds: f32,
    pub(in crate::ui::tests) end_seconds: f32,
    pub(in crate::ui::tests) start_frame: u64,
    pub(in crate::ui::tests) end_frame: u64,
}

#[derive(Debug, Deserialize)]
pub(in crate::ui::tests) struct W30RegressionCapture {
    pub(in crate::ui::tests) capture_id: String,
    pub(in crate::ui::tests) bank: String,
    pub(in crate::ui::tests) pad: String,
    pub(in crate::ui::tests) pinned: bool,
    #[serde(default)]
    pub(in crate::ui::tests) notes: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::ui::tests) enum W30RegressionAction {
    LiveRecall,
    RawCaptureAudition,
    PromotedAudition,
    TriggerPad,
    SwapBank,
    ApplyDamageProfile,
    LoopFreeze,
    BrowseSlicePool,
}

pub(in crate::ui::tests) fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub(in crate::ui::tests) struct W30RegressionBoundary {
    pub(in crate::ui::tests) kind: W30RegressionBoundaryKind,
    pub(in crate::ui::tests) beat_index: u64,
    pub(in crate::ui::tests) bar_index: u64,
    pub(in crate::ui::tests) phrase_index: u64,
    pub(in crate::ui::tests) scene_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::ui::tests) enum W30RegressionBoundaryKind {
    Immediate,
    Beat,
    HalfBar,
    Bar,
    Phrase,
    Scene,
}

#[derive(Debug, Deserialize)]
pub(in crate::ui::tests) struct W30RegressionExpected {
    #[serde(default)]
    pub(in crate::ui::tests) jam_contains: Vec<String>,
    pub(in crate::ui::tests) capture_contains: Vec<String>,
    pub(in crate::ui::tests) log_contains: Vec<String>,
}

pub(in crate::ui::tests) fn w30_preview_mode_state(
    value: &str,
) -> riotbox_core::session::W30PreviewModeState {
    match value {
        "live_recall" => riotbox_core::session::W30PreviewModeState::LiveRecall,
        "raw_capture_audition" => riotbox_core::session::W30PreviewModeState::RawCaptureAudition,
        "promoted_audition" => riotbox_core::session::W30PreviewModeState::PromotedAudition,
        other => panic!("unsupported W-30 preview mode fixture value: {other}"),
    }
}

impl Mc202RegressionBoundary {
    pub(in crate::ui::tests) fn to_commit_boundary_state(&self) -> CommitBoundaryState {
        CommitBoundaryState {
            kind: match self.kind {
                Mc202RegressionBoundaryKind::Immediate => {
                    riotbox_core::action::CommitBoundary::Immediate
                }
                Mc202RegressionBoundaryKind::Beat => riotbox_core::action::CommitBoundary::Beat,
                Mc202RegressionBoundaryKind::HalfBar => {
                    riotbox_core::action::CommitBoundary::HalfBar
                }
                Mc202RegressionBoundaryKind::Bar => riotbox_core::action::CommitBoundary::Bar,
                Mc202RegressionBoundaryKind::Phrase => riotbox_core::action::CommitBoundary::Phrase,
                Mc202RegressionBoundaryKind::Scene => riotbox_core::action::CommitBoundary::Scene,
            },
            beat_index: self.beat_index,
            bar_index: self.bar_index,
            phrase_index: self.phrase_index,
            scene_id: self.scene_id.clone().map(SceneId::from),
        }
    }
}

impl SceneRegressionBoundary {
    pub(in crate::ui::tests) fn to_commit_boundary_state(&self) -> CommitBoundaryState {
        CommitBoundaryState {
            kind: match self.kind {
                SceneRegressionBoundaryKind::Immediate => {
                    riotbox_core::action::CommitBoundary::Immediate
                }
                SceneRegressionBoundaryKind::Beat => riotbox_core::action::CommitBoundary::Beat,
                SceneRegressionBoundaryKind::HalfBar => {
                    riotbox_core::action::CommitBoundary::HalfBar
                }
                SceneRegressionBoundaryKind::Bar => riotbox_core::action::CommitBoundary::Bar,
                SceneRegressionBoundaryKind::Phrase => riotbox_core::action::CommitBoundary::Phrase,
                SceneRegressionBoundaryKind::Scene => riotbox_core::action::CommitBoundary::Scene,
            },
            beat_index: self.beat_index,
            bar_index: self.bar_index,
            phrase_index: self.phrase_index,
            scene_id: self.scene_id.clone().map(SceneId::from),
        }
    }
}

pub(in crate::ui::tests) fn scene_regression_graph(section_labels: &[String]) -> SourceGraph {
    let mut graph = sample_shell_state()
        .app
        .source_graph
        .clone()
        .expect("sample shell source graph");
    graph.sections.clear();

    for (index, label) in section_labels.iter().enumerate() {
        let bar_start = (index as u32 * 8) + 1;
        graph.sections.push(riotbox_core::source_graph::Section {
            section_id: riotbox_core::ids::SectionId::from(format!("section-{index}")),
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

pub(in crate::ui::tests) fn seed_scene_fixture_state(
    shell: &mut JamShellState,
    fixture: &SceneRegressionFixture,
) {
    if let Some(current_scene) = fixture.initial_current_scene.as_deref() {
        shell.app.session.runtime_state.transport.current_scene =
            Some(SceneId::from(current_scene));
    }
    if let Some(active_scene) = fixture.initial_active_scene.as_deref() {
        shell.app.session.runtime_state.scene_state.active_scene =
            Some(SceneId::from(active_scene));
    }
    if let Some(restore_scene) = fixture.initial_restore_scene.as_deref() {
        shell.app.session.runtime_state.scene_state.restore_scene =
            Some(SceneId::from(restore_scene));
    }
    if let Some(reinforcement_mode) = fixture.tr909_reinforcement_mode {
        shell
            .app
            .session
            .runtime_state
            .lane_state
            .tr909
            .takeover_enabled = false;
        shell
            .app
            .session
            .runtime_state
            .lane_state
            .tr909
            .takeover_profile = None;
        shell
            .app
            .session
            .runtime_state
            .lane_state
            .tr909
            .reinforcement_mode = Some(reinforcement_mode);
    }
    if let Some(pattern_ref) = fixture.tr909_pattern_ref.as_deref() {
        shell.app.session.runtime_state.lane_state.tr909.pattern_ref = Some(pattern_ref.into());
    }
    shell.app.refresh_view();
}

impl W30RegressionBoundary {
    pub(in crate::ui::tests) fn to_commit_boundary_state(&self) -> CommitBoundaryState {
        CommitBoundaryState {
            kind: match self.kind {
                W30RegressionBoundaryKind::Immediate => {
                    riotbox_core::action::CommitBoundary::Immediate
                }
                W30RegressionBoundaryKind::Beat => riotbox_core::action::CommitBoundary::Beat,
                W30RegressionBoundaryKind::HalfBar => riotbox_core::action::CommitBoundary::HalfBar,
                W30RegressionBoundaryKind::Bar => riotbox_core::action::CommitBoundary::Bar,
                W30RegressionBoundaryKind::Phrase => riotbox_core::action::CommitBoundary::Phrase,
                W30RegressionBoundaryKind::Scene => riotbox_core::action::CommitBoundary::Scene,
            },
            beat_index: self.beat_index,
            bar_index: self.bar_index,
            phrase_index: self.phrase_index,
            scene_id: self.scene_id.clone().map(SceneId::from),
        }
    }
}
