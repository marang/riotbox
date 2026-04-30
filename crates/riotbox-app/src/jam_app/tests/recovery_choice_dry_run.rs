#[test]
fn recovery_surface_dry_runs_manual_choice_without_selecting_or_mutating_files() {
    let dir = tempdir().expect("create temp dir");
    let target_path = dir.path().join("session.json");
    let autosave_path = dir.path().join("session.autosave.2026-04-30T164500Z.json");
    let captures_dir = dir.path().join("captures");

    fs::create_dir(&captures_dir).expect("create captures dir");
    fs::write(captures_dir.join("cap-01.wav"), [0u8; 44]).expect("write autosave artifact");
    save_session_json(
        &target_path,
        &SessionFile::new("canonical", "riotbox-test", "2026-04-30T16:45:00Z"),
    )
    .expect("save canonical session");

    let graph = sample_graph();
    let autosave_session = sample_session(&graph);
    save_session_json(&autosave_path, &autosave_session)
        .expect("save parseable autosave candidate");
    let target_before = fs::read(&target_path).expect("read target before dry-run");
    let autosave_before = fs::read(&autosave_path).expect("read autosave before dry-run");

    let surface =
        JamAppState::scan_session_recovery_surface(&target_path).expect("scan recovery surface");
    let dry_run = surface
        .dry_run_manual_choice(&autosave_path)
        .expect("autosave dry-run candidate");

    assert_eq!(surface.selected_candidate, None);
    assert!(!dry_run.selected_for_restore);
    assert_eq!(
        dry_run.safety_note,
        "Dry-run only: candidate inspected, not selected for restore."
    );
    assert_eq!(dry_run.candidate_path, autosave_path);
    assert_eq!(
        dry_run.decision_label,
        "decision: reviewable | full replay required"
    );
    assert_eq!(
        dry_run.artifact_availability_label,
        "artifacts ready: 1 capture(s)"
    );
    assert_eq!(
        dry_run.payload_readiness_label,
        "payload missing | snapshot restore blocked"
    );
    assert_eq!(dry_run.replay_suffix_label, "suffix none | target cursor 1");
    assert_eq!(dry_run.replay_unsupported_label, "unsupported none");
    assert_eq!(dry_run.trust, RecoveryCandidateTrust::RecoverableClue);
    assert_eq!(dry_run.action_hint, "review before manual recovery");
    assert_eq!(
        surface.dry_run_manual_choice(dir.path().join("does-not-exist.json")),
        None
    );

    assert_eq!(
        fs::read(&target_path).expect("read target after dry-run"),
        target_before
    );
    assert_eq!(
        fs::read(&autosave_path).expect("read autosave after dry-run"),
        autosave_before
    );
    assert_eq!(surface.selected_candidate, None);
}
