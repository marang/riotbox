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
            (SourceTimingAnchorType::Kick, 8, 29, 0.94),
            (SourceTimingAnchorType::Backbeat, 8, 31, 0.88),
            (SourceTimingAnchorType::AnswerSlot, 8, 30, 0.97),
        ],
    );
    let mut late_answer_graph = early_answer_graph.clone();
    late_answer_graph.source.source_id = SourceId::from("src-answer-late");
    late_answer_graph.source.content_hash = "hash-answer-groove-late".into();
    late_answer_graph.provenance.source_hash = "hash-answer-groove-late".into();
    set_source_phrase_anchors(
        &mut late_answer_graph,
        &[
            (SourceTimingAnchorType::Kick, 8, 29, 0.94),
            (SourceTimingAnchorType::Backbeat, 8, 31, 0.88),
            (SourceTimingAnchorType::AnswerSlot, 8, 32, 0.97),
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

#[test]
fn committed_mc202_source_phrase_roles_render_distinct_acid_bass_expression() {
    let mut graph =
        source_phrase_test_graph("src-role-expression", "hash-role-expression", 132.0, 149, 2);
    add_phrase_audio_features(
        &mut graph, 2, 0.30, 0.76, 0.84, 0.48, 0.58, 0.52, 0.26, 0.16,
    );
    set_source_phrase_anchors(
        &mut graph,
        &[
            (SourceTimingAnchorType::Kick, 8, 32, 0.96),
            (SourceTimingAnchorType::Backbeat, 8, 34, 0.92),
            (SourceTimingAnchorType::AnswerSlot, 8, 35, 0.94),
            (SourceTimingAnchorType::Fill, 8, 39, 0.86),
        ],
    );

    let mut pressure_state = confirmed_source_phrase_state(graph.clone());
    let mut answer_state = confirmed_source_phrase_state(graph.clone());
    let mut instigator_state = confirmed_source_phrase_state(graph);
    let pressure_render = commit_source_derived_role(&mut pressure_state, Mc202RoleState::Pressure);
    let answer_render = commit_source_derived_role(&mut answer_state, Mc202RoleState::Answer);
    let instigator_render =
        commit_source_derived_role(&mut instigator_state, Mc202RoleState::Instigator);
    let pressure_plan = pressure_state
        .runtime
        .mc202_render
        .source_phrase_plan
        .expect("pressure role render plan");
    let answer_plan = answer_state
        .runtime
        .mc202_render
        .source_phrase_plan
        .expect("answer role render plan");
    let instigator_plan = instigator_state
        .runtime
        .mc202_render
        .source_phrase_plan
        .expect("instigator role render plan");

    assert!(
        pressure_plan.bass_weight > answer_plan.bass_weight + 0.22,
        "pressure role did not project stronger bass body than answer: pressure={pressure_plan:?} answer={answer_plan:?}"
    );
    assert!(
        answer_plan.stab_bite > pressure_plan.stab_bite + 0.20,
        "answer role did not project sharper stab bite than pressure: pressure={pressure_plan:?} answer={answer_plan:?}"
    );
    assert!(
        answer_plan.gate_snap > pressure_plan.gate_snap + 0.24,
        "answer role did not project tighter gate than pressure: pressure={pressure_plan:?} answer={answer_plan:?}"
    );
    assert!(
        instigator_plan.destructive_mask != 0
            && instigator_plan.gate_snap > answer_plan.gate_snap
            && instigator_plan.stab_bite >= answer_plan.stab_bite,
        "instigator role did not project a destructive spike above answer: instigator={instigator_plan:?} answer={answer_plan:?}"
    );
    let pressure_low_band_rms = source_phrase_low_band_rms(&pressure_render, 44_100, 2);
    let answer_low_band_rms = source_phrase_low_band_rms(&answer_render, 44_100, 2);
    let pressure_metrics = signal_metrics(&pressure_render);
    let answer_metrics = signal_metrics(&answer_render);
    let pressure_low_band_share = pressure_low_band_rms / pressure_metrics.rms.max(f32::EPSILON);
    let answer_low_band_share = answer_low_band_rms / answer_metrics.rms.max(f32::EPSILON);
    assert!(
        pressure_low_band_rms > answer_low_band_rms * 1.10,
        "pressure role did not render stronger absolute low-band body than answer: pressure_low={pressure_low_band_rms:.6} answer_low={answer_low_band_rms:.6}"
    );
    assert!(
        pressure_low_band_share > answer_low_band_share + 0.02,
        "pressure role did not render a larger low-band share than answer: pressure_share={pressure_low_band_share:.6} answer_share={answer_low_band_share:.6} pressure_rms={:.6} answer_rms={:.6}",
        pressure_metrics.rms,
        answer_metrics.rms,
    );
    assert!(
        signal_delta_metrics(&pressure_render, &answer_render).rms > 0.0015,
        "pressure and answer roles rendered too similarly"
    );
    assert!(
        signal_delta_metrics(&answer_render, &instigator_render).rms > 0.0015,
        "answer and instigator roles rendered too similarly"
    );
}

fn commit_source_derived_role(state: &mut JamAppState, role: Mc202RoleState) -> Vec<f32> {
    let result = match role {
        Mc202RoleState::Leader => panic!("leader source phrase role is not covered by this helper"),
        Mc202RoleState::Follower => state.queue_mc202_generate_follower(300),
        Mc202RoleState::Answer => state.queue_mc202_generate_answer(300),
        Mc202RoleState::Pressure => state.queue_mc202_generate_pressure(300),
        Mc202RoleState::Instigator => state.queue_mc202_generate_instigator(300),
    };
    assert_eq!(result, QueueControlResult::Enqueued);
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 32,
            bar_index: 8,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-role-expression")),
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
        .expect("source phrase role plan");
    assert_eq!(plan.role, role);
    assert!(plan.is_source_derived(), "{plan:?}");
    render_mc202_recipe_buffer(&state.runtime.mc202_render)
}
