use crate::test_support::{scene_energy_for_label, scene_label_hint};
use riotbox_core::{
    TimestampMs,
    action::{
        Action, ActionCommand, ActionDraft, ActionParams, ActionResult, ActionStatus, ActionTarget,
        ActorType, GhostMode, Quantization, TargetScope, UndoPolicy,
    },
    ids::{ActionId, AssetId, BankId, CaptureId, PadId, SceneId, SectionId, SourceId},
    queue::ActionQueue,
    session::{
        SceneMovementDirectionState, SceneMovementKindState, SceneMovementLaneIntentState,
        SceneMovementState, SessionFile, Tr909ReinforcementModeState, Tr909TakeoverProfileState,
    },
    source_graph::{
        AnalysisSummary, AnalysisWarning, Asset, AssetType, Candidate, CandidateType,
        DecodeProfile, EnergyClass, GraphProvenance, QualityClass, Relationship, RelationshipType,
        Section, SectionLabelHint, SourceDescriptor, SourceGraph,
    },
    transport::{CommitBoundaryState, TransportClockState},
};
use serde::Deserialize;

use super::*;

#[test]
fn footer_keys_line_styles_top_legend_key_tokens() {
    let line = footer_keys_line("i jam inspect", "re-ingest source");
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(
        rendered,
        "Keys: q quit | ? help | 1-4 screens | Tab switch | i inspect | space play/pause | [ ] drum | r re-ingest"
    );
    assert_eq!(line.spans[1].content.as_ref(), "q");
    assert_eq!(line.spans[1].style.fg, Some(Color::Cyan));
    assert_eq!(line.spans[4].content.as_ref(), "?");
    assert_eq!(line.spans[4].style.fg, Some(Color::Cyan));
    assert_eq!(line.spans[7].content.as_ref(), "1-4");
    assert_eq!(line.spans[7].style.fg, Some(Color::Cyan));
    assert_eq!(line.spans[10].content.as_ref(), "Tab");
    assert_eq!(line.spans[10].style.fg, Some(Color::Cyan));
    assert_eq!(line.spans[13].content.as_ref(), "i");
    assert_eq!(line.spans[13].style.fg, Some(Color::Cyan));
    assert_eq!(line.spans[16].content.as_ref(), "space");
    assert_eq!(line.spans[16].style.fg, Some(Color::Cyan));
    assert_eq!(line.spans[19].content.as_ref(), "[ ]");
    assert_eq!(line.spans[19].style.fg, Some(Color::Cyan));
    assert_eq!(line.spans[22].content.as_ref(), "r");
    assert_eq!(line.spans[22].style.fg, Some(Color::Cyan));
}

#[test]
fn footer_keys_line_compacts_load_mode_labels() {
    let line = footer_keys_line("i return to perform", "reload session");
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(
        rendered,
        "Keys: q quit | ? help | 1-4 screens | Tab switch | i perform | space play/pause | [ ] drum | r reload"
    );
}

#[test]
fn footer_line_styles_define_first_visual_hierarchy() {
    let primary = footer_primary_line("y scene jump | g follow | f fill");
    let primary_text = primary
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(primary_text, "Primary: y scene jump | g follow | f fill");
    assert_eq!(primary.spans[0].content.as_ref(), "Primary:");
    assert_eq!(primary.spans[0].style.fg, Some(Color::Cyan));
    assert!(
        primary.spans[0].style.add_modifier.contains(Modifier::BOLD),
        "{primary:?}"
    );
    assert_eq!(primary.spans[2].content.as_ref(), "y");
    assert_eq!(primary.spans[2].style.fg, Some(Color::Cyan));
    assert_eq!(primary.spans[5].content.as_ref(), "g");
    assert_eq!(primary.spans[5].style.fg, Some(Color::Cyan));
    assert_eq!(primary.spans[8].content.as_ref(), "f");
    assert_eq!(primary.spans[8].style.fg, Some(Color::Cyan));

    let scene = footer_scene_line("launch drop @ next bar | rise [===>] | 2 trail");
    assert_eq!(scene.spans[0].content.as_ref(), "Scene:");
    assert_eq!(scene.spans[0].style.fg, Some(Color::Yellow));
    assert!(
        scene.spans[0].style.add_modifier.contains(Modifier::BOLD),
        "{scene:?}"
    );
    assert_eq!(scene.spans[1].style.fg, Some(Color::Yellow));

    let status = footer_status_line("Status: playing");
    assert_eq!(status.spans[0].style.fg, Some(Color::DarkGray));

    let ok = footer_ok_line("Warnings clear");
    assert_eq!(ok.spans[0].style.fg, Some(Color::Green));

    let warning = footer_warning_line("tempo weak");
    assert_eq!(warning.spans[0].content.as_ref(), "Warning:");
    assert_eq!(warning.spans[0].style.fg, Some(Color::Red));
    assert!(
        warning.spans[0].style.add_modifier.contains(Modifier::BOLD),
        "{warning:?}"
    );
    assert_eq!(warning.spans[1].style.fg, Some(Color::Yellow));
}

#[test]
fn footer_advanced_line_styles_gesture_key_prefixes() {
    let line = footer_advanced_line("Y restore | a answer | b voice | d push");
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(
        rendered,
        "Advanced: Y restore | a answer | b voice | d push | more in ? help"
    );
    assert_eq!(line.spans[1].content.as_ref(), "Y");
    assert_eq!(line.spans[1].style.fg, Some(Color::Cyan));
    assert_eq!(line.spans[4].content.as_ref(), "a");
    assert_eq!(line.spans[4].style.fg, Some(Color::Cyan));
    assert_eq!(line.spans[7].content.as_ref(), "b");
    assert_eq!(line.spans[7].style.fg, Some(Color::Cyan));
    assert_eq!(line.spans[10].content.as_ref(), "d");
    assert_eq!(line.spans[10].style.fg, Some(Color::Cyan));
}

#[test]
fn footer_lane_ops_line_styles_gesture_key_prefixes() {
    let line = footer_lane_ops_line("t trigger | s step | x swap | z freeze");
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(rendered, "Lane ops: t trigger | s step | x swap | z freeze");
    assert_eq!(line.spans[1].content.as_ref(), "t");
    assert_eq!(line.spans[1].style.fg, Some(Color::Cyan));
    assert_eq!(line.spans[4].content.as_ref(), "s");
    assert_eq!(line.spans[4].style.fg, Some(Color::Cyan));
    assert_eq!(line.spans[7].content.as_ref(), "x");
    assert_eq!(line.spans[7].style.fg, Some(Color::Cyan));
    assert_eq!(line.spans[10].content.as_ref(), "z");
    assert_eq!(line.spans[10].style.fg, Some(Color::Cyan));
}

