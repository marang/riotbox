#[test]
fn renders_ghost_watch_summary_and_blocker_status() {
    let mut shell = sample_shell_state();
    shell.app.session.ghost_state.mode = GhostMode::Watch;
    shell.app.session.ghost_state.suggestion_history = vec![GhostSuggestionRecord {
        proposal_id: "ghost-watch-1".into(),
        summary: "capture the source-backed hit".into(),
        accepted: false,
        rejected: false,
    }];
    shell
        .app
        .session
        .runtime_state
        .lock_state
        .locked_object_ids
        .push("ghost.main".into());
    shell.app.refresh_view();

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("blocked"));
    assert!(rendered.contains("blocked ghost.main"));
}

#[test]
fn renders_ghost_assist_decision_hint() {
    let mut shell = sample_shell_state();
    shell.app.session.ghost_state.mode = GhostMode::Assist;
    shell.app.session.ghost_state.suggestion_history = vec![GhostSuggestionRecord {
        proposal_id: "ghost-assist-1".into(),
        summary: "trigger next-bar drum fill".into(),
        accepted: false,
        rejected: false,
    }];
    shell.app.refresh_view();

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("accept/reject"), "{rendered}");
}

#[test]
fn renders_current_ghost_suggestion_controls_in_jam() {
    let mut shell = sample_shell_state();
    shell
        .app
        .set_current_ghost_suggestion(sample_ghost_fill_suggestion(GhostMode::Assist));

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("ghost: add a next-bar drum answer"), "{rendered}");
    assert!(rendered.contains("[Enter] accept  [N] reject"), "{rendered}");
}

#[test]
fn renders_ghost_assist_request_when_useful() {
    let shell = ghost_assist_ready_shell_state();

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("ghost assist: [Enter] ask"), "{rendered}");
}

#[test]
fn hides_ghost_assist_request_during_first_run() {
    let mut shell = first_run_shell_state();
    shell.app.set_transport_playing(true);

    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Start Here"), "{rendered}");
    assert!(!rendered.contains("ghost assist: [Enter] ask"), "{rendered}");
}

#[test]
fn help_explains_current_ghost_suggestion_controls() {
    let mut shell = sample_shell_state();
    shell
        .app
        .set_current_ghost_suggestion(sample_ghost_fill_suggestion(GhostMode::Assist));
    shell.show_help = true;

    let rendered = render_jam_shell_snapshot(&shell, 120, 40);

    assert!(rendered.contains("Ghost suggestion"), "{rendered}");
    assert!(
        rendered.contains("Enter: accept and queue the Ghost move"),
        "{rendered}"
    );
    assert!(
        rendered.contains("N: reject and clear the suggestion"),
        "{rendered}"
    );
}

#[test]
fn help_explains_ghost_assist_request_when_useful() {
    let mut shell = ghost_assist_ready_shell_state();
    shell.show_help = true;

    let rendered = render_jam_shell_snapshot(&shell, 120, 40);

    assert!(rendered.contains("Ghost Assist"), "{rendered}");
    assert!(
        rendered.contains("Enter: ask Ghost for the current best move"),
        "{rendered}"
    );
    assert!(rendered.contains("Enter again: queue it"), "{rendered}");
}

fn ghost_assist_ready_shell_state() -> JamShellState {
    let mut shell = first_run_shell_state();
    shell.first_run_onramp = false;
    shell.app.set_transport_playing(true);
    shell
}

fn sample_ghost_fill_suggestion(mode: GhostMode) -> GhostWatchSuggestion {
    GhostWatchSuggestion {
        proposal_id: "ghost-fill-1".into(),
        mode,
        tool_name: GhostWatchTool::SuggestMacroShift,
        summary: "add a next-bar drum answer".into(),
        rationale: "the current loop has room before the next scene move".into(),
        suggested_action: Some(GhostSuggestedAction {
            command: ActionCommand::Tr909FillNext,
            target: ActionTarget {
                scope: Some(TargetScope::LaneTr909),
                ..Default::default()
            },
            quantization: Quantization::NextBar,
            intent: "add a next-bar drum answer".into(),
        }),
        confidence: GhostSuggestionConfidence::Medium,
        safety: GhostSuggestionSafety::NeedsAssistAcceptance,
        blockers: Vec::new(),
        created_at: "2026-04-29T17:00:00Z".into(),
    }
}
