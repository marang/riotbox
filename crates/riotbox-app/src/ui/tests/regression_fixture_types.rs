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
