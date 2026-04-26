use crate::{
    queue::ActionQueue,
    session::{
        Mc202PhraseVariantState, SessionFile, Tr909ReinforcementModeState,
        Tr909TakeoverProfileState,
    },
    source_graph::{EnergyClass, Section, SourceGraph},
};

#[cfg(test)]
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq)]
pub struct JamViewModel {
    pub transport: JamTransportView,
    pub source: SourceSummaryView,
    pub scene: SceneSummaryView,
    pub macros: MacroStripView,
    pub lanes: LaneSummaryView,
    pub capture: CaptureSummaryView,
    pub pending_actions: Vec<PendingActionView>,
    pub recent_actions: Vec<RecentActionView>,
    pub ghost: GhostStatusView,
    pub warnings: Vec<String>,
}

impl JamViewModel {
    #[must_use]
    pub fn build(session: &SessionFile, queue: &ActionQueue, graph: Option<&SourceGraph>) -> Self {
        let pending_actions = queue.pending_actions();
        let mc202_pending_role =
            pending_actions
                .iter()
                .rev()
                .find_map(|action| match action.command {
                    crate::action::ActionCommand::Mc202SetRole => action
                        .target
                        .object_id
                        .clone()
                        .or_else(|| match &action.params {
                            crate::action::ActionParams::Mutation { target_id, .. } => {
                                target_id.clone()
                            }
                            _ => None,
                        }),
                    _ => None,
                });
        let mc202_pending_follower_generation = pending_actions.iter().any(|action| {
            matches!(
                action.command,
                crate::action::ActionCommand::Mc202GenerateFollower
            )
        });
        let mc202_pending_answer_generation = pending_actions.iter().any(|action| {
            matches!(
                action.command,
                crate::action::ActionCommand::Mc202GenerateAnswer
            )
        });
        let mc202_pending_pressure_generation = pending_actions.iter().any(|action| {
            matches!(
                action.command,
                crate::action::ActionCommand::Mc202GeneratePressure
            )
        });
        let mc202_pending_phrase_mutation = pending_actions.iter().any(|action| {
            matches!(
                action.command,
                crate::action::ActionCommand::Mc202MutatePhrase
            )
        });
        let w30_pending_recall_target =
            pending_actions
                .iter()
                .rev()
                .find_map(|action| match action.command {
                    crate::action::ActionCommand::W30LiveRecall => action
                        .target
                        .bank_id
                        .as_ref()
                        .zip(action.target.pad_id.as_ref())
                        .map(|(bank_id, pad_id)| format!("{bank_id}/{pad_id}")),
                    _ => None,
                });
        let w30_pending_audition =
            pending_actions
                .iter()
                .rev()
                .find_map(|action| match action.command {
                    crate::action::ActionCommand::W30AuditionRawCapture => {
                        w30_pending_audition_view(action, W30PendingAuditionKind::RawCapture)
                    }
                    crate::action::ActionCommand::W30AuditionPromoted => {
                        w30_pending_audition_view(action, W30PendingAuditionKind::Promoted)
                    }
                    _ => None,
                });
        let w30_pending_audition_target = w30_pending_audition
            .as_ref()
            .map(|pending| pending.target.clone());
        let w30_pending_trigger_target =
            pending_actions
                .iter()
                .rev()
                .find_map(|action| match action.command {
                    crate::action::ActionCommand::W30TriggerPad => action
                        .target
                        .bank_id
                        .as_ref()
                        .zip(action.target.pad_id.as_ref())
                        .map(|(bank_id, pad_id)| format!("{bank_id}/{pad_id}")),
                    _ => None,
                });
        let w30_pending_bank_swap_target =
            pending_actions
                .iter()
                .rev()
                .find_map(|action| match action.command {
                    crate::action::ActionCommand::W30SwapBank => action
                        .target
                        .bank_id
                        .as_ref()
                        .zip(action.target.pad_id.as_ref())
                        .map(|(bank_id, pad_id)| format!("{bank_id}/{pad_id}")),
                    _ => None,
                });
        let w30_pending_slice_pool_target =
            pending_actions
                .iter()
                .rev()
                .find_map(|action| match action.command {
                    crate::action::ActionCommand::W30BrowseSlicePool => action
                        .target
                        .bank_id
                        .as_ref()
                        .zip(action.target.pad_id.as_ref())
                        .map(|(bank_id, pad_id)| format!("{bank_id}/{pad_id}")),
                    _ => None,
                });
        let w30_pending_slice_pool_capture_id =
            pending_actions
                .iter()
                .rev()
                .find_map(|action| match action.command {
                    crate::action::ActionCommand::W30BrowseSlicePool => match &action.params {
                        crate::action::ActionParams::Mutation {
                            target_id: Some(target_id),
                            ..
                        } => Some(target_id.clone()),
                        _ => None,
                    },
                    _ => None,
                });
        let w30_pending_damage_profile_target =
            pending_actions
                .iter()
                .rev()
                .find_map(|action| match action.command {
                    crate::action::ActionCommand::W30ApplyDamageProfile => action
                        .target
                        .bank_id
                        .as_ref()
                        .zip(action.target.pad_id.as_ref())
                        .map(|(bank_id, pad_id)| format!("{bank_id}/{pad_id}")),
                    _ => None,
                });
        let w30_pending_loop_freeze_target =
            pending_actions
                .iter()
                .rev()
                .find_map(|action| match action.command {
                    crate::action::ActionCommand::W30LoopFreeze => action
                        .target
                        .bank_id
                        .as_ref()
                        .zip(action.target.pad_id.as_ref())
                        .map(|(bank_id, pad_id)| format!("{bank_id}/{pad_id}")),
                    _ => None,
                });
        let w30_pending_focus_step_target =
            pending_actions
                .iter()
                .rev()
                .find_map(|action| match action.command {
                    crate::action::ActionCommand::W30StepFocus => action
                        .target
                        .bank_id
                        .as_ref()
                        .zip(action.target.pad_id.as_ref())
                        .map(|(bank_id, pad_id)| format!("{bank_id}/{pad_id}")),
                    _ => None,
                });
        let w30_pending_resample_capture_id =
            pending_actions
                .iter()
                .rev()
                .find_map(|action| match action.command {
                    crate::action::ActionCommand::PromoteResample
                        if action.target.scope == Some(crate::action::TargetScope::LaneW30) =>
                    {
                        match &action.params {
                            crate::action::ActionParams::Promotion {
                                capture_id: Some(capture_id),
                                ..
                            } => Some(capture_id.to_string()),
                            _ => Some("pending".into()),
                        }
                    }
                    _ => None,
                });
        let tr909_takeover_pending_target =
            pending_actions
                .iter()
                .rev()
                .find_map(|action| match action.command {
                    crate::action::ActionCommand::Tr909Takeover => Some(true),
                    crate::action::ActionCommand::Tr909SceneLock => Some(true),
                    crate::action::ActionCommand::Tr909Release => Some(false),
                    _ => None,
                });
        let tr909_takeover_pending_profile =
            pending_actions
                .iter()
                .rev()
                .find_map(|action| match action.command {
                    crate::action::ActionCommand::Tr909Takeover => {
                        Some(Tr909TakeoverProfileState::ControlledPhraseTakeover)
                    }
                    crate::action::ActionCommand::Tr909SceneLock => {
                        Some(Tr909TakeoverProfileState::SceneLockTakeover)
                    }
                    crate::action::ActionCommand::Tr909Release => None,
                    _ => None,
                });
        let tr909_fill_pending = pending_actions
            .iter()
            .any(|action| matches!(action.command, crate::action::ActionCommand::Tr909FillNext));
        let pending_capture_count = pending_actions
            .iter()
            .filter(|action| is_capture_command(action))
            .count();
        let pending_capture_items = pending_actions
            .iter()
            .filter(|action| is_capture_command(action))
            .take(4)
            .map(|action| PendingCaptureActionView {
                id: action.id.to_string(),
                actor: action.actor.to_string(),
                command: action.command.to_string(),
                quantization: action.quantization.to_string(),
                target: capture_action_target_label(action),
                explanation: action.explanation.clone(),
            })
            .collect();
        let pending_actions: Vec<PendingActionView> = pending_actions
            .into_iter()
            .map(|action| PendingActionView {
                id: action.id.to_string(),
                actor: action.actor.to_string(),
                command: action.command.to_string(),
                quantization: action.quantization.to_string(),
            })
            .collect();

        let recent_actions = session
            .action_log
            .actions
            .iter()
            .rev()
            .take(5)
            .map(|action| RecentActionView {
                id: action.id.to_string(),
                actor: action.actor.to_string(),
                command: action.command.to_string(),
                status: format!("{:?}", action.status).to_lowercase(),
            })
            .collect();

        let source = match graph {
            Some(graph) => SourceSummaryView {
                source_id: graph.source.source_id.to_string(),
                bpm_estimate: graph.timing.bpm_estimate,
                bpm_confidence: graph.timing.bpm_confidence,
                section_count: graph.sections.len(),
                loop_candidate_count: graph.loop_candidate_count(),
                hook_candidate_count: graph.hook_candidate_count(),
            },
            None => SourceSummaryView::default(),
        };

        let mut warnings = graph.map_or_else(Vec::new, SourceGraph::warnings);
        if pending_actions.is_empty() && !session.transport().is_playing {
            warnings.push("transport idle".into());
        }

        let next_scene = next_scene_launch_candidate(session, graph).map(ToString::to_string);
        let scene_jump_availability =
            scene_jump_availability(session, next_scene.as_deref().is_some());
        let next_scene_energy = graph
            .and_then(|graph| projected_scene_energy_label(next_scene.as_deref(), false, graph));

        Self {
            transport: JamTransportView {
                is_playing: session.transport().is_playing,
                position_beats: session.transport().position_beats,
            },
            source,
            scene: SceneSummaryView {
                active_scene: session
                    .runtime_state
                    .scene_state
                    .active_scene
                    .as_ref()
                    .map(ToString::to_string),
                restore_scene: session
                    .runtime_state
                    .scene_state
                    .restore_scene
                    .as_ref()
                    .map(ToString::to_string),
                next_scene,
                scene_jump_availability,
                active_scene_energy: graph
                    .and_then(|graph| current_scene_energy_label(session, graph)),
                restore_scene_energy: graph
                    .and_then(|graph| restore_scene_energy_label(session, graph)),
                next_scene_energy,
                scene_count: session.runtime_state.scene_state.scenes.len(),
            },
            macros: MacroStripView {
                source_retain: session.runtime_state.macro_state.source_retain,
                chaos: session.runtime_state.macro_state.chaos,
                mc202_touch: session.runtime_state.macro_state.mc202_touch,
                w30_grit: session.runtime_state.macro_state.w30_grit,
                tr909_slam: session.runtime_state.macro_state.tr909_slam,
            },
            lanes: LaneSummaryView {
                mc202_role: session.runtime_state.lane_state.mc202.role.clone(),
                mc202_pending_role,
                mc202_pending_follower_generation,
                mc202_pending_answer_generation,
                mc202_pending_pressure_generation,
                mc202_pending_phrase_mutation,
                mc202_phrase_ref: session.runtime_state.lane_state.mc202.phrase_ref.clone(),
                mc202_phrase_variant: session
                    .runtime_state
                    .lane_state
                    .mc202
                    .phrase_variant
                    .map(Mc202PhraseVariantState::label)
                    .map(str::to_string),
                w30_active_bank: session
                    .runtime_state
                    .lane_state
                    .w30
                    .active_bank
                    .as_ref()
                    .map(ToString::to_string),
                w30_focused_pad: session
                    .runtime_state
                    .lane_state
                    .w30
                    .focused_pad
                    .as_ref()
                    .map(ToString::to_string),
                w30_pending_trigger_target,
                w30_pending_recall_target,
                w30_pending_audition,
                w30_pending_audition_target,
                w30_pending_bank_swap_target,
                w30_pending_slice_pool_target,
                w30_pending_slice_pool_capture_id,
                w30_pending_damage_profile_target,
                w30_pending_loop_freeze_target,
                w30_pending_focus_step_target,
                w30_pending_resample_capture_id,
                tr909_slam_enabled: session.runtime_state.lane_state.tr909.slam_enabled,
                tr909_takeover_enabled: session.runtime_state.lane_state.tr909.takeover_enabled,
                tr909_takeover_pending_target,
                tr909_takeover_pending_profile,
                tr909_takeover_profile: session.runtime_state.lane_state.tr909.takeover_profile,
                tr909_fill_armed_next_bar: tr909_fill_pending,
                tr909_last_fill_bar: session.runtime_state.lane_state.tr909.last_fill_bar,
                tr909_reinforcement_mode: session.runtime_state.lane_state.tr909.reinforcement_mode,
            },
            capture: CaptureSummaryView {
                capture_count: session.captures.len(),
                pinned_capture_count: session
                    .captures
                    .iter()
                    .filter(|capture| capture.is_pinned)
                    .count(),
                promoted_capture_count: session
                    .captures
                    .iter()
                    .filter(|capture| capture.assigned_target.is_some())
                    .count(),
                unassigned_capture_count: session
                    .captures
                    .iter()
                    .filter(|capture| capture.assigned_target.is_none())
                    .count(),
                pending_capture_count,
                pending_capture_items,
                last_capture_id: session
                    .captures
                    .last()
                    .map(|capture| capture.capture_id.to_string()),
                last_capture_target: session.captures.last().and_then(|capture| {
                    capture.assigned_target.as_ref().map(|target| match target {
                        crate::session::CaptureTarget::W30Pad { bank_id, pad_id } => {
                            format!("pad {bank_id}/{pad_id}")
                        }
                        crate::session::CaptureTarget::Scene(scene_id) => {
                            format!("scene {scene_id}")
                        }
                    })
                }),
                last_capture_target_kind: session.captures.last().and_then(|capture| {
                    capture
                        .assigned_target
                        .as_ref()
                        .map(capture_target_kind_view)
                }),
                last_capture_handoff_readiness: session
                    .captures
                    .last()
                    .map(capture_handoff_readiness_view),
                last_capture_origin_count: session
                    .captures
                    .last()
                    .map_or(0, |capture| capture.source_origin_refs.len()),
                last_capture_notes: session
                    .captures
                    .last()
                    .and_then(|capture| capture.notes.clone()),
                last_promotion_result: session.captures.last().and_then(|capture| {
                    capture.assigned_target.as_ref().map(|target| match target {
                        crate::session::CaptureTarget::W30Pad { bank_id, pad_id } => {
                            format!("promoted to pad {bank_id}/{pad_id}")
                        }
                        crate::session::CaptureTarget::Scene(scene_id) => {
                            format!("promoted to scene {scene_id}")
                        }
                    })
                }),
                latest_w30_promoted_capture_label: latest_w30_promoted_capture_label(session),
                recent_capture_rows: recent_capture_rows(session),
                latest_capture_provenance_lines: latest_capture_provenance_lines(session),
                pinned_capture_ids: session
                    .captures
                    .iter()
                    .filter(|capture| capture.is_pinned)
                    .rev()
                    .take(4)
                    .map(|capture| capture.capture_id.to_string())
                    .collect(),
            },
            pending_actions,
            recent_actions,
            ghost: GhostStatusView {
                mode: session.ghost_state.mode.to_string(),
                suggestion_count: session.ghost_state.suggestion_history.len(),
                is_blocked: session
                    .runtime_state
                    .lock_state
                    .locked_object_ids
                    .iter()
                    .any(|lock| lock.contains("ghost")),
            },
            warnings,
        }
    }
}

