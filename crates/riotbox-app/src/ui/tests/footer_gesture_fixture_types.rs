use crate::test_support::{scene_energy_for_label, scene_label_hint};
use riotbox_core::{
    TimestampMs,
    action::{
        Action, ActionCommand, ActionDraft, ActionParams, ActionResult, ActionStatus, ActionTarget,
        ActorType, GhostMode, Quantization, TargetScope, UndoPolicy,
    },
    ghost::{
        GhostSuggestedAction, GhostSuggestionConfidence, GhostSuggestionSafety,
        GhostWatchSuggestion, GhostWatchTool,
    },
    ids::{
        ActionId, AssetId, BankId, CaptureId, PadId, SceneId, SectionId, SnapshotId, SourceId,
    },
    queue::ActionQueue,
    session::{
        ActionCommitRecord, GhostSuggestionRecord, SceneMovementDirectionState,
        SceneMovementKindState, SceneMovementLaneIntentState, SceneMovementState, SessionFile,
        Snapshot, Tr909ReinforcementModeState, Tr909TakeoverProfileState,
    },
    source_graph::{
        AnalysisSummary, AnalysisWarning, Asset, AssetType, Candidate, CandidateType,
        DecodeProfile, EnergyClass, GraphProvenance, MeterHint, QualityClass, Relationship,
        RelationshipType, Section, SectionLabelHint, SourceDescriptor, SourceGraph,
        TimingDegradedPolicy, TimingHypothesis, TimingHypothesisKind, TimingQuality,
        TimingWarning, TimingWarningCode,
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
