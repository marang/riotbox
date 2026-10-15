use std::fs;

use riotbox_audio::source_audio::SourceAudioCache;
use riotbox_core::action::LiveRecordingDuration;

use super::{RenderWindow, report_render};

fn window(frames: usize) -> RenderWindow {
    RenderWindow {
        rate: 44_100,
        channels: 2,
        duration: LiveRecordingDuration::TwoBars,
        frames,
        start_beats: 4.0,
    }
}

#[test]
fn offline_publication_reads_back_geometry_and_hashes_exact_bytes() {
    let owner = tempfile::tempdir().unwrap();
    let path = owner.path().join("master.wav");
    let report = report_render(&path, window(2), &[0.5, -0.5, 0.25, -0.25], 3);
    assert!(report.passed);
    assert_eq!(
        report.artifact_sha256.unwrap(),
        super::file_hash(&path).unwrap()
    );
    assert!(report.artifact_error.is_none());
    let cache = SourceAudioCache::load_pcm_wav(&path).unwrap();
    assert_eq!(cache.sample_rate, 44_100);
    assert_eq!(cache.channel_count, 2);
    assert_eq!(cache.frame_count(), 2);
    assert_eq!(report.limiter_applied_sample_count, 3);
}

#[test]
fn occupied_artifact_is_preserved_and_computed_metrics_survive() {
    let owner = tempfile::tempdir().unwrap();
    let path = owner.path().join("master.wav");
    let sentinel = b"unowned destination";
    fs::write(&path, sentinel).unwrap();
    let report = report_render(&path, window(2), &[0.5, -0.5, 0.25, -0.25], 7);
    assert!(!report.passed);
    assert_eq!(fs::read(&path).unwrap(), sentinel);
    assert!(report.finite_samples && report.expected_sample_count);
    assert_eq!(report.peak_abs, Some(0.5));
    assert!(report.rms.unwrap() > 0.0);
    assert_eq!(report.active_samples, 4);
    assert_eq!(report.limiter_applied_sample_count, 7);
    assert!(report.artifact_sha256.is_none());
    assert!(report.artifact_error.is_some());
    // The temporary file is owned and cleaned; the sentinel alone remains.
    assert_eq!(fs::read_dir(owner.path()).unwrap().count(), 1);
}

#[cfg(unix)]
#[test]
fn artifact_symlink_is_not_followed_or_replaced() {
    let owner = tempfile::tempdir().unwrap();
    let target = owner.path().join("sentinel");
    let path = owner.path().join("master.wav");
    fs::write(&target, b"keep").unwrap();
    std::os::unix::fs::symlink(&target, &path).unwrap();
    let report = report_render(&path, window(1), &[0.5, -0.5], 0);
    assert!(!report.passed);
    assert_eq!(fs::read(&target).unwrap(), b"keep");
    assert!(fs::symlink_metadata(path).unwrap().file_type().is_symlink());
}

#[test]
fn missing_parent_retains_render_evidence_without_creating_directories() {
    let owner = tempfile::tempdir().unwrap();
    let path = owner.path().join("absent/master.wav");
    let report = report_render(&path, window(1), &[0.5, -0.5], 0);
    assert!(!report.passed);
    assert_eq!(report.peak_abs, Some(0.5));
    assert_eq!(report.active_samples, 2);
    assert!(report.artifact_error.is_some());
    assert!(!path.parent().unwrap().exists());
}

#[test]
fn invalid_payload_never_publishes_and_silent_or_clipped_output_never_passes() {
    let owner = tempfile::tempdir().unwrap();
    for (name, frames, samples) in [
        ("nonfinite", 1, vec![f32::NAN, 0.5]),
        ("short", 2, vec![0.5, -0.5]),
        ("overflow", usize::MAX, vec![0.5, -0.5]),
    ] {
        let path = owner.path().join(name);
        let report = report_render(&path, window(frames), &samples, 0);
        assert!(!report.passed);
        assert!(report.artifact_error.is_some());
        assert!(!path.exists());
        // Nonfinite metrics serialize as null, not invalid JSON.
        serde_json::to_vec(&report).unwrap();
    }
    for (name, samples) in [("silent", [0.0, 0.0]), ("clipped", [1.0, -1.0])] {
        let report = report_render(&owner.path().join(name), window(1), &samples, 0);
        assert!(!report.passed);
    }
}
