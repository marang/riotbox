#[test]
fn committed_w30_promoted_audition_updates_lane_focus_grit_and_log_result() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-b"),
        pad_id: PadId::from("pad-03"),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_promoted_audition(640),
        Some(QueueControlResult::Enqueued)
    );

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 33,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        700,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .w30
            .active_bank
            .as_ref()
            .map(ToString::to_string),
        Some("bank-b".into())
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .w30
            .focused_pad
            .as_ref()
            .map(ToString::to_string),
        Some("pad-03".into())
    );
    assert_eq!(
        state
            .session
            .runtime_state
            .lane_state
            .w30
            .last_capture
            .as_ref()
            .map(ToString::to_string),
        Some("cap-01".into())
    );
    assert_eq!(state.session.runtime_state.macro_state.w30_grit, 0.68);
    assert_eq!(
        state.jam_view.lanes.w30_active_bank.as_deref(),
        Some("bank-b")
    );
    assert_eq!(
        state.jam_view.lanes.w30_focused_pad.as_deref(),
        Some("pad-03")
    );
    assert_eq!(state.jam_view.lanes.w30_pending_recall_target, None);
    assert_eq!(state.jam_view.lanes.w30_pending_audition_target, None);
    assert_eq!(
        state.runtime.w30_preview.mode,
        W30PreviewRenderMode::PromotedAudition
    );
    assert_eq!(
        state.runtime.w30_preview.routing,
        W30PreviewRenderRouting::MusicBusPreview
    );
    assert_eq!(
        state.runtime.w30_preview.source_profile,
        Some(W30PreviewSourceProfile::PromotedAudition)
    );
    assert_eq!(
        state.runtime.w30_preview.capture_id.as_deref(),
        Some("cap-01")
    );
    assert_eq!(state.runtime_view.w30_preview_mode, "promoted_audition");
    assert_eq!(state.runtime_view.w30_preview_profile, "promoted_audition");
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("auditioned cap-01 on W-30 pad bank-b/pad-03")
    );
}

#[test]
fn committed_w30_trigger_updates_preview_trigger_revision_and_log_result() {
    let graph = sample_graph();
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-b"),
        pad_id: PadId::from("pad-03"),
    });
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_trigger_pad(645),
        Some(QueueControlResult::Enqueued)
    );

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Beat,
            beat_index: 33,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        740,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(state.jam_view.lanes.w30_pending_trigger_target, None);
    assert_eq!(
        state.runtime.w30_preview.mode,
        W30PreviewRenderMode::LiveRecall
    );
    assert_eq!(
        state.runtime.w30_preview.capture_id.as_deref(),
        Some("cap-01")
    );
    assert_eq!(state.runtime.w30_preview.trigger_revision, 2);
    assert!((state.runtime.w30_preview.trigger_velocity - 0.84).abs() < f32::EPSILON);
    assert_eq!(state.runtime_view.w30_preview_mode, "live_recall");
    assert_eq!(state.runtime_view.w30_preview_profile, "promoted_recall");
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("triggered cap-01 on W-30 pad bank-b/pad-03 at beat 33 / phrase 2")
    );
}

