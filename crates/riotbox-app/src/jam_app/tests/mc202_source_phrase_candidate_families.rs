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

    let metrics = signal_metrics(&rendered);
    assert!(metrics.rms > 0.001, "candidate-backed answer rendered silent");
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