#[test]
fn suggested_gesture_key_tokens_use_primary_control_style() {
    let line = line_with_primary_keys("what next: [c] capture  [u] undo");
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(rendered, "what next: [c] capture  [u] undo");
    assert_eq!(line.spans[0].content.as_ref(), "what next: ");
    assert_eq!(line.spans[1].content.as_ref(), "[c]");
    assert_eq!(line.spans[1].style.fg, Some(Color::Cyan));
    assert!(
        line.spans[1].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[3].content.as_ref(), "[u]");
    assert_eq!(line.spans[3].style.fg, Some(Color::Cyan));
    assert!(
        line.spans[3].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
}

#[test]
fn suggested_gesture_lines_style_start_key_token() {
    let shell = sample_shell_state();
    let lines = suggested_gesture_lines(&shell);

    assert_eq!(lines[0].spans[0].content.as_ref(), "[Space]");
    assert_eq!(lines[0].spans[0].style.fg, Some(Color::Cyan));
    assert!(
        lines[0].spans[0]
            .style
            .add_modifier
            .contains(Modifier::BOLD),
        "{:?}",
        lines[0]
    );
}

#[test]
fn suggested_gesture_lines_promote_feral_ready_moves() {
    let mut shell = sample_shell_state();
    shell.app.session.runtime_state.transport.is_playing = true;
    shell.app.session.runtime_state.scene_state.scenes.clear();
    shell.app.queue = ActionQueue::new();
    shell.app.refresh_view();
    let lines = suggested_gesture_lines(&shell);
    let rendered = lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(rendered.contains("feral ready: [j] browse  [f] fill"));
    assert!(rendered.contains("[g] follow  [a] answer"));
    assert!(rendered.contains("[c] capture if it bites"));
    assert_eq!(lines[0].spans[1].content.as_ref(), "[j]");
    assert_eq!(lines[0].spans[1].style.fg, Some(Color::Cyan));
}

#[test]
fn suggested_gesture_lines_do_not_promote_near_miss_feral_moves() {
    let mut shell = sample_shell_state();
    shell.app.session.runtime_state.transport.is_playing = true;
    shell.app.session.runtime_state.scene_state.scenes.clear();
    shell.app.queue = ActionQueue::new();
    let graph = shell
        .app
        .source_graph
        .as_mut()
        .expect("sample shell should include source graph");
    graph.relationships.retain(|relationship| {
        relationship.relation_type != RelationshipType::SupportsBreakRebuild
    });
    shell.app.refresh_view();
    let lines = suggested_gesture_lines(&shell);
    let rendered = lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(!rendered.contains("feral ready:"));
    assert!(rendered.contains("what changed:"), "{rendered}");
}

#[test]
fn help_key_prefixes_use_primary_control_style() {
    let line = line_with_primary_key_prefixes("space: play / pause | y: jump | Tab: next");
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(rendered, "space: play / pause | y: jump | Tab: next");
    assert_eq!(line.spans[0].content.as_ref(), "space");
    assert_eq!(line.spans[0].style.fg, Some(Color::Cyan));
    assert!(
        line.spans[0].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[3].content.as_ref(), "y");
    assert_eq!(line.spans[3].style.fg, Some(Color::Cyan));
    assert_eq!(line.spans[6].content.as_ref(), "Tab");
    assert_eq!(line.spans[6].style.fg, Some(Color::Cyan));
}

#[test]
fn help_primary_gesture_line_styles_key_prefixes_without_rewriting_text() {
    let shell = sample_shell_state();
    let line = line_with_primary_key_prefixes(format!(
        "space: play / pause | {}",
        render_help_primary_gesture_items(&shell)
    ));
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(
        rendered,
        "space: play / pause | y: scene jump | g: follow | f: fill"
    );
    assert_eq!(line.spans[0].content.as_ref(), "space");
    assert_eq!(line.spans[0].style.fg, Some(Color::Cyan));
    assert_eq!(line.spans[3].content.as_ref(), "y");
    assert_eq!(line.spans[3].style.fg, Some(Color::Cyan));
    assert_eq!(line.spans[6].content.as_ref(), "g");
    assert_eq!(line.spans[6].style.fg, Some(Color::Cyan));
    assert_eq!(line.spans[9].content.as_ref(), "f");
    assert_eq!(line.spans[9].style.fg, Some(Color::Cyan));
}

#[test]
fn capture_pending_do_next_styles_define_pending_hierarchy() {
    let intent = capture_pending_intent_line("queued [c] capture @ next_phrase");
    assert_eq!(
        intent.spans[0].content.as_ref(),
        "queued [c] capture @ next_phrase"
    );
    assert_eq!(intent.spans[0].style.fg, Some(Color::Yellow));
    assert!(
        intent.spans[0].style.add_modifier.contains(Modifier::BOLD),
        "{intent:?}"
    );

    let detail = capture_pending_detail_line("wait for commit");
    assert_eq!(detail.spans[0].content.as_ref(), "wait for commit");
    assert_eq!(detail.spans[0].style.fg, Some(Color::Yellow));
    assert!(
        !detail.spans[0].style.add_modifier.contains(Modifier::BOLD),
        "{detail:?}"
    );
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
    active_scene: String,
    #[allow(dead_code)]
    current_scene: String,
    #[allow(dead_code)]
    scenes: Vec<String>,
    #[serde(default)]
    result_summary: Option<String>,
    jam_contains: Vec<String>,
    log_contains: Vec<String>,
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
    jam_contains: Vec<String>,
    log_contains: Vec<String>,
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
    #[serde(default)]
    jam_contains: Vec<String>,
    capture_contains: Vec<String>,
    log_contains: Vec<String>,
}

fn w30_preview_mode_state(value: &str) -> riotbox_core::session::W30PreviewModeState {
    match value {
        "live_recall" => riotbox_core::session::W30PreviewModeState::LiveRecall,
        "raw_capture_audition" => riotbox_core::session::W30PreviewModeState::RawCaptureAudition,
        "promoted_audition" => riotbox_core::session::W30PreviewModeState::PromotedAudition,
        other => panic!("unsupported W-30 preview mode fixture value: {other}"),
    }
}

impl Mc202RegressionBoundary {
    fn to_commit_boundary_state(&self) -> CommitBoundaryState {
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
    fn to_commit_boundary_state(&self) -> CommitBoundaryState {
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

fn scene_regression_graph(section_labels: &[String]) -> SourceGraph {
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

fn seed_scene_fixture_state(shell: &mut JamShellState, fixture: &SceneRegressionFixture) {
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
    fn to_commit_boundary_state(&self) -> CommitBoundaryState {
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

fn sample_shell_state() -> JamShellState {
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-12T00:00:00Z");
    session.runtime_state.transport.position_beats = 32.0;
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-a"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-a"));
    session.runtime_state.macro_state.source_retain = 0.7;
    session.runtime_state.macro_state.chaos = 0.2;
    session.runtime_state.macro_state.mc202_touch = 0.8;
    session.runtime_state.macro_state.w30_grit = 0.5;
    session.runtime_state.macro_state.tr909_slam = 0.9;
    session.runtime_state.mixer_state.drum_level = 0.82;
    session.runtime_state.mixer_state.music_level = 0.64;
    session.runtime_state.lane_state.mc202.role = Some("leader".into());
    session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
    session.runtime_state.lane_state.tr909.takeover_enabled = true;
    session.runtime_state.lane_state.tr909.takeover_profile =
        Some(Tr909TakeoverProfileState::ControlledPhraseTakeover);
    session.runtime_state.lane_state.tr909.pattern_ref = Some("scene-a-main".into());
    session.ghost_state.mode = GhostMode::Assist;
    session.runtime_state.lane_state.tr909.last_fill_bar = Some(6);
    session.runtime_state.lane_state.tr909.reinforcement_mode =
        Some(Tr909ReinforcementModeState::Takeover);
    session.action_log.actions.push(Action {
        id: ActionId(1),
        actor: ActorType::User,
        command: ActionCommand::CaptureNow,
        params: ActionParams::Capture { bars: Some(2) },
        target: ActionTarget {
            scope: Some(TargetScope::LaneW30),
            ..Default::default()
        },
        requested_at: 100,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(120),
        result: Some(ActionResult {
            accepted: true,
            summary: "captured".into(),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: Some("capture opener".into()),
    });
    session.action_log.actions.push(Action {
        id: ActionId(2),
        actor: ActorType::Ghost,
        command: ActionCommand::MutateScene,
        params: ActionParams::Mutation {
            intensity: 0.4,
            target_id: Some("scene-a".into()),
        },
        target: ActionTarget {
            scope: Some(TargetScope::Scene),
            scene_id: Some(SceneId::from("scene-a")),
            ..Default::default()
        },
        requested_at: 125,
        quantization: Quantization::NextPhrase,
        status: ActionStatus::Rejected,
        committed_at: None,
        result: Some(ActionResult {
            accepted: false,
            summary: "scene lock blocked ghost mutation".into(),
        }),
        undo_policy: UndoPolicy::NotUndoable {
            reason: "rejected actions do not create undo state".into(),
        },
        explanation: Some("ghost suggestion rejected".into()),
    });
    session.action_log.actions.push(Action {
        id: ActionId(3),
        actor: ActorType::User,
        command: ActionCommand::UndoLast,
        params: ActionParams::Empty,
        target: ActionTarget {
            scope: Some(TargetScope::Session),
            ..Default::default()
        },
        requested_at: 140,
        quantization: Quantization::Immediate,
        status: ActionStatus::Undone,
        committed_at: Some(140),
        result: Some(ActionResult {
            accepted: true,
            summary: "undid most recent musical action".into(),
        }),
        undo_policy: UndoPolicy::NotUndoable {
            reason: "undo markers are not undoable".into(),
        },
        explanation: Some("user trust correction".into()),
    });

    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: SourceId::from("src-1"),
            path: "fixtures/input.wav".into(),
            content_hash: "hash-1".into(),
            duration_seconds: 12.0,
            sample_rate: 44_100,
            channel_count: 2,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "0.1.0".into(),
            provider_set: vec!["decoded.wav_baseline".into()],
            generated_at: "2026-04-12T00:00:00Z".into(),
            source_hash: "hash-1".into(),
            analysis_seed: 1,
            run_notes: Some("test".into()),
        },
    );
    graph.timing.bpm_estimate = Some(126.0);
    graph.timing.bpm_confidence = 0.76;
    graph.sections.push(Section {
        section_id: SectionId::from("section-a"),
        label_hint: SectionLabelHint::Intro,
        start_seconds: 0.0,
        end_seconds: 4.0,
        bar_start: 1,
        bar_end: 2,
        energy_class: EnergyClass::Medium,
        confidence: 0.71,
        tags: vec!["decoded_wave".into()],
    });
    graph.sections.push(Section {
        section_id: SectionId::from("section-b"),
        label_hint: SectionLabelHint::Drop,
        start_seconds: 4.0,
        end_seconds: 12.0,
        bar_start: 3,
        bar_end: 6,
        energy_class: EnergyClass::High,
        confidence: 0.83,
        tags: vec!["decoded_wave".into()],
    });
    graph.assets.push(Asset {
        asset_id: AssetId::from("asset-a"),
        asset_type: AssetType::LoopWindow,
        start_seconds: 0.0,
        end_seconds: 4.0,
        start_bar: 1,
        end_bar: 2,
        confidence: 0.79,
        tags: vec!["loop".into()],
        source_refs: vec!["src-1".into()],
    });
    graph.assets.push(Asset {
        asset_id: AssetId::from("asset-hook"),
        asset_type: AssetType::HookFragment,
        start_seconds: 4.0,
        end_seconds: 5.0,
        start_bar: 3,
        end_bar: 3,
        confidence: 0.81,
        tags: vec!["hook".into()],
        source_refs: vec!["src-1".into()],
    });
    graph.candidates.push(Candidate {
        candidate_id: "cand-loop".into(),
        candidate_type: CandidateType::LoopCandidate,
        asset_ref: AssetId::from("asset-a"),
        score: 0.84,
        confidence: 0.78,
        tags: vec!["decoded_wave".into()],
        constraints: vec!["bar_aligned".into()],
        provenance_refs: vec!["provider:decoded.wav_baseline".into()],
    });
    graph.candidates.push(Candidate {
        candidate_id: "cand-capture".into(),
        candidate_type: CandidateType::CaptureCandidate,
        asset_ref: AssetId::from("asset-hook"),
        score: 0.79,
        confidence: 0.74,
        tags: vec!["feral".into()],
        constraints: vec!["capture_first".into()],
        provenance_refs: vec!["provider:decoded.wav_baseline".into()],
    });
    graph.relationships.push(Relationship {
        relation_type: RelationshipType::SupportsBreakRebuild,
        from_id: "asset-hook".into(),
        to_id: "asset-a".into(),
        weight: 0.78,
        notes: Some("hook supports loop rebuild".into()),
    });
    graph.relationships.push(Relationship {
        relation_type: RelationshipType::HighQuoteRiskWith,
        from_id: "asset-hook".into(),
        to_id: "src-1".into(),
        weight: 0.64,
        notes: Some("recognizable hook".into()),
    });
    graph.analysis_summary = AnalysisSummary {
        overall_confidence: 0.74,
        timing_quality: QualityClass::Medium,
        section_quality: QualityClass::High,
        loop_candidate_count: 1,
        hook_candidate_count: 0,
        break_rebuild_potential: QualityClass::High,
        warnings: vec![AnalysisWarning {
            code: "wav_baseline_only".into(),
            message: "decoded-source baseline used WAV metadata and simple energy heuristics"
                .into(),
        }],
    };

    let mut queue = ActionQueue::new();
    queue.enqueue(
        ActionDraft::new(
            ActorType::Ghost,
            ActionCommand::MutateScene,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::Scene),
                ..Default::default()
            },
        ),
        130,
    );
    queue.enqueue(
        ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909FillNext,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        ),
        130,
    );
    let mut promote_draft = ActionDraft::new(
        ActorType::User,
        ActionCommand::PromoteCaptureToPad,
        Quantization::NextBar,
        ActionTarget {
            scope: Some(TargetScope::LaneW30),
            bank_id: Some(BankId::from("bank-a")),
            pad_id: Some(PadId::from("pad-01")),
            ..Default::default()
        },
    );
    promote_draft.params = ActionParams::Promotion {
        capture_id: Some("cap-01".into()),
        destination: Some("w30:bank-a/pad-01".into()),
    };
    promote_draft.explanation = Some("promote keeper capture into the live pad".into());
    queue.enqueue(promote_draft, 131);

    session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    session.captures.push(riotbox_core::session::CaptureRef {
        capture_id: "cap-01".into(),
        capture_type: riotbox_core::session::CaptureType::Pad,
        source_origin_refs: vec!["asset-a".into(), "src-1".into()],
        source_window: None,
        lineage_capture_refs: Vec::new(),
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: "captures/cap-01.wav".into(),
        assigned_target: None,
        is_pinned: false,
        notes: Some("keeper capture".into()),
    });

    let app = JamAppState::from_parts(session, Some(graph), queue);
    JamShellState::new(app, ShellLaunchMode::Ingest)
}

fn first_run_shell_state() -> JamShellState {
    let sample_shell = sample_shell_state();
    let mut session = sample_shell.app.session.clone();
    session.action_log.actions.clear();
    session.captures.clear();
    session.runtime_state.lane_state.w30.last_capture = None;

    let app = JamAppState::from_parts(
        session,
        sample_shell.app.source_graph.clone(),
        ActionQueue::new(),
    );
    JamShellState::new(app, ShellLaunchMode::Ingest)
}

fn first_result_shell_state() -> JamShellState {
    let mut shell = first_run_shell_state();
    shell.app.session.action_log.actions.push(Action {
        id: ActionId(1),
        actor: ActorType::User,
        command: ActionCommand::Tr909FillNext,
        params: ActionParams::Empty,
        target: ActionTarget {
            scope: Some(TargetScope::LaneTr909),
            ..Default::default()
        },
        requested_at: 200,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(220),
        result: Some(ActionResult {
            accepted: true,
            summary: "committed fill on next bar".into(),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: Some("first committed fill".into()),
    });

    shell.app.refresh_view();
    shell
}

fn sample_shell_without_pending_queue() -> JamShellState {
    let sample_shell = sample_shell_state();
    JamShellState::new(
        JamAppState::from_parts(
            sample_shell.app.session.clone(),
            sample_shell.app.source_graph.clone(),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Ingest,
    )
}

fn scene_post_commit_shell_state(
    command: ActionCommand,
    active_scene: &str,
    restore_scene: &str,
) -> JamShellState {
    let sample_shell = sample_shell_state();
    let mut session = sample_shell.app.session.clone();
    session.action_log.actions.clear();
    session.runtime_state.transport.current_scene = Some(SceneId::from(active_scene));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from(active_scene));
    session.runtime_state.scene_state.restore_scene = Some(SceneId::from(restore_scene));
    session.runtime_state.lane_state.tr909.takeover_enabled = false;
    session.runtime_state.lane_state.tr909.takeover_profile = None;
    session.runtime_state.lane_state.tr909.reinforcement_mode =
        Some(Tr909ReinforcementModeState::SourceSupport);
    session.runtime_state.lane_state.tr909.pattern_ref = Some("scene-support".into());
    session.action_log.actions.push(Action {
        id: ActionId(1),
        actor: ActorType::User,
        command,
        params: ActionParams::Scene {
            scene_id: Some(SceneId::from(active_scene)),
        },
        target: ActionTarget {
            scope: Some(TargetScope::Scene),
            scene_id: Some(SceneId::from(active_scene)),
            ..Default::default()
        },
        requested_at: 300,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(320),
        result: Some(ActionResult {
            accepted: true,
            summary: format!("scene {active_scene} landed"),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: Some(format!("landed {active_scene} scene move")),
    });

    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            session,
            sample_shell.app.source_graph.clone(),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );
    shell.app.set_transport_playing(true);
    shell
}

#[test]
fn renders_more_musical_jam_shell_snapshot() {
    let shell = sample_shell_state();
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("trust usable"));
    assert!(rendered.contains("scene scene-a | energy med"));
    assert!(rendered.contains("ghost"));
    assert!(rendered.contains("warnings"));
    assert!(rendered.contains("MC-202"));
    assert!(rendered.contains("W-30"));
    assert!(rendered.contains("TR-909"));
    assert!(rendered.contains("Suggested gestures"));
    assert!(rendered.contains("Pending / landed"));
    assert!(rendered.contains("next fill"));
    assert!(rendered.contains("wait [===>] next bar"), "{rendered}");
    assert!(
        rendered.contains("Primary: y scene jump | g follow | f fill"),
        "{rendered}"
    );
    assert!(
        rendered.contains("Advanced: Y restore | a answer | b voice | P pressure | I instigate"),
        "{rendered}"
    );
    assert!(!rendered.contains("Sections"), "{rendered}");
}

#[test]
fn renders_jam_shell_inspect_snapshot() {
    let mut shell = sample_shell_state();
    shell.jam_mode = JamViewMode::Inspect;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Screen jam/inspect"), "{rendered}");
    assert!(rendered.contains("MC-202 detail"), "{rendered}");
    assert!(rendered.contains("W-30 detail"), "{rendered}");
    assert!(rendered.contains("TR-909 detail"), "{rendered}");
    assert!(rendered.contains("accent off"), "{rendered}");
    assert!(rendered.contains("Source structure"), "{rendered}");
    assert!(rendered.contains("Material flow"), "{rendered}");
    assert!(rendered.contains("Diagnostics"), "{rendered}");
    assert!(!rendered.contains("Suggested gestures"), "{rendered}");
}

#[test]
fn renders_jam_shell_with_scene_brain_summary() {
    let sample_shell = sample_shell_state();
    let mut session = sample_shell.app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-intro"),
        SceneId::from("scene-02-drop"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            session,
            sample_shell.app.source_graph.clone(),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );
    assert_eq!(
        shell.app.queue_scene_select(300),
        crate::jam_app::QueueControlResult::Enqueued
    );

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("idle @ 32.0"));
    assert!(rendered.contains("scene-01-intro"));
    assert!(rendered.contains("energy medium"));
    assert!(
        rendered.contains("source src-1 | next scene drop/high"),
        "{rendered}"
    );
    assert!(rendered.contains("scene-01-intro"));
    assert!(rendered.contains("live intro/med <> restore none"));
    assert!(rendered.contains("launch ->"), "{rendered}");
    assert!(rendered.contains("@ next bar"), "{rendered}");
    assert!(
        rendered.contains("pulse [===>] b32 | b8 | p1"),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "Scene: launch drop @ next bar | rise [===>] | 909 drive | 202 lift | 2 trail"
        ),
        "{rendered}"
    );
    assert!(rendered.contains("policy rise"), "{rendered}");
}

#[test]
fn scene_pending_line_styles_define_intent_hierarchy() {
    let sample_shell = sample_shell_state();
    let mut session = sample_shell.app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-intro"),
        SceneId::from("scene-02-drop"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            session,
            sample_shell.app.source_graph.clone(),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );
    assert_eq!(
        shell.app.queue_scene_select(300),
        crate::jam_app::QueueControlResult::Enqueued
    );

    let line = scene_pending_line(&shell);
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(
        rendered,
        "launch -> scene-02-drop @ next bar | policy rise | 909 drive | 202 lift"
    );
    assert_eq!(line.spans[0].content.as_ref(), "launch");
    assert_eq!(line.spans[0].style.fg, Some(Color::Yellow));
    assert!(
        line.spans[0].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[2].content.as_ref(), "scene-02-drop");
    assert_eq!(line.spans[2].style.fg, Some(Color::Yellow));
    assert_eq!(line.spans[4].content.as_ref(), "next bar");
    assert_eq!(line.spans[4].style.fg, Some(Color::Yellow));
    assert!(
        line.spans[4].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[6].content.as_ref(), "rise");
    assert_eq!(line.spans[6].style.fg, Some(Color::Green));
    assert!(
        line.spans[6].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[8].content.as_ref(), "drive");
    assert_eq!(line.spans[10].content.as_ref(), "lift");
}

#[test]
fn renders_jam_shell_with_pending_scene_restore_summary() {
    let graph = sample_shell_state()
        .app
        .source_graph
        .clone()
        .expect("sample shell source graph");
    let mut session = sample_shell_state().app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-drop"),
        SceneId::from("scene-02-intro"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-drop"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-drop"));
    session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));

    let mut shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Load,
    );
    shell.app.session.runtime_state.scene_state.restore_scene =
        Some(SceneId::from("scene-02-intro"));
    assert_eq!(
        shell.app.queue_scene_restore(300),
        crate::jam_app::QueueControlResult::Enqueued
    );

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("scene-01-drop"), "{rendered}");
    assert!(rendered.contains("energy medium"), "{rendered}");
    assert!(
        rendered.contains("live drop/med <> restore intro/high"),
        "{rendered}"
    );
    assert!(
        rendered.contains("restore -> scene-02-intro @ next bar"),
        "{rendered}"
    );
    assert!(rendered.contains("policy rise"), "{rendered}");
    assert!(
        rendered.contains("pulse [===>] b32 | b8 | p1"),
        "{rendered}"
    );
    assert!(
        rendered
            .contains("restore intro @ next bar | rise [===>] | 909 drive | 202 lift | 2 trail"),
        "{rendered}"
    );
}

