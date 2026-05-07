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
