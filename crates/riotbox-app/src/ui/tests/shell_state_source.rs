#[test]
fn renders_source_shell_snapshot_with_feral_scorecard() {
    let mut shell = sample_shell_state();
    shell.active_screen = ShellScreen::Source;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("[3 Source]"));
    assert!(rendered.contains("Identity"));
    assert!(rendered.contains("Timing"));
    assert!(rendered.contains("Sections"));
    assert!(rendered.contains("Candidates"));
    assert!(rendered.contains("Provenance"));
    assert!(rendered.contains("Source Graph Warnings"));
    assert!(
        rendered.contains("readiness needs confirm | 126.0 BPM | conf 0.76"),
        "{rendered}"
    );
    assert!(rendered.contains("conf 0.76"), "{rendered}");
    assert!(
        rendered.contains("beat tempo | bars 1 | phase amb p0 | phrase uncertain(0)"),
        "{rendered}"
    );
    assert!(rendered.contains("meter 4/4 | hypotheses 1 | anchors 2 | kick+backbeat"));
    assert!(rendered.contains("mode manual confirm | grid manual_confirm_only | trust low"));
    assert!(rendered.contains("action confirm grid first"));
    assert!(rendered.contains("warning ambiguous_downbeat"));
    assert!(rendered.contains("feral ready"));
    assert!(rendered.contains("break high"));
    assert!(rendered.contains("quote risk 1"));
    assert!(rendered.contains("use capture before quoting"));
    assert!(rendered.contains("decoded.wav_baseline"));
    assert!(rendered.contains("fixtures/input.wav"));
    assert!(rendered.contains("wav_baseline_only"));
}

#[test]
fn renders_source_shell_snapshot_with_grid_locked_timing_summary() {
    let mut shell = sample_shell_state();
    let graph = shell
        .app
        .source_graph
        .as_mut()
        .expect("sample shell should include source graph");
    graph.timing.quality = TimingQuality::High;
    graph.timing.degraded_policy = TimingDegradedPolicy::Locked;
    graph.timing.warnings.clear();
    shell.app.refresh_view();
    shell.active_screen = ShellScreen::Source;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("readiness grid locked | 126.0 BPM | conf 0.76"),
        "{rendered}"
    );
    assert!(
        rendered.contains("mode locked | grid locked_grid | trust high"),
        "{rendered}"
    );
    assert!(rendered.contains("action grid can steer moves"), "{rendered}");
    assert!(rendered.contains("warning none"), "{rendered}");
    assert!(
        rendered.contains("meter 4/4 | hypotheses 1 | anchors 2 | kick+backbeat"),
        "{rendered}"
    );
}

#[test]
fn renders_source_shell_snapshot_with_missing_source_timing_summary() {
    let mut shell = sample_shell_state();
    shell.app.source_graph = None;
    shell.app.refresh_view();
    shell.active_screen = ShellScreen::Source;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("readiness not available | trust unknown"),
        "{rendered}"
    );
    assert!(
        rendered.contains("mode disabled | grid unavailable | warning none"),
        "{rendered}"
    );
    assert!(rendered.contains("action timing unavailable"), "{rendered}");
    assert!(rendered.contains("no timing information available"), "{rendered}");
}

#[test]
fn renders_source_shell_snapshot_with_near_miss_feral_readiness() {
    let mut shell = sample_shell_state();
    let graph = shell
        .app
        .source_graph
        .as_mut()
        .expect("sample shell should include source graph");
    graph.relationships.retain(|relationship| {
        relationship.relation_type != RelationshipType::SupportsBreakRebuild
    });
    shell.app.refresh_view();
    shell.active_screen = ShellScreen::Source;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("feral needs support"), "{rendered}");
    assert!(rendered.contains("quote risk 1 | support 0"), "{rendered}");
    assert!(rendered.contains("hooks 1 | capture 1"), "{rendered}");
}
