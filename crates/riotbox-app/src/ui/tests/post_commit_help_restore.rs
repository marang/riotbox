#[test]
fn renders_jam_shell_with_post_commit_next_step_cue() {
    let first_result_shell = first_result_shell_state();
    let mut shell = JamShellState::new(first_result_shell.app, ShellLaunchMode::Load);
    shell.app.set_transport_playing(true);
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("landed user fill"), "{rendered}");
    assert!(
        rendered.contains("feral ready: [j] browse  [f] fill"),
        "{rendered}"
    );
    assert!(rendered.contains("[g] follow  [a] answer"), "{rendered}");
    assert!(rendered.contains("[c] capture if it bites"), "{rendered}");
}

#[test]
fn renders_jam_shell_with_single_scene_jump_waiting_cue() {
    let sample_shell = sample_shell_state();
    let mut session = sample_shell.app.session.clone();
    session.runtime_state.scene_state.scenes = vec![SceneId::from("scene-01-intro")];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));

    let shell = JamShellState::new(
        JamAppState::from_parts(
            session,
            sample_shell.app.source_graph.clone(),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("source src-1 | next scene waits for 2") && rendered.contains("scenes"),
        "{rendered}"
    );
    assert!(
        rendered.contains("[y] jump waits for 2 scenes"),
        "{rendered}"
    );
    assert!(
        rendered.contains("Primary: y jump waits | g follow | f fill"),
        "{rendered}"
    );

    let mut shell = shell;
    shell.show_help = true;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("space: play / pause | y: jump waits | g: follow | f: fill"),
        "{rendered}"
    );
}

#[test]
fn renders_scene_jump_post_commit_guidance() {
    let shell = scene_post_commit_shell_state(
        ActionCommand::SceneLaunch,
        "scene-02-break",
        "scene-01-drop",
    );
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("scene break/high | restore drop/med"),
        "{rendered}"
    );
    assert!(
        rendered.contains("live break/high <> restore drop/med"),
        "{rendered}"
    );
    assert!(
        rendered.contains("landed user scene jump | energy rise"),
        "{rendered}"
    );
    assert!(rendered.contains("909 lift"), "{rendered}");
    assert!(rendered.contains("next [Y]"), "{rendered}");
    assert!(rendered.contains("restore [c] capture"), "{rendered}");
    assert!(rendered.contains("[c] capture"), "{rendered}");
}

#[test]
fn scene_post_commit_cue_styles_define_performance_hierarchy() {
    let shell = scene_post_commit_shell_state(
        ActionCommand::SceneLaunch,
        "scene-02-break",
        "scene-01-drop",
    );
    let line = scene_post_commit_cue_line(&shell).expect("scene post-commit cue");
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(
        rendered,
        "scene break/high | restore drop/med | 909 lift | next [Y] restore [c] capture"
    );
    assert_eq!(line.spans[0].style.fg, Some(Color::DarkGray));
    assert_eq!(line.spans[1].content.as_ref(), "break/high");
    assert_eq!(line.spans[1].style.fg, Some(Color::Green));
    assert!(
        line.spans[1].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[3].content.as_ref(), "drop/med");
    assert_eq!(line.spans[3].style.fg, Some(Color::Yellow));
    assert_eq!(line.spans[5].content.as_ref(), "909 lift");
    assert_eq!(line.spans[5].style.fg, Some(Color::Yellow));
    assert_eq!(line.spans[7].content.as_ref(), "[Y]");
    assert_eq!(line.spans[7].style.fg, Some(Color::Cyan));
    assert!(
        line.spans[7].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[9].content.as_ref(), "[c]");
    assert_eq!(line.spans[9].style.fg, Some(Color::Cyan));
    assert!(
        line.spans[9].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
}

#[test]
fn scene_post_commit_cue_surfaces_landed_movement() {
    let mut shell = scene_post_commit_shell_state(
        ActionCommand::SceneLaunch,
        "scene-02-break",
        "scene-01-drop",
    );
    shell.app.session.runtime_state.scene_state.last_movement = Some(SceneMovementState {
        action_id: ActionId(1),
        from_scene: Some(SceneId::from("scene-01-drop")),
        to_scene: SceneId::from("scene-02-break"),
        kind: SceneMovementKindState::Launch,
        direction: SceneMovementDirectionState::Rise,
        tr909_intent: SceneMovementLaneIntentState::Drive,
        mc202_intent: SceneMovementLaneIntentState::Lift,
        intensity: 0.75,
        committed_bar_index: 9,
        committed_phrase_index: 2,
    });
    shell.app.refresh_view();

    let rendered = scene_post_commit_cue_line(&shell)
        .expect("scene post-commit cue")
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert!(
        rendered.contains("move rise 909 drive 202 lift"),
        "{rendered}"
    );
}

#[test]
fn latest_landed_line_styles_define_result_hierarchy() {
    let shell = scene_post_commit_shell_state(
        ActionCommand::SceneLaunch,
        "scene-02-break",
        "scene-01-drop",
    );
    let line = latest_landed_line(&shell);
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(rendered, "landed user scene jump | energy rise");
    assert_eq!(latest_landed_text(&shell), rendered);
    assert_eq!(line.spans[0].content.as_ref(), "landed ");
    assert_eq!(line.spans[0].style.fg, Some(Color::DarkGray));
    assert_eq!(line.spans[1].content.as_ref(), "user ");
    assert_eq!(line.spans[1].style.fg, Some(Color::DarkGray));
    assert_eq!(line.spans[2].content.as_ref(), "scene jump");
    assert_eq!(line.spans[2].style.fg, Some(Color::Green));
    assert!(
        line.spans[2].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[4].content.as_ref(), "energy rise");
    assert_eq!(line.spans[4].style.fg, Some(Color::Green));
    assert!(
        line.spans[4].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
}

#[test]
fn renders_scene_restore_post_commit_guidance() {
    let shell = scene_post_commit_shell_state(
        ActionCommand::SceneRestore,
        "scene-01-drop",
        "scene-02-break",
    );
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("scene drop/med | restore break/high"),
        "{rendered}"
    );
    assert!(
        rendered.contains("live drop/med <> restore break/high"),
        "{rendered}"
    );
    assert!(
        rendered.contains("landed user restore | energy drop"),
        "{rendered}"
    );
    assert!(rendered.contains("909 lift"), "{rendered}");
    assert!(rendered.contains("next [y]"), "{rendered}");
    assert!(rendered.contains("jump [c] capture"), "{rendered}");
    assert!(rendered.contains("[c] capture"), "{rendered}");
}

#[test]
fn omits_scene_post_commit_tr909_lift_without_scene_accent() {
    let mut shell = scene_post_commit_shell_state(
        ActionCommand::SceneRestore,
        "scene-01-drop",
        "scene-02-break",
    );
    shell
        .app
        .session
        .runtime_state
        .lane_state
        .tr909
        .reinforcement_mode = None;
    shell.app.session.runtime_state.lane_state.tr909.pattern_ref = None;
    shell.app.refresh_view();
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("scene drop/med | restore break/high | next"),
        "{rendered}"
    );
    assert!(!rendered.contains("909 lift"), "{rendered}");
}

#[test]
fn renders_help_overlay_with_first_run_guidance() {
    let mut shell = first_run_shell_state();
    shell.show_help = true;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("First run"), "{rendered}");
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
        rendered.contains("hear ... stored means [p] promote, then [w] hit"),
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

