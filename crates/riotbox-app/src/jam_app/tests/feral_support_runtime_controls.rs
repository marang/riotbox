#[test]
fn feral_break_support_bias_changes_mc202_hook_response_output() {
    let mut control_graph = sample_graph();
    control_graph.sections.clear();
    control_graph.sections.push(Section {
        section_id: SectionId::from("section-steady"),
        label_hint: SectionLabelHint::Verse,
        start_seconds: 0.0,
        end_seconds: 16.0,
        bar_start: 1,
        bar_end: 8,
        energy_class: EnergyClass::Medium,
        confidence: 0.88,
        tags: vec!["steady".into()],
    });
    control_graph.assets.clear();
    control_graph.candidates.clear();
    control_graph.relationships.clear();
    control_graph.analysis_summary.break_rebuild_potential = QualityClass::High;
    control_graph.analysis_summary.hook_candidate_count = 0;

    let mut feral_graph = control_graph.clone();
    feral_graph.assets.push(Asset {
        asset_id: AssetId::from("asset-feral-hook"),
        asset_type: AssetType::HookFragment,
        start_seconds: 1.0,
        end_seconds: 3.0,
        start_bar: 1,
        end_bar: 2,
        confidence: 0.9,
        tags: vec!["feral".into()],
        source_refs: vec!["src-1".into()],
    });
    feral_graph.candidates.push(Candidate {
        candidate_id: "candidate-feral-capture".into(),
        candidate_type: CandidateType::CaptureCandidate,
        asset_ref: AssetId::from("asset-feral-hook"),
        score: 0.9,
        confidence: 0.85,
        tags: vec!["feral".into()],
        constraints: vec!["capture_first".into()],
        provenance_refs: vec!["provider:fixture".into()],
    });
    feral_graph.relationships.push(Relationship {
        relation_type: RelationshipType::SupportsBreakRebuild,
        from_id: "asset-feral-hook".into(),
        to_id: "section-steady".into(),
        weight: 0.85,
        notes: Some("feral hook supports rebuild".into()),
    });
    feral_graph.analysis_summary.hook_candidate_count = 1;

    fn follower_session(graph: &SourceGraph) -> SessionFile {
        let mut session = sample_session(graph);
        session.runtime_state.lane_state.mc202.role = Some("follower".into());
        session.runtime_state.lane_state.mc202.phrase_ref = Some("follower-feral".into());
        session.runtime_state.transport.position_beats = 4.0;
        session
    }

    let control_state = JamAppState::from_parts(
        follower_session(&control_graph),
        Some(control_graph),
        ActionQueue::new(),
    );
    let feral_state = JamAppState::from_parts(
        follower_session(&feral_graph),
        Some(feral_graph),
        ActionQueue::new(),
    );

    assert_eq!(
        control_state.runtime.mc202_render.hook_response,
        Mc202HookResponse::Direct
    );
    assert_eq!(
        control_state.runtime.mc202_render.note_budget,
        riotbox_audio::mc202::Mc202NoteBudget::Balanced
    );
    assert_eq!(
        feral_state.runtime.mc202_render.hook_response,
        Mc202HookResponse::AnswerSpace
    );
    assert_eq!(
        feral_state.runtime.mc202_render.note_budget,
        riotbox_audio::mc202::Mc202NoteBudget::Sparse
    );
    assert!(
        feral_state
            .runtime_view
            .mc202_render_mix_summary
            .contains("hook answer_space")
    );

    let frame_count = 44_100;
    let control_buffer =
        render_mc202_offline(&control_state.runtime.mc202_render, 44_100, 2, frame_count);
    let feral_buffer =
        render_mc202_offline(&feral_state.runtime.mc202_render, 44_100, 2, frame_count);
    let control_metrics = signal_metrics(&control_buffer);
    let feral_metrics = signal_metrics(&feral_buffer);

    assert!(
        control_metrics.rms > 0.001,
        "control MC-202 follower rendered silence: {}",
        control_metrics.rms
    );
    assert!(
        feral_metrics.rms > 0.001,
        "Feral MC-202 hook-response render collapsed to silence: {}",
        feral_metrics.rms
    );
    assert_recipe_buffers_differ(
        "Feral MC-202 hook response",
        &control_buffer,
        &feral_buffer,
        0.002,
    );
}

