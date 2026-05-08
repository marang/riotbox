#[test]
fn renders_more_musical_jam_shell_snapshot() {
    let shell = sample_shell_state();
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("trust usable"));
    assert!(rendered.contains("source timing low | policy"));
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
