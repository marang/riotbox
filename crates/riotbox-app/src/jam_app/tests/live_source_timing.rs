#[test]
fn live_ingest_rust_timing_is_stable_and_carries_source_identity() {
    let temp = tempdir().expect("tempdir");
    let source_path = temp.path().join("stable-128.wav");
    let samples = accented_drum_grid_samples(15_360, 32);
    riotbox_audio::source_audio::write_interleaved_pcm16_wav(
        &source_path,
        32_768,
        1,
        &samples,
    )
    .expect("write source");
    let mut first = sample_graph();
    let mut second = sample_graph();

    enrich_graph_with_rust_source_timing(&mut first, &source_path).expect("enrich first graph");
    enrich_graph_with_rust_source_timing(&mut second, &source_path).expect("enrich second graph");

    assert_eq!(first.timing, second.timing);
    assert!((first.timing.bpm_estimate.expect("BPM") - 128.0).abs() <= 1.0);
    assert!(
        first
            .timing
            .primary_hypothesis()
            .expect("primary hypothesis")
            .provenance
            .contains(&first.source.source_id.to_string())
    );
    assert!(
        first
            .provenance
            .provider_set
            .contains(&"riotbox-rust-source-timing-probe".into())
    );
}

#[test]
fn different_live_sources_do_not_reuse_stale_timing_or_identity() {
    let temp = tempdir().expect("tempdir");
    let source_128 = temp.path().join("source-128.wav");
    let source_120 = temp.path().join("source-120.wav");
    riotbox_audio::source_audio::write_interleaved_pcm16_wav(
        &source_128,
        32_768,
        1,
        &accented_drum_grid_samples(15_360, 32),
    )
    .expect("write 128 BPM source");
    riotbox_audio::source_audio::write_interleaved_pcm16_wav(
        &source_120,
        32_768,
        1,
        &accented_drum_grid_samples(16_384, 32),
    )
    .expect("write 120 BPM source");
    let mut first = sample_graph();
    let mut second = sample_graph();
    second.source.source_id = SourceId::from("src-2");

    enrich_graph_with_rust_source_timing(&mut first, &source_128).expect("enrich first graph");
    enrich_graph_with_rust_source_timing(&mut second, &source_120).expect("enrich second graph");

    let first_primary = first.timing.primary_hypothesis().expect("first primary");
    let second_primary = second.timing.primary_hypothesis().expect("second primary");
    assert!((first_primary.bpm - second_primary.bpm).abs() >= 6.0);
    assert!(first_primary.provenance.contains(&"src-1".into()));
    assert!(!first_primary.provenance.contains(&"src-2".into()));
    assert!(second_primary.provenance.contains(&"src-2".into()));
    assert!(!second_primary.provenance.contains(&"src-1".into()));
}

#[test]
fn jam_transport_uses_selected_primary_downbeat_phase_and_phrase_grid() {
    let mut graph = sample_graph();
    let seconds_per_beat = 0.5_f32;
    graph.timing.phrase_grid = vec![riotbox_core::source_graph::PhraseSpan {
        phrase_index: 2,
        start_bar: 5,
        end_bar: 8,
        confidence: 0.5,
    }];
    graph.timing.primary_hypothesis_id = Some("phase-three".into());
    graph.timing.hypotheses = vec![TimingHypothesis {
        hypothesis_id: "phase-three".into(),
        kind: TimingHypothesisKind::Primary,
        bpm: 120.0,
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
        confidence: 0.95,
        score: 0.95,
        beat_grid: (4..=43)
            .map(|beat_index| BeatPoint {
                beat_index,
                time_seconds: (beat_index - 4) as f32 * seconds_per_beat,
                confidence: 0.95,
            })
            .collect(),
        bar_grid: (1..=10)
            .map(|bar_index| riotbox_core::source_graph::BarSpan {
                bar_index,
                start_seconds: (bar_index - 1) as f32 * 4.0 * seconds_per_beat,
                end_seconds: bar_index as f32 * 4.0 * seconds_per_beat,
                downbeat_confidence: 0.95,
                phrase_index: Some((bar_index - 1) / 4 + 1),
            })
            .collect(),
        phrase_grid: vec![riotbox_core::source_graph::PhraseSpan {
            phrase_index: 9,
            start_bar: 5,
            end_bar: 8,
            confidence: 0.95,
        }],
        anchors: Vec::new(),
        drift: Vec::new(),
        groove: Vec::new(),
        quality: TimingQuality::High,
        warnings: Vec::new(),
        provenance: vec!["test:phase-three".into()],
    }];

    let clock = super::transport_helpers::transport_clock_for_state(19.0, true, None, Some(&graph));

    assert_eq!(clock.beat_index, 19);
    assert_eq!(clock.bar_index, 5);
    assert_eq!(clock.phrase_index, 9);
}

