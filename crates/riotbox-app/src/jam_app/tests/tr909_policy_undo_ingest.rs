#[test]
fn pattern_adoption_can_be_derived_without_pattern_ref() {
    let graph = sample_graph();

    let mut support_session = sample_session(&graph);
    support_session
        .runtime_state
        .lane_state
        .tr909
        .reinforcement_mode = Some(Tr909ReinforcementModeState::SourceSupport);
    support_session.runtime_state.lane_state.tr909.pattern_ref = None;
    let support_state =
        JamAppState::from_parts(support_session, Some(graph.clone()), ActionQueue::new());
    assert_eq!(
        support_state.runtime.tr909_render.pattern_adoption,
        Some(Tr909PatternAdoption::MainlineDrive)
    );

    let mut takeover_session = sample_session(&graph);
    takeover_session
        .runtime_state
        .lane_state
        .tr909
        .takeover_enabled = true;
    takeover_session
        .runtime_state
        .lane_state
        .tr909
        .takeover_profile = Some(Tr909TakeoverProfileState::ControlledPhraseTakeover);
    takeover_session.runtime_state.lane_state.tr909.pattern_ref = None;
    let takeover_state = JamAppState::from_parts(takeover_session, Some(graph), ActionQueue::new());
    assert_eq!(
        takeover_state.runtime.tr909_render.pattern_adoption,
        Some(Tr909PatternAdoption::TakeoverGrid)
    );
    assert_eq!(
        takeover_state.runtime.tr909_render.phrase_variation,
        Some(Tr909PhraseVariation::PhraseLift)
    );
}

#[test]
fn phrase_variation_tracks_phrase_context_and_release_patterns() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.tr909.reinforcement_mode =
        Some(Tr909ReinforcementModeState::SourceSupport);
    session.runtime_state.lane_state.tr909.pattern_ref = Some("release-scene-1".into());
    let release_state =
        JamAppState::from_parts(session.clone(), Some(graph.clone()), ActionQueue::new());
    assert_eq!(
        release_state.runtime.tr909_render.phrase_variation,
        Some(Tr909PhraseVariation::PhraseRelease)
    );

    session.runtime_state.lane_state.tr909.pattern_ref = Some("scene-1-main".into());
    session.runtime_state.transport.position_beats = 64.0;
    let drive_state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    assert_eq!(
        drive_state.runtime.tr909_render.phrase_variation,
        Some(Tr909PhraseVariation::PhraseAnchor)
    );
}

#[test]
fn committed_state_fixture_backed_render_projections_hold() {
    let fixtures: Vec<RenderProjectionFixture> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/tr909_committed_render_projection.json"
    ))
    .expect("parse committed render projection fixture");

    let mut graph = sample_graph();
    graph.sections.push(Section {
        section_id: SectionId::from("section-b"),
        label_hint: SectionLabelHint::Break,
        start_seconds: 16.0,
        end_seconds: 32.0,
        bar_start: 9,
        bar_end: 16,
        energy_class: EnergyClass::Medium,
        confidence: 0.85,
        tags: vec!["break".into()],
    });
    for fixture in fixtures {
        let mut session = sample_session(&graph);
        session.runtime_state.transport.position_beats = fixture.transport_position_beats;
        if let Some(scene_context) = fixture.scene_context.as_deref() {
            let scene_id = SceneId::from(scene_context);
            session.runtime_state.scene_state.active_scene = Some(scene_id.clone());
            session.runtime_state.transport.current_scene = Some(scene_id);
        }
        session.runtime_state.lane_state.tr909.reinforcement_mode =
            Some(fixture.reinforcement_mode);
        session.runtime_state.lane_state.tr909.takeover_enabled = fixture.takeover_enabled;
        session.runtime_state.lane_state.tr909.takeover_profile = fixture.takeover_profile;
        session.runtime_state.lane_state.tr909.pattern_ref = fixture.pattern_ref.clone();

        let state = JamAppState::from_parts(session, Some(graph.clone()), ActionQueue::new());

        assert_eq!(
            state.runtime.tr909_render.mode.label(),
            fixture.expected_mode,
            "{} render mode drifted",
            fixture.name
        );
        assert_eq!(
            state.runtime.tr909_render.routing.label(),
            fixture.expected_routing,
            "{} render routing drifted",
            fixture.name
        );
        assert_eq!(
            state
                .runtime
                .tr909_render
                .pattern_adoption
                .map(|pattern| pattern.label().to_string()),
            fixture.expected_pattern_adoption,
            "{} pattern adoption drifted",
            fixture.name
        );
        assert_eq!(
            state
                .runtime
                .tr909_render
                .phrase_variation
                .map(|variation| variation.label().to_string()),
            fixture.expected_phrase_variation,
            "{} phrase variation drifted",
            fixture.name
        );
        assert_eq!(
            state
                .runtime
                .tr909_render
                .source_support_profile
                .map(|profile| profile.label().to_string()),
            fixture.expected_source_support_profile,
            "{} support profile drifted",
            fixture.name
        );
        assert_eq!(
            state
                .runtime
                .tr909_render
                .source_support_context
                .map(|context| context.label().to_string()),
            fixture.expected_source_support_context,
            "{} support context drifted",
            fixture.name
        );
        assert_eq!(
            state.runtime_view.tr909_render_support_accent, fixture.expected_support_accent,
            "{} support accent drifted",
            fixture.name
        );
        assert_eq!(
            state
                .runtime
                .tr909_render
                .takeover_profile
                .map(|profile| profile.label().to_string()),
            fixture.expected_takeover_profile,
            "{} takeover profile drifted",
            fixture.name
        );
    }
}

