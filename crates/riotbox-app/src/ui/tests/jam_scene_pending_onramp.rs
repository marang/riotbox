#[test]
fn renders_jam_shell_with_scene_brain_summary() {
    let sample_shell = sample_shell_state();
    let mut session = sample_shell.app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-intro"),
        SceneId::from("scene-02-drop"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            session,
            sample_shell.app.source_graph.clone(),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );
    assert_eq!(
        shell.app.queue_scene_select(300),
        crate::jam_app::QueueControlResult::Enqueued
    );

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("idle @ 32.0"));
    assert!(rendered.contains("scene-01-intro"));
    assert!(rendered.contains("energy medium"));
    assert!(
        rendered.contains("source src-1 | next scene drop/high"),
        "{rendered}"
    );
    assert!(rendered.contains("scene-01-intro"));
    assert!(rendered.contains("live intro/med <> restore none"));
    assert!(rendered.contains("launch ->"), "{rendered}");
    assert!(rendered.contains("@ next bar"), "{rendered}");
    assert!(
        rendered.contains("pulse [===>] b32 | b8 | p1"),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "Scene: launch drop @ next bar | rise [===>] | 909 drive | 202 lift | 2 trail"
        ),
        "{rendered}"
    );
    assert!(rendered.contains("policy rise"), "{rendered}");
}

#[test]
fn scene_pending_line_styles_define_intent_hierarchy() {
    let sample_shell = sample_shell_state();
    let mut session = sample_shell.app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-intro"),
        SceneId::from("scene-02-drop"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            session,
            sample_shell.app.source_graph.clone(),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );
    assert_eq!(
        shell.app.queue_scene_select(300),
        crate::jam_app::QueueControlResult::Enqueued
    );

    let line = scene_pending_line(&shell);
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(
        rendered,
        "launch -> scene-02-drop @ next bar | policy rise | 909 drive | 202 lift"
    );
    assert_eq!(line.spans[0].content.as_ref(), "launch");
    assert_eq!(line.spans[0].style.fg, Some(Color::Yellow));
    assert!(
        line.spans[0].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[2].content.as_ref(), "scene-02-drop");
    assert_eq!(line.spans[2].style.fg, Some(Color::Yellow));
    assert_eq!(line.spans[4].content.as_ref(), "next bar");
    assert_eq!(line.spans[4].style.fg, Some(Color::Yellow));
    assert!(
        line.spans[4].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[6].content.as_ref(), "rise");
    assert_eq!(line.spans[6].style.fg, Some(Color::Green));
    assert!(
        line.spans[6].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[8].content.as_ref(), "drive");
    assert_eq!(line.spans[10].content.as_ref(), "lift");
}

#[test]
fn renders_jam_shell_with_pending_scene_restore_summary() {
    let graph = sample_shell_state()
        .app
        .source_graph
        .clone()
        .expect("sample shell source graph");
    let mut session = sample_shell_state().app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-drop"),
        SceneId::from("scene-02-intro"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-drop"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-drop"));
    session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-drop"));

    let mut shell = JamShellState::new(
        JamAppState::from_parts(session, Some(graph), ActionQueue::new()),
        ShellLaunchMode::Load,
    );
    shell.app.session.runtime_state.scene_state.restore_scene =
        Some(SceneId::from("scene-02-intro"));
    assert_eq!(
        shell.app.queue_scene_restore(300),
        crate::jam_app::QueueControlResult::Enqueued
    );

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("scene-01-drop"), "{rendered}");
    assert!(rendered.contains("energy medium"), "{rendered}");
    assert!(
        rendered.contains("live drop/med <> restore intro/high"),
        "{rendered}"
    );
    assert!(
        rendered.contains("restore -> scene-02-intro @ next bar"),
        "{rendered}"
    );
    assert!(rendered.contains("policy rise"), "{rendered}");
    assert!(
        rendered.contains("pulse [===>] b32 | b8 | p1"),
        "{rendered}"
    );
    assert!(
        rendered
            .contains("restore intro @ next bar | rise [===>] | 909 drive | 202 lift | 2 trail"),
        "{rendered}"
    );
}

#[test]
fn renders_log_shell_with_pending_scene_restore_summary() {
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
    assert_eq!(
        shell.app.queue_scene_restore(300),
        crate::jam_app::QueueControlResult::Enqueued
    );
    shell.active_screen = ShellScreen::Log;

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("restore scene-01-drop"), "{rendered}");
    assert!(
        rendered.contains("requested 300 | restore scene"),
        "{rendered}"
    );
    assert!(rendered.contains("scene-01-drop on next bar"), "{rendered}");
}

#[test]
fn renders_jam_shell_with_pending_mc202_role_change() {
    let mut shell = sample_shell_state();
    assert_eq!(
        shell.app.queue_mc202_role_toggle(200),
        crate::jam_app::QueueControlResult::Enqueued
    );

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("current voice leader"));
    assert!(rendered.contains("next voice follower"));
}

#[test]
fn renders_jam_shell_with_pending_mc202_follower_generation() {
    let first_run_shell = first_run_shell_state();
    let mut shell = JamShellState::new(first_run_shell.app, ShellLaunchMode::Load);
    shell.app.set_transport_playing(true);
    assert_eq!(
        shell.app.queue_mc202_generate_follower(200),
        crate::jam_app::QueueControlResult::Enqueued
    );

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("next follow"));
    assert!(
        rendered.contains("wait [=======>] next phrase"),
        "{rendered}"
    );
}

#[test]
fn renders_jam_shell_with_pending_mc202_answer_generation() {
    let mut shell = sample_shell_state();
    assert_eq!(
        shell.app.queue_mc202_generate_answer(200),
        crate::jam_app::QueueControlResult::Enqueued
    );

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("next answer"));
}