fn is_capture_command(action: &crate::action::Action) -> bool {
    matches!(
        action.command,
        crate::action::ActionCommand::CaptureNow
            | crate::action::ActionCommand::CaptureLoop
            | crate::action::ActionCommand::CaptureBarGroup
            | crate::action::ActionCommand::W30CaptureToPad
            | crate::action::ActionCommand::PromoteCaptureToPad
            | crate::action::ActionCommand::PromoteCaptureToScene
            | crate::action::ActionCommand::W30LoopFreeze
            | crate::action::ActionCommand::PromoteResample
    )
}

fn capture_action_target_label(action: &crate::action::Action) -> String {
    match action.target.scope {
        Some(crate::action::TargetScope::LaneW30) => {
            if let (Some(bank_id), Some(pad_id)) = (
                action.target.bank_id.as_ref(),
                action.target.pad_id.as_ref(),
            ) {
                format!("lanew30:{bank_id}/{pad_id}")
            } else {
                "lanew30".into()
            }
        }
        Some(crate::action::TargetScope::Scene) => action
            .target
            .scene_id
            .as_ref()
            .map_or_else(|| "scene".into(), |scene_id| format!("scene:{scene_id}")),
        Some(crate::action::TargetScope::LaneMc202) => {
            action.target.object_id.as_ref().map_or_else(
                || "lanemc202".into(),
                |object_id| format!("lanemc202:{object_id}"),
            )
        }
        Some(crate::action::TargetScope::LaneTr909) => "lanetr909".into(),
        Some(crate::action::TargetScope::Global) => "global".into(),
        Some(crate::action::TargetScope::Mixer) => "mixer".into(),
        Some(crate::action::TargetScope::Ghost) => "ghost".into(),
        Some(crate::action::TargetScope::Session) | None => "session".into(),
    }
}

