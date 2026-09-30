use crate::session::SessionFile;
use crate::source_graph::EnergyClass;
use crate::source_graph::Section;
use crate::source_graph::SourceGraph;
use crate::view::jam::arrangement_contract::ArrangementSceneContractView;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneJumpAvailabilityView {
    Ready,
    WaitingForMoreScenes,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneLaunchTargetReason {
    Ordered,
    EnergyContrast,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneLaunchCandidateView<'a> {
    pub scene_id: &'a crate::ids::SceneId,
    pub reason: SceneLaunchTargetReason,
}

pub(super) fn scene_jump_availability(
    session: &SessionFile,
    has_next_scene: bool,
) -> SceneJumpAvailabilityView {
    if has_next_scene {
        return SceneJumpAvailabilityView::Ready;
    }

    if session.runtime_state.scene_state.scenes.len() <= 1 {
        return SceneJumpAvailabilityView::WaitingForMoreScenes;
    }

    SceneJumpAvailabilityView::Unknown
}

pub fn next_scene_launch_candidate<'a>(
    session: &'a SessionFile,
    graph: Option<&SourceGraph>,
) -> Option<&'a crate::ids::SceneId> {
    next_scene_launch_candidate_with_reason(session, graph).map(|candidate| candidate.scene_id)
}

pub fn next_scene_launch_candidate_with_reason<'a>(
    session: &'a SessionFile,
    graph: Option<&SourceGraph>,
) -> Option<SceneLaunchCandidateView<'a>> {
    let scenes = &session.runtime_state.scene_state.scenes;
    if scenes.is_empty() {
        return None;
    }

    let current_scene = session
        .runtime_state
        .scene_state
        .active_scene
        .as_ref()
        .or(session.runtime_state.transport.current_scene.as_ref());

    let candidates = ordered_next_scene_candidates(scenes, current_scene);
    let candidate = *candidates.first()?;

    if scenes.len() <= 1 && current_scene == Some(candidate) {
        return None;
    }

    let Some(graph) = graph else {
        return Some(SceneLaunchCandidateView {
            scene_id: candidate,
            reason: SceneLaunchTargetReason::Ordered,
        });
    };
    let Some(current_energy) =
        current_scene.and_then(|scene_id| known_scene_energy_label(scene_id, session, graph))
    else {
        return Some(SceneLaunchCandidateView {
            scene_id: candidate,
            reason: SceneLaunchTargetReason::Ordered,
        });
    };

    if let Some(contrast_candidate) = candidates.iter().copied().find(|candidate| {
        known_scene_energy_label(candidate, session, graph)
            .is_some_and(|candidate_energy| candidate_energy != current_energy)
    }) {
        return Some(SceneLaunchCandidateView {
            scene_id: contrast_candidate,
            reason: if contrast_candidate == candidate {
                SceneLaunchTargetReason::Ordered
            } else {
                SceneLaunchTargetReason::EnergyContrast
            },
        });
    }

    Some(SceneLaunchCandidateView {
        scene_id: candidate,
        reason: SceneLaunchTargetReason::Ordered,
    })
}

fn ordered_next_scene_candidates<'a>(
    scenes: &'a [crate::ids::SceneId],
    current_scene: Option<&crate::ids::SceneId>,
) -> Vec<&'a crate::ids::SceneId> {
    let start_index = current_scene
        .and_then(|current_scene| scenes.iter().position(|scene_id| scene_id == current_scene))
        .map_or(0, |index| (index + 1) % scenes.len());

    (0..scenes.len())
        .map(|offset| &scenes[(start_index + offset) % scenes.len()])
        .collect()
}

fn known_scene_energy_label(
    scene_id: &crate::ids::SceneId,
    session: &SessionFile,
    graph: &SourceGraph,
) -> Option<String> {
    projected_scene_energy_label(Some(scene_id), false, session, graph)
        .filter(|energy| energy != "unknown")
}

pub(super) fn scene_movement_view(session: &SessionFile) -> Option<SceneMovementView> {
    let movement = session.runtime_state.scene_state.last_movement.as_ref()?;
    Some(SceneMovementView {
        kind: movement.kind.label().into(),
        direction: movement.direction.label().into(),
        tr909_intent: movement.tr909_intent.label().into(),
        mc202_intent: movement.mc202_intent.label().into(),
        w30_intent: movement.w30_intent.label().into(),
        intensity: movement.intensity,
        from_scene: movement.from_scene.as_ref().map(ToString::to_string),
        to_scene: movement.to_scene.to_string(),
        committed_bar_index: movement.committed_bar_index,
        committed_phrase_index: movement.committed_phrase_index,
    })
}

pub(super) fn scene_transition_policy(
    kind: SceneTransitionKindView,
    from_energy: Option<&str>,
    to_energy: Option<&str>,
) -> Option<SceneTransitionPolicyView> {
    let direction = scene_transition_direction(from_energy?, to_energy?)?;
    Some(SceneTransitionPolicyView {
        kind,
        direction,
        tr909_intent: tr909_transition_intent(direction),
        mc202_intent: mc202_transition_intent(direction),
        w30_intent: SceneTransitionW30IntentView::Pin,
        intensity: scene_transition_intensity(direction),
    })
}

