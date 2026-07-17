#[test]
fn committed_scene_select_projects_target_scene_into_tr909_source_support() {
    let mut graph = sample_graph();
    graph.sections.push(Section {
        section_id: SectionId::from("section-b"),
        label_hint: SectionLabelHint::Break,
        start_seconds: 16.0,
        end_seconds: 24.0,
        bar_start: 9,
        bar_end: 12,
        energy_class: EnergyClass::Medium,
        confidence: 0.84,
        tags: vec!["break".into()],
    });

    let mut session = sample_session(&graph);
    session.runtime_state.transport.position_beats = 32.0;
    session.runtime_state.transport.current_scene = None;
    session.runtime_state.scene_state.active_scene = None;
    session.runtime_state.scene_state.scenes.clear();
    session.runtime_state.lane_state.tr909.reinforcement_mode =
        Some(Tr909ReinforcementModeState::SourceSupport);
    session.runtime_state.lane_state.tr909.pattern_ref = Some("support-scene".into());

    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.runtime.tr909_render.source_support_profile,
        Some(Tr909SourceSupportProfile::DropDrive)
    );
    assert_eq!(state.queue_scene_select(300), QueueControlResult::Enqueued);

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 32,
            bar_index: 8,
            phrase_index: 1,
            scene_id: Some(SceneId::from("scene-01-drop")),
        },
        350,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state.session.runtime_state.scene_state.active_scene,
        Some(SceneId::from("scene-02-break"))
    );
    assert_eq!(
        state.runtime.tr909_render.current_scene_id.as_deref(),
        Some("scene-02-break")
    );
    assert_eq!(
        state.runtime.tr909_render.source_support_profile,
        Some(Tr909SourceSupportProfile::BreakLift)
    );
    assert_eq!(
        state.runtime.tr909_render.source_support_context,
        Some(Tr909SourceSupportContext::SceneTarget)
    );
    assert_eq!(
        state.runtime_view.tr909_render_support_context,
        "scene_target"
    );
    assert_eq!(state.runtime_view.tr909_render_support_accent, "scene");
    assert_eq!(
        state.runtime.tr909_render.pattern_adoption,
        Some(Tr909PatternAdoption::SupportPulse)
    );
}

#[test]
fn queue_scene_restore_enqueues_scene_restore_for_next_bar() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-02-break"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-02-break"));
    session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));

    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    assert_eq!(state.queue_scene_restore(300), QueueControlResult::Enqueued);
    assert_eq!(
        state.queue_scene_restore(301),
        QueueControlResult::AlreadyPending
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::SceneRestore);
    assert_eq!(pending[0].quantization, Quantization::NextBar);
    assert_eq!(
        pending[0].target.scene_id,
        Some(SceneId::from("scene-01-drop"))
    );
    assert_eq!(
        pending[0].params,
        ActionParams::Scene {
            scene_id: Some(SceneId::from("scene-01-drop"))
        }
    );
}

#[test]
fn committed_scene_restore_updates_transport_scene_and_restore_pointer() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-02-break"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-02-break"));
    session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));

    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    assert_eq!(state.queue_scene_restore(300), QueueControlResult::Enqueued);

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 36,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-02-break")),
        },
        420,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state.session.runtime_state.scene_state.active_scene,
        Some(SceneId::from("scene-01-drop"))
    );
    assert_eq!(
        state.session.runtime_state.transport.current_scene,
        Some(SceneId::from("scene-01-drop"))
    );
    assert_eq!(
        state.session.runtime_state.scene_state.restore_scene,
        Some(SceneId::from("scene-02-break"))
    );
    assert_eq!(
        state.runtime.transport.current_scene,
        Some(SceneId::from("scene-01-drop"))
    );
    assert_eq!(
        state.jam_view.scene.active_scene.as_deref(),
        Some("scene-01-drop")
    );
    assert_eq!(
        state.jam_view.scene.restore_scene.as_deref(),
        Some("scene-02-break")
    );
    assert_eq!(
        state.jam_view.scene.active_scene_energy.as_deref(),
        Some("high")
    );
    assert_eq!(state.jam_view.scene.restore_scene_energy.as_deref(), None);
    assert_eq!(
        state.runtime.tr909_render.current_scene_id.as_deref(),
        Some("scene-01-drop")
    );
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("restored scene scene-01-drop at bar 9 / phrase 2")
    );
}