#[test]
fn renders_log_shell_with_pending_scene_restore_summary() {
    let graph = scene_regression_graph(&["drop".into(), "break".into()]);
    let mut session = sample_shell_state().app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-drop"),
        SceneId::from("scene-02-break"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-02-break"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-02-break"));
    session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));

    let mut shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Load,
    );
    assert_eq!(
        shell.app.queue_scene_restore(300),
        crate::jam_app::QueueControlResult::Enqueued
    );
    shell.active_screen = ShellScreen::Log;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("restore scene-01-drop"), "{rendered}");
    assert!(
        rendered.contains("requested 300 | restore scene"),
        "{rendered}"
    );
    assert!(rendered.contains("scene-01-drop on next bar"), "{rendered}");
}

#[test]
fn renders_jam_shell_with_pending_mc202_role_change() {
    let mut shell = sample_shell_state();
    assert_eq!(
        shell.app.queue_mc202_role_toggle(200),
        crate::jam_app::QueueControlResult::Enqueued
    );

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("current voice leader"));
    assert!(rendered.contains("next voice follower"));
}

#[test]
fn renders_jam_shell_with_pending_mc202_follower_generation() {
    let first_run_shell = first_run_shell_state();
    let mut shell = JamShellState::new(first_run_shell.app, ShellLaunchMode::Load);
    shell.app.set_transport_playing(true);
    assert_eq!(
        shell.app.queue_mc202_generate_follower(200),
        crate::jam_app::QueueControlResult::Enqueued
    );

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("next follow"));
    assert!(
        rendered.contains("wait [=======>] next phrase"),
        "{rendered}"
    );
}

#[test]
fn renders_jam_shell_with_pending_mc202_answer_generation() {
    let mut shell = sample_shell_state();
    assert_eq!(
        shell.app.queue_mc202_generate_answer(200),
        crate::jam_app::QueueControlResult::Enqueued
    );

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("next answer"));
}

#[test]
fn renders_jam_shell_with_pending_mc202_pressure_generation() {
    let mut shell = sample_shell_state();
    assert_eq!(
        shell.app.queue_mc202_generate_pressure(200),
        crate::jam_app::QueueControlResult::Enqueued
    );

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("next pressure"), "{rendered}");
}

#[test]
fn renders_jam_shell_with_pending_mc202_instigator_generation() {
    let mut shell = sample_shell_state();
    assert_eq!(
        shell.app.queue_mc202_generate_instigator(200),
        crate::jam_app::QueueControlResult::Enqueued
    );

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("next instigate"), "{rendered}");
}

#[test]
fn renders_jam_shell_with_two_promoted_pending_actions_and_queue_summary() {
    let first_run_shell = first_run_shell_state();
    let mut shell = JamShellState::new(first_run_shell.app, ShellLaunchMode::Load);
    shell.app.queue_scene_mutation(200);
    shell.app.queue_tr909_fill(201);
    shell.app.queue_capture_bar(202);

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("next 1 user mutate"), "{rendered}");
    assert!(rendered.contains("next 2 user fill"), "{rendered}");
    assert!(rendered.contains("+1 more"), "{rendered}");
    assert!(!rendered.contains("more queued"), "{rendered}");
}

#[test]
fn quantization_countdown_cues_match_boundary_widths() {
    assert_eq!(
        quantization_countdown_cue(Quantization::NextBeat, 32, 8),
        "[>]"
    );
    assert_eq!(
        quantization_countdown_cue(Quantization::NextHalfBar, 3, 1),
        "[=>]"
    );
    assert_eq!(
        quantization_countdown_cue(Quantization::NextBar, 32, 8),
        "[===>]"
    );
    assert_eq!(
        quantization_countdown_cue(Quantization::NextPhrase, 32, 8),
        "[=======>]"
    );
}

#[test]
fn queued_timing_rail_styles_define_boundary_hierarchy() {
    let shell = sample_shell_state();
    let line = queued_timing_rail_line(&shell).expect("queued timing rail");
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(rendered, "wait [===>] next bar | b32 | bar8 | p1");
    assert_eq!(line.spans[0].content.as_ref(), "wait ");
    assert_eq!(line.spans[0].style.fg, Some(Color::DarkGray));
    assert_eq!(line.spans[1].content.as_ref(), "[===>]");
    assert_eq!(line.spans[1].style.fg, Some(Color::Yellow));
    assert!(
        line.spans[1].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[3].content.as_ref(), "next bar");
    assert_eq!(line.spans[3].style.fg, Some(Color::Yellow));
    assert!(
        line.spans[3].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[4].style.fg, Some(Color::DarkGray));
}

#[test]
fn queued_scene_timing_rail_styles_pulse_hierarchy() {
    let sample_shell = sample_shell_state();
    let mut session = sample_shell.app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-intro"),
        SceneId::from("scene-02-drop"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            session,
            sample_shell.app.source_graph.clone(),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );
    assert_eq!(
        shell.app.queue_scene_select(300),
        crate::jam_app::QueueControlResult::Enqueued
    );

    let line = queued_timing_rail_line(&shell).expect("scene timing rail");
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(rendered, "pulse [===>] b32 | b8 | p1");
    assert_eq!(line.spans[0].content.as_ref(), "pulse ");
    assert_eq!(line.spans[0].style.fg, Some(Color::DarkGray));
    assert_eq!(line.spans[1].content.as_ref(), "[===>]");
    assert_eq!(line.spans[1].style.fg, Some(Color::Yellow));
    assert!(
        line.spans[1].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[2].style.fg, Some(Color::DarkGray));
}

