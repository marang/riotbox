#[test]
fn suggested_gesture_key_tokens_use_primary_control_style() {
    let line = line_with_primary_keys("what next: [c] capture  [u] undo");
    let rendered = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(rendered, "what next: [c] capture  [u] undo");
    assert_eq!(line.spans[0].content.as_ref(), "what next: ");
    assert_eq!(line.spans[1].content.as_ref(), "[c]");
    assert_eq!(line.spans[1].style.fg, Some(Color::Cyan));
    assert!(
        line.spans[1].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
    assert_eq!(line.spans[3].content.as_ref(), "[u]");
    assert_eq!(line.spans[3].style.fg, Some(Color::Cyan));
    assert!(
        line.spans[3].style.add_modifier.contains(Modifier::BOLD),
        "{line:?}"
    );
}

#[test]
fn suggested_gesture_lines_style_start_key_token() {
    let shell = sample_shell_state();
    let lines = suggested_gesture_lines(&shell);

    assert_eq!(lines[0].spans[0].content.as_ref(), "[Space]");
    assert_eq!(lines[0].spans[0].style.fg, Some(Color::Cyan));
    assert!(
        lines[0].spans[0]
            .style
            .add_modifier
            .contains(Modifier::BOLD),
        "{:?}",
        lines[0]
    );
}

#[test]
fn suggested_gesture_lines_promote_feral_ready_moves() {
    let mut shell = sample_shell_state();
    shell.app.session.runtime_state.transport.is_playing = true;
    shell.app.session.runtime_state.scene_state.scenes.clear();
    shell.app.queue = ActionQueue::new();
    shell.app.refresh_view();
    let lines = suggested_gesture_lines(&shell);
    let rendered = lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(rendered.contains("feral ready: [j] browse  [f] fill"));
    assert!(rendered.contains("[g] follow  [a] answer"));
    assert!(rendered.contains("[c] capture if it bites"));
    assert_eq!(lines[0].spans[1].content.as_ref(), "[j]");
    assert_eq!(lines[0].spans[1].style.fg, Some(Color::Cyan));
}

#[test]
fn suggested_gesture_lines_do_not_promote_near_miss_feral_moves() {
    let mut shell = sample_shell_state();
    shell.app.session.runtime_state.transport.is_playing = true;
    shell.app.session.runtime_state.scene_state.scenes.clear();
    shell.app.queue = ActionQueue::new();
    let graph = shell
        .app
        .source_graph
        .as_mut()
        .expect("sample shell should include source graph");
    graph.relationships.retain(|relationship| {
        relationship.relation_type != RelationshipType::SupportsBreakRebuild
    });
    shell.app.refresh_view();
    let lines = suggested_gesture_lines(&shell);
    let rendered = lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(!rendered.contains("feral ready:"));
    assert!(rendered.contains("what changed:"), "{rendered}");
}
