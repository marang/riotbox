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
        let mc202_pending_instigator_generation = pending_actions.iter().any(|action| {
            matches!(
                action.command,
                crate::action::ActionCommand::Mc202GenerateInstigator
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
        let w30_pending_slice_pool_reason =
            pending_actions
                .iter()
                .rev()
                .find_map(|action| match action.command {
                    crate::action::ActionCommand::W30BrowseSlicePool => action
                        .explanation
                        .as_deref()
                        .map(|explanation| {
                            if explanation.contains("feral slice pool") {
                                "feral".to_string()
                            } else {
                                "cycle".to_string()
                            }
                        })
                        .or_else(|| Some("cycle".into())),
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
                timing: SourceTimingSummaryView::from_graph(graph),
                section_count: graph.sections.len(),
                loop_candidate_count: graph.loop_candidate_count(),
                hook_candidate_count: graph.hook_candidate_count(),
                feral_scorecard: FeralScorecardView::from_graph(graph),
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
        let active_scene_energy =
            graph.and_then(|graph| current_scene_energy_label(session, graph));
        let restore_scene_energy =
            graph.and_then(|graph| restore_scene_energy_label(session, graph));
        let next_scene_policy = scene_transition_policy(
            SceneTransitionKindView::Launch,
            active_scene_energy.as_deref(),
            next_scene_energy.as_deref(),
        );
        let restore_scene_policy = scene_transition_policy(
            SceneTransitionKindView::Restore,
            active_scene_energy.as_deref(),
            restore_scene_energy.as_deref(),
        );

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
                active_scene_energy,
                restore_scene_energy,
                next_scene_energy,
                next_scene_policy,
                restore_scene_policy,
                last_movement: scene_movement_view(session),
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
                mc202_pending_instigator_generation,
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
                w30_pending_slice_pool_reason,
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
            ghost: ghost_status_view(session),
            warnings,
        }
    }
}
