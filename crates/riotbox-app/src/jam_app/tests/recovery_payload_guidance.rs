#[test]
fn recovery_surface_reports_missing_snapshot_payload_guidance_without_mutating_candidate() {
    let dir = tempdir().expect("create temp dir");
    let target_path = dir.path().join("session.json");
    let missing_payload_path = dir.path().join("session.autosave.missing-payload.json");
    let captures_dir = dir.path().join("captures");

    fs::create_dir(&captures_dir).expect("create captures dir");
    fs::write(captures_dir.join("cap-01.wav"), [0u8; 44]).expect("write ready artifact");
    save_session_json(
        &target_path,
        &SessionFile::new("canonical", "riotbox-test", "2026-04-30T12:35:00Z"),
    )
    .expect("save canonical session");

    let graph = sample_graph();
    let missing_payload_session = sample_session(&graph);
    save_session_json(&missing_payload_path, &missing_payload_session)
        .expect("save missing-payload autosave session");
    let autosave_before =
        fs::read(&missing_payload_path).expect("read missing-payload autosave before scan");

    let surface =
        JamAppState::scan_session_recovery_surface(&target_path).expect("scan recovery surface");
    let candidate = surface
        .candidates
        .iter()
        .find(|candidate| matches!(candidate.trust, RecoveryCandidateTrust::RecoverableClue))
        .expect("missing-payload autosave candidate");

    assert_eq!(candidate.artifact_availability_label, "artifacts ready: 1 capture(s)");
    assert_eq!(
        candidate.payload_readiness_label,
        "payload missing | snapshot restore blocked"
    );
    assert_eq!(
        candidate.decision_label,
        "decision: reviewable | full replay required"
    );
    assert_eq!(
        candidate.guidance,
        Some(RecoveryCandidateGuidance::MissingSnapshotPayload {
            detail: "snapshot snap-1 at cursor 1 has no payload; recovery would need full replay to target cursor 1".into(),
        })
    );
    assert_eq!(surface.selected_candidate, None);
    assert_eq!(
        fs::read(&missing_payload_path).expect("read missing-payload autosave after scan"),
        autosave_before
    );
}

#[test]
fn recovery_surface_classifies_invalid_snapshot_payload_candidate_as_broken_clue() {
    let dir = tempdir().expect("create temp dir");
    let target_path = dir.path().join("session.json");
    let invalid_payload_path = dir.path().join("session.autosave.invalid-payload.json");

    save_session_json(
        &target_path,
        &SessionFile::new("canonical", "riotbox-test", "2026-04-30T12:40:00Z"),
    )
    .expect("save canonical session");

    let graph = sample_graph();
    let mut invalid_payload_session = sample_session(&graph);
    invalid_payload_session.snapshots[0].payload = Some(riotbox_core::session::SnapshotPayload {
        payload_version: riotbox_core::session::SnapshotPayloadVersion::V1,
        snapshot_id: SnapshotId::from("snap-other"),
        action_cursor: invalid_payload_session.snapshots[0].action_cursor,
        runtime_state: invalid_payload_session.runtime_state.clone(),
    });
    save_session_json(&invalid_payload_path, &invalid_payload_session)
        .expect("save invalid-payload autosave session");
    let autosave_before =
        fs::read(&invalid_payload_path).expect("read invalid-payload autosave before scan");

    let surface =
        JamAppState::scan_session_recovery_surface(&target_path).expect("scan recovery surface");
    let candidate = surface
        .candidates
        .iter()
        .find(|candidate| candidate.path == invalid_payload_path)
        .expect("invalid-payload autosave candidate");

    assert_eq!(surface.selected_candidate, None);
    assert!(!surface.has_manual_candidates());
    assert_eq!(candidate.kind_label, "autosave file");
    assert_eq!(candidate.status_label, "app-invalid session");
    assert_eq!(candidate.artifact_availability_label, "artifacts unchecked");
    assert_eq!(
        candidate.payload_readiness_label,
        "payload invalid | snapshot restore blocked"
    );
    assert_eq!(candidate.decision_label, "decision: broken candidate");
    assert_eq!(candidate.trust, RecoveryCandidateTrust::BrokenClue);
    assert_eq!(candidate.guidance, None);
    assert_eq!(
        candidate.detail,
        "Snapshot payload identity is invalid; normal app load rejects this candidate."
    );
    assert_eq!(
        fs::read(&invalid_payload_path).expect("read invalid-payload autosave after scan"),
        autosave_before
    );
}
