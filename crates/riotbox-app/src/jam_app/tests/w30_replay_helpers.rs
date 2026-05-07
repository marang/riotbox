fn w30_source_backed_replay_state() -> (
    tempfile::TempDir,
    SourceGraph,
    SourceAudioCache,
    JamAppState,
) {
    let tempdir = tempdir().expect("create source audio tempdir");
    let source_path = tempdir.path().join("source.wav");
    write_pcm16_wave(&source_path, 48_000, 2, 2.0);
    let source_audio_cache =
        SourceAudioCache::load_pcm_wav(&source_path).expect("load source audio cache");

    let mut graph = sample_graph();
    graph.source.path = source_path.to_string_lossy().into_owned();
    graph.source.duration_seconds = 2.0;
    let mut state =
        w30_slice_pool_state_with_source_windows(graph.clone(), source_audio_cache.clone());
    let cap_02 = state
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
    state.refresh_view();

    (tempdir, graph, source_audio_cache, state)
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

fn render_w30_preview_buffer(state: &JamAppState) -> Vec<f32> {
    render_w30_preview_offline(
        &state.runtime.w30_preview,
        48_000,
        2,
        W30_PREVIEW_SAMPLE_WINDOW_LEN,
    )
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
