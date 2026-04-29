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

