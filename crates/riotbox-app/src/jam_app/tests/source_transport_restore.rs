#[test]
fn source_transport_capture_projection_survives_save_restore_with_confirmed_grid() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("sessions").join("session.json");
    let graph_path = dir.path().join("graphs").join("source-graph.json");
    let graph = manual_confirm_source_map_graph();
    let original_timing = graph.timing.clone();
    let mut session = sample_session(&graph);
    session.runtime_state.transport.is_playing = true;
    session.runtime_state.transport.position_beats = 5.25;
    session.runtime_state.source_monitor.mode = SourceMonitorMode::Blend;
    session.runtime_state.capture.length_intent = CaptureLengthIntent::OneBar;
    session.runtime_state.capture.length_set_by_action = Some(ActionId(77));
    session.runtime_state.capture.length_set_at = Some(777);
    session.runtime_state.source_timing.confirmed_grid =
        Some(SourceTimingGridConfirmationState {
            source_id: graph.source.source_id.clone(),
            hypothesis_id: graph.timing.primary_hypothesis_id.clone(),
            confirmed_by_action: ActionId(78),
            confirmed_at: 778,
        });
    let state = JamAppState::from_parts(session, Some(graph.clone()), ActionQueue::new());
    let source_map_before_save = state.jam_view.source.source_map.clone();

    assert_eq!(
        source_map_before_save.mode,
        riotbox_core::view::jam::SourceMapModeView::BarGrid
    );
    assert_eq!(source_map_before_save.trust_label, "grid confirmed");
    assert!(
        source_map_before_save.capture_range_row.contains('['),
        "confirmed grid should allow bar-accurate capture projection"
    );
    save_session_json(&session_path, &state.session).expect("save confirmed source session");
    save_source_graph_json(&graph_path, &graph).expect("save confirmed source graph");

    let restored =
        JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("restore app state");

    assert!(restored.session.runtime_state.transport.is_playing);
    assert_eq!(restored.session.runtime_state.transport.position_beats, 5.25);
    assert_eq!(
        restored.session.runtime_state.source_monitor.mode,
        SourceMonitorMode::Blend
    );
    assert_eq!(
        restored.session.runtime_state.capture.length_intent,
        CaptureLengthIntent::OneBar
    );
    assert_eq!(
        restored
            .session
            .runtime_state
            .source_timing
            .confirmed_grid,
        state.session.runtime_state.source_timing.confirmed_grid
    );
    assert_eq!(
        restored.source_graph.as_ref().expect("source graph").timing,
        original_timing,
        "user confirmation must not rewrite Source Graph timing evidence"
    );
    assert_eq!(restored.jam_view.source.source_map, source_map_before_save);
}

#[test]
fn unconfirmed_source_transport_capture_range_stays_unavailable_after_restore() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("sessions").join("session.json");
    let graph_path = dir.path().join("graphs").join("source-graph.json");
    let graph = manual_confirm_source_map_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.transport.is_playing = true;
    session.runtime_state.transport.position_beats = 5.25;
    session.runtime_state.capture.length_intent = CaptureLengthIntent::OneBar;
    let state = JamAppState::from_parts(session, Some(graph.clone()), ActionQueue::new());
    let source_map_before_save = state.jam_view.source.source_map.clone();

    assert_eq!(
        source_map_before_save.mode,
        riotbox_core::view::jam::SourceMapModeView::TimeFallback
    );
    assert_eq!(source_map_before_save.trust_label, "needs confirm");
    assert_eq!(source_map_before_save.capture_range_row, ".".repeat(32));
    assert_eq!(
        source_map_before_save.capture_hint,
        "cap listen first | map time fallback | no bar-accurate claim"
    );
    save_session_json(&session_path, &state.session).expect("save unconfirmed source session");
    save_source_graph_json(&graph_path, &graph).expect("save unconfirmed source graph");

    let restored =
        JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("restore app state");

    assert!(restored.session.runtime_state.source_timing.confirmed_grid.is_none());
    assert_eq!(restored.jam_view.source.source_map, source_map_before_save);
}

fn manual_confirm_source_map_graph() -> SourceGraph {
    let mut graph = source_map_navigation_graph();
    graph.timing.quality = TimingQuality::Low;
    graph.timing.degraded_policy = TimingDegradedPolicy::ManualConfirm;
    graph.timing.bpm_confidence = 0.72;
    graph.timing.hypotheses[0].quality = TimingQuality::Low;
    graph.timing.hypotheses[0].confidence = 0.72;
    graph.timing.hypotheses[0].score = 0.68;
    graph
}
