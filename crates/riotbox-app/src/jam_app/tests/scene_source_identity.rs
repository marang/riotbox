use std::{fs, path::Path};

use super::{JamAppState, sample_graph, sample_session};
use riotbox_core::{
    persistence::{load_session_json, save_session_json},
    session::SceneSourceBinding,
};

#[test]
fn legacy_scene_restore_save_reload_materializes_identity_without_rewriting_graph_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let mut graph = sample_graph();
    graph.source.path = dir
        .path()
        .join("deliberately-absent.wav")
        .to_string_lossy()
        .into_owned();
    let original_graph = serde_json::to_vec(&graph).unwrap();
    let mut session = sample_session(&graph);
    session.runtime_state.scene_state.active_scene = Some("scene-01-old-label".into());
    session.runtime_state.scene_state.scenes = vec!["scene-01-old-label".into()];
    session.runtime_state.transport.current_scene = Some("scene-01-old-label".into());
    let path = dir.path().join("session.json");
    save_session_json(&path, &session).unwrap();
    let original_session = fs::read(&path).unwrap();
    let restored = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
    assert_eq!(
        fs::read(&path).unwrap(),
        original_session,
        "load is not an implicit disk migration"
    );
    assert!(
        restored
            .session
            .runtime_state
            .scene_state
            .source_bindings
            .is_some()
    );
    restored.save().unwrap();
    let reloaded = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
    assert_eq!(
        reloaded.session.runtime_state.scene_state,
        restored.session.runtime_state.scene_state
    );
    assert_eq!(
        serde_json::to_vec(reloaded.source_graph.as_ref().unwrap()).unwrap(),
        original_graph
    );
    assert_eq!(
        reloaded.session.source_graph_refs,
        session.source_graph_refs
    );
    assert_eq!(
        load_session_json(&path).unwrap().runtime_state.scene_state,
        restored.session.runtime_state.scene_state
    );
}

#[test]
fn invalid_explicit_scene_identity_fails_restore_and_save_without_touching_disk() {
    let dir = tempfile::tempdir().unwrap();
    let mut graph = sample_graph();
    graph.source.path = dir
        .path()
        .join("deliberately-absent.wav")
        .to_string_lossy()
        .into_owned();
    let session = sample_session(&graph);
    let path = dir.path().join("session.json");
    save_session_json(&path, &session).unwrap();
    let bytes = fs::read(&path).unwrap();
    let mut restored = JamAppState::from_json_files(&path, None::<&Path>).unwrap();
    restored.session.runtime_state.scene_state.source_bindings = Some(vec![SceneSourceBinding {
        scene_id: "scene-01-drop".into(),
        source_id: "other-source".into(),
        section_id: "section-a".into(),
    }]);
    let error = restored.save().unwrap_err().to_string();
    assert!(error.contains("invalid source binding"), "{error}");
    assert_eq!(fs::read(&path).unwrap(), bytes);
    save_session_json(&path, &restored.session).unwrap();
    let error = JamAppState::from_json_files(&path, None::<&Path>)
        .unwrap_err()
        .to_string();
    assert!(error.contains("invalid source binding"), "{error}");
}
