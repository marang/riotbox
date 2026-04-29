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
                candidate.trust,
                candidate.action_hint,
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                "normal session path",
                "parseable session JSON",
                RecoveryCandidateTrust::NormalLoadTarget,
                "load normally",
            ),
            (
                "orphan temp file",
                "invalid session JSON",
                RecoveryCandidateTrust::BrokenClue,
                "do not recover automatically",
            ),
            (
                "autosave file",
                "parseable session JSON",
                RecoveryCandidateTrust::RecoverableClue,
                "review before manual recovery",
            ),
            (
                "autosave file",
                "invalid session JSON",
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
