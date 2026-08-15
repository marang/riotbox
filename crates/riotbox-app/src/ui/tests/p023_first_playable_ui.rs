use super::*;

#[test]
fn first_run_onramp_tracks_capture_promotion_monitor_and_performance_readiness() {
    let mut shell = first_run_shell_state();
    assert_eq!(
        first_run_onramp_stage(&shell),
        Some(FirstRunOnrampStage::Capture)
    );

    shell.app.queue_tr909_fill(200);
    assert_eq!(
        first_run_onramp_stage(&shell),
        Some(FirstRunOnrampStage::Capture),
        "an unrelated pending gesture must not impersonate capture progress"
    );

    let mut shell = first_run_shell_state();
    shell.app.queue_capture_bar(201);
    assert_eq!(
        first_run_onramp_stage(&shell),
        Some(FirstRunOnrampStage::CapturePending)
    );

    let mut shell = first_run_captured_shell_state(true);
    assert_eq!(
        first_run_onramp_stage(&shell),
        Some(FirstRunOnrampStage::CaptureReadiness)
    );

    let mut recapture_shell = first_run_captured_shell_state(false);
    recapture_shell.app.queue_capture_bar(219);
    assert_eq!(
        first_run_onramp_stage(&recapture_shell),
        Some(FirstRunOnrampStage::CapturePending),
        "a recapture must remain visibly pending even after an earlier capture"
    );

    assert!(shell.app.queue_promote_last_capture(220));
    assert_eq!(
        first_run_onramp_stage(&shell),
        Some(FirstRunOnrampStage::PromotionPending)
    );

    let shell = first_run_promoted_shell_state(
        riotbox_core::action::SourceMonitorMode::Source,
        SourceMonitorAudioRoute::SourceOnly,
    );
    assert_eq!(
        first_run_onramp_stage(&shell),
        Some(FirstRunOnrampStage::Monitor)
    );
    assert!(
        first_run_onramp_lines(&shell)
            .iter()
            .any(|line| line.contains("[M] choose Blend: source + Riotbox"))
    );

    let shell = first_run_promoted_shell_state(
        riotbox_core::action::SourceMonitorMode::Source,
        SourceMonitorAudioRoute::SourceUnavailable,
    );
    let unavailable_lines = first_run_onramp_lines(&shell).join("\n");
    assert_eq!(
        first_run_onramp_stage(&shell),
        Some(FirstRunOnrampStage::Monitor)
    );
    assert!(unavailable_lines.contains("Blend unavailable"));
    assert!(!unavailable_lines.contains("choose Blend"));
    assert!(!unavailable_lines.contains("hear source"));

    let mut shell = first_run_promoted_shell_state(
        riotbox_core::action::SourceMonitorMode::Blend,
        SourceMonitorAudioRoute::Blend,
    );
    assert_eq!(
        first_run_onramp_stage(&shell),
        Some(FirstRunOnrampStage::Performance)
    );

    push_first_run_gesture(&mut shell, ActionCommand::W30TriggerPad, 4);
    assert_eq!(
        first_run_onramp_stage(&shell),
        Some(FirstRunOnrampStage::Performance),
        "one landed gesture must not complete the guided instrument path"
    );
    push_first_run_gesture(&mut shell, ActionCommand::Tr909FillNext, 5);
    push_first_run_gesture(&mut shell, ActionCommand::Tr909SetSlam, 6);
    assert_eq!(
        first_run_onramp_stage(&shell),
        Some(FirstRunOnrampStage::Performance)
    );
    push_first_run_gesture(&mut shell, ActionCommand::SceneLaunch, 7);
    assert_eq!(first_run_onramp_stage(&shell), None);

    shell.app.session.runtime_state.source_monitor.mode =
        riotbox_core::action::SourceMonitorMode::Source;
    set_first_run_audio_runtime(
        &mut shell,
        Some(AudioRuntimeLifecycle::Running),
        SourceMonitorAudioRoute::SourceOnly,
    );
    shell.app.queue_capture_bar(300);
    assert_eq!(
        first_run_onramp_stage(&shell),
        None,
        "later monitor cycles and recaptures must not reopen a completed onramp"
    );
}

