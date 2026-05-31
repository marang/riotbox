#[test]
fn p015_recipe_keeps_taste_short_and_proof_details_inspectable() {
    let mut shell = sample_shell_state();
    let perform = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        perform.contains("taste cautious | confirm grid"),
        "{perform}"
    );
    assert!(perform.contains("before scene moves"), "{perform}");
    assert!(
        perform.contains("proof none yet | audible moves"),
        "{perform}"
    );
    assert!(perform.contains("need output evidence"), "{perform}");
    assert!(!perform.contains("scene contract"), "{perform}");
    assert!(!perform.contains("proof p012/p013/replay/output yes"), "{perform}");

    shell.jam_mode = JamViewMode::Inspect;
    let inspect = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(inspect.contains("scene contract"), "{inspect}");
    assert!(
        inspect.contains("needs_timing_confirmation"),
        "{inspect}"
    );
    assert!(inspect.contains("truth product spine"), "{inspect}");
    assert!(
        inspect.contains("proof p012/p013/replay/output yes"),
        "{inspect}"
    );
}

#[test]
fn p015_recipe_names_scene_ready_taste_only_for_locked_timing() {
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

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("taste scene-ready | trusted grid"),
        "{rendered}"
    );
    assert!(rendered.contains("can steer scene moves"), "{rendered}");
    assert!(!rendered.contains("confirm grid before scene moves"), "{rendered}");
}
