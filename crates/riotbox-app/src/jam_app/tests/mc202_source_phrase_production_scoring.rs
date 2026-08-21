#[test]
fn committed_mc202_selection_prefers_source_production_impact_dimensions() {
    let mut pressure_graph =
        source_phrase_test_graph("src-score-pressure", "hash-score-pressure", 136.0, 131, 2);
    add_phrase_audio_features(
        &mut pressure_graph,
        2,
        0.36,
        0.82,
        0.86,
        0.28,
        0.14,
        0.18,
        0.10,
        0.12,
    );
    let mut pickup_graph =
        source_phrase_test_graph("src-score-pickup", "hash-score-pickup", 136.0, 137, 2);
    add_phrase_audio_features(
        &mut pickup_graph,
        2,
        0.16,
        0.22,
        0.18,
        0.86,
        0.72,
        0.74,
        0.48,
        0.14,
    );
    let mut pressure_state = confirmed_source_phrase_state(pressure_graph);
    let mut pickup_state = confirmed_source_phrase_state(pickup_graph);

    let pressure_render = commit_source_derived_pressure(&mut pressure_state);
    let pickup_render = commit_source_derived_instigator(&mut pickup_state);
    let pressure_plan = pressure_state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("pressure production-impact plan");
    let pickup_plan = pickup_state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("pickup production-impact plan");

    assert_eq!(
        pressure_plan.candidate_family,
        Some(Mc202SourcePhraseCandidateFamilyState::SubPressureShove)
    );
    assert_eq!(
        pickup_plan.candidate_family,
        Some(Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator),
        "{pickup_plan:?}"
    );
    let pressure_score = selected_source_phrase_scorecard(pressure_plan);
    let pickup_score = selected_source_phrase_scorecard(pickup_plan);

    assert!(
        pressure_score.low_end_impact >= 0.80,
        "{pressure_score:?}"
    );
    assert!(
        pressure_score.low_end_impact > pressure_score.answer_contrast * 2.0,
        "{pressure_score:?}"
    );
    assert!(
        pickup_score.destructive_usefulness > pickup_score.low_end_impact,
        "{pickup_score:?}"
    );
    assert!(
        pickup_score.answer_contrast > pressure_score.answer_contrast,
        "pickup source did not expose stronger answer contrast: pickup={pickup_score:?} pressure={pressure_score:?}"
    );
    assert!(
        pressure_plan
            .candidate_provenance_refs
            .iter()
            .any(|reference| reference.starts_with("candidate_production_impact_score:")),
        "{pressure_plan:?}"
    );
    assert!(
        pickup_plan
            .candidate_provenance_refs
            .iter()
            .any(|reference| reference.starts_with("candidate_selected_dimensions:")),
        "{pickup_plan:?}"
    );
    assert_ne!(
        pressure_plan.rhythm_cells, pickup_plan.rhythm_cells,
        "production-impact selection collapsed pressure and pickup source plans"
    );
    let render_delta = signal_delta_metrics(&pressure_render, &pickup_render);
    assert!(
        render_delta.rms > 0.001,
        "production-impact-selected pressure and pickup renders were too similar: {render_delta:?}"
    );
}

#[test]
fn explicit_pressure_stays_silent_when_only_a_pickup_family_is_eligible() {
    let mut graph =
        source_phrase_test_graph("src-role-mismatch", "hash-role-mismatch", 136.0, 139, 2);
    add_phrase_audio_features(
        &mut graph, 2, 0.16, 0.22, 0.18, 0.86, 0.72, 0.74, 0.48, 0.14,
    );
    let mut state = confirmed_source_phrase_state(graph);

    assert_eq!(
        state.queue_mc202_generate_pressure(300),
        QueueControlResult::Enqueued
    );
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 32,
            bar_index: 8,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    assert_eq!(committed.len(), 1);
    let plan = state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("typed degraded pressure plan");
    assert_eq!(plan.role, Mc202RoleState::Pressure);
    assert_eq!(
        plan.candidate_family,
        Some(Mc202SourcePhraseCandidateFamilyState::FallbackControl)
    );
    assert_eq!(
        plan.fallback_reason.as_deref(),
        Some("no_role_compatible_source_candidate")
    );
    assert!(!plan.is_source_derived());
    assert!(plan.rhythm_cells.iter().all(Option::is_none));
    assert!(plan.candidate_scorecards.iter().any(|score| {
        score.family == Mc202SourcePhraseCandidateFamilyState::FillPickupInstigator
            && score.rejection_reason.as_deref() == Some("requested_role_family_mismatch")
    }));
    assert_eq!(state.runtime.mc202_render.routing, Mc202RenderRouting::Silent);
    assert!(state.runtime.mc202_render.source_phrase_plan.is_none());

    let result = state
        .session
        .action_log
        .actions
        .last()
        .and_then(|action| action.result.as_ref())
        .expect("visible degraded result");
    assert!(result.accepted);
    assert!(
        result
            .summary
            .contains("committed silent degraded state"),
        "{result:?}"
    );
    let persisted_plan = state
        .session
        .action_log
        .commit_records
        .last()
        .and_then(|record| record.mc202_source_phrase_plan.as_ref())
        .expect("degraded replay truth")
        .clone();
    assert_eq!(&persisted_plan, plan);

    let restored = JamAppState::from_parts(
        state.session.clone(),
        state.source_graph.clone(),
        riotbox_core::queue::ActionQueue::new(),
    );
    assert_eq!(
        restored
            .session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .as_ref(),
        Some(&persisted_plan)
    );
    assert_eq!(
        restored.runtime.mc202_render.routing,
        Mc202RenderRouting::Silent
    );
    assert!(restored.runtime.mc202_render.source_phrase_plan.is_none());
}

fn selected_source_phrase_scorecard(
    plan: &riotbox_core::session::Mc202SourcePhrasePlanState,
) -> &riotbox_core::session::Mc202SourcePhraseCandidateScoreState {
    plan.candidate_scorecards
        .iter()
        .find(|score| score.selected)
        .expect("selected MC-202 scorecard")
}

fn commit_source_derived_instigator(state: &mut JamAppState) -> Vec<f32> {
    assert_eq!(
        state.queue_mc202_generate_instigator(300),
        QueueControlResult::Enqueued
    );
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 32,
            bar_index: 8,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    assert_eq!(committed.len(), 1);
    render_mc202_recipe_buffer(&state.runtime.mc202_render)
}
