#[test]
fn raw_capture_audition_projects_source_window_preview_samples() {
    let tempdir = tempdir().expect("create source audio tempdir");
    let source_path = tempdir.path().join("source.wav");
    write_pcm16_wave(&source_path, 48_000, 2, 1.0);

    let mut graph = sample_graph();
    graph.source.path = source_path.to_string_lossy().into_owned();
    graph.source.duration_seconds = 1.0;
    let mut session = sample_session(&graph);
    session.runtime_state.lane_state.w30.preview_mode =
        Some(W30PreviewModeState::RawCaptureAudition);
    session.captures[0].source_window = Some(CaptureSourceWindow {
        source_id: graph.source.source_id.clone(),
        start_seconds: 0.0,
        end_seconds: 1.0,
        start_frame: 0,
        end_frame: 48_000,
    });
    let source_audio_cache =
        SourceAudioCache::load_pcm_wav(&source_path).expect("load source audio cache");
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    state.source_audio_cache = Some(source_audio_cache);

    state.refresh_view();

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
fn captured_source_window_promotes_to_pad_and_auditions_source_preview() {
    let tempdir = tempdir().expect("create source audio tempdir");
    let source_path = tempdir.path().join("source.wav");
    write_pcm16_wave(&source_path, 48_000, 2, 8.0);

    let mut graph = sample_graph();
    graph.source.path = source_path.to_string_lossy().into_owned();
    graph.source.duration_seconds = 8.0;
    let mut session = sample_session(&graph);
    session.captures.clear();
    session.runtime_state.lane_state.w30.last_capture = None;
    let source_audio_cache =
        SourceAudioCache::load_pcm_wav(&source_path).expect("load source audio cache");
    let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
    state.source_audio_cache = Some(source_audio_cache);
    state.refresh_view();

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
    assert_eq!(state.session.captures.len(), 1);
    assert_eq!(
        state.session.captures[0]
            .source_window
            .as_ref()
            .map(|source_window| source_window.start_frame),
        Some(0)
    );
    assert!(state.session.captures[0].source_window.is_some());

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
        state.session.captures[0].assigned_target,
        Some(CaptureTarget::W30Pad {
            bank_id: BankId::from("bank-a"),
            pad_id: PadId::from("pad-01"),
        })
    );

    assert_eq!(
        state.queue_w30_audition(520),
        Some(QueueControlResult::Enqueued)
    );
    let committed_audition = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 8,
            bar_index: 3,
            phrase_index: 0,
            scene_id: Some(SceneId::from("scene-1")),
        },
        600,
    );

    assert_eq!(committed_audition.len(), 1);
    assert_eq!(
        state.runtime.w30_preview.mode,
        W30PreviewRenderMode::PromotedAudition
    );
    assert_eq!(
        state.runtime.w30_preview.source_profile,
        Some(W30PreviewSourceProfile::PromotedAudition)
    );
    assert_eq!(
        state.runtime.w30_preview.capture_id.as_deref(),
        Some("cap-01")
    );
    let preview = state
        .runtime
        .w30_preview
        .source_window_preview
        .as_ref()
        .expect("source-backed promoted audition preview");
    assert_eq!(preview.source_start_frame, 0);
    assert!(preview.source_end_frame > preview.source_start_frame);
    assert_eq!(preview.sample_count, W30_PREVIEW_SAMPLE_WINDOW_LEN);
    assert!(preview.samples.iter().any(|sample| sample.abs() > 0.001));
}

#[test]
fn promoted_and_recall_w30_previews_project_source_window_preview_samples() {
    for preview_mode in [
        W30PreviewModeState::PromotedAudition,
        W30PreviewModeState::LiveRecall,
    ] {
        let tempdir = tempdir().expect("create source audio tempdir");
        let source_path = tempdir.path().join("source.wav");
        write_pcm16_wave(&source_path, 48_000, 2, 1.0);

        let mut graph = sample_graph();
        graph.source.path = source_path.to_string_lossy().into_owned();
        graph.source.duration_seconds = 1.0;
        let mut session = sample_session(&graph);
        session.runtime_state.lane_state.w30.preview_mode = Some(preview_mode);
        session.captures[0].source_window = Some(CaptureSourceWindow {
            source_id: graph.source.source_id.clone(),
            start_seconds: 0.0,
            end_seconds: 1.0,
            start_frame: 0,
            end_frame: 48_000,
        });
        let source_audio_cache =
            SourceAudioCache::load_pcm_wav(&source_path).expect("load source audio cache");
        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
        state.source_audio_cache = Some(source_audio_cache);

        state.refresh_view();

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
}
