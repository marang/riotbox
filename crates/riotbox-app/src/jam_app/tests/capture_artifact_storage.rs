use std::{fs, path::Path};

use riotbox_audio::source_audio::write_interleaved_pcm16_wav;
use riotbox_core::action::CommitBoundary;

use super::{
    JamAppState, bind_synthetic_wav_identity, commit_w30_replay_step, sample_graph, sample_session,
    save_session_json,
};
use crate::jam_app::{CaptureAudioStatus, QueueControlResult};

fn session_in(dir: &Path, name: &str, level: f32) -> JamAppState {
    let source = dir.join(format!("{name}-synthetic.wav"));
    write_interleaved_pcm16_wav(&source, 48_000, 1, &vec![level; 48_000 * 8]).unwrap();
    let mut graph = sample_graph();
    graph.source.path = source.to_string_lossy().into_owned();
    graph.source.duration_seconds = 8.0;
    graph.source.channel_count = 1;
    bind_synthetic_wav_identity(&mut graph, &source);
    let mut session = sample_session(&graph);
    session.session_id = name.into();
    session.captures.clear();
    session.runtime_state.lane_state.w30.last_capture = None;
    let path = dir.join(format!("{name}.json"));
    save_session_json(&path, &session).unwrap();
    JamAppState::from_json_files(&path, None::<&Path>).unwrap()
}

fn capture(state: &mut JamAppState) {
    state.queue_capture_bar(300);
    commit_w30_replay_step(state, CommitBoundary::Phrase, 0, 1, 0, 400);
    assert_eq!(state.session.captures.len(), 1);
    assert!(state.session.captures[0].audio_identity.is_some());
}

fn assert_captures_loaded(dir: &Path, name: &str, count: usize) {
    let restored =
        JamAppState::from_json_files(dir.join(format!("{name}.json")), None::<&Path>).unwrap();
    assert_eq!(restored.session.captures.len(), count);
    for capture in &restored.session.captures {
        assert_eq!(
            restored
                .runtime
                .capture_audio_status
                .get(&capture.capture_id),
            Some(&CaptureAudioStatus::Loaded)
        );
    }
}

#[test]
fn sessions_sharing_a_directory_preserve_each_others_capture_audio() {
    let dir = tempfile::tempdir().unwrap();
    let mut first = session_in(dir.path(), "first", 0.15);
    capture(&mut first);
    first.save().unwrap();
    let first_ref = &first.session.captures[0];
    let first_path = dir.path().join(&first_ref.storage_path);
    let original = fs::read(&first_path).unwrap();
    assert_captures_loaded(dir.path(), "first", 1);

    let mut second = session_in(dir.path(), "second", -0.4);
    capture(&mut second);
    second.save().unwrap();
    assert!(
        fs::read(&first_path).unwrap() == original,
        "other Session overwrote capture"
    );
    assert_eq!(first_ref.capture_id, second.session.captures[0].capture_id);
    assert_ne!(
        first_ref.storage_path,
        second.session.captures[0].storage_path
    );
    assert_captures_loaded(dir.path(), "first", 1);
    assert_captures_loaded(dir.path(), "second", 1);
}

fn print(state: &mut JamAppState) {
    assert!(state.queue_promote_last_capture(410));
    commit_w30_replay_step(state, CommitBoundary::Bar, 4, 2, 0, 500);
    assert_eq!(
        state.queue_w30_internal_resample(650),
        Some(QueueControlResult::Enqueued)
    );
    commit_w30_replay_step(state, CommitBoundary::Phrase, 32, 8, 2, 740);
    assert_eq!(state.session.captures.len(), 2);
    assert!(state.session.captures[1].audio_identity.is_some());
}

