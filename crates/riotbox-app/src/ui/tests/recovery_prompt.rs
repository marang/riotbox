use riotbox_core::persistence::save_session_json;
use tempfile::tempdir;

#[test]
fn renders_manual_recovery_prompt_in_warnings_and_help() {
    let dir = tempdir().expect("create temp dir");
    let target_path = dir.path().join("session.json");
    let autosave_path = dir.path().join("session.autosave.2026-04-29T211500Z.json");

    save_session_json(
        &target_path,
        &SessionFile::new("canonical", "riotbox-test", "2026-04-29T21:15:00Z"),
    )
    .expect("save canonical session");
    save_session_json(
        &autosave_path,
        &SessionFile::new("autosave", "riotbox-test", "2026-04-29T21:15:01Z"),
    )
    .expect("save autosave session");

    let mut shell = sample_shell_state();
    shell.set_recovery_surface(
        JamAppState::scan_session_recovery_surface(&target_path)
            .expect("scan recovery surface"),
    );

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("recovery: 1 manual recovery"),
        "{rendered}"
    );
    assert!(
        rendered.contains("candidate(s) need explicit review"),
        "{rendered}"
    );
    assert!(rendered.contains("manual review only"), "{rendered}");
    assert!(!rendered.contains("Warnings clear"), "{rendered}");

    shell.show_help = true;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Session recovery"), "{rendered}");
    assert!(
        rendered.contains("Manual recovery only: Riotbox did not choose, load, replace, or delete"),
        "{rendered}"
    );
    assert!(
        rendered.contains("No candidate is selected here; reload an explicit reviewed path"),
        "{rendered}"
    );
    assert!(
        rendered.contains("autosave file | parseable session JSON | review before manual recovery"),
        "{rendered}"
    );
    assert!(
        rendered.contains("session.autosave.2026-04-29T211500Z.json"),
        "{rendered}"
    );
}
