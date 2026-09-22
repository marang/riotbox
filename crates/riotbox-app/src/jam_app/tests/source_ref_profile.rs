use std::{fs, path::Path};

use super::{
    JamAppState, SourceAudioStatus, bind_synthetic_wav_identity, sample_graph, sample_session,
    write_pcm16_wave,
};

#[test]
fn unknown_legacy_decode_profile_cannot_admit_source_audio_with_a_standard_graph_profile() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("synthetic.wav");
    write_pcm16_wave(&source, 48_000, 2, 0.1);
    let mut graph = sample_graph();
    graph.source.path = source.to_string_lossy().into_owned();
    graph.source.duration_seconds = 0.1;
    bind_synthetic_wav_identity(&mut graph, &source);
    let mut json = serde_json::to_value(sample_session(&graph)).unwrap();
    json["source_refs"][0]["decode_profile"] = "unknown-historical-profile".into();
    let path = dir.path().join("session.json");
    fs::write(&path, serde_json::to_vec(&json).unwrap()).unwrap();
    let restored = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
    assert!(
        restored.source_audio_cache.is_none(),
        "unverified decode profile was admitted"
    );
    assert!(
        matches!(&restored.runtime.source_audio.status, SourceAudioStatus::Unavailable { reason, .. }
        if reason.contains("decode profile mismatch"))
    );
    fs::remove_file(&source).unwrap();
    let unavailable = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
    assert!(
        matches!(unavailable.runtime.source_audio.status, SourceAudioStatus::Unavailable { reason, .. }
        if reason.contains("decode profile mismatch") && !reason.contains("I/O"))
    );
}

#[test]
fn supported_custom_profiles_restore_but_ambiguous_legacy_names_do_not_get_guessed() {
    use riotbox_core::source_graph::DecodeProfile;
    use serde_json::json;

    for (profile, wire, expected_loaded) in [
        (DecodeProfile::Native, json!("native"), true),
        (
            DecodeProfile::Custom("my-decoder-v1".into()),
            json!("my-decoder-v1"),
            true,
        ),
        (
            DecodeProfile::Custom("native".into()),
            json!("native"),
            false,
        ),
        (
            DecodeProfile::Custom("native".into()),
            json!({"Custom": "native"}),
            true,
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("synthetic.wav");
        write_pcm16_wave(&source, 48_000, 2, 0.1);
        let mut graph = sample_graph();
        graph.source.path = source.to_string_lossy().into_owned();
        graph.source.duration_seconds = 0.1;
        graph.source.decode_profile = profile;
        bind_synthetic_wav_identity(&mut graph, &source);
        let mut session = serde_json::to_value(sample_session(&graph)).unwrap();
        session["source_refs"][0]["decode_profile"] = wire;
        let path = dir.path().join("session.json");
        let original = serde_json::to_vec(&session).unwrap();
        fs::write(&path, &original).unwrap();
        let restored = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
        assert_eq!(restored.source_audio_cache.is_some(), expected_loaded);
        assert_eq!(restored.source_graph, Some(graph));
        assert_eq!(
            fs::read(path).unwrap(),
            original,
            "restore must not rewrite legacy metadata"
        );
    }
}