const fn capture_target_kind_view(target: &crate::session::CaptureTarget) -> CaptureTargetKindView {
    match target {
        crate::session::CaptureTarget::W30Pad { .. } => CaptureTargetKindView::W30Pad,
        crate::session::CaptureTarget::Scene(_) => CaptureTargetKindView::Scene,
    }
}

const fn capture_handoff_readiness_view(
    capture: &crate::session::CaptureRef,
) -> CaptureHandoffReadinessView {
    if capture.source_window.is_some() {
        CaptureHandoffReadinessView::Source
    } else {
        CaptureHandoffReadinessView::Fallback
    }
}

fn latest_w30_promoted_capture_label(session: &SessionFile) -> Option<String> {
    session
        .captures
        .iter()
        .rev()
        .find_map(|capture| match capture.assigned_target.as_ref() {
            Some(crate::session::CaptureTarget::W30Pad { bank_id, pad_id }) => {
                Some(format!("{} -> {bank_id}/{pad_id}", capture.capture_id))
            }
            _ => None,
        })
}

fn recent_capture_rows(session: &SessionFile) -> Vec<String> {
    session
        .captures
        .iter()
        .rev()
        .take(5)
        .map(|capture| {
            if let Some(source_window) = &capture.source_window {
                return format!(
                    "{} | {}{}",
                    capture.capture_id,
                    format_source_window_span(source_window),
                    if capture.is_pinned { " | pinned" } else { "" }
                );
            }

            let target = capture
                .assigned_target
                .as_ref()
                .map_or_else(|| "unassigned".into(), capture_recent_target_label);

            format!(
                "{} | {} | {} origins{}",
                capture.capture_id,
                target,
                capture.source_origin_refs.len(),
                if capture.is_pinned { " | pinned" } else { "" }
            )
        })
        .collect()
}