#[test]
fn undo_marks_last_undoable_action_and_appends_undo_marker() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let undo_action = state.undo_last_action(500).expect("undo latest action");

    assert_eq!(undo_action.command, ActionCommand::UndoLast);
    assert_eq!(state.session.action_log.actions.len(), 2);
    assert_eq!(
        state.session.action_log.actions[0].status,
        ActionStatus::Undone
    );
    assert_eq!(
        state.session.action_log.actions[1].command,
        ActionCommand::UndoLast
    );
    assert_eq!(state.jam_view.recent_actions[0].status, "committed");
    assert_eq!(state.jam_view.recent_actions[1].status, "undone");
}

#[test]
fn saving_with_pending_tr909_fill_does_not_persist_committed_lane_state() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph_path = dir.path().join("source-graph.json");
    let graph = sample_graph();
    let session = sample_session(&graph);

    save_session_json(&session_path, &session).expect("save session fixture");
    save_source_graph_json(&graph_path, &graph).expect("save graph fixture");

    let mut state =
        JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("load app state");
    state.queue_tr909_fill(700);

    assert!(state.jam_view.lanes.tr909_fill_armed_next_bar);
    assert!(
        !state
            .session
            .runtime_state
            .lane_state
            .tr909
            .fill_armed_next_bar
    );

    state.save().expect("save app state");

    let persisted_session = load_session_json(&session_path).expect("reload session");
    let reloaded =
        JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("reload app");

    assert!(
        !persisted_session
            .runtime_state
            .lane_state
            .tr909
            .fill_armed_next_bar
    );
    assert!(
        !reloaded
            .session
            .runtime_state
            .lane_state
            .tr909
            .fill_armed_next_bar
    );
    assert!(!reloaded.jam_view.lanes.tr909_fill_armed_next_bar);
    assert_eq!(reloaded.queue.pending_actions().len(), 0);
}