#[test]
fn feral_break_support_evidence_drives_current_lane_consumers_consistently() {
    let tempdir = tempdir().expect("create source audio tempdir");
    let source_path = tempdir.path().join("source.wav");
    write_pcm16_wave(&source_path, 48_000, 2, 2.0);
    let source_audio_cache =
        SourceAudioCache::load_pcm_wav(&source_path).expect("load source audio cache");

    let mut near_miss_graph = sample_graph();
    near_miss_graph.source.path = source_path.to_string_lossy().into_owned();
    near_miss_graph.source.duration_seconds = 2.0;
    near_miss_graph.sections.clear();
    near_miss_graph.sections.push(Section {
        section_id: SectionId::from("section-steady"),
        label_hint: SectionLabelHint::Verse,
        start_seconds: 0.0,
        end_seconds: 16.0,
        bar_start: 1,
        bar_end: 8,
        energy_class: EnergyClass::Medium,
        confidence: 0.88,
        tags: vec!["steady".into()],
    });
    near_miss_graph.assets.clear();
    near_miss_graph.candidates.clear();
    near_miss_graph.relationships.clear();
    near_miss_graph.analysis_summary.break_rebuild_potential = QualityClass::High;
    near_miss_graph.analysis_summary.hook_candidate_count = 1;
    near_miss_graph.analysis_summary.warnings.clear();
    near_miss_graph.assets.push(Asset {
        asset_id: AssetId::from("asset-b"),
        asset_type: AssetType::HookFragment,
        start_seconds: 0.123,
        end_seconds: 0.623,
        start_bar: 1,
        end_bar: 1,
        confidence: 0.86,
        tags: vec!["feral".into()],
        source_refs: vec!["src-1".into()],
    });
    near_miss_graph.candidates.push(Candidate {
        candidate_id: "candidate-feral-capture".into(),
        candidate_type: CandidateType::CaptureCandidate,
        asset_ref: AssetId::from("asset-b"),
        score: 0.92,
        confidence: 0.84,
        tags: vec!["feral".into()],
        constraints: vec!["capture_first".into()],
        provenance_refs: vec!["provider:fixture".into()],
    });

    let mut feral_graph = near_miss_graph.clone();
    feral_graph.relationships.push(Relationship {
        relation_type: RelationshipType::SupportsBreakRebuild,
        from_id: "asset-b".into(),
        to_id: "section-steady".into(),
        weight: 0.85,
        notes: Some("feral hook supports rebuild".into()),
    });

    assert!(!near_miss_graph.has_feral_break_support_evidence());
    assert!(feral_graph.has_feral_break_support_evidence());

    let mut near_miss_w30_state = w30_slice_pool_state_with_source_windows(
        near_miss_graph.clone(),
        source_audio_cache.clone(),
    );
    let mut feral_w30_state =
        w30_slice_pool_state_with_source_windows(feral_graph.clone(), source_audio_cache);
    assert_eq!(
        near_miss_w30_state.queue_w30_browse_slice_pool(710),
        Some(QueueControlResult::Enqueued)
    );
    assert_eq!(
        feral_w30_state.queue_w30_browse_slice_pool(711),
        Some(QueueControlResult::Enqueued)
    );
    let near_miss_w30_pending = near_miss_w30_state.queue.pending_actions();
    let feral_w30_pending = feral_w30_state.queue.pending_actions();
    assert!(matches!(
        &near_miss_w30_pending[0].params,
        ActionParams::Mutation {
            target_id: Some(target_id),
            ..
        } if target_id == "cap-02"
    ));
    assert!(matches!(
        &feral_w30_pending[0].params,
        ActionParams::Mutation {
            target_id: Some(target_id),
            ..
        } if target_id == "cap-03"
    ));
    assert_eq!(
        near_miss_w30_pending[0].explanation.as_deref(),
        Some("browse W-30 slice pool to cap-02 on bank-a/pad-01 on next beat")
    );
    assert_eq!(
        feral_w30_pending[0].explanation.as_deref(),
        Some("browse W-30 feral slice pool to cap-03 on bank-a/pad-01 on next beat")
    );

    fn source_support_session(graph: &SourceGraph) -> SessionFile {
        let mut session = sample_session(graph);
        session.runtime_state.lane_state.tr909.reinforcement_mode =
            Some(Tr909ReinforcementModeState::SourceSupport);
        session.runtime_state.lane_state.tr909.pattern_ref = Some("support-feral-break".into());
        session.runtime_state.transport.position_beats = 4.0;
        session
    }

    let near_miss_tr909_state = JamAppState::from_parts(
        source_support_session(&near_miss_graph),
        Some(near_miss_graph.clone()),
        ActionQueue::new(),
    );
    let feral_tr909_state = JamAppState::from_parts(
        source_support_session(&feral_graph),
        Some(feral_graph.clone()),
        ActionQueue::new(),
    );
    assert_eq!(
        near_miss_tr909_state
            .runtime
            .tr909_render
            .source_support_profile,
        Some(Tr909SourceSupportProfile::SteadyPulse)
    );
    assert_eq!(
        feral_tr909_state
            .runtime
            .tr909_render
            .source_support_profile,
        Some(Tr909SourceSupportProfile::BreakLift)
    );
    assert_eq!(
        feral_tr909_state.runtime_view.tr909_render_support_reason,
        "feral break lift"
    );

    fn follower_session(graph: &SourceGraph) -> SessionFile {
        let mut session = sample_session(graph);
        session.runtime_state.lane_state.mc202.role = Some("follower".into());
        session.runtime_state.lane_state.mc202.phrase_ref = Some("follower-feral".into());
        session.runtime_state.transport.position_beats = 4.0;
        session
    }

    let near_miss_mc202_state = JamAppState::from_parts(
        follower_session(&near_miss_graph),
        Some(near_miss_graph),
        ActionQueue::new(),
    );
    let feral_mc202_state = JamAppState::from_parts(
        follower_session(&feral_graph),
        Some(feral_graph),
        ActionQueue::new(),
    );
    assert_eq!(
        near_miss_mc202_state.runtime.mc202_render.hook_response,
        Mc202HookResponse::Direct
    );
    assert_eq!(
        feral_mc202_state.runtime.mc202_render.hook_response,
        Mc202HookResponse::AnswerSpace
    );
    assert_eq!(
        feral_mc202_state.runtime.mc202_render.note_budget,
        riotbox_audio::mc202::Mc202NoteBudget::Sparse
    );
}

