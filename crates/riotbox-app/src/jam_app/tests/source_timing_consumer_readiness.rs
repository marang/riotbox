#[test]
fn manual_confirm_capture_does_not_materialize_source_window_until_confirmed() {
    let graph = manual_confirm_source_window_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.queue_capture_bar(300);
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
    assert_eq!(
        riotbox_core::view::jam::SourceTimingSummaryView::from_graph(
            state.source_graph.as_ref().expect("source graph")
        )
        .cue,
        "needs confirm"
    );
    assert!(
        state.session.captures[1].source_window.is_none(),
        "unconfirmed manual-confirm timing must not create bar-accurate source-window reuse"
    );
}

#[test]
fn user_confirmed_manual_grid_allows_capture_source_window() {
    let graph = manual_confirm_source_window_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.source_timing.confirmed_grid =
        Some(SourceTimingGridConfirmationState {
            source_id: graph.source.source_id.clone(),
            hypothesis_id: graph.timing.primary_hypothesis_id.clone(),
            confirmed_by_action: ActionId(42),
            confirmed_at: 390,
        });
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.queue_capture_bar(300);
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
    let source_window = state.session.captures[1]
        .source_window
        .as_ref()
        .expect("confirmed manual grid can drive source-window reuse");
    assert_eq!(source_window.source_id, SourceId::from("src-1"));
    assert!((source_window.start_seconds - 15.238).abs() < 0.01);
    assert!((source_window.end_seconds - 22.857).abs() < 0.01);
    assert_eq!(
        riotbox_core::view::jam::SourceTimingSummaryView::from_graph(
            state.source_graph.as_ref().expect("source graph")
        )
        .cue,
        "needs confirm"
    );
}

fn manual_confirm_source_window_graph() -> SourceGraph {
    let mut graph = sample_graph();
    graph.timing.quality = TimingQuality::Low;
    graph.timing.degraded_policy = TimingDegradedPolicy::ManualConfirm;
    graph.timing.bpm_confidence = 0.72;
    graph.timing.primary_hypothesis_id = Some("primary-manual".into());
    graph.timing.hypotheses = vec![TimingHypothesis {
        hypothesis_id: "primary-manual".into(),
        kind: TimingHypothesisKind::Primary,
        bpm: 126.0,
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
        confidence: 0.72,
        score: 0.68,
        beat_grid: Vec::new(),
        bar_grid: Vec::new(),
        phrase_grid: Vec::new(),
        anchors: Vec::new(),
        drift: Vec::new(),
        groove: Vec::new(),
        quality: TimingQuality::Low,
        warnings: Vec::new(),
        provenance: vec!["source_timing_consumer_readiness.manual_confirm".into()],
    }];
    graph
}
