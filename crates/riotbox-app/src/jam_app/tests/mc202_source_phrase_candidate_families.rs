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
        plan.source_section_id,
        Some(SectionId::from("section-a")),
        "typed ownership must follow the section that supplied phrase features"
    );
    let expression = plan
        .source_expression
        .as_ref()
        .expect("candidate-backed source expression");
    assert!(
        expression.offbeat_answer_space > expression.bass_pressure,
        "{expression:?}"
    );
    assert!(
        expression
            .provenance_refs
            .iter()
            .any(|reference| reference.starts_with("expression:offbeat_answer_space:")),
        "{expression:?}"
    );
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
fn cross_section_phrase_at_second_section_boundary_is_explicitly_unavailable() {
    let mut graph = source_phrase_test_graph("src-cross-section", "hash-cross-section", 132.0, 83, 2);
    graph.sections.push(Section {
        section_id: SectionId::from("section-b"),
        label_hint: SectionLabelHint::Drop,
        start_seconds: 16.0,
        end_seconds: 32.0,
        bar_start: 9,
        bar_end: 16,
        energy_class: EnergyClass::High,
        confidence: 0.9,
        tags: vec!["drop".into()],
    });
    let mut state = confirmed_source_phrase_state(graph);

    assert_eq!(
        state.queue_mc202_generate_answer(300),
        QueueControlResult::Enqueued
    );
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 36,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    assert_eq!(committed.len(), 1);
    assert!(
        state
            .session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .is_none()
    );
    assert_eq!(state.runtime.mc202_render.routing, Mc202RenderRouting::Silent);
    let result = state
        .session
        .action_log
        .actions
        .last()
        .and_then(|action| action.result.as_ref())
        .expect("committed action result");
    assert!(!result.accepted);
    assert!(result.summary.contains("phrase crosses source sections"));
}

#[test]
fn section_owned_phrase_at_uncovered_boundary_is_explicitly_unavailable() {
    let graph = source_phrase_test_graph(
        "src-uncovered-section",
        "hash-uncovered-section",
        132.0,
        89,
        2,
    );
    let mut state = confirmed_source_phrase_state(graph);

    assert_eq!(
        state.queue_mc202_generate_answer(300),
        QueueControlResult::Enqueued
    );
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 60,
            bar_index: 15,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    assert_eq!(committed.len(), 1);
    assert!(
        state
            .session
            .runtime_state
            .lane_state
            .mc202
            .source_phrase_plan
            .is_none()
    );
    assert_eq!(state.runtime.mc202_render.routing, Mc202RenderRouting::Silent);
    let result = state
        .session
        .action_log
        .actions
        .last()
        .and_then(|action| action.result.as_ref())
        .expect("committed action result");
    assert!(!result.accepted);
    assert!(result.summary.contains("ownership cannot be proven"));
}

