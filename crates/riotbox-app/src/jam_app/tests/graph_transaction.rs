use super::{
    GraphStorageMode, JamAppError, JamAppState, Path, SessionFile, SourceGraph, fs, io,
    load_session_json, load_source_graph_json, sample_graph, sample_session, save_session_json,
    save_source_graph_json, tempdir,
};
use crate::jam_app::persistence::{
    graph_transaction::{
        SaveCheckpoint, generation_path, save_graph_and_session, save_with_checkpoint,
    },
    source_graph_hash,
};

fn external_session(graph: &SourceGraph, alias: &Path) -> SessionFile {
    let mut session = sample_session(graph);
    let graph_ref = &mut session.source_graph_refs[0];
    graph_ref.storage_mode = GraphStorageMode::External;
    graph_ref.embedded_graph = None;
    graph_ref.external_path = Some(alias.to_string_lossy().into_owned());
    session
}

#[test]
fn each_transaction_interruption_preserves_legacy_session_and_exact_graph() {
    for failure in [
        SaveCheckpoint::PreviousGenerationReady,
        SaveCheckpoint::CurrentGenerationReady,
        SaveCheckpoint::AliasPublished,
    ] {
        let dir = tempdir().unwrap();
        let session_path = dir.path().join("session.json");
        let alias = dir.path().join("graph.json");
        let old_graph = sample_graph();
        let old_session = external_session(&old_graph, &alias);
        save_source_graph_json(&alias, &old_graph).unwrap();
        save_session_json(&session_path, &old_session).unwrap();
        let old_bytes = fs::read(&session_path).unwrap();
        let mut new_graph = old_graph.clone();
        new_graph.source.duration_seconds = 121.0;
        let new_session = external_session(&new_graph, &alias);

        let error = save_with_checkpoint(
            &session_path,
            &new_session,
            Some(&new_graph),
            Some(&alias),
            |at| {
                if at == failure {
                    Err(io::Error::other("injected process interruption").into())
                } else {
                    Ok(())
                }
            },
        );
        assert!(error.is_err());
        assert_eq!(fs::read(&session_path).unwrap(), old_bytes);
        for explicit in [None, Some(alias.as_path())] {
            let restored = JamAppState::from_json_files(&session_path, explicit).unwrap();
            assert_eq!(
                restored.source_graph,
                Some(old_graph.clone()),
                "{failure:?}"
            );
        }
        assert!(
            generation_path(&alias, &source_graph_hash(&old_graph).unwrap())
                .unwrap()
                .exists()
        );
    }
}

#[test]
fn first_ingest_interruption_never_publishes_session_before_graph() {
    for failure in [
        SaveCheckpoint::PreviousGenerationReady,
        SaveCheckpoint::CurrentGenerationReady,
        SaveCheckpoint::AliasPublished,
    ] {
        let dir = tempdir().unwrap();
        let session_path = dir.path().join("session.json");
        let alias = dir.path().join("graph.json");
        let graph = sample_graph();
        let session = external_session(&graph, &alias);
        assert!(
            save_with_checkpoint(&session_path, &session, Some(&graph), Some(&alias), |at| {
                if at == failure {
                    Err(io::Error::other("injected ingest interruption").into())
                } else {
                    Ok(())
                }
            })
            .is_err()
        );
        assert!(!session_path.exists());
        save_graph_and_session(&session_path, &session, Some(&graph), Some(&alias)).unwrap();
        assert_eq!(
            JamAppState::from_json_files(&session_path, None::<&Path>)
                .unwrap()
                .source_graph,
            Some(graph)
        );
    }
}