#[test]
fn committed_w30_trigger_preserves_source_window_preview_samples() {
    let tempdir = tempdir().expect("create source audio tempdir");
    let source_path = tempdir.path().join("source.wav");
    write_pcm16_wave(&source_path, 48_000, 2, 1.0);

    let mut graph = sample_graph();
    graph.source.path = source_path.to_string_lossy().into_owned();
    graph.source.duration_seconds = 1.0;
    let mut session = sample_session(&graph);
    session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-b"),
        pad_id: PadId::from("pad-03"),
    });
    session.captures[0].source_window = Some(CaptureSourceWindow {
        source_id: graph.source.source_id.clone(),
        start_seconds: 0.0,
        end_seconds: 1.0,
        start_frame: 0,
        end_frame: 48_000,
    });
    session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
    session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
    let source_audio_cache =
        SourceAudioCache::load_pcm_wav(&source_path).expect("load source audio cache");
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    state.source_audio_cache = Some(source_audio_cache);
    state.refresh_view();

    assert_eq!(
        state.queue_w30_trigger_pad(645),
        Some(QueueControlResult::Enqueued)
    );
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Beat,
            beat_index: 33,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        740,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(
        state.runtime.w30_preview.mode,
        W30PreviewRenderMode::LiveRecall
    );
    assert_eq!(
        state.runtime.w30_preview.source_profile,
        Some(W30PreviewSourceProfile::PromotedRecall)
    );
    assert_eq!(
        state.runtime.w30_preview.capture_id.as_deref(),
        Some("cap-01")
    );
    assert_eq!(state.runtime.w30_preview.trigger_revision, 2);
    let preview = state
        .runtime
        .w30_preview
        .source_window_preview
        .as_ref()
        .expect("source-window preview");
    assert_eq!(preview.source_start_frame, 0);
    assert_eq!(preview.source_end_frame, 48_000);
    assert_eq!(preview.sample_count, W30_PREVIEW_SAMPLE_WINDOW_LEN);
    assert!(preview.samples.iter().any(|sample| sample.abs() > 0.001));
}

#[test]
fn committed_w30_internal_resample_materializes_lineage_safe_capture() {
    let mut graph = sample_graph();
    add_feral_ready_evidence(&mut graph);
    let session = sample_session(&graph);
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());

    state.session.captures[0].assigned_target = Some(CaptureTarget::W30Pad {
        bank_id: BankId::from("bank-b"),
        pad_id: PadId::from("pad-03"),
    });
    state.session.captures[0].is_pinned = true;
    state.session.captures[0].lineage_capture_refs = vec![CaptureId::from("cap-root")];
    state.session.captures[0].resample_generation_depth = 1;
    state.session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-b"));
    state.session.runtime_state.lane_state.w30.focused_pad = Some(PadId::from("pad-03"));
    state.session.runtime_state.lane_state.w30.last_capture = Some(CaptureId::from("cap-01"));
    state.refresh_view();

    assert_eq!(
        state.queue_w30_internal_resample(650),
        Some(QueueControlResult::Enqueued)
    );

    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 33,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        740,
    );

    assert_eq!(committed.len(), 1);
    let capture = state
        .session
        .captures
        .last()
        .expect("new resample capture should exist");
    assert_eq!(capture.capture_type, CaptureType::Resample);
    assert_eq!(capture.capture_id, CaptureId::from("cap-02"));
    assert_eq!(
        capture.lineage_capture_refs,
        vec![CaptureId::from("cap-root"), CaptureId::from("cap-01")]
    );
    assert_eq!(capture.resample_generation_depth, 2);
    assert_eq!(capture.assigned_target, None);
    assert!(
        capture.notes.as_deref().is_some_and(|notes| {
            notes.contains("feral rebake approved: lineage-safe W-30 reuse")
        }),
        "resample capture notes should expose the Feral rebake policy decision: {:?}",
        capture.notes
    );
    assert_eq!(
        state
            .session
            .action_log
            .actions
            .last()
            .and_then(|action| action.result.as_ref())
            .map(|result| result.summary.as_str()),
        Some("feral rebake approved: lineage-safe W-30 reuse, gen 2, lineage 2")
    );
    assert_eq!(
        state.jam_view.capture.last_capture_notes.as_deref(),
        capture.notes.as_deref()
    );
    assert_eq!(
        state.session.runtime_state.lane_state.w30.last_capture,
        Some(CaptureId::from("cap-02"))
    );
    assert_eq!(
        state.runtime.w30_resample_tap.source_capture_id.as_deref(),
        Some("cap-02")
    );
    assert_eq!(state.runtime.w30_resample_tap.lineage_capture_count, 2);
    assert_eq!(state.runtime.w30_resample_tap.generation_depth, 2);
}

