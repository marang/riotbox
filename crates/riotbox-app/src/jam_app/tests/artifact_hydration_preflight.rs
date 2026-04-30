use super::capture_artifacts::CaptureArtifactHydrationPreflightError;

fn state_with_capture_artifact_path(
    dir: &Path,
    storage_path: &str,
) -> (JamAppState, CaptureRef, PathBuf) {
    let session_path = dir.join("session.json");
    let graph_path = dir.join("source_graph.json");
    let mut graph = sample_graph();
    graph.source.path = dir.join("source.wav").to_string_lossy().into_owned();

    let mut session = sample_session(&graph);
    session.captures.truncate(1);
    session.captures[0].storage_path = storage_path.into();
    let capture = session.captures[0].clone();

    save_source_graph_json(&graph_path, &graph).expect("save graph");
    save_session_json(&session_path, &session).expect("save session");

    let state =
        JamAppState::from_json_files(&session_path, Some(&graph_path)).expect("load app state");
    let expected_path = if Path::new(storage_path).is_absolute() {
        PathBuf::from(storage_path)
    } else {
        dir.join(storage_path)
    };

    (state, capture, expected_path)
}

#[test]
fn capture_artifact_hydration_preflight_rejects_missing_storage_identity() {
    let dir = tempdir().expect("create temp dir");
    let (state, capture, _) = state_with_capture_artifact_path(dir.path(), " ");

    let error = state
        .require_capture_artifact_for_hydration(&capture)
        .expect_err("missing storage path should reject");

    assert_eq!(
        error,
        CaptureArtifactHydrationPreflightError::MissingStoragePath {
            capture_id: CaptureId::from("cap-01"),
        }
    );
    assert!(state.capture_audio_cache.is_empty());
}

#[test]
fn capture_artifact_hydration_preflight_rejects_missing_artifact_file() {
    let dir = tempdir().expect("create temp dir");
    let (state, capture, expected_path) =
        state_with_capture_artifact_path(dir.path(), "captures/missing-cap.wav");

    let error = state
        .require_capture_artifact_for_hydration(&capture)
        .expect_err("missing artifact file should reject");

    assert_eq!(
        error,
        CaptureArtifactHydrationPreflightError::MissingArtifact {
            capture_id: CaptureId::from("cap-01"),
            path: expected_path,
        }
    );
    assert!(state.capture_audio_cache.is_empty());
}

#[test]
fn capture_artifact_hydration_preflight_accepts_existing_artifact_file() {
    let dir = tempdir().expect("create temp dir");
    let captures_dir = dir.path().join("captures");
    fs::create_dir(&captures_dir).expect("create captures dir");
    fs::write(captures_dir.join("cap-01.wav"), [0u8; 44]).expect("write artifact file");
    let (state, capture, expected_path) =
        state_with_capture_artifact_path(dir.path(), "captures/cap-01.wav");

    let path = state
        .require_capture_artifact_for_hydration(&capture)
        .expect("existing artifact file should pass preflight");

    assert_eq!(path, expected_path);
    assert!(state.capture_audio_cache.is_empty());
}