#[test]
fn public_save_keeps_old_generation_and_latest_explicit_alias() {
    let dir = tempdir().unwrap();
    let session_path = dir.path().join("session.json");
    let alias = dir.path().join("graph.json");
    let graph = sample_graph();
    save_source_graph_json(&alias, &graph).unwrap();
    save_session_json(&session_path, &external_session(&graph, &alias)).unwrap();
    let mut state = JamAppState::from_json_files(&session_path, Some(&alias)).unwrap();
    state.source_graph.as_mut().unwrap().source.duration_seconds = 121.0;
    state.session.notes = Some("new save".into());
    state.save().unwrap();
    state.save().unwrap();
    assert_eq!(
        load_source_graph_json(&alias).unwrap(),
        state.source_graph.clone().unwrap()
    );
    let old_path = generation_path(&alias, &source_graph_hash(&graph).unwrap()).unwrap();
    assert_eq!(load_source_graph_json(old_path).unwrap(), graph);
    assert_eq!(
        load_session_json(&session_path).unwrap().notes,
        state.session.notes
    );
    for explicit in [None, Some(alias.as_path())] {
        assert_eq!(
            JamAppState::from_json_files(&session_path, explicit)
                .unwrap()
                .source_graph,
            state.source_graph
        );
    }
}

#[test]
fn public_save_generation_failure_leaves_original_pair_untouched() {
    let dir = tempdir().unwrap();
    let session_path = dir.path().join("session.json");
    let alias = dir.path().join("graph.json");
    let graph = sample_graph();
    save_source_graph_json(&alias, &graph).unwrap();
    save_session_json(&session_path, &external_session(&graph, &alias)).unwrap();
    let old_session = fs::read(&session_path).unwrap();
    let old_alias = fs::read(&alias).unwrap();
    let generation = generation_path(&alias, &source_graph_hash(&graph).unwrap()).unwrap();
    fs::write(
        generation.parent().unwrap(),
        b"blocked generation directory",
    )
    .unwrap();
    let mut state = JamAppState::from_json_files(&session_path, Some(&alias)).unwrap();
    state.source_graph.as_mut().unwrap().source.duration_seconds = 121.0;
    assert!(state.save().is_err());
    assert_eq!(fs::read(&session_path).unwrap(), old_session);
    assert_eq!(fs::read(&alias).unwrap(), old_alias);
}

#[test]
fn missing_or_corrupt_alias_loads_only_exact_valid_generation() {
    let dir = tempdir().unwrap();
    let session_path = dir.path().join("session.json");
    let alias = dir.path().join("graph.json");
    let graph = sample_graph();
    let session = external_session(&graph, &alias);
    save_graph_and_session(&session_path, &session, Some(&graph), Some(&alias)).unwrap();
    fs::remove_file(&alias).unwrap();
    assert_eq!(
        JamAppState::from_json_files(&session_path, Some(&alias))
            .unwrap()
            .source_graph,
        Some(graph.clone())
    );
    fs::write(&alias, b"truncated alias").unwrap();
    assert!(JamAppState::from_json_files(&session_path, None::<&Path>).is_ok());
    let generation = generation_path(&alias, &source_graph_hash(&graph).unwrap()).unwrap();
    let mut wrong_graph = graph.clone();
    wrong_graph.source.duration_seconds = 121.0;
    save_source_graph_json(&generation, &wrong_graph).unwrap();
    let error = JamAppState::from_json_files(&session_path, None::<&Path>).unwrap_err();
    assert!(error.to_string().contains("hash mismatch"));
    fs::remove_file(&generation).unwrap();
    let unrelated = generation_path(&alias, &source_graph_hash(&wrong_graph).unwrap()).unwrap();
    save_source_graph_json(unrelated, &wrong_graph).unwrap();
    assert!(JamAppState::from_json_files(&session_path, None::<&Path>).is_err());
}