#[test]
fn manual_confirm_timing_stays_silent_until_explicit_bpm_confirmation_commits() {
    let graph = manual_confirm_source_window_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(state.runtime.tr909_render.tempo_bpm, 0.0);
    assert_eq!(state.runtime.mc202_render.tempo_bpm, 0.0);
    assert_eq!(state.runtime.w30_preview.tempo_bpm, 0.0);
    assert_eq!(state.source_monitor_render_state().tempo_bpm, 0.0);

    confirm_explicit_source_bpm(&mut state, 126.0).expect("confirm explicit BPM");

    assert_eq!(state.runtime.tr909_render.tempo_bpm, 126.0);
    assert_eq!(state.runtime.mc202_render.tempo_bpm, 126.0);
    assert_eq!(state.runtime.w30_preview.tempo_bpm, 126.0);
    assert_eq!(state.source_monitor_render_state().tempo_bpm, 126.0);
    assert_eq!(
        state.session.action_log.actions.last().map(|action| action.command),
        Some(ActionCommand::SourceTimingConfirmGrid)
    );
    assert!(state.session.runtime_state.source_timing.confirmed_grid.is_some());
}

#[test]
fn committed_mc202_fallback_stays_silent_without_losing_trusted_transport_timing() {
    let graph = manual_confirm_source_window_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    confirm_explicit_source_bpm(&mut state, 126.0).expect("confirm explicit BPM");
    state.session.runtime_state.lane_state.mc202.source_phrase_plan =
        Some(Mc202SourcePhrasePlanState {
            source_id: SourceId::from("src-1"),
            source_section_id: None,
            phrase_slot: Mc202SourcePhraseSlotState {
                phrase_index: 0,
                start_bar: 1,
                end_bar: 8,
            },
            source_expression: None,
            role: Mc202RoleState::Follower,
            rhythm_cells: [None; 16],
            note_budget: Mc202SourcePhraseNoteBudgetState::Sparse,
            touch: 0.0,
            confidence: 0.0,
            candidate_family: Some(Mc202SourcePhraseCandidateFamilyState::FallbackControl),
            candidate_count: 0,
            rejected_candidate_count: 0,
            candidate_provenance_refs: Vec::new(),
            candidate_scorecards: Vec::new(),
            phrase_memory_distance: 0.0,
            fallback_reason: Some("source_evidence_untrusted".into()),
        });
    state.refresh_view();

    assert_eq!(state.runtime.mc202_render.tempo_bpm, 126.0);
    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Idle);
    assert_eq!(state.runtime.mc202_render.routing, Mc202RenderRouting::Silent);
}

#[test]
fn explicit_bpm_rejects_a_mismatched_probe_grid() {
    let graph = manual_confirm_source_window_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let error = confirm_explicit_source_bpm(&mut state, 140.0).expect_err("reject mismatch");

    assert!(error.to_string().contains("does not match Rust timing candidate"));
    assert!(state.session.runtime_state.source_timing.confirmed_grid.is_none());
    assert!(state.queue.pending_actions().is_empty());
}

