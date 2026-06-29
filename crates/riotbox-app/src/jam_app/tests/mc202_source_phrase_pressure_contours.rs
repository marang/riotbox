#[test]
fn committed_mc202_pressure_contour_tracks_source_low_band_movement() {
    let mut high_movement_graph =
        source_phrase_test_graph("src-pressure-move", "hash-pressure-move", 134.0, 101, 2);
    add_phrase_audio_features(
        &mut high_movement_graph,
        2,
        0.34,
        0.84,
        0.92,
        0.28,
        0.18,
        0.22,
        0.12,
        0.10,
    );
    let mut low_movement_graph = high_movement_graph.clone();
    add_phrase_audio_features(
        &mut low_movement_graph,
        2,
        0.34,
        0.84,
        0.18,
        0.28,
        0.18,
        0.22,
        0.12,
        0.10,
    );
    let mut high_state = confirmed_source_phrase_state(high_movement_graph);
    let mut low_state = confirmed_source_phrase_state(low_movement_graph);

    let high_render = commit_source_derived_pressure(&mut high_state);
    let low_render = commit_source_derived_pressure(&mut low_state);
    let high_plan = high_state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("high-movement pressure plan");
    let low_plan = low_state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("low-movement pressure plan");

    assert_eq!(
        high_plan.candidate_family,
        Some(Mc202SourcePhraseCandidateFamilyState::SubPressureShove)
    );
    assert_eq!(
        low_plan.candidate_family,
        Some(Mc202SourcePhraseCandidateFamilyState::SubPressureShove)
    );
    let high_notes = active_source_phrase_notes(high_plan);
    let low_notes = active_source_phrase_notes(low_plan);
    let high_min_note = high_notes.iter().copied().min().expect("high notes");
    let low_min_note = low_notes.iter().copied().min().expect("low notes");

    assert!(
        high_notes.len() > low_notes.len(),
        "high low-band movement should add a pressure movement step: high={high_plan:?} low={low_plan:?}"
    );
    assert!(
        high_min_note <= low_min_note - 3,
        "high low-band movement should deepen the pressure contour: high_min={high_min_note} low_min={low_min_note}; high={high_plan:?} low={low_plan:?}"
    );
    assert_ne!(
        high_plan.rhythm_cells, low_plan.rhythm_cells,
        "low-band movement did not affect pressure rhythm/interval cells"
    );
    assert!(
        high_plan
            .candidate_provenance_refs
            .iter()
            .any(|reference| reference.starts_with("groove_pressure_movement_step:")),
        "{high_plan:?}"
    );
    let render_delta = signal_delta_metrics(&high_render, &low_render);
    assert!(
        render_delta.rms > 0.001,
        "high/low movement pressure sources rendered too similarly: {render_delta:?}"
    );
}

fn commit_source_derived_pressure(state: &mut JamAppState) -> Vec<f32> {
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
    render_mc202_recipe_buffer(&state.runtime.mc202_render)
}

fn active_source_phrase_notes(
    plan: &riotbox_core::session::Mc202SourcePhrasePlanState,
) -> Vec<i8> {
    plan.rhythm_cells.iter().flatten().copied().collect()
}