#[test]
fn committed_scene_restore_projects_target_scene_into_tr909_source_support() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.transport.position_beats = 32.0;
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-02-break"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-02-break"));
    session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));
    session.runtime_state.lane_state.tr909.reinforcement_mode =
        Some(Tr909ReinforcementModeState::SourceSupport);
    session.runtime_state.lane_state.tr909.pattern_ref = Some("restore-support".into());

    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    assert_eq!(state.queue_scene_restore(300), QueueControlResult::Enqueued);

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Bar,
            beat_index: 36,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-02-break")),
        },
        420,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state.session.runtime_state.scene_state.active_scene,
        Some(SceneId::from("scene-01-drop"))
    );
    assert_eq!(
        state.runtime.tr909_render.current_scene_id.as_deref(),
        Some("scene-01-drop")
    );
    assert_eq!(
        state.runtime.tr909_render.source_support_profile,
        Some(Tr909SourceSupportProfile::DropDrive)
    );
    assert_eq!(
        state.runtime.tr909_render.source_support_context,
        Some(Tr909SourceSupportContext::SceneTarget)
    );
    assert_eq!(
        state.runtime_view.tr909_render_support_context,
        "scene_target"
    );
    assert_eq!(state.runtime_view.tr909_render_support_accent, "scene");
}