#[test]
fn renders_jam_shell_with_first_run_onramp() {
    let shell = first_run_shell_state();
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Start Here"), "{rendered}");
    assert!(rendered.contains("1 [Space] start transport"), "{rendered}");
    assert!(
        rendered.contains("2 [f] queue one first fill"),
        "{rendered}"
    );
    assert!(
        rendered.contains("3 [2] watch Log when it lands on the next bar"),
        "{rendered}"
    );
}

#[test]
fn renders_jam_shell_with_queued_first_move_guidance() {
    let mut shell = first_run_shell_state();
    shell.app.queue_tr909_fill(200);

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Your first move is armed."), "{rendered}");
    assert!(rendered.contains("next bar"), "{rendered}");
    assert!(rendered.contains("confirm it in Log"), "{rendered}");
    assert!(rendered.contains("[c] capture"), "{rendered}");
}

#[test]
fn renders_jam_shell_with_first_result_guidance() {
    let shell = first_result_shell_state();
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("What changed: landed user fill"),
        "{rendered}"
    );
    assert!(
        rendered.contains("What next: [c] capture it or [u] undo it if it missed."),
        "{rendered}"
    );
    assert!(
        rendered.contains("Then try one more move: [y] jump or [g] follow."),
        "{rendered}"
    );
}

#[test]
fn next_panel_promotes_timing_rail_above_landed_history() {
    let mut shell = first_result_shell_state();
    shell.app.queue_tr909_fill(240);

    let line_texts = next_panel_lines(&shell)
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>();

    assert_eq!(line_texts[0], "user tr909.fill_next @ next_bar");
    assert_eq!(line_texts[1], "scene transition idle");
    assert!(
        line_texts[2].starts_with("wait [===>] next bar"),
        "{line_texts:?}"
    );
    assert_eq!(line_texts[3], "landed user fill");
}

#[test]
fn renders_jam_shell_with_post_commit_next_step_cue() {
    let first_result_shell = first_result_shell_state();
    let mut shell = JamShellState::new(first_result_shell.app, ShellLaunchMode::Load);
    shell.app.set_transport_playing(true);
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("landed user fill"), "{rendered}");
    assert!(
        rendered.contains("feral ready: [j] browse  [f] fill"),
        "{rendered}"
    );
    assert!(rendered.contains("[g] follow  [a] answer"), "{rendered}");
    assert!(rendered.contains("[c] capture if it bites"), "{rendered}");
}

#[test]
fn renders_jam_shell_with_single_scene_jump_waiting_cue() {
    let sample_shell = sample_shell_state();
    let mut session = sample_shell.app.session.clone();
    session.runtime_state.scene_state.scenes = vec![SceneId::from("scene-01-intro")];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));

    let shell = JamShellState::new(
        JamAppState::from_parts(
            session,
            sample_shell.app.source_graph.clone(),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("source src-1 | next scene waits for 2") && rendered.contains("scenes"),
        "{rendered}"
    );
    assert!(
        rendered.contains("[y] jump waits for 2 scenes"),
        "{rendered}"
    );
    assert!(
        rendered.contains("Primary: y jump waits | g follow | f fill"),
        "{rendered}"
    );

    let mut shell = shell;
    shell.show_help = true;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("space: play / pause | y: jump waits | g: follow | f: fill"),
        "{rendered}"
    );
}

#[test]
fn renders_scene_jump_post_commit_guidance() {
    let shell = scene_post_commit_shell_state(
        ActionCommand::SceneLaunch,
        "scene-02-break",
        "scene-01-drop",
    );
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("scene break/high | restore drop/med"),
        "{rendered}"
    );
    assert!(
        rendered.contains("live break/high <> restore drop/med"),
        "{rendered}"
    );
    assert!(
        rendered.contains("landed user scene jump | energy rise"),
        "{rendered}"
    );
    assert!(rendered.contains("909 lift"), "{rendered}");
    assert!(rendered.contains("next [Y]"), "{rendered}");
    assert!(rendered.contains("restore [c] capture"), "{rendered}");
    assert!(rendered.contains("[c] capture"), "{rendered}");
}

#[test]
fn scene_post_commit_cue_styles_define_performance_hierarchy() {
    let shell = scene_post_commit_shell_state(
        ActionCommand::SceneLaunch,
        "scene-02-break",
        "scene-01-drop",
    );
    let line = scene_post_commit_cue_line(&shell).expect("scene post-commit cue");
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(
        rendered,
        "scene break/high | restore drop/med | 909 lift | next [Y] restore [c] capture"
    );
    assert_eq!(line.spans[0].style.fg, Some(Color::DarkGray));
    assert_eq!(line.spans[1].content.as_ref(), "break/high");
    assert_eq!(line.spans[1].style.fg, Some(Color::Green));
    assert!(
        line.spans[1].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[3].content.as_ref(), "drop/med");
    assert_eq!(line.spans[3].style.fg, Some(Color::Yellow));
    assert_eq!(line.spans[5].content.as_ref(), "909 lift");
    assert_eq!(line.spans[5].style.fg, Some(Color::Yellow));
    assert_eq!(line.spans[7].content.as_ref(), "[Y]");
    assert_eq!(line.spans[7].style.fg, Some(Color::Cyan));
    assert!(
        line.spans[7].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[9].content.as_ref(), "[c]");
    assert_eq!(line.spans[9].style.fg, Some(Color::Cyan));
    assert!(
        line.spans[9].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
}

#[test]
fn scene_post_commit_cue_surfaces_landed_movement() {
    let mut shell = scene_post_commit_shell_state(
        ActionCommand::SceneLaunch,
        "scene-02-break",
        "scene-01-drop",
    );
    shell.app.session.runtime_state.scene_state.last_movement = Some(SceneMovementState {
        action_id: ActionId(1),
        from_scene: Some(SceneId::from("scene-01-drop")),
        to_scene: SceneId::from("scene-02-break"),
        kind: SceneMovementKindState::Launch,
        direction: SceneMovementDirectionState::Rise,
        tr909_intent: SceneMovementLaneIntentState::Drive,
        mc202_intent: SceneMovementLaneIntentState::Lift,
        intensity: 0.75,
        committed_bar_index: 9,
        committed_phrase_index: 2,
    });
    shell.app.refresh_view();

    let rendered = scene_post_commit_cue_line(&shell)
        .expect("scene post-commit cue")
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert!(
        rendered.contains("move rise 909 drive 202 lift"),
        "{rendered}"
    );
}

#[test]
fn latest_landed_line_styles_define_result_hierarchy() {
    let shell = scene_post_commit_shell_state(
        ActionCommand::SceneLaunch,
        "scene-02-break",
        "scene-01-drop",
    );
    let line = latest_landed_line(&shell);
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(rendered, "landed user scene jump | energy rise");
    assert_eq!(latest_landed_text(&shell), rendered);
    assert_eq!(line.spans[0].content.as_ref(), "landed ");
    assert_eq!(line.spans[0].style.fg, Some(Color::DarkGray));
    assert_eq!(line.spans[1].content.as_ref(), "user ");
    assert_eq!(line.spans[1].style.fg, Some(Color::DarkGray));
    assert_eq!(line.spans[2].content.as_ref(), "scene jump");
    assert_eq!(line.spans[2].style.fg, Some(Color::Green));
    assert!(
        line.spans[2].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[4].content.as_ref(), "energy rise");
    assert_eq!(line.spans[4].style.fg, Some(Color::Green));
    assert!(
        line.spans[4].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
}

#[test]
fn renders_scene_restore_post_commit_guidance() {
    let shell = scene_post_commit_shell_state(
        ActionCommand::SceneRestore,
        "scene-01-drop",
        "scene-02-break",
    );
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("scene drop/med | restore break/high"),
        "{rendered}"
    );
    assert!(
        rendered.contains("live drop/med <> restore break/high"),
        "{rendered}"
    );
    assert!(
        rendered.contains("landed user restore | energy drop"),
        "{rendered}"
    );
    assert!(rendered.contains("909 lift"), "{rendered}");
    assert!(rendered.contains("next [y]"), "{rendered}");
    assert!(rendered.contains("jump [c] capture"), "{rendered}");
    assert!(rendered.contains("[c] capture"), "{rendered}");
}

#[test]
fn omits_scene_post_commit_tr909_lift_without_scene_accent() {
    let mut shell = scene_post_commit_shell_state(
        ActionCommand::SceneRestore,
        "scene-01-drop",
        "scene-02-break",
    );
    shell
        .app
        .session
        .runtime_state
        .lane_state
        .tr909
        .reinforcement_mode = None;
    shell.app.session.runtime_state.lane_state.tr909.pattern_ref = None;
    shell.app.refresh_view();
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("scene drop/med | restore break/high | next"),
        "{rendered}"
    );
    assert!(!rendered.contains("909 lift"), "{rendered}");
}

#[test]
fn renders_help_overlay_with_first_run_guidance() {
    let mut shell = first_run_shell_state();
    shell.show_help = true;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("First run"), "{rendered}");
    assert!(rendered.contains("space: start transport"), "{rendered}");
    assert!(rendered.contains("f: queue one first fill"), "{rendered}");
    assert!(
        rendered.contains("2: switch to Log and watch it land"),
        "{rendered}"
    );
    assert!(
        rendered.contains("After first loop: docs/jam_recipes.md -> Recipe 2 / Recipe 5"),
        "{rendered}"
    );
}

#[test]
fn renders_help_overlay_with_pending_scene_jump_cue() {
    let mut shell = sample_shell_state();
    assert_eq!(
        shell.app.queue_scene_select(300),
        crate::jam_app::QueueControlResult::Enqueued
    );
    shell.show_help = true;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Scene timing"), "{rendered}");
    assert!(
        rendered.contains("launch intro: lands at next bar"),
        "{rendered}"
    );
    assert!(
        rendered.contains("Jam: read launch/restore, pulse, live/restore energy"),
        "{rendered}"
    );
    assert!(
        rendered.contains("2: confirm the landed trail on Log"),
        "{rendered}"
    );
}

#[test]
fn renders_help_overlay_with_pending_scene_restore_cue() {
    let graph = scene_regression_graph(&["drop".into(), "break".into()]);
    let mut session = sample_shell_state().app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-drop"),
        SceneId::from("scene-02-break"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-02-break"));
    session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));

    let mut shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Load,
    );
    shell.app.set_transport_playing(true);
    assert_eq!(
        shell.app.queue_scene_restore(300),
        crate::jam_app::QueueControlResult::Enqueued
    );
    shell.show_help = true;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Scene timing"), "{rendered}");
    assert!(
        rendered.contains("restore drop: lands at next bar"),
        "{rendered}"
    );
    assert!(
        rendered.contains("2: confirm the landed trail on Log"),
        "{rendered}"
    );
}

#[test]
fn renders_help_overlay_with_capture_path_cue() {
    let mut shell = sample_shell_state();
    shell.active_screen = ShellScreen::Capture;
    shell.show_help = true;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Capture path"), "{rendered}");
    assert!(
        rendered.contains("Do Next: read capture -> promote -> hit"),
        "{rendered}"
    );
    assert!(
        rendered.contains("hear ... stored means [p] promote, then [w] hit"),
        "{rendered}"
    );
    assert!(
        rendered.contains("2: confirm promote, hit, and audition results in Log"),
        "{rendered}"
    );
}

#[test]
fn renders_jam_shell_with_restore_readiness_cue() {
    let graph = scene_regression_graph(&["intro".into(), "drop".into()]);
    let mut session = sample_shell_state().app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-intro"),
        SceneId::from("scene-02-drop"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.scene_state.restore_scene = None;

    let mut shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Load,
    );
    shell.app.set_transport_playing(true);

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("[y] jump first"), "{rendered}");
    assert!(
        rendered.contains("[Y] restore waits for one landed"),
        "{rendered}"
    );
    assert!(rendered.contains("jump"), "{rendered}");
}