fn latest_capture_provenance_lines(session: &SessionFile) -> Vec<String> {
    let Some(capture) = session.captures.last() else {
        return Vec::new();
    };

    let mut lines = vec![
        format!("file {}", capture.storage_path),
        format!(
            "from action {}",
            capture
                .created_from_action
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "manual or unknown".into())
        ),
        format!(
            "origins {}",
            if capture.source_origin_refs.is_empty() {
                "none".into()
            } else {
                capture.source_origin_refs.join(", ")
            }
        ),
    ];

    if let Some(source_window) = &capture.source_window {
        lines.push(format_source_window_provenance(source_window));
    }

    lines
}

fn capture_recent_target_label(target: &crate::session::CaptureTarget) -> String {
    match target {
        crate::session::CaptureTarget::W30Pad { bank_id, pad_id } => {
            format!("{bank_id}/{pad_id}")
        }
        crate::session::CaptureTarget::Scene(scene_id) => scene_id.to_string(),
    }
}

fn format_source_window_span(source_window: &crate::session::CaptureSourceWindow) -> String {
    format!(
        "{:.2}-{:.2}s",
        source_window.start_seconds, source_window.end_seconds
    )
}

fn format_source_window_provenance(source_window: &crate::session::CaptureSourceWindow) -> String {
    format!(
        "win {} {}",
        source_window.source_id,
        format_source_window_span(source_window)
    )
}

trait SessionAccessors {
    fn transport(&self) -> &crate::session::TransportRuntimeState;
}

