#[test]
fn committed_mc202_answer_rejects_feature_empty_source_phrase_as_fallback() {
    let mut graph = sample_graph();
    graph.sections.clear();
    graph.assets.clear();
    graph.candidates.clear();
    graph.timing.bpm_confidence = 0.12;
    graph.timing.primary_hypothesis_id = None;
    graph.timing.hypotheses.clear();
    graph.timing.phrase_grid = vec![riotbox_core::source_graph::PhraseSpan {
        phrase_index: 2,
        start_bar: 8,
        end_bar: 15,
        confidence: 0.18,
    }];
    graph.analysis_summary = AnalysisSummary::default();
    let mut session = sample_session(&graph);
    session.runtime_state.source_timing.confirmed_grid = Some(SourceTimingGridConfirmationState {
        source_id: graph.source.source_id.clone(),
        hypothesis_id: None,
        confirmed_by_action: ActionId(77),
        confirmed_at: 1_771_156_800_000,
    });
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_mc202_generate_answer(300),
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
        .expect("fallback MC-202 source phrase plan");
    assert!(!plan.is_source_derived());
    assert_eq!(
        plan.fallback_reason.as_deref(),
        Some("stay_out_source_context")
    );
    assert!(plan.rhythm_cells.iter().all(Option::is_none));
    assert_eq!(state.runtime.mc202_render.routing, Mc202RenderRouting::Silent);
    assert!(state.runtime.mc202_render.source_phrase_plan.is_none());
}
