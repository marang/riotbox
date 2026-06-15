#[test]
fn loads_and_saves_jam_app_state_from_files() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("sessions").join("session.json");
    let graph_path = dir.path().join("graphs").join("source-graph.json");

    let graph = sample_graph();
    let session = sample_session(&graph);
    save_session_json(&session_path, &session).expect("save session fixture");
    save_source_graph_json(&graph_path, &graph).expect("save graph fixture");

    let mut state =
        JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("load app state");
    assert!(state.jam_view.transport.is_playing);
    assert_eq!(state.jam_view.source.section_count, 1);

    state.session.notes = Some("updated".into());
    state.refresh_view();
    state.save().expect("save app state");

    let persisted_session = load_session_json(&session_path).expect("reload session");
    let persisted_graph = load_source_graph_json(&graph_path).expect("reload graph");

    assert_eq!(persisted_session.notes.as_deref(), Some("updated"));
    assert_eq!(persisted_graph, graph);
}

#[test]
fn loads_pcm24_source_audio_cache_from_app_files() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("sessions").join("session.json");
    let graph_path = dir.path().join("graphs").join("source-graph.json");
    let source_path = dir.path().join("source24.wav");

    write_pcm24_wave(&source_path, 48_000, 2);

    let mut graph = sample_graph();
    graph.source.path = source_path.to_string_lossy().into_owned();
    graph.source.sample_rate = 48_000;
    graph.source.channel_count = 2;
    graph.source.duration_seconds = 2.0 / 48_000.0;
    let session = sample_session(&graph);
    save_session_json(&session_path, &session).expect("save session fixture");
    save_source_graph_json(&graph_path, &graph).expect("save graph fixture");

    let state =
        JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("load app state");
    let cache = state
        .source_audio_cache
        .as_ref()
        .expect("source audio cache");

    assert_eq!(cache.sample_rate, 48_000);
    assert_eq!(cache.channel_count, 2);
    assert_eq!(cache.frame_count(), 2);
    assert_eq!(cache.interleaved_samples()[0], -1.0);
    assert_eq!(cache.interleaved_samples()[1], 0.0);
    assert!((cache.interleaved_samples()[2] - 1.0).abs() < 0.000001);
    assert!(
        state
            .runtime_view
            .runtime_warnings
            .iter()
            .all(|warning| !warning.contains("source audio"))
    );
}

#[test]
fn source_audio_load_failure_surfaces_runtime_warning() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("sessions").join("session.json");
    let graph_path = dir.path().join("graphs").join("source-graph.json");
    let missing_source_path = dir.path().join("missing-source.wav");

    let mut graph = sample_graph();
    graph.source.path = missing_source_path.to_string_lossy().into_owned();
    let session = sample_session(&graph);
    save_session_json(&session_path, &session).expect("save session fixture");
    save_source_graph_json(&graph_path, &graph).expect("save graph fixture");

    let state =
        JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("load app state");

    assert!(state.source_audio_cache.is_none());
    assert_eq!(
        state.runtime_view.source_monitor_audio_route,
        "source_unavailable"
    );
    assert!(
        state
            .runtime_view
            .runtime_warnings
            .iter()
            .any(|warning| warning.contains("source audio unavailable for source monitor")
                && warning.contains("missing-source.wav")
                && warning.contains("source audio I/O failed"))
    );
}

#[test]
fn invalid_source_audio_surfaces_decode_warning() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("sessions").join("session.json");
    let graph_path = dir.path().join("graphs").join("source-graph.json");
    let invalid_source_path = dir.path().join("not-a-wave.wav");

    fs::write(&invalid_source_path, b"not a wave").expect("write invalid source fixture");

    let mut graph = sample_graph();
    graph.source.path = invalid_source_path.to_string_lossy().into_owned();
    let session = sample_session(&graph);
    save_session_json(&session_path, &session).expect("save session fixture");
    save_source_graph_json(&graph_path, &graph).expect("save graph fixture");

    let state =
        JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("load app state");

    assert!(state.source_audio_cache.is_none());
    assert!(
        state
            .runtime_view
            .runtime_warnings
            .iter()
            .any(|warning| warning.contains("source audio unavailable for source monitor")
                && warning.contains("invalid WAV source audio"))
    );
}

