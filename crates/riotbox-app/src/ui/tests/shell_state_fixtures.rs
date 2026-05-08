fn sample_shell_state() -> JamShellState {
    let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-12T00:00:00Z");
    session.runtime_state.transport.position_beats = 32.0;
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-a"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-a"));
    session.runtime_state.macro_state.source_retain = 0.7;
    session.runtime_state.macro_state.chaos = 0.2;
    session.runtime_state.macro_state.mc202_touch = 0.8;
    session.runtime_state.macro_state.w30_grit = 0.5;
    session.runtime_state.macro_state.tr909_slam = 0.9;
    session.runtime_state.mixer_state.drum_level = 0.82;
    session.runtime_state.mixer_state.music_level = 0.64;
    session.runtime_state.lane_state.mc202.role = Some("leader".into());
    session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
    session.runtime_state.lane_state.tr909.takeover_enabled = true;
    session.runtime_state.lane_state.tr909.takeover_profile =
        Some(Tr909TakeoverProfileState::ControlledPhraseTakeover);
    session.runtime_state.lane_state.tr909.pattern_ref = Some("scene-a-main".into());
    session.ghost_state.mode = GhostMode::Assist;
    session.runtime_state.lane_state.tr909.last_fill_bar = Some(6);
    session.runtime_state.lane_state.tr909.reinforcement_mode =
        Some(Tr909ReinforcementModeState::Takeover);
    session.action_log.actions.push(Action {
        id: ActionId(1),
        actor: ActorType::User,
        command: ActionCommand::CaptureNow,
        params: ActionParams::Capture { bars: Some(2) },
        target: ActionTarget {
            scope: Some(TargetScope::LaneW30),
            ..Default::default()
        },
        requested_at: 100,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(120),
        result: Some(ActionResult {
            accepted: true,
            summary: "captured".into(),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: Some("capture opener".into()),
    });
    session.action_log.actions.push(Action {
        id: ActionId(2),
        actor: ActorType::Ghost,
        command: ActionCommand::MutateScene,
        params: ActionParams::Mutation {
            intensity: 0.4,
            target_id: Some("scene-a".into()),
        },
        target: ActionTarget {
            scope: Some(TargetScope::Scene),
            scene_id: Some(SceneId::from("scene-a")),
            ..Default::default()
        },
        requested_at: 125,
        quantization: Quantization::NextPhrase,
        status: ActionStatus::Rejected,
        committed_at: None,
        result: Some(ActionResult {
            accepted: false,
            summary: "scene lock blocked ghost mutation".into(),
        }),
        undo_policy: UndoPolicy::NotUndoable {
            reason: "rejected actions do not create undo state".into(),
        },
        explanation: Some("ghost suggestion rejected".into()),
    });
    session.action_log.actions.push(Action {
        id: ActionId(3),
        actor: ActorType::User,
        command: ActionCommand::UndoLast,
        params: ActionParams::Empty,
        target: ActionTarget {
            scope: Some(TargetScope::Session),
            ..Default::default()
        },
        requested_at: 140,
        quantization: Quantization::Immediate,
        status: ActionStatus::Undone,
        committed_at: Some(140),
        result: Some(ActionResult {
            accepted: true,
            summary: "undid most recent musical action".into(),
        }),
        undo_policy: UndoPolicy::NotUndoable {
            reason: "undo markers are not undoable".into(),
        },
        explanation: Some("user trust correction".into()),
    });

    let mut graph = SourceGraph::new(
        SourceDescriptor {
            source_id: SourceId::from("src-1"),
            path: "fixtures/input.wav".into(),
            content_hash: "hash-1".into(),
            duration_seconds: 12.0,
            sample_rate: 44_100,
            channel_count: 2,
            decode_profile: DecodeProfile::Native,
        },
        GraphProvenance {
            sidecar_version: "0.1.0".into(),
            provider_set: vec!["decoded.wav_baseline".into()],
            generated_at: "2026-04-12T00:00:00Z".into(),
            source_hash: "hash-1".into(),
            analysis_seed: 1,
            run_notes: Some("test".into()),
        },
    );
    graph.timing.bpm_estimate = Some(126.0);
    graph.timing.bpm_confidence = 0.76;
    graph.timing.meter_hint = Some(MeterHint {
        beats_per_bar: 4,
        beat_unit: 4,
    });
    graph.timing.quality = TimingQuality::Low;
    graph.timing.degraded_policy = TimingDegradedPolicy::ManualConfirm;
    graph.timing.primary_hypothesis_id = Some("timing-primary".into());
    graph.timing.hypotheses.push(TimingHypothesis {
        hypothesis_id: "timing-primary".into(),
        kind: TimingHypothesisKind::Primary,
        bpm: 126.0,
        meter: MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
        confidence: 0.76,
        score: 0.68,
        beat_grid: Vec::new(),
        bar_grid: Vec::new(),
        phrase_grid: Vec::new(),
        anchors: Vec::new(),
        drift: Vec::new(),
        groove: Vec::new(),
        quality: TimingQuality::Low,
        warnings: Vec::new(),
        provenance: vec!["fixture.source_timing".into()],
    });
    graph.timing.warnings = vec![
        TimingWarning {
            code: TimingWarningCode::AmbiguousDownbeat,
            message: "downbeat candidates are close".into(),
        },
        TimingWarning {
            code: TimingWarningCode::PhraseUncertain,
            message: "phrase grid needs confirmation".into(),
        },
    ];
    graph.sections.push(Section {
        section_id: SectionId::from("section-a"),
        label_hint: SectionLabelHint::Intro,
        start_seconds: 0.0,
        end_seconds: 4.0,
        bar_start: 1,
        bar_end: 2,
        energy_class: EnergyClass::Medium,
        confidence: 0.71,
        tags: vec!["decoded_wave".into()],
    });
    graph.sections.push(Section {
        section_id: SectionId::from("section-b"),
        label_hint: SectionLabelHint::Drop,
        start_seconds: 4.0,
        end_seconds: 12.0,
        bar_start: 3,
        bar_end: 6,
        energy_class: EnergyClass::High,
        confidence: 0.83,
        tags: vec!["decoded_wave".into()],
    });
    graph.assets.push(Asset {
        asset_id: AssetId::from("asset-a"),
        asset_type: AssetType::LoopWindow,
        start_seconds: 0.0,
        end_seconds: 4.0,
        start_bar: 1,
        end_bar: 2,
        confidence: 0.79,
        tags: vec!["loop".into()],
        source_refs: vec!["src-1".into()],
    });
    graph.assets.push(Asset {
        asset_id: AssetId::from("asset-hook"),
        asset_type: AssetType::HookFragment,
        start_seconds: 4.0,
        end_seconds: 5.0,
        start_bar: 3,
        end_bar: 3,
        confidence: 0.81,
        tags: vec!["hook".into()],
        source_refs: vec!["src-1".into()],
    });
    graph.candidates.push(Candidate {
        candidate_id: "cand-loop".into(),
        candidate_type: CandidateType::LoopCandidate,
        asset_ref: AssetId::from("asset-a"),
        score: 0.84,
        confidence: 0.78,
        tags: vec!["decoded_wave".into()],
        constraints: vec!["bar_aligned".into()],
        provenance_refs: vec!["provider:decoded.wav_baseline".into()],
    });
    graph.candidates.push(Candidate {
        candidate_id: "cand-capture".into(),
        candidate_type: CandidateType::CaptureCandidate,
        asset_ref: AssetId::from("asset-hook"),
        score: 0.79,
        confidence: 0.74,
        tags: vec!["feral".into()],
        constraints: vec!["capture_first".into()],
        provenance_refs: vec!["provider:decoded.wav_baseline".into()],
    });
    graph.relationships.push(Relationship {
        relation_type: RelationshipType::SupportsBreakRebuild,
        from_id: "asset-hook".into(),
        to_id: "asset-a".into(),
        weight: 0.78,
        notes: Some("hook supports loop rebuild".into()),
    });
    graph.relationships.push(Relationship {
        relation_type: RelationshipType::HighQuoteRiskWith,
        from_id: "asset-hook".into(),
        to_id: "src-1".into(),
        weight: 0.64,
        notes: Some("recognizable hook".into()),
    });
    graph.analysis_summary = AnalysisSummary {
        overall_confidence: 0.74,
        timing_quality: QualityClass::Medium,
        section_quality: QualityClass::High,
        loop_candidate_count: 1,
        hook_candidate_count: 0,
        break_rebuild_potential: QualityClass::High,
        warnings: vec![AnalysisWarning {
            code: "wav_baseline_only".into(),
            message: "decoded-source baseline used WAV metadata and simple energy heuristics"
                .into(),
        }],
    };

    let mut queue = ActionQueue::new();
    queue.enqueue(
        ActionDraft::new(
            ActorType::Ghost,
            ActionCommand::MutateScene,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::Scene),
                ..Default::default()
            },
        ),
        130,
    );
    queue.enqueue(
        ActionDraft::new(
            ActorType::User,
            ActionCommand::Tr909FillNext,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
        ),
        130,
    );
    let mut promote_draft = ActionDraft::new(
        ActorType::User,
        ActionCommand::PromoteCaptureToPad,
        Quantization::NextBar,
        ActionTarget {
            scope: Some(TargetScope::LaneW30),
            bank_id: Some(BankId::from("bank-a")),
            pad_id: Some(PadId::from("pad-01")),
            ..Default::default()
        },
    );
    promote_draft.params = ActionParams::Promotion {
        capture_id: Some("cap-01".into()),
        destination: Some("w30:bank-a/pad-01".into()),
    };
    promote_draft.explanation = Some("promote keeper capture into the live pad".into());
    queue.enqueue(promote_draft, 131);

    session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
    session.captures.push(riotbox_core::session::CaptureRef {
        capture_id: "cap-01".into(),
        capture_type: riotbox_core::session::CaptureType::Pad,
        source_origin_refs: vec!["asset-a".into(), "src-1".into()],
        source_window: None,
        lineage_capture_refs: Vec::new(),
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: "captures/cap-01.wav".into(),
        assigned_target: None,
        is_pinned: false,
        notes: Some("keeper capture".into()),
    });

    let app = JamAppState::from_parts(session, Some(graph), queue);
    JamShellState::new(app, ShellLaunchMode::Ingest)
}