#[test]
fn committed_w30_internal_resample_prints_reusable_bus_artifact() {
    let tempdir = tempdir().expect("create resample artifact tempdir");
    let source_path = tempdir.path().join("source.wav");
    let session_path = tempdir.path().join("session.json");
    let graph_path = tempdir.path().join("source_graph.json");
    write_pcm16_wave(&source_path, 48_000, 2, 8.0);

    let mut graph = sample_graph();
    graph.source.path = source_path.to_string_lossy().into_owned();
    graph.source.duration_seconds = 8.0;
    graph.source.sample_rate = 48_000;
    graph.source.channel_count = 2;
    let mut session = sample_session(&graph);
    session.captures.clear();
    session.runtime_state.lane_state.w30.last_capture = None;
    session.runtime_state.macro_state.w30_grit = 0.73;
    save_source_graph_json(&graph_path, &graph).expect("save source graph");
    save_session_json(&session_path, &session).expect("save session");

    let mut state =
        JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("load app state");
    state.queue_capture_bar(300);
    let committed_capture = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 0,
            bar_index: 1,
            phrase_index: 0,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );
    assert_eq!(committed_capture.len(), 1);
    let raw_capture_path = tempdir.path().join("captures/cap-01.wav");
    let raw_artifact =
        SourceAudioCache::load_pcm_wav(&raw_capture_path).expect("load raw capture artifact");

    assert!(state.queue_promote_last_capture(410));
    let committed_promotion = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 4,
            bar_index: 2,
            phrase_index: 0,
            scene_id: Some(SceneId::from("scene-1")),
        },
        500,
    );
    assert_eq!(committed_promotion.len(), 1);

    assert_eq!(
        state.queue_w30_internal_resample(650),
        Some(QueueControlResult::Enqueued)
    );
    let committed_resample = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 32,
            bar_index: 8,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        740,
    );
    assert_eq!(committed_resample.len(), 1);

    let capture = state
        .session
        .captures
        .last()
        .expect("new resample capture should exist");
    assert_eq!(capture.capture_type, CaptureType::Resample);
    assert_eq!(capture.capture_id, CaptureId::from("cap-02"));
    assert_eq!(
        capture.created_from_action,
        Some(committed_resample[0].action_id)
    );
    assert_eq!(capture.storage_path, "captures/cap-02.wav");
    assert_eq!(capture.source_window, None);
    assert_eq!(
        capture.lineage_capture_refs,
        vec![CaptureId::from("cap-01")]
    );
    assert_eq!(capture.resample_generation_depth, 1);
    assert!(
        capture
            .notes
            .as_deref()
            .is_some_and(|notes| notes.contains("bus print artifact written"))
    );
    assert!(
        state
            .capture_audio_cache
            .contains_key(&CaptureId::from("cap-02"))
    );

    let printed_path = tempdir.path().join("captures/cap-02.wav");
    assert!(printed_path.exists());
    let printed =
        SourceAudioCache::load_pcm_wav(&printed_path).expect("load printed resample artifact");
    assert_eq!(printed.sample_rate, 48_000);
    assert_eq!(printed.channel_count, 2);
    assert_eq!(printed.frame_count(), raw_artifact.frame_count());
    let printed_metrics = signal_metrics(printed.interleaved_samples());
    assert!(
        printed_metrics.active_samples > 1_000,
        "printed W-30 bus artifact rendered too few active samples: {}",
        printed_metrics.active_samples
    );
    assert!(
        printed_metrics.rms > 0.001,
        "printed W-30 bus artifact RMS too low: {}",
        printed_metrics.rms
    );

    let compare_len = printed
        .interleaved_samples()
        .len()
        .min(raw_artifact.interleaved_samples().len());
    assert_recipe_buffers_differ(
        "printed W-30 bus artifact vs raw capture artifact",
        &printed.interleaved_samples()[..compare_len],
        &raw_artifact.interleaved_samples()[..compare_len],
        0.001,
    );

    let fallback = render_w30_resample_tap_offline(
        &state.runtime.w30_resample_tap,
        printed.sample_rate,
        printed.channel_count,
        printed.frame_count(),
    );
    assert_recipe_buffers_differ(
        "printed W-30 bus artifact vs synthetic resample tap",
        printed.interleaved_samples(),
        &fallback,
        0.001,
    );
}
