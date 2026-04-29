struct JamViewFixture {
    graph: SourceGraph,
    session: SessionFile,
    queue: ActionQueue,
}

impl JamViewFixture {
    fn build_view_model(&self) -> JamViewModel {
        JamViewModel::build(&self.session, &self.queue, Some(&self.graph))
    }
}

fn jam_view_fixture() -> JamViewFixture {
    let graph = source_graph_with_feral_capture_evidence();
    let session = session_with_committed_jam_state(&graph);
    let queue = queue_with_lane_pending_actions();

    JamViewFixture {
        graph,
        session,
        queue,
    }
}

fn source_graph_with_feral_capture_evidence() -> SourceGraph {
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
            run_notes: None,
        },
    );

    graph.sections.push(crate::source_graph::Section {
        section_id: "sec-a".into(),
        label_hint: crate::source_graph::SectionLabelHint::Drop,
        start_seconds: 0.0,
        end_seconds: 16.0,
        bar_start: 1,
        bar_end: 8,
        energy_class: crate::source_graph::EnergyClass::High,
        confidence: 0.9,
        tags: vec![],
    });
    graph.assets.push(Asset {
        asset_id: "asset-a".into(),
        asset_type: AssetType::LoopWindow,
        start_seconds: 0.0,
        end_seconds: 4.0,
        start_bar: 1,
        end_bar: 2,
        confidence: 0.8,
        tags: vec![],
        source_refs: vec![],
    });
    graph.assets.push(Asset {
        asset_id: "asset-hook".into(),
        asset_type: AssetType::HookFragment,
        start_seconds: 4.0,
        end_seconds: 5.0,
        start_bar: 2,
        end_bar: 2,
        confidence: 0.82,
        tags: vec!["hook".into()],
        source_refs: vec!["asset-a".into()],
    });
    graph.candidates.push(Candidate {
        candidate_id: "cand-loop".into(),
        candidate_type: CandidateType::LoopCandidate,
        asset_ref: "asset-a".into(),
        score: 0.9,
        confidence: 0.9,
        tags: vec![],
        constraints: vec![],
        provenance_refs: vec![],
    });
    graph.candidates.push(Candidate {
        candidate_id: "cand-hook".into(),
        candidate_type: CandidateType::HookCandidate,
        asset_ref: "asset-a".into(),
        score: 0.7,
        confidence: 0.8,
        tags: vec![],
        constraints: vec![],
        provenance_refs: vec![],
    });
    graph.candidates.push(Candidate {
        candidate_id: "cand-capture".into(),
        candidate_type: CandidateType::CaptureCandidate,
        asset_ref: "asset-hook".into(),
        score: 0.86,
        confidence: 0.77,
        tags: vec!["feral".into()],
        constraints: vec![],
        provenance_refs: vec![],
    });
    graph.relationships.push(Relationship {
        relation_type: RelationshipType::SupportsBreakRebuild,
        from_id: "asset-hook".into(),
        to_id: "asset-a".into(),
        weight: 0.8,
        notes: Some("hook supports loop rebuild".into()),
    });
    graph.relationships.push(Relationship {
        relation_type: RelationshipType::HighQuoteRiskWith,
        from_id: "asset-hook".into(),
        to_id: "src-1".into(),
        weight: 0.6,
        notes: Some("recognizable hook".into()),
    });
    graph.analysis_summary = AnalysisSummary {
        overall_confidence: 0.85,
        timing_quality: QualityClass::High,
        section_quality: QualityClass::Medium,
        loop_candidate_count: 1,
        hook_candidate_count: 1,
        break_rebuild_potential: QualityClass::High,
        warnings: vec![],
    };

    graph
}

fn session_with_committed_jam_state(graph: &SourceGraph) -> SessionFile {
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-12T18:00:00Z");
    session.runtime_state.transport.is_playing = true;
    session.runtime_state.transport.position_beats = 16.0;
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-1"));
    session.runtime_state.scene_state.scenes = vec![SceneId::from("scene-1")];
    session.runtime_state.lane_state.mc202.role = Some("follower".into());
    session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
    session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
    session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    session.runtime_state.lane_state.tr909.takeover_enabled = true;
    session.runtime_state.lane_state.tr909.takeover_profile =
        Some(Tr909TakeoverProfileState::SceneLockTakeover);
    session.runtime_state.lane_state.tr909.slam_enabled = true;
    session.runtime_state.lane_state.tr909.last_fill_bar = Some(8);
    session.runtime_state.lane_state.tr909.reinforcement_mode =
        Some(Tr909ReinforcementModeState::Takeover);
    session.ghost_state.mode = GhostMode::Assist;
    session.ghost_state.suggestion_history = vec![GhostSuggestionRecord {
        proposal_id: "gp-1".into(),
        summary: "capture next bar".into(),
        accepted: false,
        rejected: false,
    }];
    session.action_log = ActionLog {
        actions: vec![],
        commit_records: vec![],
        replay_policy: crate::session::ReplayPolicy::DeterministicPreferred,
    };
    session.source_graph_refs = vec![SourceGraphRef {
        source_id: SourceId::from("src-1"),
        graph_version: crate::source_graph::SourceGraphVersion::V1,
        graph_hash: "graph-1".into(),
        storage_mode: crate::session::GraphStorageMode::Embedded,
        embedded_graph: Some(graph.clone()),
        external_path: None,
        provenance: graph.provenance.clone(),
    }];
    session.runtime_state = RuntimeState {
        transport: session.runtime_state.transport.clone(),
        macro_state: session.runtime_state.macro_state.clone(),
        lane_state: session.runtime_state.lane_state.clone(),
        mixer_state: session.runtime_state.mixer_state.clone(),
        scene_state: session.runtime_state.scene_state.clone(),
        lock_state: session.runtime_state.lock_state.clone(),
        pending_policy: session.runtime_state.pending_policy.clone(),
        undo_state: session.runtime_state.undo_state.clone(),
    };
    session.captures.push(crate::session::CaptureRef {
        capture_id: "cap-01".into(),
        capture_type: crate::session::CaptureType::Pad,
        source_origin_refs: vec!["asset-a".into(), "src-1".into()],
        source_window: None,
        lineage_capture_refs: Vec::new(),
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: "captures/cap-01.wav".into(),
        assigned_target: Some(crate::session::CaptureTarget::W30Pad {
            bank_id: "bank-a".into(),
            pad_id: "pad-01".into(),
        }),
        is_pinned: false,
        notes: Some("keeper capture".into()),
    });

    session
}