#[test]
fn renders_jam_shell_with_pending_mc202_pressure_generation() {
    let mut shell = sample_shell_state();
    assert_eq!(
        shell.app.queue_mc202_generate_pressure(200),
        crate::jam_app::QueueControlResult::Enqueued
    );

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("next pressure"), "{rendered}");
}

#[test]
fn renders_jam_shell_with_pending_mc202_instigator_generation() {
    let mut shell = sample_shell_state();
    assert_eq!(
        shell.app.queue_mc202_generate_instigator(200),
        crate::jam_app::QueueControlResult::Enqueued
    );

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("next instigate"), "{rendered}");
}

#[test]
fn renders_jam_shell_with_two_promoted_pending_actions_and_queue_summary() {
    let first_run_shell = first_run_shell_state();
    let mut shell = JamShellState::new(first_run_shell.app, ShellLaunchMode::Load);
    shell.app.queue_scene_mutation(200);
    shell.app.queue_tr909_fill(201);
    shell.app.queue_capture_bar(202);

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("next 1 user mutate"), "{rendered}");
    assert!(rendered.contains("next 2 user fill"), "{rendered}");
    assert!(rendered.contains("+1 more"), "{rendered}");
    assert!(!rendered.contains("more queued"), "{rendered}");
}

#[test]
fn quantization_countdown_cues_match_boundary_widths() {
    assert_eq!(
        quantization_countdown_cue(Quantization::NextBeat, 32, 8),
        "[>]"
    );
    assert_eq!(
        quantization_countdown_cue(Quantization::NextHalfBar, 3, 1),
        "[=>]"
    );
    assert_eq!(
        quantization_countdown_cue(Quantization::NextBar, 32, 8),
        "[===>]"
    );
    assert_eq!(
        quantization_countdown_cue(Quantization::NextPhrase, 32, 8),
        "[=======>]"
    );
}

#[test]
fn queued_timing_rail_styles_define_boundary_hierarchy() {
    let shell = sample_shell_state();
    let line = queued_timing_rail_line(&shell).expect("queued timing rail");
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(rendered, "wait [===>] next bar | b32 | bar8 | p1");
    assert_eq!(line.spans[0].content.as_ref(), "wait ");
    assert_eq!(line.spans[0].style.fg, Some(Color::DarkGray));
    assert_eq!(line.spans[1].content.as_ref(), "[===>]");
    assert_eq!(line.spans[1].style.fg, Some(Color::Yellow));
    assert!(
        line.spans[1].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[3].content.as_ref(), "next bar");
    assert_eq!(line.spans[3].style.fg, Some(Color::Yellow));
    assert!(
        line.spans[3].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[4].style.fg, Some(Color::DarkGray));
}

#[test]
fn queued_scene_timing_rail_styles_pulse_hierarchy() {
    let sample_shell = sample_shell_state();
    let mut session = sample_shell.app.session.clone();
    session.runtime_state.scene_state.scenes = vec![
        SceneId::from("scene-01-intro"),
        SceneId::from("scene-02-drop"),
    ];
    session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
    session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
    let mut shell = JamShellState::new(
        JamAppState::from_parts(
            session,
            sample_shell.app.source_graph.clone(),
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );
    assert_eq!(
        shell.app.queue_scene_select(300),
        crate::jam_app::QueueControlResult::Enqueued
    );

    let line = queued_timing_rail_line(&shell).expect("scene timing rail");
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(rendered, "pulse [===>] b32 | b8 | p1");
    assert_eq!(line.spans[0].content.as_ref(), "pulse ");
    assert_eq!(line.spans[0].style.fg, Some(Color::DarkGray));
    assert_eq!(line.spans[1].content.as_ref(), "[===>]");
    assert_eq!(line.spans[1].style.fg, Some(Color::Yellow));
    assert!(
        line.spans[1].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[2].style.fg, Some(Color::DarkGray));
}

#[test]
fn renders_jam_shell_with_first_run_onramp() {
    let shell = first_run_shell_state();
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Start Here"), "{rendered}");
    assert!(rendered.contains("1 [Space] start transport"), "{rendered}");
    assert!(
        rendered.contains("2 [f] queue one first fill"),
        "{rendered}"
    );
    assert!(
        rendered.contains("3 [2] watch Log when it lands on the next bar"),
        "{rendered}"
    );
}

#[test]
fn renders_jam_shell_with_queued_first_move_guidance() {
    let mut shell = first_run_shell_state();
    shell.app.queue_tr909_fill(200);

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Your first move is armed."), "{rendered}");
    assert!(rendered.contains("next bar"), "{rendered}");
    assert!(rendered.contains("confirm it in Log"), "{rendered}");
    assert!(rendered.contains("[c] capture"), "{rendered}");
}

#[test]
fn renders_jam_shell_with_first_result_guidance() {
    let shell = first_result_shell_state();
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(
        rendered.contains("What changed: landed user fill"),
        "{rendered}"
    );
    assert!(
        rendered.contains("What next: [c] capture it or [u] undo it if it missed."),
        "{rendered}"
    );
    assert!(
        rendered.contains("Then try one more move: [y] jump or [g] follow."),
        "{rendered}"
    );
}

#[test]
fn next_panel_promotes_timing_rail_above_landed_history() {
    let mut shell = first_result_shell_state();
    shell.app.queue_tr909_fill(240);

    let line_texts = next_panel_lines(&shell)
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>();

    assert_eq!(line_texts[0], "user tr909.fill_next @ next_bar");
    assert_eq!(line_texts[1], "scene transition idle");
    assert!(
        line_texts[2].starts_with("wait [===>] next bar"),
        "{line_texts:?}"
    );
    assert_eq!(line_texts[3], "landed user fill");
}

