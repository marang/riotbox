#[test]
fn renders_log_shell_snapshot_with_action_trust_history() {
    let mut shell = sample_shell_state();
    shell.active_screen = ShellScreen::Log;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("[2 Log]"));
    assert!(rendered.contains("Queued / Pending"));
    assert!(rendered.contains("Accepted / Committed"));
    assert!(rendered.contains("Rejected / Undone"));
    assert!(rendered.contains("MC-202 Lane"));
    assert!(rendered.contains("W-30 Lane"));
    assert!(rendered.contains("role leader"));
    assert!(rendered.contains("cue idle"));
    assert!(rendered.contains("cue idle | none"));
    assert!(rendered.contains("recall/unavailable"), "{rendered}");
    assert!(rendered.contains("mix 0.00/0.50 idle"));
    assert!(rendered.contains("result captured"));
    assert!(rendered.contains("ghost"));
    assert!(rendered.contains("mutate.scene"));
    assert!(rendered.contains("TR-909 Render"));
    assert!(rendered.contains("accent off"));
    assert!(rendered.contains("takeover"));
    assert!(rendered.contains("scene lock blocked ghost"));
    assert!(rendered.contains("undid most recent musical"));
}

#[test]
fn renders_tr909_feral_support_reason_cue() {
    let mut shell = sample_shell_state();
    shell
        .app
        .session
        .runtime_state
        .lane_state
        .tr909
        .takeover_enabled = false;
    shell
        .app
        .session
        .runtime_state
        .lane_state
        .tr909
        .takeover_profile = None;
    shell
        .app
        .session
        .runtime_state
        .lane_state
        .tr909
        .reinforcement_mode = Some(Tr909ReinforcementModeState::SourceSupport);
    shell.app.session.runtime_state.lane_state.tr909.pattern_ref =
        Some("support-feral-break".into());
    shell.app.update_transport_clock(TransportClockState {
        is_playing: true,
        position_beats: 4.0,
        beat_index: 4,
        bar_index: 1,
        phrase_index: 1,
        current_scene: Some(SceneId::from("scene-a")),
    });
    shell.active_screen = ShellScreen::Log;

    assert_eq!(
        shell.app.runtime_view.tr909_render_support_reason,
        "feral break lift"
    );
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("feral break lift"), "{rendered}");
}

#[test]
fn renders_log_shell_snapshot_with_scene_brain_diagnostics() {
    let mut shell = sample_shell_state();
    assert_eq!(
        shell.app.queue_scene_select(300),
        crate::jam_app::QueueControlResult::Enqueued
    );
    shell.active_screen = ShellScreen::Log;
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Counts"));
    assert!(rendered.contains("scene scene-a | medium"));
    assert!(rendered.contains("restore none"));
    assert!(rendered.contains("pending"));
}
