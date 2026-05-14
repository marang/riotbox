fn sample_graph() -> SourceGraph {
    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: SourceId::from("src-1"),
            path: "input.wav".into(),
            content_hash: "hash-1".into(),
            duration_seconds: 120.0,
            sample_rate: 48_000,
            channel_count: 2,
            decode_profile: DecodeProfile::NormalizedStereo,
        },
        GraphProvenance {
            sidecar_version: "0.1.0".into(),
            provider_set: vec!["beat".into(), "section".into()],
            generated_at: "2026-04-12T18:00:00Z".into(),
            source_hash: "hash-1".into(),
            analysis_seed: 7,
            run_notes: Some("app-test".into()),
        },
    );
    graph.sections.push(Section {
        section_id: SectionId::from("section-a"),
        label_hint: SectionLabelHint::Drop,
        start_seconds: 0.0,
        end_seconds: 16.0,
        bar_start: 1,
        bar_end: 8,
        energy_class: EnergyClass::High,
        confidence: 0.9,
        tags: vec!["main".into()],
    });
    graph.assets.push(Asset {
        asset_id: AssetId::from("asset-a"),
        asset_type: AssetType::LoopWindow,
        start_seconds: 0.0,
        end_seconds: 4.0,
        start_bar: 1,
        end_bar: 2,
        confidence: 0.8,
        tags: vec!["loop".into()],
        source_refs: vec!["src-1".into()],
    });
    graph.candidates.push(Candidate {
        candidate_id: "candidate-a".into(),
        candidate_type: CandidateType::LoopCandidate,
        asset_ref: "asset-a".into(),
        score: 0.88,
        confidence: 0.91,
        tags: vec!["useful".into()],
        constraints: vec!["bar_aligned".into()],
        provenance_refs: vec!["provider:beats".into()],
    });
    graph.relationships.push(Relationship {
        relation_type: RelationshipType::BelongsToSection,
        from_id: "asset-a".into(),
        to_id: "section-a".into(),
        weight: 1.0,
        notes: Some("primary loop".into()),
    });
    graph.timing.bpm_estimate = Some(126.0);
    graph.timing.bpm_confidence = 0.81;
    graph.analysis_summary = AnalysisSummary {
        overall_confidence: 0.87,
        timing_quality: QualityClass::High,
        section_quality: QualityClass::Medium,
        loop_candidate_count: 1,
        hook_candidate_count: 0,
        break_rebuild_potential: QualityClass::High,
        warnings: vec![AnalysisWarning {
            code: "low_hook_density".into(),
            message: "few hook fragments".into(),
        }],
    };
    graph
}

fn scene_regression_graph(section_labels: &[String]) -> SourceGraph {
    let mut graph = sample_graph();
    graph.sections.clear();

    for (index, label) in section_labels.iter().enumerate() {
        let bar_start = (index as u32 * 8) + 1;
        graph.sections.push(Section {
            section_id: SectionId::from(format!("section-{index}")),
            label_hint: scene_label_hint(label),
            start_seconds: index as f32 * 16.0,
            end_seconds: (index + 1) as f32 * 16.0,
            bar_start,
            bar_end: bar_start + 7,
            energy_class: scene_energy_for_label(label),
            confidence: 0.9,
            tags: vec![label.clone()],
        });
    }

    graph
}

fn seed_scene_fixture_state(state: &mut JamAppState, fixture: &SceneRegressionFixture) {
    if let Some(current_scene) = fixture.initial_current_scene.as_deref() {
        state.session.runtime_state.transport.current_scene = Some(SceneId::from(current_scene));
    }
    if let Some(active_scene) = fixture.initial_active_scene.as_deref() {
        state.session.runtime_state.scene_state.active_scene = Some(SceneId::from(active_scene));
    }
    if let Some(restore_scene) = fixture.initial_restore_scene.as_deref() {
        state.session.runtime_state.scene_state.restore_scene = Some(SceneId::from(restore_scene));
    }
    if let Some(reinforcement_mode) = fixture.tr909_reinforcement_mode {
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .takeover_enabled = false;
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .takeover_profile = None;
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .reinforcement_mode = Some(reinforcement_mode);
    }
    if let Some(pattern_ref) = fixture.tr909_pattern_ref.as_deref() {
        state.session.runtime_state.lane_state.tr909.pattern_ref = Some(pattern_ref.into());
    }
    state.refresh_view();
}