#[test]
fn rejected_cross_section_phrase_is_atomic_and_replay_is_a_noop() {
    let mut graph = source_phrase_test_graph(
        "src-atomic-reject",
        "hash-atomic-reject",
        132.0,
        97,
        2,
    );
    graph.sections.push(Section {
        section_id: SectionId::from("section-b"),
        label_hint: SectionLabelHint::Drop,
        start_seconds: 16.0,
        end_seconds: 32.0,
        bar_start: 9,
        bar_end: 16,
        energy_class: EnergyClass::High,
        confidence: 0.9,
        tags: vec!["drop".into()],
    });
    add_phrase_audio_features(
        &mut graph, 2, 0.12, 0.20, 0.18, 0.36, 0.78, 0.30, 0.18, 0.15,
    );
    let mut state = confirmed_source_phrase_state(graph);
    let replay_base = state.session.clone();

    assert_eq!(
        state.queue_mc202_generate_follower(300),
        QueueControlResult::Enqueued
    );
    let first_commit = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 31,
            bar_index: 8,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );
    assert_eq!(first_commit.len(), 1);
    let previous_mc202 = state.session.runtime_state.lane_state.mc202.clone();
    let previous_touch = state.session.runtime_state.macro_state.mc202_touch;
    assert!(previous_mc202.source_phrase_plan.is_some());

    assert_eq!(
        state.queue_mc202_generate_answer(500),
        QueueControlResult::Enqueued
    );
    let rejected_commit = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 32,
            bar_index: 9,
            phrase_index: 3,
            scene_id: Some(SceneId::from("scene-1")),
        },
        600,
    );
    assert_eq!(rejected_commit.len(), 1);
    let rejected_action_id = rejected_commit[0].action_id;

    assert_eq!(state.session.runtime_state.lane_state.mc202, previous_mc202);
    assert_eq!(
        state.session.runtime_state.macro_state.mc202_touch,
        previous_touch
    );
    let rejected_action = state
        .session
        .action_log
        .actions
        .iter()
        .find(|action| action.id == rejected_action_id)
        .expect("rejected committed action remains durable evidence");
    assert_eq!(rejected_action.status, ActionStatus::Committed);
    assert!(
        rejected_action
            .result
            .as_ref()
            .is_some_and(|result| !result.accepted)
    );
    assert!(
        state
            .session
            .action_log
            .commit_records
            .iter()
            .find(|record| record.action_id == rejected_action_id)
            .expect("rejected action commit record")
            .mc202_source_phrase_plan
            .is_none()
    );
    assert!(
        state
            .session
            .runtime_state
            .undo_state
            .mc202_snapshots
            .iter()
            .all(|snapshot| snapshot.action_id != rejected_action_id)
    );

    let plan = riotbox_core::replay::build_committed_replay_plan(&state.session.action_log)
        .expect("rejected committed action remains valid history");
    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].action.id, first_commit[0].action_id);
    let mut replayed = replay_base;
    let report = riotbox_core::replay::apply_replay_plan_to_session(&mut replayed, &plan)
        .expect("accepted predecessor replays without rejected successor");
    assert_eq!(report.applied_action_ids, vec![first_commit[0].action_id]);
    assert_eq!(
        replayed.runtime_state.lane_state.mc202,
        state.session.runtime_state.lane_state.mc202
    );
    assert_eq!(
        replayed.runtime_state.macro_state.mc202_touch,
        state.session.runtime_state.macro_state.mc202_touch
    );
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
        pressure_render_plan.bass_weight >= hook_render_plan.bass_weight + 0.20,
        "pressure family bass articulation margin is too small for producer-grade low-end proof: pressure={pressure_render_plan:?} hook={hook_render_plan:?}"
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
    let pressure_low = source_phrase_low_band_rms(&pressure_render, 44_100, 2);
    let hook_low = source_phrase_low_band_rms(&hook_render, 44_100, 2);
    let pressure_metrics = signal_metrics(&pressure_render);
    let hook_metrics = signal_metrics(&hook_render);
    let pressure_low_share = pressure_low / pressure_metrics.rms.max(f32::EPSILON);
    let hook_low_share = hook_low / hook_metrics.rms.max(f32::EPSILON);
    assert!(
        delta.rms > 0.001,
        "source candidate families rendered too similarly: {delta:?}"
    );
    assert!(
        pressure_low > hook_low * 1.30,
        "pressure source did not render stronger low-band movement than hook source: pressure_low={pressure_low:.6} hook_low={hook_low:.6}"
    );
    assert!(
        pressure_low_share > hook_low_share * 1.12,
        "pressure source did not carry a larger low-band share than hook source: pressure_share={pressure_low_share:.6} hook_share={hook_low_share:.6}"
    );

    pressure_state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_mut()
        .expect("pressure plan for legacy projection")
        .source_expression = None;
    pressure_state.refresh_view();
    let legacy_render_plan = pressure_state
        .runtime
        .mc202_render
        .source_phrase_plan
        .expect("legacy source-derived render plan");
    assert!(
        legacy_render_plan.bass_weight >= pressure_render_plan.bass_weight - 0.06,
        "legacy source-derived pressure plan without source_expression lost bass body: original={pressure_render_plan:?} legacy={legacy_render_plan:?}"
    );
}

