#[test]
fn committed_mc202_answer_candidate_scoring_is_deterministic_for_same_source_seed() {
    let mut graph = source_phrase_test_graph("src-deterministic", "hash-deterministic", 132.0, 37, 2);
    add_phrase_audio_features(
        &mut graph, 2, 0.12, 0.20, 0.18, 0.36, 0.78, 0.30, 0.18, 0.15,
    );
    let mut first_state = confirmed_source_phrase_state(graph.clone());
    let mut second_state = confirmed_source_phrase_state(graph);

    commit_source_derived_answer(&mut first_state);
    commit_source_derived_answer(&mut second_state);
    let first_plan = first_state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("first candidate plan");
    let second_plan = second_state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("second candidate plan");

    assert_eq!(first_plan.candidate_family, second_plan.candidate_family);
    assert_eq!(first_plan.rhythm_cells, second_plan.rhythm_cells);
    assert_eq!(first_plan.source_expression, second_plan.source_expression);
    assert_eq!(first_plan.candidate_scorecards, second_plan.candidate_scorecards);
    assert_eq!(
        first_state.runtime.mc202_render.source_phrase_plan,
        second_state.runtime.mc202_render.source_phrase_plan
    );
    assert_eq!(
        render_signal_fingerprint(&commit_source_derived_answer(&mut first_state)),
        render_signal_fingerprint(&commit_source_derived_answer(&mut second_state))
    );
}

#[test]
fn committed_mc202_answer_passes_cross_source_diversity_gate_for_feature_families() {
    let cases = [
        source_family_case(
            "pressure",
            "hash-gate-pressure",
            131.0,
            71,
            Mc202SourcePhraseCandidateFamilyState::SubPressureShove,
            [0.36, 0.82, 0.90, 0.30, 0.18, 0.34, 0.16, 0.08],
        ),
        source_family_case(
            "offbeat",
            "hash-gate-offbeat",
            137.0,
            73,
            Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer,
            [0.13, 0.18, 0.20, 0.42, 0.82, 0.30, 0.20, 0.18],
        ),
        source_family_case(
            "callback",
            "hash-gate-callback",
            124.0,
            79,
            Mc202SourcePhraseCandidateFamilyState::CallBackStab,
            [0.10, 0.18, 0.12, 0.90, 0.12, 0.46, 0.18, 0.20],
        ),
        source_family_case(
            "hook",
            "hash-gate-hook",
            145.0,
            83,
            Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer,
            [0.07, 0.12, 0.05, 0.62, 0.22, 0.34, 0.50, 0.86],
        ),
    ];
    let mut outputs = Vec::new();

    for case in cases {
        let mut state = confirmed_source_phrase_state(case.graph);
        let rendered = commit_source_derived_answer(&mut state);
        let plan = state
            .session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .as_ref()
            .expect("source family gate plan");
        let render_plan = state
            .runtime
            .mc202_render
            .source_phrase_plan
            .expect("source family gate render plan");

        assert!(plan.is_source_derived(), "{} {plan:?}", case.label);
        assert!(
            plan.source_expression.is_some(),
            "{} source-derived plan did not record expression state: {plan:?}",
            case.label
        );
        assert_eq!(
            plan.candidate_family,
            Some(case.expected_family),
            "{} {plan:?}",
            case.label
        );
        assert_ne!(
            render_plan.active_mask, 0,
            "{} source-derived render plan lost active steps",
            case.label
        );
        outputs.push((case.label, plan.clone(), render_plan, rendered));
    }

    for left_index in 0..outputs.len() {
        for right_index in (left_index + 1)..outputs.len() {
            let (left_label, left_plan, left_render_plan, left_render) = &outputs[left_index];
            let (right_label, right_plan, right_render_plan, right_render) = &outputs[right_index];
            let plan_distance = source_phrase_plan_distance(left_plan, right_plan);
            let expression_distance = source_expression_distance(left_plan, right_plan);
            let render_delta = signal_delta_metrics(left_render, right_render);

            assert!(
                plan_distance >= 0.30,
                "{left_label} -> {right_label} source plans collapsed: {plan_distance:.3}; left={left_plan:?} right={right_plan:?}"
            );
            assert!(
                expression_distance >= 0.20,
                "{left_label} -> {right_label} source expressions collapsed: {expression_distance:.3}; left={left_plan:?} right={right_plan:?}"
            );
            assert_ne!(
                left_render_plan.active_mask, right_render_plan.active_mask,
                "{left_label} -> {right_label} active masks collapsed despite distinct source families"
            );
            assert!(
                render_delta.rms >= 0.002,
                "{left_label} -> {right_label} rendered MC-202 answers too similar: {render_delta:?}"
            );
        }
    }
}