#[test]
fn runtime_view_surfaces_w30_resample_tap_diagnostics() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-b"),
        pad_id: PadId::from("pad-03"),
    });
    session.captures[0].is_pinned = true;
    session.captures[0].lineage_capture_refs =
        vec![CaptureId::from("cap-seed"), CaptureId::from("cap-bar-02")];
    session.captures[0].resample_generation_depth = 2;
    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.runtime.w30_resample_tap.mode,
        W30ResampleTapMode::CaptureLineageReady
    );
    assert_eq!(
        state.runtime.w30_resample_tap.routing,
        W30ResampleTapRouting::InternalCaptureTap
    );
    assert_eq!(
        state.runtime.w30_resample_tap.source_profile,
        Some(W30ResampleTapSourceProfile::PinnedCapture)
    );
    assert_eq!(
        state.runtime.w30_resample_tap.source_capture_id.as_deref(),
        Some("cap-01")
    );
    assert_eq!(state.runtime.w30_resample_tap.lineage_capture_count, 2);
    assert_eq!(state.runtime.w30_resample_tap.generation_depth, 2);
    assert_eq!(
        state.runtime_view.w30_resample_tap_mode,
        "capture_lineage_ready"
    );
    assert_eq!(
        state.runtime_view.w30_resample_tap_routing,
        "internal_capture_tap"
    );
    assert_eq!(
        state.runtime_view.w30_resample_tap_profile,
        "pinned_capture"
    );
    assert_eq!(
        state.runtime_view.w30_resample_tap_source_summary,
        "cap-01 | gen 2 | lineage 2"
    );
    assert_eq!(
        state.runtime_view.w30_resample_tap_mix_summary,
        "music bus 0.64 | grit 0.40"
    );
}

#[test]
fn adjusting_drum_bus_level_updates_session_and_runtime_view() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.tr909.takeover_enabled = true;
    session.runtime_state.lane_state.tr909.takeover_profile =
        Some(Tr909TakeoverProfileState::ControlledPhraseTakeover);
    session.runtime_state.lane_state.tr909.pattern_ref = Some("scene-1-main".into());
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let raised = state.adjust_drum_bus_level(0.18);
    assert!((raised - 0.90).abs() < f32::EPSILON);
    assert!((state.session.runtime_state.mixer_state.drum_level - 0.90).abs() < f32::EPSILON);
    assert_eq!(
        state.runtime_view.tr909_render_mix_summary,
        "drum bus 0.90 | slam 0.55"
    );

    let lowered = state.adjust_drum_bus_level(-1.5);
    assert_eq!(lowered, 0.0);
    assert_eq!(state.session.runtime_state.mixer_state.drum_level, 0.0);
    assert_eq!(
        state.runtime_view.tr909_render_mix_summary,
        "drum bus 0.00 | slam 0.55"
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
fn adjusting_mc202_touch_updates_session_and_runtime_view() {
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.mc202.role = Some("follower".into());
    session.runtime_state.lane_state.mc202.phrase_ref = Some("follower-scene-1".into());
    session.runtime_state.macro_state.mc202_touch = 0.40;
    session.runtime_state.mixer_state.music_level = 0.64;
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    let raised = state.adjust_mc202_touch(0.24);
    assert!((raised - 0.64).abs() < f32::EPSILON);
    assert!((state.session.runtime_state.macro_state.mc202_touch - 0.64).abs() < f32::EPSILON);
    assert_eq!(state.runtime.mc202_render.mode, Mc202RenderMode::Follower);
    assert_eq!(
        state.runtime.mc202_render.phrase_shape,
        Mc202PhraseShape::FollowerDrive
    );
    assert_eq!(
        state.runtime.mc202_render.routing,
        Mc202RenderRouting::MusicBusBass
    );
    assert!((state.runtime.mc202_render.touch - 0.64).abs() < f32::EPSILON);
    assert_eq!(
        state.runtime_view.mc202_render_mix_summary,
        "music bus 0.64 | touch 0.64 | budget balanced | contour drop | hook direct"
    );

    let lowered = state.adjust_mc202_touch(-1.5);
    assert_eq!(lowered, 0.0);
    assert_eq!(state.session.runtime_state.macro_state.mc202_touch, 0.0);
    assert_eq!(state.runtime.mc202_render.touch, 0.0);
    assert_eq!(
        state.runtime_view.mc202_render_mix_summary,
        "music bus 0.64 | touch 0.00 | budget balanced | contour drop | hook direct"
    );
}

