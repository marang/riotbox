#[test]
fn committed_mc202_answer_records_source_backed_candidate_family_metadata() {
    let mut graph = source_phrase_test_graph("src-candidate", "hash-candidate", 132.0, 19, 2);
    add_phrase_audio_features(
        &mut graph, 2, 0.12, 0.20, 0.18, 0.36, 0.78, 0.30, 0.18, 0.15,
    );
    let mut state = confirmed_source_phrase_state(graph);

    let rendered = commit_source_derived_answer(&mut state);
    let plan = state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("candidate-backed MC-202 source phrase plan");

    assert!(plan.is_source_derived(), "{plan:?}");
    assert_eq!(
        plan.candidate_family,
        Some(Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer)
    );
    assert!(plan.candidate_count >= 6, "{plan:?}");
    assert!(plan.rejected_candidate_count >= 1, "{plan:?}");
    assert!(
        plan.candidate_provenance_refs
            .iter()
            .any(|reference| reference == "candidate_family:sparse_offbeat_answer"),
        "{plan:?}"
    );
    assert!(
        plan.candidate_provenance_refs
            .iter()
            .any(|reference| reference.starts_with("candidate_rejected:fallback_control")),
        "{plan:?}"
    );
    assert_eq!(
        plan.candidate_scorecards.len(),
        plan.candidate_count as usize,
        "{plan:?}"
    );
    let selected_score = plan
        .candidate_scorecards
        .iter()
        .find(|score| score.selected)
        .expect("selected candidate scorecard");
    assert_eq!(
        selected_score.family,
        Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer
    );
    assert!(selected_score.total_score > 0.50, "{selected_score:?}");
    assert!(
        selected_score.answer_contrast > selected_score.low_end_impact,
        "{selected_score:?}"
    );
    assert!(
        plan.candidate_scorecards.iter().any(|score| {
            score.family == Mc202SourcePhraseCandidateFamilyState::FallbackControl
                && score.rejection_reason.as_deref()
                    == Some("control_template_not_source_derived")
        }),
        "{plan:?}"
    );
    assert!(plan.phrase_memory_distance > 0.90, "{plan:?}");
    let render_plan = state
        .runtime
        .mc202_render
        .source_phrase_plan
        .expect("source phrase render plan");
    assert!(render_plan.pressure > 0.10, "{render_plan:?}");
    assert!(render_plan.contrast > 0.40, "{render_plan:?}");
    assert_ne!(render_plan.accent_mask, 0, "{render_plan:?}");
    assert_ne!(render_plan.destructive_mask, 0, "{render_plan:?}");

    let metrics = signal_metrics(&rendered);
    assert!(metrics.rms > 0.001, "candidate-backed answer rendered silent");
}

#[test]
fn committed_mc202_answer_scorecards_record_phrase_memory_after_previous_plan() {
    let mut graph = source_phrase_test_graph("src-memory", "hash-memory", 132.0, 43, 2);
    add_phrase_audio_features(
        &mut graph, 2, 0.12, 0.20, 0.18, 0.36, 0.78, 0.30, 0.18, 0.15,
    );
    let mut state = confirmed_source_phrase_state(graph);

    let first_render = commit_source_derived_answer(&mut state);
    let first_plan = state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("first candidate plan")
        .clone();
    let first_memory = first_plan.phrase_memory_distance;
    let second_render = commit_source_derived_answer(&mut state);
    let second_plan = state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("second candidate plan");
    let selected_score = second_plan
        .candidate_scorecards
        .iter()
        .find(|score| score.selected)
        .expect("selected candidate scorecard");

    assert!(first_memory > 0.90, "{first_memory}");
    assert!(second_plan.phrase_memory_distance < 1.0, "{second_plan:?}");
    assert!(
        second_plan.candidate_family != first_plan.candidate_family
            || second_plan.rhythm_cells != first_plan.rhythm_cells
            || second_plan.fallback_reason.is_some(),
        "repeated live trigger reused the previous MC-202 source phrase without variation: first={first_plan:?} second={second_plan:?}"
    );
    assert_eq!(
        selected_score.phrase_memory,
        second_plan.phrase_memory_distance
    );
    assert!(
        second_plan
            .candidate_provenance_refs
            .iter()
            .any(|reference| reference.starts_with("phrase_memory_selected_distance:")),
        "{second_plan:?}"
    );
    assert!(
        second_plan.candidate_scorecards.iter().any(|score| {
            matches!(
                score.rejection_reason.as_deref(),
                Some("phrase_memory_static_repeat")
                    | Some("phrase_memory_too_close_to_previous")
            )
        }),
        "{second_plan:?}"
    );
    let render_delta = signal_delta_metrics(&first_render, &second_render);
    assert!(
        render_delta.rms > 0.0005,
        "repeated live trigger did not materially change MC-202 render: {render_delta:?}"
    );
}