#[test]
fn first_run_completion_requires_all_gestures_inside_one_playable_monitor_epoch() {
    let mut shell = first_run_promoted_shell_state(
        riotbox_core::action::SourceMonitorMode::Blend,
        SourceMonitorAudioRoute::Blend,
    );

    push_first_run_monitor_mode(
        &mut shell,
        riotbox_core::action::SourceMonitorMode::Source,
        4,
    );
    set_first_run_audio_runtime(
        &mut shell,
        Some(AudioRuntimeLifecycle::Running),
        SourceMonitorAudioRoute::SourceOnly,
    );
    push_first_run_gesture(&mut shell, ActionCommand::W30TriggerPad, 5);
    push_first_run_gesture(&mut shell, ActionCommand::Tr909FillNext, 6);
    push_first_run_gesture(&mut shell, ActionCommand::Tr909SetSlam, 7);
    push_first_run_gesture(&mut shell, ActionCommand::SceneLaunch, 8);

    assert_eq!(
        first_run_onramp_stage(&shell),
        Some(FirstRunOnrampStage::Monitor),
        "gestures landed after a Source switch must not complete the audible onramp"
    );

    push_first_run_monitor_mode(
        &mut shell,
        riotbox_core::action::SourceMonitorMode::Blend,
        9,
    );
    set_first_run_audio_runtime(
        &mut shell,
        Some(AudioRuntimeLifecycle::Running),
        SourceMonitorAudioRoute::Blend,
    );
    assert_eq!(
        first_run_onramp_stage(&shell),
        Some(FirstRunOnrampStage::Performance),
        "gestures from the earlier Source epoch must not leak into the new Blend epoch"
    );

    push_first_run_gesture(&mut shell, ActionCommand::W30TriggerPad, 10);
    push_first_run_gesture(&mut shell, ActionCommand::Tr909FillNext, 11);
    push_first_run_gesture(&mut shell, ActionCommand::Tr909SetSlam, 12);
    push_first_run_gesture(&mut shell, ActionCommand::SceneLaunch, 13);
    assert_eq!(first_run_onramp_stage(&shell), None);

    push_first_run_monitor_mode(
        &mut shell,
        riotbox_core::action::SourceMonitorMode::Source,
        14,
    );
    set_first_run_audio_runtime(
        &mut shell,
        Some(AudioRuntimeLifecycle::Running),
        SourceMonitorAudioRoute::SourceOnly,
    );
    assert_eq!(
        first_run_onramp_stage(&shell),
        None,
        "a completed historical Blend epoch must remain complete after later monitor changes"
    );
}

#[test]
fn first_run_completion_does_not_hide_a_faulted_audio_runtime() {
    let mut shell = first_run_promoted_shell_state(
        riotbox_core::action::SourceMonitorMode::Blend,
        SourceMonitorAudioRoute::Blend,
    );
    push_first_run_gesture(&mut shell, ActionCommand::W30TriggerPad, 4);
    push_first_run_gesture(&mut shell, ActionCommand::Tr909FillNext, 5);
    push_first_run_gesture(&mut shell, ActionCommand::Tr909SetSlam, 6);
    push_first_run_gesture(&mut shell, ActionCommand::SceneLaunch, 7);
    set_first_run_audio_runtime(
        &mut shell,
        Some(AudioRuntimeLifecycle::Faulted),
        SourceMonitorAudioRoute::Blend,
    );

    assert_eq!(
        first_run_onramp_stage(&shell),
        Some(FirstRunOnrampStage::Monitor),
        "historical gesture commits must not hide a currently faulted audio path"
    );
}

#[test]
fn first_run_playable_requires_running_audio_and_a_compatible_typed_route() {
    let mut shell = first_run_promoted_shell_state(
        riotbox_core::action::SourceMonitorMode::Blend,
        SourceMonitorAudioRoute::Blend,
    );

    set_first_run_audio_runtime(&mut shell, None, SourceMonitorAudioRoute::Blend);
    assert_eq!(
        first_run_onramp_stage(&shell),
        Some(FirstRunOnrampStage::Monitor)
    );
    assert!(
        first_run_onramp_lines(&shell)
            .join("\n")
            .contains("not started")
    );

    set_first_run_audio_runtime(
        &mut shell,
        Some(AudioRuntimeLifecycle::Faulted),
        SourceMonitorAudioRoute::Blend,
    );
    assert_eq!(
        first_run_onramp_stage(&shell),
        Some(FirstRunOnrampStage::Monitor)
    );
    assert!(
        first_run_onramp_lines(&shell)
            .join("\n")
            .contains("faulted")
    );

    set_first_run_audio_runtime(
        &mut shell,
        Some(AudioRuntimeLifecycle::Running),
        SourceMonitorAudioRoute::RiotboxOnly,
    );
    assert_eq!(
        first_run_onramp_stage(&shell),
        Some(FirstRunOnrampStage::Monitor),
        "Blend mode must not claim a Riotbox-only route as compatible"
    );

    set_first_run_audio_runtime(
        &mut shell,
        Some(AudioRuntimeLifecycle::Running),
        SourceMonitorAudioRoute::Blend,
    );
    assert_eq!(
        first_run_onramp_stage(&shell),
        Some(FirstRunOnrampStage::Performance)
    );

    let riotbox_only = first_run_promoted_shell_state(
        riotbox_core::action::SourceMonitorMode::Riotbox,
        SourceMonitorAudioRoute::RiotboxOnly,
    );
    assert_eq!(
        first_run_onramp_stage(&riotbox_only),
        Some(FirstRunOnrampStage::Performance),
        "a running typed Riotbox-only handoff remains a playable degraded-source path"
    );
}

