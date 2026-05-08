#[test]
fn renders_more_musical_jam_shell_snapshot() {
    let shell = sample_shell_state();
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("trust usable"));
    assert!(rendered.contains("idle @ 32.0 | source b32 bar8 p1"));
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

#[test]
fn source_timing_readiness_styles_locked_cue_as_confirmed() {
    let mut shell = sample_shell_state();
    let graph = shell
        .app
        .source_graph
        .as_mut()
        .expect("sample shell should include source graph");
    graph.timing.quality = TimingQuality::High;
    graph.timing.degraded_policy = TimingDegradedPolicy::Locked;
    graph.timing.warnings.clear();

    let line = source_timing_readiness_line(&shell);
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(rendered, "source timing grid locked | quality high | policy locked");
    assert_eq!(line.spans[2].content.as_ref(), "grid locked");
    assert_eq!(line.spans[2].style.fg, Some(Color::Green));
    assert!(
        line.spans[2].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
}

#[test]
fn source_timing_readiness_styles_manual_confirm_as_pending() {
    let shell = sample_shell_state();

    let line = source_timing_readiness_line(&shell);
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(
        rendered,
        "source timing needs confirm | quality low | policy manual_confirm"
    );
    assert_eq!(line.spans[2].content.as_ref(), "needs confirm");
    assert_eq!(line.spans[2].style.fg, Some(Color::Yellow));
    assert!(
        line.spans[2].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
}

#[test]
fn source_timing_help_styles_missing_source_as_low_emphasis() {
    let mut shell = sample_shell_state();
    shell.app.source_graph = None;
    shell.app.refresh_view();

    let line = source_timing_help_line(&shell);
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(
        rendered,
        "Timing: unknown | clock unavailable | timing trust unknown"
    );
    assert_eq!(line.spans[2].content.as_ref(), "unknown");
    assert_eq!(line.spans[2].style.fg, Some(Color::DarkGray));
    assert!(
        !line.spans[2].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
}

#[test]
fn renders_missing_source_timing_clock_as_unavailable() {
    let mut shell = sample_shell_state();
    shell.app.source_graph = None;
    shell.app.refresh_view();

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("clock unavailable"), "{rendered}");
}
