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
fn committed_source_backed_capture_writes_wav_artifact() {
    let tempdir = tempdir().expect("create capture artifact tempdir");
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
    save_source_graph_json(&graph_path, &graph).expect("save source graph");
    save_session_json(&session_path, &session).expect("save session");

    let mut state =
        JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("load app state");
    state.queue_capture_bar(300);
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Phrase,
            beat_index: 0,
            bar_index: 1,
            phrase_index: 0,
            scene_id: Some(SceneId::from("scene-1")),
        },
        400,
    );

    assert_eq!(committed.len(), 1);
    assert_eq!(state.session.captures.len(), 1);
    let capture = &state.session.captures[0];
    assert_eq!(capture.storage_path, "captures/cap-01.wav");
    let source_window = capture.source_window.as_ref().expect("source window");
    let capture_path = tempdir.path().join(&capture.storage_path);
    assert!(capture_path.exists());
    assert!(
        capture
            .notes
            .as_deref()
            .is_some_and(|notes| notes.contains("audio artifact written"))
    );

    let artifact = SourceAudioCache::load_pcm_wav(&capture_path).expect("load capture artifact");
    let expected_frames = usize::try_from(
        source_window
            .end_frame
            .saturating_sub(source_window.start_frame),
    )
    .expect("expected source-window frame count");
    assert_eq!(artifact.sample_rate, 48_000);
    assert_eq!(artifact.channel_count, 2);
    assert_eq!(artifact.frame_count(), expected_frames);
    assert!((artifact.duration_seconds() - 7.619).abs() < 0.02);
    assert!(
        artifact
            .interleaved_samples()
            .iter()
            .any(|sample| sample.abs() > 0.01)
    );

    state.save().expect("save session with capture artifact");
    let reloaded =
        JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("reload app");
    assert_eq!(
        reloaded.session.captures[0].storage_path,
        capture.storage_path
    );
    assert!(capture_path.exists());
}

#[test]
fn focused_w30_pad_trigger_uses_capture_artifact_preview_when_source_cache_unavailable() {
    let tempdir = tempdir().expect("create artifact-backed playback tempdir");
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

    let capture_id = CaptureId::from("cap-01");
    let capture_path = tempdir.path().join("captures/cap-01.wav");
    let artifact = SourceAudioCache::load_pcm_wav(&capture_path).expect("load capture artifact");
    assert!(state.capture_audio_cache.contains_key(&capture_id));

    state.source_audio_cache = None;
    fs::remove_file(&source_path).expect("remove source to prove artifact-backed preview");

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
        state.queue_w30_trigger_pad(645),
        Some(QueueControlResult::Enqueued)
    );
    let committed_trigger = state.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Beat,
            beat_index: 33,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        740,
    );
    assert_eq!(committed_trigger.len(), 1);

    assert_eq!(
        state.runtime.w30_preview.mode,
        W30PreviewRenderMode::LiveRecall
    );
    assert_eq!(
        state.runtime.w30_preview.capture_id.as_deref(),
        Some("cap-01")
    );
    assert_eq!(state.runtime.w30_preview.trigger_revision, 4);
    let preview = state
        .runtime
        .w30_preview
        .source_window_preview
        .as_ref()
        .expect("artifact-backed W-30 preview");
    assert_eq!(preview.source_start_frame, 0);
    assert_eq!(
        preview.source_end_frame,
        u64::try_from(artifact.frame_count()).expect("artifact frame count fits u64")
    );
    assert_eq!(preview.sample_count, W30_PREVIEW_SAMPLE_WINDOW_LEN);
    assert!(preview.samples.iter().any(|sample| sample.abs() > 0.001));
    let pad_playback = state
        .runtime
        .w30_preview
        .pad_playback
        .as_ref()
        .expect("artifact-backed W-30 pad playback");
    assert_eq!(pad_playback.source_start_frame, 0);
    assert_eq!(
        pad_playback.source_end_frame,
        u64::try_from(artifact.frame_count()).expect("artifact frame count fits u64")
    );
    assert!(pad_playback.loop_enabled);
    assert_eq!(
        pad_playback.sample_count,
        artifact
            .frame_count()
            .min(W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN)
    );
    assert!(pad_playback.sample_count > W30_PREVIEW_SAMPLE_WINDOW_LEN);
    assert!(
        pad_playback.samples[..pad_playback.sample_count]
            .iter()
            .any(|sample| sample.abs() > 0.001)
    );

    let artifact_buffer = render_w30_preview_offline(
        &state.runtime.w30_preview,
        48_000,
        2,
        pad_playback.sample_count,
    );
    let artifact_metrics = signal_metrics(&artifact_buffer);
    assert!(
        artifact_metrics.active_samples > 1_000,
        "artifact-backed W-30 pad playback rendered too few active samples: {}",
        artifact_metrics.active_samples
    );
    assert!(
        artifact_metrics.rms > 0.001,
        "artifact-backed W-30 pad playback RMS too low: {}",
        artifact_metrics.rms
    );
    let late_playback_start = W30_PREVIEW_SAMPLE_WINDOW_LEN * 2;
    assert!(
        artifact_buffer[late_playback_start..]
            .iter()
            .any(|sample| sample.abs() > 0.001),
        "artifact-backed W-30 pad playback should remain audible beyond the fixed preview window"
    );

    let mut fixed_preview_only = state.runtime.w30_preview.clone();
    fixed_preview_only.pad_playback = None;
    let fixed_preview_buffer =
        render_w30_preview_offline(&fixed_preview_only, 48_000, 2, pad_playback.sample_count);
    assert_recipe_buffers_differ(
        "duration-aware W-30 pad playback vs fixed preview window",
        &artifact_buffer,
        &fixed_preview_buffer,
        0.001,
    );

    let mut fallback_preview = state.runtime.w30_preview.clone();
    fallback_preview.source_window_preview = None;
    fallback_preview.pad_playback = None;
    let fallback_buffer =
        render_w30_preview_offline(&fallback_preview, 48_000, 2, pad_playback.sample_count);
    assert_recipe_buffers_differ(
        "artifact-backed W-30 pad playback vs fallback preview",
        &artifact_buffer,
        &fallback_buffer,
        0.001,
    );
}