fn queue_with_lane_pending_actions() -> ActionQueue {
    let mut queue = ActionQueue::new();
    let mut capture_draft = action_draft(
        ActorType::Ghost,
        ActionCommand::CaptureNow,
        Quantization::NextBar,
        Some(TargetScope::LaneW30),
    );
    capture_draft.undo_policy = UndoPolicy::Undoable;
    capture_draft.explanation = Some("capture current break".into());
    queue.enqueue(capture_draft, 100);

    let mut mc202_role = action_draft(
        ActorType::User,
        ActionCommand::Mc202SetRole,
        Quantization::NextPhrase,
        Some(TargetScope::LaneMc202),
    );
    mc202_role.target.object_id = Some("leader".into());
    queue.enqueue(mc202_role, 101);

    enqueue_w30_action(
        &mut queue,
        ActionCommand::W30LiveRecall,
        Quantization::NextBar,
        "bank-a",
        "pad-02",
        102,
    );
    enqueue_w30_action(
        &mut queue,
        ActionCommand::W30SwapBank,
        Quantization::NextBar,
        "bank-c",
        "pad-01",
        103,
    );
    enqueue_w30_mutation_action(
        &mut queue,
        ActionCommand::W30BrowseSlicePool,
        Quantization::NextBeat,
        "bank-a",
        "pad-04",
        "cap-02",
        104,
    );
    enqueue_w30_action(
        &mut queue,
        ActionCommand::W30ApplyDamageProfile,
        Quantization::NextBar,
        "bank-d",
        "pad-03",
        104,
    );
    enqueue_w30_action(
        &mut queue,
        ActionCommand::W30TriggerPad,
        Quantization::NextBeat,
        "bank-a",
        "pad-03",
        103,
    );
    enqueue_w30_action(
        &mut queue,
        ActionCommand::W30StepFocus,
        Quantization::NextBeat,
        "bank-c",
        "pad-01",
        104,
    );
    queue.enqueue(
        action_draft(
            ActorType::User,
            ActionCommand::Tr909Release,
            Quantization::NextPhrase,
            Some(TargetScope::LaneTr909),
        ),
        104,
    );
    queue.enqueue(
        action_draft(
            ActorType::User,
            ActionCommand::Tr909FillNext,
            Quantization::NextBar,
            Some(TargetScope::LaneTr909),
        ),
        105,
    );

    let mut resample_draft = action_draft(
        ActorType::User,
        ActionCommand::PromoteResample,
        Quantization::NextPhrase,
        Some(TargetScope::LaneW30),
    );
    resample_draft.params = crate::action::ActionParams::Promotion {
        capture_id: Some("cap-01".into()),
        destination: Some("w30:resample".into()),
    };
    queue.enqueue(resample_draft, 106);

    queue
}

fn action_draft(
    actor: ActorType,
    command: ActionCommand,
    quantization: Quantization,
    scope: Option<TargetScope>,
) -> ActionDraft {
    ActionDraft::new(
        actor,
        command,
        quantization,
        ActionTarget {
            scope,
            ..Default::default()
        },
    )
}

fn enqueue_w30_action(
    queue: &mut ActionQueue,
    command: ActionCommand,
    quantization: Quantization,
    bank_id: &str,
    pad_id: &str,
    requested_at: u64,
) {
    let draft = w30_target_draft(command, quantization, bank_id, pad_id);
    queue.enqueue(draft, requested_at);
}

fn enqueue_w30_mutation_action(
    queue: &mut ActionQueue,
    command: ActionCommand,
    quantization: Quantization,
    bank_id: &str,
    pad_id: &str,
    target_id: &str,
    requested_at: u64,
) {
    let mut draft = w30_target_draft(command, quantization, bank_id, pad_id);
    draft.params = ActionParams::Mutation {
        intensity: 1.0,
        target_id: Some(target_id.into()),
    };
    queue.enqueue(draft, requested_at);
}

fn w30_target_draft(
    command: ActionCommand,
    quantization: Quantization,
    bank_id: &str,
    pad_id: &str,
) -> ActionDraft {
    let mut draft = action_draft(
        ActorType::User,
        command,
        quantization,
        Some(TargetScope::LaneW30),
    );
    draft.target.bank_id = Some(bank_id.into());
    draft.target.pad_id = Some(pad_id.into());
    draft
}
