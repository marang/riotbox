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

fn add_timing_impulse(samples: &mut [f32], start: usize, frames: usize, amplitude: f32) {
    let end = start.saturating_add(frames).min(samples.len());
    for sample in samples.iter_mut().take(end).skip(start) {
        *sample = amplitude;
    }
}