#[test]
fn loads_embedded_graph_session_without_separate_graph_file() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("sessions").join("session.json");

    let graph = sample_graph();
    let session = sample_session(&graph);
    save_session_json(&session_path, &session).expect("save embedded session fixture");

    let state = JamAppState::from_json_files(&session_path, None::<&Path>).expect("load app state");

    assert_eq!(state.source_graph, Some(graph));
    assert_eq!(state.jam_view.source.section_count, 1);
}

#[test]
fn save_persists_embedded_graph_into_session_file() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("sessions").join("session.json");

    let graph = sample_graph();
    let session = sample_session(&graph);
    save_session_json(&session_path, &session).expect("save embedded session fixture");

    let mut state =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect("load app state");
    state.session.notes = Some("updated embedded session".into());
    state
        .source_graph
        .as_mut()
        .expect("source graph")
        .source
        .duration_seconds = 121.0;
    state.save().expect("save app state");

    let persisted_session = load_session_json(&session_path).expect("reload session");
    let persisted_graph = persisted_session.source_graph_refs[0]
        .embedded_graph
        .clone()
        .expect("embedded graph should persist");

    assert_eq!(
        persisted_session.notes.as_deref(),
        Some("updated embedded session")
    );
    assert_eq!(persisted_graph.source.duration_seconds, 121.0);
    assert_eq!(
        persisted_session.source_graph_refs[0].graph_hash,
        crate::jam_app::persistence::source_graph_hash(&persisted_graph)
            .expect("hash persisted graph")
    );
    JamAppState::from_json_files(&session_path, None::<&Path>)
        .expect("reload with refreshed source graph hash");
}

#[test]
fn save_materializes_payload_for_latest_explicit_snapshot_and_restore_uses_it() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("sessions").join("session.json");

    let graph = sample_graph();
    let session = sample_session(&graph);
    save_session_json(&session_path, &session).expect("save embedded session fixture");

    let state =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect("load app state");
    assert!(state.session.snapshots[0].payload.is_none());

    state.save().expect("save app state");

    let persisted_session = load_session_json(&session_path).expect("reload session");
    let payload = persisted_session.snapshots[0]
        .payload
        .as_ref()
        .expect("latest explicit snapshot gets payload");
    assert_eq!(payload.snapshot_id, persisted_session.snapshots[0].snapshot_id);
    assert_eq!(payload.action_cursor, persisted_session.snapshots[0].action_cursor);
    assert_eq!(payload.runtime_state, state.session.runtime_state);

    let mut restored =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect("reload app state");
    assert_eq!(
        restored.runtime_view.replay_restore_payload,
        "payload ready | snapshot restore ok"
    );
    assert_eq!(
        restored.runtime_view.replay_restore_status,
        "ready: snapshot current"
    );

    restored.session.runtime_state = Default::default();
    let report = restored
        .apply_restore_target_from_snapshot_payload(persisted_session.action_log.actions.len())
        .expect("produced snapshot payload hydrates through app restore");

    assert_eq!(report.applied_action_ids, Vec::<ActionId>::new());
    assert_eq!(restored.session.runtime_state, state.session.runtime_state);
}

#[test]
fn runtime_view_updates_from_audio_and_sidecar_state() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.source_monitor.mode = SourceMonitorMode::Blend;
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.set_audio_health(sample_audio_health(AudioRuntimeLifecycle::Running));
    state.set_sidecar_state(SidecarState::Ready {
        version: Some("0.1.0".into()),
        transport: "stdio-ndjson".into(),
    });

    assert_eq!(state.runtime_view.audio_status, "running");
    assert_eq!(state.runtime_view.audio_callback_count, 18);
    assert_eq!(state.runtime_view.sidecar_status, "ready");
    assert_eq!(state.runtime_view.sidecar_version.as_deref(), Some("0.1.0"));
    assert_eq!(state.runtime_view.source_monitor_mode, "blend");
    assert!(state.runtime_view.runtime_warnings.iter().any(
        |warning| warning
            == "MC-202 source phrase unavailable; primitive fallback is not routed to music_bus_bass"
    ));
}