#[test]
fn renders_help_overlay_with_restore_readiness_cue() {
    let graph = scene_regression_graph(&["intro".into(), "drop".into()]);
    let mut session = sample_shell_state().app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-intro"),
        SceneId::from("scene-02-drop"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.scene_state.restore_scene = None;

    let mut shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Load,
    );
    shell.app.set_transport_playing(true);
    shell.show_help = true;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Scene restore"), "{rendered}");
    assert!(
        rendered.contains("Y waits for one landed jump"),
        "{rendered}"
    );
    assert!(
        rendered.contains("land one jump, then Y can restore the last scene"),
        "{rendered}"
    );
}

#[test]
fn renders_jam_shell_with_restore_ready_cue() {
    let graph = scene_regression_graph(&["drop".into(), "break".into()]);
    let mut session = sample_shell_state().app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-drop"),
        SceneId::from("scene-02-break"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-02-break"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-02-break"));
    session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));

    let mut shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Load,
    );
    shell.app.set_transport_playing(true);

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("[Y] restore drop now (rise)"),
        "{rendered}"
    );
    assert!(
        rendered.contains("Scene: restore drop/high ready | rise | Y brings back drop/high"),
        "{rendered}"
    );
}

#[test]
fn renders_help_overlay_with_restore_ready_cue() {
    let graph = scene_regression_graph(&["drop".into(), "break".into()]);
    let mut session = sample_shell_state().app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-drop"),
        SceneId::from("scene-02-break"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-02-break"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-02-break"));
    session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));

    let mut shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Load,
    );
    shell.app.set_transport_playing(true);
    shell.show_help = true;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Scene restore"), "{rendered}");
    assert!(
        rendered.contains("Y is live now for drop/high (rise)"),
        "{rendered}"
    );
    assert!(
        rendered.contains("press Y to bring drop/high back on the next bar"),
        "{rendered}"
    );
}

fn mc202_committed_shell_state(fixture: &Mc202RegressionFixture) -> JamShellState {
    let sample_shell = sample_shell_state();
    let mut session = sample_shell.app.session.clone();
    session.action_log.actions.clear();
    session.captures.clear();
    session.runtime_state.lane_state.w30.last_capture = None;
    session.runtime_state.lane_state.mc202.role = Some(fixture.initial_role.clone());
    session.runtime_state.lane_state.mc202.phrase_ref = None;
    session.runtime_state.macro_state.mc202_touch = 0.4;

    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            session,
            sample_shell.app.source_graph.clone(),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );

    let queue_result = match fixture.action {
        Mc202RegressionAction::SetRole => shell.app.queue_mc202_role_toggle(fixture.requested_at),
        Mc202RegressionAction::GenerateFollower => shell
            .app
            .queue_mc202_generate_follower(fixture.requested_at),
        Mc202RegressionAction::GenerateAnswer => {
            shell.app.queue_mc202_generate_answer(fixture.requested_at)
        }
        Mc202RegressionAction::GeneratePressure => shell
            .app
            .queue_mc202_generate_pressure(fixture.requested_at),
        Mc202RegressionAction::GenerateInstigator => shell
            .app
            .queue_mc202_generate_instigator(fixture.requested_at),
    };
    assert_eq!(
        queue_result,
        crate::jam_app::QueueControlResult::Enqueued,
        "{} did not enqueue",
        fixture.name
    );

    let committed = shell.app.commit_ready_actions(
        fixture.boundary.to_commit_boundary_state(),
        fixture.committed_at,
    );
    assert_eq!(
        committed.len(),
        1,
        "{} did not commit exactly one action",
        fixture.name
    );
    assert_eq!(
        shell
            .app
            .session
            .runtime_state
            .lane_state
            .mc202
            .role
            .as_deref(),
        Some(fixture.expected.role.as_str()),
        "{} role drifted",
        fixture.name
    );
    assert_eq!(
        shell
            .app
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
        shell.app.session.runtime_state.macro_state.mc202_touch, fixture.expected.touch,
        "{} touch drifted",
        fixture.name
    );
    assert_eq!(
        shell
            .app
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

    shell
}

fn scene_committed_shell_state(fixture: &SceneRegressionFixture) -> JamShellState {
    let sample_shell = sample_shell_state();
    let graph = scene_regression_graph(&fixture.section_labels);
    let mut session = sample_shell.app.session.clone();
    session.runtime_state.transport.current_scene = None;
    session.runtime_state.scene_state.active_scene = None;
    session.runtime_state.scene_state.scenes.clear();

    let mut shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Load,
    );
    seed_scene_fixture_state(&mut shell, fixture);

    match fixture.action {
        SceneRegressionAction::ProjectCandidates => {}
        SceneRegressionAction::SelectNextScene => {
            assert_eq!(
                shell
                    .app
                    .queue_scene_select(fixture.requested_at.expect("scene select requested_at")),
                crate::jam_app::QueueControlResult::Enqueued,
                "{} did not enqueue",
                fixture.name
            );

            let committed = shell.app.commit_ready_actions(
                fixture
                    .boundary
                    .as_ref()
                    .expect("scene select boundary")
                    .to_commit_boundary_state(),
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
                shell
                    .app
                    .queue_scene_restore(fixture.requested_at.expect("scene restore requested_at")),
                crate::jam_app::QueueControlResult::Enqueued,
                "{} did not enqueue",
                fixture.name
            );

            let committed = shell.app.commit_ready_actions(
                fixture
                    .boundary
                    .as_ref()
                    .expect("scene restore boundary")
                    .to_commit_boundary_state(),
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

    assert_eq!(
        shell.app.jam_view.scene.active_scene.as_deref(),
        Some(fixture.expected.active_scene.as_str()),
        "{} active scene drifted",
        fixture.name
    );
    if let Some(expected_summary) = &fixture.expected.result_summary {
        assert_eq!(
            shell
                .app
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

    shell
}

#[test]
fn mc202_fixture_backed_shell_regressions_hold() {
    let fixtures: Vec<Mc202RegressionFixture> =
        serde_json::from_str(include_str!("../../tests/fixtures/mc202_regression.json"))
            .expect("parse MC-202 regression fixtures");

    for fixture in fixtures {
        let mut shell = mc202_committed_shell_state(&fixture);
        shell.active_screen = ShellScreen::Jam;
        let jam_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        for needle in &fixture.expected.jam_contains {
            assert!(
                jam_rendered.contains(needle),
                "{} jam snapshot missing {needle}\n{jam_rendered}",
                fixture.name,
                jam_rendered = jam_rendered
            );
        }

        shell.active_screen = ShellScreen::Log;
        let log_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        for needle in &fixture.expected.log_contains {
            assert!(
                log_rendered.contains(needle),
                "{} log snapshot missing {needle}",
                fixture.name
            );
        }
    }
}

#[test]
fn scene_fixture_backed_shell_regressions_hold() {
    let fixtures: Vec<SceneRegressionFixture> =
        serde_json::from_str(include_str!("../../tests/fixtures/scene_regression.json"))
            .expect("parse Scene Brain regression fixtures");

    for fixture in fixtures {
        let mut shell = scene_committed_shell_state(&fixture);
        shell.active_screen = ShellScreen::Jam;
        let jam_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        for needle in &fixture.expected.jam_contains {
            assert!(
                jam_rendered.contains(needle),
                "{} jam snapshot missing {needle}\n{jam_rendered}",
                fixture.name,
                jam_rendered = jam_rendered
            );
        }

        shell.active_screen = ShellScreen::Log;
        let log_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        for needle in &fixture.expected.log_contains {
            assert!(
                log_rendered.contains(needle),
                "{} log snapshot missing {needle}\n{log_rendered}",
                fixture.name,
                log_rendered = log_rendered
            );
        }
    }
}

fn w30_committed_shell_state(fixture: &W30RegressionFixture) -> JamShellState {
    let sample_shell = sample_shell_state();
    let mut session = sample_shell.app.session.clone();
    session.action_log.actions.clear();
    session.runtime_state.macro_state.w30_grit = fixture.initial_w30_grit.unwrap_or(0.0);
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
    session.captures[0].assigned_target =
        fixture
            .capture_assigned
            .then(|| riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: fixture.capture_bank.clone().into(),
                pad_id: fixture.capture_pad.clone().into(),
            });
    session.captures[0].is_pinned = fixture.capture_pinned;
    session.captures[0].source_window = fixture.source_window.as_ref().map(|source_window| {
        riotbox_core::session::CaptureSourceWindow {
            source_id: SourceId::from(source_window.source_id.clone()),
            start_seconds: source_window.start_seconds,
            end_seconds: source_window.end_seconds,
            start_frame: source_window.start_frame,
            end_frame: source_window.end_frame,
        }
    });
    for extra in &fixture.extra_captures {
        session.captures.push(riotbox_core::session::CaptureRef {
            capture_id: extra.capture_id.clone().into(),
            capture_type: riotbox_core::session::CaptureType::Pad,
            source_origin_refs: vec!["fixture-extra".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: format!("captures/{}.wav", extra.capture_id),
            assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: extra.bank.clone().into(),
                pad_id: extra.pad.clone().into(),
            }),
            is_pinned: extra.pinned,
            notes: extra.notes.clone(),
        });
    }

    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            session,
            sample_shell.app.source_graph.clone(),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );

    let queue_result = match fixture.action {
        W30RegressionAction::LiveRecall => shell.app.queue_w30_live_recall(fixture.requested_at),
        W30RegressionAction::RawCaptureAudition => {
            shell.app.queue_w30_audition(fixture.requested_at)
        }
        W30RegressionAction::PromotedAudition => {
            shell.app.queue_w30_promoted_audition(fixture.requested_at)
        }
        W30RegressionAction::TriggerPad => shell.app.queue_w30_trigger_pad(fixture.requested_at),
        W30RegressionAction::SwapBank => shell.app.queue_w30_swap_bank(fixture.requested_at),
        W30RegressionAction::ApplyDamageProfile => shell
            .app
            .queue_w30_apply_damage_profile(fixture.requested_at),
        W30RegressionAction::LoopFreeze => shell.app.queue_w30_loop_freeze(fixture.requested_at),
        W30RegressionAction::BrowseSlicePool => {
            shell.app.queue_w30_browse_slice_pool(fixture.requested_at)
        }
    };
    assert_eq!(
        queue_result,
        Some(crate::jam_app::QueueControlResult::Enqueued),
        "{} did not enqueue",
        fixture.name
    );

    let committed = shell.app.commit_ready_actions(
        fixture.boundary.to_commit_boundary_state(),
        fixture.committed_at,
    );
    assert_eq!(
        committed.len(),
        1,
        "{} did not commit exactly one action",
        fixture.name
    );

    shell
}

#[test]
fn w30_fixture_backed_shell_regressions_hold() {
    let fixtures: Vec<W30RegressionFixture> =
        serde_json::from_str(include_str!("../../tests/fixtures/w30_regression.json"))
            .expect("parse W-30 regression fixtures");

    for fixture in fixtures {
        let mut shell = w30_committed_shell_state(&fixture);
        shell.active_screen = ShellScreen::Jam;
        let jam_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        for needle in &fixture.expected.jam_contains {
            assert!(
                jam_rendered.contains(needle),
                "{} jam snapshot missing {needle}\n{jam_rendered}",
                fixture.name,
                jam_rendered = jam_rendered
            );
        }

        shell.active_screen = ShellScreen::Capture;
        let capture_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        for needle in &fixture.expected.capture_contains {
            assert!(
                capture_rendered.contains(needle),
                "{} capture snapshot missing {needle}\n{capture_rendered}",
                fixture.name,
                capture_rendered = capture_rendered
            );
        }

        shell.active_screen = ShellScreen::Log;
        let log_rendered = render_jam_shell_snapshot(&shell, 120, 34);
        for needle in &fixture.expected.log_contains {
            assert!(
                log_rendered.contains(needle),
                "{} log snapshot missing {needle}\n{log_rendered}",
                fixture.name,
                log_rendered = log_rendered
            );
        }
    }
}

#[test]
fn shell_state_handles_help_refresh_and_action_keys() {
    let mut shell = sample_shell_state();

    assert_eq!(
        shell.handle_key_code(KeyCode::Char('?')),
        ShellKeyOutcome::Continue
    );
    assert!(shell.show_help);
    assert_eq!(shell.status_message, "help overlay opened");

    assert_eq!(
        shell.handle_key_code(KeyCode::Char('r')),
        ShellKeyOutcome::RequestRefresh
    );
    assert_eq!(shell.status_message, "re-ingest source requested");
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('2')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.active_screen, ShellScreen::Log);
    assert_eq!(shell.status_message, "switched to log screen");
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('3')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.active_screen, ShellScreen::Source);
    assert_eq!(shell.status_message, "switched to source screen");
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('4')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.active_screen, ShellScreen::Capture);
    assert_eq!(shell.status_message, "switched to capture screen");
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('i')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(
        shell.status_message,
        "open Jam first if you want to use inspect"
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Tab),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.active_screen, ShellScreen::Jam);
    assert_eq!(shell.status_message, "switched to jam screen");
    assert_eq!(shell.jam_mode, JamViewMode::Perform);
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('i')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.jam_mode, JamViewMode::Inspect);
    assert_eq!(
        shell.status_message,
        "opened Jam inspect | press i to return to perform"
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('i')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.jam_mode, JamViewMode::Perform);
    assert_eq!(shell.status_message, "returned Jam to perform mode");
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('m')),
        ShellKeyOutcome::QueueSceneMutation
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('y')),
        ShellKeyOutcome::QueueSceneSelect
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('Y')),
        ShellKeyOutcome::QueueSceneRestore
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('b')),
        ShellKeyOutcome::QueueMc202RoleToggle
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('g')),
        ShellKeyOutcome::QueueMc202GenerateFollower
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('a')),
        ShellKeyOutcome::QueueMc202GenerateAnswer
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('P')),
        ShellKeyOutcome::QueueMc202GeneratePressure
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('I')),
        ShellKeyOutcome::QueueMc202GenerateInstigator
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('G')),
        ShellKeyOutcome::QueueMc202MutatePhrase
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('f')),
        ShellKeyOutcome::QueueTr909Fill
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('d')),
        ShellKeyOutcome::QueueTr909Reinforce
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('s')),
        ShellKeyOutcome::QueueTr909Slam
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('t')),
        ShellKeyOutcome::QueueTr909Takeover
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('k')),
        ShellKeyOutcome::QueueTr909SceneLock
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('x')),
        ShellKeyOutcome::QueueTr909Release
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('c')),
        ShellKeyOutcome::QueueCaptureBar
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('p')),
        ShellKeyOutcome::PromoteLastCapture
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('w')),
        ShellKeyOutcome::QueueW30TriggerPad
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('n')),
        ShellKeyOutcome::QueueW30StepFocus
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('B')),
        ShellKeyOutcome::QueueW30SwapBank
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('j')),
        ShellKeyOutcome::QueueW30BrowseSlicePool
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('D')),
        ShellKeyOutcome::QueueW30ApplyDamageProfile
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('z')),
        ShellKeyOutcome::QueueW30LoopFreeze
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('l')),
        ShellKeyOutcome::QueueW30LiveRecall
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('o')),
        ShellKeyOutcome::QueueW30Audition
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('e')),
        ShellKeyOutcome::QueueW30Resample
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('v')),
        ShellKeyOutcome::TogglePinLatestCapture
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('[')),
        ShellKeyOutcome::LowerDrumBusLevel
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char(']')),
        ShellKeyOutcome::RaiseDrumBusLevel
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('<')),
        ShellKeyOutcome::LowerMc202Touch
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('>')),
        ShellKeyOutcome::RaiseMc202Touch
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('u')),
        ShellKeyOutcome::UndoLast
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char(' ')),
        ShellKeyOutcome::ToggleTransport
    );

    assert_eq!(shell.handle_key_code(KeyCode::Esc), ShellKeyOutcome::Quit);
}

