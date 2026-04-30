#[test]
fn recovery_surface_lists_candidates_without_selecting_or_mutating_files() {
    let dir = tempdir().expect("create temp dir");
    let target_path = dir.path().join("session.json");
    let temp_path = dir.path().join(".session.json.tmp-42-100");
    let autosave_path = dir.path().join("session.autosave.2026-04-29T205800Z.json");
    let invalid_autosave_path = dir.path().join("session.autosave.json");

    save_session_json(
        &target_path,
        &SessionFile::new("canonical", "riotbox-test", "2026-04-29T20:58:00Z"),
    )
    .expect("save canonical session");
    fs::write(&temp_path, "{ broken").expect("write invalid temp");
    save_session_json(
        &autosave_path,
        &SessionFile::new("autosave", "riotbox-test", "2026-04-29T20:58:01Z"),
    )
    .expect("save autosave session");
    fs::write(&invalid_autosave_path, "not json").expect("write invalid autosave");

    let surface =
        JamAppState::scan_session_recovery_surface(&target_path).expect("scan recovery surface");

    assert_eq!(surface.target_path, target_path);
    assert_eq!(surface.selected_candidate, None);
    assert_eq!(
        surface.safety_note,
        "Manual recovery only: Riotbox did not choose, load, replace, or delete any candidate."
    );
    assert_eq!(
        surface.headline,
        "1 manual recovery candidate(s) need explicit review"
    );
    assert!(surface.has_manual_candidates());
    assert_eq!(surface.candidates.len(), 4);
    assert_eq!(
        surface
            .candidates
            .iter()
            .map(|candidate| (
                candidate.kind_label,
                candidate.status_label,
                candidate.artifact_availability_label.as_str(),
                candidate.replay_readiness_label.as_str(),
                candidate.payload_readiness_label.as_str(),
                candidate.replay_suffix_label.as_str(),
                candidate.replay_unsupported_label.as_str(),
                candidate.trust,
                candidate.action_hint,
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                "normal session path",
                "parseable session JSON",
                "artifacts n/a | no captures",
                "ready: no replay entries",
                "payload none | full replay",
                "suffix none | target cursor 0",
                "unsupported none",
                RecoveryCandidateTrust::NormalLoadTarget,
                "load normally",
            ),
            (
                "orphan temp file",
                "invalid session JSON",
                "artifacts unchecked",
                "replay unchecked",
                "payload unchecked",
                "suffix unchecked",
                "unsupported unchecked",
                RecoveryCandidateTrust::BrokenClue,
                "do not recover automatically",
            ),
            (
                "autosave file",
                "parseable session JSON",
                "artifacts n/a | no captures",
                "ready: no replay entries",
                "payload none | full replay",
                "suffix none | target cursor 0",
                "unsupported none",
                RecoveryCandidateTrust::RecoverableClue,
                "review before manual recovery",
            ),
            (
                "autosave file",
                "invalid session JSON",
                "artifacts unchecked",
                "replay unchecked",
                "payload unchecked",
                "suffix unchecked",
                "unsupported unchecked",
                RecoveryCandidateTrust::BrokenClue,
                "do not recover automatically",
            ),
        ]
    );
    assert!(
        surface.candidates[2]
            .detail
            .contains("remains untrusted until the user explicitly chooses recovery")
    );
    assert!(temp_path.exists());
    assert!(autosave_path.exists());
    assert!(invalid_autosave_path.exists());
}

