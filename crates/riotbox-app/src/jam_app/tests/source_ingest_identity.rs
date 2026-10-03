use std::{fs, path::Path};

use crate::jam_app::tests::fixtures::source_io::{sidecar_script_path, write_pcm16_wave};
use crate::jam_app::{JamAppError, JamAppState};

#[test]
fn source_replacement_after_sidecar_analysis_preserves_saved_session_and_graph() {
    assert_replacement_preserves_saved_state(true);
}

#[test]
fn source_replacement_after_sidecar_analysis_preserves_embedded_graph() {
    assert_replacement_preserves_saved_state(false);
}

fn assert_replacement_preserves_saved_state(external_graph: bool) {
    let dir = tempfile::tempdir().expect("temporary fixture directory");
    let source_path = dir.path().join("source.wav");
    let session_path = dir.path().join("session.json");
    let graph_path = dir.path().join("source-graph.json");
    let external_graph_path = external_graph.then(|| graph_path.clone());
    write_pcm16_wave(&source_path, 8_000, 1, 0.1);
    let original_source = fs::read(&source_path).unwrap();
    let original = JamAppState::analyze_source_file_to_json(
        &source_path,
        &session_path,
        external_graph_path.clone(),
        sidecar_script_path(),
        17,
    )
    .expect("initial valid ingest");
    assert!(original.source_audio_cache.is_some());
    let saved_session = fs::read(&session_path).unwrap();
    let saved_graph = external_graph.then(|| fs::read(&graph_path).unwrap());

    // Change only PCM payload, keeping a valid file with identical format and length.
    let mut replacement = original_source.clone();
    *replacement.last_mut().unwrap() ^= 1;
    fs::write(source_path.with_extension("replacement.wav"), &replacement).unwrap();
    let wrapper = dir.path().join("replacing_sidecar.py");
    write_replacing_sidecar(&wrapper);
    let result = JamAppState::analyze_source_file_to_json(
        &source_path,
        &session_path,
        external_graph_path,
        &wrapper,
        23,
    );

    assert_eq!(
        fs::read(&source_path).unwrap(),
        replacement,
        "replacement ran"
    );
    assert!(
        matches!(result, Err(JamAppError::InvalidSession(ref reason))
        if reason.contains("source audio hash mismatch during ingest")),
        "changed source must fail at ingest admission"
    );
    assert_eq!(fs::read(&session_path).unwrap(), saved_session);
    if let Some(saved_graph) = saved_graph {
        assert_eq!(fs::read(&graph_path).unwrap(), saved_graph);
    } else {
        assert!(!graph_path.exists());
    }

    // The external replacement is independent of persistence. Restoring A must
    // recover the complete old state, without mixed analysis left on disk.
    fs::write(&source_path, original_source).unwrap();
    let reloaded = JamAppState::from_json_files(&session_path, None::<&Path>)
        .expect("previous saved state remains loadable");
    assert_eq!(reloaded.session, original.session);
    assert_eq!(reloaded.source_graph, original.source_graph);
    assert_eq!(reloaded.source_audio_cache, original.source_audio_cache);
}

#[test]
fn source_replacement_after_sidecar_analysis_creates_no_new_saved_state() {
    for external_graph in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("source.wav");
        let session_path = dir.path().join("sessions/session.json");
        let graph_path = dir.path().join("graphs/source-graph.json");
        write_pcm16_wave(&source_path, 8_000, 1, 0.1);
        write_pcm16_wave(
            source_path.with_extension("replacement.wav"),
            16_000,
            2,
            0.2,
        );
        let wrapper = dir.path().join("replacing_sidecar.py");
        write_replacing_sidecar(&wrapper);

        let result = JamAppState::analyze_source_file_to_json(
            &source_path,
            &session_path,
            external_graph.then(|| graph_path.clone()),
            &wrapper,
            29,
        );
        assert!(
            matches!(result, Err(JamAppError::InvalidSession(ref reason))
            if reason.contains("source audio hash mismatch during ingest"))
        );
        assert!(!session_path.parent().unwrap().exists());
        assert!(!graph_path.parent().unwrap().exists());
    }
}

fn write_replacing_sidecar(path: &Path) {
    let sidecar_dir = serde_json::to_string(sidecar_script_path().parent().unwrap()).unwrap();
    // Exercise the real protocol/provider; the only injected behavior is an
    // atomic filesystem replacement after its complete analysis and before reply.
    let script = format!(
        r#"import os
import sys
from pathlib import Path

sys.path.insert(0, {sidecar_dir})
import json_stdio_sidecar as sidecar

analyze = sidecar.build_graph_from_decoded_wave

def replace_after_analysis(source_path, analysis_seed):
    graph = analyze(source_path, analysis_seed)
    os.replace(Path(source_path).with_suffix('.replacement.wav'), source_path)
    return graph

sidecar.build_graph_from_decoded_wave = replace_after_analysis
sidecar.main()
"#
    );
    fs::write(path, script).expect("write test sidecar wrapper");
}