#[test]
fn first_run_shell_blocks_jam_inspect_toggle() {
    let mut shell = first_run_shell_state();

    assert_eq!(shell.jam_mode, JamViewMode::Perform);
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('i')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(shell.jam_mode, JamViewMode::Perform);
    assert_eq!(
        shell.status_message,
        "finish the first guided move before opening inspect"
    );
}

#[test]
fn renders_log_shell_snapshot_with_action_trust_history() {
    let mut shell = sample_shell_state();
    shell.active_screen = ShellScreen::Log;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("[2 Log]"));
    assert!(rendered.contains("Queued / Pending"));
    assert!(rendered.contains("Accepted / Committed"));
    assert!(rendered.contains("Rejected / Undone"));
    assert!(rendered.contains("MC-202 Lane"));
    assert!(rendered.contains("W-30 Lane"));
    assert!(rendered.contains("role leader"));
    assert!(rendered.contains("cue idle"));
    assert!(rendered.contains("cue idle | none"));
    assert!(rendered.contains("prev recall/fallback"));
    assert!(rendered.contains("mix 0.64/0.50 idle"));
    assert!(rendered.contains("cap cap-01 | pending"));
    assert!(rendered.contains("ghost"));
    assert!(rendered.contains("mutate.scene"));
    assert!(rendered.contains("TR-909 Render"));
    assert!(rendered.contains("accent off"));
    assert!(rendered.contains("takeover"));
    assert!(rendered.contains("scene lock blocked ghost"));
    assert!(rendered.contains("undid most recent musical"));
}

#[test]
fn renders_tr909_feral_support_reason_cue() {
    let mut shell = sample_shell_state();
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
        .reinforcement_mode = Some(Tr909ReinforcementModeState::SourceSupport);
    shell.app.session.runtime_state.lane_state.tr909.pattern_ref =
        Some("support-feral-break".into());
    shell.app.update_transport_clock(TransportClockState {
        is_playing: true,
        position_beats: 4.0,
        beat_index: 4,
        bar_index: 1,
        phrase_index: 1,
        current_scene: Some(SceneId::from("scene-a")),
    });
    shell.active_screen = ShellScreen::Log;

    assert_eq!(
        shell.app.runtime_view.tr909_render_support_reason,
        "feral break lift"
    );
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("feral break lift"), "{rendered}");
}

#[test]
fn renders_log_shell_snapshot_with_scene_brain_diagnostics() {
    let mut shell = sample_shell_state();
    assert_eq!(
        shell.app.queue_scene_select(300),
        crate::jam_app::QueueControlResult::Enqueued
    );
    shell.active_screen = ShellScreen::Log;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Counts"));
    assert!(rendered.contains("scene scene-a | medium"));
    assert!(rendered.contains("restore none"));
    assert!(rendered.contains("pending"));
}

#[test]
fn renders_source_shell_snapshot_with_feral_scorecard() {
    let mut shell = sample_shell_state();
    shell.active_screen = ShellScreen::Source;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("[3 Source]"));
    assert!(rendered.contains("Identity"));
    assert!(rendered.contains("Timing"));
    assert!(rendered.contains("Sections"));
    assert!(rendered.contains("Candidates"));
    assert!(rendered.contains("Provenance"));
    assert!(rendered.contains("Source Graph Warnings"));
    assert!(rendered.contains("feral ready"));
    assert!(rendered.contains("break high"));
    assert!(rendered.contains("quote risk 1"));
    assert!(rendered.contains("use capture before quoting"));
    assert!(rendered.contains("decoded.wav_baseline"));
    assert!(rendered.contains("fixtures/input.wav"));
    assert!(rendered.contains("wav_baseline_only"));
}

#[test]
fn renders_source_shell_snapshot_with_near_miss_feral_readiness() {
    let mut shell = sample_shell_state();
    let graph = shell
        .app
        .source_graph
        .as_mut()
        .expect("sample shell should include source graph");
    graph.relationships.retain(|relationship| {
        relationship.relation_type != RelationshipType::SupportsBreakRebuild
    });
    shell.app.refresh_view();
    shell.active_screen = ShellScreen::Source;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("feral needs support"), "{rendered}");
    assert!(rendered.contains("quote risk 1 | support 0"), "{rendered}");
    assert!(rendered.contains("hooks 1 | capture 1"), "{rendered}");
}

#[test]
fn renders_capture_shell_snapshot_with_capture_context() {
    let mut shell = sample_shell_state();
    shell.active_screen = ShellScreen::Capture;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("[4 Capture]"));
    assert!(rendered.contains("Readiness"));
    assert!(rendered.contains("Latest Capture"));
    assert!(rendered.contains("Do Next"));
    assert!(rendered.contains("Provenance"));
    assert!(rendered.contains("Pending Capture Cues"));
    assert!(rendered.contains("Recent Captures"));
    assert!(rendered.contains("Advanced Routing"));
    assert!(rendered.contains("cap-01"));
    assert!(rendered.contains("promote keeper capture"));
    assert!(rendered.contains("promotion result pending"));
    assert!(rendered.contains("captures total 1"));
    assert!(rendered.contains("pinned 0 | promoted 0"));
    assert!(
        rendered.contains("queued [p] promote @ next_bar"),
        "{rendered}"
    );
    assert!(
        rendered.contains("wait, then hear with [w] hit"),
        "{rendered}"
    );
    assert!(
        rendered.contains("target lanew30:bank-a/pad-01"),
        "{rendered}"
    );
    assert!(rendered.contains("pending W-30 cue idle"));
    assert!(
        rendered.contains("hear cap-01 stored fallback [o] raw"),
        "{rendered}"
    );
    assert!(rendered.contains("[p]->[w]"), "{rendered}");
    assert!(
        rendered.contains("forge idle | tap ready/raw"),
        "{rendered}"
    );
    assert!(rendered.contains("g0"), "{rendered}");
    assert!(rendered.contains("latest promoted none"));
}

#[test]
fn renders_capture_provenance_source_window_when_available() {
    let mut shell = sample_shell_state();
    shell.app.session.captures[0].source_window =
        Some(riotbox_core::session::CaptureSourceWindow {
            source_id: SourceId::from("src-1"),
            start_seconds: 1.25,
            end_seconds: 3.75,
            start_frame: 60_000,
            end_frame: 180_000,
        });
    shell.app.refresh_view();
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("win src-1 1.25-3.75s"), "{rendered}");
}

#[test]
fn renders_recent_capture_source_window_shorthand_when_available() {
    let mut shell = sample_shell_state();
    shell.app.session.captures[0].source_window =
        Some(riotbox_core::session::CaptureSourceWindow {
            source_id: SourceId::from("src-1"),
            start_seconds: 1.25,
            end_seconds: 3.75,
            start_frame: 60_000,
            end_frame: 180_000,
        });
    shell.app.refresh_view();
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("cap-01 | 1.25-3.75s"), "{rendered}");
}

#[test]
fn source_window_formatters_keep_surface_shapes_stable() {
    let source_window = riotbox_core::session::CaptureSourceWindow {
        source_id: SourceId::from("src-1"),
        start_seconds: 1.25,
        end_seconds: 3.75,
        start_frame: 60_000,
        end_frame: 180_000,
    };

    assert_eq!(format_source_window_span(&source_window), "1.25-3.75s");
    assert_eq!(
        format_source_window_log_compact(&source_window),
        "win 1.25-3.75s src-1"
    );
}