#[test]
fn recovery_surface_reports_capture_artifact_availability_for_parseable_candidates() {
    let dir = tempdir().expect("create temp dir");
    let target_path = dir.path().join("session.json");
    let ready_path = dir.path().join("session.autosave.ready-artifact.json");
    let identity_path = dir.path().join("session.autosave.identity-artifact.json");
    let missing_path = dir.path().join("session.autosave.missing-artifact.json");
    let captures_dir = dir.path().join("captures");

    fs::create_dir(&captures_dir).expect("create captures dir");
    fs::write(captures_dir.join("cap-01.wav"), [0u8; 44]).expect("write ready artifact");
    save_session_json(
        &target_path,
        &SessionFile::new("canonical", "riotbox-test", "2026-04-30T08:50:00Z"),
    )
    .expect("save canonical session");

    let graph = sample_graph();
    let mut ready_session = sample_session(&graph);
    ready_session.captures.truncate(1);
    save_session_json(&ready_path, &ready_session).expect("save ready autosave session");

    let mut missing_session = ready_session.clone();
    missing_session.captures[0].storage_path = "captures/missing-cap.wav".into();
    save_session_json(&missing_path, &missing_session).expect("save missing autosave session");

    let mut identity_session = ready_session.clone();
    identity_session.captures[0].storage_path = " ".into();
    save_session_json(&identity_path, &identity_session).expect("save missing-identity session");

    let surface =
        JamAppState::scan_session_recovery_surface(&target_path).expect("scan recovery surface");
    let artifact_labels = surface
        .candidates
        .iter()
        .filter(|candidate| matches!(candidate.trust, RecoveryCandidateTrust::RecoverableClue))
        .map(|candidate| candidate.artifact_availability_label.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        artifact_labels,
        vec![
            "artifacts blocked: 1 of 1 | 1 missing identity",
            "artifacts blocked: 1 of 1 | 1 missing",
            "artifacts ready: 1 capture(s)",
        ]
    );
    assert_eq!(surface.selected_candidate, None);
}

#[test]
fn recovery_surface_reports_missing_target_without_selecting_candidate() {
    let dir = tempdir().expect("create temp dir");
    let target_path = dir.path().join("missing").join("session.json");

    let surface =
        JamAppState::scan_session_recovery_surface(&target_path).expect("scan recovery surface");

    assert_eq!(surface.selected_candidate, None);
    assert!(!surface.has_manual_candidates());
    assert_eq!(surface.headline, "No manual recovery candidate selected");
    assert_eq!(surface.candidates.len(), 1);
    assert_eq!(surface.candidates[0].kind_label, "normal session path");
    assert_eq!(surface.candidates[0].status_label, "missing");
    assert_eq!(
        surface.candidates[0].trust,
        RecoveryCandidateTrust::MissingTarget
    );
    assert_eq!(
        surface.candidates[0].action_hint,
        "normal load cannot start from this path"
    );
}

#[test]
fn recovery_surface_reports_snapshot_payload_readiness_for_parseable_candidates() {
    let dir = tempdir().expect("create temp dir");
    let target_path = dir.path().join("session.json");
    let missing_payload_path = dir.path().join("session.autosave.missing.json");
    let ready_payload_path = dir.path().join("session.autosave.ready.json");

    save_session_json(
        &target_path,
        &SessionFile::new("canonical", "riotbox-test", "2026-04-30T08:15:00Z"),
    )
    .expect("save canonical session");

    let graph = sample_graph();
    let missing_payload_session = sample_session(&graph);
    save_session_json(&missing_payload_path, &missing_payload_session)
        .expect("save missing-payload autosave session");

    let mut ready_payload_session = sample_session(&graph);
    ready_payload_session.snapshots[0].payload =
        Some(riotbox_core::session::SnapshotPayload::from_runtime_state(
            &ready_payload_session.snapshots[0].snapshot_id,
            ready_payload_session.snapshots[0].action_cursor,
            &ready_payload_session.runtime_state,
        ));
    save_session_json(&ready_payload_path, &ready_payload_session)
        .expect("save ready-payload autosave session");

    let surface =
        JamAppState::scan_session_recovery_surface(&target_path).expect("scan recovery surface");
    let payload_labels = surface
        .candidates
        .iter()
        .filter(|candidate| matches!(candidate.trust, RecoveryCandidateTrust::RecoverableClue))
        .map(|candidate| candidate.payload_readiness_label.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        payload_labels,
        vec![
            "payload missing | snapshot restore blocked",
            "payload ready | snapshot restore ok",
        ]
    );
    assert_eq!(surface.selected_candidate, None);
}

