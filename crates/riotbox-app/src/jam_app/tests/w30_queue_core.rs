#[test]
fn queue_w30_trigger_pad_targets_focused_lane_capture_on_next_beat() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-a"),
        pad_id: PadId::from("pad-01"),
    });
    state.session.captures.push(CaptureRef {
        capture_id: CaptureId::from("cap-02"),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["asset-b".into()],
        source_window: None,
        lineage_capture_refs: Vec::new(),
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: "captures/cap-02.wav".into(),
        assigned_target: Some(CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-04".into(),
        }),
        is_pinned: false,
        notes: Some("secondary".into()),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-04"));
    state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_trigger_pad(625),
        Some(QueueControlResult::Enqueued)
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::W30TriggerPad);
    assert_eq!(pending[0].quantization, Quantization::NextBeat);
    assert_eq!(
        pending[0].target.bank_id.as_ref().map(ToString::to_string),
        Some("bank-b".into())
    );
    assert_eq!(
        pending[0].target.pad_id.as_ref().map(ToString::to_string),
        Some("pad-04".into())
    );
    assert_eq!(
        pending[0].explanation.as_deref(),
        Some("trigger W-30 pad bank-b/pad-04 from cap-02 on next beat")
    );
    assert_eq!(
        state.jam_view.lanes.w30_pending_trigger_target.as_deref(),
        Some("bank-b/pad-04")
    );
    assert_eq!(state.jam_view.lanes.w30_pending_recall_target, None);
    assert_eq!(state.jam_view.lanes.w30_pending_audition_target, None);
}

#[test]
fn queue_w30_step_focus_targets_next_promoted_pad_on_next_beat() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-a"),
        pad_id: PadId::from("pad-01"),
    });
    state.session.captures.push(CaptureRef {
        capture_id: CaptureId::from("cap-02"),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["asset-b".into()],
        source_window: None,
        lineage_capture_refs: Vec::new(),
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: "captures/cap-02.wav".into(),
        assigned_target: Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-b"),
            pad_id: PadId::from("pad-04"),
        }),
        is_pinned: false,
        notes: Some("secondary".into()),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_step_focus(622),
        Some(QueueControlResult::Enqueued)
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::W30StepFocus);
    assert_eq!(pending[0].quantization, Quantization::NextBeat);
    assert_eq!(
        pending[0].target.bank_id.as_ref().map(ToString::to_string),
        Some("bank-b".into())
    );
    assert_eq!(
        pending[0].target.pad_id.as_ref().map(ToString::to_string),
        Some("pad-04".into())
    );
    assert_eq!(
        pending[0].explanation.as_deref(),
        Some("step W-30 focus to bank-b/pad-04 on next beat")
    );
    assert_eq!(
        state
            .jam_view
            .lanes
            .w30_pending_focus_step_target
            .as_deref(),
        Some("bank-b/pad-04")
    );
    assert_eq!(state.jam_view.lanes.w30_pending_recall_target, None);
    assert_eq!(state.jam_view.lanes.w30_pending_audition_target, None);
}

#[test]
fn queue_w30_internal_resample_targets_focused_lane_capture_on_next_phrase() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-a"),
        pad_id: PadId::from("pad-01"),
    });
    state.session.captures.push(CaptureRef {
        capture_id: CaptureId::from("cap-02"),
        capture_type: CaptureType::Resample,
        source_origin_refs: vec!["asset-b".into()],
        source_window: None,
        lineage_capture_refs: vec![CaptureId::from("cap-01")],
        resample_generation_depth: 1,
        created_from_action: None,
        storage_path: "captures/cap-02.wav".into(),
        assigned_target: Some(CaptureTarget::W30Pad {
            bank_id: "bank-b".into(),
            pad_id: "pad-03".into(),
        }),
        is_pinned: false,
        notes: Some("resampled".into()),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
    state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_internal_resample(627),
        Some(QueueControlResult::Enqueued)
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::PromoteResample);
    assert_eq!(pending[0].quantization, Quantization::NextPhrase);
    assert_eq!(pending[0].target.scope, Some(TargetScope::LaneW30));
    assert!(matches!(
        &pending[0].params,
        ActionParams::Promotion {
            capture_id: Some(capture_id),
            ..
        } if capture_id == &CaptureId::from("cap-02")
    ));
    assert_eq!(
        pending[0].explanation.as_deref(),
        Some("resample cap-02 through W-30 on next phrase")
    );
}

#[test]
fn queue_w30_swap_bank_targets_next_bank_on_next_bar() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-a"),
        pad_id: PadId::from("pad-01"),
    });
    state.session.captures.push(CaptureRef {
        capture_id: CaptureId::from("cap-02"),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["asset-b".into()],
        source_window: None,
        lineage_capture_refs: Vec::new(),
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: "captures/cap-02.wav".into(),
        assigned_target: Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-b"),
            pad_id: PadId::from("pad-01"),
        }),
        is_pinned: false,
        notes: Some("bank b".into()),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_swap_bank(628),
        Some(QueueControlResult::Enqueued)
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::W30SwapBank);
    assert_eq!(pending[0].quantization, Quantization::NextBar);
    assert_eq!(
        pending[0].target.bank_id.as_ref().map(ToString::to_string),
        Some("bank-b".into())
    );
    assert_eq!(
        pending[0].target.pad_id.as_ref().map(ToString::to_string),
        Some("pad-01".into())
    );
    assert!(matches!(
        &pending[0].params,
        ActionParams::Mutation {
            target_id: Some(target_id),
            ..
        } if target_id == "cap-02"
    ));
    assert_eq!(
        pending[0].explanation.as_deref(),
        Some("swap W-30 bank to bank-b/pad-01 with cap-02 on next bar")
    );
    assert_eq!(
        state.jam_view.lanes.w30_pending_bank_swap_target.as_deref(),
        Some("bank-b/pad-01")
    );
}

