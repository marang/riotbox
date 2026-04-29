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

fn scene_jump_availability(
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
        current_scene.and_then(|scene_id| known_scene_energy_label(scene_id.as_str(), graph))
    else {
        return Some(SceneLaunchCandidateView {
            scene_id: candidate,
            reason: SceneLaunchTargetReason::Ordered,
        });
    };

    if let Some(contrast_candidate) = candidates.iter().copied().find(|candidate| {
        known_scene_energy_label(candidate.as_str(), graph)
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

fn known_scene_energy_label(scene_id: &str, graph: &SourceGraph) -> Option<String> {
    projected_scene_energy_label(Some(scene_id), false, graph).filter(|energy| energy != "unknown")
}

fn scene_movement_view(session: &SessionFile) -> Option<SceneMovementView> {
    let movement = session.runtime_state.scene_state.last_movement.as_ref()?;
    Some(SceneMovementView {
        kind: movement.kind.label().into(),
        direction: movement.direction.label().into(),
        tr909_intent: movement.tr909_intent.label().into(),
        mc202_intent: movement.mc202_intent.label().into(),
        intensity: movement.intensity,
        from_scene: movement.from_scene.as_ref().map(ToString::to_string),
        to_scene: movement.to_scene.to_string(),
        committed_bar_index: movement.committed_bar_index,
        committed_phrase_index: movement.committed_phrase_index,
    })
}

fn scene_transition_policy(
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

fn current_scene_energy_label(session: &SessionFile, graph: &SourceGraph) -> Option<String> {
    projected_scene_energy_label(
        session
            .runtime_state
            .scene_state
            .active_scene
            .as_ref()
            .or(session.runtime_state.transport.current_scene.as_ref())
            .map(|scene_id| scene_id.as_str()),
        true,
        graph,
    )
}

fn restore_scene_energy_label(session: &SessionFile, graph: &SourceGraph) -> Option<String> {
    projected_scene_energy_label(
        session
            .runtime_state
            .scene_state
            .restore_scene
            .as_ref()
            .map(|scene_id| scene_id.as_str()),
        false,
        graph,
    )
}

fn projected_scene_energy_label(
    scene_id: Option<&str>,
    fallback_to_first_section: bool,
    graph: &SourceGraph,
) -> Option<String> {
    let sections = sorted_sections(graph);
    let section = scene_id
        .and_then(parse_projected_scene_index)
        .and_then(|scene_index| sections.get(scene_index).copied())
        .or_else(|| {
            fallback_to_first_section
                .then(|| sections.first().copied())
                .flatten()
        })?;
    Some(section_energy_label(section).to_string())
}

fn parse_projected_scene_index(scene_id: &str) -> Option<usize> {
    let mut parts = scene_id.splitn(3, '-');
    match (parts.next(), parts.next()) {
        (Some("scene"), Some(index)) => index.parse::<usize>().ok()?.checked_sub(1),
        _ => None,
    }
}

fn sorted_sections(graph: &SourceGraph) -> Vec<&Section> {
    let mut sections = graph.sections.iter().collect::<Vec<_>>();
    sections.sort_by(|left, right| {
        left.bar_start
            .cmp(&right.bar_start)
            .then(left.bar_end.cmp(&right.bar_end))
            .then(left.section_id.as_str().cmp(right.section_id.as_str()))
    });
    sections
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

fn w30_pending_audition_view(
    action: &crate::action::Action,
    kind: W30PendingAuditionKind,
) -> Option<W30PendingAuditionView> {
    action
        .target
        .bank_id
        .as_ref()
        .zip(action.target.pad_id.as_ref())
        .map(|(bank_id, pad_id)| W30PendingAuditionView {
            kind,
            target: format!("{bank_id}/{pad_id}"),
            quantization: action.quantization.to_string(),
        })
}

#[derive(Clone, Debug, PartialEq)]
pub struct MacroStripView {
    pub source_retain: f32,
    pub chaos: f32,
    pub mc202_touch: f32,
    pub w30_grit: f32,
    pub tr909_slam: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneSummaryView {
    pub mc202_role: Option<String>,
    pub mc202_pending_role: Option<String>,
    pub mc202_pending_follower_generation: bool,
    pub mc202_pending_answer_generation: bool,
    pub mc202_pending_pressure_generation: bool,
    pub mc202_pending_instigator_generation: bool,
    pub mc202_pending_phrase_mutation: bool,
    pub mc202_phrase_ref: Option<String>,
    pub mc202_phrase_variant: Option<String>,
    pub w30_active_bank: Option<String>,
    pub w30_focused_pad: Option<String>,
    pub w30_pending_trigger_target: Option<String>,
    pub w30_pending_recall_target: Option<String>,
    pub w30_pending_audition: Option<W30PendingAuditionView>,
    pub w30_pending_audition_target: Option<String>,
    pub w30_pending_bank_swap_target: Option<String>,
    pub w30_pending_slice_pool_target: Option<String>,
    pub w30_pending_slice_pool_capture_id: Option<String>,
    pub w30_pending_slice_pool_reason: Option<String>,
    pub w30_pending_damage_profile_target: Option<String>,
    pub w30_pending_loop_freeze_target: Option<String>,
    pub w30_pending_focus_step_target: Option<String>,
    pub w30_pending_resample_capture_id: Option<String>,
    pub tr909_slam_enabled: bool,
    pub tr909_takeover_enabled: bool,
    pub tr909_takeover_pending_target: Option<bool>,
    pub tr909_takeover_pending_profile: Option<Tr909TakeoverProfileState>,
    pub tr909_takeover_profile: Option<Tr909TakeoverProfileState>,
    pub tr909_fill_armed_next_bar: bool,
    pub tr909_last_fill_bar: Option<u64>,
    pub tr909_reinforcement_mode: Option<Tr909ReinforcementModeState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct W30PendingAuditionView {
    pub kind: W30PendingAuditionKind,
    pub target: String,
    pub quantization: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum W30PendingAuditionKind {
    RawCapture,
    Promoted,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CaptureSummaryView {
    pub capture_count: usize,
    pub pinned_capture_count: usize,
    pub promoted_capture_count: usize,
    pub unassigned_capture_count: usize,
    pub pending_capture_count: usize,
    pub pending_capture_items: Vec<PendingCaptureActionView>,
    pub last_capture_id: Option<String>,
    pub last_capture_target: Option<String>,
    pub last_capture_target_kind: Option<CaptureTargetKindView>,
    pub last_capture_handoff_readiness: Option<CaptureHandoffReadinessView>,
    pub last_capture_origin_count: usize,
    pub last_capture_notes: Option<String>,
    pub last_promotion_result: Option<String>,
    pub latest_w30_promoted_capture_label: Option<String>,
    pub recent_capture_rows: Vec<String>,
    pub latest_capture_provenance_lines: Vec<String>,
    pub pinned_capture_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureTargetKindView {
    W30Pad,
    Scene,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureHandoffReadinessView {
    Source,
    Fallback,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PendingActionView {
    pub id: String,
    pub actor: String,
    pub command: String,
    pub quantization: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PendingCaptureActionView {
    pub id: String,
    pub actor: String,
    pub command: String,
    pub quantization: String,
    pub target: String,
    pub explanation: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentActionView {
    pub id: String,
    pub actor: String,
    pub command: String,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GhostStatusView {
    pub mode: String,
    pub suggestion_count: usize,
    pub is_blocked: bool,
}