#[test]
fn committed_mc202_answer_changes_candidate_family_between_pressure_and_hook_sources() {
    let mut pressure_graph =
        source_phrase_test_graph("src-pressure", "hash-pressure", 134.0, 23, 2);
    add_phrase_audio_features(
        &mut pressure_graph,
        2,
        0.32,
        0.70,
        0.82,
        0.34,
        0.12,
        0.46,
        0.22,
        0.18,
    );
    let mut hook_graph = source_phrase_test_graph("src-hook", "hash-hook", 134.0, 29, 2);
    hook_graph.sections[0].label_hint = SectionLabelHint::Chorus;
    hook_graph.sections[0].tags = vec!["hook".into(), "vocal".into()];
    add_phrase_audio_features(
        &mut hook_graph,
        2,
        0.07,
        0.10,
        0.05,
        0.60,
        0.16,
        0.42,
        0.50,
        0.88,
    );
    let mut pressure_state = confirmed_source_phrase_state(pressure_graph);
    let mut hook_state = confirmed_source_phrase_state(hook_graph);

    let pressure_render = commit_source_derived_answer(&mut pressure_state);
    let hook_render = commit_source_derived_answer(&mut hook_state);
    let pressure_render_plan = pressure_state
        .runtime
        .mc202_render
        .source_phrase_plan
        .expect("pressure source render plan");
    let hook_render_plan = hook_state
        .runtime
        .mc202_render
        .source_phrase_plan
        .expect("hook source render plan");
    let pressure_plan = pressure_state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("pressure candidate plan");
    let hook_plan = hook_state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("hook candidate plan");

    assert_eq!(
        pressure_plan.candidate_family,
        Some(Mc202SourcePhraseCandidateFamilyState::SubPressureShove)
    );
    assert_eq!(
        hook_plan.candidate_family,
        Some(Mc202SourcePhraseCandidateFamilyState::HookRestraintGhostAnswer)
    );
    assert_ne!(pressure_plan.rhythm_cells, hook_plan.rhythm_cells);
    assert!(
        pressure_render_plan.bass_weight > hook_render_plan.bass_weight,
        "pressure family did not project stronger bass articulation: pressure={pressure_render_plan:?} hook={hook_render_plan:?}"
    );
    assert!(
        hook_render_plan.stab_bite > pressure_render_plan.stab_bite,
        "hook answer family did not project stronger stab articulation: pressure={pressure_render_plan:?} hook={hook_render_plan:?}"
    );
    assert!(
        hook_render_plan.gate_snap > pressure_render_plan.gate_snap,
        "hook answer family did not project snappier gate articulation: pressure={pressure_render_plan:?} hook={hook_render_plan:?}"
    );
    let delta = signal_delta_metrics(&pressure_render, &hook_render);
    assert!(
        delta.rms > 0.001,
        "source candidate families rendered too similarly: {delta:?}"
    );
}