#[test]
fn recovery_surface_reports_blocked_replay_status_for_parseable_candidates() {
    let dir = tempdir().expect("create temp dir");
    let target_path = dir.path().join("session.json");
    let blocked_autosave_path = dir.path().join("session.autosave.blocked.json");

    save_session_json(
        &target_path,
        &SessionFile::new("canonical", "riotbox-test", "2026-04-30T08:30:00Z"),
    )
    .expect("save canonical session");

    let graph = sample_graph();
    let mut blocked_session = sample_session(&graph);
    let snapshot = blocked_session.snapshots[0].clone();
    blocked_session.snapshots[0].payload =
        Some(riotbox_core::session::SnapshotPayload::from_runtime_state(
            &snapshot.snapshot_id,
            snapshot.action_cursor,
            &blocked_session.runtime_state,
        ));
    blocked_session
        .action_log
        .actions
        .push(unsupported_capture_bar_group_action(88));
    blocked_session
        .action_log
        .commit_records
        .push(loop_freeze_commit_record(88));
    save_session_json(&blocked_autosave_path, &blocked_session)
        .expect("save blocked autosave session");

    let surface =
        JamAppState::scan_session_recovery_surface(&target_path).expect("scan recovery surface");
    let blocked_candidate = surface
        .candidates
        .iter()
        .find(|candidate| matches!(candidate.trust, RecoveryCandidateTrust::RecoverableClue))
        .expect("blocked autosave candidate");

    assert_eq!(
        blocked_candidate.replay_readiness_label,
        "blocked: 1 unsupported suffix action(s)"
    );
    assert_eq!(
        blocked_candidate.payload_readiness_label,
        "payload ready | snapshot restore ok"
    );
    assert_eq!(
        blocked_candidate.replay_suffix_label,
        "suffix 1 action(s): capture.bar_group"
    );
    assert_eq!(
        blocked_candidate.replay_unsupported_label,
        "unsupported suffix 1: capture.bar_group"
    );
    assert_eq!(blocked_candidate.guidance, None);
    assert_eq!(surface.selected_candidate, None);
}

#[test]
fn recovery_surface_projects_artifact_ready_replay_blocker_guidance() {
    let dir = tempdir().expect("create temp dir");
    let target_path = dir.path().join("session.json");
    let blocked_autosave_path = dir.path().join("session.autosave.artifact-ready-blocked.json");

    save_session_json(
        &target_path,
        &SessionFile::new("canonical", "riotbox-test", "2026-04-30T09:38:00Z"),
    )
    .expect("save canonical session");

    let graph = sample_graph();
    let mut blocked_session = sample_session(&graph);
    let snapshot = blocked_session.snapshots[0].clone();
    blocked_session.snapshots[0].payload =
        Some(riotbox_core::session::SnapshotPayload::from_runtime_state(
            &snapshot.snapshot_id,
            snapshot.action_cursor,
            &blocked_session.runtime_state,
        ));
    blocked_session
        .action_log
        .actions
        .push(unsupported_capture_bar_group_action(89));
    blocked_session
        .action_log
        .commit_records
        .push(loop_freeze_commit_record(89));
    save_session_json(&blocked_autosave_path, &blocked_session)
        .expect("save blocked autosave session");
    let captures_dir = dir.path().join("captures");
    fs::create_dir_all(&captures_dir).expect("create capture artifacts dir");
    fs::write(captures_dir.join("cap-01.wav"), [0u8; 44])
        .expect("write ready capture artifact");

    let surface =
        JamAppState::scan_session_recovery_surface(&target_path).expect("scan recovery surface");
    let blocked_candidate = surface
        .candidates
        .iter()
        .find(|candidate| matches!(candidate.trust, RecoveryCandidateTrust::RecoverableClue))
        .expect("blocked autosave candidate");

    assert_eq!(
        blocked_candidate.artifact_availability_label,
        "artifacts ready: 1 capture(s)"
    );
    assert_eq!(
        blocked_candidate.replay_unsupported_label,
        "unsupported suffix 1: capture.bar_group"
    );
    assert_eq!(
        blocked_candidate.guidance,
        Some(RecoveryCandidateGuidance::ArtifactReadyReplayHydrationBlocked)
    );
    assert_eq!(surface.selected_candidate, None);
}