#[test]
fn promoted_capture_with_unavailable_audio_handoff_stays_at_capture_readiness() {
    let mut shell = first_run_promoted_shell_state(
        riotbox_core::action::SourceMonitorMode::Blend,
        SourceMonitorAudioRoute::Blend,
    );
    shell.app.session.captures[0].source_window = None;
    set_first_run_audio_runtime(
        &mut shell,
        Some(AudioRuntimeLifecycle::Running),
        SourceMonitorAudioRoute::Blend,
    );

    assert_eq!(
        first_run_onramp_stage(&shell),
        Some(FirstRunOnrampStage::CaptureReadiness)
    );
    let guidance = first_run_onramp_lines(&shell).join("\n");
    assert!(
        guidance.contains("audio handoff is unavailable"),
        "{guidance}"
    );
    assert!(!guidance.contains("Playable monitor"), "{guidance}");
}

#[test]
fn first_run_performance_names_scene_jump_degradation_explicitly() {
    let mut shell = first_run_promoted_shell_state(
        riotbox_core::action::SourceMonitorMode::Blend,
        SourceMonitorAudioRoute::Blend,
    );
    shell.app.jam_view.scene.scene_jump_availability =
        riotbox_core::view::jam::SceneJumpAvailabilityView::WaitingForMoreScenes;

    let guidance = first_run_onramp_lines(&shell).join("\n");
    assert!(guidance.contains("degraded scene set"), "{guidance}");
    assert!(guidance.contains("[y] unavailable"), "{guidance}");
    assert!(guidance.contains("all four gestures land"), "{guidance}");
}

fn push_first_run_gesture(shell: &mut JamShellState, command: ActionCommand, id: u64) {
    shell.app.session.action_log.actions.push(Action {
        id: ActionId(id),
        actor: ActorType::User,
        command,
        params: ActionParams::Empty,
        target: ActionTarget::default(),
        requested_at: 250 + id,
        quantization: Quantization::NextBeat,
        status: ActionStatus::Committed,
        committed_at: Some(260 + id),
        result: Some(ActionResult {
            accepted: true,
            summary: "first-run gesture landed".into(),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: Some("prove the complete first-run performance set".into()),
    });
}

fn push_first_run_monitor_mode(
    shell: &mut JamShellState,
    mode: riotbox_core::action::SourceMonitorMode,
    id: u64,
) {
    shell.app.session.action_log.actions.push(Action {
        id: ActionId(id),
        actor: ActorType::User,
        command: ActionCommand::SourceMonitorSetMode,
        params: ActionParams::SourceMonitor { mode: Some(mode) },
        target: ActionTarget {
            scope: Some(TargetScope::Session),
            ..Default::default()
        },
        requested_at: 250 + id,
        quantization: Quantization::Immediate,
        status: ActionStatus::Committed,
        committed_at: Some(250 + id),
        result: Some(ActionResult {
            accepted: true,
            summary: format!("monitor changed to {mode}"),
        }),
        undo_policy: UndoPolicy::Undoable,
        explanation: Some("bound the first-run monitor epoch".into()),
    });
    shell.app.session.runtime_state.source_monitor.mode = mode;
    shell.app.refresh_view();
}

#[test]
fn help_overlay_routes_close_keys_before_suppressing_performance_keys() {
    let mut shell = sample_shell_state();
    let monitor_before = shell.app.session.runtime_state.source_monitor.mode;

    assert_eq!(
        shell.handle_key_code(KeyCode::Char('?')),
        ShellKeyOutcome::Continue
    );
    assert!(shell.show_help);
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('M')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(
        shell.app.session.runtime_state.source_monitor.mode,
        monitor_before
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('w')),
        ShellKeyOutcome::Continue
    );
    assert!(shell.show_help);

    assert_eq!(
        shell.handle_key_code(KeyCode::Esc),
        ShellKeyOutcome::Continue
    );
    assert!(!shell.show_help);
    assert_eq!(shell.status_message, "help overlay closed");

    assert_eq!(
        shell.handle_key_code(KeyCode::Char('h')),
        ShellKeyOutcome::Continue
    );
    assert!(shell.show_help);
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('h')),
        ShellKeyOutcome::Continue
    );
    assert!(!shell.show_help);

    assert_eq!(
        shell.handle_key_code(KeyCode::Char('?')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('?')),
        ShellKeyOutcome::Continue
    );
    assert!(!shell.show_help);

    assert_eq!(
        shell.handle_key_code(KeyCode::Char('?')),
        ShellKeyOutcome::Continue
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('q')),
        ShellKeyOutcome::Quit
    );
}