#[test]
fn committed_mc202_answer_changes_or_rejects_candidates_when_measured_audio_is_removed() {
    let mut measured_graph =
        source_phrase_test_graph("src-measured", "hash-measured", 132.0, 31, 2);
    add_phrase_audio_features(
        &mut measured_graph,
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
    let mut metadata_only_graph = measured_graph.clone();
    metadata_only_graph.phrase_audio_features.clear();
    let mut measured_state = confirmed_source_phrase_state(measured_graph);
    let mut metadata_only_state = confirmed_source_phrase_state(metadata_only_graph);

    commit_source_derived_answer(&mut measured_state);
    commit_source_derived_answer(&mut metadata_only_state);
    let measured_plan = measured_state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("measured candidate plan");
    let metadata_only_plan = metadata_only_state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("metadata-only candidate plan");

    assert!(measured_plan.is_source_derived(), "{measured_plan:?}");
    assert!(
        metadata_only_plan.fallback_reason.is_some()
            || measured_plan.candidate_family != metadata_only_plan.candidate_family
            || measured_plan.rhythm_cells != metadata_only_plan.rhythm_cells,
        "removing measured phrase audio did not alter or reject the MC-202 candidate plan: measured={measured_plan:?} metadata_only={metadata_only_plan:?}"
    );
    assert!(
        measured_plan
            .candidate_provenance_refs
            .iter()
            .any(|reference| reference.contains("phrase_audio")),
        "{measured_plan:?}"
    );
}

#[allow(clippy::too_many_arguments)]
fn add_phrase_audio_features(
    graph: &mut SourceGraph,
    phrase_index: u32,
    low_band_rms: f32,
    low_mid_ratio: f32,
    low_band_movement: f32,
    transient_density: f32,
    offbeat_onset_density: f32,
    spectral_roughness: f32,
    spectral_brightness: f32,
    hook_restraint_hint: f32,
) {
    graph.phrase_audio_features = vec![PhraseAudioFeatures {
        phrase_index,
        start_seconds: 0.0,
        end_seconds: 16.0,
        start_bar: 8,
        end_bar: 15,
        low_band_rms,
        low_mid_ratio,
        low_band_movement,
        transient_density,
        offbeat_onset_density,
        spectral_roughness,
        spectral_brightness,
        hook_restraint_hint,
        confidence: 0.92,
        provenance_refs: vec!["mc202.test.phrase-audio-features".into()],
    }];
}

fn set_source_phrase_anchors(
    graph: &mut SourceGraph,
    anchors: &[(SourceTimingAnchorType, u32, u32, f32)],
) {
    graph.timing.primary_hypothesis_id = Some("primary-mc202-groove".into());
    graph.timing.hypotheses = vec![TimingHypothesis {
        hypothesis_id: "primary-mc202-groove".into(),
        kind: TimingHypothesisKind::Primary,
        bpm: graph.timing.bpm_estimate.unwrap_or(132.0),
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
        confidence: 0.94,
        score: 0.94,
        beat_grid: Vec::new(),
        bar_grid: Vec::new(),
        phrase_grid: graph.timing.phrase_grid.clone(),
        anchors: anchors
            .iter()
            .enumerate()
            .map(|(index, (anchor_type, bar_index, beat_index, strength))| {
                riotbox_core::source_graph::SourceTimingAnchor {
                    anchor_id: format!("mc202-groove-anchor-{index}"),
                    anchor_type: *anchor_type,
                    time_seconds: *beat_index as f32 * 0.45,
                    bar_index: Some(*bar_index),
                    beat_index: Some(*beat_index),
                    confidence: 0.94,
                    strength: *strength,
                    tags: vec!["mc202_groove_test".into()],
                }
            })
            .collect(),
        drift: Vec::new(),
        groove: Vec::new(),
        quality: TimingQuality::High,
        warnings: Vec::new(),
        provenance: vec!["mc202.test.groove-anchors".into()],
    }];
}

fn provenance_step(
    plan: &riotbox_core::session::Mc202SourcePhrasePlanState,
    prefix: &str,
) -> usize {
    plan.candidate_provenance_refs
        .iter()
        .find_map(|reference| {
            reference
                .strip_prefix(prefix)
                .and_then(|value| value.strip_prefix(':'))
                .and_then(|value| value.parse::<usize>().ok())
        })
        .unwrap_or_else(|| panic!("missing {prefix} provenance in {plan:?}"))
}