#[test]
fn renders_log_w30_source_window_when_available() {
    let mut shell = sample_shell_state();
    shell.app.session.captures[0].source_window =
        Some(riotbox_core::session::CaptureSourceWindow {
            source_id: SourceId::from("src-1"),
            start_seconds: 1.25,
            end_seconds: 3.75,
            start_frame: 60_000,
            end_frame: 180_000,
        });
    shell.active_screen = ShellScreen::Log;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("win 1.25-3.75s src-1"), "{rendered}");
    assert_eq!(w30_capture_log_compact(&shell), "win 1.25-3.75s src-1");
}

#[test]
fn renders_capture_do_next_with_pending_capture_state() {
    let first_run_shell = first_run_shell_state();
    let mut shell = JamShellState::new(first_run_shell.app, ShellLaunchMode::Load);
    shell.app.queue_capture_bar(240);
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("queued [c] capture @ next_phrase"),
        "{rendered}"
    );
    assert!(
        rendered.contains("then [o] audition raw or [p] promote"),
        "{rendered}"
    );
    assert!(rendered.contains("[2] confirm capture"), "{rendered}");
}

#[test]
fn renders_capture_shell_snapshot_with_raw_capture_audition_cue() {
    let mut shell = sample_shell_without_pending_queue();
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_audition(260),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 160, 34);

    assert!(rendered.contains("pending W-30 cue audition"), "{rendered}");
    assert!(rendered.contains("bank-a/pad-01"), "{rendered}");
    assert!(rendered.contains("queued [o] audition raw @"), "{rendered}");
    assert!(
        rendered.contains("wait, then hear raw preview"),
        "{rendered}"
    );
    assert!(
        rendered.contains("hear cap-01 stored fallback [o] raw or [p]->[w]"),
        "{rendered}"
    );

    shell.active_screen = ShellScreen::Log;
    let rendered_log = render_jam_shell_snapshot(&shell, 160, 34);
    assert!(
        rendered_log.contains("w30.audition_raw_capture"),
        "{rendered_log}"
    );
}

#[test]
fn committed_raw_capture_audition_surfaces_source_fallback_readiness() {
    let mut shell = sample_shell_without_pending_queue();
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_audition(260),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 33,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        320,
    );
    shell.active_screen = ShellScreen::Jam;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("current preview audition"), "{rendered}");
    assert!(rendered.contains("raw/fallback"), "{rendered}");

    shell.active_screen = ShellScreen::Capture;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(rendered.contains("| fallback"), "{rendered}");
}

#[test]
fn source_backed_raw_capture_audition_compact_label_uses_src_cue() {
    let mut shell = sample_shell_without_pending_queue();
    shell.app.session.runtime_state.lane_state.w30.preview_mode =
        Some(riotbox_core::session::W30PreviewModeState::RawCaptureAudition);
    shell.app.refresh_view();
    shell.app.runtime.w30_preview.source_window_preview =
        Some(riotbox_audio::w30::W30PreviewSampleWindow {
            source_start_frame: 0,
            source_end_frame: 64,
            sample_count: 64,
            samples: [0.0; riotbox_audio::w30::W30_PREVIEW_SAMPLE_WINDOW_LEN],
        });

    assert_eq!(w30_preview_mode_profile_compact(&shell), "audition raw/src");
    assert_eq!(w30_preview_source_readiness(&shell), Some("source-backed"));
}

#[test]
fn source_backed_promoted_and_recall_compact_labels_use_src_cue() {
    let mut shell = sample_shell_without_pending_queue();
    let sample_window = riotbox_audio::w30::W30PreviewSampleWindow {
        source_start_frame: 0,
        source_end_frame: 64,
        sample_count: 64,
        samples: [0.0; riotbox_audio::w30::W30_PREVIEW_SAMPLE_WINDOW_LEN],
    };

    shell.app.runtime.w30_preview.mode = W30PreviewRenderMode::PromotedAudition;
    shell.app.runtime.w30_preview.source_profile =
        Some(riotbox_audio::w30::W30PreviewSourceProfile::PromotedAudition);
    shell.app.runtime.w30_preview.source_window_preview = Some(sample_window.clone());

    assert_eq!(w30_preview_mode_profile_compact(&shell), "audition/src");
    assert_eq!(w30_preview_log_compact(&shell), "audition/src");
    assert_eq!(w30_preview_source_readiness(&shell), Some("source-backed"));

    shell.app.runtime.w30_preview.mode = W30PreviewRenderMode::LiveRecall;
    shell.app.runtime.w30_preview.source_profile =
        Some(riotbox_audio::w30::W30PreviewSourceProfile::PromotedRecall);
    shell.app.runtime.w30_preview.source_window_preview = Some(sample_window);

    assert_eq!(
        w30_preview_mode_profile_compact(&shell),
        "recall/promoted/src"
    );
    assert_eq!(w30_preview_log_compact(&shell), "recall/src");
    assert_eq!(w30_preview_source_readiness(&shell), Some("source-backed"));
}

#[test]
fn renders_capture_pending_cues_panel_as_first_item_with_log_overflow() {
    let first_run_shell = first_run_shell_state();
    let mut shell = JamShellState::new(first_run_shell.app, ShellLaunchMode::Load);
    shell.app.queue_capture_bar(240);
    shell.app.queue_capture_bar(241);

    let lines = pending_capture_lines(&shell);
    let rendered: Vec<String> = lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect();

    assert_eq!(rendered[0], "next user capture.bar_group");
    assert_eq!(rendered[1], "when next_phrase | target lanew30");
    assert_eq!(rendered[2], "note capture next phrase into W-30 path");
    assert_eq!(rendered[3], "+1 more in [2] Log");
    assert_eq!(rendered.len(), 4);
}

#[test]
fn renders_capture_shell_snapshot_with_w30_live_recall_cue() {
    let mut shell = sample_shell_state();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-03".into(),
        });
    shell.app.session.captures[0].is_pinned = true;
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_live_recall(200),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("pending W-30 cue"));
    assert!(rendered.contains("recall"));
}

#[test]
fn renders_capture_shell_snapshot_with_w30_trigger_cue() {
    let mut shell = sample_shell_state();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-a".into(),
            pad_id: "pad-01".into(),
        });
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_trigger_pad(205),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("pending W-30 cue"));
    assert!(rendered.contains("trigger"));
    assert!(rendered.contains("bank-a/pad-01"));
}

#[test]
fn renders_capture_shell_snapshot_with_w30_step_cue() {
    let mut shell = sample_shell_state();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-a".into(),
            pad_id: "pad-01".into(),
        });
    shell
        .app
        .session
        .captures
        .push(riotbox_core::session::CaptureRef {
            capture_id: "cap-02".into(),
            capture_type: riotbox_core::session::CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-04".into(),
            }),
            is_pinned: false,
            notes: Some("secondary".into()),
        });
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_step_focus(207),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("pending W-30 cue"));
    assert!(rendered.contains("step"));
    assert!(rendered.contains("bank-b/pad-04"));
}

#[test]
fn renders_capture_shell_snapshot_with_w30_bank_swap_cue() {
    let mut shell = sample_shell_state();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-a".into(),
            pad_id: "pad-01".into(),
        });
    shell
        .app
        .session
        .captures
        .push(riotbox_core::session::CaptureRef {
            capture_id: "cap-02".into(),
            capture_type: riotbox_core::session::CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-01".into(),
            }),
            is_pinned: false,
            notes: Some("bank b".into()),
        });
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_swap_bank(208),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("pending W-30 cue"));
    assert!(rendered.contains("bank"));
    assert!(rendered.contains("bank-b/pad-01"));
    assert!(rendered.contains("pending W-30 cue bank"), "{rendered}");
    assert!(rendered.contains("mgr next bank-b/pad-01"), "{rendered}");
}

#[test]
fn renders_capture_shell_snapshot_with_w30_slice_pool_browse_cue() {
    let mut shell = sample_shell_state();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-a".into(),
            pad_id: "pad-01".into(),
        });
    shell
        .app
        .session
        .captures
        .push(riotbox_core::session::CaptureRef {
            capture_id: "cap-02".into(),
            capture_type: riotbox_core::session::CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: vec!["cap-01".into()],
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-a".into(),
                pad_id: "pad-01".into(),
            }),
            is_pinned: false,
            notes: Some("alt slice".into()),
        });
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_browse_slice_pool(209),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("pending W-30 cue"));
    assert!(rendered.contains("browse"));
    assert!(rendered.contains("bank-a/pad-01"), "{rendered}");
    assert!(rendered.contains("bank/pad bank-a/pad-01"), "{rendered}");
    assert!(rendered.contains("pool cap-01 1/2 -> cap-02"), "{rendered}");
}

#[test]
fn renders_capture_shell_snapshot_with_feral_w30_slice_pool_browse_cue() {
    let mut shell = sample_shell_state();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-a".into(),
            pad_id: "pad-01".into(),
        });
    shell
        .app
        .session
        .captures
        .push(riotbox_core::session::CaptureRef {
            capture_id: "cap-02".into(),
            capture_type: riotbox_core::session::CaptureType::Pad,
            source_origin_refs: vec!["asset-hook".into()],
            source_window: None,
            lineage_capture_refs: vec!["cap-01".into()],
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-a".into(),
                pad_id: "pad-01".into(),
            }),
            is_pinned: false,
            notes: Some("feral hook slice".into()),
        });
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_browse_slice_pool(210),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    assert_eq!(
        shell
            .app
            .jam_view
            .lanes
            .w30_pending_slice_pool_reason
            .as_deref(),
        Some("feral")
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("pending W-30 cue feral") && rendered.contains("browse cap-02"),
        "{rendered}"
    );
    assert!(
        rendered.contains("pool cap-01 1/2 -> feral") && rendered.contains("cap-02"),
        "{rendered}"
    );
}

#[test]
fn renders_log_shell_snapshot_with_committed_w30_slice_pool_browse_diagnostics() {
    let mut shell = sample_shell_state();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-a".into(),
            pad_id: "pad-01".into(),
        });
    shell
        .app
        .session
        .captures
        .push(riotbox_core::session::CaptureRef {
            capture_id: "cap-02".into(),
            capture_type: riotbox_core::session::CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: vec!["cap-01".into()],
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-a".into(),
                pad_id: "pad-01".into(),
            }),
            is_pinned: false,
            notes: Some("alt slice".into()),
        });
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_browse_slice_pool(320),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.app.commit_ready_actions(
        riotbox_core::transport::CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Beat,
            beat_index: 42,
            bar_index: 11,
            phrase_index: 3,
            scene_id: Some("scene-1".into()),
        },
        420,
    );
    shell.active_screen = ShellScreen::Log;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("cue idle | browse"), "{rendered}");
    assert!(rendered.contains("bank bank-a/pad-01"), "{rendered}");
    assert!(rendered.contains("tap cap-02 g0/l1 int"), "{rendered}");
}

#[test]
fn renders_capture_shell_snapshot_with_w30_damage_profile_cue() {
    let mut shell = sample_shell_state();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-a".into(),
            pad_id: "pad-01".into(),
        });
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_apply_damage_profile(210),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("pending W-30 cue"));
    assert!(rendered.contains("damage"));
    assert!(rendered.contains("bank-a/pad-01"));
    assert!(rendered.contains("next bank-a/pad-01"), "{rendered}");
}