#[test]
fn committed_mc202_answer_rejects_template_collapse_when_source_features_are_neutralized() {
    let measured = source_family_case(
        "measured-pressure",
        "hash-collapse",
        132.0,
        89,
        Mc202SourcePhraseCandidateFamilyState::SubPressureShove,
        [0.34, 0.82, 0.88, 0.34, 0.20, 0.42, 0.18, 0.12],
    )
    .graph;
    let mut neutralized = measured.clone();
    neutralize_source_phrase_features(&mut neutralized);

    let mut measured_state = confirmed_source_phrase_state(measured);
    let mut neutral_state = confirmed_source_phrase_state(neutralized);
    commit_source_answer_without_render(&mut measured_state);
    commit_source_answer_without_render(&mut neutral_state);
    let measured_plan = measured_state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("measured source plan");
    let neutral_plan = neutral_state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("neutralized source plan");

    assert!(measured_plan.is_source_derived(), "{measured_plan:?}");
    assert!(
        !neutral_plan.is_source_derived(),
        "neutralized low/transient/hook features still passed source-derived proof: {neutral_plan:?}"
    );
    let measured_expression = measured_plan
        .source_expression
        .as_ref()
        .expect("measured source expression");
    let neutral_expression = neutral_plan
        .source_expression
        .as_ref()
        .expect("neutralized source expression");
    assert!(
        measured_expression.confidence > neutral_expression.confidence,
        "neutralization did not lower expression confidence: measured={measured_expression:?} neutral={neutral_expression:?}"
    );
    assert!(
        neutral_expression.stay_out_pressure > measured_expression.stay_out_pressure,
        "neutralization did not increase stay-out pressure: measured={measured_expression:?} neutral={neutral_expression:?}"
    );
    assert!(
        neutral_plan.rhythm_cells.iter().all(Option::is_none),
        "{neutral_plan:?}"
    );
    assert!(
        neutral_state.runtime.mc202_render.source_phrase_plan.is_none(),
        "neutralized source leaked a rendered source phrase plan"
    );
    let neutral_render = render_mc202_recipe_silent_buffer(&neutral_state.runtime.mc202_render);
    let neutral_metrics = signal_metrics(&neutral_render);
    assert_eq!(
        neutral_metrics.active_samples, 0,
        "neutralized source leaked audible MC-202 fallback output: {neutral_metrics:?}"
    );
    assert!(
        source_phrase_plan_distance(measured_plan, neutral_plan) >= 0.45,
        "neutralized source collapsed to measured plan: measured={measured_plan:?} neutral={neutral_plan:?}"
    );
    let measured_render = render_mc202_recipe_buffer(&measured_state.runtime.mc202_render);
    assert!(
        signal_metrics(&measured_render).rms > 0.001,
        "measured source control render unexpectedly silent"
    );
}

struct SourceFamilyGateCase {
    label: &'static str,
    graph: SourceGraph,
    expected_family: Mc202SourcePhraseCandidateFamilyState,
}

fn source_family_case(
    label: &'static str,
    hash: &str,
    bpm: f32,
    analysis_seed: u64,
    expected_family: Mc202SourcePhraseCandidateFamilyState,
    features: [f32; 8],
) -> SourceFamilyGateCase {
    let mut graph = source_phrase_test_graph(
        &format!("src-gate-{label}"),
        hash,
        bpm,
        analysis_seed,
        2,
    );
    add_phrase_audio_features(
        &mut graph,
        2,
        features[0],
        features[1],
        features[2],
        features[3],
        features[4],
        features[5],
        features[6],
        features[7],
    );
    SourceFamilyGateCase {
        label,
        graph,
        expected_family,
    }
}

fn neutralize_source_phrase_features(graph: &mut SourceGraph) {
    graph.sections.clear();
    graph.assets.clear();
    graph.candidates.clear();
    graph.phrase_audio_features.clear();
    graph.analysis_summary = AnalysisSummary::default();
    graph.timing.bpm_confidence = 0.12;
    graph.timing.primary_hypothesis_id = None;
    graph.timing.hypotheses.clear();
}

fn commit_source_answer_without_render(state: &mut JamAppState) {
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
}

fn source_phrase_plan_distance(
    left: &riotbox_core::session::Mc202SourcePhrasePlanState,
    right: &riotbox_core::session::Mc202SourcePhrasePlanState,
) -> f32 {
    let family_distance = if left.candidate_family == right.candidate_family {
        0.0
    } else {
        0.35
    };
    let cell_distance = left
        .rhythm_cells
        .iter()
        .zip(right.rhythm_cells.iter())
        .filter(|(left, right)| left != right)
        .count() as f32
        / 16.0;
    let active_distance = (left
        .rhythm_cells
        .iter()
        .filter(|cell| cell.is_some())
        .count() as f32
        - right
            .rhythm_cells
            .iter()
            .filter(|cell| cell.is_some())
            .count() as f32)
        .abs()
        / 8.0;

    (family_distance + cell_distance * 0.50 + active_distance * 0.15).clamp(0.0, 1.0)
}

fn source_expression_distance(
    left: &riotbox_core::session::Mc202SourcePhrasePlanState,
    right: &riotbox_core::session::Mc202SourcePhrasePlanState,
) -> f32 {
    let Some(left) = left.source_expression.as_ref() else {
        return 0.0;
    };
    let Some(right) = right.source_expression.as_ref() else {
        return 0.0;
    };

    let deltas = [
        left.low_pressure_contour - right.low_pressure_contour,
        left.bass_pressure - right.bass_pressure,
        left.transient_backbeat - right.transient_backbeat,
        left.offbeat_answer_space - right.offbeat_answer_space,
        left.phrase_density - right.phrase_density,
        left.hook_restraint - right.hook_restraint,
        left.stab_bite - right.stab_bite,
        left.stay_out_pressure - right.stay_out_pressure,
    ];
    let mean = deltas.iter().map(|delta| delta.abs()).sum::<f32>() / deltas.len() as f32;
    let strongest_axis = deltas
        .iter()
        .map(|delta| delta.abs())
        .fold(0.0_f32, f32::max);

    (mean * 0.65 + strongest_axis * 0.35).clamp(0.0, 1.0)
}

fn render_signal_fingerprint(buffer: &[f32]) -> (u32, u32, u32) {
    let metrics = signal_metrics(buffer);
    (
        metrics.active_samples as u32,
        (metrics.peak_abs * 1_000_000.0).round() as u32,
        (metrics.rms * 1_000_000.0).round() as u32,
    )
}