fn first_run_shell_state() -> JamShellState {
    let sample_shell = sample_shell_state();
    let mut session = sample_shell.app.session.clone();
    session.action_log.actions.clear();
    session.captures.clear();
    session.runtime_state.lane_state.w30.last_capture = None;

    let app = JamAppState::from_parts(
        session,
        sample_shell.app.source_graph.clone(),
        ActionQueue::new(),
    );
    JamShellState::new(app, ShellLaunchMode::Ingest)
}

fn first_result_shell_state() -> JamShellState {
    let mut shell = first_run_shell_state();
    shell.app.session.action_log.actions.push(Action {
        id: ActionId(1),
        actor: ActorType::User,
        command: ActionCommand::Tr909FillNext,
        params: ActionParams::Empty,
        target: ActionTarget {
            scope: Some(TargetScope::LaneTr909),
            ..Default::default()
        },
        requested_at: 200,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(220),
        result: Some(ActionResult {
            accepted: true,
            summary: "committed fill on next bar".into(),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: Some("first committed fill".into()),
    });

    shell.app.refresh_view();
    shell
}

fn sample_shell_without_pending_queue() -> JamShellState {
    let sample_shell = sample_shell_state();
    JamShellState::new(
        JamAppState::from_parts(
            sample_shell.app.session.clone(),
            sample_shell.app.source_graph.clone(),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Ingest,
    )
}

fn scene_post_commit_shell_state(
    command: ActionCommand,
    active_scene: &str,
    restore_scene: &str,
) -> JamShellState {
    let sample_shell = sample_shell_state();
    let mut session = sample_shell.app.session.clone();
    session.action_log.actions.clear();
    session.runtime_state.transport.current_scene = Some(SceneId::from(active_scene));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from(active_scene));
    session.runtime_state.scene_state.restore_scene = Some(SceneId::from(restore_scene));
    session.runtime_state.lane_state.tr909.takeover_enabled = false;
    session.runtime_state.lane_state.tr909.takeover_profile = None;
    session.runtime_state.lane_state.tr909.reinforcement_mode =
        Some(Tr909ReinforcementModeState::SourceSupport);
    session.runtime_state.lane_state.tr909.pattern_ref = Some("scene-support".into());
    session.action_log.actions.push(Action {
        id: ActionId(1),
        actor: ActorType::User,
        command,
        params: ActionParams::Scene {
            scene_id: Some(SceneId::from(active_scene)),
        },
        target: ActionTarget {
            scope: Some(TargetScope::Scene),
            scene_id: Some(SceneId::from(active_scene)),
            ..Default::default()
        },
        requested_at: 300,
        quantization: Quantization::NextBar,
        status: ActionStatus::Committed,
        committed_at: Some(320),
        result: Some(ActionResult {
            accepted: true,
            summary: format!("scene {active_scene} landed"),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: Some(format!("landed {active_scene} scene move")),
    });

    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            session,
            sample_shell.app.source_graph.clone(),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );
    shell.app.set_transport_playing(true);
    shell
}