#[test]
fn renders_w30_bank_manager_and_damage_profile_diagnostics_across_shell_surfaces() {
    let mut shell = sample_shell_state();
    shell.app.queue = ActionQueue::new();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-a".into(),
            pad_id: "pad-01".into(),
        });
    shell
        .app
        .session
        .captures
        .push(riotbox_core::session::CaptureRef {
            capture_id: "cap-02".into(),
            capture_type: riotbox_core::session::CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-01".into(),
            }),
            is_pinned: false,
            notes: Some("bank b".into()),
        });
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-a".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();

    assert_eq!(
        shell.app.queue_w30_swap_bank(208),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 17,
            bar_index: 5,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-a")),
        },
        220,
    );
    assert_eq!(committed.len(), 1);

    assert_eq!(
        shell.app.queue_w30_apply_damage_profile(222),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 21,
            bar_index: 6,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-a")),
        },
        240,
    );
    assert_eq!(committed.len(), 1);

    let jam_rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(
        jam_rendered.contains("current pad bank-b/pad-01"),
        "{jam_rendered}"
    );
    assert!(jam_rendered.contains("next swap+shred"), "{jam_rendered}");

    shell.active_screen = ShellScreen::Capture;
    let capture_rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(
        capture_rendered.contains("mgr bank-b/pad-01"),
        "{capture_rendered}"
    );
    assert!(
        capture_rendered.contains("forge bank-b/pad-01"),
        "{capture_rendered}"
    );

    shell.active_screen = ShellScreen::Log;
    let log_rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(
        log_rendered.contains("bank bank-b/pad-01"),
        "{log_rendered}"
    );
    assert!(log_rendered.contains("cue idle | damage"), "{log_rendered}");
    assert!(log_rendered.contains("mix 0.64/0.82"), "{log_rendered}");
    assert!(log_rendered.contains("swap+shred"), "{log_rendered}");
}

#[test]
fn w30_operation_diagnostics_follow_current_lane_target() {
    let mut shell = sample_shell_state();
    shell.app.queue = ActionQueue::new();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-a".into(),
            pad_id: "pad-01".into(),
        });
    shell
        .app
        .session
        .captures
        .push(riotbox_core::session::CaptureRef {
            capture_id: "cap-02".into(),
            capture_type: riotbox_core::session::CaptureType::Pad,
            source_origin_refs: vec!["asset-b".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-02.wav".into(),
            assigned_target: Some(riotbox_core::session::CaptureTarget::W30Pad {
                bank_id: "bank-b".into(),
                pad_id: "pad-01".into(),
            }),
            is_pinned: false,
            notes: Some("bank b".into()),
        });
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-b".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-02".into());
    shell.app.refresh_view();

    assert_eq!(
        shell.app.queue_w30_swap_bank(208),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 17,
            bar_index: 5,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-a")),
        },
        220,
    );
    assert_eq!(committed.len(), 1);

    assert_eq!(
        shell.app.queue_w30_apply_damage_profile(222),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 21,
            bar_index: 6,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-a")),
        },
        240,
    );
    assert_eq!(committed.len(), 1);

    assert_eq!(
        shell.app.queue_w30_loop_freeze(245),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Phrase,
            beat_index: 29,
            bar_index: 8,
            phrase_index: 3,
            scene_id: Some(SceneId::from("scene-a")),
        },
        260,
    );
    assert_eq!(committed.len(), 1);

    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-c".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();

    let jam_rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(
        jam_rendered.contains("current pad bank-c/pad-01"),
        "{jam_rendered}"
    );
    assert!(jam_rendered.contains("next idle"), "{jam_rendered}");

    shell.active_screen = ShellScreen::Capture;
    let capture_rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(
        capture_rendered.contains("bank/pad bank-c/pad-01"),
        "{capture_rendered}"
    );
    assert!(capture_rendered.contains("mgr idle"), "{capture_rendered}");
    assert!(
        capture_rendered.contains("forge idle"),
        "{capture_rendered}"
    );
    assert!(
        capture_rendered.contains("freeze idle"),
        "{capture_rendered}"
    );

    shell.active_screen = ShellScreen::Log;
    let log_rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(
        log_rendered.contains("mix 0.64/0.82 idle"),
        "{log_rendered}"
    );
}

#[test]
fn renders_capture_shell_snapshot_with_w30_audition_cue() {
    let mut shell = sample_shell_without_pending_queue();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-03".into(),
        });
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_promoted_audition(210),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("pending W-30 cue"));
    assert!(rendered.contains("audition"));
    assert!(rendered.contains("[w]/[o]"), "{rendered}");
    assert!(rendered.contains("queued [o] audition pad @"), "{rendered}");
    assert!(
        rendered.contains("wait, then hear promoted preview"),
        "{rendered}"
    );
    assert_eq!(
        shell
            .app
            .jam_view
            .capture
            .latest_w30_promoted_capture_label
            .as_deref(),
        Some("cap-01 -> bank-b/pad-03")
    );
    assert!(rendered.contains("latest promoted cap-01 ->"), "{rendered}");
    assert!(rendered.contains("cap-01"));
}

#[test]
fn renders_capture_heard_path_for_scene_targets_without_w30_audition_keys() {
    let mut shell = sample_shell_without_pending_queue();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::Scene("drop-1".into()));
    shell.app.refresh_view();
    shell.active_screen = ShellScreen::Capture;

    assert_eq!(
        shell.app.jam_view.capture.last_capture_target_kind,
        Some(CaptureTargetKindView::Scene)
    );
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("hear cap-01->scene drop-1 ready"),
        "{rendered}"
    );
    assert!(rendered.contains("scene target scene drop-1"), "{rendered}");
}

#[test]
fn renders_capture_handoff_source_readiness_for_w30_targets() {
    let mut shell = sample_shell_without_pending_queue();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-03".into(),
        });
    shell.app.session.captures[0].source_window =
        Some(riotbox_core::session::CaptureSourceWindow {
            source_id: SourceId::from("src-1"),
            start_seconds: 1.25,
            end_seconds: 3.75,
            start_frame: 60_000,
            end_frame: 180_000,
        });
    shell.app.refresh_view();
    shell.active_screen = ShellScreen::Capture;

    assert_eq!(
        shell.app.jam_view.capture.last_capture_handoff_readiness,
        Some(CaptureHandoffReadinessView::Source)
    );
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("hear cap-01->pad bank-b/pad-03"),
        "{rendered}"
    );
    assert!(rendered.contains("[w]/[o] src"), "{rendered}");
    assert!(
        rendered.contains("hear now: [w] hit pad bank-b/pad-03"),
        "{rendered}"
    );
    assert!(rendered.contains("(src)"), "{rendered}");
}

#[test]
fn renders_capture_shell_snapshot_with_w30_resample_cue() {
    let mut shell = sample_shell_state();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-03".into(),
        });
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-b".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-03".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_internal_resample(215),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("pending W-30 cue"));
    assert!(rendered.contains("+1 more in [2] Log"));
    assert!(rendered.contains("resample"));
    assert!(rendered.contains("cap-01"));
}

#[test]
fn renders_capture_shell_snapshot_with_committed_w30_resample_lineage_diagnostics() {
    let mut shell = sample_shell_state();
    shell.app.queue = ActionQueue::new();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-03".into(),
        });
    shell.app.session.captures[0].lineage_capture_refs = vec!["cap-root".into()];
    shell.app.session.captures[0].resample_generation_depth = 1;
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-b".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-03".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_internal_resample(220),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Phrase,
            beat_index: 33,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-a")),
        },
        240,
    );
    assert_eq!(committed.len(), 1);
    shell.active_screen = ShellScreen::Capture;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("forge idle | tap ready/raw"),
        "{rendered}"
    );
    assert!(rendered.contains("g2"), "{rendered}");
    assert!(rendered.contains("lineage"));
    assert!(
        rendered.contains("cap-root>cap-01>cap-02 | g2"),
        "{rendered}"
    );
    assert!(rendered.contains("tap src cap-02 g2/l2 |"), "{rendered}");
    assert!(rendered.contains("route internal"), "{rendered}");
    assert!(rendered.contains("tap mix 0.64/0.50"), "{rendered}");
    assert!(
        rendered.matches("latest promoted").count() <= 1,
        "{rendered}"
    );
}

#[test]
fn renders_log_shell_snapshot_with_committed_w30_audition_diagnostics() {
    let mut shell = sample_shell_state();
    shell.app.queue = ActionQueue::new();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-03".into(),
        });
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_promoted_audition(220),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 33,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-a")),
        },
        240,
    );
    assert_eq!(committed.len(), 1);
    shell.active_screen = ShellScreen::Log;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("W-30 Lane"));
    assert!(rendered.contains("cue idle"));
    assert!(rendered.contains("auditioned cap-01"));
    assert!(rendered.contains("bank-b"));
    assert!(rendered.contains("pad-03"));
    assert!(rendered.contains("cue idle | audition"));
    assert!(rendered.contains("prev audition/fallback"));
    assert!(rendered.contains("mix 0.64/0.68"));
    assert!(rendered.contains("cap cap-01 | pending"), "{rendered}");
}

#[test]
fn renders_log_shell_snapshot_with_committed_w30_trigger_preview_diagnostics() {
    let mut shell = sample_shell_state();
    shell.app.queue = ActionQueue::new();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-a".into(),
            pad_id: "pad-01".into(),
        });
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_trigger_pad(230),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Beat,
            beat_index: 34,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-a")),
        },
        250,
    );
    assert_eq!(committed.len(), 1);
    shell.active_screen = ShellScreen::Log;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("W-30 Lane"));
    assert!(rendered.contains("cue idle | trigger"));
    assert!(rendered.contains("prev recall/fallback"));
    assert!(rendered.contains("mix 0.64/0.69"));
    assert!(rendered.contains("cap cap-01 | r1@0.84"), "{rendered}");
}

#[test]
fn renders_log_shell_snapshot_with_committed_w30_resample_lineage_diagnostics() {
    let mut shell = sample_shell_state();
    shell.app.queue = ActionQueue::new();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-03".into(),
        });
    shell.app.session.captures[0].lineage_capture_refs = vec!["cap-root".into()];
    shell.app.session.captures[0].resample_generation_depth = 1;
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-b".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-03".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();
    assert_eq!(
        shell.app.queue_w30_internal_resample(245),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Phrase,
            beat_index: 34,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-a")),
        },
        260,
    );
    assert_eq!(committed.len(), 1);
    shell.active_screen = ShellScreen::Log;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("W-30 Lane"));
    assert!(rendered.contains("cue idle | resample"));
    assert!(rendered.contains("tapmix 0.64/0.50"), "{rendered}");
    assert!(rendered.contains("tap cap-02 g2/l2 int"), "{rendered}");
}

#[test]
fn renders_w30_resample_lab_diagnostics_across_shell_surfaces() {
    let mut shell = sample_shell_state();
    shell.app.queue = ActionQueue::new();
    shell.app.session.captures[0].assigned_target =
        Some(riotbox_core::session::CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-03".into(),
        });
    shell.app.session.captures[0].lineage_capture_refs = vec!["cap-root".into()];
    shell.app.session.captures[0].resample_generation_depth = 1;
    shell.app.session.runtime_state.lane_state.w30.active_bank = Some("bank-b".into());
    shell.app.session.runtime_state.lane_state.w30.focused_pad = Some("pad-03".into());
    shell.app.session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    shell.app.refresh_view();

    assert_eq!(
        shell.app.queue_w30_internal_resample(265),
        Some(crate::jam_app::QueueControlResult::Enqueued)
    );
    let committed = shell.app.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Phrase,
            beat_index: 36,
            bar_index: 10,
            phrase_index: 3,
            scene_id: Some(SceneId::from("scene-a")),
        },
        280,
    );
    assert_eq!(committed.len(), 1);

    let jam_rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(
        jam_rendered.contains("current pad bank-b/pad-03"),
        "{jam_rendered}"
    );
    assert!(jam_rendered.contains("next idle"), "{jam_rendered}");

    shell.active_screen = ShellScreen::Capture;
    let capture_rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(
        capture_rendered.contains("tap src cap-02 g2/l2 |"),
        "{capture_rendered}"
    );
    assert!(
        capture_rendered.contains("route internal"),
        "{capture_rendered}"
    );
    assert!(
        capture_rendered.contains("tap mix 0.64/0.50"),
        "{capture_rendered}"
    );

    shell.active_screen = ShellScreen::Log;
    let log_rendered = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(
        log_rendered.contains("tap cap-02 g2/l2 int"),
        "{log_rendered}"
    );
    assert!(log_rendered.contains("tapmix 0.64/0.50"), "{log_rendered}");
}
