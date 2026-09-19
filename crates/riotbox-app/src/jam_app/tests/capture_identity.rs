use super::{JamAppState, fs, sample_graph, sample_session, save_session_json, tempdir};
use crate::jam_app::{CaptureAudioStatus, migrate_legacy_capture_identities};
use riotbox_audio::source_audio::write_interleaved_pcm16_wav;
use riotbox_core::session::CaptureAudioIdentityProvenance;

fn legacy_fixture() -> (
    tempfile::TempDir,
    std::path::PathBuf,
    riotbox_core::ids::CaptureId,
) {
    let dir = tempdir().unwrap();
    let session_path = dir.path().join("session.json");
    write_interleaved_pcm16_wav(dir.path().join("capture.wav"), 48_000, 1, &[0.1; 128]).unwrap();
    let mut session = sample_session(&sample_graph());
    session.captures.truncate(1);
    session.captures[0].storage_path = "capture.wav".into();
    let id = session.captures[0].capture_id.clone();
    session.runtime_state.lane_state.w30.last_capture = Some(id.clone());
    save_session_json(&session_path, &session).unwrap();
    (dir, session_path, id)
}

#[test]
fn explicit_legacy_adoption_is_read_only_in_preview_and_detects_later_pcm_changes() {
    let (dir, session_path, id) = legacy_fixture();
    let original = fs::read(&session_path).unwrap();
    let selected = [id.clone()];
    let preview = migrate_legacy_capture_identities(&session_path, &selected, false).unwrap();
    assert!(!preview[0].adopted);
    assert_eq!(fs::read(&session_path).unwrap(), original);
    let adopted = migrate_legacy_capture_identities(&session_path, &selected, true).unwrap();
    assert!(adopted[0].adopted);
    assert_eq!(adopted[0].sha256, preview[0].sha256);
    let restored = JamAppState::from_json_files(&session_path, None::<&std::path::Path>).unwrap();
    assert!(restored.capture_audio_cache.contains_key(&id));
    assert!(matches!(
        restored.session.captures[0]
            .audio_identity
            .as_ref()
            .unwrap()
            .provenance,
        CaptureAudioIdentityProvenance::AdoptedLegacyV1 { .. }
    ));
    let accepted_session = fs::read(&session_path).unwrap();
    write_interleaved_pcm16_wav(dir.path().join("capture.wav"), 48_000, 1, &[0.4; 128]).unwrap();
    let mut changed =
        JamAppState::from_json_files(&session_path, None::<&std::path::Path>).unwrap();
    assert_eq!(
        changed.runtime.capture_audio_status.get(&id),
        Some(&CaptureAudioStatus::Changed)
    );
    assert!(changed.capture_audio_cache.is_empty());
    changed.source_audio_cache = Some(
        riotbox_audio::source_audio::SourceAudioCache::load_pcm_wav(dir.path().join("capture.wav"))
            .unwrap(),
    );
    changed.refresh_view();
    assert!(changed.runtime.w30_preview.source_window_preview.is_none());
    assert!(changed.runtime.w30_preview.pad_playback.is_none());
    assert!(
        changed
            .runtime_view
            .runtime_warnings
            .iter()
            .any(|warning| warning.contains("Changed"))
    );
    assert!(migrate_legacy_capture_identities(&session_path, &selected, true).is_err());
    assert_eq!(fs::read(&session_path).unwrap(), accepted_session);
}

#[test]
fn migration_is_atomic_for_missing_selected_capture_and_does_not_rewrite_known_identity() {
    let (_dir, session_path, id) = legacy_fixture();
    let original = fs::read(&session_path).unwrap();
    assert!(
        migrate_legacy_capture_identities(&session_path, &[id.clone(), "unknown".into()], true)
            .is_err()
    );
    assert_eq!(fs::read(&session_path).unwrap(), original);
    assert!(
        migrate_legacy_capture_identities(&session_path, &[id.clone(), id.clone()], true).is_err()
    );
    migrate_legacy_capture_identities(&session_path, std::slice::from_ref(&id), true).unwrap();
    let accepted = fs::read(&session_path).unwrap();
    let repeated = migrate_legacy_capture_identities(&session_path, &[id], true).unwrap();
    assert!(!repeated[0].adopted);
    assert_eq!(fs::read(&session_path).unwrap(), accepted);
}

#[test]
fn duplicate_capture_identity_never_leaves_a_trusted_cache_entry() {
    let (_dir, session_path, id) = legacy_fixture();
    migrate_legacy_capture_identities(&session_path, &[id], true).unwrap();
    let mut session = riotbox_core::persistence::load_session_json(&session_path).unwrap();
    let mut duplicate = session.captures[0].clone();
    duplicate.audio_identity = None;
    session.captures.push(duplicate);
    save_session_json(&session_path, &session).unwrap();
    let restored = JamAppState::from_json_files(&session_path, None::<&std::path::Path>).unwrap();
    assert!(restored.capture_audio_cache.is_empty());
}

#[test]
fn legacy_capture_is_not_silently_trusted_on_restore() {
    let dir = tempdir().unwrap();
    let session_path = dir.path().join("session.json");
    let artifact = dir.path().join("capture.wav");
    write_interleaved_pcm16_wav(&artifact, 48_000, 1, &[0.1; 128]).unwrap();
    let mut session = sample_session(&sample_graph());
    session.captures.truncate(1);
    session.captures[0].storage_path = "capture.wav".into();
    save_session_json(&session_path, &session).unwrap();
    let original = fs::read(&session_path).unwrap();
    let restored = JamAppState::from_json_files(&session_path, None::<&std::path::Path>).unwrap();
    assert!(restored.capture_audio_cache.is_empty());
    assert_eq!(fs::read(&session_path).unwrap(), original);
}

#[test]
fn malformed_identity_missing_and_invalid_wav_are_never_loaded() {
    let (dir, session_path, id) = legacy_fixture();
    migrate_legacy_capture_identities(&session_path, std::slice::from_ref(&id), true).unwrap();
    let accepted = fs::read(&session_path).unwrap();
    let mut session = riotbox_core::persistence::load_session_json(&session_path).unwrap();
    session.captures[0].audio_identity.as_mut().unwrap().sha256 = "sha256:bad".into();
    save_session_json(&session_path, &session).unwrap();
    let restored = JamAppState::from_json_files(&session_path, None::<&std::path::Path>).unwrap();
    assert!(restored.capture_audio_cache.is_empty());
    assert_eq!(
        restored.runtime.capture_audio_status.get(&id),
        Some(&CaptureAudioStatus::InvalidIdentity)
    );
    fs::write(&session_path, &accepted).unwrap();
    fs::write(dir.path().join("capture.wav"), b"not a WAV").unwrap();
    let restored = JamAppState::from_json_files(&session_path, None::<&std::path::Path>).unwrap();
    assert!(restored.capture_audio_cache.is_empty());
    assert!(matches!(
        restored.runtime.capture_audio_status.get(&id),
        Some(CaptureAudioStatus::Unavailable { .. })
    ));
    fs::remove_file(dir.path().join("capture.wav")).unwrap();
    let restored = JamAppState::from_json_files(&session_path, None::<&std::path::Path>).unwrap();
    assert!(restored.capture_audio_cache.is_empty());
    assert!(matches!(
        restored.runtime.capture_audio_status.get(&id),
        Some(CaptureAudioStatus::Unavailable { .. })
    ));
    assert_eq!(fs::read(&session_path).unwrap(), accepted);
}
