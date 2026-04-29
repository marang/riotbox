#[test]
fn rejects_session_with_multiple_source_refs_in_mvp_mode() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.source_refs.push(SourceRef {
        source_id: SourceId::from("src-2"),
        path_hint: "other.wav".into(),
        content_hash: "hash-2".into(),
        duration_seconds: 64.0,
        decode_profile: "normalized_stereo".into(),
    });
    save_session_json(&session_path, &session).expect("save multi-source session fixture");

    let error =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect_err("load should fail");

    match error {
        JamAppError::InvalidSession(message) => {
            assert!(message.contains("exactly one source reference"));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn rejects_session_with_mismatched_single_source_and_graph_refs() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.source_graph_refs[0].source_id = SourceId::from("src-other");
    save_session_json(&session_path, &session).expect("save mismatched session fixture");

    let error =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect_err("load should fail");

    match error {
        JamAppError::InvalidSession(message) => {
            assert!(message.contains("does not match source graph ref"));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn rejects_session_with_snapshot_cursor_beyond_action_log() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.snapshots[0].action_cursor = session.action_log.actions.len() + 1;
    save_session_json(&session_path, &session).expect("save bad snapshot cursor session");

    let error =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect_err("load should fail");

    match error {
        JamAppError::InvalidSession(message) => {
            assert!(message.contains("snapshot snap-1 action cursor 2"));
            assert!(message.contains("exceeds action log length 1"));
        }
        other => panic!("unexpected error: {other}"),
    }
}

fn sample_commit_record(action_id: ActionId, commit_sequence: u32) -> ActionCommitRecord {
    ActionCommitRecord {
        action_id,
        boundary: CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 8,
            bar_index: 2,
            phrase_index: 0,
            scene_id: Some(SceneId::from("scene-1")),
        },
        commit_sequence,
        committed_at: 200,
    }
}

#[test]
fn loads_session_with_commit_record_referencing_persisted_action() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session
        .action_log
        .commit_records
        .push(sample_commit_record(ActionId(1), 1));
    save_session_json(&session_path, &session).expect("save valid commit-record session");

    let restored =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect("load should pass");

    assert_eq!(restored.session.action_log.commit_records.len(), 1);
    assert_eq!(
        restored.runtime.last_commit_boundary,
        Some(CommitBoundaryState {
            kind: CommitBoundary::Bar,
            beat_index: 8,
            bar_index: 2,
            phrase_index: 0,
            scene_id: Some(SceneId::from("scene-1")),
        })
    );
}

#[test]
fn rejects_session_with_commit_record_for_missing_action() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session
        .action_log
        .commit_records
        .push(sample_commit_record(ActionId(999), 1));
    save_session_json(&session_path, &session).expect("save orphan commit-record session");

    let error =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect_err("load should fail");

    match error {
        JamAppError::InvalidSession(message) => {
            assert!(message.contains("commit record references missing action a-0999"));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn rejects_session_with_commit_record_for_uncommitted_action() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.action_log.actions[0].status = ActionStatus::Queued;
    session
        .action_log
        .commit_records
        .push(sample_commit_record(ActionId(1), 1));
    save_session_json(&session_path, &session)
        .expect("save non-committed commit-record session");

    let error =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect_err("load should fail");

    match error {
        JamAppError::InvalidSession(message) => {
            assert!(message.contains(
                "commit record references action a-0001 with non-committed status Queued"
            ));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn rejects_session_with_commit_record_for_action_without_committed_at() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session.action_log.actions[0].committed_at = None;
    session
        .action_log
        .commit_records
        .push(sample_commit_record(ActionId(1), 1));
    save_session_json(&session_path, &session)
        .expect("save missing action committed_at commit-record session");

    let error =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect_err("load should fail");

    match error {
        JamAppError::InvalidSession(message) => {
            assert!(
                message.contains("commit record references action a-0001 without committed_at")
            );
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn rejects_session_with_commit_record_committed_at_mismatch() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    let mut commit_record = sample_commit_record(ActionId(1), 1);
    commit_record.committed_at = 201;
    session.action_log.commit_records.push(commit_record);
    save_session_json(&session_path, &session)
        .expect("save mismatched committed_at commit-record session");

    let error =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect_err("load should fail");

    match error {
        JamAppError::InvalidSession(message) => {
            assert!(message.contains(
                "commit record for action a-0001 has committed_at 201 but action has committed_at 200"
            ));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn rejects_session_with_zero_commit_record_sequence() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session
        .action_log
        .commit_records
        .push(sample_commit_record(ActionId(1), 0));
    save_session_json(&session_path, &session).expect("save zero-sequence commit-record session");

    let error =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect_err("load should fail");

    match error {
        JamAppError::InvalidSession(message) => {
            assert!(message.contains("commit record for action a-0001 has invalid sequence 0"));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn rejects_session_with_duplicate_commit_record_sequence_for_boundary() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    let mut second_action = session.action_log.actions[0].clone();
    second_action.id = ActionId(2);
    session.action_log.actions.push(second_action);
    session
        .action_log
        .commit_records
        .push(sample_commit_record(ActionId(1), 1));
    session
        .action_log
        .commit_records
        .push(sample_commit_record(ActionId(2), 1));
    save_session_json(&session_path, &session)
        .expect("save duplicate-sequence commit-record session");

    let error =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect_err("load should fail");

    match error {
        JamAppError::InvalidSession(message) => {
            assert!(message.contains("commit record sequence 1 is duplicated"));
            assert!(message.contains("boundary Bar beat 8 bar 2 phrase 0"));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn rejects_session_with_duplicate_commit_record_for_same_action() {
    let dir = tempdir().expect("create temp dir");
    let session_path = dir.path().join("jam-session.json");
    let graph = sample_graph();
    let mut session = sample_session(&graph);
    session
        .action_log
        .commit_records
        .push(sample_commit_record(ActionId(1), 1));
    session
        .action_log
        .commit_records
        .push(sample_commit_record(ActionId(1), 2));
    save_session_json(&session_path, &session)
        .expect("save duplicate-action commit-record session");

    let error =
        JamAppState::from_json_files(&session_path, None::<&Path>).expect_err("load should fail");

    match error {
        JamAppError::InvalidSession(message) => {
            assert!(message.contains("commit record is duplicated for action a-0001"));
        }
        other => panic!("unexpected error: {other}"),
    }
}
