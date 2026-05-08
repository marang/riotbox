#[test]
fn renders_more_musical_jam_shell_snapshot() {
    let shell = sample_shell_state();
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("trust usable"));
    assert!(rendered.contains("source timing needs confirm | quality"));
    assert!(rendered.contains("low | policy manual_confirm"));
    assert!(rendered.contains("manual_confirm"));
    assert!(rendered.contains("timing warning ambiguous_downbeat"));
    assert!(rendered.contains("scene scene-a | energy med"));
    assert!(rendered.contains("ghost"));
    assert!(rendered.contains("warnings"));
    assert!(rendered.contains("MC-202"));
    assert!(rendered.contains("W-30"));
    assert!(rendered.contains("TR-909"));
    assert!(rendered.contains("Suggested gestures"));
    assert!(rendered.contains("Pending / landed"));
    assert!(rendered.contains("next fill"));
    assert!(rendered.contains("wait [===>] next bar"), "{rendered}");
    assert!(
        rendered.contains("Primary: y scene jump | g follow | f fill"),
        "{rendered}"
    );
    assert!(
        rendered.contains("Advanced: Y restore | a answer | b voice | P pressure | I instigate"),
        "{rendered}"
    );
    assert!(!rendered.contains("Sections"), "{rendered}");
}

#[test]
fn renders_locked_source_timing_as_grid_locked_cue() {
    let mut shell = sample_shell_state();
    let graph = shell
        .app
        .source_graph
        .as_mut()
        .expect("sample shell should include source graph");
    graph.timing.quality = TimingQuality::High;
    graph.timing.degraded_policy = TimingDegradedPolicy::Locked;
    graph.timing.warnings.clear();

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("source timing grid locked | quality"),
        "{rendered}"
    );
    assert!(
        rendered.contains("high | policy locked"),
        "{rendered}"
    );
    assert!(rendered.contains("timing warning none"), "{rendered}");
}
