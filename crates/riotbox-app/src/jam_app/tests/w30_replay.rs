#[test]
fn w30_replay_executor_matches_committed_app_state_and_preview_output() {
    let tempdir = tempdir().expect("create source audio tempdir");
    let source_path = tempdir.path().join("source.wav");
    write_pcm16_wave(&source_path, 48_000, 2, 2.0);
    let source_audio_cache =
        SourceAudioCache::load_pcm_wav(&source_path).expect("load source audio cache");

    let mut graph = sample_graph();
    graph.source.path = source_path.to_string_lossy().into_owned();
    graph.source.duration_seconds = 2.0;
    let mut committed_state =
        w30_slice_pool_state_with_source_windows(graph.clone(), source_audio_cache.clone());
    let cap_02 = committed_state
        .session
        .captures
        .iter_mut()
        .find(|capture| capture.capture_id == CaptureId::from("cap-02"))
        .expect("cap-02 fixture");
    cap_02.source_window = Some(CaptureSourceWindow {
        source_id: graph.source.source_id.clone(),
        start_seconds: 0.0625,
        end_seconds: 0.5625,
        start_frame: 3_000,
        end_frame: 27_000,
    });
    committed_state.refresh_view();
    let replay_base_session = committed_state.session.clone();
    let initial_source_window = committed_state
        .runtime
        .w30_preview
        .source_window_preview
        .as_ref()
        .expect("initial source-window preview")
        .clone();
    let initial_preview = render_w30_replay_buffer(&committed_state);

    assert_eq!(
        committed_state.queue_w30_browse_slice_pool(300),
        Some(QueueControlResult::Enqueued)
    );
    commit_w30_replay_step(&mut committed_state, CommitBoundary::Beat, 33, 9, 2, 400);
    let committed_browse_preview = committed_state
        .runtime
        .w30_preview
        .source_window_preview
        .as_ref()
        .expect("committed browse source-window preview")
        .clone();
    let committed_browse = render_w30_replay_buffer(&committed_state);

    let plan = riotbox_core::replay::build_committed_replay_plan(
        &committed_state.session.action_log,
    )
    .expect("committed W-30 action log builds replay plan");
    let mut replayed_session = replay_base_session;
    replayed_session.action_log = committed_state.session.action_log.clone();
    let report = riotbox_core::replay::apply_replay_plan_to_session(&mut replayed_session, &plan)
        .expect("W-30 replay executor applies cue subset");
    let mut replayed_state =
        JamAppState::from_parts(replayed_session, Some(graph), ActionQueue::new());
    replayed_state.source_audio_cache = Some(source_audio_cache);
    replayed_state.refresh_view();
    let replayed_browse = render_w30_replay_buffer(&replayed_state);

    assert_eq!(report.applied_action_ids.len(), 1);
    assert_eq!(
        replayed_state.session.runtime_state.lane_state.w30,
        committed_state.session.runtime_state.lane_state.w30
    );
    assert_eq!(
        replayed_state.session.runtime_state.macro_state.w30_grit,
        committed_state.session.runtime_state.macro_state.w30_grit
    );
    assert_eq!(
        replayed_state.runtime.w30_preview,
        committed_state.runtime.w30_preview
    );
    assert_recipe_buffers_match(
        "replayed W-30 browse -> committed W-30 browse",
        &replayed_browse,
        &committed_browse,
        0.00001,
    );
    assert_w30_replay_buffers_differ(
        "initial W-30 recall -> browse",
        &initial_preview,
        &committed_browse,
        0.0003,
        0.0008,
    );
    assert!(
        recipe_signal_delta_rms(
            &initial_source_window.samples,
            &committed_browse_preview.samples,
        ) > 0.01,
        "W-30 browse should change the source-window preview samples"
    );
}

fn commit_w30_replay_step(
    state: &mut JamAppState,
    kind: CommitBoundary,
    beat_index: u64,
    bar_index: u64,
    phrase_index: u64,
    committed_at: u64,
) {
    let committed = state.commit_ready_actions(
        CommitBoundaryState {
            kind,
            beat_index,
            bar_index,
            phrase_index,
            scene_id: Some(SceneId::from("scene-1")),
        },
        committed_at,
    );
    assert_eq!(committed.len(), 1);
}

fn render_w30_replay_buffer(state: &JamAppState) -> Vec<f32> {
    let buffer = render_w30_preview_offline(
        &state.runtime.w30_preview,
        48_000,
        2,
        W30_PREVIEW_SAMPLE_WINDOW_LEN,
    );
    let metrics = signal_metrics(&buffer);
    assert!(
        metrics.active_samples > 100 && metrics.peak_abs > 0.001 && metrics.rms > 0.0001,
        "W-30 replay render too close to silence: active {}, peak {}, rms {}",
        metrics.active_samples,
        metrics.peak_abs,
        metrics.rms
    );
    buffer
}

fn assert_w30_replay_buffers_differ(
    label: &str,
    left: &[f32],
    right: &[f32],
    min_rms_delta: f32,
    min_peak_delta: f32,
) {
    let delta = signal_delta_metrics(left, right);
    assert!(
        delta.rms >= min_rms_delta,
        "{label} signal delta RMS {} below {min_rms_delta}; peak {}, active {}, zero crossings {}",
        delta.rms,
        delta.peak_abs,
        delta.active_samples,
        delta.zero_crossings
    );
    assert!(
        delta.peak_abs >= min_peak_delta,
        "{label} signal delta peak {} below {min_peak_delta}",
        delta.peak_abs
    );
}
