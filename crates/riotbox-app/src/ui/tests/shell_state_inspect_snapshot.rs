#[test]
fn renders_jam_shell_inspect_snapshot() {
    let mut shell = sample_shell_state();
    shell.jam_mode = JamViewMode::Inspect;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Screen jam/inspect"), "{rendered}");
    assert!(rendered.contains("MC-202 detail"), "{rendered}");
    assert!(rendered.contains("W-30 detail"), "{rendered}");
    assert!(rendered.contains("TR-909 detail"), "{rendered}");
    assert!(rendered.contains("accent off"), "{rendered}");
    assert!(rendered.contains("Source structure"), "{rendered}");
    assert!(rendered.contains("Material flow"), "{rendered}");
    assert!(rendered.contains("Diagnostics"), "{rendered}");
    assert!(!rendered.contains("Suggested gestures"), "{rendered}");
}
