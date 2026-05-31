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
    assert!(rendered.contains("map time fallback"), "{rendered}");
    assert!(
        rendered.contains("source clock beat - | bar 8 | phrase -"),
        "{rendered}"
    );
    assert!(rendered.contains("timing needs confirm | confirm grid"), "{rendered}");
    assert!(rendered.contains("p0:b0/1/0"), "{rendered}");
    assert!(rendered.contains("timing warning ambiguous_downbeat"), "{rendered}");
    assert!(rendered.contains("Material flow"), "{rendered}");
    assert!(rendered.contains("Diagnostics"), "{rendered}");
    assert!(rendered.contains("scene contract"), "{rendered}");
    assert!(rendered.contains("needs_timing_confirmation"), "{rendered}");
    assert!(rendered.contains("truth product spine"), "{rendered}");
    assert!(rendered.contains("timing needs_user_confirmation"), "{rendered}");
    assert!(rendered.contains("proof p012/p013/replay/output yes"), "{rendered}");
    assert!(rendered.contains("export full_grid_mix | feral-grid"), "{rendered}");
    assert!(!rendered.contains("Suggested gestures"), "{rendered}");
}