#[test]
fn sessions_sharing_a_directory_preserve_each_others_bus_prints() {
    let dir = tempfile::tempdir().unwrap();
    let mut first = session_in(dir.path(), "first", 0.15);
    capture(&mut first);
    print(&mut first);
    first.save().unwrap();
    let first_print = &first.session.captures[1];
    let path = dir.path().join(&first_print.storage_path);
    let original = fs::read(&path).unwrap();

    let mut second = session_in(dir.path(), "second", -0.4);
    capture(&mut second);
    print(&mut second);
    second.save().unwrap();
    assert!(
        fs::read(&path).unwrap() == original,
        "other Session overwrote bus print"
    );
    assert_eq!(
        first_print.capture_id,
        second.session.captures[1].capture_id
    );
    assert_ne!(
        first_print.storage_path,
        second.session.captures[1].storage_path
    );
    assert_captures_loaded(dir.path(), "first", 2);
    assert_captures_loaded(dir.path(), "second", 2);
}

#[test]
fn failed_second_session_save_cannot_replace_first_sessions_audio() {
    let dir = tempfile::tempdir().unwrap();
    let mut first = session_in(dir.path(), "first", 0.15);
    capture(&mut first);
    first.save().unwrap();
    let first_path = dir.path().join(&first.session.captures[0].storage_path);
    let original = fs::read(&first_path).unwrap();
    let mut second = session_in(dir.path(), "second", -0.4);
    let second_path = dir.path().join("second.json");
    let previous_session = fs::read(&second_path).unwrap();
    capture(&mut second);
    let blocked_graph_destination = dir.path().join("directory-not-a-graph");
    fs::create_dir(&blocked_graph_destination).unwrap();
    second.files.as_mut().unwrap().source_graph_path = Some(blocked_graph_destination);
    assert!(second.save().is_err());
    assert!(fs::read(&first_path).unwrap() == original);
    assert_eq!(fs::read(&second_path).unwrap(), previous_session);
    assert_captures_loaded(dir.path(), "first", 1);
    assert_captures_loaded(dir.path(), "second", 0);
}

#[test]
fn new_capture_preserves_legacy_locator_and_legacy_restore_still_works() {
    let dir = tempfile::tempdir().unwrap();
    let mut legacy = session_in(dir.path(), "legacy", 0.15);
    capture(&mut legacy);
    let legacy_capture = &mut legacy.session.captures[0];
    let old_path = dir.path().join(&legacy_capture.storage_path);
    legacy_capture.storage_path = "captures/cap-01.wav".into();
    let legacy_path = dir.path().join(&legacy_capture.storage_path);
    fs::rename(&old_path, &legacy_path).unwrap();
    legacy.save().unwrap();
    let original = fs::read(&legacy_path).unwrap();
    let mut second = session_in(dir.path(), "second", -0.4);
    capture(&mut second);
    second.save().unwrap();
    assert!(fs::read(&legacy_path).unwrap() == original);
    assert_captures_loaded(dir.path(), "legacy", 1);
    assert_captures_loaded(dir.path(), "second", 1);
}

#[test]
fn artifact_write_failure_never_installs_a_trusted_capture() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = session_in(dir.path(), "session", 0.15);
    let blocked_parent = dir.path().join("regular-file-not-a-directory");
    fs::write(&blocked_parent, b"existing user content").unwrap();
    state.files.as_mut().unwrap().session_path = blocked_parent.join("session.json");
    state.queue_capture_bar(300);
    commit_w30_replay_step(&mut state, CommitBoundary::Phrase, 0, 1, 0, 400);
    let capture = &state.session.captures[0];
    assert!(capture.audio_identity.is_none());
    assert!(!state.capture_audio_cache.contains_key(&capture.capture_id));
    assert_ne!(
        state.runtime.capture_audio_status.get(&capture.capture_id),
        Some(&CaptureAudioStatus::Loaded)
    );
    assert!(
        capture
            .notes
            .as_deref()
            .unwrap()
            .contains("audio artifact pending:")
    );
    assert_eq!(fs::read(blocked_parent).unwrap(), b"existing user content");
}