#[test]
fn source_monitor_audio_route_tracks_source_cache_and_output_format() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.source_monitor.mode = SourceMonitorMode::Source;
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(state.runtime_view.source_monitor_audio_route, "source_unavailable");

    state.source_audio_cache = Some(
        SourceAudioCache::from_interleaved_samples(
            "source.wav",
            44_100,
            1,
            vec![0.25, 0.5, 0.75, 1.0],
        )
        .expect("source cache"),
    );
    state.refresh_view();

    assert_eq!(state.runtime_view.source_monitor_audio_route, "source_only");

    state.set_audio_health(sample_audio_health(AudioRuntimeLifecycle::Running));
    assert_eq!(state.runtime_view.source_monitor_audio_route, "source_only");

    state
        .source_audio_cache
        .as_mut()
        .expect("source audio cache")
        .sample_rate = 48_000;
    state.refresh_view();

    assert_eq!(state.runtime_view.source_monitor_audio_route, "source_unavailable");
    assert!(
        state
            .runtime_view
            .runtime_warnings
            .iter()
            .any(|warning| warning.contains("source monitor unavailable")
                && warning.contains("48000 Hz"))
    );
}

#[test]
fn source_monitor_mode_queues_commits_and_surfaces_in_runtime_view() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(state.runtime_view.source_monitor_mode, "source");
    assert_eq!(
        state.queue_source_monitor_mode(SourceMonitorMode::Riotbox, 100),
        QueueControlResult::Enqueued
    );
    assert_eq!(
        state.queue_source_monitor_mode(SourceMonitorMode::Riotbox, 101),
        QueueControlResult::AlreadyPending
    );
    assert_eq!(
        state.queue_source_monitor_mode(SourceMonitorMode::Source, 102),
        QueueControlResult::AlreadyPending
    );

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Immediate,
            beat_index: 0,
            bar_index: 0,
            phrase_index: 0,
            scene_id: None,
        },
        120,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state.session.runtime_state.source_monitor.mode,
        SourceMonitorMode::Riotbox
    );
    assert_eq!(state.runtime_view.source_monitor_mode, "riotbox");
    assert_eq!(
        state.queue_source_monitor_mode(SourceMonitorMode::Riotbox, 121),
        QueueControlResult::AlreadyInState
    );
}

#[test]
fn runtime_view_surfaces_faulted_and_degraded_states() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.set_audio_health(sample_audio_health(AudioRuntimeLifecycle::Faulted));
    state.set_sidecar_state(SidecarState::Degraded {
        reason: "worker restart pending".into(),
    });

    assert_eq!(state.runtime_view.audio_status, "faulted");
    assert_eq!(
        state.runtime_view.audio_last_error.as_deref(),
        Some("stream stalled")
    );
    assert_eq!(state.runtime_view.sidecar_status, "degraded");
    assert!(
        state
            .runtime_view
            .runtime_warnings
            .iter()
            .any(|warning| warning == "audio runtime faulted")
    );
    assert!(
        state
            .runtime_view
            .runtime_warnings
            .iter()
            .any(|warning| warning.contains("sidecar degraded"))
    );
}