#[test]
fn ingests_source_file_through_sidecar_and_persists_state() {
    let dir = tempdir().expect("create temp dir");
    let source_path = dir.path().join("input.wav");
    let session_path = dir.path().join("sessions").join("session.json");
    let graph_path = dir.path().join("graphs").join("source-graph.json");

    write_pcm16_wave(&source_path, 44_100, 2, 2.0);

    let state = JamAppState::analyze_source_file_to_json(
        &source_path,
        &session_path,
        Some(graph_path.clone()),
        sidecar_script_path(),
        29,
    )
    .expect("ingest source file");

    assert_eq!(state.runtime_view.sidecar_status, "ready");
    assert_eq!(state.runtime_view.sidecar_version.as_deref(), Some("0.1.0"));
    assert_eq!(
        state
            .source_graph
            .as_ref()
            .map(|graph| graph.source.path.clone()),
        Some(source_path.to_string_lossy().into_owned())
    );
    assert_eq!(state.session.source_refs.len(), 1);
    assert_eq!(state.session.source_graph_refs.len(), 1);
    assert_eq!(state.session.runtime_state.mixer_state.music_level, 0.64);
    assert_eq!(state.jam_view.scene.scene_count, 2);
    assert_eq!(
        state.jam_view.scene.active_scene.as_deref(),
        Some("scene-01-intro")
    );
    assert_eq!(
        state.jam_view.scene.active_scene_energy.as_deref(),
        Some("medium")
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .scene_state
            .scenes
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["scene-01-intro".to_string(), "scene-02-drop".to_string()]
    );
    assert_eq!(
        state.session.source_graph_refs[0].storage_mode,
        GraphStorageMode::External
    );
    assert_eq!(
        state.session.source_graph_refs[0].external_path.as_deref(),
        Some(graph_path.to_string_lossy().as_ref())
    );
    assert!(session_path.exists());
    assert!(graph_path.exists());

    let persisted_graph = load_source_graph_json(&graph_path).expect("reload graph");
    assert_eq!(
        persisted_graph.provenance.provider_set,
        vec!["decoded.wav_baseline"]
    );
    assert_eq!(persisted_graph.provenance.analysis_seed, 29);
    assert_eq!(persisted_graph.source.sample_rate, 44_100);
    assert_eq!(persisted_graph.source.channel_count, 2);
    assert!(persisted_graph.source.duration_seconds >= 1.9);
    assert!(persisted_graph.timing.bpm_estimate.is_some());
    let persisted_session = load_session_json(&session_path).expect("reload session");
    assert_eq!(
        persisted_session
            .runtime_state
            .scene_state
            .scenes
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["scene-01-intro".to_string(), "scene-02-drop".to_string()]
    );
    assert_eq!(
        persisted_session.runtime_state.scene_state.active_scene,
        Some(SceneId::from("scene-01-intro"))
    );
}

#[test]
fn ingest_defaults_to_embedded_graph_storage_when_no_external_path_is_requested() {
    let dir = tempdir().expect("create temp dir");
    let source_path = dir.path().join("input.wav");
    let session_path = dir.path().join("sessions").join("session.json");

    write_pcm16_wave(&source_path, 44_100, 2, 2.0);

    let state = JamAppState::analyze_source_file_to_json(
        &source_path,
        &session_path,
        None,
        sidecar_script_path(),
        31,
    )
    .expect("ingest source file");

    assert_eq!(state.session.source_graph_refs.len(), 1);
    assert_eq!(state.session.runtime_state.mixer_state.music_level, 0.64);
    assert_eq!(
        state.session.source_graph_refs[0].storage_mode,
        GraphStorageMode::Embedded
    );
    assert!(state.session.source_graph_refs[0].external_path.is_none());
    assert!(state.session.source_graph_refs[0].embedded_graph.is_some());
    assert!(session_path.exists());
}

#[test]
fn ingest_surfaces_missing_source_file_as_sidecar_error() {
    let dir = tempdir().expect("create temp dir");
    let source_path = dir.path().join("missing.wav");
    let session_path = dir.path().join("sessions").join("session.json");
    let graph_path = dir.path().join("graphs").join("source-graph.json");

    let error = JamAppState::analyze_source_file_to_json(
        &source_path,
        &session_path,
        Some(graph_path.clone()),
        sidecar_script_path(),
        29,
    )
    .expect_err("missing source should fail");

    match error {
        JamAppError::Io(io_error) => {
            assert_eq!(io_error.kind(), io::ErrorKind::NotFound);
        }
        JamAppError::Sidecar(SidecarClientError::Sidecar(payload)) => {
            assert_eq!(payload.code, "source_missing");
        }
        other => panic!("unexpected error: {other}"),
    }
}