#[test]
fn queue_w30_browse_slice_pool_targets_next_capture_on_current_pad() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-a"),
        pad_id: PadId::from("pad-01"),
    });
    state.session.captures.push(CaptureRef {
        capture_id: CaptureId::from("cap-02"),
        capture_type: CaptureType::Pad,
        source_origin_refs: vec!["asset-b".into()],
        source_window: None,
        lineage_capture_refs: vec![CaptureId::from("cap-01")],
        resample_generation_depth: 0,
        created_from_action: None,
        storage_path: "captures/cap-02.wav".into(),
        assigned_target: Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        }),
        is_pinned: false,
        notes: Some("alt slice".into()),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-01"));
    state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_browse_slice_pool(629),
        Some(QueueControlResult::Enqueued)
    );

    let pending = state.queue.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].command, ActionCommand::W30BrowseSlicePool);
    assert_eq!(pending[0].quantization, Quantization::NextBeat);
    assert_eq!(
        pending[0].target.bank_id.as_ref().map(ToString::to_string),
        Some("bank-a".into())
    );
    assert_eq!(
        pending[0].target.pad_id.as_ref().map(ToString::to_string),
        Some("pad-01".into())
    );
    assert!(matches!(
        &pending[0].params,
        ActionParams::Mutation {
            target_id: Some(target_id),
            ..
        } if target_id == "cap-02"
    ));
    assert_eq!(
        pending[0].explanation.as_deref(),
        Some("browse W-30 slice pool to cap-02 on bank-a/pad-01 on next beat")
    );
    assert_eq!(
        state
            .jam_view
            .lanes
            .w30_pending_slice_pool_target
            .as_deref(),
        Some("bank-a/pad-01")
    );
}

#[test]
fn queue_w30_browse_slice_pool_prefers_feral_capture_and_changes_preview_window() {
    let tempdir = tempdir().expect("create source audio tempdir");
    let source_path = tempdir.path().join("source.wav");
    write_pcm16_wave(&source_path, 48_000, 2, 2.0);
    let source_audio_cache =
        SourceAudioCache::load_pcm_wav(&source_path).expect("load source audio cache");

    let mut control_graph = sample_graph();
    control_graph.source.path = source_path.to_string_lossy().into_owned();
    control_graph.source.duration_seconds = 2.0;
    let mut feral_graph = control_graph.clone();
    feral_graph.assets.push(Asset {
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
    feral_graph.candidates.push(Candidate {
        candidate_id: "candidate-feral-capture".into(),
        candidate_type: CandidateType::CaptureCandidate,
        asset_ref: AssetId::from("asset-b"),
        score: 0.92,
        confidence: 0.84,
        tags: vec!["feral".into()],
        constraints: vec!["capture_first".into()],
        provenance_refs: vec!["provider:fixture".into()],
    });
    feral_graph.relationships.push(Relationship {
        relation_type: RelationshipType::SupportsBreakRebuild,
        from_id: "asset-b".into(),
        to_id: "section-a".into(),
        weight: 0.81,
        notes: Some("feral hook supports rebuild".into()),
    });
    feral_graph.analysis_summary.hook_candidate_count = 1;
    feral_graph.analysis_summary.warnings.clear();

    let mut control_state =
        w30_slice_pool_state_with_source_windows(control_graph, source_audio_cache.clone());
    let mut feral_state = w30_slice_pool_state_with_source_windows(feral_graph, source_audio_cache);

    assert_eq!(
        control_state.queue_w30_browse_slice_pool(629),
        Some(QueueControlResult::Enqueued)
    );
    assert_eq!(
        feral_state.queue_w30_browse_slice_pool(630),
        Some(QueueControlResult::Enqueued)
    );

    let control_pending = control_state.queue.pending_actions();
    let feral_pending = feral_state.queue.pending_actions();
    assert!(matches!(
        &control_pending[0].params,
        ActionParams::Mutation {
            target_id: Some(target_id),
            ..
        } if target_id == "cap-02"
    ));
    assert!(matches!(
        &feral_pending[0].params,
        ActionParams::Mutation {
            target_id: Some(target_id),
            ..
        } if target_id == "cap-03"
    ));
    assert_eq!(
        feral_pending[0].explanation.as_deref(),
        Some("browse W-30 feral slice pool to cap-03 on bank-a/pad-01 on next beat")
    );

    let boundary = CommitBoundaryState {
        kind: CommitBoundary::Beat,
        beat_index: 42,
        bar_index: 11,
        phrase_index: 3,
        scene_id: Some(SceneId::from("scene-1")),
    };
    assert_eq!(
        control_state
            .commit_ready_actions(boundary.clone(), 813)
            .len(),
        1
    );
    assert_eq!(feral_state.commit_ready_actions(boundary, 814).len(), 1);

    let control_preview = control_state
        .runtime
        .w30_preview
        .source_window_preview
        .as_ref()
        .expect("control source-window preview");
    let feral_preview = feral_state
        .runtime
        .w30_preview
        .source_window_preview
        .as_ref()
        .expect("feral source-window preview");

    assert_eq!(control_preview.source_start_frame, 2_400);
    assert_eq!(feral_preview.source_start_frame, 5_904);
    assert!(
        control_preview
            .samples
            .iter()
            .any(|sample| sample.abs() > 0.001)
    );
    assert!(
        feral_preview
            .samples
            .iter()
            .any(|sample| sample.abs() > 0.001)
    );
    assert!(
        recipe_signal_delta_rms(&control_preview.samples, &feral_preview.samples) > 0.01,
        "feral W-30 preview should differ from cyclic slice-pool preview"
    );
}

