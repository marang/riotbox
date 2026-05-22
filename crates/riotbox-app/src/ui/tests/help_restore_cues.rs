#[test]
fn renders_help_overlay_with_first_run_guidance() {
    let mut shell = first_run_shell_state();
    shell.show_help = true;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("First run"), "{rendered}");
    assert!(
        rendered.contains("Timing: needs confirm | grid manual_confirm_only | phase 0 amb | low"),
        "{rendered}"
    );
    assert!(rendered.contains("kick+bb | b32 bar8 p1"), "{rendered}");
    assert!(rendered.contains("b32 bar8 p1"), "{rendered}");
    assert!(rendered.contains("confirm grid first"), "{rendered}");
    assert!(rendered.contains("space: start transport"), "{rendered}");
    assert!(rendered.contains("f: queue one first fill"), "{rendered}");
    assert!(
        rendered.contains("2: switch to Log and watch it land"),
        "{rendered}"
    );
    assert!(
        rendered.contains("After first loop: docs/jam_recipes.md -> Recipe 2 / Recipe 5"),
        "{rendered}"
    );
}

#[test]
fn renders_help_overlay_with_locked_source_timing_guidance() {
    let mut shell = first_run_shell_state();
    let graph = shell
        .app
        .source_graph
        .as_mut()
        .expect("first-run shell should include source graph");
    graph.timing.quality = TimingQuality::High;
    graph.timing.degraded_policy = TimingDegradedPolicy::Locked;
    graph.timing.warnings.clear();
    shell.app.refresh_view();
    shell.show_help = true;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("Timing: grid locked | grid locked_grid | phase 0 | high | kick+bb"),
        "{rendered}"
    );
    assert!(rendered.contains("bar8 p1 | grid can steer moves"), "{rendered}");
    assert!(rendered.contains("steer moves"), "{rendered}");
}

#[test]
fn renders_help_overlay_with_pending_scene_jump_cue() {
    let mut shell = sample_shell_state();
    assert_eq!(
        shell.app.queue_scene_select(300),
        crate::jam_app::QueueControlResult::Enqueued
    );
    shell.show_help = true;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Scene timing"), "{rendered}");
    assert!(
        rendered.contains("launch intro: lands at next bar"),
        "{rendered}"
    );
    assert!(
        rendered.contains("Jam: read launch/restore, pulse, live/restore energy"),
        "{rendered}"
    );
    assert!(
        rendered.contains("2: confirm the landed trail on Log"),
        "{rendered}"
    );
}

#[test]
fn renders_help_overlay_with_pending_scene_restore_cue() {
    let graph = scene_regression_graph(&["drop".into(), "break".into()]);
    let mut session = sample_shell_state().app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-drop"),
        SceneId::from("scene-02-break"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-02-break"));
    session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));

    let mut shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Load,
    );
    shell.app.set_transport_playing(true);
    assert_eq!(
        shell.app.queue_scene_restore(300),
        crate::jam_app::QueueControlResult::Enqueued
    );
    shell.show_help = true;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Scene timing"), "{rendered}");
    assert!(
        rendered.contains("restore drop: lands at next bar"),
        "{rendered}"
    );
    assert!(
        rendered.contains("2: confirm the landed trail on Log"),
        "{rendered}"
    );
}

#[test]
fn renders_help_overlay_with_capture_path_cue() {
    let mut shell = sample_shell_state();
    shell.active_screen = ShellScreen::Capture;
    shell.show_help = true;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Capture path"), "{rendered}");
    assert!(
        rendered.contains("Do Next: read capture -> promote -> hit"),
        "{rendered}"
    );
    assert!(
        rendered.contains("src means source-backed; fallback is safe preview"),
        "{rendered}"
    );
    assert!(
        rendered.contains("hear src/fallback: [o] raw, [p] promote, [w] hit"),
        "{rendered}"
    );
    assert!(
        rendered.contains("2: confirm promote, hit, and audition results in Log"),
        "{rendered}"
    );
}

#[test]
fn renders_jam_shell_with_restore_readiness_cue() {
    let graph = scene_regression_graph(&["intro".into(), "drop".into()]);
    let mut session = sample_shell_state().app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-intro"),
        SceneId::from("scene-02-drop"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.scene_state.restore_scene = None;

    let mut shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Load,
    );
    shell.app.set_transport_playing(true);

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("[y] jump first"), "{rendered}");
    assert!(
        rendered.contains("[Y] restore waits for one landed"),
        "{rendered}"
    );
    assert!(rendered.contains("jump"), "{rendered}");
}

#[test]
fn renders_help_overlay_with_restore_readiness_cue() {
    let graph = scene_regression_graph(&["intro".into(), "drop".into()]);
    let mut session = sample_shell_state().app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-intro"),
        SceneId::from("scene-02-drop"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.scene_state.restore_scene = None;

    let mut shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Load,
    );
    shell.app.set_transport_playing(true);
    shell.show_help = true;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Scene restore"), "{rendered}");
    assert!(
        rendered.contains("Y waits for one landed jump"),
        "{rendered}"
    );
    assert!(
        rendered.contains("land one jump, then Y can restore the last scene"),
        "{rendered}"
    );
}

#[test]
fn renders_jam_shell_with_restore_ready_cue() {
    let graph = scene_regression_graph(&["drop".into(), "break".into()]);
    let mut session = sample_shell_state().app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-drop"),
        SceneId::from("scene-02-break"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-02-break"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-02-break"));
    session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));

    let mut shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Load,
    );
    shell.app.set_transport_playing(true);

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("[Y] restore drop now (rise)"),
        "{rendered}"
    );
    assert!(
        rendered.contains("Scene: restore drop/high ready | rise | Y brings back drop/high"),
        "{rendered}"
    );
}
