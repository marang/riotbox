#[test]
fn committed_tr909_takeover_and_release_update_lane_state() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.queue_tr909_takeover(300),
        QueueControlResult::Enqueued
    );
    let committed_takeover = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 36,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    assert_eq!(committed_takeover.len(), 1);
    assert!(
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .takeover_enabled
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .takeover_profile,
        Some(Tr909TakeoverProfileState::ControlledPhraseTakeover)
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .reinforcement_mode,
        Some(Tr909ReinforcementModeState::Takeover)
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .pattern_ref
            .as_deref(),
        Some("takeover-scene-1")
    );
    assert!(state.jam_view.lanes.tr909_takeover_enabled);
    assert_eq!(
        state.jam_view.lanes.tr909_takeover_profile,
        Some(Tr909TakeoverProfileState::ControlledPhraseTakeover)
    );
    assert_eq!(state.jam_view.lanes.tr909_takeover_pending_profile, None);
    assert_eq!(state.jam_view.lanes.tr909_takeover_pending_target, None);
    assert_eq!(state.runtime.tr909_render.mode, Tr909RenderMode::Takeover);
    assert_eq!(
        state.runtime.tr909_render.routing,
        Tr909RenderRouting::DrumBusTakeover
    );
    assert_eq!(
        state.runtime.tr909_render.takeover_profile,
        Some(Tr909TakeoverRenderProfile::ControlledPhrase)
    );
    assert_eq!(
        state.runtime.tr909_render.phrase_variation,
        Some(Tr909PhraseVariation::PhraseLift)
    );

    state.update_transport_clock(TransportClockState {
        is_playing: true,
        position_beats: 64.0,
        beat_index: 64,
        bar_index: 16,
        phrase_index: 2,
        current_scene: Some(SceneId::from("scene-1")),
    });

    assert_eq!(
        state.queue_tr909_scene_lock(450),
        QueueControlResult::Enqueued
    );
    let committed_scene_lock = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 44,
            bar_index: 11,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        500,
    );

    assert_eq!(committed_scene_lock.len(), 1);
    assert!(
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .takeover_enabled
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .takeover_profile,
        Some(Tr909TakeoverProfileState::SceneLockTakeover)
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .pattern_ref
            .as_deref(),
        Some("lock-scene-1")
    );
    assert_eq!(
        state.jam_view.lanes.tr909_takeover_profile,
        Some(Tr909TakeoverProfileState::SceneLockTakeover)
    );
    assert_eq!(
        state.runtime.tr909_render.takeover_profile,
        Some(Tr909TakeoverRenderProfile::SceneLock)
    );
    assert_eq!(
        state.runtime.tr909_render.phrase_variation,
        Some(Tr909PhraseVariation::PhraseDrive)
    );

    state.update_transport_clock(TransportClockState {
        is_playing: true,
        position_beats: 32.0,
        beat_index: 32,
        bar_index: 8,
        phrase_index: 1,
        current_scene: Some(SceneId::from("scene-1")),
    });

    assert_eq!(state.queue_tr909_release(500), QueueControlResult::Enqueued);
    let committed_release = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 52,
            bar_index: 13,
            phrase_index: 3,
            scene_id: Some(SceneId::from("scene-1")),
        },
        600,
    );

    assert_eq!(committed_release.len(), 1);
    assert!(
        !state
            .session
            .runtime_state
            .lane_state
            .tr909
            .takeover_enabled
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .takeover_profile,
        None
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .reinforcement_mode,
        Some(Tr909ReinforcementModeState::SourceSupport)
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .tr909
            .pattern_ref
            .as_deref(),
        Some("release-scene-1")
    );
    assert!(!state.jam_view.lanes.tr909_takeover_enabled);
    assert_eq!(state.jam_view.lanes.tr909_takeover_profile, None);
    assert_eq!(state.jam_view.lanes.tr909_takeover_pending_target, None);
    assert_eq!(
        state.runtime.tr909_render.mode,
        Tr909RenderMode::SourceSupport
    );
    assert_eq!(
        state.runtime.tr909_render.routing,
        Tr909RenderRouting::DrumBusSupport
    );
    assert_eq!(
        state.runtime.tr909_render.pattern_ref.as_deref(),
        Some("release-scene-1")
    );
    assert_eq!(
        state.runtime.tr909_render.source_support_profile,
        Some(Tr909SourceSupportProfile::DropDrive)
    );
    assert_eq!(
        state.runtime.tr909_render.pattern_adoption,
        Some(Tr909PatternAdoption::MainlineDrive)
    );
    assert_eq!(
        state.runtime.tr909_render.phrase_variation,
        Some(Tr909PhraseVariation::PhraseRelease)
    );
}

