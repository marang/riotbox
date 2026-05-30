#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArrangementSceneContractReadinessView {
    Ready,
    MissingSourceGraph,
    NeedsSceneMaterial,
    NeedsTimingEvidence,
    NeedsTimingConfirmation,
    FallbackTimingOnly,
}

impl ArrangementSceneContractReadinessView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::MissingSourceGraph => "missing_source_graph",
            Self::NeedsSceneMaterial => "needs_scene_material",
            Self::NeedsTimingEvidence => "needs_timing_evidence",
            Self::NeedsTimingConfirmation => "needs_timing_confirmation",
            Self::FallbackTimingOnly => "fallback_timing_only",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArrangementSceneTruthSourceView {
    ProductSpine,
}

impl ArrangementSceneTruthSourceView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ProductSpine => "source_graph_session_actions_queue_commit",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArrangementSceneActionSurfaceView {
    SceneLaunchRestore,
}

impl ArrangementSceneActionSurfaceView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SceneLaunchRestore => "scene.launch_scene.restore",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArrangementSceneContractView {
    pub readiness: ArrangementSceneContractReadinessView,
    pub truth_source: ArrangementSceneTruthSourceView,
    pub action_surface: ArrangementSceneActionSurfaceView,
    pub timing_readiness: SourceTimingConsumerReadiness,
    pub scene_count: usize,
    pub has_active_scene: bool,
    pub has_next_scene: bool,
    pub has_restore_scene: bool,
    pub has_pending_scene_transition: bool,
    pub has_landed_movement: bool,
    pub can_use_source_locked_scene_movement: bool,
    pub requires_p012_source_grid_gate: bool,
    pub requires_p013_musical_quality_gate: bool,
    pub requires_replay_state_proof: bool,
    pub requires_output_path_proof_for_audible_changes: bool,
}

#[must_use]
pub fn arrangement_scene_contract_view(
    session: &crate::session::SessionFile,
    queue: &crate::queue::ActionQueue,
    graph: Option<&crate::source_graph::SourceGraph>,
    scene_jump_availability: SceneJumpAvailabilityView,
    has_next_scene: bool,
) -> ArrangementSceneContractView {
    let timing_readiness = source_timing_consumer_readiness(graph, session);
    let readiness = arrangement_scene_contract_readiness(
        graph,
        timing_readiness,
        scene_jump_availability,
        has_next_scene,
    );

    ArrangementSceneContractView {
        readiness,
        truth_source: ArrangementSceneTruthSourceView::ProductSpine,
        action_surface: ArrangementSceneActionSurfaceView::SceneLaunchRestore,
        timing_readiness,
        scene_count: session.runtime_state.scene_state.scenes.len(),
        has_active_scene: session.runtime_state.scene_state.active_scene.is_some()
            || session.runtime_state.transport.current_scene.is_some(),
        has_next_scene,
        has_restore_scene: session.runtime_state.scene_state.restore_scene.is_some(),
        has_pending_scene_transition: queue.pending_actions().iter().any(|action| {
            matches!(
                action.command,
                crate::action::ActionCommand::SceneLaunch
                    | crate::action::ActionCommand::SceneRestore
            )
        }),
        has_landed_movement: session.runtime_state.scene_state.last_movement.is_some(),
        can_use_source_locked_scene_movement: timing_readiness.can_use_source_window_grid(),
        requires_p012_source_grid_gate: true,
        requires_p013_musical_quality_gate: true,
        requires_replay_state_proof: true,
        requires_output_path_proof_for_audible_changes: true,
    }
}

fn arrangement_scene_contract_readiness(
    graph: Option<&crate::source_graph::SourceGraph>,
    timing_readiness: SourceTimingConsumerReadiness,
    scene_jump_availability: SceneJumpAvailabilityView,
    has_next_scene: bool,
) -> ArrangementSceneContractReadinessView {
    if graph.is_none() {
        return ArrangementSceneContractReadinessView::MissingSourceGraph;
    }

    if !has_next_scene || scene_jump_availability != SceneJumpAvailabilityView::Ready {
        return ArrangementSceneContractReadinessView::NeedsSceneMaterial;
    }

    match timing_readiness {
        SourceTimingConsumerReadiness::AnalyzerLocked
        | SourceTimingConsumerReadiness::UserConfirmed => {
            ArrangementSceneContractReadinessView::Ready
        }
        SourceTimingConsumerReadiness::NeedsUserConfirmation => {
            ArrangementSceneContractReadinessView::NeedsTimingConfirmation
        }
        SourceTimingConsumerReadiness::FallbackGrid => {
            ArrangementSceneContractReadinessView::FallbackTimingOnly
        }
        SourceTimingConsumerReadiness::Unavailable => {
            ArrangementSceneContractReadinessView::NeedsTimingEvidence
        }
    }
}
