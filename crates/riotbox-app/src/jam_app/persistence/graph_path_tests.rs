use std::{fs, path::Path, process::Command};

use riotbox_core::persistence::{load_session_json, save_session_json, save_source_graph_json};

use super::{JamAppState, session_from_ingested_graph};
use crate::jam_app::tests::sample_graph;

const CHILD_SESSION: &str = "RIOTBOX_1488_CHILD_SESSION";

#[test]
fn public_ingest_with_relative_destinations_restores_from_session_alone() {
    let cwd = std::env::current_dir().unwrap();
    let dir = tempfile::tempdir_in(&cwd).unwrap();
    let source = dir.path().join("synthetic.wav");
    let samples: Vec<f32> = (0..44_100)
        .map(|frame| (frame as f32 * 440.0 * std::f32::consts::TAU / 44_100.0).sin() * 0.2)
        .collect();
    riotbox_audio::source_audio::write_interleaved_pcm16_wav(&source, 44_100, 1, &samples).unwrap();
    let session = dir.path().join("sessions/jam.json");
    let graph = dir.path().join("sessions/graph.json");
    let sidecar =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../python/sidecar/json_stdio_sidecar.py");
    let state = JamAppState::analyze_source_file_to_json(
        &source,
        session.strip_prefix(&cwd).unwrap(),
        Some(graph.strip_prefix(&cwd).unwrap().to_path_buf()),
        sidecar,
        29,
    )
    .unwrap();
    assert_eq!(
        load_session_json(&session).unwrap().source_graph_refs[0]
            .external_path
            .as_deref(),
        Some("graph.json")
    );
    assert_eq!(
        JamAppState::from_json_files(&session, None::<&Path>)
            .unwrap()
            .source_graph,
        state.source_graph
    );
}

#[test]
fn relative_ingest_refs_restore_without_override_from_another_cwd() {
    if let Some(session) = std::env::var_os(CHILD_SESSION) {
        let mut state = JamAppState::from_json_files(session, None::<&Path>).unwrap();
        state.session.notes = Some("restored from another cwd".into());
        state.save().unwrap();
        return;
    }
    let cwd = std::env::current_dir().unwrap();
    let dir = tempfile::tempdir_in(&cwd).unwrap();
    for graph_folder in ["sessions", "graphs"] {
        let session_path = dir.path().join("sessions/jam.json");
        let graph_path = dir.path().join(graph_folder).join("source-graph.json");
        let relative_session = session_path.strip_prefix(&cwd).unwrap();
        let relative_graph = graph_path.strip_prefix(&cwd).unwrap();
        let mut graph = sample_graph();
        graph.source.path = dir.path().join("absent.wav").to_string_lossy().into_owned();
        let session = session_from_ingested_graph(
            &graph,
            Path::new(&graph.source.path),
            relative_session,
            Some(relative_graph),
        )
        .unwrap();
        assert_eq!(
            session.source_graph_refs[0].external_path.as_deref(),
            Some(if graph_folder == "sessions" {
                "source-graph.json"
            } else {
                "../graphs/source-graph.json"
            })
        );
        save_source_graph_json(relative_graph, &graph).unwrap();
        save_session_json(relative_session, &session).unwrap();
        let mut state =
            JamAppState::from_json_files(relative_session, Some(relative_graph)).unwrap();
        assert!(state.files.as_ref().unwrap().session_path.is_absolute());
        assert!(
            state
                .files
                .as_ref()
                .unwrap()
                .source_graph_path
                .as_ref()
                .unwrap()
                .is_absolute()
        );
        state.source_graph.as_mut().unwrap().source.duration_seconds += 1.0;
        state.save().unwrap();
        assert_eq!(
            JamAppState::from_json_files(&session_path, None::<&Path>)
                .unwrap()
                .source_graph,
            state.source_graph
        );
        let other = tempfile::tempdir().unwrap();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "jam_app::persistence::graph_path_tests::relative_ingest_refs_restore_without_override_from_another_cwd"])
            .env(CHILD_SESSION, &session_path)
            .current_dir(other.path()).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            load_session_json(&session_path).unwrap().notes.as_deref(),
            Some("restored from another cwd")
        );
    }
}

#[test]
fn ambiguous_legacy_reference_requires_explicit_hash_checked_override_and_resave() {
    let cwd = std::env::current_dir().unwrap();
    let dir = tempfile::tempdir_in(&cwd).unwrap();
    let session_path = dir.path().join("sessions/jam.json");
    let graph_path = dir.path().join("sessions/graph.json");
    let relative_graph = graph_path.strip_prefix(&cwd).unwrap();
    let mut graph = sample_graph();
    graph.source.path = dir.path().join("absent.wav").to_string_lossy().into_owned();
    let mut session = session_from_ingested_graph(
        &graph,
        Path::new(&graph.source.path),
        &session_path,
        Some(&graph_path),
    )
    .unwrap();
    session.source_graph_refs[0].external_path =
        Some(relative_graph.to_string_lossy().into_owned());
    save_source_graph_json(&graph_path, &graph).unwrap();
    save_session_json(&session_path, &session).unwrap();
    assert!(JamAppState::from_json_files(&session_path, None::<&Path>).is_err());
    let repaired = JamAppState::from_json_files(&session_path, Some(relative_graph)).unwrap();
    repaired.save().unwrap();
    assert_eq!(
        load_session_json(&session_path).unwrap().source_graph_refs[0]
            .external_path
            .as_deref(),
        Some("graph.json")
    );
    JamAppState::from_json_files(&session_path, None::<&Path>).unwrap();
    let wrong = dir.path().join("wrong.json");
    graph.source.duration_seconds += 1.0;
    save_source_graph_json(&wrong, &graph).unwrap();
    let before = fs::read(&session_path).unwrap();
    assert!(JamAppState::from_json_files(&session_path, Some(&wrong)).is_err());
    assert_eq!(before, fs::read(&session_path).unwrap());
}