fn sample_session(graph: &SourceGraph) -> SessionFile {
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-12T18:00:00Z");
    session.source_refs.push(SourceRef {
        source_id: SourceId::from("src-1"),
        path_hint: "input.wav".into(),
        content_hash: "hash-1".into(),
        duration_seconds: 120.0,
        decode_profile: "normalized_stereo".into(),
    });
    session.source_graph_refs.push(SourceGraphRef {
        source_id: SourceId::from("src-1"),
        graph_version: SourceGraphVersion::V1,
        graph_hash: crate::jam_app::persistence::source_graph_hash(graph).expect("hash sample graph"),
        storage_mode: GraphStorageMode::Embedded,
        embedded_graph: Some(graph.clone()),
        external_path: None,
        provenance: graph.provenance.clone(),
    });
    session.runtime_state.transport.is_playing = true;
    session.runtime_state.transport.position_beats = 32.0;
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-1"));
    session.runtime_state.macro_state.scene_aggression = 0.75;
    session.runtime_state.macro_state.tr909_slam = 0.55;
    session.runtime_state.lane_state.mc202.role = Some("follower".into());
    session.runtime_state.lane_state.w30.preview_mode = Some(W30PreviewModeState::LiveRecall);
    session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
    session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
    session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
    session.runtime_state.mixer_state.drum_level = 0.72;
    session.runtime_state.mixer_state.music_level = 0.64;
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-1"));
    session.runtime_state.scene_state.scenes = vec![SceneId::from("scene-1")];
    session.runtime_state.lock_state.locked_object_ids = vec!["ghost.main".into()];
    session.action_log.actions.push(Action {
        id: ActionId(1),
        actor: ActorType::User,
        command: ActionCommand::CaptureNow,
        params: ActionParams::Capture { bars: Some(2) },
        target: ActionTarget {
            scope: Some(TargetScope::LaneW30),
            bank_id: Some(BankId::from("bank-a")),
            pad_id: Some(PadId::from("pad-01")),
            ..Default::default()
        },
        requested_at: 100,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(200),
        result: Some(ActionResult {
            accepted: true,
            summary: "captured".into(),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: Some("capture current break".into()),
    });
    session.snapshots.push(Snapshot {
        snapshot_id: SnapshotId::from("snap-1"),
        created_at: "2026-04-12T18:05:00Z".into(),
        label: "first jam".into(),
        action_cursor: 1,
        payload: None,
    });
    session.captures.push(CaptureRef {
        capture_id: CaptureId::from("cap-01"),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["asset-a".into()],
        source_window: None,
        lineage_capture_refs: Vec::new(),
        resample_generation_depth: 0,
        created_from_action: Some(ActionId(1)),
        storage_path: "captures/cap-01.wav".into(),
        assigned_target: None,
        is_pinned: false,
        notes: Some("keeper".into()),
    });
    session.ghost_state = GhostState {
        mode: GhostMode::Assist,
        budgets: GhostBudgetState {
            max_actions_per_phrase: 2,
            max_destructive_actions_per_scene: 1,
            max_pending_actions: 2,
        },
        suggestion_history: vec![GhostSuggestionRecord {
            proposal_id: "gp-1".into(),
            summary: "capture next bar".into(),
            accepted: false,
            rejected: false,
        }],
        lock_awareness_enabled: true,
    };
    session.notes = Some("keeper session".into());
    session
}

fn w30_slice_pool_state_with_source_windows(
    graph: SourceGraph,
    source_audio_cache: SourceAudioCache,
) -> JamAppState {
    let mut session = sample_session(&graph);
    session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-a"),
        pad_id: PadId::from("pad-01"),
    });
    session.captures[0].source_window = Some(CaptureSourceWindow {
        source_id: graph.source.source_id.clone(),
        start_seconds: 0.0,
        end_seconds: 0.5,
        start_frame: 0,
        end_frame: 24_000,
    });
    session.captures.push(CaptureRef {
        capture_id: CaptureId::from("cap-02"),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["asset-c".into()],
        source_window: Some(CaptureSourceWindow {
            source_id: graph.source.source_id.clone(),
            start_seconds: 0.05,
            end_seconds: 0.55,
            start_frame: 2_400,
            end_frame: 26_400,
        }),
        lineage_capture_refs: vec![CaptureId::from("cap-01")],
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: "captures/cap-02.wav".into(),
        assigned_target: Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        }),
        is_pinned: false,
        notes: Some("cyclic slice".into()),
    });
    session.captures.push(CaptureRef {
        capture_id: CaptureId::from("cap-03"),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["asset-b".into()],
        source_window: Some(CaptureSourceWindow {
            source_id: graph.source.source_id.clone(),
            start_seconds: 0.123,
            end_seconds: 0.623,
            start_frame: 5_904,
            end_frame: 29_904,
        }),
        lineage_capture_refs: vec![CaptureId::from("cap-01")],
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: "captures/cap-03.wav".into(),
        assigned_target: Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        }),
        is_pinned: false,
        notes: Some("feral hook slice".into()),
    });
    session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
    session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
    session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));

    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    state.source_audio_cache = Some(source_audio_cache);
    state.refresh_view();
    state
}

