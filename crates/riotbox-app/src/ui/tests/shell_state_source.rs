#[test]
fn renders_source_shell_snapshot_with_feral_scorecard() {
    let mut shell = sample_shell_state();
    shell.active_screen = ShellScreen::Source;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("[3 Source]"));
    assert!(rendered.contains("Identity"));
    assert!(rendered.contains("Timing"));
    assert!(rendered.contains("Source Map"));
    assert!(rendered.contains("Sections"));
    assert!(rendered.contains("Candidates"));
    assert!(rendered.contains("Provenance"));
    assert!(rendered.contains("Source Graph Warnings"));
    assert!(rendered.contains("ready needs confirm"), "{rendered}");
    assert!(rendered.contains("126.0 BPM"), "{rendered}");
    assert!(rendered.contains("c0.76"), "{rendered}");
    assert!(rendered.contains("beat tempo | bars 1 | phase p0 amb"), "{rendered}");
    assert!(rendered.contains("phr u0"), "{rendered}");
    assert!(rendered.contains("meter 4/4 | hyp 1 | anchors kick+bb"));
    assert!(rendered.contains("mode manual | grid manual"));
    assert!(rendered.contains("trust low"));
    assert!(rendered.contains("act confirm grid"));
    assert!(rendered.contains("warn ambiguous"));
    assert!(rendered.contains("mode time fallback | needs confirm"), "{rendered}");
    assert!(rendered.contains("now bar - | section -"), "{rendered}");
    assert!(rendered.contains("nav -"), "{rendered}");
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

    assert!(rendered.contains("ready grid locked"), "{rendered}");
    assert!(rendered.contains("126.0 BPM"), "{rendered}");
    assert!(rendered.contains("c0.76"), "{rendered}");
    assert!(
        rendered.contains("mode locked | grid locked | trust high"),
        "{rendered}"
    );
    assert!(rendered.contains("mode bar grid | grid locked"), "{rendered}");
    assert!(rendered.contains("now bar - | section -"), "{rendered}");
    assert!(rendered.contains("nav bar"), "{rendered}");
    assert!(rendered.contains("act grid steer"), "{rendered}");
    assert!(rendered.contains("warn none"), "{rendered}");
    assert!(
        rendered.contains("meter 4/4 | hyp 1 | anchors kick+bb"),
        "{rendered}"
    );
}

#[test]
fn renders_source_shell_snapshot_with_user_confirmed_grid() {
    let mut shell = sample_shell_state();
    shell.app.session.runtime_state.source_timing.confirmed_grid =
        Some(SourceTimingGridConfirmationState {
            source_id: SourceId::from("src-1"),
            hypothesis_id: Some("timing-primary".into()),
            confirmed_by_action: ActionId(8),
            confirmed_at: 1_777_777,
        });
    shell.active_screen = ShellScreen::Source;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("ready grid confirmed"), "{rendered}");
    assert!(rendered.contains("act user confirmed"), "{rendered}");
    assert!(rendered.contains("mode manual | grid manual"), "{rendered}");
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
    assert!(rendered.contains("mode disabled | grid unavailable"), "{rendered}");
    assert!(rendered.contains("warning none"), "{rendered}");
    assert!(rendered.contains("action timing unavailable"), "{rendered}");
    assert!(rendered.contains("no timing information available"), "{rendered}");
    assert!(rendered.contains("mode missing | not available"), "{rendered}");
    assert!(rendered.contains("now unavailable"), "{rendered}");
    assert!(rendered.contains("nav -"), "{rendered}");
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
