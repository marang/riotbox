use std::fs;
#[cfg(unix)]
use std::path::Path;

use super::{SourceAudioCache, SourceAudioError, pcm16_wave_bytes};

#[test]
fn regular_source_wav_still_decodes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic.wav");
    let bytes = pcm16_wave_bytes(48_000, 1, &[0.25, -0.25]).unwrap();
    fs::write(&path, bytes).unwrap();
    let cache = SourceAudioCache::load_pcm_wav(&path).unwrap();
    assert_eq!(cache.frame_count(), 2);
    assert_eq!(cache.sample_rate, 48_000);
}

#[test]
fn source_directory_is_an_io_error() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(
        SourceAudioCache::load_pcm_wav(dir.path()),
        Err(SourceAudioError::Io(_))
    ));
}

#[cfg(unix)]
#[test]
fn regular_source_symlink_still_decodes() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("synthetic.wav");
    let link = dir.path().join("source-link.wav");
    fs::write(
        &source,
        pcm16_wave_bytes(48_000, 1, &[0.25, -0.25]).unwrap(),
    )
    .unwrap();
    std::os::unix::fs::symlink(&source, &link).unwrap();
    let cache = SourceAudioCache::load_pcm_wav(&link).unwrap();
    assert_eq!(cache.path, link);
    assert_eq!(cache.frame_count(), 2);
}

#[cfg(unix)]
#[test]
fn fifo_sources_fail_without_a_writer() {
    const PROBE: &str = "RIOTBOX_AUDIO_FIFO_PROBE_PATH";
    if let Some(path) = std::env::var_os(PROBE) {
        let error = SourceAudioCache::load_pcm_wav(path).unwrap_err();
        assert!(matches!(error, SourceAudioError::Io(_)));
        assert!(error.to_string().contains("not a regular file"));
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let fifo = dir.path().join("synthetic.wav");
    assert!(
        std::process::Command::new("mkfifo")
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

#[cfg(unix)]
fn run_bounded_fifo_child(path: &Path, probe: &str) {
    use std::{
        process::{Command, Stdio},
        thread,
        time::{Duration, Instant},
    };
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "source_audio::file_admission_tests::fifo_sources_fail_without_a_writer",
            "--nocapture",
        ])
        .env(probe, path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
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
        "FIFO admission must reject without a producer (timed_out={timed_out}): {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
