use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use super::{
    JamAppState, SourceAudioStatus, bind_synthetic_wav_identity, sample_graph, sample_session,
    save_session_json, write_pcm16_wave,
};

#[test]
fn source_restore_rejects_fifos_without_a_producer() {
    const PROBE: &str = "RIOTBOX_APP_FIFO_PROBE_PATH";
    if let Some(path) = std::env::var_os(PROBE) {
        let path = Path::new(&path);
        let dir = tempfile::tempdir().unwrap();
        let session_path = dir.path().join("session.json");
        let mut graph = sample_graph();
        graph.source.path = path.to_string_lossy().into_owned();
        let mut session = sample_session(&graph);
        session.runtime_state.source_monitor.mode = riotbox_core::action::SourceMonitorMode::Source;
        save_session_json(&session_path, &session).unwrap();
        let state = JamAppState::from_json_files(&session_path, None::<&Path>).unwrap();
        assert!(state.source_audio_cache.is_none());
        assert!(state.source_monitor_render_state().source.is_none());
        assert!(matches!(&state.runtime.source_audio.status,
            SourceAudioStatus::Unavailable { reason, .. } if reason.contains("not a regular file")));
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let fifo = dir.path().join("synthetic.wav");
    assert!(
        Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    let link = dir.path().join("fifo-link.wav");
    std::os::unix::fs::symlink(&fifo, &link).unwrap();
    for path in [&fifo, &link] {
        run_bounded_fifo_child(path, PROBE);
    }
}

#[test]
fn source_restore_rejects_a_directory_without_admitting_a_cache() {
    let dir = tempfile::tempdir().unwrap();
    let session_path = dir.path().join("session.json");
    let mut graph = sample_graph();
    graph.source.path = dir.path().to_string_lossy().into_owned();
    save_session_json(&session_path, &sample_session(&graph)).unwrap();
    let state = JamAppState::from_json_files(&session_path, None::<&Path>).unwrap();
    assert!(state.source_audio_cache.is_none());
    assert!(matches!(&state.runtime.source_audio.status,
        SourceAudioStatus::Unavailable { reason, .. } if reason.contains("not a regular file")));
}

#[test]
fn source_restore_preserves_regular_symlinks_and_hash_validation() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("synthetic.wav");
    let link = dir.path().join("source-link.wav");
    let session_path = dir.path().join("session.json");
    write_pcm16_wave(&source, 48_000, 2, 0.1);
    std::os::unix::fs::symlink(&source, &link).unwrap();
    let mut graph = sample_graph();
    graph.source.path = link.to_string_lossy().into_owned();
    graph.source.duration_seconds = 0.1;
    bind_synthetic_wav_identity(&mut graph, &source);
    save_session_json(&session_path, &sample_session(&graph)).unwrap();
    let original_session = fs::read(&session_path).unwrap();
    let loaded = JamAppState::from_json_files(&session_path, None::<&Path>).unwrap();
    assert!(loaded.source_audio_cache.is_some());
    assert!(matches!(
        loaded.runtime.source_audio.status,
        SourceAudioStatus::Loaded { .. }
    ));
    let mut changed = fs::read(&source).unwrap();
    let index = changed.len() - 1;
    changed[index] ^= 1;
    fs::write(&source, changed).unwrap();
    let unavailable = JamAppState::from_json_files(&session_path, None::<&Path>).unwrap();
    assert!(unavailable.source_audio_cache.is_none());
    assert!(matches!(&unavailable.runtime.source_audio.status,
        SourceAudioStatus::Unavailable { reason, .. } if reason.contains("hash mismatch")));
    assert_eq!(fs::read(&session_path).unwrap(), original_session);
}

fn run_bounded_fifo_child(path: &Path, probe: &str) {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "jam_app::tests::source_file_admission::source_restore_rejects_fifos_without_a_producer", "--nocapture"])
        .env(probe, path)
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(4);
    let mut timed_out = false;
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() >= deadline {
            timed_out = true;
            child.kill().unwrap();
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    assert!(
        !timed_out && output.status.success(),
        "restore must reject FIFO without a producer (timed_out={timed_out}): {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
