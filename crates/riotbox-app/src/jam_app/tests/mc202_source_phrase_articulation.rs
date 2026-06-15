#[test]
fn committed_mc202_answer_places_sparse_answer_from_source_answer_slot() {
    let mut early_answer_graph =
        source_phrase_test_graph("src-answer-early", "hash-answer-groove", 132.0, 53, 2);
    add_phrase_audio_features(
        &mut early_answer_graph,
        2,
        0.12,
        0.20,
        0.18,
        0.36,
        0.78,
        0.30,
        0.18,
        0.15,
    );
    set_source_phrase_anchors(
        &mut early_answer_graph,
        &[
            (SourceTimingAnchorType::Kick, 8, 32, 0.94),
            (SourceTimingAnchorType::Backbeat, 8, 34, 0.88),
            (SourceTimingAnchorType::AnswerSlot, 8, 33, 0.97),
        ],
    );
    let mut late_answer_graph = early_answer_graph.clone();
    late_answer_graph.source.source_id = SourceId::from("src-answer-late");
    late_answer_graph.source.content_hash = "hash-answer-groove-late".into();
    late_answer_graph.provenance.source_hash = "hash-answer-groove-late".into();
    set_source_phrase_anchors(
        &mut late_answer_graph,
        &[
            (SourceTimingAnchorType::Kick, 8, 32, 0.94),
            (SourceTimingAnchorType::Backbeat, 8, 34, 0.88),
            (SourceTimingAnchorType::AnswerSlot, 8, 35, 0.97),
        ],
    );
    let mut early_state = confirmed_source_phrase_state(early_answer_graph);
    let mut late_state = confirmed_source_phrase_state(late_answer_graph);

    commit_source_derived_answer(&mut early_state);
    commit_source_derived_answer(&mut late_state);
    let early_plan = early_state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("early source-answer plan");
    let late_plan = late_state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("late source-answer plan");

    assert_eq!(
        early_plan.candidate_family,
        Some(Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer)
    );
    assert_eq!(
        late_plan.candidate_family,
        Some(Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer)
    );
    assert_eq!(provenance_step(early_plan, "groove_answer_step"), 4);
    assert_eq!(provenance_step(late_plan, "groove_answer_step"), 12);
    assert_eq!(early_plan.rhythm_cells[4], Some(5), "{early_plan:?}");
    assert_eq!(late_plan.rhythm_cells[12], Some(5), "{late_plan:?}");
    assert_ne!(early_plan.rhythm_cells, late_plan.rhythm_cells);
}

#[test]
fn committed_mc202_hook_restraint_ghost_answer_avoids_downbeat_template_slots() {
    let mut graph =
        source_phrase_test_graph("src-hook-restraint", "hash-hook-restraint", 134.0, 59, 2);
    graph.sections[0].label_hint = SectionLabelHint::Chorus;
    graph.sections[0].tags = vec!["hook".into(), "vocal".into()];
    add_phrase_audio_features(
        &mut graph, 2, 0.07, 0.12, 0.05, 0.62, 0.22, 0.34, 0.50, 0.86,
    );
    set_source_phrase_anchors(
        &mut graph,
        &[
            (SourceTimingAnchorType::Kick, 8, 32, 0.96),
            (SourceTimingAnchorType::Backbeat, 8, 34, 0.92),
        ],
    );
    let mut state = confirmed_source_phrase_state(graph);

    commit_source_derived_answer(&mut state);
    let plan = state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("hook-restraint source-answer plan");
    let hook_safe_step = provenance_step(plan, "groove_hook_safe_step");

    assert_eq!(
        plan.candidate_family,
        Some(Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer)
    );
    assert_ne!(hook_safe_step, 0, "{plan:?}");
    assert_ne!(hook_safe_step, 8, "{plan:?}");
    assert!(plan.rhythm_cells[0].is_none(), "{plan:?}");
    assert!(plan.rhythm_cells[8].is_none(), "{plan:?}");
    assert!(
        plan.rhythm_cells
            .iter()
            .enumerate()
            .any(|(step, cell)| step == hook_safe_step && cell.is_some()),
        "{plan:?}"
    );
}
