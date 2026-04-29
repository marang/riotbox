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
    assert_eq!(persisted_graph, graph);
}

#[test]
fn rejects_session_with_multiple_source_refs_in_mvp_mode() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.source_refs.push(SourceRef {
        source_id: SourceId::from("src-2"),
        path_hint: "other.wav".into(),
        content_hash: "hash-2".into(),
        duration_seconds: 64.0,
        decode_profile: "normalized_stereo".into(),
    });
    save_session_json(&session_path, &session).expect("save multi-source session fixture");

    let error =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect_err("load should fail");

    match error {
        JamAppError::InvalidSession(message) => {
            assert!(message.contains("exactly one source reference"));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn rejects_session_with_mismatched_single_source_and_graph_refs() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.source_graph_refs[0].source_id = SourceId::from("src-other");
    save_session_json(&session_path, &session).expect("save mismatched session fixture");

    let error =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect_err("load should fail");

    match error {
        JamAppError::InvalidSession(message) => {
            assert!(message.contains("does not match source graph ref"));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn rejects_session_with_snapshot_cursor_beyond_action_log() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.snapshots[0].action_cursor = session.action_log.actions.len() + 1;
    save_session_json(&session_path, &session).expect("save bad snapshot cursor session");

    let error =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect_err("load should fail");

    match error {
        JamAppError::InvalidSession(message) => {
            assert!(message.contains("snapshot snap-1 action cursor 2"));
            assert!(message.contains("exceeds action log length 1"));
        }
        other => panic!("unexpected error: {other}"),
    }
}

fn sample_commit_record(action_id: ActionId, commit_sequence: u32) -> ActionCommitRecord {
    ActionCommitRecord {
        action_id,
        boundary: CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 8,
            bar_index: 2,
            phrase_index: 0,
            scene_id: Some(SceneId::from("scene-1")),
        },
        commit_sequence,
        committed_at: 200,
    }
}

#[test]
fn loads_session_with_commit_record_referencing_persisted_action() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session
        .action_log
        .commit_records
        .push(sample_commit_record(ActionId(1), 1));
    save_session_json(&session_path, &session).expect("save valid commit-record session");

    let restored =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect("load should pass");

    assert_eq!(restored.session.action_log.commit_records.len(), 1);
    assert_eq!(
        restored.runtime.last_commit_boundary,
        Some(CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 8,
            bar_index: 2,
            phrase_index: 0,
            scene_id: Some(SceneId::from("scene-1")),
        })
    );
}

#[test]
fn rejects_session_with_commit_record_for_missing_action() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session
        .action_log
        .commit_records
        .push(sample_commit_record(ActionId(999), 1));
    save_session_json(&session_path, &session).expect("save orphan commit-record session");

    let error =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect_err("load should fail");

    match error {
        JamAppError::InvalidSession(message) => {
            assert!(message.contains("commit record references missing action a-0999"));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn rejects_session_with_commit_record_for_uncommitted_action() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.action_log.actions[0].status = ActionStatus::Queued;
    session
        .action_log
        .commit_records
        .push(sample_commit_record(ActionId(1), 1));
    save_session_json(&session_path, &session)
        .expect("save non-committed commit-record session");

    let error =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect_err("load should fail");

    match error {
        JamAppError::InvalidSession(message) => {
            assert!(message.contains(
                "commit record references action a-0001 with non-committed status Queued"
            ));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn rejects_session_with_zero_commit_record_sequence() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session
        .action_log
        .commit_records
        .push(sample_commit_record(ActionId(1), 0));
    save_session_json(&session_path, &session).expect("save zero-sequence commit-record session");

    let error =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect_err("load should fail");

    match error {
        JamAppError::InvalidSession(message) => {
            assert!(message.contains("commit record for action a-0001 has invalid sequence 0"));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn rejects_session_with_duplicate_commit_record_sequence_for_boundary() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    let mut second_action = session.action_log.actions[0].clone();
    second_action.id = ActionId(2);
    session.action_log.actions.push(second_action);
    session
        .action_log
        .commit_records
        .push(sample_commit_record(ActionId(1), 1));
    session
        .action_log
        .commit_records
        .push(sample_commit_record(ActionId(2), 1));
    save_session_json(&session_path, &session)
        .expect("save duplicate-sequence commit-record session");

    let error =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect_err("load should fail");

    match error {
        JamAppError::InvalidSession(message) => {
            assert!(message.contains("commit record sequence 1 is duplicated"));
            assert!(message.contains("boundary Bar beat 8 bar 2 phrase 0"));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn rejects_session_with_duplicate_commit_record_for_same_action() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session
        .action_log
        .commit_records
        .push(sample_commit_record(ActionId(1), 1));
    session
        .action_log
        .commit_records
        .push(sample_commit_record(ActionId(1), 2));
    save_session_json(&session_path, &session)
        .expect("save duplicate-action commit-record session");

    let error =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect_err("load should fail");

    match error {
        JamAppError::InvalidSession(message) => {
            assert!(message.contains("commit record is duplicated for action a-0001"));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn runtime_view_updates_from_audio_and_sidecar_state() {
    let graph = sample_graph();
    let session = sample_session(&graph);
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
    assert!(state.runtime_view.runtime_warnings.is_empty());
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
    session.runtime_state.lane_state.mc202.role = Some("answer".into());
    session.runtime_state.lane_state.mc202.phrase_ref = Some("answer-scene-1".into());
    session.runtime_state.macro_state.mc202_touch = 0.82;
    session.runtime_state.mixer_state.music_level = 0.0;
    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Answer);
    assert_eq!(
        state.runtime.mc202_render.routing,
        Mc202RenderRouting::MusicBusBass
    );
    assert_eq!(
        state.runtime.mc202_render.phrase_shape,
        Mc202PhraseShape::AnswerHook
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
    assert_eq!(state.runtime_view.mc202_render_routing, "music_bus_bass");
    assert_eq!(state.runtime_view.mc202_render_phrase_shape, "answer_hook");
    assert_eq!(
        state.runtime_view.mc202_render_mix_summary,
        "music bus 0.00 | touch 0.82 | budget balanced | contour drop | hook direct"
    );
    assert!(
        state
            .runtime_view
            .mc202_render_transport_summary
            .contains("running @ 32.0 beats")
    );
    assert!(
        state.runtime_view.runtime_warnings.iter().any(
            |warning| warning == "MC-202 render is routed to the music bus at zero music level"
        )
    );
}

#[test]
fn mc202_render_contour_hint_follows_source_section_context() {
    let mut graph = sample_graph();
    graph.sections[0].label_hint = SectionLabelHint::Build;
    graph.sections[0].energy_class = EnergyClass::Medium;
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.mc202.role = Some("follower".into());
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
fn mc202_hook_section_uses_answer_space_guardrail() {
    let mut graph = sample_graph();
    graph.sections[0].label_hint = SectionLabelHint::Chorus;
    graph.sections[0].energy_class = EnergyClass::High;
    graph.sections[0].tags.push("hook".into());
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.mc202.role = Some("follower".into());
    session.runtime_state.lane_state.mc202.phrase_ref = Some("follower-scene-1".into());
    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.runtime.mc202_render.hook_response,
        Mc202HookResponse::AnswerSpace
    );
    assert_eq!(
        state.runtime.mc202_render.note_budget,
        riotbox_audio::mc202::Mc202NoteBudget::Sparse
    );
    assert!(
        state
            .runtime_view
            .mc202_render_mix_summary
            .contains("hook answer_space")
    );
}