#[test]
fn committed_mc202_answer_preserves_sub_beat_source_anchor_timing() {
    let mut straight_graph =
        source_phrase_test_graph("src-subbeat-straight", "hash-subbeat", 132.0, 73, 2);
    add_phrase_audio_features(
        &mut straight_graph,
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
    set_source_phrase_anchors_with_subbeat_offsets(
        &mut straight_graph,
        &[
            (SourceTimingAnchorType::Kick, 8, 32, 0.00, 0.92),
            (SourceTimingAnchorType::AnswerSlot, 8, 33, 0.00, 0.95),
        ],
    );
    let mut pushed_graph = straight_graph.clone();
    pushed_graph.source.source_id = SourceId::from("src-subbeat-pushed");
    set_source_phrase_anchors_with_subbeat_offsets(
        &mut pushed_graph,
        &[
            (SourceTimingAnchorType::Kick, 8, 32, 0.00, 0.92),
            (SourceTimingAnchorType::AnswerSlot, 8, 33, 0.50, 0.95),
        ],
    );
    let mut straight_state = confirmed_source_phrase_state(straight_graph);
    let mut pushed_state = confirmed_source_phrase_state(pushed_graph);

    let straight_render = commit_source_derived_answer(&mut straight_state);
    let pushed_render = commit_source_derived_answer(&mut pushed_state);
    let straight_plan = straight_state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("straight sub-beat source phrase plan");
    let pushed_plan = pushed_state
        .session
        .runtime_state
        .lane_state
        .mc202
        .source_phrase_plan
        .as_ref()
        .expect("pushed sub-beat source phrase plan");

    assert!(straight_plan.is_source_derived(), "{straight_plan:?}");
    assert!(pushed_plan.is_source_derived(), "{pushed_plan:?}");
    assert_eq!(
        straight_plan.candidate_family,
        Some(Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer)
    );
    assert_eq!(
        pushed_plan.candidate_family,
        Some(Mc202SourcePhraseCandidateFamilyState::SparseOffbeatAnswer)
    );
    assert_ne!(
        provenance_step(straight_plan, "groove_answer_step"),
        provenance_step(pushed_plan, "groove_answer_step"),
        "sub-beat answer timing collapsed to the same MC-202 answer step: straight={straight_plan:?} pushed={pushed_plan:?}"
    );
    assert_ne!(
        straight_plan.rhythm_cells, pushed_plan.rhythm_cells,
        "sub-beat answer timing did not change MC-202 source phrase rhythm cells"
    );
    let delta = signal_delta_metrics(&straight_render, &pushed_render);
    assert!(
        delta.rms > 0.0005,
        "sub-beat source anchor timing did not materially change rendered MC-202 output: {delta:?}"
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
    let anchors_with_offsets = anchors
        .iter()
        .map(|(anchor_type, bar_index, beat_index, strength)| {
            (*anchor_type, *bar_index, *beat_index, 0.0, *strength)
        })
        .collect::<Vec<_>>();
    set_source_phrase_anchors_with_subbeat_offsets(graph, &anchors_with_offsets);
}

fn set_source_phrase_anchors_with_subbeat_offsets(
    graph: &mut SourceGraph,
    anchors: &[(SourceTimingAnchorType, u32, u32, f32, f32)],
) {
    let bpm = graph.timing.bpm_estimate.unwrap_or(132.0);
    let seconds_per_beat = 60.0 / bpm.max(1.0);
    graph.timing.primary_hypothesis_id = Some("primary-mc202-groove".into());
    graph.timing.hypotheses = vec![TimingHypothesis {
        hypothesis_id: "primary-mc202-groove".into(),
        kind: TimingHypothesisKind::Primary,
        bpm,
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
        confidence: 0.94,
        score: 0.94,
        beat_grid: anchors
            .iter()
            .map(
                |(_anchor_type, _bar_index, beat_index, _beat_offset, _strength)| BeatPoint {
                    beat_index: *beat_index,
                    time_seconds: *beat_index as f32 * seconds_per_beat,
                    confidence: 0.94,
                },
            )
            .collect(),
        bar_grid: Vec::new(),
        phrase_grid: graph.timing.phrase_grid.clone(),
        anchors: anchors
            .iter()
            .enumerate()
            .map(|(index, (anchor_type, bar_index, beat_index, beat_offset, strength))| {
                riotbox_core::source_graph::SourceTimingAnchor {
                    anchor_id: format!("mc202-groove-anchor-{index}"),
                    anchor_type: *anchor_type,
                    time_seconds: (*beat_index as f32 + *beat_offset) * seconds_per_beat,
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

fn source_phrase_low_band_rms(buffer: &[f32], sample_rate: u32, channel_count: usize) -> f32 {
    if buffer.is_empty() || sample_rate == 0 || channel_count == 0 {
        return 0.0;
    }
    let mut low = vec![0.0_f32; channel_count];
    let mut energy = 0.0_f32;
    let alpha = 1.0 - (-std::f32::consts::TAU * 160.0 / sample_rate as f32).exp();
    let mut sample_count = 0_usize;
    for frame in buffer.chunks(channel_count) {
        for (channel, sample) in frame.iter().enumerate() {
            low[channel] += (*sample - low[channel]) * alpha;
            energy += low[channel] * low[channel];
            sample_count += 1;
        }
    }
    (energy / sample_count.max(1) as f32).sqrt()
}