#[test]
fn tempo_guided_grid_preserves_analyzer_evidence_and_restores_without_fake_user_confirmation() {
    let mut graph = manual_confirm_source_window_graph();
    let analyzer_hypothesis_id = graph
        .timing
        .primary_hypothesis_id
        .clone()
        .expect("analyzer hypothesis");
    let input = tempo_guided_test_input(0.137, 130.0);

    let evidence = install_tempo_guided_source_grid(&mut graph, &input, 130.0)
        .expect("derive source-backed phase");

    assert_eq!(evidence.decision, TempoGuidedTimingDecision::Selected);
    assert!((evidence.selected_downbeat_seconds.expect("downbeat") - 0.137).abs() < 0.001);
    let primary = graph.timing.primary_hypothesis().expect("guided primary");
    assert_eq!(primary.kind, TimingHypothesisKind::TempoGuided);
    assert!((primary.bpm - 130.0).abs() < f32::EPSILON);
    assert!(graph
        .timing
        .hypotheses
        .iter()
        .any(|hypothesis| hypothesis.hypothesis_id == analyzer_hypothesis_id));

    let temp = tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let graph_path = temp.path().join("graph.json");
    let session = sample_session(&graph);
    save_session_json(&session_path, &session).expect("save session");
    save_source_graph_json(&graph_path, &graph).expect("save graph");

    let restored =
        JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("restore guided grid");
    let restored_graph = restored.source_graph.as_ref().expect("restored graph");
    assert_eq!(
        restored_graph.timing.primary_hypothesis(),
        graph.timing.primary_hypothesis()
    );
    assert_eq!(
        riotbox_core::view::jam::source_timing_consumer_readiness(
            Some(restored_graph),
            &restored.session,
        ),
        riotbox_core::view::jam::SourceTimingConsumerReadiness::AnalyzerLocked
    );
    assert!(restored.session.runtime_state.source_timing.confirmed_grid.is_none());
    assert!(!restored
        .session
        .action_log
        .actions
        .iter()
        .any(|action| action.command == ActionCommand::SourceTimingConfirmGrid));
}

#[test]
fn tempo_guided_grid_fails_closed_when_source_accents_do_not_select_a_phase() {
    let mut graph = manual_confirm_source_window_graph();
    let original_timing = graph.timing.clone();
    let mut input = tempo_guided_test_input(0.137, 130.0);
    input.onset_strengths.fill(1.0);

    let error = install_tempo_guided_source_grid(&mut graph, &input, 130.0)
        .expect_err("reject ambiguous source phase");

    assert!(error.to_string().contains("ambiguous_phase"));
    assert_eq!(graph.timing, original_timing);
}

#[test]
fn live_ingest_explicit_bpm_persists_graph_confirmation_and_restore_identity() {
    let temp = tempdir().expect("tempdir");
    let source_path = temp.path().join("ingest-128.wav");
    let session_path = temp.path().join("session.json");
    let graph_path = temp.path().join("graph.json");
    riotbox_audio::source_audio::write_interleaved_pcm16_wav(
        &source_path,
        32_768,
        1,
        &accented_drum_grid_samples(15_360, 32),
    )
    .expect("write source");

    let state = JamAppState::analyze_source_file_to_json_with_source_bpm_confirmation(
        &source_path,
        &session_path,
        Some(graph_path.clone()),
        sidecar_script_path(),
        73,
        Some(128.0),
    )
    .expect("ingest and confirm source timing");
    let confirmed = state
        .session
        .runtime_state
        .source_timing
        .confirmed_grid
        .as_ref()
        .expect("persisted confirmation");
    assert_eq!(
        confirmed.hypothesis_id.as_deref(),
        state
            .source_graph
            .as_ref()
            .and_then(|graph| graph.timing.primary_hypothesis_id.as_deref())
    );
    assert_eq!(
        state.session.action_log.actions.last().map(|action| action.command),
        Some(ActionCommand::SourceTimingConfirmGrid)
    );

    let restored = JamAppState::from_json_files(&session_path, Some(&graph_path))
        .expect("restore confirmed timing state");
    assert_eq!(
        restored.session.runtime_state.source_timing.confirmed_grid,
        state.session.runtime_state.source_timing.confirmed_grid
    );
    assert!((
        super::transport_helpers::trusted_source_timing_bpm(
            &restored.session,
            restored.source_graph.as_ref(),
        )
        .expect("trusted restored BPM")
            - 128.0
    )
        .abs()
        <= 1.0);
    assert!(
        restored
            .source_graph
            .as_ref()
            .expect("restored graph")
            .provenance
            .provider_set
            .contains(&"riotbox-rust-source-timing-probe".into())
    );
}