fn sidecar_script_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../python/sidecar/json_stdio_sidecar.py")
        .canonicalize()
        .expect("resolve sidecar script path")
}

fn write_pcm16_wave(
    path: impl AsRef<Path>,
    sample_rate: u32,
    channel_count: u16,
    duration_seconds: f32,
) {
    let path = path.as_ref();
    let frame_count = (sample_rate as f32 * duration_seconds) as u32;
    let bits_per_sample = 16_u16;
    let bytes_per_sample = (bits_per_sample / 8) as u32;
    let byte_rate = sample_rate * channel_count as u32 * bytes_per_sample;
    let block_align = channel_count * (bits_per_sample / 8);
    let data_len = frame_count * channel_count as u32 * bytes_per_sample;

    let mut bytes = Vec::with_capacity((44 + data_len) as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&channel_count.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());

    for frame_index in 0..frame_count {
        let phase = (frame_index as f32 / sample_rate as f32) * 220.0 * 2.0 * PI;
        let sample = (phase.sin() * i16::MAX as f32 * 0.25) as i16;
        for _ in 0..channel_count {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
    }

    fs::write(path, bytes).expect("write PCM wave fixture");
}

fn write_pcm24_wave(path: impl AsRef<Path>, sample_rate: u32, channel_count: u16) {
    let path = path.as_ref();
    let samples = [-8_388_608_i32, 0, 8_388_607, 4_194_304];
    assert_eq!(samples.len() % usize::from(channel_count), 0);
    let bits_per_sample = 24_u16;
    let bytes_per_sample = u32::from(bits_per_sample / 8);
    let byte_rate = sample_rate * u32::from(channel_count) * bytes_per_sample;
    let block_align = channel_count * (bits_per_sample / 8);
    let data_len = samples.len() as u32 * bytes_per_sample;

    let mut bytes = Vec::with_capacity((44 + data_len) as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&channel_count.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());

    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes()[..3]);
    }

    fs::write(path, bytes).expect("write PCM24 wave fixture");
}

#[test]
fn builds_jam_app_state_from_parts() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert!(state.jam_view.transport.is_playing);
    assert_eq!(state.jam_view.scene.scene_count, 1);
    assert_eq!(state.jam_view.lanes.mc202_role.as_deref(), Some("follower"));
    assert_eq!(state.runtime_view.audio_status, "unknown");
    assert_eq!(state.runtime_view.sidecar_status, "unknown");
}

#[test]
fn derives_scene_candidates_from_source_sections_when_session_is_empty() {
    let mut graph = sample_graph();
    graph.sections.push(Section {
        section_id: SectionId::from("section-b"),
        label_hint: SectionLabelHint::Break,
        start_seconds: 16.0,
        end_seconds: 24.0,
        bar_start: 9,
        bar_end: 12,
        energy_class: EnergyClass::Medium,
        confidence: 0.84,
        tags: vec!["contrast".into()],
    });

    let mut session = sample_session(&graph);
    session.runtime_state.transport.current_scene = None;
    session.runtime_state.scene_state.active_scene = None;
    session.runtime_state.scene_state.scenes.clear();

    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state
            .session
            .runtime_state
            .scene_state
            .scenes
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["scene-01-drop".to_string(), "scene-02-break".to_string()]
    );
    assert_eq!(
        state.session.runtime_state.scene_state.active_scene,
        Some(SceneId::from("scene-01-drop"))
    );
    assert_eq!(
        state.session.runtime_state.transport.current_scene,
        Some(SceneId::from("scene-01-drop"))
    );
    assert_eq!(state.jam_view.scene.scene_count, 2);
    assert_eq!(
        state.jam_view.scene.active_scene.as_deref(),
        Some("scene-01-drop")
    );
}