#[test]
fn source_support_render_profile_tracks_current_source_section() {
    let mut graph = sample_graph();
    graph.sections.push(Section {
        section_id: SectionId::from("section-b"),
        label_hint: SectionLabelHint::Break,
        start_seconds: 16.0,
        end_seconds: 32.0,
        bar_start: 9,
        bar_end: 16,
        energy_class: EnergyClass::Medium,
        confidence: 0.86,
        tags: vec!["break".into()],
    });

    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.tr909.reinforcement_mode =
        Some(Tr909ReinforcementModeState::SourceSupport);
    session.runtime_state.lane_state.tr909.pattern_ref = Some("support-scene-1".into());
    session.runtime_state.transport.position_beats = 16.0;

    let state = JamAppState::from_parts(session.clone(), Some(graph.clone()), ActionQueue::new());

    assert_eq!(
        state.runtime.tr909_render.mode,
        Tr909RenderMode::SourceSupport
    );
    assert_eq!(
        state.runtime.tr909_render.source_support_profile,
        Some(Tr909SourceSupportProfile::DropDrive)
    );
    assert_eq!(
        state.runtime.tr909_render.source_support_context,
        Some(Tr909SourceSupportContext::TransportBar)
    );
    assert_eq!(
        state.runtime_view.tr909_render_support_context,
        "transport_bar"
    );
    assert_eq!(
        state.runtime_view.tr909_render_support_accent,
        "off fallback"
    );
    assert_eq!(state.runtime_view.tr909_render_support_reason, "section");

    session.runtime_state.transport.position_beats = 36.0;
    let state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    assert_eq!(
        state.runtime.tr909_render.mode,
        Tr909RenderMode::SourceSupport
    );
    assert_eq!(
        state.runtime.tr909_render.source_support_profile,
        Some(Tr909SourceSupportProfile::BreakLift)
    );
    assert_eq!(
        state.runtime.tr909_render.source_support_context,
        Some(Tr909SourceSupportContext::TransportBar)
    );
    assert_eq!(
        state.runtime_view.tr909_render_support_context,
        "transport_bar"
    );
    assert_eq!(
        state.runtime_view.tr909_render_support_accent,
        "off fallback"
    );
    assert_eq!(state.runtime_view.tr909_render_support_reason, "section");
    assert_eq!(
        state.runtime.tr909_render.pattern_adoption,
        Some(Tr909PatternAdoption::SupportPulse)
    );
}

#[test]
fn feral_break_support_bias_changes_tr909_source_support_output() {
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

    fn source_support_session(graph: &SourceGraph) -> SessionFile {
        let mut session = sample_session(graph);
        session.runtime_state.lane_state.tr909.reinforcement_mode =
            Some(Tr909ReinforcementModeState::SourceSupport);
        session.runtime_state.lane_state.tr909.pattern_ref = Some("support-feral-break".into());
        session.runtime_state.transport.position_beats = 4.0;
        session
    }

    let control_state = JamAppState::from_parts(
        source_support_session(&control_graph),
        Some(control_graph),
        ActionQueue::new(),
    );
    let feral_state = JamAppState::from_parts(
        source_support_session(&feral_graph),
        Some(feral_graph),
        ActionQueue::new(),
    );

    assert_eq!(
        control_state.runtime.tr909_render.source_support_profile,
        Some(Tr909SourceSupportProfile::SteadyPulse)
    );
    assert_eq!(
        control_state.runtime_view.tr909_render_support_reason,
        "section"
    );
    assert_eq!(
        feral_state.runtime.tr909_render.source_support_profile,
        Some(Tr909SourceSupportProfile::BreakLift)
    );
    assert_eq!(
        feral_state.runtime_view.tr909_render_support_reason,
        "feral break lift"
    );
    assert_eq!(
        feral_state.runtime.tr909_render.source_support_context,
        Some(Tr909SourceSupportContext::TransportBar)
    );
    assert_eq!(
        feral_state.runtime.tr909_render.phrase_variation,
        Some(Tr909PhraseVariation::PhraseDrive)
    );

    let frame_count = 44_100;
    let control_buffer =
        render_tr909_offline(&control_state.runtime.tr909_render, 44_100, 2, frame_count);
    let feral_buffer =
        render_tr909_offline(&feral_state.runtime.tr909_render, 44_100, 2, frame_count);
    let control_metrics = signal_metrics(&control_buffer);
    let feral_metrics = signal_metrics(&feral_buffer);

    assert!(
        control_metrics.rms > 0.001,
        "control TR-909 source support rendered silence: {}",
        control_metrics.rms
    );
    assert!(
        feral_metrics.rms > 0.001,
        "Feral TR-909 source support rendered silence: {}",
        feral_metrics.rms
    );
    assert_recipe_buffers_differ(
        "Feral TR-909 source-support lift",
        &control_buffer,
        &feral_buffer,
        0.002,
    );
}