#[test]
fn tonal_live_ingest_requires_and_persists_explicit_manual_grid_phase() {
    let temp = tempdir().expect("tempdir");
    let source_path = temp.path().join("tonal-120.wav");
    let sample_rate = 48_000_u32;
    let samples = (0..sample_rate as usize * 4)
        .map(|frame| {
            let phase = frame as f32 * 440.0 * std::f32::consts::TAU / sample_rate as f32;
            phase.sin() * 0.2
        })
        .collect::<Vec<_>>();
    riotbox_audio::source_audio::write_interleaved_pcm16_wav(
        &source_path,
        sample_rate,
        1,
        &samples,
    )
    .expect("write tonal source");

    let bpm_only_error = JamAppState::analyze_source_file_to_json_with_source_bpm_confirmation(
        &source_path,
        temp.path().join("bpm-only-session.json"),
        Some(temp.path().join("bpm-only-graph.json")),
        sidecar_script_path(),
        74,
        Some(120.0),
    )
    .expect_err("tonal source must not gain an inferred grid from BPM alone");
    assert!(bpm_only_error
        .to_string()
        .contains("could not derive a trusted source-backed downbeat phase"));
    assert!(bpm_only_error.to_string().contains("insufficient_onsets"));

    let session_path = temp.path().join("manual-session.json");
    let graph_path = temp.path().join("manual-graph.json");
    let state = JamAppState::analyze_source_file_to_json_with_source_timing_confirmation(
        &source_path,
        &session_path,
        Some(graph_path.clone()),
        sidecar_script_path(),
        74,
        Some(120.0),
        Some(0.0),
    )
    .expect("manual tonal source grid");

    let graph = state.source_graph.as_ref().expect("source graph");
    let primary = graph.timing.primary_hypothesis().expect("manual primary");
    assert_eq!(primary.kind, TimingHypothesisKind::Manual);
    assert!(primary.hypothesis_id.starts_with("manual-source-grid-v1-"));
    assert_eq!(primary.bar_grid[0].start_seconds, 0.0);
    assert!(primary
        .provenance
        .contains(&"musician-manual-source-grid.v1".into()));
    assert_eq!(
        riotbox_core::view::jam::source_timing_consumer_readiness(Some(graph), &state.session),
        riotbox_core::view::jam::SourceTimingConsumerReadiness::UserConfirmed
    );
    assert_eq!(
        state.session.action_log.actions.last().map(|action| action.command),
        Some(ActionCommand::SourceTimingConfirmGrid)
    );

    let restored = JamAppState::from_json_files(&session_path, Some(&graph_path))
        .expect("restore manual tonal timing");
    let restored_primary = restored
        .source_graph
        .as_ref()
        .and_then(|graph| graph.timing.primary_hypothesis())
        .expect("restored manual primary");
    assert_eq!(restored_primary, primary);
    assert_eq!(
        restored.session.runtime_state.source_timing.confirmed_grid,
        state.session.runtime_state.source_timing.confirmed_grid
    );
}

fn accented_drum_grid_samples(beat_frames: usize, beats: usize) -> Vec<f32> {
    let mut samples = vec![0.0_f32; beat_frames * beats];
    for beat in 0..beats {
        let start = beat * beat_frames;
        let amplitude = if beat % 4 == 0 { 1.0 } else { 0.45 };
        add_timing_impulse(&mut samples, start, 96, amplitude);
        add_timing_impulse(&mut samples, start + beat_frames / 2, 32, 0.08);
    }
    samples
}

fn tempo_guided_test_input(
    downbeat_seconds: f32,
    bpm: f32,
) -> SourceTimingProbeBpmCandidateInput {
    let seconds_per_beat = 60.0 / bpm;
    let mut onset_times_seconds = Vec::new();
    let mut onset_strengths = Vec::new();
    for beat in 0..32 {
        onset_times_seconds.push(downbeat_seconds + beat as f32 * seconds_per_beat);
        onset_strengths.push(if beat % 4 == 0 { 1.0 } else { 0.45 });
    }
    SourceTimingProbeBpmCandidateInput {
        source_id: "src-1".into(),
        duration_seconds: downbeat_seconds + 32.0 * seconds_per_beat,
        onset_times_seconds,
        onset_strengths,
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
    }
}

fn add_timing_impulse(samples: &mut [f32], start: usize, frames: usize, amplitude: f32) {
    let end = start.saturating_add(frames).min(samples.len());
    for sample in samples.iter_mut().take(end).skip(start) {
        *sample = amplitude;
    }
}