#[test]
fn scene_jump_restore_replay_proves_state_and_mixed_audio_path() {
    let graph = scene_regression_graph(&["drop".into(), "break".into()]);
    let mut session = sample_session(&graph);
    session.runtime_state.transport.position_beats = 32.0;
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-drop"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-drop"));
    session.runtime_state.scene_state.restore_scene = None;
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-drop"),
        SceneId::from("scene-02-break"),
    ];
    session.runtime_state.lane_state.tr909.reinforcement_mode =
        Some(Tr909ReinforcementModeState::SourceSupport);
    session.runtime_state.lane_state.tr909.pattern_ref = Some("scene-recipe-support".into());
    session.runtime_state.lane_state.mc202.role = Some(Mc202RoleState::Follower);
    session.runtime_state.lane_state.mc202.phrase_variant = None;

    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.runtime.tr909_render.source_support_profile,
        Some(Tr909SourceSupportProfile::DropDrive)
    );
    assert_eq!(
        state.runtime.tr909_render.source_support_context,
        Some(Tr909SourceSupportContext::SceneTarget)
    );
    assert_eq!(
        state.runtime.mc202_render.contour_hint,
        Mc202ContourHint::Drop
    );
    assert_eq!(
        state.jam_view.scene.next_scene_policy,
        Some(SceneTransitionPolicyView {
            kind: SceneTransitionKindView::Launch,
            direction: SceneTransitionDirectionView::Drop,
            tr909_intent: SceneTransitionLaneIntentView::Release,
            mc202_intent: SceneTransitionLaneIntentView::Anchor,
            intensity: 0.55,
        })
    );
    let before_jump = render_scene_recipe_mix_buffer(&state);
    let before_jump_tr909 = state.runtime.tr909_render.clone();
    let before_jump_mc202 = state.runtime.mc202_render;
    assert_eq!(
        state
            .session
            .runtime_state
            .scene_state
            .active_projection_movement,
        None
    );

    assert_eq!(state.queue_scene_select(300), QueueControlResult::Enqueued);
    let launched = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 36,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-01-drop")),
        },
        360,
    );
    assert_eq!(launched.len(), 1);
    assert_eq!(
        state.session.runtime_state.scene_state.active_scene,
        Some(SceneId::from("scene-02-break"))
    );
    assert_eq!(
        state.session.runtime_state.scene_state.restore_scene,
        Some(SceneId::from("scene-01-drop"))
    );
    assert_eq!(
        state.jam_view.scene.active_scene.as_deref(),
        Some("scene-02-break")
    );
    assert_eq!(
        state.jam_view.scene.restore_scene.as_deref(),
        Some("scene-01-drop")
    );
    assert_eq!(
        state.runtime.tr909_render.source_support_profile,
        Some(Tr909SourceSupportProfile::BreakLift)
    );
    assert_eq!(
        state.runtime.tr909_render.source_support_context,
        Some(Tr909SourceSupportContext::SceneTarget)
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .scene_state
            .last_movement
            .as_ref()
            .map(|movement| (
                movement.direction,
                movement.tr909_intent,
                movement.mc202_intent
            )),
        Some((
            SceneMovementDirectionState::Drop,
            SceneMovementLaneIntentState::Release,
            SceneMovementLaneIntentState::Anchor,
        ))
    );
    assert_eq!(
        state.runtime.tr909_render.phrase_variation,
        Some(Tr909PhraseVariation::PhraseRelease)
    );
    assert_eq!(
        state.runtime.mc202_render.contour_hint,
        Mc202ContourHint::Hold
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .scene_state
            .active_projection_movement,
        state.session.runtime_state.scene_state.last_movement
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .scene_state
            .restore_projection_movement,
        None
    );
    assert_eq!(
        state.jam_view.scene.restore_scene_policy,
        Some(SceneTransitionPolicyView {
            kind: SceneTransitionKindView::Restore,
            direction: SceneTransitionDirectionView::Rise,
            tr909_intent: SceneTransitionLaneIntentView::Drive,
            mc202_intent: SceneTransitionLaneIntentView::Lift,
            intensity: 0.75,
        })
    );
    let after_jump = render_scene_recipe_mix_buffer(&state);
    assert_recipe_buffers_differ("scene launch mixed audio", &before_jump, &after_jump, 0.002);

    assert_eq!(state.queue_scene_restore(420), QueueControlResult::Enqueued);
    let restored = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 40,
            bar_index: 10,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-02-break")),
        },
        480,
    );
    assert_eq!(restored.len(), 1);
    assert_eq!(
        state.session.runtime_state.scene_state.active_scene,
        Some(SceneId::from("scene-01-drop"))
    );
    assert_eq!(
        state.session.runtime_state.scene_state.restore_scene,
        Some(SceneId::from("scene-02-break"))
    );
    assert_eq!(
        state.jam_view.scene.active_scene.as_deref(),
        Some("scene-01-drop")
    );
    assert_eq!(
        state.jam_view.scene.restore_scene.as_deref(),
        Some("scene-02-break")
    );
    assert_eq!(
        state.runtime.tr909_render.source_support_profile,
        Some(Tr909SourceSupportProfile::DropDrive)
    );
    assert_eq!(
        state.runtime.tr909_render.source_support_context,
        Some(Tr909SourceSupportContext::SceneTarget)
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .scene_state
            .last_movement
            .as_ref()
            .map(|movement| (
                movement.direction,
                movement.tr909_intent,
                movement.mc202_intent
            )),
        Some((
            SceneMovementDirectionState::Rise,
            SceneMovementLaneIntentState::Drive,
            SceneMovementLaneIntentState::Lift,
        ))
    );
    assert_eq!(state.runtime.tr909_render, before_jump_tr909);
    assert_eq!(state.runtime.mc202_render, before_jump_mc202);
    assert_eq!(
        state
            .session
            .runtime_state
            .scene_state
            .active_projection_movement,
        None,
        "restore must recover the pre-launch baseline projection"
    );
    assert!(
        state
            .session
            .runtime_state
            .scene_state
            .restore_projection_movement
            .as_ref()
            .is_some_and(|movement| movement.to_scene == SceneId::from("scene-02-break")),
        "the launched projection must remain paired with the reverse restore target"
    );
    let after_restore = render_scene_recipe_mix_buffer(&state);

    assert_recipe_buffers_differ(
        "scene restore mixed audio leaves launched state",
        &after_jump,
        &after_restore,
        0.002,
    );
    assert_recipe_buffers_match(
        "scene restore returns to the exact pre-jump lane projection",
        &before_jump,
        &after_restore,
        0.00001,
    );
}