fn scene_transition_direction(
    from_energy: &str,
    to_energy: &str,
) -> Option<SceneTransitionDirectionView> {
    let from = energy_rank(from_energy)?;
    let to = energy_rank(to_energy)?;

    Some(match to.cmp(&from) {
        std::cmp::Ordering::Greater => SceneTransitionDirectionView::Rise,
        std::cmp::Ordering::Less => SceneTransitionDirectionView::Drop,
        std::cmp::Ordering::Equal => SceneTransitionDirectionView::Hold,
    })
}

fn tr909_transition_intent(
    direction: SceneTransitionDirectionView,
) -> SceneTransitionLaneIntentView {
    match direction {
        SceneTransitionDirectionView::Rise => SceneTransitionLaneIntentView::Drive,
        SceneTransitionDirectionView::Drop => SceneTransitionLaneIntentView::Release,
        SceneTransitionDirectionView::Hold => SceneTransitionLaneIntentView::Anchor,
    }
}

fn mc202_transition_intent(
    direction: SceneTransitionDirectionView,
) -> SceneTransitionLaneIntentView {
    match direction {
        SceneTransitionDirectionView::Rise => SceneTransitionLaneIntentView::Lift,
        SceneTransitionDirectionView::Drop => SceneTransitionLaneIntentView::Anchor,
        SceneTransitionDirectionView::Hold => SceneTransitionLaneIntentView::Anchor,
    }
}

const fn scene_transition_intensity(direction: SceneTransitionDirectionView) -> f32 {
    match direction {
        SceneTransitionDirectionView::Rise => 0.75,
        SceneTransitionDirectionView::Drop => 0.55,
        SceneTransitionDirectionView::Hold => 0.35,
    }
}

fn energy_rank(label: &str) -> Option<u8> {
    match label {
        "low" => Some(0),
        "medium" => Some(1),
        "high" => Some(2),
        "peak" => Some(3),
        _ => None,
    }
}

pub(super) fn current_scene_energy_label(
    session: &SessionFile,
    graph: &SourceGraph,
) -> Option<String> {
    projected_scene_energy_label(
        session
            .runtime_state
            .scene_state
            .active_scene
            .as_ref()
            .or(session.runtime_state.transport.current_scene.as_ref()),
        true,
        session,
        graph,
    )
}

pub(super) fn restore_scene_energy_label(
    session: &SessionFile,
    graph: &SourceGraph,
) -> Option<String> {
    projected_scene_energy_label(
        session.runtime_state.scene_state.restore_scene.as_ref(),
        false,
        session,
        graph,
    )
}

pub(super) fn projected_scene_energy_label(
    scene_id: Option<&crate::ids::SceneId>,
    fallback_to_first_section: bool,
    session: &SessionFile,
    graph: &SourceGraph,
) -> Option<String> {
    let sections = crate::source_graph::sorted_sections(graph);
    let section = scene_id
        .and_then(|scene_id| {
            session
                .runtime_state
                .scene_state
                .source_section(graph, scene_id)
        })
        .or_else(|| {
            fallback_to_first_section
                .then(|| sections.first().copied())
                .flatten()
        })?;
    Some(section_energy_label(section).to_string())
}

const fn section_energy_label(section: &Section) -> &'static str {
    match section.energy_class {
        EnergyClass::Low => "low",
        EnergyClass::Medium => "medium",
        EnergyClass::High => "high",
        EnergyClass::Peak => "peak",
        EnergyClass::Unknown => "unknown",
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneSummaryView {
    pub active_scene: Option<String>,
    pub restore_scene: Option<String>,
    pub next_scene: Option<String>,
    pub scene_jump_availability: SceneJumpAvailabilityView,
    pub active_scene_energy: Option<String>,
    pub restore_scene_energy: Option<String>,
    pub next_scene_energy: Option<String>,
    pub next_scene_policy: Option<SceneTransitionPolicyView>,
    pub restore_scene_policy: Option<SceneTransitionPolicyView>,
    pub last_movement: Option<SceneMovementView>,
    pub arrangement_contract: ArrangementSceneContractView,
    pub scene_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneMovementView {
    pub kind: String,
    pub direction: String,
    pub tr909_intent: String,
    pub mc202_intent: String,
    pub w30_intent: String,
    pub intensity: f32,
    pub from_scene: Option<String>,
    pub to_scene: String,
    pub committed_bar_index: u64,
    pub committed_phrase_index: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneTransitionKindView {
    Launch,
    Restore,
}

impl SceneTransitionKindView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Launch => "launch",
            Self::Restore => "restore",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneTransitionDirectionView {
    Rise,
    Drop,
    Hold,
}

impl SceneTransitionDirectionView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Rise => "rise",
            Self::Drop => "drop",
            Self::Hold => "hold",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneTransitionLaneIntentView {
    Drive,
    Lift,
    Release,
    Anchor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneTransitionW30IntentView {
    Pin,
}

impl SceneTransitionW30IntentView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Pin => "pin",
        }
    }
}

impl SceneTransitionLaneIntentView {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Drive => "drive",
            Self::Lift => "lift",
            Self::Release => "release",
            Self::Anchor => "anchor",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneTransitionPolicyView {
    pub kind: SceneTransitionKindView,
    pub direction: SceneTransitionDirectionView,
    pub tr909_intent: SceneTransitionLaneIntentView,
    pub mc202_intent: SceneTransitionLaneIntentView,
    pub w30_intent: SceneTransitionW30IntentView,
    pub intensity: f32,
}