#[test]
fn runtime_view_surfaces_tr909_render_diagnostics() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.tr909.takeover_enabled = true;
    session.runtime_state.lane_state.tr909.takeover_profile =
        Some(Tr909TakeoverProfileState::ControlledPhraseTakeover);
    session.runtime_state.lane_state.tr909.pattern_ref = Some("scene-1-main".into());
    session.runtime_state.macro_state.tr909_slam = 0.91;
    session.runtime_state.mixer_state.drum_level = 0.0;
    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(state.runtime_view.tr909_render_mode, "takeover");
    assert_eq!(state.runtime_view.tr909_render_routing, "drum_bus_takeover");
    assert_eq!(state.runtime_view.tr909_render_profile, "controlled_phrase");
    assert_eq!(state.runtime_view.tr909_render_support_context, "unset");
    assert_eq!(state.runtime_view.tr909_render_support_accent, "off");
    assert_eq!(state.runtime_view.tr909_render_support_reason, "unset");
    assert_eq!(
        state.runtime_view.tr909_render_pattern_ref.as_deref(),
        Some("scene-1-main")
    );
    assert_eq!(
        state.runtime_view.tr909_render_pattern_adoption,
        "takeover_grid"
    );
    assert_eq!(
        state.runtime_view.tr909_render_phrase_variation,
        "phrase_lift"
    );
    assert_eq!(
        state.runtime_view.tr909_render_mix_summary,
        "drum bus 0.00 | slam 0.91"
    );
    assert_eq!(
        state.runtime_view.tr909_render_alignment,
        "takeover aligned"
    );
    assert!(
        state
            .runtime_view
            .tr909_render_transport_summary
            .contains("running @ 32.0 beats")
    );
    assert!(
        state
            .runtime_view
            .runtime_warnings
            .iter()
            .any(|warning| warning == "909 render is routed to the drum bus at zero drum level")
    );
}

#[test]
fn runtime_view_surfaces_mc202_render_diagnostics() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.mc202.role = Some(Mc202RoleState::Answer);
    session.runtime_state.lane_state.mc202.phrase_ref = Some("answer-scene-1".into());
    session.runtime_state.macro_state.mc202_touch = 0.82;
    session.runtime_state.mixer_state.music_level = 0.0;
    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Answer);
    assert_eq!(
        state.runtime.mc202_render.routing,
        Mc202RenderRouting::Silent
    );
    assert_eq!(
        state.runtime.mc202_render.phrase_shape,
        Mc202PhraseShape::RootPulse
    );
    assert_eq!(
        state.runtime.mc202_render.contour_hint,
        Mc202ContourHint::Drop
    );
    assert_eq!(
        state.runtime.mc202_render.hook_response,
        Mc202HookResponse::Direct
    );
    assert_eq!(state.runtime_view.mc202_render_mode, "answer");
    assert_eq!(state.runtime_view.mc202_render_routing, "silent");
    assert_eq!(state.runtime_view.mc202_render_phrase_shape, "root_pulse");
    assert_eq!(
        state.runtime_view.mc202_render_mix_summary,
        "music bus 0.00 | touch 0.82 | budget balanced | contour drop | hook direct | source plan timing_untrusted"
    );
    assert!(
        state
            .runtime_view
            .mc202_render_transport_summary
            .contains("running @ 32.0 beats")
    );
    assert!(
        state.runtime_view.runtime_warnings.iter().any(
            |warning| warning
                == "MC-202 source phrase unavailable; primitive fallback is not routed to music_bus_bass"
        )
    );
}

#[test]
fn mc202_render_contour_hint_follows_source_section_context() {
    let mut graph = sample_graph();
    graph.sections[0].label_hint = SectionLabelHint::Build;
    graph.sections[0].energy_class = EnergyClass::Medium;
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.mc202.role = Some(Mc202RoleState::Follower);
    session.runtime_state.lane_state.mc202.phrase_ref = Some("follower-scene-1".into());
    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.runtime.mc202_render.contour_hint,
        Mc202ContourHint::Lift
    );
    assert!(
        state
            .runtime_view
            .mc202_render_mix_summary
            .contains("contour lift")
    );
}

#[test]
fn mc202_hook_section_does_not_inject_synthetic_answer_guardrail() {
    let mut graph = sample_graph();
    graph.sections[0].label_hint = SectionLabelHint::Chorus;
    graph.sections[0].energy_class = EnergyClass::High;
    graph.sections[0].tags.push("hook".into());
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.mc202.role = Some(Mc202RoleState::Follower);
    session.runtime_state.lane_state.mc202.phrase_ref = Some("follower-scene-1".into());
    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.runtime.mc202_render.hook_response,
        Mc202HookResponse::Direct
    );
    assert_eq!(
        state.runtime.mc202_render.note_budget,
        riotbox_audio::mc202::Mc202NoteBudget::Balanced
    );
    assert!(
        state
            .runtime_view
            .mc202_render_mix_summary
            .contains("hook direct")
    );
}