#[test]
fn uppercase_m_cycles_monitor_while_lowercase_m_keeps_scene_mutation() {
    let mut shell = sample_shell_state();

    assert_eq!(
        shell.handle_key_code(KeyCode::Char('M')),
        ShellKeyOutcome::QueueSourceMonitorMode(riotbox_core::action::SourceMonitorMode::Blend)
    );
    assert_eq!(
        shell.status_message,
        "queue monitor blend for immediate commit"
    );
    assert_eq!(
        shell.handle_key_code(KeyCode::Char('m')),
        ShellKeyOutcome::QueueSceneMutation
    );
}

#[test]
fn monitor_title_preserves_existing_now_semantics_at_standard_width() {
    let shell = sample_shell_state();
    let rendered = render_jam_shell_snapshot(&shell, 120, 34);

    assert!(rendered.contains("Now | M src>mix/no-src"), "{rendered}");
    assert!(
        rendered.contains("idle @ 31.0 | source b- bar8 p-"),
        "{rendered}"
    );
    assert!(
        rendered.contains("source src-1 | next scene intro/med"),
        "{rendered}"
    );
    assert!(
        rendered.contains("live scene-a/med <> restore none"),
        "{rendered}"
    );
}

#[test]
fn narrow_terminal_keeps_monitor_onramp_and_help_recovery_keys_visible() {
    let shell = first_run_shell_state();
    let rendered = render_jam_shell_snapshot(&shell, 80, 24);

    assert!(rendered.contains("Now | M src>mix/no-src"), "{rendered}");
    assert!(rendered.contains("Start Here"), "{rendered}");
    assert!(rendered.contains("[c] capture"), "{rendered}");
    assert!(rendered.contains("[C] confirm grid"), "{rendered}");

    let mut help_shell = shell;
    help_shell.show_help = true;
    let help = render_jam_shell_snapshot(&help_shell, 80, 24);
    assert!(help.contains("Esc / ? / h: close help"), "{help}");
    assert!(help.contains("Primary gestures"), "{help}");
    assert!(help.contains("M: monitor source -> blend"), "{help}");
    assert!(
        help.contains("w: hit | f: fill | S: cut-hit | s: slam"),
        "{help}"
    );
    assert!(
        help.contains("y: request next scene (when ready) | Y: restore prior scene"),
        "{help}"
    );
}

#[test]
fn compact_capture_onramp_does_not_offer_grid_confirmation_when_timing_is_unavailable() {
    let mut shell = first_run_shell_state();
    let graph = shell
        .app
        .source_graph
        .as_mut()
        .expect("first-run shell should include source graph");
    graph.timing.bpm_estimate = None;
    graph.timing.bpm_confidence = 0.0;
    graph.timing.quality = TimingQuality::Unknown;
    graph.timing.degraded_policy = TimingDegradedPolicy::Disabled;
    graph.timing.primary_hypothesis_id = None;
    graph.timing.hypotheses.clear();
    graph.timing.beat_grid.clear();
    graph.timing.bar_grid.clear();
    graph.timing.phrase_grid.clear();
    shell.app.refresh_view();

    let guidance = first_run_onramp_compact_lines(&shell).join("\n");
    assert!(guidance.contains("timing unavailable"), "{guidance}");
    assert!(guidance.contains("source preview only"), "{guidance}");
    assert!(!guidance.contains("[C]"), "{guidance}");
}

#[test]
fn footer_prioritizes_controls_status_and_audio_fault_across_supported_sizes() {
    let mut shell = sample_shell_state();
    set_first_run_audio_runtime(
        &mut shell,
        Some(AudioRuntimeLifecycle::Faulted),
        SourceMonitorAudioRoute::SourceUnavailable,
    );

    let wide = render_jam_shell_snapshot(&shell, 120, 34);
    assert!(wide.contains("Keys:"), "{wide}");
    assert!(wide.contains("Primary:"), "{wide}");
    assert!(wide.contains("Status: audio faulted"), "{wide}");
    assert!(wide.contains("Warning: audio runtime faulted"), "{wide}");
    assert!(wide.contains("Advanced:"), "{wide}");
    assert!(!wide.contains("Lane ops:"), "{wide}");

    let narrow = render_jam_shell_snapshot(&shell, 80, 24);
    assert!(narrow.contains("Keys:"), "{narrow}");
    assert!(narrow.contains("Primary:"), "{narrow}");
    assert!(narrow.contains("Status: audio faulted"), "{narrow}");
    assert!(
        narrow.contains("Warning: audio runtime faulted"),
        "{narrow}"
    );
    assert!(!narrow.contains("Advanced:"), "{narrow}");
    assert!(!narrow.contains("Lane ops:"), "{narrow}");
}
