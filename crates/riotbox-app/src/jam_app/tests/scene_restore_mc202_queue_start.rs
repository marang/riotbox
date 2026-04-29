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
    session.runtime_state.lane_state.mc202.role = Some("follower".into());
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
    assert_recipe_buffers_differ("scene launch mixed audio", &before_jump, &after_jump, 0.004);

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
    assert_eq!(
        state.runtime.tr909_render.phrase_variation,
        Some(Tr909PhraseVariation::PhraseDrive)
    );
    assert_eq!(
        state.runtime.mc202_render.contour_hint,
        Mc202ContourHint::Lift
    );
    let after_restore = render_scene_recipe_mix_buffer(&state);

    assert_recipe_buffers_differ(
        "scene restore mixed audio leaves launched state",
        &after_jump,
        &after_restore,
        0.004,
    );
    assert_recipe_buffers_differ(
        "scene restore keeps movement energy instead of collapsing to baseline",
        &before_jump,
        &after_restore,
        0.002,
    );
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

