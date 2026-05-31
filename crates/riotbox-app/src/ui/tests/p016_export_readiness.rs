#[test]
fn jam_inspect_surfaces_export_readiness_without_export_action() {
    let mut shell = sample_shell_state();
    shell.jam_mode = JamViewMode::Inspect;

    let inspect = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(inspect.contains("export full_grid_mix | feral-grid"), "{inspect}");
    assert!(
        inspect.contains("reproducible | no stem/live/DAW/host"),
        "{inspect}"
    );
    assert!(!inspect.contains("queue export"), "{inspect}");
}

#[test]
fn jam_perform_does_not_claim_export_readiness_as_a_play_control() {
    let shell = sample_shell_state();

    let perform = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(!perform.contains("export full_grid_mix"), "{perform}");
    assert!(!perform.contains("queue export"), "{perform}");
}