impl SessionAccessors for SessionFile {
    fn transport(&self) -> &crate::session::TransportRuntimeState {
        &self.runtime_state.transport
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct JamTransportView {
    pub is_playing: bool,
    pub position_beats: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SourceSummaryView {
    pub source_id: String,
    pub bpm_estimate: Option<f32>,
    pub bpm_confidence: f32,
    pub section_count: usize,
    pub loop_candidate_count: usize,
    pub hook_candidate_count: usize,
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
    pub scene_count: usize,
}

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

#[cfg(test)]
mod tests {
    use crate::{
        action::{
            ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType, GhostMode,
            Quantization, TargetScope, UndoPolicy,
        },
        ids::{BankId, SceneId, SourceId},
        queue::ActionQueue,
        session::{ActionLog, GhostSuggestionRecord, RuntimeState, SessionFile, SourceGraphRef},
        source_graph::{
            AnalysisSummary, Asset, AssetType, Candidate, CandidateType, DecodeProfile,
            GraphProvenance, QualityClass, SourceDescriptor, SourceGraph,
        },
    };

    use super::*;

    fn sample_graph_with_sections(section_labels: &[String]) -> SourceGraph {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: "src-1".into(),
                path: "input.wav".into(),
                content_hash: "hash-1".into(),
                duration_seconds: 120.0,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "0.1.0".into(),
                provider_set: vec!["beat".into(), "section".into()],
                generated_at: "2026-04-12T18:00:00Z".into(),
                source_hash: "hash-1".into(),
                analysis_seed: 7,
                run_notes: Some("scene-energy-projection-fixture".into()),
            },
        );

        for (index, label) in section_labels.iter().enumerate() {
            let bar_start = (index as u32 * 8) + 1;
            graph.sections.push(crate::source_graph::Section {
                section_id: format!("section-{index}").into(),
                label_hint: match label.as_str() {
                    "intro" => crate::source_graph::SectionLabelHint::Intro,
                    "break" => crate::source_graph::SectionLabelHint::Break,
                    "build" => crate::source_graph::SectionLabelHint::Build,
                    "drop" => crate::source_graph::SectionLabelHint::Drop,
                    "verse" => crate::source_graph::SectionLabelHint::Verse,
                    "chorus" => crate::source_graph::SectionLabelHint::Chorus,
                    "bridge" => crate::source_graph::SectionLabelHint::Bridge,
                    "outro" => crate::source_graph::SectionLabelHint::Outro,
                    _ => crate::source_graph::SectionLabelHint::Unknown,
                },
                start_seconds: index as f32 * 16.0,
                end_seconds: (index + 1) as f32 * 16.0,
                bar_start,
                bar_end: bar_start + 7,
                energy_class: fixture_energy_for_label(label),
                confidence: 0.9,
                tags: vec![label.clone()],
            });
        }

        graph
    }

    fn fixture_energy_for_label(label: &str) -> crate::source_graph::EnergyClass {
        match label {
            "drop" | "chorus" => crate::source_graph::EnergyClass::High,
            "break" | "outro" => crate::source_graph::EnergyClass::Low,
            "intro" | "build" | "verse" | "bridge" => crate::source_graph::EnergyClass::Medium,
            _ => crate::source_graph::EnergyClass::Unknown,
        }
    }

    #[test]
    fn builds_minimal_jam_view_model() {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: SourceId::from("src-1"),
                path: "input.wav".into(),
                content_hash: "hash-1".into(),
                duration_seconds: 120.0,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "0.1.0".into(),
                provider_set: vec!["beat".into(), "section".into()],
                generated_at: "2026-04-12T18:00:00Z".into(),
                source_hash: "hash-1".into(),
                analysis_seed: 7,
                run_notes: None,
            },
        );
        graph.sections.push(crate::source_graph::Section {
            section_id: "sec-a".into(),
            label_hint: crate::source_graph::SectionLabelHint::Drop,
            start_seconds: 0.0,
            end_seconds: 16.0,
            bar_start: 1,
            bar_end: 8,
            energy_class: crate::source_graph::EnergyClass::High,
            confidence: 0.9,
            tags: vec![],
        });
        graph.assets.push(Asset {
            asset_id: "asset-a".into(),
            asset_type: AssetType::LoopWindow,
            start_seconds: 0.0,
            end_seconds: 4.0,
            start_bar: 1,
            end_bar: 2,
            confidence: 0.8,
            tags: vec![],
            source_refs: vec![],
        });
        graph.candidates.push(Candidate {
            candidate_id: "cand-loop".into(),
            candidate_type: CandidateType::LoopCandidate,
            asset_ref: "asset-a".into(),
            score: 0.9,
            confidence: 0.9,
            tags: vec![],
            constraints: vec![],
            provenance_refs: vec![],
        });
        graph.candidates.push(Candidate {
            candidate_id: "cand-hook".into(),
            candidate_type: CandidateType::HookCandidate,
            asset_ref: "asset-a".into(),
            score: 0.7,
            confidence: 0.8,
            tags: vec![],
            constraints: vec![],
            provenance_refs: vec![],
        });
        graph.analysis_summary = AnalysisSummary {
            overall_confidence: 0.85,
            timing_quality: QualityClass::High,
            section_quality: QualityClass::Medium,
            loop_candidate_count: 1,
            hook_candidate_count: 1,
            break_rebuild_potential: QualityClass::High,
            warnings: vec![],
        };

        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-12T18:00:00Z");
        session.runtime_state.transport.is_playing = true;
        session.runtime_state.transport.position_beats = 16.0;
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-1"));
        session.runtime_state.scene_state.scenes = vec![SceneId::from("scene-1")];
        session.runtime_state.lane_state.mc202.role = Some("follower".into());
        session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
        session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
        session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
        session.runtime_state.lane_state.tr909.takeover_enabled = true;
        session.runtime_state.lane_state.tr909.takeover_profile =
            Some(Tr909TakeoverProfileState::SceneLockTakeover);
        session.runtime_state.lane_state.tr909.slam_enabled = true;
        session.runtime_state.lane_state.tr909.last_fill_bar = Some(8);
        session.runtime_state.lane_state.tr909.reinforcement_mode =
            Some(Tr909ReinforcementModeState::Takeover);
        session.ghost_state.mode = GhostMode::Assist;
        session.ghost_state.suggestion_history = vec![GhostSuggestionRecord {
            proposal_id: "gp-1".into(),
            summary: "capture next bar".into(),
            accepted: false,
        }];
        session.action_log = ActionLog {
            actions: vec![],
            replay_policy: crate::session::ReplayPolicy::DeterministicPreferred,
        };
        session.source_graph_refs = vec![SourceGraphRef {
            source_id: SourceId::from("src-1"),
            graph_version: crate::source_graph::SourceGraphVersion::V1,
            graph_hash: "graph-1".into(),
            storage_mode: crate::session::GraphStorageMode::Embedded,
            embedded_graph: Some(graph.clone()),
            external_path: None,
            provenance: graph.provenance.clone(),
        }];
        session.runtime_state = RuntimeState {
            transport: session.runtime_state.transport.clone(),
            macro_state: session.runtime_state.macro_state.clone(),
            lane_state: session.runtime_state.lane_state.clone(),
            mixer_state: session.runtime_state.mixer_state.clone(),
            scene_state: session.runtime_state.scene_state.clone(),
            lock_state: session.runtime_state.lock_state.clone(),
            pending_policy: session.runtime_state.pending_policy.clone(),
        };
        session.captures.push(crate::session::CaptureRef {
            capture_id: "cap-01".into(),
            capture_type: crate::session::CaptureType::Pad,
            source_origin_refs: vec!["asset-a".into(), "src-1".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-01.wav".into(),
            assigned_target: Some(crate::session::CaptureTarget::W30Pad {
                bank_id: "bank-a".into(),
                pad_id: "pad-01".into(),
            }),
            is_pinned: false,
            notes: Some("keeper capture".into()),
        });

        let mut queue = ActionQueue::new();
        let mut draft = ActionDraft::new(
            ActorType::Ghost,
            ActionCommand::CaptureNow,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::LaneW30),
                ..Default::default()
            },
        );
        draft.undo_policy = UndoPolicy::Undoable;
        draft.explanation = Some("capture current break".into());
        queue.enqueue(draft, 100);
        queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::Mc202SetRole,
                Quantization::NextPhrase,
                ActionTarget {
                    scope: Some(TargetScope::LaneMc202),
                    object_id: Some("leader".into()),
                    ..Default::default()
                },
            ),
            101,
        );
        queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::W30LiveRecall,
                Quantization::NextBar,
                ActionTarget {
                    scope: Some(TargetScope::LaneW30),
                    bank_id: Some("bank-a".into()),
                    pad_id: Some("pad-02".into()),
                    ..Default::default()
                },
            ),
            102,
        );
        queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::W30SwapBank,
                Quantization::NextBar,
                ActionTarget {
                    scope: Some(TargetScope::LaneW30),
                    bank_id: Some("bank-c".into()),
                    pad_id: Some("pad-01".into()),
                    ..Default::default()
                },
            ),
            103,
        );
        queue.enqueue(
            {
                let mut draft = ActionDraft::new(
                    ActorType::User,
                    ActionCommand::W30BrowseSlicePool,
                    Quantization::NextBeat,
                    ActionTarget {
                        scope: Some(TargetScope::LaneW30),
                        bank_id: Some("bank-a".into()),
                        pad_id: Some("pad-04".into()),
                        ..Default::default()
                    },
                );
                draft.params = ActionParams::Mutation {
                    intensity: 1.0,
                    target_id: Some("cap-02".into()),
                };
                draft
            },
            104,
        );
        queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::W30ApplyDamageProfile,
                Quantization::NextBar,
                ActionTarget {
                    scope: Some(TargetScope::LaneW30),
                    bank_id: Some("bank-d".into()),
                    pad_id: Some("pad-03".into()),
                    ..Default::default()
                },
            ),
            104,
        );
        queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::W30TriggerPad,
                Quantization::NextBeat,
                ActionTarget {
                    scope: Some(TargetScope::LaneW30),
                    bank_id: Some("bank-a".into()),
                    pad_id: Some("pad-03".into()),
                    ..Default::default()
                },
            ),
            103,
        );
        queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::W30StepFocus,
                Quantization::NextBeat,
                ActionTarget {
                    scope: Some(TargetScope::LaneW30),
                    bank_id: Some("bank-c".into()),
                    pad_id: Some("pad-01".into()),
                    ..Default::default()
                },
            ),
            104,
        );
        queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::Tr909Release,
                Quantization::NextPhrase,
                ActionTarget {
                    scope: Some(TargetScope::LaneTr909),
                    ..Default::default()
                },
            ),
            104,
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
            105,
        );
        let mut resample_draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::PromoteResample,
            Quantization::NextPhrase,
            ActionTarget {
                scope: Some(TargetScope::LaneW30),
                ..Default::default()
            },
        );
        resample_draft.params = crate::action::ActionParams::Promotion {
            capture_id: Some("cap-01".into()),
            destination: Some("w30:resample".into()),
        };
        queue.enqueue(resample_draft, 106);

        let vm = JamViewModel::build(&session, &queue, Some(&graph));

        assert!(vm.transport.is_playing);
        assert_eq!(vm.source.loop_candidate_count, 1);
        assert_eq!(vm.source.hook_candidate_count, 1);
        assert_eq!(vm.scene.scene_count, 1);
        assert_eq!(vm.scene.restore_scene, None);
        assert_eq!(
            vm.scene.scene_jump_availability,
            SceneJumpAvailabilityView::WaitingForMoreScenes
        );
        assert_eq!(vm.scene.active_scene_energy.as_deref(), Some("high"));
        assert_eq!(vm.scene.restore_scene_energy, None);
        assert_eq!(vm.capture.capture_count, 1);
        assert_eq!(vm.capture.pinned_capture_count, 0);
        assert_eq!(vm.capture.promoted_capture_count, 1);
        assert_eq!(vm.capture.unassigned_capture_count, 0);
        assert_eq!(vm.capture.pending_capture_count, 2);
        assert_eq!(vm.capture.last_capture_id.as_deref(), Some("cap-01"));
        assert_eq!(
            vm.capture.last_capture_target.as_deref(),
            Some("pad bank-a/pad-01")
        );
        assert_eq!(
            vm.capture.last_capture_target_kind,
            Some(CaptureTargetKindView::W30Pad)
        );
        assert_eq!(
            vm.capture.last_capture_handoff_readiness,
            Some(CaptureHandoffReadinessView::Fallback)
        );
        assert_eq!(
            vm.capture.last_promotion_result.as_deref(),
            Some("promoted to pad bank-a/pad-01")
        );
        assert_eq!(
            vm.capture.latest_w30_promoted_capture_label.as_deref(),
            Some("cap-01 -> bank-a/pad-01")
        );
        assert_eq!(
            vm.capture.recent_capture_rows,
            vec!["cap-01 | bank-a/pad-01 | 2 origins"]
        );
        assert_eq!(
            vm.capture.latest_capture_provenance_lines,
            vec![
                "file captures/cap-01.wav",
                "from action manual or unknown",
                "origins asset-a, src-1",
            ]
        );
        assert!(vm.capture.pinned_capture_ids.is_empty());
        assert_eq!(vm.capture.pending_capture_items.len(), 2);
        assert_eq!(vm.capture.pending_capture_items[0].command, "capture.now");
        assert_eq!(vm.capture.pending_capture_items[0].target, "lanew30");
        assert_eq!(
            vm.capture.pending_capture_items[0].explanation.as_deref(),
            Some("capture current break")
        );
        assert_eq!(
            vm.capture.pending_capture_items[1].command,
            "promote.resample"
        );
        assert_eq!(vm.capture.pending_capture_items[1].target, "lanew30");
        assert_eq!(vm.lanes.mc202_pending_role.as_deref(), Some("leader"));
        assert!(!vm.lanes.mc202_pending_follower_generation);
        assert!(!vm.lanes.mc202_pending_answer_generation);
        assert_eq!(vm.lanes.mc202_phrase_ref, None);
        assert_eq!(vm.lanes.w30_active_bank.as_deref(), Some("bank-a"));
        assert_eq!(vm.lanes.w30_focused_pad.as_deref(), Some("pad-01"));
        assert_eq!(
            vm.lanes.w30_pending_trigger_target.as_deref(),
            Some("bank-a/pad-03")
        );
        assert_eq!(
            vm.lanes.w30_pending_recall_target.as_deref(),
            Some("bank-a/pad-02")
        );
        assert_eq!(
            vm.lanes.w30_pending_bank_swap_target.as_deref(),
            Some("bank-c/pad-01")
        );
        assert_eq!(
            vm.lanes.w30_pending_slice_pool_target.as_deref(),
            Some("bank-a/pad-04")
        );
        assert_eq!(
            vm.lanes.w30_pending_slice_pool_capture_id.as_deref(),
            Some("cap-02")
        );
        assert_eq!(
            vm.lanes.w30_pending_damage_profile_target.as_deref(),
            Some("bank-d/pad-03")
        );
        assert_eq!(vm.lanes.w30_pending_audition_target, None);
        assert_eq!(
            vm.lanes.w30_pending_focus_step_target.as_deref(),
            Some("bank-c/pad-01")
        );
        assert_eq!(
            vm.lanes.w30_pending_resample_capture_id.as_deref(),
            Some("cap-01")
        );
        assert!(vm.lanes.tr909_takeover_enabled);
        assert_eq!(vm.lanes.tr909_takeover_pending_target, Some(false));
        assert_eq!(vm.lanes.tr909_takeover_pending_profile, None);
        assert_eq!(
            vm.lanes.tr909_takeover_profile,
            Some(Tr909TakeoverProfileState::SceneLockTakeover)
        );
        assert!(vm.lanes.tr909_fill_armed_next_bar);
        assert_eq!(vm.lanes.tr909_last_fill_bar, Some(8));
        assert_eq!(
            vm.lanes.tr909_reinforcement_mode,
            Some(Tr909ReinforcementModeState::Takeover)
        );
        assert_eq!(vm.pending_actions.len(), 11);
        assert_eq!(vm.ghost.mode, "assist");
    }

    #[test]
    fn derives_scene_energy_from_projected_scene_id() {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: "src-1".into(),
                path: "audio/test.wav".into(),
                content_hash: "graph-1".into(),
                duration_seconds: 32.0,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "0.1.0".into(),
                provider_set: vec!["beat".into(), "section".into()],
                generated_at: "2026-04-12T18:00:00Z".into(),
                source_hash: "graph-1".into(),
                analysis_seed: 7,
                run_notes: Some("scene-energy-test".into()),
            },
        );
        graph.sections.push(crate::source_graph::Section {
            section_id: "sec-a".into(),
            label_hint: crate::source_graph::SectionLabelHint::Intro,
            start_seconds: 0.0,
            end_seconds: 16.0,
            bar_start: 1,
            bar_end: 8,
            energy_class: crate::source_graph::EnergyClass::Medium,
            confidence: 0.9,
            tags: vec![],
        });
        graph.sections.push(crate::source_graph::Section {
            section_id: "sec-b".into(),
            label_hint: crate::source_graph::SectionLabelHint::Drop,
            start_seconds: 16.0,
            end_seconds: 32.0,
            bar_start: 9,
            bar_end: 16,
            energy_class: crate::source_graph::EnergyClass::High,
            confidence: 0.9,
            tags: vec![],
        });

        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-12T18:00:00Z");
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-02-drop"));
        session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-intro"));
        session.runtime_state.scene_state.scenes = vec![
            SceneId::from("scene-01-intro"),
            SceneId::from("scene-02-drop"),
        ];

        let vm = JamViewModel::build(&session, &ActionQueue::new(), Some(&graph));

        assert_eq!(vm.scene.active_scene.as_deref(), Some("scene-02-drop"));
        assert_eq!(vm.scene.restore_scene.as_deref(), Some("scene-01-intro"));
        assert_eq!(vm.scene.next_scene.as_deref(), Some("scene-01-intro"));
        assert_eq!(
            vm.scene.scene_jump_availability,
            SceneJumpAvailabilityView::Ready
        );
        assert_eq!(vm.scene.active_scene_energy.as_deref(), Some("high"));
        assert_eq!(vm.scene.restore_scene_energy.as_deref(), Some("medium"));
        assert_eq!(vm.scene.next_scene_energy.as_deref(), Some("medium"));

        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
        session.runtime_state.scene_state.scenes = vec![SceneId::from("scene-01-intro")];

        let vm = JamViewModel::build(&session, &ActionQueue::new(), Some(&graph));

        assert_eq!(vm.scene.next_scene, None);
        assert_eq!(
            vm.scene.scene_jump_availability,
            SceneJumpAvailabilityView::WaitingForMoreScenes
        );
        assert_eq!(vm.scene.next_scene_energy, None);
    }

    #[test]
    fn prefers_contrast_next_scene_when_energy_data_is_available() {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: "src-1".into(),
                path: "audio/test.wav".into(),
                content_hash: "graph-1".into(),
                duration_seconds: 48.0,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "0.1.0".into(),
                provider_set: vec!["beat".into(), "section".into()],
                generated_at: "2026-04-25T00:00:00Z".into(),
                source_hash: "graph-1".into(),
                analysis_seed: 7,
                run_notes: Some("scene-contrast-test".into()),
            },
        );
        graph.sections.push(crate::source_graph::Section {
            section_id: "sec-a".into(),
            label_hint: crate::source_graph::SectionLabelHint::Drop,
            start_seconds: 0.0,
            end_seconds: 16.0,
            bar_start: 1,
            bar_end: 8,
            energy_class: crate::source_graph::EnergyClass::High,
            confidence: 0.9,
            tags: vec![],
        });
        graph.sections.push(crate::source_graph::Section {
            section_id: "sec-b".into(),
            label_hint: crate::source_graph::SectionLabelHint::Break,
            start_seconds: 16.0,
            end_seconds: 32.0,
            bar_start: 9,
            bar_end: 16,
            energy_class: crate::source_graph::EnergyClass::High,
            confidence: 0.9,
            tags: vec![],
        });
        graph.sections.push(crate::source_graph::Section {
            section_id: "sec-c".into(),
            label_hint: crate::source_graph::SectionLabelHint::Intro,
            start_seconds: 32.0,
            end_seconds: 48.0,
            bar_start: 17,
            bar_end: 24,
            energy_class: crate::source_graph::EnergyClass::Medium,
            confidence: 0.9,
            tags: vec![],
        });

        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-25T00:00:00Z");
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-drop"));
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-drop"));
        session.runtime_state.scene_state.scenes = vec![
            SceneId::from("scene-01-drop"),
            SceneId::from("scene-02-break"),
            SceneId::from("scene-03-intro"),
        ];

        let vm = JamViewModel::build(&session, &ActionQueue::new(), Some(&graph));

        assert_eq!(vm.scene.next_scene.as_deref(), Some("scene-03-intro"));
        assert_eq!(vm.scene.next_scene_energy.as_deref(), Some("medium"));
        assert_eq!(
            next_scene_launch_candidate_with_reason(&session, Some(&graph))
                .map(|candidate| (candidate.scene_id.to_string(), candidate.reason,)),
            Some((
                "scene-03-intro".into(),
                SceneLaunchTargetReason::EnergyContrast,
            ))
        );

        let mut graph_with_unknown_current_energy = graph.clone();
        graph_with_unknown_current_energy.sections[0].energy_class =
            crate::source_graph::EnergyClass::Unknown;
        let vm = JamViewModel::build(
            &session,
            &ActionQueue::new(),
            Some(&graph_with_unknown_current_energy),
        );

        assert_eq!(vm.scene.next_scene.as_deref(), Some("scene-02-break"));
        assert_eq!(
            next_scene_launch_candidate_with_reason(
                &session,
                Some(&graph_with_unknown_current_energy)
            )
            .map(|candidate| candidate.reason),
            Some(SceneLaunchTargetReason::Ordered)
        );
    }

    #[derive(Debug, Deserialize)]
    struct SceneEnergyProjectionFixture {
        name: String,
        section_labels: Vec<String>,
        expected: SceneEnergyProjectionExpected,
    }

    #[derive(Debug, Deserialize)]
    struct SceneEnergyProjectionExpected {
        scenes: Vec<String>,
        active_scene: String,
        current_scene: String,
        active_scene_energy: String,
        #[serde(default)]
        restore_scene: Option<String>,
        #[serde(default)]
        restore_scene_energy: Option<String>,
    }

    #[test]
    fn fixture_backed_scene_energy_projection_holds() {
        let fixtures: Vec<SceneEnergyProjectionFixture> = serde_json::from_str(include_str!(
            "../../../riotbox-app/tests/fixtures/scene_regression.json"
        ))
        .expect("parse scene energy projection fixtures");

        for fixture in fixtures {
            let graph = sample_graph_with_sections(&fixture.section_labels);
            let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-12T18:00:00Z");
            session.runtime_state.scene_state.scenes = fixture
                .expected
                .scenes
                .iter()
                .map(|scene| scene.as_str().into())
                .collect();
            session.runtime_state.scene_state.active_scene =
                Some(fixture.expected.active_scene.as_str().into());
            session.runtime_state.transport.current_scene =
                Some(fixture.expected.current_scene.as_str().into());
            session.runtime_state.scene_state.restore_scene =
                fixture.expected.restore_scene.as_deref().map(Into::into);

            let vm = JamViewModel::build(&session, &ActionQueue::new(), Some(&graph));

            assert_eq!(
                vm.scene.active_scene.as_deref(),
                Some(fixture.expected.active_scene.as_str()),
                "{} active scene drifted",
                fixture.name
            );
            assert_eq!(
                vm.scene.restore_scene.as_deref(),
                fixture.expected.restore_scene.as_deref(),
                "{} restore scene drifted",
                fixture.name
            );
            assert_eq!(
                vm.scene.active_scene_energy.as_deref(),
                Some(fixture.expected.active_scene_energy.as_str()),
                "{} active energy drifted",
                fixture.name
            );
            assert_eq!(
                vm.scene.restore_scene_energy.as_deref(),
                fixture.expected.restore_scene_energy.as_deref(),
                "{} restore energy drifted",
                fixture.name
            );
        }
    }
}