#[test]
fn reloaded_session_uses_capture_artifact_cache_without_source_audio() {
    let tempdir = tempdir().expect("create artifact restore tempdir");
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

    let capture_id = CaptureId::from("cap-01");
    let capture_path = tempdir.path().join("captures/cap-01.wav");
    assert!(capture_path.is_file());
    state.save().expect("save artifact-backed session");
    fs::remove_file(&source_path).expect("remove source to prove reload uses artifact");

    let mut reloaded =
        JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("reload app state");
    assert!(reloaded.source_audio_cache.is_none());
    assert!(reloaded.capture_audio_cache.contains_key(&capture_id));
    let capture = reloaded
        .session
        .captures
        .iter()
        .find(|capture| capture.capture_id == capture_id)
        .expect("reloaded capture")
        .clone();
    assert_eq!(
        reloaded
            .require_capture_artifact_for_hydration(&capture)
            .expect("artifact preflight passes after reload"),
        capture_path
    );

    assert_eq!(
        reloaded.queue_w30_trigger_pad(645),
        Some(QueueControlResult::Enqueued)
    );
    let committed_trigger = reloaded.commit_ready_actions(
        CommitBoundaryState {
            kind: CommitBoundary::Beat,
            beat_index: 33,
            bar_index: 9,
            phrase_index: 2,
            scene_id: Some(SceneId::from("scene-1")),
        },
        740,
    );
    assert_eq!(committed_trigger.len(), 1);

    let pad_playback = reloaded
        .runtime
        .w30_preview
        .pad_playback
        .as_ref()
        .expect("artifact-backed W-30 pad playback after reload");
    assert!(pad_playback.sample_count > W30_PREVIEW_SAMPLE_WINDOW_LEN);
    assert!(
        pad_playback.samples[..pad_playback.sample_count]
            .iter()
            .any(|sample| sample.abs() > 0.001)
    );

    let buffer = render_w30_preview_offline(&reloaded.runtime.w30_preview, 48_000, 2, 48_000);
    let metrics = signal_metrics(&buffer);
    assert!(
        metrics.rms > 0.001,
        "artifact-backed restore render RMS too low: {}",
        metrics.rms
    );
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