#[test]
fn scene_jump_silences_mc202_plan_from_another_source_section_and_restore_recovers_it() {
    let graph = scene_regression_graph(&["intro".into(), "drop".into()]);
    let mut session = sample_session(&graph);
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-intro"),
        SceneId::from("scene-02-drop"),
    ];
    session.runtime_state.lane_state.mc202.role = Some(Mc202RoleState::Follower);
    session.runtime_state.lane_state.mc202.source_phrase_plan =
        Some(section_one_mc202_source_plan(graph.source.source_id.clone()));

    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    let before_jump = state.runtime.mc202_render;
    assert_eq!(before_jump.routing, Mc202RenderRouting::MusicBusBass);

    assert_eq!(state.queue_scene_select(300), QueueControlResult::Enqueued);
    let launched = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 36,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-01-intro")),
        },
        360,
    );
    assert_eq!(launched.len(), 1);
    assert_eq!(
        state.runtime.mc202_render.routing,
        Mc202RenderRouting::Silent,
        "target section has no trusted MC-202 phrase; no tonal fallback may leak"
    );
    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Idle);

    assert_eq!(state.queue_scene_restore(420), QueueControlResult::Enqueued);
    let restored = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 40,
            bar_index: 10,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-02-drop")),
        },
        480,
    );
    assert_eq!(restored.len(), 1);
    assert_eq!(state.runtime.mc202_render, before_jump);
}

#[test]
fn mc202_plan_from_another_source_stays_silent_without_a_current_section() {
    let mut graph = scene_regression_graph(&["intro".into(), "drop".into()]);
    graph.sections.clear();
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.mc202.role = Some(Mc202RoleState::Follower);
    session.runtime_state.lane_state.mc202.source_phrase_plan = Some(
        section_one_mc202_source_plan(SourceId::from("src-from-another-loaded-file")),
    );

    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(state.runtime.mc202_render.routing, Mc202RenderRouting::Silent);
    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Idle);
}

#[test]
fn typed_mc202_section_plan_fails_closed_when_current_section_is_unknown() {
    let mut graph = scene_regression_graph(&["intro".into(), "drop".into()]);
    let source_id = graph.source.source_id.clone();
    graph.sections.clear();
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.mc202.role = Some(Mc202RoleState::Follower);
    session.runtime_state.lane_state.mc202.source_phrase_plan =
        Some(section_one_mc202_source_plan(source_id));

    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(state.runtime.mc202_render.routing, Mc202RenderRouting::Silent);
    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Idle);
}

fn section_one_mc202_source_plan(source_id: SourceId) -> Mc202SourcePhrasePlanState {
    Mc202SourcePhrasePlanState {
        source_id,
        source_section_id: Some(SectionId::from("section-0")),
        phrase_slot: Mc202SourcePhraseSlotState {
            phrase_index: 1,
            start_bar: 1,
            end_bar: 8,
        },
        source_expression: None,
        role: Mc202RoleState::Follower,
        rhythm_cells: [Some(0), None, None, None, Some(3), None, None, None, Some(0), None, None, None, Some(-2), None, None, None],
        note_budget: Mc202SourcePhraseNoteBudgetState::Balanced,
        touch: 0.72,
        confidence: 0.8,
        candidate_family: Some(Mc202SourcePhraseCandidateFamilyState::CallBackStab),
        candidate_count: 1,
        rejected_candidate_count: 0,
        candidate_provenance_refs: vec!["source_section:section-0".into()],
        candidate_scorecards: Vec::new(),
        phrase_memory_distance: 1.0,
        fallback_reason: None,
    }
}

#[test]
fn queueing_mc202_role_change_blocks_duplicate_pending_actions() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_mc202_role_toggle(300),
        QueueControlResult::Enqueued
    );
    assert_eq!(
        state.queue_mc202_role_toggle(301),
        QueueControlResult::AlreadyPending
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::Mc202SetRole);
    assert_eq!(
        state.jam_view.lanes.mc202_pending_role.as_deref(),
        Some("leader")
    );
}