#[test]
fn session_only_save_rejects_dangling_graph_hash_before_session_publication() {
    let dir = tempdir().unwrap();
    let session_path = dir.path().join("session.json");
    let alias = dir.path().join("graph.json");
    let graph = sample_graph();
    save_source_graph_json(&alias, &graph).unwrap();
    save_session_json(&session_path, &external_session(&graph, &alias)).unwrap();
    let mut state = JamAppState::from_json_files(&session_path, Some(&alias)).unwrap();
    state.session.notes = Some("valid session-only save".into());
    state.save_session_without_source_graph_write().unwrap();
    let old_bytes = fs::read(&session_path).unwrap();
    state.source_graph.as_mut().unwrap().source.duration_seconds = 121.0;
    assert!(state.save_session_without_source_graph_write().is_err());
    assert_eq!(fs::read(&session_path).unwrap(), old_bytes);
    assert_eq!(load_source_graph_json(&alias).unwrap(), graph);
}

#[test]
fn generation_collision_fails_closed_without_replacing_evidence() {
    let dir = tempdir().unwrap();
    let session_path = dir.path().join("session.json");
    let alias = dir.path().join("graph.json");
    let graph = sample_graph();
    let session = external_session(&graph, &alias);
    let generation = generation_path(&alias, &source_graph_hash(&graph).unwrap()).unwrap();
    fs::create_dir_all(generation.parent().unwrap()).unwrap();
    fs::write(&generation, b"corrupt immutable evidence").unwrap();
    assert!(save_graph_and_session(&session_path, &session, Some(&graph), Some(&alias)).is_err());
    assert_eq!(fs::read(generation).unwrap(), b"corrupt immutable evidence");
    assert!(!session_path.exists());
    assert!(!alias.exists());
}

#[test]
fn session_graph_and_generation_path_collisions_fail_before_writes() {
    let dir = tempdir().unwrap();
    let alias = dir.path().join("graph.json");
    let graph = sample_graph();
    let session = external_session(&graph, &alias);
    for session_path in [
        alias.clone(),
        dir.path().join("nested/../graph.json"),
        generation_path(&alias, &source_graph_hash(&graph).unwrap()).unwrap(),
    ] {
        let error = save_graph_and_session(&session_path, &session, Some(&graph), Some(&alias))
            .unwrap_err();
        assert!(matches!(error, JamAppError::InvalidSession(_)));
        assert!(!alias.exists());
    }
}

#[test]
fn generation_namespace_cannot_be_used_as_mutable_alias_or_session_destination() {
    let dir = tempdir().unwrap();
    let alias = dir.path().join("graph.json");
    let graph = sample_graph();
    let session = external_session(&graph, &alias);
    let generation = generation_path(&alias, &source_graph_hash(&graph).unwrap()).unwrap();
    save_source_graph_json(&generation, &graph).unwrap();
    let before = fs::read(&generation).unwrap();
    assert!(
        save_graph_and_session(
            &dir.path().join("session.json"),
            &session,
            Some(&graph),
            Some(&generation)
        )
        .is_err()
    );
    assert!(save_graph_and_session(&generation, &session, None, None).is_err());
    assert_eq!(fs::read(&generation).unwrap(), before);
}

#[cfg(unix)]
#[test]
fn symlink_collisions_and_generation_redirects_fail_before_writes() {
    use std::os::unix::fs::symlink;
    let dir = tempdir().unwrap();
    let session_path = dir.path().join("session.json");
    let alias = dir.path().join("graph.json");
    let graph = sample_graph();
    let session = external_session(&graph, &alias);
    save_session_json(&session_path, &session).unwrap();
    let before = fs::read(&session_path).unwrap();
    symlink(&session_path, &alias).unwrap();
    assert!(save_graph_and_session(&session_path, &session, Some(&graph), Some(&alias)).is_err());
    assert_eq!(fs::read(&session_path).unwrap(), before);
    fs::remove_file(&alias).unwrap();
    let generation = generation_path(&alias, &source_graph_hash(&graph).unwrap()).unwrap();
    symlink(dir.path(), generation.parent().unwrap()).unwrap();
    assert!(save_graph_and_session(&session_path, &session, Some(&graph), Some(&alias)).is_err());
    assert_eq!(fs::read(&session_path).unwrap(), before);
    assert!(!alias.exists());
}
